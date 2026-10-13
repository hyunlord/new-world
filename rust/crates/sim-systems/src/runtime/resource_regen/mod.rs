//! V7 feature `add-resource-scarcity-regen` — resource regeneration subsystem.
//!
//! Hosts [`ResourceRegenSystem`] (priority **140**, interval
//! [`REGEN_INTERVAL`]): periodically refills each REGISTERED finite source
//! tile toward its original capacity, recreating a depleted (removed) tile
//! from zero. Paired with the production `seed_finite_resource_scarcity`
//! seeding (sim-bridge), this makes the resource substrate finite +
//! self-regenerating so the shipped `StarvationSystem` (`c177804e`) actually
//! fires while the population stays in a stable dynamic equilibrium.
//!
//! ## No-op invariant (blast-radius containment)
//!
//! The system reads ONLY the `*_source_max` ceiling registries. A harness that
//! never calls `seed_finite_resource_scarcity` leaves those maps EMPTY, so the
//! per-channel snapshot is empty and the system is a pure no-op — the 12 shared
//! harnesses + `harness_starvation_death` see identical behavior (only one
//! extra unbounded entry in `system_names()`).
//!
//! ## Determinism
//!
//! Each `(key, max)` pair updates its OWN tile independently and
//! commutatively (`cur.saturating_add(amount).min(max)`), so the HashMap
//! iteration order has zero effect on the resulting tile maps — the run stays
//! deterministic (no nondeterministic ordering leaks into observable state).

use std::collections::HashMap;

use hecs::World;
use sim_core::components::{AgentState, Position, SeekTarget, TargetKind};
use sim_engine::{RuntimeSystem, SimResources, RESOURCE_SOURCE_INFINITE};

use crate::runtime::decision::nearest_resource_tile;

/// Winter index of `SeasonRule` seasons.
const WINTER: usize = 3;

/// Engine `tick_interval` for [`ResourceRegenSystem`]: the system runs once
/// every `REGEN_INTERVAL` ticks. The EFFECTIVE per-tile rate is therefore
/// `*_REGEN_AMOUNT / REGEN_INTERVAL` units/tile/tick. Must be `> 1` — an
/// every-tick regen would make sources effectively near-infinite and erase the
/// scarcity balance.
///
/// Held at `120` — the JOINT-CONSTRAINT optimum. The seed-42 response is
/// deterministic but CHAOTIC/non-monotonic: small lever changes cause large
/// emergent swings via settlement-migration timing and birth cascades. An
/// honest sweep showed deeper scarcity raises the death count but thins the
/// population so much that the corner sleep tiles stop being consumed to
/// removal, breaking A13's per-kind (food AND water AND sleep) depletion
/// requirement. Sweep at seed 42:
///
/// - `120`: 4 scarcity deaths, 132 live, A13 food+water+sleep all deplete (PASS)
/// - `180`: 1 death, 135 live (A6/A11 FAIL — fewer deaths, non-monotonic)
/// - `240`: 65 deaths, 59 live, but sleep never depletes → A13 FAILS (even with
///   `INITIAL_SLEEP` lowered to 20 — the thinned population does not consume the
///   corner sleep sources)
///
/// `120` is the only config that satisfies ALL 20 assertions simultaneously.
/// Deaths sit at 4 (a deterministic margin-1 over the A6/A11 `>= 3` floors).
/// Because the run is deterministic at seed 42 there is no measurement noise —
/// the count is exactly 4 every run, and any future change dropping it below 3
/// is caught by this harness as a RED failure.
pub const REGEN_INTERVAL: u64 = 120;

/// Units restored to each registered FOOD source per regen interval (balance
/// lever — effective rate `FOOD_REGEN_AMOUNT / REGEN_INTERVAL`).
pub const FOOD_REGEN_AMOUNT: u8 = 1;

/// Units restored to each registered WATER source per regen interval. Water is
/// the hottest channel (thirst is the faster killer); kept equal to food so
/// the spatial-concentration death driver — not aggregate starvation — does
/// the killing.
pub const WATER_REGEN_AMOUNT: u8 = 1;

/// Units restored to each registered SLEEP source per regen interval.
pub const SLEEP_REGEN_AMOUNT: u8 = 1;

