//! `InfluenceUpdateSystem` — priority 100, every tick.
//!
//! Phase 0 Section 2.6 budget: Hot tier ≤ 0.5 ms @ 1K agents.
//!
//! Phase 2 land (T7.6) is a *dispatch shell*: it clears every pending
//! buffer and swaps double-buffers each tick. Actual source iteration
//! (BFS / shadowcast / linear propagation) lands together with
//! BuildingStampSystem plumbing in later phases — the shell guarantees
//! a deterministic zero-state baseline regardless of registration order.
//!
//! The clear-before-swap invariant is load-bearing: any system writing
//! pending stamps (e.g. BuildingStampSystem at priority 90) must have
//! already run before this system clears and swaps. Priority ordering
//! enforces this: 90 < 100.

use hecs::World;
use sim_engine::{RuntimeSystem, SimResources};

/// Phase 2 influence update dispatcher (clear + swap shell).
pub struct InfluenceUpdateSystem;

impl InfluenceUpdateSystem {
    /// Construct a new dispatcher.
    pub fn new() -> Self {
        Self
    }
}

impl Default for InfluenceUpdateSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSystem for InfluenceUpdateSystem {
    fn name(&self) -> &str {
        "InfluenceUpdateSystem"
    }

    fn priority(&self) -> u32 {
        100
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, _world: &mut World, resources: &mut SimResources) {
        // Phase 2 dispatch shell: zero-state baseline.
        // Order matters: clear pending FIRST, then swap so current receives
        // the zeroed buffer. Any higher-priority system (e.g. BuildingStamp
        // at 90) has already written its stamps; those are in pending when
        // we arrive. Clearing here discards them intentionally in Phase 2
        // (no real sources yet). When propagation lands in Phase 3+, this
        // system will run the propagation pass BEFORE clearing the outgoing
        // buffer and AFTER receiving stamps from priority-90 systems.
        resources.influence_grid.clear_all_pending();
        resources.influence_grid.swap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::influence::InfluenceChannel;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    fn empty_engine() -> SimEngine {
        SimEngine::new(64, 64, MaterialRegistry::new())
    }

    #[test]
    fn metadata() {
        let s = InfluenceUpdateSystem::new();
        assert_eq!(s.name(), "InfluenceUpdateSystem");
        assert_eq!(s.priority(), 100);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn tick_does_not_panic_on_empty_world() {
        let mut e = empty_engine();
        e.register_system(Box::new(InfluenceUpdateSystem::new()));
        for _ in 0..10 {
            e.tick();
        }
        assert_eq!(e.current_tick(), 10);
    }

    #[test]
    fn baseline_remains_zero_after_ticks() {
        let mut e = empty_engine();
        e.register_system(Box::new(InfluenceUpdateSystem::new()));
        for _ in 0..5 {
            e.tick();
        }
        for ch in InfluenceChannel::all() {
            assert_eq!(e.resources.influence_grid.sample(0, 0, *ch), 0);
            assert_eq!(e.resources.influence_grid.sample(32, 32, *ch), 0);
            assert_eq!(e.resources.influence_grid.sample(63, 63, *ch), 0);
        }
    }

    #[test]
    fn pending_write_cleared_before_swap() {
        // Write into pending, then tick: system clears pending first, then
        // swaps, so current receives the zeroed buffer (value is gone).
        let mut e = empty_engine();
        let buf = e
            .resources
            .influence_grid
            .pending_buf_mut(InfluenceChannel::Warmth);
        let i = 10 * 64 + 10;
        buf[i] = 200;
        let mut s = InfluenceUpdateSystem::new();
        s.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            e.resources.influence_grid.sample(10, 10, InfluenceChannel::Warmth),
            0
        );
    }
}
