//! T7.10.E — Spiritual channel BFS exponential-decay wiring (fifth-channel Phase 2 escape).
//!
//! After T7.10.A (Warmth BFS), T7.10.B (Light shadowcast), T7.10.C (Noise
//! linear-decay), and T7.10.D (Danger linear-decay+cap), the Spiritual channel
//! is the fifth to escape the dispatch shell. A BuildingPlacedEvent at (32, 32)
//! with radius 12 must produce a Spiritual field in `current[Spiritual]`
//! centered at (32, 32) after one tick, computed by BFS with exponential decay
//! k=0.08 (channel.rs:69 Phase 0 spec — gentler than Warmth's k=0.15) and
//! max_radius=15 (longer reach than Warmth's 12, matching the ritual-influence
//! archetype).
//!
//! Setup note: 64×64 engine with empty MaterialRegistry (open field, no wall
//! blocking). All thresholds are LOCKED from plan — do NOT modify them.
//!
//! Run: `cargo test -p sim-test --test harness_t7_10_e_spiritual_bfs_wiring -- --nocapture`

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

fn place_spiritual_source(engine: &mut SimEngine) {
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (SX, SY),
        radius: 12,
    });
}

// ── Plan E1: source center reaches initial intensity ─────────────────────────

/// Type A: influence_grid.sample(SX, SY, Spiritual) == 200 after 1 tick post-event.
///
/// Physical invariant. The BFS source tile must receive full
/// SPIRITUAL_INITIAL_INTENSITY (200) via apply_agg(Max) at the start of
/// propagation. Any value ≠ 200 means propagation never ran or the source
/// seeding is broken.
#[test]
fn harness_t7_10_e_source_center_lit() {
    let mut e = fresh_engine();
    place_spiritual_source(&mut e);
    e.tick();
    // Type A: threshold == 200
    let v = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Spiritual);
    assert_eq!(v, 200, "source center must equal SPIRITUAL_INITIAL_INTENSITY (200); got {v}");
}

// ── Plan E2: exp decay one cardinal step (k=0.08 discriminator vs Warmth k=0.15)

/// Type A: influence_grid.sample(SX+1, SY, Spiritual) ∈ [182, 186] after 1 tick.
///
/// Mathematical invariant. Spiritual decay k=0.08 (channel.rs:69 Phase 0 spec),
/// SPIRITUAL_DECAY_PER_STEP = exp(-0.08) ≈ 0.923116. Chain step:
/// floor(200.0 * 0.923116) = 184. Range [182, 186] adds ±2 tolerance for f32
/// rounding.
///
/// Discriminates against Warmth's k=0.15 (would give floor(200*0.8607)=172 —
/// OUTSIDE [182,186]) and the prior k=0.10 drift (would give floor(200*0.9048)
/// =180 — OUTSIDE [182,186]). Also discriminates against Light's shadowcast
/// falloff 200/(1+0.1)=181 — Spiritual uses BFS (Manhattan distance) not
/// Euclidean, so Plan E3 nails the remaining difference.
#[test]
fn harness_t7_10_e_exp_decay_one_step_discriminator() {
    let mut e = fresh_engine();
    place_spiritual_source(&mut e);
    e.tick();
    // Type A: threshold ∈ [182, 186]
    let v = e.resources.influence_grid.sample(SX + 1, SY, InfluenceChannel::Spiritual);
    assert!(
        (182..=186).contains(&v),
        "1-step cardinal neighbor must decay to ~184 (200*exp(-0.08)); got {v}. \
         Value ~180 = stale k=0.10 (T7.10.E pre-fix drift). \
         Value ~172 = Warmth k=0.15 (wrong decay constant). \
         Below 182 or above 186 = wrong primitive or no decay applied."
    );
}

// ── Plan E3: BFS distance metric (diagonal at Manhattan d=2) ─────────────────

