//! T7.10.A — Warmth channel BFS wiring (single-channel Phase 2 escape).
//!
//! After T7.6 dispatch shell + T7.9.A scaffold + T7.9.B render, the
//! Warmth channel is the first to escape the shell. A BuildingPlacedEvent
//! at (32, 32) with radius 12 must produce a radial warmth disc in
//! `current[Warmth]` centered at (32, 32) after one tick.
//!
//! Setup note: 64×64 engine with empty MaterialRegistry (open field,
//! no wall blocking). All thresholds are LOCKED from plan_final.md —
//! do NOT modify them.
//!
//! Run: `cargo test -p sim-test --test harness_t7_10_a_warmth_wiring -- --nocapture`

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

fn place_warmth_source(engine: &mut SimEngine) {
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (SX, SY),
        radius: 12,
    });
}

// ── Plan A1: source center reaches initial intensity ─────────────────────────

/// Type A: influence_grid.sample(SX, SY, Warmth) == 200 after 1 tick post-event.
///
/// Physical invariant. The BFS source tile must receive full initial_intensity
/// (200) via apply_agg at the start of propagation. Any value ≠ 200 means
/// propagation never ran or the source seeding is broken.
#[test]
fn harness_t7_10_a_source_center_lit() {
    let mut e = fresh_engine();
    place_warmth_source(&mut e);
    e.tick();
    // Type A: threshold == 200
    let v = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Warmth);
    assert_eq!(v, 200, "source center must equal WARMTH_INITIAL_INTENSITY (200); got {v}");
}

// ── Plan A2: radial decay one step ───────────────────────────────────────────

/// Type A: influence_grid.sample(SX, SY+1, Warmth) ∈ [170, 174] after 1 tick.
///
/// Mathematical invariant. Warmth decay k=0.15, WARMTH_DECAY_PER_STEP =
/// exp(-0.15) ≈ 0.8607. Chain step: floor(200.0 * 0.8607) = 172.
/// Range [170, 174] adds ±2 tolerance for f32 rounding.
#[test]
fn harness_t7_10_a_radial_decay_one_step() {
    let mut e = fresh_engine();
    place_warmth_source(&mut e);
    e.tick();
    // Type A: threshold ∈ [170, 174]
    let v = e.resources.influence_grid.sample(SX, SY + 1, InfluenceChannel::Warmth);
    assert!(
        (170..=174).contains(&v),
        "1-step neighbor must decay to ~172 (200*exp(-0.15)); got {v}. \
         Below 170 = k wrong; above 174 = no decay applied."
    );
}

// ── Plan A3: gradient monotonically decreasing along cardinal axis ────────────

/// Type A: sample(SX+d, SY) > sample(SX+d+1, SY) for all d in 0..=11.
///
/// Mathematical invariant. Exponential decay with k=0.15 > 0 is strictly
/// monotonically decreasing. Violation at any step indicates BFS ordering
/// error or wrong decay direction.
#[test]
fn harness_t7_10_a_gradient_monotone() {
    let mut e = fresh_engine();
    place_warmth_source(&mut e);
    e.tick();
    for d in 0u32..=11 {
        let v_near = e.resources.influence_grid.sample(SX + d, SY, InfluenceChannel::Warmth);
        let v_far = e.resources.influence_grid.sample(SX + d + 1, SY, InfluenceChannel::Warmth);
        // Type A: strict monotone decrease for each pair (d, d+1)
        assert!(
            v_near > v_far,
            "gradient must be strictly decreasing: \
             sample({},{})={} must > sample({},{})={}; \
             violation at d={d} means BFS decay is broken",
            SX + d, SY, v_near,
            SX + d + 1, SY, v_far,
            d = d
        );
    }
}

// ── Plan A4: boundary tile at max_radius is nonzero ──────────────────────────

/// Type A: influence_grid.sample(44, 32, Warmth) ∈ [26, 34] after 1 tick.
///
/// Mathematical invariant. The BFS must reach max_radius=12 tiles.
/// Chain-computed value at d=12: floor(36.0 * 0.8607) = 30.
/// Range [26, 34] gives ±4 margin for accumulated f32 truncation.
/// Value == 0 means propagation stopped prematurely.
#[test]
fn harness_t7_10_a_boundary_at_max_radius() {
    let mut e = fresh_engine();
    place_warmth_source(&mut e);
    e.tick();
    // Tile (44, 32) is exactly Manhattan distance 12 from source (32, 32).
    // Type A: threshold ∈ [26, 34]
    let v = e.resources.influence_grid.sample(44, 32, InfluenceChannel::Warmth);
    assert!(
        (26..=34).contains(&v),
        "tile at max_radius=12 must be in [26,34] (chain end ≈30); got {v}; \
         value==0 means propagation stopped before max_radius"
    );
}

// ── Plan A5: beyond max_radius is zero ───────────────────────────────────────

