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

use std::collections::{BTreeMap, BTreeSet};

use hecs::World;
use sim_core::components::{Agent, AgentId, Cold, ResourceKind, SettlementId};
use sim_engine::{FireState, RuntimeSystem, SimResources};

use crate::runtime::decision::agent_decision::clock_in_window;
use crate::runtime::settlement::firewood::night_season;

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
        let per_hour = u32::from(rule.burn_per_hour[season]);
        let mut ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
        ids.sort_unstable();
        let mut fires = BTreeMap::new();
        for sid in &ids {
            let lit = burning
                && resources
                    .settlements
                    .get_mut(sid)
                    .is_some_and(|settlement| settlement.withdraw(ResourceKind::Wood, per_hour) == per_hour);
            fires.insert(*sid, FireState { lit });
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
        }
        let mut agents: Vec<(AgentId, hecs::Entity, bool)> = world
            .query::<(&Agent, Option<&Cold>)>()
            .iter()
            .map(|(entity, (agent, marker))| (agent.id, entity, marker.is_some()))
            .collect();
        agents.sort_by_key(|(id, ..)| *id);
        let mut marked = 0u64;
        for (id, entity, has) in agents {
            match (cold.contains(&id), has) {
                (true, false) => {
                    let _ = world.insert_one(entity, Cold);
                    marked += 1;
                }
                (true, true) => marked += 1,
                (false, true) => {
                    let _ = world.remove_one::<Cold>(entity);
                }
                (false, false) => {}
            }
        }
        resources.tallies.cold_member_hours += marked;
    }
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
        assert_eq!((s.name(), s.priority(), s.tick_interval()), ("FireSystem", 147, 60));
    }

    #[test]
    fn no_rule_is_a_no_op() {
        let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
        resources.rules = WorldRules::game_v10();
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.store(ResourceKind::Wood, 10);
        resources.settlements.insert(1, settlement);
        let mut world = World::new();
        FireSystem::new().tick(&mut world, &mut resources);
        assert!(resources.fires.is_empty());
        assert_eq!(resources.settlements[&1].stockpile.get(&ResourceKind::Wood), Some(&10));
    }
}
