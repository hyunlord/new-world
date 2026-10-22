//! SleepDecaySystem: advances every agent's sleep need.
//!
//! Priority: 132 · Interval: 1 tick · Gate: none
//! Reads: `Sleep`, `AgentState`, `Position`; `SimResources.colony.rooms` (M3)
//! Writes: `Sleep`
//! Events: none
//!
//! M3 (ADR 0023, colony with built space): while asleep (`Consuming { Sleep }`)
//! in a roofed bedroom, fatigue grows at `SpaceRule::bedroom_sleep_permille`
//! (800 ‰) of its rate, so the night's rest goes about 20 % further.
//!
//! V7 Phase 5-γ — `SleepDecaySystem` (priority 132, every tick).
//!
//! Mirrors [`HungerDecaySystem`](super::HungerDecaySystem) and
//! [`ThirstDecaySystem`](super::ThirstDecaySystem) for the third need.
//! Walks every entity carrying [`Sleep`] and calls [`Sleep::tick`].
//!
//! # Priority ordering
//!
//! ```text
//! 90   BuildingStampSystem
//! 100  InfluenceUpdateSystem
//! 110  AgentInfluenceSampleSystem
//! 120  AgentMovementSystem
//! 125  AgentDecisionSystem          ← reads Hunger/Thirst/Sleep BEFORE decay
//! 130  HungerDecaySystem
//! 131  ThirstDecaySystem
//! 132  SleepDecaySystem             ← lands here in P5γ
//! 1000 InfluenceVisualizationSystem
//! ```
//!
//! Running at 132 (right after `ThirstDecaySystem` at 131) keeps the
//! three need-decay systems adjacent and uniform: AgentDecisionSystem at
//! 125 still reads the pre-decay need values for the current tick, all
//! three decays apply afterward, and visualisation observes the post-
//! decay state.

use hecs::World;
use sim_core::components::{AgentState, Position, Sleep, TargetKind};
use sim_engine::{RuntimeSystem, SimResources};

/// Phase 5-γ need-decay system. Stateless — all per-agent state lives
/// in the [`Sleep`] component.
#[derive(Debug, Default)]
pub struct SleepDecaySystem;

impl SleepDecaySystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for SleepDecaySystem {
    fn name(&self) -> &str {
        "SleepDecaySystem"
    }

    fn priority(&self) -> u32 {
        132
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let bedroom = resources
            .rules
            .colony
            .and_then(|rule| rule.space)
            .map(|space| space.bedroom_sleep_permille)
            .filter(|_| !resources.colony.rooms.is_empty());
        let Some(permille) = bedroom else {
            for (_entity, sleep) in world.query::<&mut Sleep>().iter() {
                sleep.tick();
            }
            return;
        };
        let factor = f64::from(permille) / 1000.0;
        for (_entity, (sleep, state, pos)) in world
            .query::<(&mut Sleep, Option<&AgentState>, Option<&Position>)>()
            .iter()
        {
            let asleep = matches!(
                state,
                Some(AgentState::Consuming {
                    target: TargetKind::Sleep
                })
            );
            let in_bedroom = asleep
                && pos.is_some_and(|p| {
                    crate::runtime::rooms::roofed_as(
                        resources,
                        (p.x, p.y),
                        sim_core::rooms::RoomRole::Bedroom,
                    )
                });
            if in_bedroom {
                sleep.fatigue =
                    (sleep.fatigue + sleep.growth_rate * factor).clamp(0.0, Sleep::SATURATION);
            } else {
                sleep.tick();
            }
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
    fn priority_is_132_interval_is_1() {
        let s = SleepDecaySystem::new();
        assert_eq!(s.priority(), 132);
        assert_eq!(s.tick_interval(), 1);
        assert_eq!(s.name(), "SleepDecaySystem");
    }

    #[test]
    fn tick_advances_fatigue() {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(1, 1);
        e.world.insert_one(entity, Sleep::new(0.0, 3.0)).unwrap();

        let mut sys = SleepDecaySystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let s = e.world.get::<&Sleep>(entity).unwrap();
        assert_eq!(s.fatigue, 3.0);
    }

    #[test]
    fn tick_respects_saturation_clamp() {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(5, 5);
        e.world.insert_one(entity, Sleep::new(99.0, 5.0)).unwrap();

        let mut sys = SleepDecaySystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let s = e.world.get::<&Sleep>(entity).unwrap();
        assert_eq!(s.fatigue, Sleep::SATURATION);
    }
}