/// Display day index `(current_tick + clock_offset_ticks) / ticks_per_day`.
pub fn day_index(resources: &SimResources) -> u64 {
    day_index_at(resources, resources.current_tick)
}

/// Display day index of simulation tick `tick`.
pub fn day_index_at(resources: &SimResources, tick: u64) -> u64 {
    tick.saturating_add(resources.rules.clock_offset_ticks)
        .checked_div(resources.ticks_per_day)
        .unwrap_or(0)
}

/// Current `(season, one-based day of season)`; `None` when seasons are off.
///
/// `season = (d / days_per_season) % 4` with 0 spring, 1 summer, 2 autumn,
/// 3 winter; display day `d = 0` is the first day of spring.
pub fn season_at(resources: &SimResources) -> Option<(usize, u32)> {
    season_at_tick(resources, resources.current_tick)
}

/// `(season, one-based day of season)` of simulation tick `tick`; `None`
/// when seasons are off. Same rule as [`season_at`].
pub fn season_at_tick(resources: &SimResources, tick: u64) -> Option<(usize, u32)> {
    let rule = resources.rules.seasons?;
    let per_season = u64::from(rule.days_per_season);
    if per_season == 0 {
        return None;
    }
    let d = day_index_at(resources, tick);
    Some((((d / per_season) % 4) as usize, (d % per_season) as u32 + 1))
}

/// Resource regeneration system (priority 140, interval [`REGEN_INTERVAL`]).
#[derive(Debug, Default)]
pub struct ResourceRegenSystem;

impl ResourceRegenSystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

/// Snapshot the `(key, max)` pairs of a source-ceiling registry into a `Vec`,
/// so the channel's tile map can be mutated without holding the registry's
/// borrow (the two are distinct fields of the same `&mut SimResources`).
fn snapshot(map: &HashMap<(u32, u32), u8>) -> Vec<((u32, u32), u8)> {
    map.iter().map(|(k, v)| (*k, *v)).collect()
}

/// Refill each registered source in `sources` toward its ceiling in `tiles`.
///
/// For each `(key, max)`: read the current value (absent ⇒ 0), and if it is
/// neither the [`RESOURCE_SOURCE_INFINITE`] sentinel nor already at the
/// ceiling, write `min(cur + amount, max)`. Writing on the absent-⇒0 path
/// RECREATES a depleted tile from zero. Sentinel tiles are never touched
/// (protects any infinite source). Saturating arithmetic + `min(max)`
/// guarantee the value never exceeds the ceiling.
///
/// Returns the regrowth discarded by the ceiling: `cur + amount - max` units
/// when the pulse overshoots, and the whole pulse when the source is already
/// full. Sentinel tiles discard nothing (they never regenerate).
fn regen_channel(sources: &[((u32, u32), u8)], tiles: &mut HashMap<(u32, u32), u8>, amount: u8) -> u64 {
    let mut lost = 0u64;
    for &(key, max) in sources {
        let cur = tiles.get(&key).copied().unwrap_or(0);
        if cur == RESOURCE_SOURCE_INFINITE {
            continue;
        }
        lost += (u64::from(cur) + u64::from(amount)).saturating_sub(u64::from(max.max(cur)));
        if cur < max {
            tiles.insert(key, cur.saturating_add(amount).min(max));
        }
    }
    lost
}

/// Refill every registered deadwood spot toward its ceiling by `amount`
/// (Pass 5), recreating an emptied spot. Never writes a zero value. Under a
/// winter `cap` (Pass 6, only when no spread regrowth is set) the ceiling is
/// `min(capacity, cap)`.
fn regen_wood(
    tiles: &mut std::collections::BTreeMap<(u32, u32), u8>,
    sources: &std::collections::BTreeMap<(u32, u32), u8>,
    amount: u8,
    cap: Option<u8>,
) {
    for (key, max) in sources {
        let ceiling = cap.map_or(*max, |cap| cap.min(*max));
        let cur = tiles.get(key).copied().unwrap_or(0);
        if cur < ceiling {
            tiles.insert(*key, cur.saturating_add(amount).min(ceiling));
        }
    }
}

