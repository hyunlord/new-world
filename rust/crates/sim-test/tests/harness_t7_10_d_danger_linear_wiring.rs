//! T7.10.D — Danger channel linear-decay+cap wiring (fourth-channel Phase 2 escape).
//!
//! After T7.10.A (Warmth BFS), T7.10.B (Light shadowcast), and T7.10.C (Noise
//! linear-decay), the Danger channel is the fourth to escape the dispatch
//! shell. A BuildingPlacedEvent at (32, 32) with radius 12 must produce a
//! Danger field in `current[Danger]` centered at (32, 32) after one tick,
//! computed by linear-decay BFS with alpha=5 per step AND a hard sight-radius
//! cap of 15 tiles (Phase 0 ISSUE 3 fix via `propagate_danger`, wraps
//! `propagate_linear` with max_radius=15 and blocking_cache=None — Danger
//! pierces walls).
//!
//! Setup note: 64×64 engine with empty MaterialRegistry (open field, no
//! wall blocking would apply anyway since Danger ignores walls). All
//! thresholds are LOCKED from plan — do NOT modify them.
//!
//! Run: `cargo test -p sim-test --test harness_t7_10_d_danger_linear_wiring -- --nocapture`

use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, SimEngine};
use sim_systems::register_phase2_systems;

const W: u32 = 64;
const H: u32 = 64;
const SX: u32 = 32;
const SY: u32 = 32;

fn fresh_engine() -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    engine
}

fn place_danger_source(engine: &mut SimEngine) {
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (SX, SY),
        radius: 12,
    });
}

// ── Plan D1: source center reaches initial intensity ─────────────────────────

/// Type A: influence_grid.sample(SX, SY, Danger) == 200 after 1 tick post-event.
///
/// Physical invariant. The linear-decay source tile must receive full
/// DANGER_INITIAL_INTENSITY (200) via apply_agg(Max) at the start of
/// propagation (propagate.rs:358). Any value ≠ 200 means propagation never
/// ran or the source seeding is broken.
#[test]
fn harness_t7_10_d_source_center_lit() {
    let mut e = fresh_engine();
    place_danger_source(&mut e);
    e.tick();
    // Type A: threshold == 200
    let v = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Danger);
    assert_eq!(v, 200, "source center must equal DANGER_INITIAL_INTENSITY (200); got {v}");
}

// ── Plan D2: linear decay one cardinal step (alpha=5 discriminator) ──────────

/// Type A: influence_grid.sample(SX+1, SY, Danger) == 195 after 1 tick.
///
/// Mathematical invariant. Linear decay alpha=5 per BFS step (Phase 0
/// ISSUE 3 fix locked in `propagate_danger`). At BFS d=1: 200 - 5 = 195 exact.
/// No tolerance — integer arithmetic, no f32.
///
/// Discriminates against Noise's alpha=15 (would give 185) and against
/// shadowcast falloff 200/(1+0.1*1)=181. Proves linear-decay alpha=5 path
/// is wired, not Noise's or Light's primitive.
#[test]
fn harness_t7_10_d_linear_one_step_alpha_discriminator() {
    let mut e = fresh_engine();
    place_danger_source(&mut e);
    e.tick();
    // Type A: threshold == 195 (exact)
    let v = e.resources.influence_grid.sample(SX + 1, SY, InfluenceChannel::Danger);
    assert_eq!(
        v, 195,
        "1-step cardinal neighbor must equal 200-5=195 (linear alpha=5); got {v}. \
         Value 185 = Noise alpha=15 (wrong primitive). \
         Value 181 = shadowcast falloff (wrong primitive). \
         Other value = wrong alpha or propagation never ran."
    );
}

// ── Plan D3: BFS distance metric (diagonal at Manhattan d=2) ─────────────────

/// Type A: influence_grid.sample(SX+1, SY+1, Danger) == 190 after 1 tick.
///
/// BFS distance invariant. Diagonal (SX+1, SY+1) reached in 2 cardinal steps
/// via 4-neighbor BFS: 200 - 2*5 = 190. NO Euclidean shortcut — BFS uses
/// Manhattan-like distance under 4-neighborhood expansion.
///
/// Discriminates against Euclidean 200 - 5*sqrt(2) ≈ 192.9 (would round to
/// 193). Proves linear-decay uses BFS frontier, not Euclidean distance.
#[test]
fn harness_t7_10_d_bfs_distance_manhattan_discriminator() {
    let mut e = fresh_engine();
    place_danger_source(&mut e);
    e.tick();
    // Type A: threshold == 190 (200 - 2*5)
    let v = e.resources.influence_grid.sample(SX + 1, SY + 1, InfluenceChannel::Danger);
    assert_eq!(
        v, 190,
        "diagonal (SX+1,SY+1) must equal 190 (BFS d=2, 200-2*5); got {v}. \
         Value ≈193 = Euclidean (wrong distance metric). \
         Other value = wrong alpha or BFS broken."
    );
}

