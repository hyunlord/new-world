//! `AgentInfluenceSampleSystem` — priority 110, every tick.
//!
//! Phase 0 Section 2.5.4 base. Reads the **current-side** influence buffer
//! for every agent entity and writes the result into a debug component.
//!
//! Read-after-update contract: priority 110 > 100 (InfluenceUpdateSystem),
//! so every read sees the freshly-swapped current buffer.
//!
//! # Position component
//!
//! V7 Phase 4-α landed the canonical [`sim_core::components::Position`]
//! and this module's placeholder was removed (`agent_sample.rs:9-15`
//! landmark — single-line rewire). The re-export below preserves the
//! `agent_sample::Position` symbol for downstream tests during the
//! migration window.

use hecs::World;
pub use sim_core::components::Position;
use sim_core::influence::InfluenceChannel;
use sim_engine::{RuntimeSystem, SimResources};

/// Most recent influence sample observed by an agent (debug component).
#[derive(Debug, Clone, Copy, Default)]
pub struct InfluenceSample {
    /// Warmth value at the agent's tile as of the last tick.
    pub warmth: u8,
    /// Danger value at the agent's tile as of the last tick.
    pub danger: u8,
}

/// Phase 2 agent influence sampler.
pub struct AgentInfluenceSampleSystem;

impl AgentInfluenceSampleSystem {
    /// Construct a new sampler.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AgentInfluenceSampleSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSystem for AgentInfluenceSampleSystem {
    fn name(&self) -> &str {
        "AgentInfluenceSampleSystem"
    }

    fn priority(&self) -> u32 {
        110
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let w = resources.influence_grid.width;
        let h = resources.influence_grid.height;
        for (_, (pos, sample)) in world.query::<(&Position, &mut InfluenceSample)>().iter() {
            // Skip out-of-bounds positions without panicking.
            if pos.x >= w || pos.y >= h {
                continue;
            }
            sample.warmth = resources
                .influence_grid
                .sample(pos.x, pos.y, InfluenceChannel::Warmth);
            sample.danger = resources
                .influence_grid
                .sample(pos.x, pos.y, InfluenceChannel::Danger);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    fn empty_engine() -> SimEngine {
        SimEngine::new(32, 32, MaterialRegistry::new())
    }

    #[test]
    fn metadata() {
        let s = AgentInfluenceSampleSystem::new();
        assert_eq!(s.name(), "AgentInfluenceSampleSystem");
        assert_eq!(s.priority(), 110);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn samples_zero_when_grid_empty() {
        let mut e = empty_engine();
        let id = e
            .world
            .spawn((Position { x: 5, y: 5 }, InfluenceSample::default()));
        e.register_system(Box::new(AgentInfluenceSampleSystem::new()));
        e.tick();
        let s = *e.world.get::<&InfluenceSample>(id).unwrap();
        assert_eq!(s.warmth, 0);
        assert_eq!(s.danger, 0);
    }

    #[test]
    fn samples_warmth_after_manual_write_and_swap() {
        let mut e = empty_engine();
        // Write into pending then swap so current has the value.
        let buf = e
            .resources
            .influence_grid
            .pending_buf_mut(InfluenceChannel::Warmth);
        buf[5 * 32 + 5] = 123;
        e.resources.influence_grid.swap();

        let id = e
            .world
            .spawn((Position { x: 5, y: 5 }, InfluenceSample::default()));
        // Run sampler manually (not via engine.tick — that re-swaps via InfluenceUpdateSystem).
        let mut s = AgentInfluenceSampleSystem::new();
        s.tick(&mut e.world, &mut e.resources);

        let sample = *e.world.get::<&InfluenceSample>(id).unwrap();
        assert_eq!(sample.warmth, 123);
        assert_eq!(sample.danger, 0);
    }

    #[test]
    fn out_of_bounds_position_is_skipped() {
        let mut e = empty_engine();
        let id = e.world.spawn((
            Position { x: 999, y: 999 },
            InfluenceSample {
                warmth: 42,
                danger: 7,
            },
        ));
        let mut s = AgentInfluenceSampleSystem::new();
        s.tick(&mut e.world, &mut e.resources);
        // Sample must remain unchanged — bounds check must have fired.
        let sample = *e.world.get::<&InfluenceSample>(id).unwrap();
        assert_eq!(sample.warmth, 42);
        assert_eq!(sample.danger, 7);
    }

    #[test]
    fn many_agents_read_own_tile() {
        let mut e = empty_engine();
        // Stamp distinct danger values at three tiles via pending + swap.
        {
            let buf = e
                .resources
                .influence_grid
                .pending_buf_mut(InfluenceChannel::Danger);
            buf[32 + 1] = 10;
            buf[2 * 32 + 2] = 20;
            buf[3 * 32 + 3] = 30;
        }
        e.resources.influence_grid.swap();

        let a = e
            .world
            .spawn((Position { x: 1, y: 1 }, InfluenceSample::default()));
        let b = e
            .world
            .spawn((Position { x: 2, y: 2 }, InfluenceSample::default()));
        let c = e
            .world
            .spawn((Position { x: 3, y: 3 }, InfluenceSample::default()));

        let mut s = AgentInfluenceSampleSystem::new();
        s.tick(&mut e.world, &mut e.resources);

        assert_eq!(e.world.get::<&InfluenceSample>(a).unwrap().danger, 10);
        assert_eq!(e.world.get::<&InfluenceSample>(b).unwrap().danger, 20);
        assert_eq!(e.world.get::<&InfluenceSample>(c).unwrap().danger, 30);
    }
}