/// Type A: influence_grid.sample(SX+1, SY+1, Spiritual) ∈ [168, 172] after 1 tick.
///
/// BFS distance invariant. Diagonal (SX+1, SY+1) reached in 2 cardinal steps
/// via 4-neighbor BFS: chain through (SX+1, SY) → (SX+1, SY+1).
/// 200 * 0.923116 ≈ 184.6 → 184.6 * 0.923116 ≈ 170.5. Range [168, 172] gives
/// ±2 tolerance for accumulated f32 truncation across the two-step chain.
///
/// Discriminates against Euclidean (Light) at d=sqrt(2): 200/(1+0.1*1.414) ≈
/// 175 — OUTSIDE [168,172]. Proves BFS frontier, not Euclidean distance. Also
/// discriminates against stale k=0.10 chain ≈ 163 — OUTSIDE [168,172].
#[test]
fn harness_t7_10_e_bfs_distance_manhattan_discriminator() {
    let mut e = fresh_engine();
    place_spiritual_source(&mut e);
    e.tick();
    // Type A: threshold ∈ [168, 172] (two-step chain ≈170)
    let v = e.resources.influence_grid.sample(SX + 1, SY + 1, InfluenceChannel::Spiritual);
    assert!(
        (168..=172).contains(&v),
        "diagonal (SX+1,SY+1) must be in [168,172] (BFS d=2 chain ≈170 with k=0.08); got {v}. \
         Value ~163 = stale k=0.10 (T7.10.E pre-fix drift). \
         Value ~175 = Euclidean (wrong distance metric). \
         Outside [168,172] = wrong decay or BFS broken."
    );
}

// ── Plan E4: gradient monotonically strictly decreasing along cardinal axis ──

/// Type A: sample(SX+d, SY) > sample(SX+d+1, SY) for ALL d in 0..=14 (strict).
///
/// Mathematical invariant. Exponential decay with k=0.08 > 0 is strictly
/// monotone decreasing in continuous form. After u8 truncation, the minimum
/// inter-step gap (multiplicative chain by 0.923116) remains ≥ 1 across
/// d=0..=14 (the smallest expected sample near d=15 is ~200*0.9231^15 ≈ 60,
/// still gives ~5-unit drops). Strict `>` is safe for every adjacent pair.
/// Violation at any step means BFS frontier or decay function is broken.
#[test]
fn harness_t7_10_e_gradient_monotone() {
    let mut e = fresh_engine();
    place_spiritual_source(&mut e);
    e.tick();
    for d in 0..=14u32 {
        let v1 = e.resources.influence_grid.sample(SX + d, SY, InfluenceChannel::Spiritual);
        let v2 = e.resources.influence_grid.sample(SX + d + 1, SY, InfluenceChannel::Spiritual);
        // Type A: threshold strict `>` for every adjacent pair
        assert!(
            v1 > v2,
            "gradient not strictly decreasing at d={d}: sample(SX+{d}) = {v1} <= sample(SX+{}) = {v2}. \
             Spiritual k=0.08 must yield strictly decreasing chain across d=0..=14.",
            d + 1
        );
    }
}

// ── Plan E5: boundary tile at max_radius is nonzero ──────────────────────────

/// Type A: influence_grid.sample(SX+15, SY, Spiritual) ∈ [55, 65] after 1 tick.
///
/// Mathematical invariant. The BFS must reach max_radius=15 along the cardinal
/// axis. Chain end at d=15: 200 * 0.9231^15 ≈ 200 * 0.3012 ≈ 60.2. After
/// u8 truncation across 15 chained f32 multiplications: range [55, 65] gives
/// ±5 margin for accumulated rounding.
///
/// Value == 0 means BFS stopped before max_radius (wrong max_radius constant).
/// Value ~45 indicates stale k=0.10 (T7.10.E pre-fix drift; 200*0.9048^15≈44.6).
/// Value <55 may indicate Warmth's k=0.15 (would give ≈21) or stale k=0.10.
#[test]
fn harness_t7_10_e_boundary_at_max_radius() {
    let mut e = fresh_engine();
    place_spiritual_source(&mut e);
    e.tick();
    // Type A: threshold ∈ [55, 65]
    let v = e.resources.influence_grid.sample(SX + 15, SY, InfluenceChannel::Spiritual);
    assert!(
        (55..=65).contains(&v),
        "tile at max_radius=15 must be in [55,65] (chain end ≈60 with k=0.08); got {v}; \
         value==0 means propagation stopped before max_radius=15. \
         Value ~45 = stale k=0.10 (T7.10.E pre-fix drift). \
         Value <55 may indicate Warmth's k=0.15 or stale k=0.10."
    );
}

