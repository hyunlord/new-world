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
//! T7.10.D land: Danger channel wired via linear decay BFS with sight-radius
//! cap (`propagate_danger`, Phase 0 v0.1.1 ISSUE 3 fix: linear alpha=5 +
//! max_radius=15 cap, no wall blocking — Danger pierces walls per the
//! "panic spreads through walls" design intent). BSS now marks
//! `dirty_regions[Danger]` (Danger added to STAMPED_CHANNELS), so IUS drains
//! and propagates Danger linearly each tick. Danger is Hot-tier (channel.rs
//! Tier::Hot), but for V7 Phase 2 (no agents yet) the building placement is
//! the only Danger source — we propagate every tick using the same persistence
//! pattern as Warmth/Light/Noise to avoid flicker. Aggregation is Max
//! (`InfluenceChannel::Danger.aggregation()`); applied inside `propagate_linear`.
//!
//! T7.10.E land: Spiritual channel wired via BFS exponential decay
//! (`propagate_bfs` with k=0.10 — gentler than Warmth's k=0.15 because
//! ritual influence carries further than thermal heat). BSS already
//! marks `dirty_regions[Spiritual]` (Spiritual was in STAMPED_CHANNELS since
//! T7.10.A; only IUS wiring was missing). IUS drains and propagates Spiritual
//! each tick. Spiritual is Cold-tier (channel.rs Tier::Cold, parity with
//! Warmth/Beauty), so for V7 Phase 2 we use the same persistence pattern as
//! Warmth/Light/Noise/Danger to avoid flicker on event-less ticks. Aggregation
//! is Max (`InfluenceChannel::Spiritual.aggregation()`); applied inside
//! `propagate_bfs`.
//!
//! T7.10.F land: Beauty channel wired via BFS exponential decay
//! (`propagate_bfs` with k=0.12 — locked at channel.rs:74 Phase 0 spec).
//! BSS already marks `dirty_regions[Beauty]` (Beauty was in STAMPED_CHANNELS
//! since T7.10.A; only IUS wiring was missing). IUS drains and propagates
//! Beauty each tick. Beauty is Cold-tier (channel.rs Tier::Cold, parity with
//! Warmth/Spiritual). With T7.10.F all 6 stamped channels (Warmth, Light,
//! Noise, Danger, Spiritual, Beauty) are wired — the Phase 2 stamped-channel
//! dispatch-shell escape is complete. Only the 2 unstamped channels
//! (FoodAroma, Social) remain on the dispatch shell.
//!
//! FoodAroma + Social are unstamped (no BSS dirty marking) and remain in
//! the dispatch-shell baseline (clear pending → swap → zero).
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
//!
//! Danger: linear decay `intensity - alpha` per BFS step, alpha=5 with hard
//! sight-radius cap=15 tiles (propagate.rs `propagate_danger` wrapper; locked
//! Phase 0 v0.1.1 ISSUE 3 fix preventing global panic spread).
//! Danger initial intensity: 200 (parity with Warmth/Light/Noise for Max-aggregation visualisation).
//! Danger max radius: 15 (hard cap — `propagate_danger` passes max_radius=15).
//! Danger aggregation: Max.
//! Wall blocking: NONE — Danger pierces walls (`propagate_danger` passes
//! `blocking_cache=None`).
//!
//! Spiritual: BFS exponential `intensity * exp(-k)` per step, k = 0.10
//! (gentler than Warmth's 0.15 — ritual influence carries further than
//! thermal heat).
//! Spiritual initial intensity: 200 (parity with Warmth/Light/Noise/Danger).
//! Spiritual max radius: 15 (longer reach than Warmth's 12, consistent with
//! a transcendent-source archetype).
//! Spiritual aggregation: Max.
//! Wall blocking: density-derived via `MaterialBlockingCache` (4-neighbor BFS,
//! same path as Warmth/Noise).
//!
//! Beauty: BFS exponential `intensity * exp(-k)` per step, k = 0.12
//! (channel.rs:74 Phase 0 spec — between Warmth's 0.15 and Spiritual's 0.10,
//! aesthetic appreciation reaches moderate distance).
//! Beauty initial intensity: 200 (parity with all other channels).
//! Beauty max radius: 15 (Spiritual parity, Cold-tier reach archetype).
//! Beauty aggregation: Max.
//! Wall blocking: density-derived via `MaterialBlockingCache` (4-neighbor BFS,
//! same path as Warmth/Noise/Spiritual).

