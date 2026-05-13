//! T7.10.C — Noise channel linear-decay wiring (third-channel Phase 2 escape).
//!
//! After T7.10.A (Warmth BFS) and T7.10.B (Light shadowcast), the Noise
//! channel is the third to escape the dispatch shell. A BuildingPlacedEvent
//! at (32, 32) with radius 12 must produce a Noise field in `current[Noise]`
//! centered at (32, 32) after one tick, computed by linear-decay BFS with
//! alpha=15 per step (Songs of Syx 2-tile ISSUE 2 v0.1.1 fix via
//! `propagate_noise`, wraps `propagate_linear` with u32::MAX max_radius —
//! natural radius via the `intensity < 5` cutoff).
//!
//! Setup note: 64×64 engine with empty MaterialRegistry (open field, no
//! wall blocking → pure 4-neighbor linear decay). All thresholds are
//! LOCKED from plan — do NOT modify them.
//!
//! Run: `cargo test -p sim-test --test harness_t7_10_c_noise_linear_wiring -- --nocapture`

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

fn place_noise_source(engine: &mut SimEngine) {
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (SX, SY),
        radius: 12,
    });
}

// ── Plan C1: source center reaches initial intensity ─────────────────────────

/// Type A: influence_grid.sample(SX, SY, Noise) == 200 after 1 tick post-event.
///
/// Physical invariant. The linear-decay source tile must receive full
/// NOISE_INITIAL_INTENSITY (200) via apply_agg(Max) at the start of
/// propagation (propagate.rs:358). Any value ≠ 200 means propagation never
/// ran or the source seeding is broken.
#[test]
fn harness_t7_10_c_source_center_lit() {
    let mut e = fresh_engine();
    place_noise_source(&mut e);
    e.tick();
    // Type A: threshold == 200
    let v = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Noise);
    assert_eq!(v, 200, "source center must equal NOISE_INITIAL_INTENSITY (200); got {v}");
}

// ── Plan C2: linear decay one cardinal step (alpha=15 discriminator) ─────────

/// Type A: influence_grid.sample(SX+1, SY, Noise) == 185 after 1 tick.
///
/// Mathematical invariant. Linear decay alpha=15 per BFS step
/// (propagate.rs:385: `intensity.saturating_sub(alpha)`). At BFS d=1:
/// 200 - 15 = 185 exact. No tolerance — integer arithmetic, no f32.
///
/// A shadowcast implementation (200/(1+0.1*1)=181) would FAIL this — proves
/// linear-decay path is wired, not Light's falloff function.
#[test]
fn harness_t7_10_c_linear_one_step_alpha_discriminator() {
    let mut e = fresh_engine();
    place_noise_source(&mut e);
    e.tick();
    // Type A: threshold == 185 (exact)
    let v = e.resources.influence_grid.sample(SX + 1, SY, InfluenceChannel::Noise);
    assert_eq!(
        v, 185,
        "1-step cardinal neighbor must equal 200-15=185 (linear alpha=15); got {v}. \
         Value 181 = shadowcast falloff (wrong primitive). \
         Other value = wrong alpha or propagation never ran."
    );
}

// ── Plan C3: BFS distance metric (diagonal at Manhattan d=2) ─────────────────

/// Type A: influence_grid.sample(SX+1, SY+1, Noise) == 170 after 1 tick.
///
/// BFS distance invariant. Diagonal (SX+1, SY+1) reached in 2 cardinal steps
/// via 4-neighbor BFS: 200 - 2*15 = 170. NO Euclidean shortcut — BFS uses
/// Manhattan-like distance under 4-neighborhood expansion.
///
/// A Euclidean implementation (200 - 15*sqrt(2) ≈ 178.8) would FAIL — proves
/// linear-decay uses the BFS frontier, not Euclidean distance like Light.
#[test]
fn harness_t7_10_c_bfs_distance_manhattan_discriminator() {
    let mut e = fresh_engine();
    place_noise_source(&mut e);
    e.tick();
    // Type A: threshold == 170 (200 - 2*15)
    let v = e.resources.influence_grid.sample(SX + 1, SY + 1, InfluenceChannel::Noise);
    assert_eq!(
        v, 170,
        "diagonal (SX+1,SY+1) must equal 170 (BFS d=2, 200-2*15); got {v}. \
         Value ≈179 = Euclidean (wrong distance metric). \
         Other value = wrong alpha or BFS broken."
    );
}