// ── Plan D4: gradient monotonically strictly decreasing along cardinal axis ──

/// Type A: sample(SX+d, SY) > sample(SX+d+1, SY) for ALL d in 0..=14 (strict).
///
/// Mathematical invariant. Linear decay alpha=5 yields constant 5-unit step
/// drops across the cap radius: 200, 195, 190, ..., 130, 125. Adjacent pairs
/// always differ by exactly 5 (no truncation ties) so strict `>` is safe
/// across d=0..=14. Violation at any step means BFS frontier or alpha
/// decrement is broken.
#[test]
fn harness_t7_10_d_gradient_monotone() {
    let mut e = fresh_engine();
    place_danger_source(&mut e);
    e.tick();
    for d in 0..=14u32 {
        let v1 = e.resources.influence_grid.sample(SX + d, SY, InfluenceChannel::Danger);
        let v2 = e.resources.influence_grid.sample(SX + d + 1, SY, InfluenceChannel::Danger);
        // Type A: threshold strict `>` for every adjacent pair
        assert!(
            v1 > v2,
            "gradient not strictly decreasing at d={d}: sample(SX+{d}) = {v1} <= sample(SX+{}) = {v2}. \
             Linear alpha=5 must yield exact 5-unit drops up to the cap.",
            d + 1
        );
    }
}

// ── Plan D5: sight-radius cap boundary (Phase 0 ISSUE 3 fix discriminator) ───

/// Type A: sample(SX+15, SY, Danger) == 125 AND sample(SX+16, SY, Danger) == 0.
///
/// Cap invariant. propagate_danger passes max_radius=15 to propagate_linear.
/// propagate.rs:380 caps BFS expansion at `dist >= max_radius` (does not
/// queue further). At BFS d=15: 200 - 15*5 = 125 — the cap allows this tile
/// to be written but blocks d=16. At d=16: NOT reached, tile remains 0.
///
/// Discriminates: Noise (no cap) at d=16 would give 200-16*15=neg→saturating
/// then NOT pass intensity<5 check at d=14 (200-14*15=−10→0), so Noise would
/// be 0 at d=14 already. A Generator who omits the cap (uses propagate_linear
/// with u32::MAX) would write d=16=120 instead of 0. A Generator who uses
/// the wrong cap (e.g. 14 or 16) would shift the boundary by one tile.
#[test]
fn harness_t7_10_d_sight_radius_cap_boundary() {
    let mut e = fresh_engine();
    place_danger_source(&mut e);
    e.tick();
    // Type A: threshold == 125 at d=15 (cap allows this tile)
    let v15 = e.resources.influence_grid.sample(SX + 15, SY, InfluenceChannel::Danger);
    assert_eq!(
        v15, 125,
        "Danger at d=15 must equal 200-15*5=125 (cap=15 allows this tile); got {v15}. \
         Value 0 = cap too tight (max_radius<15). \
         Value ≠125 = wrong alpha."
    );
    // Type A: threshold == 0 at d=16 (cap stops propagation)
    let v16 = e.resources.influence_grid.sample(SX + 16, SY, InfluenceChannel::Danger);
    assert_eq!(
        v16, 0,
        "Danger at d=16 must be 0 (cap=15 blocks expansion past d=15); got {v16}. \
         Value 120 = no cap (would be 200-16*5=120 without cap). \
         Other nonzero value = cap implemented but wrong threshold."
    );
}

// ── Plan D6: persistence — Danger survives 10 event-less ticks ───────────────

/// Type A: sample(SX, SY, Danger) == 200 after the single event tick + 9 more
/// event-less ticks (total 10 ticks).
///
/// Persistence invariant (T7.10.D update.rs). On event-less ticks the
/// Hot-tier branch copies `current[Danger]` into `pending[Danger]` so the
/// swap is a no-op, preventing flicker. A Generator who omits persistence
/// would see Danger zero out after tick 2.
#[test]
fn harness_t7_10_d_persistence_ten_ticks() {
    let mut e = fresh_engine();
    place_danger_source(&mut e);
    for _ in 0..10 {
        e.tick();
    }
    // Type A: threshold == 200 at source after 10 ticks
    let v = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Danger);
    assert_eq!(
        v, 200,
        "source Danger must persist == 200 after 10 ticks (1 event + 9 event-less); got {v}. \
         Persistence branch (current→pending copy) is broken or missing."
    );
}

// ── Plan D7: no event = no danger ────────────────────────────────────────────

