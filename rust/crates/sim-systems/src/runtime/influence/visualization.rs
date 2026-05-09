//! `InfluenceVisualizationSystem` — priority 1000, every 6 ticks.
//!
//! Phase 0 Section 2.8 base. Captures a coarse digest of the current
//! buffer at the system's interval so the debug HUD and harness tests can
//! confirm influence activity without scanning the full grid every frame.
//!
//! Design: instantiated directly (not only via `Box<dyn RuntimeSystem>`) so
//! that [`last_digest`] is accessible to harness tests. The engine also
//! accepts a boxed instance via `register_system`; in that case the digest
//! is observable only through SimBridge (Phase 3+).

use hecs::World;
use sim_core::influence::InfluenceChannel;
use sim_engine::{RuntimeSystem, SimResources};

/// Coarse per-tick summary of the influence grid (debug / harness only).
#[derive(Debug, Default, Clone, Copy)]
pub struct VisualizationDigest {
    /// Engine tick at which this digest was captured.
    pub tick: u64,
    /// Sum of `current[Warmth]` over all tiles.
    pub warmth_total: u64,
    /// Maximum `current[Danger]` value across all tiles.
    pub danger_peak: u8,
}

/// Phase 2 influence visualiser — fires every 6 ticks at priority 1000.
pub struct InfluenceVisualizationSystem {
    last: VisualizationDigest,
}

impl InfluenceVisualizationSystem {
    /// Construct a new visualiser with a zeroed digest.
    pub fn new() -> Self {
        Self {
            last: VisualizationDigest::default(),
        }
    }

    /// Borrow the most recent digest captured by the last `tick` call.
    ///
    /// Returns a zeroed digest if `tick` has never been called.
    pub fn last_digest(&self) -> &VisualizationDigest {
        &self.last
    }
}

impl Default for InfluenceVisualizationSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSystem for InfluenceVisualizationSystem {
    fn name(&self) -> &str {
        "InfluenceVisualizationSystem"
    }

    fn priority(&self) -> u32 {
        1000
    }

    fn tick_interval(&self) -> u64 {
        6
    }

    fn tick(&mut self, _world: &mut World, resources: &mut SimResources) {
        let warmth_total: u64 = resources
            .influence_grid
            .current_buf(InfluenceChannel::Warmth)
            .iter()
            .map(|&v| v as u64)
            .sum();

        let danger_peak: u8 = resources
            .influence_grid
            .current_buf(InfluenceChannel::Danger)
            .iter()
            .copied()
            .max()
            .unwrap_or(0);

        self.last = VisualizationDigest {
            tick: resources.current_tick,
            warmth_total,
            danger_peak,
        };
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
        let s = InfluenceVisualizationSystem::new();
        assert_eq!(s.name(), "InfluenceVisualizationSystem");
        assert_eq!(s.priority(), 1000);
        assert_eq!(s.tick_interval(), 6);
    }

    #[test]
    fn digest_zero_on_fresh_engine() {
        let mut e = empty_engine();
        let mut s = InfluenceVisualizationSystem::new();
        s.tick(&mut e.world, &mut e.resources);
        assert_eq!(s.last_digest().warmth_total, 0);
        assert_eq!(s.last_digest().danger_peak, 0);
    }

    #[test]
    fn digest_captures_warmth_total_and_danger_peak() {
        let mut e = empty_engine();
        // Warmth: [0]=50, [1]=70 → sum=120
        {
            let buf = e
                .resources
                .influence_grid
                .pending_buf_mut(InfluenceChannel::Warmth);
            buf[0] = 50;
            buf[1] = 70;
        }
        // Danger: [0]=200, [10]=100 → peak=200
        {
            let buf = e
                .resources
                .influence_grid
                .pending_buf_mut(InfluenceChannel::Danger);
            buf[0] = 200;
            buf[10] = 100;
        }
        e.resources.influence_grid.swap();

        let mut s = InfluenceVisualizationSystem::new();
        s.tick(&mut e.world, &mut e.resources);

        assert_eq!(s.last_digest().warmth_total, 120);
        assert_eq!(s.last_digest().danger_peak, 200);
    }

    #[test]
    fn interval_6_fires_at_correct_ticks() {
        let mut e = empty_engine();
        e.register_system(Box::new(InfluenceVisualizationSystem::new()));
        // Run 13 ticks — system fires at 0, 6, 12. Must not panic.
        for _ in 0..13 {
            e.tick();
        }
        assert_eq!(e.current_tick(), 13);
    }
}