// ── Plan E6: beyond max_radius is zero ───────────────────────────────────────

/// Type A: influence_grid.sample(SX+16, SY, Spiritual) == 0 after 1 tick.
///
/// Hard cutoff invariant. max_radius=15 is enforced by BFS termination
/// (propagate_bfs at propagate.rs:75 uses `dist >= max_radius` cap). Tile at
/// Manhattan distance 16 must NEVER be written.
#[test]
fn harness_t7_10_e_max_radius_cutoff() {
    let mut e = fresh_engine();
    place_spiritual_source(&mut e);
    e.tick();
    // Type A: threshold == 0 (distance 16 from source, one beyond max_radius=15)
    let v = e.resources.influence_grid.sample(SX + 16, SY, InfluenceChannel::Spiritual);
    assert_eq!(v, 0, "tile beyond max_radius=15 must be 0; got {v}");
}

// ── Plan E7: cold-tier persistence across 10 event-less ticks ────────────────

/// Type A: source center still == 200 after 1 stamp tick + 10 event-less ticks.
///
/// Cold-tier event-driven semantics. Spiritual is UpdateTier::Cold — it must
/// persist indefinitely between placement events. If the persistence branch
/// (copy current→pending before swap) is missing, the disc flickers to 0
/// on every event-less tick.
#[test]
fn harness_t7_10_e_persistence_ten_ticks() {
    let mut e = fresh_engine();
    place_spiritual_source(&mut e);
    e.tick(); // tick 1: stamp + propagation
    for _ in 0..10 {
        e.tick(); // ticks 2–11: event-less persistence branch
    }
    // Type A: threshold == 200 (source persists through all 10 idle ticks)
    let v = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Spiritual);
    assert_eq!(
        v, 200,
        "source must persist across 10 event-less ticks (Cold-tier semantics); got {v}. \
         Persistence branch (current→pending copy) is broken or missing."
    );
}

// ── Plan E8: no event = no spiritual ─────────────────────────────────────────

/// Type A: Spiritual == 0 at (0,0), (SX,SY), (W-1,H-1) after 5 ticks, no events.
///
/// Without any BuildingPlacedEvent, dirty_regions[Spiritual] stays empty every
/// tick. Persistence branch copies current (all zeros) → pending → swap → 0.
/// Any nonzero value means uninitialized buffer state leaking through.
#[test]
fn harness_t7_10_e_no_event_no_spiritual() {
    let mut e = fresh_engine();
    for _ in 0..5 {
        e.tick();
    }
    for (x, y) in [(0u32, 0u32), (SX, SY), (W - 1, H - 1)] {
        // Type A: threshold == 0 at all three positions with no events
        let v = e.resources.influence_grid.sample(x, y, InfluenceChannel::Spiritual);
        assert_eq!(v, 0, "Spiritual at ({x},{y}) must be 0 with no events; got {v}");
    }
}

// ── Plan E9: T7.10.A/B/C/D regression — prior channels survive Spiritual wiring

/// Type A: Warmth/Light/Noise/Danger all == 200 at source after 1 tick post-event.
///
/// Cross-channel regression guard. T7.10.E adds the Spiritual branch to IUS;
/// the Warmth (T7.10.A), Light (T7.10.B), Noise (T7.10.C), and Danger (T7.10.D)
/// branches must still produce their respective discs. A Generator who
/// accidentally re-orders or short-circuits prior channels when adding
/// Spiritual would FAIL this guard.
#[test]
fn harness_t7_10_e_prior_channels_regression_guard() {
    let mut e = fresh_engine();
    place_spiritual_source(&mut e);
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
    // Type A: threshold == 200 for Danger source center
    let danger = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Danger);
    assert_eq!(
        danger, 200,
        "T7.10.D regression: Danger at source must remain 200; got {danger}"
    );
}

// ── Plan E10: dirty_regions[Spiritual] drained by IUS via std::mem::take ─────

