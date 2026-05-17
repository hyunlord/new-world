//! V7 Phase 7-β (re-plan, Option A) — `SocialDecaySystem` (priority 135, every tick).
//!
//! Mirrors [`HungerDecaySystem`](super::HungerDecaySystem),
//! [`ThirstDecaySystem`](super::ThirstDecaySystem), and
//! [`SleepDecaySystem`](super::SleepDecaySystem) for the fourth need
//! (`Social`). Walks every entity carrying [`Social`] and calls
//! [`Social::tick`].
//!
//! # Priority ordering
//!
//! ```text
//! 90   BuildingStampSystem
//! 100  InfluenceUpdateSystem
//! 110  AgentInfluenceSampleSystem
//! 120  AgentMovementSystem
//! 125  AgentDecisionSystem          ← reads Hunger/Thirst/Sleep/Social BEFORE decay
//! 130  HungerDecaySystem
//! 131  ThirstDecaySystem
//! 132  SleepDecaySystem
//! 133  ConstructionSystem
//! 134  SocialInteractionSystem
//! 135  SocialDecaySystem            ← lands here in P7β re-plan
//! 1000 InfluenceVisualizationSystem
//! ```
//!
//! Running at 135 (right after `SocialInteractionSystem` at 134) keeps the
//! Social-need pair adjacent: AgentDecisionSystem at 125 reads pre-decay
//! `Social.loneliness` for the current tick, the social interaction
//! handshake/progress/completion fires at 134, and `Social.loneliness`
//! advances afterward — symmetric with how Hunger/Thirst/Sleep decay
//! follows their respective consume passes.

use hecs::World;
use sim_core::components::Social;
use sim_engine::{RuntimeSystem, SimResources};

/// Phase 7-β re-plan need-decay system. Stateless — all per-agent state
/// lives in the [`Social`] component.
#[derive(Debug, Default)]
pub struct SocialDecaySystem;

impl SocialDecaySystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for SocialDecaySystem {
    fn name(&self) -> &str {
        "SocialDecaySystem"
    }

    fn priority(&self) -> u32 {
        135
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, _resources: &mut SimResources) {
        for (_entity, social) in world.query::<&mut Social>().iter() {
            social.tick();
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
    fn priority_is_135_interval_is_1() {
        let s = SocialDecaySystem::new();
        assert_eq!(s.priority(), 135);
        assert_eq!(s.tick_interval(), 1);
        assert_eq!(s.name(), "SocialDecaySystem");
    }

    #[test]
    fn tick_advances_loneliness() {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(1, 1);
        e.world.insert_one(entity, Social::new(0.0, 3.0)).unwrap();

        let mut sys = SocialDecaySystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let s = e.world.get::<&Social>(entity).unwrap();
        assert_eq!(s.loneliness, 3.0);
    }

    #[test]
    fn tick_respects_saturation_clamp() {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(5, 5);
        e.world.insert_one(entity, Social::new(99.0, 5.0)).unwrap();

        let mut sys = SocialDecaySystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let s = e.world.get::<&Social>(entity).unwrap();
        assert_eq!(s.loneliness, Social::SATURATION);
    }
}
