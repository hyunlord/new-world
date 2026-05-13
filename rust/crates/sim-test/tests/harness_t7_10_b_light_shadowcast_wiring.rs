//! T7.10.B — Light channel shadowcast wiring (second-channel Phase 2 escape).
//!
//! After T7.10.A wired Warmth via BFS, the Light channel is the second to
//! escape the dispatch shell. A BuildingPlacedEvent at (32, 32) with
//! radius 12 must produce a Light field in `current[Light]` centered at
//! (32, 32) after one tick, computed by recursive symmetric shadowcasting
//! with `intensity / (1 + 0.1 * d)` Euclidean falloff (propagate.rs:249).
//!
//! Setup note: 64×64 engine with empty MaterialRegistry (open field,
//! no wall blocking → all-open shadowcast = octagonal/circular shape).
//! All thresholds are LOCKED from plan — do NOT modify them.
//!
//! Run: `cargo test -p sim-test --test harness_t7_10_b_light_shadowcast_wiring -- --nocapture`

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

fn place_light_source(engine: &mut SimEngine) {
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (SX, SY),
        radius: 12,
    });
}

// ── Plan B1: source center reaches initial intensity ─────────────────────────

/// Type A: influence_grid.sample(SX, SY, Light) == 200 after 1 tick post-event.
///
/// Physical invariant. The shadowcast source tile must receive full
/// LIGHT_INITIAL_INTENSITY (200) via apply_agg(Max) at the start of
/// propagation (propagate.rs:161). Any value ≠ 200 means propagation
/// never ran or the source seeding is broken.
#[test]
fn harness_t7_10_b_source_center_lit() {
    let mut e = fresh_engine();
    place_light_source(&mut e);
    e.tick();
    // Type A: threshold == 200
    let v = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Light);
    assert_eq!(v, 200, "source center must equal LIGHT_INITIAL_INTENSITY (200); got {v}");
}

// ── Plan B2: shadowcast falloff one cardinal step ────────────────────────────

/// Type A: influence_grid.sample(SX+1, SY, Light) ∈ [178, 184] after 1 tick.
///
/// Mathematical invariant. Shadowcast falloff `intensity / (1 + 0.1 * d)`
/// (propagate.rs:249). At d=1: 200 / 1.1 = 181.8 → truncates to 181.
/// Range [178, 184] adds ±3 tolerance for f32 rounding across octants.
#[test]
fn harness_t7_10_b_falloff_one_step() {
    let mut e = fresh_engine();
    place_light_source(&mut e);
    e.tick();
    // Type A: threshold ∈ [178, 184]
    let v = e.resources.influence_grid.sample(SX + 1, SY, InfluenceChannel::Light);
    assert!(
        (178..=184).contains(&v),
        "1-step neighbor must decay to ~181 (200/(1+0.1)); got {v}. \
         Below 178 = falloff wrong; above 184 = no decay applied."
    );
}

// ── Plan B3: diagonal tile uses Euclidean distance (discriminator) ──────────

/// Type A: influence_grid.sample(SX+1, SY+1, Light) ∈ [172, 179] after 1 tick.
///
/// Euclidean discriminator invariant. Diagonal distance = sqrt(2) ≈ 1.4142.
/// Shadowcast falloff: 200 / (1 + 0.1 * 1.4142) = 200 / 1.14142 ≈ 175.
/// Range [172, 179] adds tolerance for f32 sqrt rounding across platforms.
///
/// A BFS-Manhattan implementation would use d=2 for (SX+1, SY+1),
/// yielding floor(200 / 1.2) = 166 — OUTSIDE [172, 179]. Any
/// implementation that produces a value in [172, 179] provably uses
/// Euclidean distance, not Manhattan distance.
#[test]
fn harness_t7_10_b_falloff_diagonal_euclidean_discriminator() {
    let mut e = fresh_engine();
    place_light_source(&mut e);
    e.tick();
    // Type A: threshold ∈ [172, 179]  (Euclidean d≈1.414 → 200/1.1414≈175)
    // BFS-Manhattan gives 166 which falls OUTSIDE this range — discriminates impls.
    let v = e.resources.influence_grid.sample(SX + 1, SY + 1, InfluenceChannel::Light);
    assert!(
        (172..=179).contains(&v),
        "diagonal tile (SX+1,SY+1) must be in [172,179] \
         (Euclidean d≈1.414 → ≈175); got {v}. \
         Value 166 = BFS-Manhattan (d=2 wrongly used). \
         Outside [172,179] = wrong falloff formula or distance metric."
    );
}