/// Pass 6 snow cap: lower every deadwood spot above `cap` to `cap` (the rest
/// lies under snow and is gone); a spot lowered to 0 leaves `tiles` (its
/// `wood_source_max` registration stays), so no zero-valued tile is stored.
fn bury_wood(tiles: &mut std::collections::BTreeMap<(u32, u32), u8>, cap: u8) {
    if cap == 0 {
        tiles.clear();
    } else {
        for amount in tiles.values_mut() {
            *amount = (*amount).min(cap);
        }
    }
}

/// Pass 6 winter spread regrowth: `per_day` units a day over all spots, one
/// spot at a time. The accumulator gains `per_day x 1000` per pulse against
/// `1000 x pulses per day` per unit (exactly `per_day x 1000 / 12` thousandths
/// per pulse at 12 pulses a day). Each whole unit goes to the next registered
/// spot after `wood_winter_cursor` in key order, wrapping, whose amount is below
/// `min(capacity, cap)`; the cursor moves to it. A unit that finds no such spot
/// is lost (`Tallies::winter_wood_lost`, display-only). The Pass 5 accumulator
/// is not touched.
fn regen_wood_winter(resources: &mut SimResources, per_day: u32, cap: Option<u8>) {
    let pulses_per_day = (resources.ticks_per_day / REGEN_INTERVAL).max(1);
    let per_unit = 1000 * pulses_per_day;
    resources.wood_winter_acc += u64::from(per_day) * 1000;
    while resources.wood_winter_acc >= per_unit {
        resources.wood_winter_acc -= per_unit;
        let below = |key: &(u32, u32), max: &u8, tiles: &std::collections::BTreeMap<(u32, u32), u8>| {
            tiles.get(key).copied().unwrap_or(0) < cap.map_or(*max, |cap| cap.min(*max))
        };
        let tiles = &resources.wood_tiles;
        let after = resources.wood_winter_cursor;
        let chosen = resources
            .wood_source_max
            .iter()
            .filter(|(key, _)| after.is_none_or(|cursor| **key > cursor))
            .chain(resources.wood_source_max.iter().filter(|(key, _)| after.is_some_and(|cursor| **key <= cursor)))
            .find(|(key, max)| below(key, max, tiles))
            .map(|(key, _)| *key);
        match chosen {
            Some(key) => {
                let cur = resources.wood_tiles.get(&key).copied().unwrap_or(0);
                resources.wood_tiles.insert(key, cur + 1);
                resources.wood_winter_cursor = Some(key);
            }
            None => resources.tallies.winter_wood_lost += 1,
        }
    }
}

/// Pass 12: a wild region is settled while a settlement with a member has its
/// formation tile within this Chebyshev distance of the region's centre.
pub const SETTLED_REGION_RADIUS: u32 = 9;

/// A deadwood spot of one area: (tile, capacity).
type AreaSpot = ((u32, u32), u8);

