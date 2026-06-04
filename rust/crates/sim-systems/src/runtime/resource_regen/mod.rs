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
fn regen_channel(sources: &[((u32, u32), u8)], tiles: &mut HashMap<(u32, u32), u8>, amount: u8) {
    for &(key, max) in sources {
        let cur = tiles.get(&key).copied().unwrap_or(0);
        if cur != RESOURCE_SOURCE_INFINITE && cur < max {
            tiles.insert(key, cur.saturating_add(amount).min(max));
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
        regen_channel(&food, &mut resources.food_tiles, FOOD_REGEN_AMOUNT);
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
