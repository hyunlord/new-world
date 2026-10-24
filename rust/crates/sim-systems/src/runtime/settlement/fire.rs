//! FireSystem: burns stockpile wood in each hearth at night and marks members `Cold` (with exposure damage) when the fire is out on a winter night.
//!
//! Priority: 147 · Interval: 60 ticks · Gate: `WorldRules.society.firewood`
//! Reads: `Agent`, `Founding`, `PorterTrip`, `Child`, `Elder`, `Hardiness`, `Hunger`, `Position`, `AgentState`, `Apparel`, `Lineage` (M5); `SimResources.settlements`, `rules`, `world_seed`, `colony.rooms` (M3), `colony.story` (M5)
//! Writes: `Cold`, `ExposureClock`, `BodyHealth`; `SimResources.fires`, `settlements` (stockpile wood), `tallies`, `colony.story` (snap cost, touched), `causal_log` (M5); deaths via `despawn_agent`
//! Events: tally `cold_member_hours`; on death `CausalEvent::AgentDied`, chronicle `Died`/`Widowed`, tally `deaths_exposure`
//!
//! Revitalize Pass 5 — `FireSystem` (priority 147, every 60 ticks).
//!
//! The hearth burns wood from the settlement stockpile through the night and
//! marks members `Cold` on winter nights when it is out. Under
//! `SocietyRules::firewood` only; without the rule the system returns at once,
//! so legacy, v8, v9 and v10 worlds are untouched.
//!
//! Each run (hourly, because the 480-tick clock offset is a multiple of 60):
//! - inside the burn window every settlement, in id order, withdraws
//!   `burn_per_hour[season]` wood; the fire is lit when the whole hour's wood
//!   was there. Outside the window every fire is out.
//! - with seasons on, in winter and inside the cold window, every member of a
//!   settlement whose fire is out gets the `Cold` marker; every other agent
//!   loses it. Markers are inserted and removed in AgentId order so archetype
//!   layout (and with it query order) stays seed-independent.
//! - `Tallies::cold_member_hours` grows by the number of marked members.
//!
//! Pass 6: under `night_season_from_evening` the burn rate and the cold check
//! use the season of the evening the night began (`firewood::night_season`),
//! so the last winter night stays a winter night until the burn window ends.
//!
//! Pass 7: in a winter of drawn severity the hourly rate is
//! `winter_severity_burn_per_hour[level]` of the night's winter
//! (`firewood::winter_burn_per_hour` at the evening's tick). (The V7 god-hand
//! rain and cold-snap interventions acted here; V8-0 removed them.)
//!
//! Pass 8: the hourly rate is `firewood::night_rate`, which adds the night's
//! drawn change (`night_burn_delta` of the evening, floor 1) when night weights
//! are set; the probe and the recorder read the same helper.
//!
//! Pass 10 (`SocietyRules::exposure`): after the markers are set, every agent
//! marked `Cold` this hour, in AgentId order, loses
//! `damage_per_cold_hour[stage]` hp (child, adult, elder) and records this run's
//! tick in its `ExposureClock`; an agent whose hp reaches 0 dies of exposure
//! through `despawn_agent`, deaths booked in AgentId order.
//!
//! Pass 11 (exposure v2, `game_v17`): the hourly damage is scaled by the
//! resident's `Hardiness` and by its hunger (`exposure_damage`).
//!
//! M3 (ADR 0023, colony with built space): a sleeper (`Consuming { Sleep }`)
//! on a tile of a roofed room is sheltered and is not marked `Cold`.
//!
//! M5 (ADR 0027): in a cold snap the night's burn is ×`snap_burn_permille`
//! (the extra wood is the snap's cost), roofed sleepers are no longer sheltered
//! when the fire is out (only a roof with the hearth lit keeps them warm), a
//! cold person outside a roofed room takes ×`snap_exposure_permille` damage, and
//! each cold person gets one `PressureCause` event per snap. A worn hide wrap or
//! fur cloak spares its wearer a cold hour with chance 1,000 − its
//! `Top::cold_permille` (one keyed roll an hour, `STREAM_APPAREL`; ADR 0029).
//!
//! (The rain rule acted only on intervention rain, removed in V8-0 with its
//! `rain_wet_burn` field; `rain_soak_permille` remains in the rules data.)