// ── Plan C4: gradient monotonically strictly decreasing along cardinal axis ──

/// Type A: sample(SX+d, SY) > sample(SX+d+1, SY) for ALL d in 0..=11 (strict).
///
/// Mathematical invariant. Linear decay alpha=15 yields constant 15-unit step
/// drops: 200, 185, 170, ..., 20, 5. Adjacent pairs always differ by exactly
/// 15 (no truncation ties) so strict `>` is safe across d=0..=11. Violation
/// at any step means BFS frontier or alpha decrement is broken.
#[test]
fn harness_t7_10_c_gradient_monotone() {
    let mut e = fresh_engine();
    place_noise_source(&mut e);
    e.tick();
    for d in 0..=11u32 {
        let v1 = e.resources.influence_grid.sample(SX + d, SY, InfluenceChannel::Noise);
        let v2 = e.resources.influence_grid.sample(SX + d + 1, SY, InfluenceChannel::Noise);
        // Type A: threshold strict `>` for every adjacent pair
        assert!(
            v1 > v2,
            "gradient not strictly decreasing at d={d}: sample(SX+{d}) = {v1} <= sample(SX+{}) = {v2}. \
             Linear alpha=15 must yield exact 15-unit drops.",
            d + 1
        );
    }
}

// ── Plan C5: natural radius boundary (intensity<5 cutoff) ────────────────────

/// Type A: sample(SX+13, SY, Noise) == 5 AND sample(SX+14, SY, Noise) == 0.
///
/// Natural cutoff invariant. propagate_linear (propagate.rs:363) exits when
/// `intensity < 5`. At BFS d=13: 200 - 13*15 = 5 → still queued (5 not < 5),
/// so the tile is written with intensity=5. At d=14: decayed = 5-15 →
/// saturating_sub = 0, `after_block > 0` is false → tile NOT written.
///
/// Discriminates: a Generator who uses `<=` cutoff or wrong alpha would
/// produce a different boundary tile.
#[test]
fn harness_t7_10_c_natural_radius_boundary() {
    let mut e = fresh_engine();
    place_noise_source(&mut e);
    e.tick();
    // Type A: threshold == 5 at d=13
    let v13 = e.resources.influence_grid.sample(SX + 13, SY, InfluenceChannel::Noise);
    assert_eq!(
        v13, 5,
        "Noise at d=13 must equal 200-13*15=5 (intensity<5 cutoff still queues 5); got {v13}"
    );
    // Type A: threshold == 0 at d=14
    let v14 = e.resources.influence_grid.sample(SX + 14, SY, InfluenceChannel::Noise);
    assert_eq!(
        v14, 0,
        "Noise at d=14 must be 0 (decayed=saturating_sub(5,15)=0, after_block>0 false); got {v14}"
    );
}

// ── Plan C6: persistence — Noise survives 10 event-less ticks ────────────────

/// Type A: sample(SX, SY, Noise) == 200 after the single event tick + 9 more
/// event-less ticks (total 10 ticks).
///
/// Persistence invariant (T7.10.C update.rs:217-254). On event-less ticks
/// the Hot-tier branch copies `current[Noise]` into `pending[Noise]` so the
/// swap is a no-op, preventing flicker. A Generator who omits persistence
/// would see Noise zero out after tick 2.
#[test]
fn harness_t7_10_c_persistence_ten_ticks() {
    let mut e = fresh_engine();
    place_noise_source(&mut e);
    for _ in 0..10 {
        e.tick();
    }
    // Type A: threshold == 200 at source after 10 ticks
    let v = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Noise);
    assert_eq!(
        v, 200,
        "source Noise must persist == 200 after 10 ticks (1 event + 9 event-less); got {v}. \
         Persistence branch (current→pending copy) is broken or missing."
    );
}

// ── Plan C7: no event = no noise ─────────────────────────────────────────────