// ── Plan B4: gradient monotonically strictly decreasing along cardinal axis ──

/// Type A: sample(SX+d, SY) > sample(SX+d+1, SY) for ALL d in 0..=14 (strict).
///
/// Mathematical invariant. Shadowcast falloff `intensity / (1 + 0.1 * d)` with
/// k=0.1 > 0 is strictly monotone decreasing in continuous form. After u8
/// truncation, the minimum inter-step gap along this cardinal axis is 3
/// (at d=14→15: floor(200/2.4)=83 > floor(200/2.5)=80), so strict `>` is safe
/// for every adjacent pair — no truncation ties possible in [0,14]. Violation
/// at any step means the Euclidean distance computation or falloff is broken.
#[test]
fn harness_t7_10_b_gradient_monotone() {
    let mut e = fresh_engine();
    place_light_source(&mut e);
    e.tick();

    // Strict pairwise decrease for every adjacent pair along cardinal axis.
    // Min gap ≥ 3 (at d=14→15: floor(200/2.4)=83 > floor(200/2.5)=80),
    // so strict `>` is safe — no u8 truncation ties possible.
    for d in 0u32..=14 {
        let v_near = e.resources.influence_grid.sample(SX + d, SY, InfluenceChannel::Light);
        let v_far = e.resources.influence_grid.sample(SX + d + 1, SY, InfluenceChannel::Light);
        // Type A: strictly decreasing for every pair (d, d+1) on [0,14]
        assert!(
            v_near > v_far,
            "gradient must be strictly decreasing: \
             sample({},{})={} must > sample({},{})={}; \
             violation at d={d} means shadowcast falloff is broken",
            SX + d, SY, v_near,
            SX + d + 1, SY, v_far,
        );
    }
}

// ── Plan B5: boundary tile at max_radius is nonzero ──────────────────────────

/// Type A: influence_grid.sample(SX+15, SY, Light) ∈ [76, 84] after 1 tick.
///
/// Mathematical invariant. The shadowcast must reach max_radius=15 along
/// the cardinal axis (Euclidean dist = 15 exactly). Expected:
/// 200 / (1 + 0.1*15) = 200 / 2.5 = 80.
/// Range [76, 84] gives ±4 margin for f32 sqrt and clamp truncation.
/// Value == 0 means propagation stopped prematurely.
#[test]
fn harness_t7_10_b_boundary_at_max_radius() {
    let mut e = fresh_engine();
    place_light_source(&mut e);
    e.tick();
    // Tile (SX+15, SY) is Euclidean distance exactly 15 from source.
    // Type A: threshold ∈ [76, 84]
    let v = e.resources.influence_grid.sample(SX + 15, SY, InfluenceChannel::Light);
    assert!(
        (76..=84).contains(&v),
        "tile at max_radius=15 must be in [76,84] (chain end ≈80); got {v}; \
         value==0 means shadowcast stopped before max_radius"
    );
}

// ── Plan B6: beyond max_radius is zero ───────────────────────────────────────

/// Type A: influence_grid.sample(SX+16, SY, Light) == 0 after 1 tick.
///
/// Hard cutoff invariant. max_radius=15 is enforced by shadowcast
/// `dist_sq <= radius²` check (propagate.rs:247). At (SX+16, SY)
/// the Euclidean distance squared = 256 > 225 → never written.
#[test]
fn harness_t7_10_b_max_radius_cutoff() {
    let mut e = fresh_engine();
    place_light_source(&mut e);
    e.tick();
    // Type A: threshold == 0 (Euclidean distance 16 > max_radius 15)
    let v = e.resources.influence_grid.sample(SX + 16, SY, InfluenceChannel::Light);
    assert_eq!(v, 0, "tile beyond max_radius=15 must be 0; got {v}");
}

// ── Plan B7: Warm-tier persistence across 10 event-less ticks ────────────────

/// Type A: source center still == 200 after 1 stamp tick + 10 event-less ticks.
///
/// Warm-tier persistence (T7.10.B). Light must survive event-less ticks
/// just like Warmth does for Cold-tier. If the persistence branch (copy
/// current[Light] → pending[Light] before swap) is missing, the field
/// flickers to 0 on every event-less tick.
#[test]
fn harness_t7_10_b_persistence_across_ticks() {
    let mut e = fresh_engine();
    place_light_source(&mut e);
    e.tick(); // tick 1: stamp + shadowcast
    for _ in 0..10 {
        e.tick(); // ticks 2–11: event-less persistence branch
    }
    // Type A: threshold == 200 (source persists through all 10 idle ticks)
    let v = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Light);
    assert_eq!(
        v, 200,
        "source must persist across 10 event-less ticks (Warm-tier persistence); got {v}"
    );
}