use hecs::World;
use sim_core::influence::{
    propagate_bfs, propagate_danger, propagate_noise, propagate_shadowcast, InfluenceChannel,
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

/// Initial intensity at the Danger source tile.
///
/// 200 matches Warmth/Light/Noise for `Max`-aggregation visualisation
/// parity. `propagate_danger` decays linearly by alpha=5 per step
/// (200 - 5*d), with a hard max_radius=15 cap (Phase 0 ISSUE 3 fix
/// preventing global panic spread). At d=15 the intensity is
/// 200 - 5*15 = 125 — the cap stops propagation before the intensity<5
/// natural cutoff would fire (which would be at d=40).
const DANGER_INITIAL_INTENSITY: u8 = 200;

/// `exp(-0.10)` — spiritual exponential decay per BFS step.
///
/// Pre-computed: Rust does not const-eval `f32::exp` in stable.
/// k = 0.10 (gentler than Warmth's 0.15 — ritual influence carries further
/// than thermal heat).
const SPIRITUAL_DECAY_PER_STEP: f32 = 0.904_837;

/// Initial intensity at the Spiritual source tile.
const SPIRITUAL_INITIAL_INTENSITY: u8 = 200;

/// Spiritual propagation radius in tiles.
///
/// 15 = longer reach than Warmth's 12, consistent with a transcendent-source
/// archetype (parity with Light's 15-tile FOV reach).
const SPIRITUAL_MAX_RADIUS: u32 = 15;

/// `exp(-0.12)` — beauty exponential decay per BFS step.
///
/// Pre-computed: Rust does not const-eval `f32::exp` in stable.
/// k = 0.12 (channel.rs:74 Phase 0 spec — between Warmth's 0.15 and
/// Spiritual's 0.10).
const BEAUTY_DECAY_PER_STEP: f32 = 0.886_920;

/// Initial intensity at the Beauty source tile.
const BEAUTY_INITIAL_INTENSITY: u8 = 200;

/// Beauty propagation radius in tiles.
///
/// 15 = Spiritual parity (Cold-tier reach archetype, longer than Warmth's 12).
const BEAUTY_MAX_RADIUS: u32 = 15;

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
        let danger_idx = InfluenceChannel::Danger as usize;
        let spiritual_idx = InfluenceChannel::Spiritual as usize;
        let beauty_idx = InfluenceChannel::Beauty as usize;

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

        // ── Danger branch (T7.10.D) ──────────────────────────────────────────
        // Drain Danger dirty regions left by BSS (Danger is in STAMPED_CHANNELS).
        let danger_dirty =
            std::mem::take(&mut resources.influence_grid.dirty_regions[danger_idx]);

        if !danger_dirty.is_empty() {
            // Fresh linear-decay pass (alpha=5, cap=15, no walls): clear pending[Danger]
            // so multiple sources accumulate (Max aggregation) onto a clean slate.
            resources.influence_grid.clear_pending(InfluenceChannel::Danger);

            for region in &danger_dirty {
                let cx = (region.min_x + region.max_x) / 2;
                let cy = (region.min_y + region.max_y) / 2;

                let tile_grid = &resources.tile_grid;
                let pending =
                    resources.influence_grid.pending_buf_mut(InfluenceChannel::Danger);

                // propagate_danger locks Phase 0 v0.1.1 ISSUE 3 fix:
                //   alpha=5, max_radius=15, blocking_cache=None (pierces walls).
                propagate_danger(
                    tile_grid,
                    pending,
                    (cx, cy),
                    DANGER_INITIAL_INTENSITY,
                );
            }
        } else {
            // Hot-tier persistence (T7.10.D, V7 Phase 2 no-agents): mirror Noise
            // semantics so the Danger field survives event-less ticks. Without
            // agents generating transient threat events every tick, the building
            // stamp is the only Danger source — persistence keeps the disc stable
            // for the visualisation toggle. When agents arrive (Phase 3+) this
            // branch can be replaced with an empty pending (no persistence,
            // true Hot-tier).
            let danger_snapshot =
                resources.influence_grid.current[danger_idx].clone();
            resources.influence_grid.pending[danger_idx]
                .copy_from_slice(&danger_snapshot);
        }

        // ── Spiritual branch (T7.10.E) ───────────────────────────────────────
        // Drain Spiritual dirty regions left by BSS (Spiritual was in
        // STAMPED_CHANNELS since T7.10.A; only IUS propagation was missing).
        let spiritual_dirty =
            std::mem::take(&mut resources.influence_grid.dirty_regions[spiritual_idx]);

        if !spiritual_dirty.is_empty() {
            // Fresh BFS exponential pass: clear pending[Spiritual] so multiple
            // sources accumulate (Max aggregation) onto a clean slate.
            resources.influence_grid.clear_pending(InfluenceChannel::Spiritual);

            for region in &spiritual_dirty {
                let cx = (region.min_x + region.max_x) / 2;
                let cy = (region.min_y + region.max_y) / 2;

                let tile_grid = &resources.tile_grid;
                let blocking_cache = &resources.material_blocking_cache;
                let pending = resources
                    .influence_grid
                    .pending_buf_mut(InfluenceChannel::Spiritual);

                propagate_bfs(
                    tile_grid,
                    blocking_cache,
                    pending,
                    (cx, cy),
                    SPIRITUAL_INITIAL_INTENSITY,
                    |i, _| i * SPIRITUAL_DECAY_PER_STEP,
                    InfluenceChannel::Spiritual,
                    SPIRITUAL_MAX_RADIUS,
                );
            }
        } else {
            // Cold-tier persistence (T7.10.E, parity with Warmth): copy
            // current[Spiritual] → pending[Spiritual] so the swap is a no-op
            // for Spiritual on event-less ticks.
            let spiritual_snapshot =
                resources.influence_grid.current[spiritual_idx].clone();
            resources.influence_grid.pending[spiritual_idx]
                .copy_from_slice(&spiritual_snapshot);
        }

        // ── Beauty branch (T7.10.F) ──────────────────────────────────────────
        // Drain Beauty dirty regions left by BSS (Beauty was in STAMPED_CHANNELS
        // since T7.10.A; only IUS propagation was missing).
        let beauty_dirty =
            std::mem::take(&mut resources.influence_grid.dirty_regions[beauty_idx]);

        if !beauty_dirty.is_empty() {
            // Fresh BFS exponential pass: clear pending[Beauty] so multiple
            // sources accumulate (Max aggregation) onto a clean slate.
            resources.influence_grid.clear_pending(InfluenceChannel::Beauty);

            for region in &beauty_dirty {
                let cx = (region.min_x + region.max_x) / 2;
                let cy = (region.min_y + region.max_y) / 2;

                let tile_grid = &resources.tile_grid;
                let blocking_cache = &resources.material_blocking_cache;
                let pending = resources
                    .influence_grid
                    .pending_buf_mut(InfluenceChannel::Beauty);

                propagate_bfs(
                    tile_grid,
                    blocking_cache,
                    pending,
                    (cx, cy),
                    BEAUTY_INITIAL_INTENSITY,
                    |i, _| i * BEAUTY_DECAY_PER_STEP,
                    InfluenceChannel::Beauty,
                    BEAUTY_MAX_RADIUS,
                );
            }
        } else {
            // Cold-tier persistence (T7.10.F, parity with Warmth/Spiritual): copy
            // current[Beauty] → pending[Beauty] so the swap is a no-op for Beauty
            // on event-less ticks.
            let beauty_snapshot =
                resources.influence_grid.current[beauty_idx].clone();
            resources.influence_grid.pending[beauty_idx]
                .copy_from_slice(&beauty_snapshot);
        }

        // Remaining 2 unstamped channels (FoodAroma, Social): dispatch-shell
        // baseline (clear pending → swap → zero). With T7.10.F all 6 stamped
        // channels are wired; these unstamped channels stay cold until later
        // V7 phases wire agent-driven sources.
        for ch in InfluenceChannel::all() {
            if *ch != InfluenceChannel::Warmth
                && *ch != InfluenceChannel::Light
                && *ch != InfluenceChannel::Noise
                && *ch != InfluenceChannel::Danger
                && *ch != InfluenceChannel::Spiritual
                && *ch != InfluenceChannel::Beauty
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
