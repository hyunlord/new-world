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
//! T7.10.B land: Light channel wired via recursive symmetric shadowcasting
//! (`propagate_shadowcast`, Adam Milazzo / Björn Bergström variant). BSS
//! already marks `dirty_regions[Light]` (STAMPED_CHANNELS includes Light),
//! so IUS drains those regions and runs the shadowcast pass into
//! `pending[Light]` before the swap. Light is Warm-tier in Phase 0
//! Section 2.6 (staggered scheduling is a future optimization); for
//! T7.10.B we propagate every tick and use the same persistence pattern
//! as Warmth to avoid flicker on event-less ticks. Aggregation is Max
//! (`InfluenceChannel::Light.aggregation()`); applied inside
//! `propagate_shadowcast`.
//!
//! T7.10.C land: Noise channel wired via linear decay BFS (`propagate_noise`,
//! Songs of Syx 2-tile ISSUE 2 v0.1.1 fix: linear alpha=15 + density-derived
//! wall blocking as orthogonal mechanisms). BSS now marks `dirty_regions[Noise]`
//! (Noise added to STAMPED_CHANNELS), so IUS drains and propagates Noise
//! linearly each tick. Noise is Hot-tier (channel.rs Tier::Hot, with Danger),
//! but for V7 Phase 2 (no agents yet) the building placement is the only
//! Noise source — we propagate every tick using the same persistence pattern
//! as Warmth/Light to avoid flicker. Aggregation is Max
//! (`InfluenceChannel::Noise.aggregation()`); applied inside `propagate_linear`.
//!
//! Other 5 channels remain dispatch-shell (clear pending + swap) and
//! will be wired in T7.10.D..F.
//!
//! Warmth: exponential k = 0.15 (Phase 0 Section 2.3.1, channel.rs:32).
//! Warmth max radius: 12 (Phase 0 Section 2.3.1 + ChannelDef fixture).
//! Warmth initial intensity: 200.
//! Warmth aggregation: Additive.
//!
//! Light: shadowcast falloff `intensity / (1 + 0.1 * d)`
//! (propagate.rs:249, Phase 0 Section 2.5.3 "gentle falloff").
//! Light max radius: 15 (longer reach than Warmth, FOV-style).
//! Light initial intensity: 200 (matches propagate.rs test conventions).
//! Light aggregation: Max.
//! Wall blocking: binary opaque via `TileGrid::is_wall`.
//!
//! Noise: linear decay `intensity - alpha` per BFS step, alpha=15
//! (propagate.rs `propagate_noise` wrapper; locked Songs of Syx 2-tile
//! ISSUE 2 v0.1.1 fix).
//! Noise initial intensity: 200 (parity with Warmth/Light for Max-aggregation visualisation).
//! Noise max radius: natural via alpha cutoff (200 / 15 ≈ 13 steps + intensity<5
//! exit). `propagate_noise` passes `u32::MAX` so decay self-terminates.
//! Noise aggregation: Max.
//! Wall blocking: density-derived via `MaterialBlockingCache` (4-neighbor BFS).

use hecs::World;
use sim_core::influence::{
    propagate_bfs, propagate_noise, propagate_shadowcast, InfluenceChannel,
};
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

/// Initial intensity at the Light source tile.
///
/// Matches propagate.rs shadowcast test conventions and provides
/// Warmth parity (same headroom for `Max` aggregation visualization).
const LIGHT_INITIAL_INTENSITY: u8 = 200;

/// Light shadowcast radius in tiles.
///
/// 15 = longer reach than Warmth's 12, consistent with Light being
/// Warm-tier (visual / FOV-style) vs Warmth being Cold-tier
/// (thermal source). `propagate_shadowcast` uses Euclidean
/// `dx*dx + dy*dy <= radius*radius` cutoff (propagate.rs:247).
const LIGHT_MAX_RADIUS: u32 = 15;