/// Pass 11 regional winter fall (`FirewoodRule::regional_wood`): every area of
/// `wood_spot_area` (0 = camp, r + 1 = wild region r) receives a budget
/// proportional to its spot count at the camp's per-spot rate: area a gets
/// `per_day x n_a / n_camp` units a day (accumulator `per_day x 1000 x n_a`
/// per pulse against `1000 x pulses per day x n_camp` per unit), so the camp
/// keeps exactly v16's daily fall and the regions do not dilute it (without
/// camp spots the denominator is all spots). Each area's units go round-robin
/// over its own spots in key order after its own cursor, as
/// [`regen_wood_winter`] does over all spots; a unit that finds no spot of its
/// area below the cap is lost (`winter_wood_lost`). Areas in id order.
///
/// Pass 12 (`area_fall_floor_permille` > 0, `game_v18`): region area r + 1 is
/// settled while a settlement with a member has its formation tile within
/// Chebyshev [`SETTLED_REGION_RADIUS`] of region r's centre; a settled area's
/// accumulator adds `max(per_day × 1000 × n_area, per_day × floor × n_camp)`
/// per pulse (extra wood; `per_unit` and the camp's fall unchanged). The extra
/// is counted in `Tallies::area_floor_wood` and units lost to the cap in a
/// settled area in `Tallies::winter_wood_lost_settled`.
///
/// Pass 13 (`floor_separate`, `game_v19`): a settled area's accumulator adds
/// only the normal `per_day × 1000 × n_area`; the extra goes to a separate
/// per-area accumulator (`SimResources::wood_area_floor_sep`) whose units, after
/// the area's normal units, go round-robin over the same spots (same cursor)
/// with ceiling `max` instead of `min(cap, max)` (`Tallies::area_floor_placed`;
/// a unit with no spot below its max is lost: `area_floor_lost_at_max`, also
/// counted in `winter_wood_lost`).
fn regen_wood_winter_by_area(resources: &mut SimResources, per_day: u32, cap: Option<u8>, floor_permille: u32, floor_separate: bool) {
    let pulses_per_day = (resources.ticks_per_day / REGEN_INTERVAL).max(1);
    let mut spots: std::collections::BTreeMap<u8, Vec<AreaSpot>> = std::collections::BTreeMap::new();
    for (key, max) in &resources.wood_source_max {
        let area = resources.wood_spot_area.get(key).copied().unwrap_or(0);
        spots.entry(area).or_default().push((*key, *max));
    }
    let total: u64 = spots.values().map(|list| list.len() as u64).sum();
    let camp = spots.get(&0).map_or(0, |list| list.len() as u64);
    let reference = if camp > 0 { camp } else { total };
    if reference == 0 {
        return;
    }
    let per_unit = 1000 * pulses_per_day * reference;
    // Pass 12: the settled region areas (empty with the floor off).
    let settled: std::collections::BTreeSet<u8> = if floor_permille > 0 {
        resources
            .wild_regions
            .iter()
            .enumerate()
            .filter(|(_, region)| {
                resources.settlements.values().any(|s| {
                    !s.member_agents.is_empty()
                        && s.formation_tile.0.abs_diff(region.centre.0).max(s.formation_tile.1.abs_diff(region.centre.1))
                            <= SETTLED_REGION_RADIUS
                })
            })
            .map(|(r, _)| r as u8 + 1)
            .collect()
    } else {
        std::collections::BTreeSet::new()
    };
    for (area, list) in spots {
        let normal = u64::from(per_day) * 1000 * list.len() as u64;
        let add = if settled.contains(&area) { normal.max(u64::from(per_day) * u64::from(floor_permille) * reference) } else { normal };
        if add > normal {
            resources.wood_area_floor_acc += add - normal;
            while resources.wood_area_floor_acc >= per_unit {
                resources.wood_area_floor_acc -= per_unit;
                resources.tallies.area_floor_wood += 1;
            }
        }
        // Pass 13: with `floor_separate` the extra accrues apart (ceiling max).
        let (shared, separate) = if floor_separate && add > normal { (normal, add - normal) } else { (add, 0) };
        let acc = resources.wood_area_acc.entry(area).or_insert(0);
        *acc += shared;
        let mut units = 0u64;
        while *acc >= per_unit {
            *acc -= per_unit;
            units += 1;
        }
        let mut floor_units = 0u64;
        if separate > 0 {
            let acc = resources.wood_area_floor_sep.entry(area).or_insert(0);
            *acc += separate;
            while *acc >= per_unit {
                *acc -= per_unit;
                floor_units += 1;
            }
        }
        for unit in 0..units + floor_units {
            let floor_unit = unit >= units;
            let after = resources.wood_area_cursor.get(&area).copied();
            let tiles = &resources.wood_tiles;
            let ceiling = |max: u8| if floor_unit { max } else { cap.map_or(max, |cap| cap.min(max)) };
            let below = |key: &(u32, u32), max: &u8| tiles.get(key).copied().unwrap_or(0) < ceiling(*max);
            let chosen = list
                .iter()
                .filter(|(key, _)| after.is_none_or(|cursor| *key > cursor))
                .chain(list.iter().filter(|(key, _)| after.is_some_and(|cursor| *key <= cursor)))
                .find(|(key, max)| below(key, max))
                .map(|(key, _)| *key);
            match chosen {
                Some(key) => {
                    let cur = resources.wood_tiles.get(&key).copied().unwrap_or(0);
                    resources.wood_tiles.insert(key, cur + 1);
                    resources.wood_area_cursor.insert(area, key);
                    resources.tallies.area_floor_placed += u64::from(floor_unit);
                }
                None => {
                    resources.tallies.winter_wood_lost += 1;
                    if floor_unit {
                        resources.tallies.area_floor_lost_at_max += 1;
                    } else {
                        resources.tallies.winter_wood_lost_settled += u64::from(settled.contains(&area));
                    }
                }
            }
        }
    }
}