use std::collections::{BTreeMap, BTreeSet};

use hecs::World;
use sim_core::causal::event::DeathReason;
use sim_core::components::{
    Agent, AgentId, BodyHealth, Child, Cold, Elder, ExposureClock, Founding, Hardiness, Hunger,
    Lineage, PorterTrip, Position, ResourceKind, SettlementId,
};
use sim_core::rng::{keyed_below, STREAM_APPAREL};
use sim_core::rules::ExposureRule;
use sim_engine::{FireState, RuntimeSystem, SimResources};

use crate::runtime::decision::clock_in_window;
use crate::runtime::settlement::firewood::{night_rate, night_season};
use crate::runtime::survival::despawn_agent;

/// Winter index of `SeasonRule` seasons.
const WINTER: usize = 3;

/// Pass 5 hearth fire and cold-night system.
#[derive(Debug, Default)]
pub struct FireSystem;

impl FireSystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

/// Apply the authoritative active-pressure fuel multiplier with the fire's rounding.
pub fn effective_burn_per_hour(resources: &SimResources, per_hour: u32) -> u32 {
    crate::runtime::story::active(resources, sim_core::colony::story::PressureKind::ColdSnap)
        .map_or(per_hour, |s| {
            (u64::from(per_hour) * u64::from(s.snap_burn_permille)).div_ceil(1000) as u32
        })
}

impl RuntimeSystem for FireSystem {
    fn name(&self) -> &str {
        "FireSystem"
    }

    fn priority(&self) -> u32 {
        147
    }