/// Type A: dirty_regions[Spiritual].len() == 0 after one full engine.tick() with
/// a BuildingPlacedEvent.
///
/// IUS drain invariant. T7.10.E uses `std::mem::take` on the Spiritual dirty
/// regions, matching T7.10.A/B/C/D semantics. A Generator who reads via
/// `iter()` or `clone()` without draining would leave the entries in place.
#[test]
fn harness_t7_10_e_dirty_regions_drained() {
    let mut e = fresh_engine();
    place_spiritual_source(&mut e);
    e.tick();
    // Type A: threshold == 0
    let len =
        e.resources.influence_grid.dirty_regions[InfluenceChannel::Spiritual as usize].len();
    assert_eq!(
        len, 0,
        "dirty_regions[Spiritual] must be drained by IUS via std::mem::take; got len={len}"
    );
}

// ── Plan E11: other-channels behavior (Beauty propagates post-T7.10.F, FoodAroma/Social stay 0)

/// Type A: Beauty propagates at source (T7.10.F regression guard); unstamped
/// channels FoodAroma/Social sample to 0.
#[test]
fn harness_t7_10_e_other_channels_remain_zero() {
    let mut e = fresh_engine();
    place_spiritual_source(&mut e);
    e.tick();
    // T7.10.F regression guard: Beauty now propagates at source center.
    let beauty = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Beauty);
    assert_eq!(
        beauty, 200,
        "T7.10.F: Beauty at source must be 200 (BFS exp k=0.12 propagation); got {beauty}"
    );
    // Unstamped channels (BSS never marks dirty).
    for ch in [InfluenceChannel::FoodAroma, InfluenceChannel::Social] {
        let v = e.resources.influence_grid.sample(SX, SY, ch);
        assert_eq!(v, 0, "{ch:?} must remain zero (not stamped); got {v}");
    }
}

// ── Plan E12: SPIRITUAL_DECAY_PER_STEP literal must match exp(-0.08) spec ─────

// ── Plan E13: Spiritual-vs-Warmth ratio at d=10 (k=0.08 vs k=0.15 discriminator)

/// Type A: ratio `Spiritual(d=10) / Warmth(d=10)` ∈ [1.9, 2.2] after 1 tick.
///
/// Cross-channel decay-rate invariant. At d=10 from a shared source:
/// - Spiritual k=0.08 chain (200 * 0.923116^10 with u8 truncation) ≈ 86
/// - Warmth k=0.15 chain (200 * 0.860708^10 with u8 truncation) ≈ 42
/// - Canonical ratio ≈ 86 / 42 ≈ 2.05.
///
/// Stale k=0.10 chain would give Spiritual(d=10) ≈ 69 → ratio ≈ 1.64 — OUTSIDE
/// [1.9, 2.2]. The window deliberately excludes the pre-fix drift while
/// permitting f32 rounding around the canonical 2.05. Warmth's max_radius=12
/// covers d=10 so it propagates here.
#[test]
fn harness_t7_10_e_spiritual_warmth_ratio_at_d10() {
    let mut e = fresh_engine();
    place_spiritual_source(&mut e);
    e.tick();
    let spiritual =
        e.resources.influence_grid.sample(SX + 10, SY, InfluenceChannel::Spiritual);
    let warmth = e.resources.influence_grid.sample(SX + 10, SY, InfluenceChannel::Warmth);

    // Type A: Warmth must propagate to d=10 (within Warmth's max_radius=12).
    assert!(
        warmth > 0,
        "Warmth must propagate to d=10 (within max_radius=12); got {warmth}. \
         If 0, the shared BuildingPlacedEvent did not stamp Warmth."
    );

    // Type A: ratio threshold ∈ [1.9, 2.2] (k=0.08 vs k=0.15 expected ≈2.05).
    let ratio = f32::from(spiritual) / f32::from(warmth);
    assert!(
        (1.9..=2.2).contains(&ratio),
        "Spiritual(d=10)/Warmth(d=10) ratio must be ∈ [1.9, 2.2] (k=0.08 vs k=0.15); \
         got ratio = {ratio:.3} (spiritual={spiritual}, warmth={warmth}). \
         Ratio ~1.64 = stale Spiritual k=0.10 drift. \
         Ratio outside [1.9, 2.2] = wrong decay constant on Spiritual or Warmth side."
    );
}

// ── Plan E14: stale-constant regression guard (scan production + renderer) ───