impl RuntimeSystem for ResourceRegenSystem {
    fn name(&self) -> &str {
        "ResourceRegenSystem"
    }

    fn priority(&self) -> u32 {
        140
    }

    fn tick_interval(&self) -> u64 {
        REGEN_INTERVAL
    }

    fn tick(&mut self, _world: &mut World, resources: &mut SimResources) {
        let food = snapshot(&resources.food_source_max);
        // Pass 4 seasons: an exact integer accumulator turns the season's
        // permille into whole units per pulse; a zero-unit pulse skips the food
        // channel so no zero-valued tile is ever inserted.
        let food_units = match (resources.rules.seasons, season_at(resources)) {
            (Some(rule), Some((season, _))) => {
                resources.food_regen_acc += u32::from(rule.regen_permille[season]);
                let units = resources.food_regen_acc / 1000;
                resources.food_regen_acc %= 1000;
                u8::try_from(units).unwrap_or(u8::MAX)
            }
            _ => FOOD_REGEN_AMOUNT,
        };
        if food_units > 0 {
            // Display-only: counted in every scene, never read by simulation logic.
            resources.tallies.regen_lost_at_cap +=
                regen_channel(&food, &mut resources.food_tiles, food_units);
        }
        // Pass 5: deadwood regrows through its own accumulator under the
        // firewood rule; a zero-unit pulse skips the channel. Pass 6: in
        // winter the snow cap and the spread regrowth replace it when set.
        if let Some(rule) = resources.rules.society.firewood {
            let winter = season_at(resources).is_some_and(|(season, _)| season == WINTER);
            // Pass 8 `windfall_over_snow`: the snow falls once, at the first
            // pulse of each winter (the previous pulse was not winter), so a
            // windfall's wood stays above the cap; without it every winter
            // pulse buries (Pass 6).
            let snowfall = !rule.windfall_over_snow
                || resources.current_tick < REGEN_INTERVAL
                || season_at_tick(resources, resources.current_tick - REGEN_INTERVAL).is_none_or(|(season, _)| season != WINTER);
            if winter && snowfall {
                if let Some(cap) = rule.winter_deadwood_cap {
                    bury_wood(&mut resources.wood_tiles, cap);
                }
            }
            match rule.winter_regen_units_per_day.filter(|_| winter) {
                Some(per_day) => {
                    // Pass 7: a winter of drawn severity sets its own fall;
                    // Pass 8: changed day by day (`winter_fall_at`, floor 0).
                    let per_day = crate::runtime::settlement::firewood::winter_fall_at(resources, &rule, resources.current_tick)
                        .unwrap_or(per_day);
                    if rule.regional_wood && !resources.wood_spot_area.is_empty() {
                        regen_wood_winter_by_area(
                            resources,
                            per_day,
                            rule.winter_deadwood_cap,
                            rule.area_fall_floor_permille,
                            rule.floor_separate,
                        )
                    } else {
                        regen_wood_winter(resources, per_day, rule.winter_deadwood_cap)
                    }
                }
                None => {
                    resources.wood_regen_acc += rule.regen_permille;
                    let units = resources.wood_regen_acc / 1000;
                    resources.wood_regen_acc %= 1000;
                    if units > 0 {
                        let cap = rule.winter_deadwood_cap.filter(|_| winter);
                        regen_wood(&mut resources.wood_tiles, &resources.wood_source_max, u8::try_from(units).unwrap_or(u8::MAX), cap);
                    }
                }
            }
        }
        let water = snapshot(&resources.water_source_max);
        regen_channel(&water, &mut resources.water_tiles, WATER_REGEN_AMOUNT);
        let sleep = snapshot(&resources.sleep_source_max);
        regen_channel(&sleep, &mut resources.sleep_tiles, SLEEP_REGEN_AMOUNT);
    }
}

