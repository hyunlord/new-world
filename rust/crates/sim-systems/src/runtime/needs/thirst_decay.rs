//! V7 Phase 5-β — `ThirstDecaySystem` (priority 131, every tick).
//!
//! Mirrors [`HungerDecaySystem`](super::HungerDecaySystem) for the
//! second need. Walks every entity carrying [`Thirst`] and calls
//! [`Thirst::tick`].
//!
//! # Priority ordering
//!
//! ```text
//! 90   BuildingStampSystem
//! 100  InfluenceUpdateSystem
//! 110  AgentInfluenceSampleSystem
//! 120  AgentMovementSystem
//! 125  AgentDecisionSystem          ← reads Hunger/Thirst BEFORE decay
//! 130  HungerDecaySystem
//! 131  ThirstDecaySystem            ← lands here in P5β
//! 1000 InfluenceVisualizationSystem
//! ```
//!
//! Running at 131 (right after `HungerDecaySystem` at 130) keeps the
//! two need-decay systems adjacent and uniform: AgentDecisionSystem at
//! 125 still reads the pre-decay need values for the current tick,
//! both decays apply afterward, and visualisation observes the post-
//! decay state.

use hecs::World;
use sim_core::components::Thirst;
use sim_engine::{RuntimeSystem, SimResources};

/// Phase 5-β need-decay system. Stateless — all per-agent state lives
/// in the [`Thirst`] component.
#[derive(Debug, Default)]
pub struct ThirstDecaySystem;

impl ThirstDecaySystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for ThirstDecaySystem {
    fn name(&self) -> &str {
        "ThirstDecaySystem"
    }

    fn priority(&self) -> u32 {
        131
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, _resources: &mut SimResources) {
        for (_entity, thirst) in world.query::<&mut Thirst>().iter() {
            thirst.tick();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    fn fresh_engine() -> SimEngine {
        SimEngine::new(32, 32, MaterialRegistry::new())
    }

    #[test]
    fn priority_is_131_interval_is_1() {
        let s = ThirstDecaySystem::new();
        assert_eq!(s.priority(), 131);
        assert_eq!(s.tick_interval(), 1);
        assert_eq!(s.name(), "ThirstDecaySystem");
    }

    #[test]
    fn tick_advances_thirst() {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(1, 1);
        e.world.insert_one(entity, Thirst::new(0.0, 3.0)).unwrap();

        let mut sys = ThirstDecaySystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let t = e.world.get::<&Thirst>(entity).unwrap();
        assert_eq!(t.value, 3.0);
    }

    #[test]
    fn tick_is_noop_when_no_thirst_components() {
        let mut e = fresh_engine();
        let _ = e.spawn_agent(0, 0); // agent without Thirst
        let mut sys = ThirstDecaySystem::new();
        sys.tick(&mut e.world, &mut e.resources);
    }

    #[test]
    fn tick_respects_saturation_clamp() {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(5, 5);
        e.world.insert_one(entity, Thirst::new(99.0, 5.0)).unwrap();

        let mut sys = ThirstDecaySystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let t = e.world.get::<&Thirst>(entity).unwrap();
        assert_eq!(t.value, Thirst::SATURATION);
    }
}
