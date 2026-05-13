//! T7.10.F — Beauty channel BFS exponential-decay wiring (sixth-channel Phase 2 escape).
//!
//! T7.10.F is the SIXTH and FINAL stamped channel to escape the dispatch shell.
//! After T7.10.A (Warmth BFS k=0.15), T7.10.B (Light shadowcast), T7.10.C
//! (Noise linear-decay), T7.10.D (Danger linear-decay+cap), and T7.10.E
//! (Spiritual BFS k=0.08), the Beauty channel completes the 6/6 stamped-channel
//! wiring milestone. A BuildingPlacedEvent at (32, 32) with radius 12 must
//! produce a Beauty field in `current[Beauty]` centered at (32, 32) after one
//! tick, computed by BFS with exponential decay k=0.12 (Phase 0 channel.rs:74
//! lock — between Warmth's 0.15 and Spiritual's 0.08) and max_radius=15
//! (Spiritual parity, Cold-tier reach archetype).
//!
//! Setup note: 64×64 engine with empty MaterialRegistry (open field, no wall
//! blocking). All thresholds are LOCKED from plan — do NOT modify them.
//!
//! Run: `cargo test -p sim-test --test harness_t7_10_f_beauty_bfs_wiring -- --nocapture`

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

fn place_beauty_source(engine: &mut SimEngine) {
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (SX, SY),
        radius: 12,
    });
}

// ── Plan F1: source center reaches initial intensity ─────────────────────────

/// Type A: influence_grid.sample(SX, SY, Beauty) == 200 after 1 tick post-event.
///
/// Physical invariant. The BFS source tile must receive full
/// BEAUTY_INITIAL_INTENSITY (200) via apply_agg(Max) at the start of
/// propagation. Any value ≠ 200 means propagation never ran or the source
/// seeding is broken.
#[test]
fn harness_t7_10_f_source_center_lit() {
    let mut e = fresh_engine();
    place_beauty_source(&mut e);
    e.tick();
    // Type A: threshold == 200
    let v = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Beauty);
    assert_eq!(v, 200, "source center must equal BEAUTY_INITIAL_INTENSITY (200); got {v}");
}

// ── Plan F2: exp decay one cardinal step (k=0.12 discriminator) ──────────────

/// Type A: influence_grid.sample(SX+1, SY, Beauty) ∈ [175, 180] after 1 tick.
///
/// Mathematical invariant. Beauty decay k=0.12, BEAUTY_DECAY_PER_STEP =
/// exp(-0.12) ≈ 0.886920. Chain step: floor(200.0 * 0.886920) = 177.
/// Range [175, 180] adds ±3 tolerance for f32 rounding.
///
/// Discriminates against:
///   - Warmth k=0.15 → floor(200*0.8607)=172 — OUTSIDE [175,180]
///   - Spiritual k=0.08 → floor(200*0.9231)=184 — OUTSIDE [175,180]; F3
///     confirms via Manhattan d=2 chain ≈157 (vs Spiritual 170).
#[test]
fn harness_t7_10_f_exp_decay_one_step_discriminator() {
    let mut e = fresh_engine();
    place_beauty_source(&mut e);
    e.tick();
    // Type A: threshold ∈ [175, 180]
    let v = e.resources.influence_grid.sample(SX + 1, SY, InfluenceChannel::Beauty);
    assert!(
        (175..=180).contains(&v),
        "1-step cardinal neighbor must decay to ~177 (200*exp(-0.12)); got {v}. \
         Value ~172 = Warmth k=0.15 (wrong decay constant). \
         Value ~184 with d=2 chain ≈170 = Spiritual k=0.08 mis-wiring. \
         Below 175 or above 180 = wrong primitive or no decay applied."
    );
}

// ── Plan F3: BFS distance metric (diagonal at Manhattan d=2) ─────────────────

/// Type A: influence_grid.sample(SX+1, SY+1, Beauty) ∈ [155, 160] after 1 tick.
///
/// BFS distance invariant. Diagonal (SX+1, SY+1) reached in 2 cardinal steps
/// via 4-neighbor BFS: chain through (SX+1, SY) → (SX+1, SY+1).
/// 200 * 0.886920 = 177 → 177 * 0.886920 ≈ 157. Range [155, 160] gives ±3
/// tolerance for accumulated f32 truncation across the two-step chain.
///
/// Discriminates against:
///   - Euclidean (Light shadowcast) at d=sqrt(2): 200/(1+0.1*1.414) ≈ 175 —
///     OUTSIDE [155,160]. Proves BFS frontier.
///   - Spiritual k=0.08 two-step chain ≈ 170 — OUTSIDE [155,160]. Proves
///     Beauty's k=0.12 constant.
#[test]
fn harness_t7_10_f_bfs_distance_manhattan_discriminator() {
    let mut e = fresh_engine();
    place_beauty_source(&mut e);
    e.tick();
    // Type A: threshold ∈ [155, 160] (two-step chain ≈157)
    let v = e.resources.influence_grid.sample(SX + 1, SY + 1, InfluenceChannel::Beauty);
    assert!(
        (155..=160).contains(&v),
        "diagonal (SX+1,SY+1) must be in [155,160] (BFS d=2 chain ≈157); got {v}. \
         Value ~175 = Euclidean (wrong distance metric). \
         Value ~170 = Spiritual k=0.08 (wrong decay constant). \
         Outside [155,160] = wrong decay or BFS broken."
    );
}