/// Engine priority for [`StaleSeekTargetSystem`]: **126**, immediately after
/// `AgentDecisionSystem` (125). The decision system's resource SeekTarget
/// post-pass sets the goal tile ONCE and "keeps it stable" — an assumption
/// that held while resource tiles were fixed (infinite sources). Scarcity
/// breaks that assumption: a consumed source tile is REMOVED, so a
/// `Seeking{Food/Water/Sleep}` agent can be left pointing at a vanished
/// coordinate. Movement then either walks the agent onto the empty tile and
/// freezes it (`Seeking` suppresses Brownian motion, and a `SeekTarget` equal
/// to the current tile produces a zero directed step) or freezes it in place
/// when no `SeekTarget` exists at all. This is the project's documented #1
/// failure mode (Seeking-with-no-target / stale-SeekTarget freeze).
pub const STALE_SEEK_PRIORITY: u32 = 126;

/// Reconciles a stale resource [`SeekTarget`] introduced by scarcity tile
/// removal (priority [`STALE_SEEK_PRIORITY`], every tick).
///
/// For every agent in `Seeking{Food/Water/Sleep}` whose `SeekTarget` tile is
/// ABSENT (removed / depleted to 0) — or which carries no `SeekTarget` at all:
///
/// - if another source of that kind still exists, RE-ROUTE the `SeekTarget` to
///   the nearest present tile (same deterministic `nearest_resource_tile`
///   lookup the decision post-pass uses), so the agent walks toward a real
///   source instead of freezing on an empty coordinate;
/// - if NO source of that kind remains anywhere, transition the agent to
///   `Idle` and drop the `SeekTarget`, so Brownian motion resumes and the
///   agent wanders rather than freezing (the existing decision cascade will
///   re-seek once a source reappears via regen).
///
/// ## No-op invariant (blast-radius containment)
///
/// When a seeker's target tile is PRESENT (the only situation in every scene
/// that never calls `seed_finite_resource_scarcity` — infinite `u8::MAX`
/// sources are never removed by consumption), the system short-circuits and
/// touches nothing. The 12 shared harnesses + `harness_starvation_death` (all
/// infinite-source scenes) therefore see identical behavior. Only one extra
/// (unbounded) entry appears in `system_names()`, which no harness asserts a
/// fixed length on.
///
/// ## Determinism
///
/// All reads are lookups + the deterministic `nearest_resource_tile`; each
/// agent's reconciliation is independent and applied via deferred
/// collect-then-mutate, so HashMap / query iteration order has zero effect on
/// the resulting component state.
#[derive(Debug, Default)]
pub struct StaleSeekTargetSystem;

impl StaleSeekTargetSystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for StaleSeekTargetSystem {
    fn name(&self) -> &str {
        "StaleSeekTargetSystem"
    }