    fn tick_interval(&self) -> u64 {
        60
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = resources.rules.society.firewood else {
            return;
        };
        let burning = clock_in_window(resources, rule.burn_start_minute, rule.burn_end_minute);
        // The night's season: the current season, or (Pass 6) the evening's.
        let season = night_season(resources, &rule);
        // Pass 7: the night's rate, the severity of its winter included;
        // Pass 8: the night's drawn change too (`night_rate`).
        let per_hour = night_rate(resources, &rule, resources.current_tick);
        // M5 (ADR 0027): a cold snap doubles the night's burn.
        let snap = crate::runtime::story::active(
            resources,
            sim_core::colony::story::PressureKind::ColdSnap,
        );
        let normal = per_hour;
        let per_hour = effective_burn_per_hour(resources, per_hour);
        let now = resources.current_tick;
        let mut ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
        ids.sort_unstable();
        let mut fires = BTreeMap::new();
        for sid in &ids {
            let lit = burning
                && resources
                    .settlements
                    .get_mut(sid)
                    .is_some_and(|settlement| {
                        settlement.withdraw(ResourceKind::Wood, per_hour) == per_hour
                    });
            fires.insert(*sid, FireState { lit });
            if lit && snap.is_some() {
                crate::runtime::story::add_cost(
                    resources,
                    sim_core::colony::story::PressureKind::ColdSnap,
                    per_hour.saturating_sub(normal),
                    0,
                );
            }
        }
        resources.fires = fires;

        let cold_night = resources.rules.seasons.is_some()
            && season == WINTER
            && clock_in_window(resources, rule.cold_start_minute, rule.cold_end_minute);
        let mut cold: BTreeSet<AgentId> = BTreeSet::new();
        if cold_night {
            for sid in &ids {
                let out = resources.fires.get(sid).is_none_or(|fire| !fire.lit);
                if let (true, Some(settlement)) = (out, resources.settlements.get(sid)) {
                    cold.extend(settlement.member_agents.iter().copied());
                }
            }
            // Pass 11: a founder on its way is not at its parent's fire
            // (no `Founding` marker exists outside `game_v17`).
            if resources.rules.society.fission.is_some() {
                for (_, (agent, _)) in world.query::<(&Agent, &Founding)>().iter() {
                    cold.remove(&agent.id);
                }
            }
            // Pass 20: nor is a porter on its trip (no `PorterTrip` outside `game_v25`).
            if resources.rules.society.relations.is_some() {
                for (_, (agent, _)) in world.query::<(&Agent, &PorterTrip)>().iter() {
                    cold.remove(&agent.id);
                }
            }
        }
        // M3: sleepers under a roof (a roofed room) keep warm — M5: not in a
        // cold snap, when only a roof with the hearth's fire lit keeps them warm.
        if cold_night
            && snap.is_none()
            && resources
                .rules
                .colony
                .is_some_and(|rule| rule.space.is_some())
        {
            for (_, (agent, pos, state)) in world
                .query::<(&Agent, &Position, &sim_core::components::AgentState)>()
                .iter()
            {
                let asleep = matches!(
                    state,
                    sim_core::components::AgentState::Consuming {
                        target: sim_core::components::TargetKind::Sleep
                    }
                );
                if asleep && crate::runtime::rooms::roofed(resources, (pos.x, pos.y)) {
                    cold.remove(&agent.id);
                }
            }
        }
        // M5 (ADR 0029): a worn top keeps its wearer warm some cold hours — they
        // are cold with chance `Top::cold_permille` (hide wrap 700 ‰, fur cloak
        // 500 ‰), one keyed roll an hour (`STREAM_APPAREL`, the lineage key), so
        // cold hours, the mood's cold night and exposure damage all scale by it.
        if !cold.is_empty() {
            let seed = resources.world_seed;
            let warm: Vec<AgentId> = world
                .query::<(&Agent, &sim_core::colony::people::Apparel, Option<&Lineage>)>()
                .iter()
                .filter(|(_, (agent, apparel, _))| {
                    apparel.top.cold_permille() < 1000 && cold.contains(&agent.id)
                })
                .filter(|(_, (agent, apparel, lineage))| {
                    let key = lineage.map_or(agent.id, |l| l.key);
                    keyed_below(seed, STREAM_APPAREL, key, now, 0, 0, 1000)
                        >= u64::from(apparel.top.cold_permille())
                })
                .map(|(_, (agent, ..))| agent.id)
                .collect();
            for id in warm {
                cold.remove(&id);
            }
        }
        let mut agents: Vec<(AgentId, hecs::Entity, bool)> = world
            .query::<(&Agent, Option<&Cold>)>()
            .iter()
            .map(|(entity, (agent, marker))| (agent.id, entity, marker.is_some()))
            .collect();
        agents.sort_by_key(|(id, ..)| *id);
        let mut marked = 0u64;
        let mut exposed: Vec<(AgentId, hecs::Entity)> = Vec::new();
        for (id, entity, has) in agents {
            match (cold.contains(&id), has) {
                (true, false) => {
                    let _ = world.insert_one(entity, Cold);
                    marked += 1;
                    exposed.push((id, entity));
                }
                (true, true) => {
                    marked += 1;
                    exposed.push((id, entity));
                }
                (false, true) => {
                    let _ = world.remove_one::<Cold>(entity);
                }
                (false, false) => {}
            }
        }
        resources.tallies.cold_member_hours += marked;
        if let Some(exposure) = resources.rules.society.exposure {
            expose(world, resources, &exposure, &exposed, now, snap);
        }
    }
}