// ── Plan F4: gradient monotonically strictly decreasing along cardinal axis ──

/// Type A: sample(SX+d, SY) > sample(SX+d+1, SY) for ALL d in 0..=14 (strict).
///
/// Mathematical invariant. Exponential decay with k=0.12 > 0 is strictly
/// monotone decreasing in continuous form. After u8 truncation, the minimum
/// inter-step gap (multiplicative chain by 0.886920) remains ≥ 1 across
/// d=0..=14 (the smallest expected sample near d=15 is ~200*0.8869^15 ≈ 33,
/// still gives ~4-unit drops). Strict `>` is safe for every adjacent pair.
/// Violation at any step means BFS frontier or decay function is broken.
#[test]
fn harness_t7_10_f_gradient_monotone() {
    let mut e = fresh_engine();
    place_beauty_source(&mut e);
    e.tick();
    for d in 0..=14u32 {
        let v1 = e.resources.influence_grid.sample(SX + d, SY, InfluenceChannel::Beauty);
        let v2 = e.resources.influence_grid.sample(SX + d + 1, SY, InfluenceChannel::Beauty);
        // Type A: threshold strict `>` for every adjacent pair
        assert!(
            v1 > v2,
            "gradient not strictly decreasing at d={d}: sample(SX+{d}) = {v1} <= sample(SX+{}) = {v2}. \
             Beauty k=0.12 must yield strictly decreasing chain across d=0..=14.",
            d + 1
        );
    }
}

// ── Plan F5: boundary tile at max_radius is nonzero ──────────────────────────

/// Type A: influence_grid.sample(SX+15, SY, Beauty) ∈ [28, 38] after 1 tick.
///
/// Mathematical invariant. The BFS must reach max_radius=15 along the cardinal
/// axis. Chain end at d=15: 200 * 0.8869^15 ≈ 200 * 0.1653 ≈ 33.1. After
/// u8 truncation across 15 chained f32 multiplications: range [28, 38] gives
/// ±5 margin for accumulated rounding.
///
/// Value == 0 means BFS stopped before max_radius (wrong max_radius constant).
/// Value > 38 means wrong decay constant (Spiritual's k=0.08 would give ≈60 —
/// OUTSIDE [28,38]).
#[test]
fn harness_t7_10_f_boundary_at_max_radius() {
    let mut e = fresh_engine();
    place_beauty_source(&mut e);
    e.tick();
    // Type A: threshold ∈ [28, 38]
    let v = e.resources.influence_grid.sample(SX + 15, SY, InfluenceChannel::Beauty);
    assert!(
        (28..=38).contains(&v),
        "tile at max_radius=15 must be in [28,38] (chain end ≈33); got {v}; \
         value==0 means propagation stopped before max_radius=15. \
         Value ~60 = Spiritual k=0.08 (wrong decay constant). \
         Value ~21 = Warmth k=0.15 (wrong decay constant)."
    );
}

// ── Plan F6: beyond max_radius is zero ───────────────────────────────────────

/// Type A: influence_grid.sample(SX+16, SY, Beauty) == 0 after 1 tick.
///
/// Hard cutoff invariant. max_radius=15 is enforced by BFS termination
/// (propagate_bfs at propagate.rs:75 uses `dist >= max_radius` cap). Tile at
/// Manhattan distance 16 must NEVER be written.
#[test]
fn harness_t7_10_f_max_radius_cutoff() {
    let mut e = fresh_engine();
    place_beauty_source(&mut e);
    e.tick();
    // Type A: threshold == 0 (distance 16 from source, one beyond max_radius=15)
    let v = e.resources.influence_grid.sample(SX + 16, SY, InfluenceChannel::Beauty);
    assert_eq!(v, 0, "tile beyond max_radius=15 must be 0; got {v}");
}

// ── Plan F7: cold-tier persistence across 10 event-less ticks ────────────────

/// Type A: source center still == 200 after 1 stamp tick + 10 event-less ticks.
///
/// Cold-tier event-driven semantics. Beauty is UpdateTier::Cold — it must
/// persist indefinitely between placement events. If the persistence branch
/// (copy current→pending before swap) is missing, the disc flickers to 0
/// on every event-less tick.
#[test]
fn harness_t7_10_f_persistence_ten_ticks() {
    let mut e = fresh_engine();
    place_beauty_source(&mut e);
    e.tick(); // tick 1: stamp + propagation
    for _ in 0..10 {
        e.tick(); // ticks 2–11: event-less persistence branch
    }
    // Type A: threshold == 200 (source persists through all 10 idle ticks)
    let v = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Beauty);
    assert_eq!(
        v, 200,
        "source must persist across 10 event-less ticks (Cold-tier semantics); got {v}. \
         Persistence branch (current→pending copy) is broken or missing."
    );
}