/// Type A: Danger == 0 at (0,0), (SX,SY), (W-1,H-1) after 5 ticks, no events.
///
/// Without any BuildingPlacedEvent, dirty_regions[Danger] stays empty every
/// tick. Persistence branch copies current (all zeros) → pending → swap → 0.
/// Any nonzero value means uninitialized buffer state leaking through.
#[test]
fn harness_t7_10_d_no_event_no_danger() {
    let mut e = fresh_engine();
    for _ in 0..5 {
        e.tick();
    }
    for (x, y) in [(0u32, 0u32), (SX, SY), (W - 1, H - 1)] {
        // Type A: threshold == 0 at all three positions with no events
        let v = e.resources.influence_grid.sample(x, y, InfluenceChannel::Danger);
        assert_eq!(v, 0, "Danger at ({x},{y}) must be 0 with no events; got {v}");
    }
}

// ── Plan D8: T7.10.A/B/C regression — Warmth+Light+Noise survive Danger wiring

/// Type A: Warmth(SX,SY) == 200 AND Light(SX,SY) == 200 AND Noise(SX,SY) == 200
/// after 1 tick post-event.
///
/// Cross-channel regression guard. T7.10.D adds the Danger branch to IUS;
/// the Warmth (T7.10.A), Light (T7.10.B), and Noise (T7.10.C) branches must
/// still produce their respective discs. A Generator who accidentally
/// re-orders or short-circuits Warmth/Light/Noise when adding Danger would
/// FAIL this guard.
#[test]
fn harness_t7_10_d_warmth_light_noise_regression_guard() {
    let mut e = fresh_engine();
    place_danger_source(&mut e);
    e.tick();
    // Type A: threshold == 200 for Warmth source center
    let warmth = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Warmth);
    assert_eq!(
        warmth, 200,
        "T7.10.A regression: Warmth at source must remain 200; got {warmth}"
    );
    // Type A: threshold == 200 for Light source center
    let light = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Light);
    assert_eq!(
        light, 200,
        "T7.10.B regression: Light at source must remain 200; got {light}"
    );
    // Type A: threshold == 200 for Noise source center
    let noise = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Noise);
    assert_eq!(
        noise, 200,
        "T7.10.C regression: Noise at source must remain 200; got {noise}"
    );
}

// ── Plan D9: dirty_regions[Danger] drained by IUS via std::mem::take ────────

/// Type A: dirty_regions[Danger].len() == 0 after one full engine.tick() with
/// a BuildingPlacedEvent.
///
/// IUS drain invariant. T7.10.D uses `std::mem::take` on the Danger dirty
/// regions, matching T7.10.A/B/C semantics. A Generator who reads via
/// `iter()` or `clone()` without draining would leave the entries in place.
#[test]
fn harness_t7_10_d_dirty_regions_drained() {
    let mut e = fresh_engine();
    place_danger_source(&mut e);
    e.tick();
    // Type A: threshold == 0
    let len =
        e.resources.influence_grid.dirty_regions[InfluenceChannel::Danger as usize].len();
    assert_eq!(
        len, 0,
        "dirty_regions[Danger] must be drained by IUS via std::mem::take; got len={len}"
    );
}

// ── Plan D10: other-channels behavior (Beauty stays 0, FoodAroma/Social stay 0)

/// Type A: T7.10.E regression — Spiritual sample == 200 at source;
/// dispatch-shell stamped channel Beauty samples to 0;
/// unstamped channels FoodAroma/Social sample to 0.
///
/// T7.10.D wired Warmth+Light+Noise+Danger. T7.10.E additionally wires
/// Spiritual via BFS exp k=0.10. The remaining stamped channel (Beauty) has
/// BSS dirty_regions but IUS does not yet propagate it. Unstamped channels
/// (FoodAroma, Social) stay completely cold.
#[test]
fn harness_t7_10_d_other_channels_remain_zero() {
    let mut e = fresh_engine();
    place_danger_source(&mut e);
    e.tick();
    // T7.10.E regression guard: Spiritual now propagates at source center.
    let spiritual = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Spiritual);
    assert_eq!(
        spiritual, 200,
        "T7.10.E: Spiritual at source must be 200 (BFS exp k=0.10 propagation); got {spiritual}"
    );
    // T7.10.F regression guard: Beauty now propagates at source center.
    let beauty = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Beauty);
    assert_eq!(
        beauty, 200,
        "T7.10.F: Beauty at source must be 200 (BFS exp k=0.12 propagation); got {beauty}"
    );
    // Unstamped channels (BSS never marks dirty) — only 2 remain post-T7.10.F.
    for ch in [InfluenceChannel::FoodAroma, InfluenceChannel::Social] {
        let v = e.resources.influence_grid.sample(SX, SY, ch);
        assert_eq!(v, 0, "{ch:?} must remain zero (not stamped); got {v}");
    }
}
