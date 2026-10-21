//! SpatialIndexSystem: rebuilds the agents' spatial index (`SimResources.spatial.agents`) every colony tick.
//!
//! Priority: 119 (`SPATIAL_INDEX_PRIORITY`). Interval: 1 tick. Gate: `WorldRules.colony`.
//! Reads: `Agent`, `Position`
//! Writes: `SimResources.spatial.agents`, `spatial.agents_tick`
//! Events: none
//!
//! M1.5 goal 6.2: O(N) per tick, before movement (120), so every later system
//! of the tick sees the positions the tick started with. The colony job
//! searches use the colony indexes (`spatial.colony`, rebuilt on change by
//! `JobAssignSystem`); the agents' index has no reader in M1.5 (M2's work
//! search by proximity is its first). The baseline world never builds it.
use hecs::World;
use sim_core::components::{Agent, Position};
use sim_engine::{RuntimeSystem, SimResources};

/// Priority of [`SpatialIndexSystem`]: just before movement (120).
pub const SPATIAL_INDEX_PRIORITY: u32 = 119;

/// See the module header.
#[derive(Debug, Default)]
pub struct SpatialIndexSystem;

impl RuntimeSystem for SpatialIndexSystem {
    fn name(&self) -> &str {
        "SpatialIndexSystem"
    }

    fn priority(&self) -> u32 {
        SPATIAL_INDEX_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        if resources.rules.colony.is_none() {
            return;
        }
        let index = &mut resources.spatial.agents;
        index.clear();
        for (_, (agent, pos)) in world.query::<(&Agent, &Position)>().iter() {
            index.insert((pos.x, pos.y), agent.id);
        }
        resources.spatial.agents_tick = Some(resources.current_tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_index_holds_every_agent_in_the_colony_only() {
        let mut world = World::new();
        let mut resources = SimResources::new(40, 40, sim_core::MaterialRegistry::new());
        world.spawn((Agent { id: 4 }, Position::new(3, 3)));
        world.spawn((Agent { id: 2 }, Position::new(30, 3)));
        world.spawn((Agent { id: 7 }, Position::new(3, 3)));
        let mut system = SpatialIndexSystem;
        system.tick(&mut world, &mut resources);
        assert!(
            resources.spatial.agents.is_empty(),
            "no colony rule: no index"
        );
        resources.rules.colony = Some(sim_core::rules::ColonyRule::M1);
        resources.current_tick = 9;
        system.tick(&mut world, &mut resources);
        assert_eq!(resources.spatial.agents.len(), 3);
        assert_eq!(resources.spatial.agents_tick, Some(9));
        assert_eq!(
            resources
                .spatial
                .agents
                .nearest_where((0, 0), 10, |_, _| true),
            Some(((3, 3), 4))
        );
        assert_eq!(
            resources
                .spatial
                .agents
                .nearest_where((29, 0), 10, |_, _| true),
            Some(((30, 3), 2))
        );
    }
}
