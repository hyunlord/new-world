//! `InfluenceUpdateSystem` — priority 100, every tick.
//!
//! Phase 0 Section 2.6 budget: Hot tier ≤ 0.5 ms @ 1K agents.
//!
//! T7.10.A land: Warmth channel escapes the Phase 2 dispatch shell.
//! On any tick where BuildingStampSystem (priority 90) populated
//! `dirty_regions[Warmth]`, IUS runs `propagate_bfs` from each region's
//! center into `pending[Warmth]` before the swap. Persistence across
//! event-less ticks is preserved by copying `current[Warmth]` into
//! `pending[Warmth]` so the swap is a no-op for Warmth (Cold-tier
//! event-driven semantics, per Phase 0 Section 2.6).
//!
//! Other 7 channels remain dispatch-shell (clear pending + swap) and
//! will be wired in T7.10.B..F.
//!
//! Decay: exponential k = 0.15 (Phase 0 Section 2.3.1, channel.rs:32).
//! Max radius: 12 (Phase 0 Section 2.3.1 + ChannelDef fixture).
//! Initial intensity: 200 (matches existing propagate.rs test conventions).
//! Aggregation: Additive (InfluenceChannel::Warmth.aggregation()).

use hecs::World;
use sim_core::influence::{propagate_bfs, InfluenceChannel};
use sim_engine::{RuntimeSystem, SimResources};

/// `exp(-0.15)` — warmth exponential decay per BFS step.
///
/// Pre-computed: Rust does not const-eval `f32::exp` in stable.
/// k = 0.15 from Phase 0 Section 2.3.1 (channel.rs:32 comment).
const WARMTH_DECAY_PER_STEP: f32 = 0.860_708;

/// Initial intensity stamped at the building's center tile.
const WARMTH_INITIAL_INTENSITY: u8 = 200;

/// Warmth propagation radius in tiles (Phase 0 Section 2.3.1).
const WARMTH_MAX_RADIUS: u32 = 12;

/// Phase 2 influence update dispatcher — T7.10.A: Warmth channel wired.
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
        let warmth_idx = InfluenceChannel::Warmth as usize;

        // Drain Warmth dirty regions left by BuildingStampSystem (priority 90).
        // std::mem::take replaces with an empty Vec — this IS the drain (Cold-tier
        // event-driven semantics require consuming the regions, not just reading them).
        let warmth_dirty =
            std::mem::take(&mut resources.influence_grid.dirty_regions[warmth_idx]);

        if !warmth_dirty.is_empty() {
            // Fresh BFS pass for this tick: clear pending[Warmth] first so multiple
            // calls to propagate_bfs accumulate (Additive aggregation) onto a clean slate.
            resources.influence_grid.clear_pending(InfluenceChannel::Warmth);

            for region in &warmth_dirty {
                // Use dirty region center as the BFS source. For a single-building event
                // the center equals the building's tile position (BSS stamps a Chebyshev
                // box centered on event.position). Multi-source accuracy lands in T7.10.B.
                let cx = (region.min_x + region.max_x) / 2;
                let cy = (region.min_y + region.max_y) / 2;

                // Borrow disjoint SimResources fields for the propagate_bfs call.
                // Rust 2021 NLL handles disjoint field borrows: tile_grid and
                // material_blocking_cache are separate fields from influence_grid.
                let tile_grid = &resources.tile_grid;
                let blocking_cache = &resources.material_blocking_cache;
                let pending =
                    resources.influence_grid.pending_buf_mut(InfluenceChannel::Warmth);

                propagate_bfs(
                    tile_grid,
                    blocking_cache,
                    pending,
                    (cx, cy),
                    WARMTH_INITIAL_INTENSITY,
                    |i, _| i * WARMTH_DECAY_PER_STEP,
                    InfluenceChannel::Warmth,
                    WARMTH_MAX_RADIUS,
                );
            }
        } else {
            // Cold-tier persistence: no building events this tick, so the Warmth
            // disc must persist. Copy current[Warmth] into pending[Warmth] so the
            // swap below is a no-op for Warmth. Without this, every event-less tick
            // would zero current[Warmth] causing the rendered disc to flicker.
            //
            // Safe approach (no unsafe needed): clone the read side, then write the
            // clone into the write side. The clone is small (64*64 = 4096 bytes).
            let warmth_snapshot =
                resources.influence_grid.current[warmth_idx].clone();
            resources.influence_grid.pending[warmth_idx]
                .copy_from_slice(&warmth_snapshot);
        }

        // Other 7 channels: dispatch-shell baseline (clear pending → swap → zero).
        // They will be wired individually in T7.10.B..F.
        for ch in InfluenceChannel::all() {
            if *ch != InfluenceChannel::Warmth {
                resources.influence_grid.clear_pending(*ch);
            }
        }

        // Swap all 8 channels together (single atomic step per tick).
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
        // No building events → dirty_regions[Warmth] empty → persistence branch
        // copies current[Warmth] (all zeros) → pending → swap → still zero.
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
        // T7.10.A semantics: manually write pending[Warmth][10,10] = 200 WITHOUT
        // adding a dirty_region. Then tick IUS directly.
        //
        // T7.10.A path: dirty_regions[Warmth] is empty → persistence branch fires.
        // Persistence branch copies current[Warmth] (all zeros, never propagated to)
        // into pending[Warmth], overwriting the manually written 200.
        // After swap: current[Warmth][10,10] = 0. Same observable result as the
        // Phase 2 dispatch-shell, but via a different code path.
        //
        // Original Phase 2 comment preserved:
        // "Write into pending, then tick: system clears pending first, then
        //  swaps, so current receives the zeroed buffer (value is gone)."
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
