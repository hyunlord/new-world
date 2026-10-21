//! FertilitySystem: once a day at 12:00, queues the births that are due and lets eligible women conceive (`BirthRule::Fertility`).
//!
//! Priority: 150 (`FERTILITY_PRIORITY`). Interval: 1,440 ticks, at 12:00 (`tick_offset`). Gate: `WorldRules.society.birth` = `Fertility`.
//! Reads: `Agent`, `Sex`, `Kin`, `Life`, `Lineage`, `Child`, `Pregnant`, `Hunger`, `Thirst`, `BodyHealth`; `SimResources.settlements`, `rules.society.birth`, `rules.calendar`, `rules.clock_offset_ticks`, `world_seed`
//! Writes: `Pregnant` (inserted at a conception, removed when due); `SimResources.due_births`
//! Events: none (the birth's events and chronicle entry come from `SettlementSystem`)
//!
//! M1.5 demographics (goal 5.3, ADR 0017). A woman is eligible on a day when
//! she is female (`Sex`), not a child, alive, aged in
//! `[fertile_from_years, fertile_to_years)` (closed form from `Life::born_tick`
//! and the calendar), not pregnant, at least `postpartum_days` past her last
//! birth (`Kin::last_birth_tick`), has a living partner (`Kin::partner`,
//! with `requires_partner`), and belongs to a fed settlement (its members'
//! mean hunger and thirst at most `max_mean_hunger` / `max_mean_thirst`, the
//! gate `BirthRule::Provisioned` uses). She conceives when the keyed draw
//! `keyed_below(seed, STREAM_CONCEIVE, lineage key, day index, 0, replicate,
//! 1000)` is below `conception_per_day_permille`: a pure function of the key
//! (pitfalls: keyed by lineage, never by `AgentId`), so skipping a woman
//! shifts no other draw. The child is due `gestation_days` later, at 12:00;
//! that day the system removes `Pregnant` and queues a `DueBirth` for
//! the settlement she belongs to, and `SettlementSystem` spawns the child at
//! its hearth through the existing birth path on the next tick (parents:
//! the mother and the father named at the conception; the household follows
//! from `Kin`, `agent::households::households`). Women and settlements are
//! visited in id order; structural changes are applied after the queries.
//!
//! The system runs at midday, not at the goal's 06:00 (M1.5 DECISIONS D7): at
//! 06:00 the members' mean thirst is at its overnight peak (56–67 against the
//! gate's 60 on seeds 1 and 4), so the fed gate would be a coin flip; at
//! 12:00 the means are 25–32 (hunger) and 32–40 (thirst).
use hecs::{Entity, World};
use sim_core::components::{
    Agent, AgentId, BodyHealth, Child, DueBirth, Hunger, Kin, Life, Lineage, Pregnant,
    SettlementId, Sex, Thirst,
};
use sim_core::rng::{keyed_below, STREAM_CONCEIVE};
use sim_core::rules::BirthRule;
use sim_engine::{RuntimeSystem, SimResources};
use std::collections::{BTreeMap, BTreeSet};

/// Priority of [`FertilitySystem`]: after the relations system (149).
pub const FERTILITY_PRIORITY: u32 = 150;
/// Interval of [`FertilitySystem`]: one day of the 1,440-tick calendar day
/// (both worlds' calendars have 1,440 ticks per day).
pub const FERTILITY_INTERVAL: u64 = 1_440;
/// Clock minute at which the system runs (12:00; DECISIONS D7).
pub const FERTILITY_MINUTE: u64 = 720;

/// See the module header.
#[derive(Debug, Default)]
pub struct FertilitySystem;

/// The settlement `agent` belongs to (the lowest id that lists it).
pub fn settlement_of(resources: &SimResources, agent: AgentId) -> Option<SettlementId> {
    let mut ids: Vec<SettlementId> = resources
        .settlements
        .iter()
        .filter(|(_, s)| s.member_agents.contains(&agent))
        .map(|(id, _)| *id)
        .collect();
    ids.sort_unstable();
    ids.first().copied()
}

/// Settlements whose members' mean hunger and thirst are within the limits.
fn fed_settlements(
    world: &World,
    resources: &SimResources,
    max_hunger: f64,
    max_thirst: f64,
) -> BTreeSet<SettlementId> {
    let mut needs: Vec<(AgentId, f64, f64)> = world
        .query::<(&Agent, &Hunger, &Thirst)>()
        .iter()
        .map(|(_, (a, h, t))| (a.id, f64::from(h.value), t.value))
        .collect();
    needs.sort_by_key(|row| row.0);
    let mut ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
    ids.sort_unstable();
    ids.into_iter()
        .filter(|sid| {
            let Some(settlement) = resources.settlements.get(sid) else {
                return false;
            };
            let members: Vec<_> = needs
                .iter()
                .filter(|(id, _, _)| settlement.member_agents.contains(id))
                .collect();
            if members.is_empty() {
                return false;
            }
            let n = members.len() as f64;
            let hunger = members.iter().map(|(_, h, _)| *h).sum::<f64>() / n;
            let thirst = members.iter().map(|(_, _, t)| *t).sum::<f64>() / n;
            hunger <= max_hunger && thirst <= max_thirst
        })
        .collect()
}