/// Type A: Noise == 0 at (0,0), (SX,SY), (W-1,H-1) after 5 ticks, no events.
///
/// Without any BuildingPlacedEvent, dirty_regions[Noise] stays empty every
/// tick. Persistence branch copies current (all zeros) → pending → swap → 0.
/// Any nonzero value means uninitialized buffer state leaking through.
#[test]
fn harness_t7_10_c_no_event_no_noise() {
    let mut e = fresh_engine();
    for _ in 0..5 {
        e.tick();
    }
    for (x, y) in [(0u32, 0u32), (SX, SY), (W - 1, H - 1)] {
        // Type A: threshold == 0 at all three positions with no events
        let v = e.resources.influence_grid.sample(x, y, InfluenceChannel::Noise);
        assert_eq!(v, 0, "Noise at ({x},{y}) must be 0 with no events; got {v}");
    }
}

// ── Plan C8: T7.10.A/B regression — Warmth+Light survive Noise wiring ────────

/// Type A: Warmth(SX,SY) == 200 AND Light(SX,SY) == 200 after 1 tick post-event.
///
/// Cross-channel regression guard. T7.10.C adds the Noise branch to IUS;
/// the Warmth (T7.10.A) and Light (T7.10.B) branches must still produce
/// their respective discs. A Generator who accidentally re-orders or
/// short-circuits Warmth/Light when adding Noise would FAIL this guard.
#[test]
fn harness_t7_10_c_warmth_and_light_regression_guard() {
    let mut e = fresh_engine();
    place_noise_source(&mut e);
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
}

// ── Plan C9: dirty_regions[Noise] drained by IUS via std::mem::take ─────────

/// Type A: dirty_regions[Noise].len() == 0 after one full engine.tick() with
/// a BuildingPlacedEvent.
///
/// IUS drain invariant. T7.10.C uses `std::mem::take` on the Noise dirty
/// regions, matching T7.10.A/B semantics. A Generator who reads via
/// `iter()` or `clone()` without draining would leave the entries in place.
#[test]
fn harness_t7_10_c_dirty_regions_drained() {
    let mut e = fresh_engine();
    place_noise_source(&mut e);
    e.tick();
    // Type A: threshold == 0
    let len =
        e.resources.influence_grid.dirty_regions[InfluenceChannel::Noise as usize].len();
    assert_eq!(
        len, 0,
        "dirty_regions[Noise] must be drained by IUS via std::mem::take; got len={len}"
    );
}

// ── Plan C10: other-channels behavior (Spiritual/Beauty stay 0, FoodAroma/Social stay 0)

/// Type A: T7.10.D regression — Danger sample == 200 at source;
/// dispatch-shell stamped channels Spiritual/Beauty sample to 0;
/// unstamped channels FoodAroma/Social sample to 0.
///
/// T7.10.C wired Warmth+Light+Noise. T7.10.D additionally wires Danger.
/// The remaining stamped channels (Spiritual, Beauty) have BSS dirty_regions
/// but IUS does not yet propagate them. Unstamped channels (FoodAroma, Social)
/// stay completely cold.
#[test]
fn harness_t7_10_c_other_channels_remain_zero() {
    let mut e = fresh_engine();
    place_noise_source(&mut e);
    e.tick();
    // T7.10.D regression guard: Danger now propagates at source center.
    let danger = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Danger);
    assert_eq!(
        danger, 200,
        "T7.10.D: Danger at source must be 200 (linear-decay+cap propagation); got {danger}"
    );
    // T7.10.E regression guard: Spiritual now propagates at source center.
    let spiritual = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Spiritual);
    assert_eq!(
        spiritual, 200,
        "T7.10.E: Spiritual at source must be 200 (BFS exp k=0.10 propagation); got {spiritual}"
    );
    // Dispatch-shell stamped channel (BSS marks dirty, IUS does NOT propagate yet).
    let beauty = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Beauty);
    assert_eq!(
        beauty, 0,
        "Beauty must remain zero at T7.10.E (T7.10.F wires it); got {beauty}"
    );
    // Unstamped channels (BSS never marks dirty).
    for ch in [
        InfluenceChannel::FoodAroma,
        InfluenceChannel::Social,
    ] {
        let v = e.resources.influence_grid.sample(SX, SY, ch);
        assert_eq!(v, 0, "{ch:?} must remain zero (not stamped); got {v}");
    }
}