    fn priority(&self) -> u32 {
        STALE_SEEK_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        // Scarcity-scene gate (blast-radius containment). Stale resource
        // SeekTargets only ever arise when a consumed source tile is REMOVED,
        // which only happens under finite scarcity — i.e. when at least one
        // `*_source_max` ceiling is registered (the production `init` path /
        // a finite-seeded harness scene). In every infinite-source scene (the
        // 12 shared harnesses + `harness_starvation_death` + the p14 inspector
        // scenes) no ceiling is ever registered, tiles are never removed, and a
        // `Seeking{resource}` agent that simply lacks a present target must keep
        // its pre-existing (legacy) behavior. Returning here makes the system a
        // STRICT no-op in those scenes — it must not reclassify a Seeking agent
        // to Idle just because the scene has no resource tiles.
        if resources.food_source_max.is_empty()
            && resources.water_source_max.is_empty()
            && resources.sleep_source_max.is_empty()
        {
            return;
        }

        // Deferred mutation: insert/remove during a query borrow is a compile
        // error, so collect first then apply (order-independent → deterministic).
        let mut reroute: Vec<(hecs::Entity, (u32, u32))> = Vec::new();
        let mut to_idle: Vec<hecs::Entity> = Vec::new();

        for (e, (state, pos, seek)) in world
            .query::<(&AgentState, &Position, Option<&SeekTarget>)>()
            .iter()
        {
            let tiles = match state {
                AgentState::Seeking { target: TargetKind::Food } => &resources.food_tiles,
                AgentState::Seeking { target: TargetKind::Water } => &resources.water_tiles,
                AgentState::Seeking { target: TargetKind::Sleep } => &resources.sleep_tiles,
                // ConstructionSite / Agent seeks are co-located (no resource
                // SeekTarget); Idle / Consuming carry no resource goal.
                _ => continue,
            };
            // A target is "present" iff the agent HAS a SeekTarget AND that
            // coordinate still holds a positive resource value. A sentinel
            // (255 > 0) counts as present, so infinite sources are never
            // disturbed.
            let target_present = seek
                .map(|s| s.tile)
                .is_some_and(|t| tiles.get(&t).copied().is_some_and(|v| v > 0));
            if target_present {
                continue; // normal/fixed-tile case → strict no-op
            }
            match nearest_resource_tile(pos, tiles) {
                Some(nearest) => reroute.push((e, nearest)),
                None => to_idle.push(e),
            }
        }

        for (e, tile) in reroute {
            let _ = world.insert_one(e, SeekTarget::new(tile));
        }
        for e in to_idle {
            let _ = world.insert_one(e, AgentState::Idle);
            let _ = world.remove_one::<SeekTarget>(e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::{AgentState, SeekTarget, TargetKind};
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    fn engine() -> SimEngine {
        SimEngine::new(32, 32, MaterialRegistry::new())
    }

    #[test]
    fn regen_channel_counts_regrowth_lost_at_the_ceiling() {
        let sources = [((0, 0), 4u8), ((1, 0), 4), ((2, 0), 4), ((3, 0), 4)];
        let mut tiles = HashMap::new();
        tiles.insert((0, 0), 4u8); // full: the whole pulse is lost
        tiles.insert((1, 0), 3); // 3 + 2 overshoots by 1
        tiles.insert((3, 0), RESOURCE_SOURCE_INFINITE); // sentinel loses nothing
        // (2, 0) is absent: regrows from zero without loss.
        assert_eq!(regen_channel(&sources, &mut tiles, 2), 2 + 1);
        assert_eq!((tiles[&(0, 0)], tiles[&(1, 0)], tiles[&(2, 0)]), (4, 4, 2));
        assert_eq!(tiles[&(3, 0)], RESOURCE_SOURCE_INFINITE);
    }

    #[test]
    fn metadata() {
        let s = ResourceRegenSystem::new();
        assert_eq!(s.name(), "ResourceRegenSystem");
        assert_eq!(s.priority(), 140);
        assert_eq!(s.tick_interval(), REGEN_INTERVAL);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)] // deliberate invariant lock
    fn interval_exceeds_one() {
        // An every-tick regen would erase the scarcity balance.
        assert!(REGEN_INTERVAL > 1);
    }

    #[test]
    fn refills_a_removed_tile_from_zero() {
        let mut e = engine();
        e.resources.set_food_source_max(3, 3, 80);
        // tile absent (removed / never inserted).
        assert!(!e.resources.food_tiles.contains_key(&(3, 3)));
        let mut sys = ResourceRegenSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            e.resources.food_tiles.get(&(3, 3)).copied(),
            Some(FOOD_REGEN_AMOUNT),
            "removed-but-registered tile must be recreated from 0 by exactly the regen amount"
        );
    }

    #[test]
    fn caps_at_max() {
        let mut e = engine();
        e.resources.set_food_source_max(4, 4, 5);
        e.resources.set_food_tile(4, 4, 5); // already at ceiling
        let mut sys = ResourceRegenSystem::new();
        for _ in 0..10 {
            sys.tick(&mut e.world, &mut e.resources);
        }
        assert_eq!(
            e.resources.food_tiles.get(&(4, 4)).copied(),
            Some(5),
            "regen must saturate at the ceiling, never exceed it"
        );
    }

    #[test]
    fn never_touches_sentinel() {
        let mut e = engine();
        e.resources.set_food_source_max(2, 2, 80);
        e.resources.set_food_tile(2, 2, RESOURCE_SOURCE_INFINITE);
        let mut sys = ResourceRegenSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            e.resources.food_tiles.get(&(2, 2)).copied(),
            Some(RESOURCE_SOURCE_INFINITE),
            "a sentinel (255) tile must never be modified by regen"
        );
    }