// ── Plan F8: no event = no beauty ────────────────────────────────────────────

/// Type A: Beauty == 0 at (0,0), (SX,SY), (W-1,H-1) after 5 ticks, no events.
///
/// Without any BuildingPlacedEvent, dirty_regions[Beauty] stays empty every
/// tick. Persistence branch copies current (all zeros) → pending → swap → 0.
/// Any nonzero value means uninitialized buffer state leaking through.
#[test]
fn harness_t7_10_f_no_event_no_beauty() {
    let mut e = fresh_engine();
    for _ in 0..5 {
        e.tick();
    }
    for (x, y) in [(0u32, 0u32), (SX, SY), (W - 1, H - 1)] {
        // Type A: threshold == 0 at all three positions with no events
        let v = e.resources.influence_grid.sample(x, y, InfluenceChannel::Beauty);
        assert_eq!(v, 0, "Beauty at ({x},{y}) must be 0 with no events; got {v}");
    }
}

// ── Plan F9: T7.10.A/B/C/D/E regression — prior channels survive Beauty wiring

/// Type A: Warmth/Light/Noise/Danger/Spiritual all == 200 at source after 1 tick.
///
/// Cross-channel regression guard. T7.10.F adds the Beauty branch to IUS;
/// the Warmth (T7.10.A), Light (T7.10.B), Noise (T7.10.C), Danger (T7.10.D),
/// and Spiritual (T7.10.E) branches must still produce their respective
/// discs. A Generator who accidentally re-orders or short-circuits prior
/// channels when adding Beauty would FAIL this guard.
#[test]
fn harness_t7_10_f_prior_channels_regression_guard() {
    let mut e = fresh_engine();
    place_beauty_source(&mut e);
    e.tick();
    // Type A: threshold == 200 for all 5 previously-wired channels' source centers
    let warmth = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Warmth);
    assert_eq!(
        warmth, 200,
        "T7.10.A regression: Warmth at source must remain 200; got {warmth}"
    );
    let light = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Light);
    assert_eq!(
        light, 200,
        "T7.10.B regression: Light at source must remain 200; got {light}"
    );
    let noise = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Noise);
    assert_eq!(
        noise, 200,
        "T7.10.C regression: Noise at source must remain 200; got {noise}"
    );
    let danger = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Danger);
    assert_eq!(
        danger, 200,
        "T7.10.D regression: Danger at source must remain 200; got {danger}"
    );
    let spiritual = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Spiritual);
    assert_eq!(
        spiritual, 200,
        "T7.10.E regression: Spiritual at source must remain 200; got {spiritual}"
    );
}

// ── Plan F10: dirty_regions[Beauty] drained by IUS via std::mem::take ────────

/// Type A: dirty_regions[Beauty].len() == 0 after one full engine.tick() with
/// a BuildingPlacedEvent.
///
/// IUS drain invariant. T7.10.F uses `std::mem::take` on the Beauty dirty
/// regions, matching T7.10.A/B/C/D/E semantics. A Generator who reads via
/// `iter()` or `clone()` without draining would leave the entries in place.
#[test]
fn harness_t7_10_f_dirty_regions_drained() {
    let mut e = fresh_engine();
    place_beauty_source(&mut e);
    e.tick();
    // Type A: threshold == 0
    let len =
        e.resources.influence_grid.dirty_regions[InfluenceChannel::Beauty as usize].len();
    assert_eq!(
        len, 0,
        "dirty_regions[Beauty] must be drained by IUS via std::mem::take; got len={len}"
    );
}

// ── Plan F11: unstamped channels (FoodAroma/Social) stay zero ────────────────

/// Type A: FoodAroma and Social sample to 0 at source after 1 tick.
///
/// T7.10.F wires the 6th and final stamped channel (Beauty). The remaining
/// 2 unstamped channels (FoodAroma, Social) have NO BSS dirty marking,
/// so they stay completely cold. This is the final-state invariant: post-
/// T7.10.F there are NO stamped-but-dispatch-shell channels — only the
/// 2 unstamped channels remain dispatch-shell.
#[test]
fn harness_t7_10_f_unstamped_channels_remain_zero() {
    let mut e = fresh_engine();
    place_beauty_source(&mut e);
    e.tick();
    // Unstamped channels (BSS never marks dirty).
    for ch in [InfluenceChannel::FoodAroma, InfluenceChannel::Social] {
        let v = e.resources.influence_grid.sample(SX, SY, ch);
        assert_eq!(v, 0, "{ch:?} must remain zero (not stamped); got {v}");
    }
}