impl RuntimeSystem for FertilitySystem {
    fn name(&self) -> &str {
        "FertilitySystem"
    }

    fn priority(&self) -> u32 {
        FERTILITY_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        FERTILITY_INTERVAL
    }

    fn tick_offset(&self, resources: &SimResources) -> u64 {
        let day = FERTILITY_INTERVAL;
        (FERTILITY_MINUTE + day - resources.rules.clock_offset_ticks % day) % day
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let BirthRule::Fertility {
            conception_per_day_permille,
            fertile_from_years,
            fertile_to_years,
            gestation_days,
            postpartum_days,
            requires_partner,
            max_mean_hunger,
            max_mean_thirst,
            ..
        } = resources.rules.society.birth
        else {
            return;
        };
        let calendar = resources.rules.calendar;
        let now = resources.current_tick;
        let day_index =
            now.saturating_add(resources.rules.clock_offset_ticks) / calendar.ticks_per_day.max(1);

        // 1. Births that are due today.
        let mut due: Vec<(AgentId, Entity, Pregnant)> = world
            .query::<(&Agent, &Pregnant)>()
            .iter()
            .filter(|(_, (_, p))| p.due_tick <= now)
            .map(|(entity, (agent, p))| (agent.id, entity, *p))
            .collect();
        due.sort_by_key(|row| row.0);
        for (mother, entity, pregnancy) in due {
            // Without a settlement the birth waits for the next day.
            let Some(settlement) = settlement_of(resources, mother) else {
                continue;
            };
            if world.remove_one::<Pregnant>(entity).is_ok() {
                resources.due_births.push(DueBirth {
                    mother,
                    father: pregnancy.father,
                    settlement,
                });
            }
        }

        // 2. Conceptions.
        let fed = fed_settlements(world, resources, max_mean_hunger, max_mean_thirst);
        if fed.is_empty() {
            return;
        }
        let alive: BTreeSet<AgentId> = world
            .query::<(&Agent, Option<&BodyHealth>)>()
            .iter()
            .filter(|(_, (_, h))| h.is_none_or(|h| !h.is_dead()))
            .map(|(_, (a, _))| a.id)
            .collect();
        let from = calendar.years_to_ticks(u64::from(fertile_from_years)) as i64;
        let to = calendar.years_to_ticks(u64::from(fertile_to_years)) as i64;
        let postpartum = calendar.days_to_ticks(u64::from(postpartum_days));
        let gestation = calendar.days_to_ticks(u64::from(gestation_days));
        let replicate = resources.noise_replicate_for(now);
        let mut women: BTreeMap<AgentId, (Entity, u64, Option<AgentId>)> = BTreeMap::new();
        for (entity, (agent, sex, kin, life, lineage, child, pregnant)) in world
            .query::<(
                &Agent,
                &Sex,
                &Kin,
                &Life,
                &Lineage,
                Option<&Child>,
                Option<&Pregnant>,
            )>()
            .iter()
        {
            let age = life.age_ticks(now);
            let partner = kin.partner.filter(|p| alive.contains(p));
            let eligible = sex.female
                && child.is_none()
                && pregnant.is_none()
                && alive.contains(&agent.id)
                && (from..to).contains(&age)
                && kin
                    .last_birth_tick
                    .is_none_or(|t| now >= t.saturating_add(postpartum))
                && (!requires_partner || partner.is_some());
            if eligible {
                women.insert(agent.id, (entity, lineage.key, partner));
            }
        }
        let mut conceived: Vec<(Entity, Pregnant)> = Vec::new();
        for (id, (entity, key, partner)) in women {
            if !settlement_of(resources, id).is_some_and(|sid| fed.contains(&sid)) {
                continue;
            }
            let draw = keyed_below(
                resources.world_seed,
                STREAM_CONCEIVE,
                key,
                day_index,
                0,
                replicate,
                1000,
            );
            if draw < u64::from(conception_per_day_permille) {
                conceived.push((
                    entity,
                    Pregnant {
                        conceived_tick: now,
                        due_tick: now + gestation,
                        father: partner.unwrap_or(id),
                    },
                ));
            }
        }
        for (entity, pregnancy) in conceived {
            let _ = world.insert_one(entity, pregnancy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_system_runs_at_midday() {
        let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        resources.rules.clock_offset_ticks = 480;
        let system = FertilitySystem;
        let offset = system.tick_offset(&resources);
        // Tick 240 + 480 offset = minute 720 (12:00) of day 0.
        assert_eq!(offset, 240);
        assert_eq!((offset + 480) % 1_440, 720);
        resources.rules.clock_offset_ticks = 0;
        assert_eq!(system.tick_offset(&resources), 720);
    }
}