/// Type A: influence_grid.sample(SX+13, SY, Warmth) == 0 after 1 tick.
///
/// Hard cutoff invariant. max_radius=12 is enforced by BFS termination.
/// Tile at Manhattan distance 13 must NEVER be written.
#[test]
fn harness_t7_10_a_max_radius_cutoff() {
    let mut e = fresh_engine();
    place_warmth_source(&mut e);
    e.tick();
    // Type A: threshold == 0 (distance 13 from source, one beyond max_radius=12)
    let v = e.resources.influence_grid.sample(SX + 13, SY, InfluenceChannel::Warmth);
    assert_eq!(v, 0, "tile beyond max_radius=12 must be 0; got {v}");
}

// ── Plan A6: cold-tier persistence across 10 event-less ticks ────────────────

/// Type A: source center still == 200 after 1 stamp tick + 10 event-less ticks.
///
/// Cold-tier event-driven semantics. Warmth is UpdateTier::Cold — it must
/// persist indefinitely between placement events. If the persistence branch
/// (copy current→pending before swap) is missing, the disc flickers to 0
/// on every event-less tick.
#[test]
fn harness_t7_10_a_persistence_across_ticks() {
    let mut e = fresh_engine();
    place_warmth_source(&mut e);
    e.tick(); // tick 1: stamp + propagation
    for _ in 0..10 {
        e.tick(); // ticks 2–11: event-less persistence branch
    }
    // Type A: threshold == 200 (source persists through all 10 idle ticks)
    let v = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Warmth);
    assert_eq!(
        v, 200,
        "source must persist across 10 event-less ticks (Cold-tier semantics); got {v}"
    );
}

// ── Plan A7+A8: non-Warmth channels remain zero at source ────────────────────

/// Type A: sample(SX, SY, ch) == 0 for Spiritual/Beauty/Light (A7) and
///         Noise/FoodAroma/Danger/Social (A8) after 1 tick with event.
///
/// T7.10.A wires the Warmth channel; T7.10.B then wired Light. Spiritual/Beauty
/// have BSS dirty_regions populated but IUS must NOT propagate them yet (T7.10.C..F).
/// Noise/FoodAroma/Danger/Social are not stamped at all. These 6 channels must
/// remain zero. Light is verified separately (it now propagates to 200 at source).
#[test]
fn harness_t7_10_a_other_channels_remain_zero() {
    let mut e = fresh_engine();
    place_warmth_source(&mut e);
    e.tick();

    // T7.10.B regression guard: Light now propagates at source center.
    let light = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Light);
    assert_eq!(
        light, 200,
        "T7.10.B: Light at source must be 200 (shadowcast propagation); got {light}"
    );

    for ch in [
        // Stamped channels still dispatch-shell (BSS marks dirty, IUS does NOT propagate yet)
        InfluenceChannel::Spiritual,
        InfluenceChannel::Beauty,
        // Unstamped channels (BSS never marks dirty)
        InfluenceChannel::Noise,
        InfluenceChannel::FoodAroma,
        InfluenceChannel::Danger,
        InfluenceChannel::Social,
    ] {
        // Type A: threshold == 0 for all non-Warmth, non-Light channels
        let v = e.resources.influence_grid.sample(SX, SY, ch);
        assert_eq!(v, 0, "{ch:?} must remain zero at T7.10.B (only Warmth+Light wired); got {v}");
    }
}

// ── Plan A9: no warmth without events ────────────────────────────────────────

/// Type A: Warmth == 0 at (0,0), (SX,SY), (W-1,H-1) after 5 ticks, no events.
///
/// Without any BuildingPlacedEvent, dirty_regions[Warmth] stays empty every
/// tick. Persistence branch copies current (all zeros) → pending → swap → 0.
/// Any nonzero value means uninitialized buffer state leaking through.
#[test]
fn harness_t7_10_a_no_event_no_warmth() {
    let mut e = fresh_engine();
    for _ in 0..5 {
        e.tick();
    }
    for (x, y) in [(0u32, 0u32), (SX, SY), (W - 1, H - 1)] {
        // Type A: threshold == 0 at all three positions with no events
        let v = e.resources.influence_grid.sample(x, y, InfluenceChannel::Warmth);
        assert_eq!(v, 0, "no events ⇒ Warmth stays zero at ({x},{y}); got {v}");
    }
}

// ── Plan A10: dirty_regions for Warmth are drained after tick ────────────────

/// Type A: dirty_regions[Warmth].len() == 0 after 1 tick processing a Warmth event.
///
/// Correctness invariant. IUS must drain dirty_regions[Warmth] during the tick
/// that processes them (via std::mem::take, not just read). If dirty_regions
/// are not drained, tick N+1 re-runs BFS from stale regions, breaking Cold-tier
/// event-driven semantics and causing incorrect additive accumulation.
#[test]
fn harness_t7_10_a_dirty_regions_drained() {
    let mut e = fresh_engine();
    place_warmth_source(&mut e);
    e.tick();
    // Type A: threshold == 0 (IUS drains via std::mem::take)
    let len = e.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize].len();
    assert_eq!(
        len, 0,
        "dirty_regions[Warmth] must be drained (len==0) after IUS processes them; got {len}"
    );
}
