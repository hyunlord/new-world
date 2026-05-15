//! V7 Phase 5-α — `HungerDecaySystem` (priority 130, every tick).
//!
//! Walks every entity carrying [`Hunger`] and calls
//! [`Hunger::tick`]. The component itself enforces the
//! `[0, SATURATION]` clamp; this system's only job is to advance the
//! per-agent state on the simulation clock.
//!
//! # Priority ordering
//!
//! ```text
//! 90   BuildingStampSystem
//! 100  InfluenceUpdateSystem
//! 110  AgentInfluenceSampleSystem
//! 120  AgentMovementSystem
//! 130  HungerDecaySystem            ← lands here in P5α
//! 1000 InfluenceVisualizationSystem
//! ```
//!
//! Decay running after movement keeps the "moved-then-grew-hungrier"
//! ordering an agent observes when introspecting its own state. Phase
//! 5-β will introduce an "eat" action that *reduces* hunger; that
//! action runs as part of behavior selection earlier in the priority
//! table, so the decay tick here is the unconditional fallback.

use hecs::World;
use sim_core::components::Hunger;
use sim_engine::{RuntimeSystem, SimResources};

/// Phase 5-α need-decay system. Stateless — all per-agent state lives in
/// the [`Hunger`] component.
#[derive(Debug, Default)]
pub struct HungerDecaySystem;

impl HungerDecaySystem {
    /// Construct a fresh instance. Always cheap and side-effect free.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for HungerDecaySystem {
    fn name(&self) -> &str {
        "HungerDecaySystem"
    }

    fn priority(&self) -> u32 {
        130
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, _resources: &mut SimResources) {
        for (_entity, hunger) in world.query::<&mut Hunger>().iter() {
            hunger.tick();
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
    fn priority_is_130_interval_is_1() {
        let s = HungerDecaySystem::new();
        assert_eq!(s.priority(), 130);
        assert_eq!(s.tick_interval(), 1);
        assert_eq!(s.name(), "HungerDecaySystem");
    }

    #[test]
    fn tick_advances_hunger() {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(1, 1);
        e.world.insert_one(entity, Hunger::new(0.0, 2.0)).unwrap();

        let mut sys = HungerDecaySystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let h = e.world.get::<&Hunger>(entity).unwrap();
        assert_eq!(h.value, 2.0);
    }

    #[test]
    fn tick_is_noop_when_no_hunger_components() {
        let mut e = fresh_engine();
        let _ = e.spawn_agent(0, 0); // agent without Hunger
        let mut sys = HungerDecaySystem::new();
        // Must not panic / iterate cleanly over zero matches.
        sys.tick(&mut e.world, &mut e.resources);
    }
}