/// Pass 10: wound every agent marked `Cold` this hour (`exposed`, in AgentId
/// order) by its life stage's hourly damage, stamp its `ExposureClock` with
/// this run's tick, and book exposure deaths in AgentId order.
///
/// Pass 11 (exposure v2): the damage is [`exposure_damage`] — the stage's
/// rate times the hardiness factor (with `use_hardiness`) times the hunger
/// factor (with `hunger_weight`). With v16's values both factors are 1.
fn expose(
    world: &mut World,
    resources: &mut SimResources,
    rule: &ExposureRule,
    exposed: &[(AgentId, hecs::Entity)],
    now: u64,
    snap: Option<sim_core::rules::StoryRule>,
) {
    let mut deaths: Vec<(hecs::Entity, AgentId, (u32, u32))> = Vec::new();
    for &(id, entity) in exposed {
        let stage = if world.get::<&Child>(entity).is_ok() {
            0
        } else if world.get::<&Elder>(entity).is_ok() {
            2
        } else {
            1
        };
        let stamped = match world.get::<&mut ExposureClock>(entity) {
            Ok(mut clock) => {
                clock.last_cold_tick = now;
                true
            }
            Err(_) => false,
        };
        if !stamped {
            let _ = world.insert_one(
                entity,
                ExposureClock {
                    last_cold_tick: now,
                },
            );
        }
        let hardiness = world.get::<&Hardiness>(entity).ok().map(|h| *h);
        let hunger = world
            .get::<&Hunger>(entity)
            .map(|h| f64::from(h.value))
            .unwrap_or(0.0);
        let mut damage = exposure_damage(rule, stage, hardiness, hunger);
        // M5: a cold snap doubles the damage outside a roofed room (ADR 0027).
        let pos = world
            .get::<&Position>(entity)
            .map_or((0, 0), |p| (p.x, p.y));
        if let Some(snap) = snap {
            if !crate::runtime::rooms::roofed(resources, pos) {
                damage = damage * f64::from(snap.snap_exposure_permille) / 1000.0;
            }
            crate::runtime::story::note_cause(
                resources,
                id,
                pos,
                sim_core::colony::story::PressureKind::ColdSnap,
                0,
            );
        }
        let dead = match world.get::<&mut BodyHealth>(entity) {
            Ok(mut health) => {
                health.apply_damage(damage);
                health.is_dead()
            }
            Err(_) => false,
        };
        if dead {
            let position = world
                .get::<&Position>(entity)
                .map(|p| (p.x, p.y))
                .unwrap_or((0, 0));
            deaths.push((entity, id, position));
        }
    }
    for (entity, id, position) in deaths {
        despawn_agent(
            world,
            resources,
            entity,
            id,
            position,
            DeathReason::Exposure,
            now,
        );
    }
}

/// Pass 11: the hourly exposure damage of a `Cold` resident of life stage
/// `stage` (0 child, 1 adult, 2 elder):
///
/// ```text
/// damage_per_cold_hour[stage]
///   × (hardiness_permille / 1000   if use_hardiness, else 1)
///   × (1 + hunger_weight × clamp((hunger − 50) / 50, 0, 1))
/// ```
///
/// A resident without a `Hardiness` (not expected under `use_hardiness`)
/// counts factor 1. With v16's rule (`use_hardiness` false, `hunger_weight`
/// 0) this is exactly `damage_per_cold_hour[stage]`.
pub fn exposure_damage(
    rule: &ExposureRule,
    stage: usize,
    hardiness: Option<Hardiness>,
    hunger: f64,
) -> f64 {
    let mut damage = rule.damage_per_cold_hour[stage];
    if rule.use_hardiness {
        if let Some(hardiness) = hardiness {
            damage *= hardiness.factor();
        }
    }
    if rule.hunger_weight != 0.0 {
        let excess = ((hunger - 50.0) / 50.0).clamp(0.0, 1.0);
        damage *= 1.0 + rule.hunger_weight * excess;
    }
    damage
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::Settlement;
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::WorldRules;

    #[test]
    fn metadata() {
        let s = FireSystem::new();
        assert_eq!(
            (s.name(), s.priority(), s.tick_interval()),
            ("FireSystem", 147, 60)
        );
    }

    #[test]
    fn no_rule_is_a_no_op() {
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        resources.rules = WorldRules::baseline(0);
        resources.rules.society.firewood = None;
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.store(ResourceKind::Wood, 10);
        resources.settlements.insert(1, settlement);
        let mut world = World::new();
        FireSystem::new().tick(&mut world, &mut resources);
        assert!(resources.fires.is_empty());
        assert_eq!(
            resources.settlements[&1].stockpile.get(&ResourceKind::Wood),
            Some(&10)
        );
    }
}