/// Initial intensity at the Noise source tile.
///
/// 200 matches Warmth/Light to give the same headroom for `Max`
/// aggregation visualisation. `propagate_noise` decays linearly by
/// alpha=15 per step (200 - 15*d), so the natural radius is
/// ~13 tiles before the intensity<5 cutoff terminates BFS.
const NOISE_INITIAL_INTENSITY: u8 = 200;

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
        let light_idx = InfluenceChannel::Light as usize;
        let noise_idx = InfluenceChannel::Noise as usize;

        // ── Warmth branch (T7.10.A) ──────────────────────────────────────────
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
                // box centered on event.position).
                let cx = (region.min_x + region.max_x) / 2;
                let cy = (region.min_y + region.max_y) / 2;

                // Borrow disjoint SimResources fields for the propagate_bfs call.
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
            let warmth_snapshot =
                resources.influence_grid.current[warmth_idx].clone();
            resources.influence_grid.pending[warmth_idx]
                .copy_from_slice(&warmth_snapshot);
        }

        // ── Light branch (T7.10.B) ───────────────────────────────────────────
        // Drain Light dirty regions left by BSS (Light is in STAMPED_CHANNELS).
        let light_dirty =
            std::mem::take(&mut resources.influence_grid.dirty_regions[light_idx]);

        if !light_dirty.is_empty() {
            // Fresh shadowcast pass: clear pending[Light] so multiple sources
            // accumulate (Max aggregation) onto a clean slate.
            resources.influence_grid.clear_pending(InfluenceChannel::Light);

            for region in &light_dirty {
                let cx = (region.min_x + region.max_x) / 2;
                let cy = (region.min_y + region.max_y) / 2;

                let tile_grid = &resources.tile_grid;
                let pending =
                    resources.influence_grid.pending_buf_mut(InfluenceChannel::Light);

                propagate_shadowcast(
                    tile_grid,
                    pending,
                    (cx, cy),
                    LIGHT_INITIAL_INTENSITY,
                    LIGHT_MAX_RADIUS,
                );
            }
        } else {
            // Warm-tier persistence (T7.10.B): mirror Cold-tier semantics so the
            // Light field survives event-less ticks. Phase 0 Warm-tier "staggered
            // 1 channel per tick" scheduling is deferred to a later optimization;
            // for T7.10.B we copy current[Light] → pending[Light] to avoid flicker.
            let light_snapshot =
                resources.influence_grid.current[light_idx].clone();
            resources.influence_grid.pending[light_idx]
                .copy_from_slice(&light_snapshot);
        }

        // ── Noise branch (T7.10.C) ───────────────────────────────────────────
        // Drain Noise dirty regions left by BSS (Noise is in STAMPED_CHANNELS).
        let noise_dirty =
            std::mem::take(&mut resources.influence_grid.dirty_regions[noise_idx]);

        if !noise_dirty.is_empty() {
            // Fresh linear-decay pass: clear pending[Noise] so multiple sources
            // accumulate (Max aggregation) onto a clean slate.
            resources.influence_grid.clear_pending(InfluenceChannel::Noise);

            for region in &noise_dirty {
                let cx = (region.min_x + region.max_x) / 2;
                let cy = (region.min_y + region.max_y) / 2;

                let tile_grid = &resources.tile_grid;
                let blocking_cache = &resources.material_blocking_cache;
                let pending =
                    resources.influence_grid.pending_buf_mut(InfluenceChannel::Noise);

                propagate_noise(
                    tile_grid,
                    blocking_cache,
                    pending,
                    (cx, cy),
                    NOISE_INITIAL_INTENSITY,
                );
            }
        } else {
            // Hot-tier persistence (T7.10.C, V7 Phase 2 no-agents): mirror Warm-tier
            // semantics so the Noise field survives event-less ticks. Without agents
            // generating transient acoustic events every tick, the building stamp is
            // the only Noise source — persistence keeps the disc stable for the
            // visualisation toggle. When agents arrive (Phase 3+) this branch can
            // be replaced with an empty pending (no persistence, true Hot-tier).
            let noise_snapshot =
                resources.influence_grid.current[noise_idx].clone();
            resources.influence_grid.pending[noise_idx]
                .copy_from_slice(&noise_snapshot);
        }

        // Other 5 channels: dispatch-shell baseline (clear pending → swap → zero).
        // They will be wired individually in T7.10.D..F.
        for ch in InfluenceChannel::all() {
            if *ch != InfluenceChannel::Warmth
                && *ch != InfluenceChannel::Light
                && *ch != InfluenceChannel::Noise
            {
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