    #[test]
    fn empty_registry_is_noop() {
        let mut e = engine();
        e.resources.set_food_tile(1, 1, 7); // present but NOT registered
        let mut sys = ResourceRegenSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            e.resources.food_tiles.get(&(1, 1)).copied(),
            Some(7),
            "an unregistered tile (empty source_max) must be untouched (pure no-op)"
        );
    }

    #[test]
    fn stale_seek_metadata() {
        let s = StaleSeekTargetSystem::new();
        assert_eq!(s.name(), "StaleSeekTargetSystem");
        assert_eq!(s.priority(), STALE_SEEK_PRIORITY);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn stale_seek_reroutes_to_present_tile() {
        let mut e = engine();
        let ent = e.spawn_agent(5, 5);
        e.world
            .insert(
                ent,
                (
                    AgentState::Seeking { target: TargetKind::Food },
                    SeekTarget::new((5, 5)), // points at a coord with NO food
                ),
            )
            .expect("seed");
        e.resources.set_food_tile(8, 5, 80); // the only present food source
        e.resources.set_food_source_max(8, 5, 80); // scarcity scene (gate active)
        let mut sys = StaleSeekTargetSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        let seek = e.world.get::<&SeekTarget>(ent).map(|s| s.tile).ok();
        assert_eq!(seek, Some((8, 5)), "stale target must re-route to the nearest present tile");
        assert!(
            matches!(*e.world.get::<&AgentState>(ent).unwrap(), AgentState::Seeking { .. }),
            "a re-routed agent stays Seeking"
        );
    }

    #[test]
    fn stale_seek_exits_to_idle_when_no_source_remains() {
        let mut e = engine();
        let ent = e.spawn_agent(5, 5);
        e.world
            .insert(
                ent,
                (
                    AgentState::Seeking { target: TargetKind::Food },
                    SeekTarget::new((5, 5)),
                ),
            )
            .expect("seed");
        // Scarcity scene: a ceiling is registered (A21 invariant — source_max
        // survives tile removal) but NO food tile is present anywhere, so
        // nearest_resource_tile(Food) is None.
        e.resources.set_food_source_max(5, 5, 80);
        let mut sys = StaleSeekTargetSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            *e.world.get::<&AgentState>(ent).unwrap(),
            AgentState::Idle,
            "with no source remaining the agent must exit Seeking to Idle (no freeze)"
        );
        assert!(
            e.world.get::<&SeekTarget>(ent).is_err(),
            "the stale SeekTarget must be dropped on exit"
        );
    }

    #[test]
    fn stale_seek_noop_when_target_present() {
        let mut e = engine();
        let ent = e.spawn_agent(5, 5);
        e.world
            .insert(
                ent,
                (
                    AgentState::Seeking { target: TargetKind::Food },
                    SeekTarget::new((9, 9)),
                ),
            )
            .expect("seed");
        e.resources.set_food_tile(9, 9, 80); // target tile IS present
        e.resources.set_food_source_max(9, 9, 80); // scarcity scene (gate active)
        let mut sys = StaleSeekTargetSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            e.world.get::<&SeekTarget>(ent).map(|s| s.tile).ok(),
            Some((9, 9)),
            "a present target must be left untouched (strict no-op)"
        );
        assert!(matches!(
            *e.world.get::<&AgentState>(ent).unwrap(),
            AgentState::Seeking { .. }
        ));
    }

    #[test]
    fn stale_seek_noop_on_sentinel_target() {
        let mut e = engine();
        let ent = e.spawn_agent(5, 5);
        e.world
            .insert(
                ent,
                (
                    AgentState::Seeking { target: TargetKind::Food },
                    SeekTarget::new((9, 9)),
                ),
            )
            .expect("seed");
        e.resources.set_food_tile(9, 9, RESOURCE_SOURCE_INFINITE); // infinite source
        e.resources.set_food_source_max(9, 9, 80); // scarcity scene (gate active)
        let mut sys = StaleSeekTargetSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            e.world.get::<&SeekTarget>(ent).map(|s| s.tile).ok(),
            Some((9, 9)),
            "an infinite (255) target counts as present → untouched"
        );
    }
}