// ── Plan B8+B9: non-Light channels behavior at source ────────────────────────

/// Type A: at source after 1 tick with event:
///   - Warmth = 200 (T7.10.A regression guard — must remain wired)
///   - Spiritual/Beauty = 0 (BSS marks dirty but IUS dispatch-shell)
///   - Noise/FoodAroma/Danger/Social = 0 (not stamped)
///
/// T7.10.B wires the Light channel. The T7.10.A Warmth wiring must NOT
/// regress (Warmth still propagates to 200 at source). Spiritual/Beauty
/// have BSS dirty_regions populated but IUS does not propagate them yet
/// (dispatch-shell clears pending → swap → 0). Unstamped channels stay 0.
#[test]
fn harness_t7_10_b_other_channels_behavior() {
    let mut e = fresh_engine();
    place_light_source(&mut e);
    e.tick();

    // T7.10.A regression guard: Warmth must still propagate to 200 at source.
    let warmth = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Warmth);
    assert_eq!(
        warmth, 200,
        "T7.10.A regression: Warmth at source must remain 200; got {warmth}"
    );

    // T7.10.C regression guard: Noise must propagate to 200 at source.
    let noise = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Noise);
    assert_eq!(
        noise, 200,
        "T7.10.C: Noise at source must be 200 (linear-decay propagation); got {noise}"
    );

    // T7.10.D regression guard: Danger must propagate to 200 at source.
    let danger = e.resources.influence_grid.sample(SX, SY, InfluenceChannel::Danger);
    assert_eq!(
        danger, 200,
        "T7.10.D: Danger at source must be 200 (linear-decay+cap propagation); got {danger}"
    );

    // Dispatch-shell stamped channels (BSS marks dirty, IUS does NOT propagate yet).
    for ch in [InfluenceChannel::Spiritual, InfluenceChannel::Beauty] {
        let v = e.resources.influence_grid.sample(SX, SY, ch);
        assert_eq!(v, 0, "{ch:?} must remain zero at T7.10.D (T7.10.E..F wires it); got {v}");
    }

    // Unstamped channels (BSS never marks dirty).
    for ch in [
        InfluenceChannel::FoodAroma,
        InfluenceChannel::Social,
    ] {
        let v = e.resources.influence_grid.sample(SX, SY, ch);
        assert_eq!(v, 0, "{ch:?} must remain zero (not stamped); got {v}");
    }
}

// ── Plan B10: no event = no light ────────────────────────────────────────────

/// Type A: Light == 0 at (0,0), (SX,SY), (W-1,H-1) after 5 ticks, no events.
///
/// Without any BuildingPlacedEvent, dirty_regions[Light] stays empty every
/// tick. Persistence branch copies current (all zeros) → pending → swap → 0.
/// Any nonzero value means uninitialized buffer state leaking through.
#[test]
fn harness_t7_10_b_no_event_no_light() {
    let mut e = fresh_engine();
    for _ in 0..5 {
        e.tick();
    }
    for (x, y) in [(0u32, 0u32), (SX, SY), (W - 1, H - 1)] {
        // Type A: threshold == 0 at all three positions with no events
        let v = e.resources.influence_grid.sample(x, y, InfluenceChannel::Light);
        assert_eq!(v, 0, "no events ⇒ Light stays zero at ({x},{y}); got {v}");
    }
}

// ── Plan B11: dirty_regions for Light are drained after tick ─────────────────

/// Type A: dirty_regions[Light].len() == 0 after 1 tick processing a Light event.
///
/// Correctness invariant. IUS must drain dirty_regions[Light] during the tick
/// that processes them (via std::mem::take, not just read). If dirty_regions
/// are not drained, tick N+1 re-runs shadowcast from stale regions, breaking
/// Warm-tier persistence semantics and wasting CPU on redundant propagation.
#[test]
fn harness_t7_10_b_dirty_regions_drained() {
    let mut e = fresh_engine();
    place_light_source(&mut e);
    e.tick();
    // Type A: threshold == 0 (IUS drains via std::mem::take)
    let len = e.resources.influence_grid.dirty_regions[InfluenceChannel::Light as usize].len();
    assert_eq!(
        len, 0,
        "dirty_regions[Light] must be drained (len==0) after IUS processes them; got {len}"
    );
}
