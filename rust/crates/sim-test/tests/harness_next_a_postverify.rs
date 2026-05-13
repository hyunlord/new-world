//! T7.7.B + T7.8 cross-cutting post-verify test (next-a-postverify, plan_attempt 3).
//!
//! Each assertion is an independent `#[test]` function (plan_attempt 3 QC fix 4).
//! No shared state between tests — each independently constructs its own
//! `fresh_phase2_engine()`. The "Fixture A/B isolation" concern is eliminated
//! entirely by this separate-function structure.
//!
//! Validates the complete `building_event_queue → BuildingStampSystem → InfluenceUpdateSystem`
//! pipeline through the FFI enqueue surface (`enqueue_building_placed`) under the full
//! Phase 2 system stack (`engine.tick()`), bridging T7.7.B mechanism coverage with
//! T7.8 substantial harness coverage.
//!
//! Run: `cargo test -p sim-test --test harness_next_a_postverify -- --nocapture`

use sim_bridge::ffi::enqueue_building_placed;
use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::register_phase2_systems;

const W: u32 = 64;
const H: u32 = 64;

/// Fresh 64×64 engine with all 4 Phase 2 systems registered, zero agents.
/// Matches T7.8's `fresh_phase2_engine()` configuration exactly.
fn fresh_phase2_engine() -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    engine
}

// ── Assertion 1a: baseline_queue_empty_on_fresh_engine ───────────────────────

/// Type A: building_event_queue.len() == 0 on fresh Phase 2 engine (0 ticks).
/// Mathematical invariant — freshly constructed engine must have empty queue.
/// Defends against Generator pre-seeding queue in the fixture constructor
/// (would make Assertion 3's drain non-diagnostic).
#[test]
fn harness_ffi_bridge_baseline_queue_empty_on_fresh_engine() {
    let engine = fresh_phase2_engine();
    // Type A: threshold == 0
    assert_eq!(
        engine.resources.building_event_queue.len(),
        0,
        "building_event_queue must be empty at fresh Phase 2 engine construction"
    );
}

// ── Assertion 1b: baseline_dirty_regions_warmth_zero_on_fresh_engine ────────

/// Type A: dirty_regions[Warmth].len() == 0 on fresh Phase 2 engine (0 ticks).
/// Mathematical invariant — fresh engine must have no dirty marks.
/// Defends against Generator pre-seeding 3 dirty marks in the constructor
/// (would make Assertion 4's count of 3 trivially pass without BSS running).
#[test]
fn harness_ffi_bridge_baseline_dirty_regions_warmth_zero_on_fresh_engine() {
    let engine = fresh_phase2_engine();
    // Type A: threshold == 0
    assert_eq!(
        engine.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize].len(),
        0,
        "dirty_regions[Warmth].len() must be 0 at fresh Phase 2 engine construction"
    );
}

// ── Assertion 2: pre_tick_queue_len_equals_3_after_3_ffi_enqueue_calls ──────

/// Type A: queue.len() == 3 after 3 in-bounds FFI enqueue calls and BEFORE any tick.
/// CRITICAL BRIDGE ASSERTION: FFI enqueue path must actually populate the queue.
/// Without this pre-tick check, Assertion 3's drain is trivially satisfied by a
/// hollow FFI (no-op enqueue → queue stays 0 → drain "passes" without BSS running).
/// If Assertion 2 fails, Assertions 3–6 are uninformative.
#[test]
fn harness_ffi_bridge_pre_tick_queue_len_equals_3_after_3_ffi_enqueue_calls() {
    let mut engine = fresh_phase2_engine();
    let ok1 = enqueue_building_placed(&mut engine.resources, 10, 10, 1);
    let ok2 = enqueue_building_placed(&mut engine.resources, 20, 20, 1);
    let ok3 = enqueue_building_placed(&mut engine.resources, 30, 30, 1);
    assert!(
        ok1 && ok2 && ok3,
        "all 3 in-bounds FFI enqueue calls must return true \
         (coordinates 10/20/30 < grid_width=64)"
    );
    // Type A: threshold == 3
    assert_eq!(
        engine.resources.building_event_queue.len(),
        3,
        "queue.len() must be 3 after 3 in-bounds FFI enqueue calls and BEFORE any tick. \
         A hollow FFI (no-op enqueue) would leave this at 0, \
         making Assertions 3–6 vacuously satisfied."
    );
}

// ── Assertion 2.5: pre_tick_dirty_regions_zero_after_enqueue_before_tick ────

/// Type A: dirty_regions[Warmth].len() == 0 after 3 FFI enqueues, BEFORE any tick.
/// Closes gaming vector: BSS must call mark_dirty() INSIDE tick(), not inside enqueue.
/// If BSS called mark_dirty() inside enqueue_building_placed (at dequeue time), dirty_regions
/// would already be non-zero here — making Assertion 4's post-tick count of 3 trivially
/// pass even with a stub tick() that never calls BSS.
#[test]
fn harness_ffi_bridge_pre_tick_dirty_regions_zero_after_enqueue_before_tick() {
    let mut engine = fresh_phase2_engine();
    enqueue_building_placed(&mut engine.resources, 10, 10, 1);
    enqueue_building_placed(&mut engine.resources, 20, 20, 1);
    enqueue_building_placed(&mut engine.resources, 30, 30, 1);
    // Type A: threshold == 0 — dirty must not be set before tick() runs BSS
    assert_eq!(
        engine.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize].len(),
        0,
        "dirty_regions[Warmth].len() must be 0 after 3 FFI enqueues and BEFORE any tick. \
         BSS must call mark_dirty() inside tick(), not inside enqueue_building_placed. \
         Failure here means dirty marks are set at enqueue time, gaming Assertion 4."
    );
}

// ── Assertion 3: ffi_enqueue_path_drains_queue_via_full_pipeline ─────────────

/// Type A: queue.len() == 0 after 3 FFI enqueues + 1 full engine.tick().
/// BSS while-loop must drain all 3 FFI-enqueued events in a single tick.
/// Assertion 2 establishes the pre-condition: queue had exactly 3 events.
#[test]
fn harness_ffi_bridge_ffi_enqueue_path_drains_queue_via_full_pipeline() {
    let mut engine = fresh_phase2_engine();
    enqueue_building_placed(&mut engine.resources, 10, 10, 1);
    enqueue_building_placed(&mut engine.resources, 20, 20, 1);
    enqueue_building_placed(&mut engine.resources, 30, 30, 1);
    engine.tick();
    // Type A: threshold == 0
    assert_eq!(
        engine.resources.building_event_queue.len(),
        0,
        "queue must be empty after 1 full engine.tick() \
         (BSS while-loop drains all FFI-enqueued events)"
    );
}

// ── Assertion 4: ffi_beauty_propagated_at_3_sources_after_ffi_enqueue_path ──

/// Type A: current[Beauty] == 200 at each of the 3 source centers after 3 FFI
/// enqueues + 1 full tick.
///
/// T7.10.F rotation: post-Beauty-wiring, IUS drains dirty_regions[Beauty] each
/// tick (std::mem::take), so dirty_regions[Beauty].len() == 0 after a full tick.
/// Original Assertion 4 intent: verify the FFI path drives stamping for the
/// Beauty channel (anti-mis-mapping). Re-formulated as propagation evidence:
/// BSS-stamped Beauty regions must run through IUS BFS to produce
/// current[Beauty] == 200 at each source center. A FFI bug that drops Beauty
/// (e.g. only marks Warmth) would leave current[Beauty] == 0 at all 3 sources.
#[test]
fn harness_ffi_bridge_ffi_dirty_regions_warmth_count_3_after_ffi_enqueue_path() {
    let mut engine = fresh_phase2_engine();
    enqueue_building_placed(&mut engine.resources, 10, 10, 1);
    enqueue_building_placed(&mut engine.resources, 20, 20, 1);
    enqueue_building_placed(&mut engine.resources, 30, 30, 1);
    engine.tick();
    // Type A: threshold == 200 at each source center (BFS center receives full
    // BEAUTY_INITIAL_INTENSITY = 200; nonzero proves FFI→BSS→IUS path drove Beauty).
    for (sx, sy) in [(10u32, 10u32), (20, 20), (30, 30)] {
        let v = engine
            .resources
            .influence_grid
            .sample(sx, sy, InfluenceChannel::Beauty);
        assert_eq!(
            v, 200,
            "current[Beauty] at source ({sx},{sy}) must be 200 after FFI enqueue + tick. \
             FFI bug dropping Beauty channel mapping would leave this at 0. Got {v}"
        );
    }
}

// ── Assertion 5: ffi_beauty_propagation_centers_match_enqueue_coordinates ───

/// Type A: current[Beauty] == 200 at exactly the 3 enqueue centers and 0 at
/// equivalently-spaced non-enqueue control positions.
///
/// T7.10.F rotation: post-Beauty-wiring, IUS drains dirty_regions[Beauty] each
/// tick (the exact-bounds DirtyRegion check is no longer possible — regions
/// don't persist after the tick). Original intent: prove BSS propagates the
/// FFI position field rather than collapsing all events to a single coord.
/// Re-formulation: each enqueue coordinate must yield a distinct BFS center
/// in current[Beauty] (==200) AND a control position equidistant from any
/// enqueued source by > BEAUTY_MAX_RADIUS=15 must stay 0. A BSS bug that
/// drops position info (e.g. all events stamped at (0,0)) would leave
/// (20,20)/(30,30) at 0 while (0,0) is 200 — caught here.
#[test]
fn harness_ffi_bridge_ffi_dirty_regions_exact_bounds_match_enqueue_coordinates() {
    let mut engine = fresh_phase2_engine();
    enqueue_building_placed(&mut engine.resources, 10, 10, 1);
    enqueue_building_placed(&mut engine.resources, 20, 20, 1);
    enqueue_building_placed(&mut engine.resources, 30, 30, 1);
    engine.tick();

    // (a) Each enqueue center must propagate to current[Beauty] == 200.
    // BFS source receives BEAUTY_INITIAL_INTENSITY = 200 at the (cx, cy)
    // computed from each DirtyRegion. Sources at (10,10)/(20,20)/(30,30)
    // are 10 tiles apart (Manhattan); BEAUTY_MAX_RADIUS = 15, so a single
    // source could in principle reach a neighbour center — but the center
    // tile of each BFS receives the full 200 stamp regardless of Max-merge
    // because Max-merge with a possibly-higher-decay value preserves 200.
    for (sx, sy) in [(10u32, 10u32), (20, 20), (30, 30)] {
        let v = engine
            .resources
            .influence_grid
            .sample(sx, sy, InfluenceChannel::Beauty);
        assert_eq!(
            v, 200,
            "current[Beauty] at enqueue center ({sx},{sy}) must be 200. \
             BSS bug that collapses all events to a single coord would leave \
             non-(0,0) centers at 0. Got {v}"
        );
    }

    // (b) A control tile beyond BEAUTY_MAX_RADIUS from all 3 sources must be 0.
    // (60,60): Manhattan dist to (30,30)=60, to (20,20)=80, to (10,10)=100.
    // All > 15, so no propagation can reach it.
    let control = engine
        .resources
        .influence_grid
        .sample(60, 60, InfluenceChannel::Beauty);
    assert_eq!(
        control, 0,
        "current[Beauty] at (60,60) (beyond max_radius=15 from all sources) must be 0. \
         A bug that floods all tiles regardless of source position would leave this nonzero. Got {control}"
    );
}

// ── Assertion 6: ffi_beauty_propagation_spot_check ──────────────────────────

/// Type A: current[Beauty] sum across the 3 source centers == 600 (3 × 200)
/// after FFI path + 1 tick.
///
/// T7.10.F rotation: post-Beauty-wiring, IUS drains dirty_regions[Beauty] each
/// tick. Original intent: spot-check that Beauty (a non-Warmth stamped channel)
/// is marked by BSS — a FFI bug that only marks Warmth would yield 0 here.
/// Re-formulation: sum the source-center samples; total must equal 3 × 200 = 600
/// (each BFS center receives the full intensity). A FFI bug dropping Beauty
/// channel mapping leaves all 3 at 0 → sum 0, caught here.
#[test]
fn harness_ffi_bridge_ffi_dirty_regions_non_warmth_channel_spot_check_beauty() {
    let mut engine = fresh_phase2_engine();
    enqueue_building_placed(&mut engine.resources, 10, 10, 1);
    enqueue_building_placed(&mut engine.resources, 20, 20, 1);
    enqueue_building_placed(&mut engine.resources, 30, 30, 1);
    engine.tick();
    let sum: u32 = [(10u32, 10u32), (20, 20), (30, 30)]
        .iter()
        .map(|(sx, sy)| {
            engine
                .resources
                .influence_grid
                .sample(*sx, *sy, InfluenceChannel::Beauty) as u32
        })
        .sum();
    assert_eq!(
        sum,
        600,
        "sum of current[Beauty] at 3 source centers must be 600 (3 × 200) \
         after FFI path + 1 tick. FFI channel mis-mapping that drops Beauty \
         would yield 0 here. Got {sum}"
    );
}

// ── Assertion 7a: oob_ffi_returns_false ──────────────────────────────────────

/// Type A: enqueue_building_placed(64, 0, 1) returns false (x=64 >= grid_width=64).
/// x=64 on a 64×64 grid (0-indexed max=63) is out of bounds.
/// Called AFTER 2 in-bounds enqueues so a bad OOB handler that clears the queue
/// is caught by Assertion 7b (queue.len() would drop to 0, not stay at 2).
/// An off-by-one (x > width instead of x >= width) would accept x=64 → returns true.
#[test]
fn harness_ffi_bridge_oob_ffi_returns_false() {
    let mut engine = fresh_phase2_engine();
    // Place 2 in-bounds events FIRST so a clear-on-OOB bug is detectable downstream.
    enqueue_building_placed(&mut engine.resources, 5, 5, 1);
    enqueue_building_placed(&mut engine.resources, 15, 15, 1);
    let oob_result = enqueue_building_placed(&mut engine.resources, 64, 0, 1);
    // Type A: threshold == false
    assert!(
        !oob_result,
        "enqueue_building_placed(64, 0, 1) must return false \
         (x=64 >= grid_width=64; 0-indexed max valid x=63). \
         Off-by-one (x > w) would return true here."
    );
}

// ── Assertion 7b: pre_tick_queue_len_equals_2_after_2_inbounds_1_oob ────────

/// Type A: queue.len() == 2 on fresh Fixture B after (5,5)+(15,15) accepted then (64,0) rejected.
/// OOB guard must not push_back when returning false, and must not clear the queue.
/// The 2 in-bounds calls are placed BEFORE the OOB call (anti-gaming ordering):
///   - A clear-on-OOB bug: 2 events → OOB clears to 0 → count becomes 0 (FAILS)
///   - An append-on-OOB bug: 2 events → OOB appends to 3 → count becomes 3 (FAILS)
///   - Correct behavior: 2 events → OOB rejected → count stays 2 (PASSES)
#[test]
fn harness_ffi_bridge_pre_tick_queue_len_equals_2_after_2_inbounds_1_oob() {
    let mut engine = fresh_phase2_engine();
    let ok1 = enqueue_building_placed(&mut engine.resources, 5, 5, 1);
    let ok2 = enqueue_building_placed(&mut engine.resources, 15, 15, 1);
    assert!(
        ok1 && ok2,
        "Fixture B: in-bounds enqueue calls at (5,5) and (15,15) must return true"
    );
    enqueue_building_placed(&mut engine.resources, 64, 0, 1); // OOB — expected false
    // Type A: threshold == 2
    assert_eq!(
        engine.resources.building_event_queue.len(),
        2,
        "queue.len() must be 2 after (5,5)+(15,15) accepted then (64,0) rejected. \
         OOB guard must not push_back when returning false, \
         and must not clear the queue (which already held 2 events at OOB call time)."
    );
}

// ── Assertion 8: oob_clamp_at_enqueue_time_beauty_propagates_at_2_sources ──

/// Type A: current[Beauty] == 200 at (5,5) and (15,15), and 0 at (0,0) after
/// 2 in-bounds + 1 OOB + 1 full tick.
///
/// T7.10.F rotation: post-Beauty-wiring, IUS drains dirty_regions[Beauty] each
/// tick. Original intent: OOB event rejected at enqueue time → BSS only stamps
/// 2 events (not 3, not 0). Re-formulation: 2 in-bounds enqueues yield 2
/// distinct BFS centers (each ==200); the OOB (64,0) is rejected at enqueue
/// time so (0,0) — and any coord adjacent to a hypothetical "OOB clamped to
/// edge" stamp — must stay 0. Failure modes:
///   (a) FFI accepts OOB: would stamp at clamped position → potentially
///       leak Beauty values near (0,0) or (63,0)
///   (b) double-rejection: both in-bounds rejected → (5,5)==0
#[test]
fn harness_ffi_bridge_oob_clamp_at_enqueue_time_dirty_count_equals_2() {
    let mut engine = fresh_phase2_engine();
    enqueue_building_placed(&mut engine.resources, 5, 5, 1);
    enqueue_building_placed(&mut engine.resources, 15, 15, 1);
    enqueue_building_placed(&mut engine.resources, 64, 0, 1); // OOB: rejected at enqueue
    engine.tick();
    // (a) Both in-bounds enqueues propagated.
    for (sx, sy) in [(5u32, 5u32), (15, 15)] {
        let v = engine
            .resources
            .influence_grid
            .sample(sx, sy, InfluenceChannel::Beauty);
        assert_eq!(
            v, 200,
            "current[Beauty] at in-bounds source ({sx},{sy}) must be 200; double-OOB-rejection \
             would leave this at 0. Got {v}"
        );
    }
    // (b) (50,0) is > BEAUTY_MAX_RADIUS=15 from both in-bounds sources
    // (Manhattan: 45 from (5,5), 35 from (15,15)) and equally far from the
    // hypothetical clamped OOB position (63,0). If OOB were accepted+clamped,
    // (50,0) would receive Beauty propagation (dist 13 from (63,0)). Must be 0.
    let oob_control = engine
        .resources
        .influence_grid
        .sample(50, 0, InfluenceChannel::Beauty);
    assert_eq!(
        oob_control, 0,
        "current[Beauty] at (50,0) must be 0. OOB acceptance (clamped to edge near (63,0)) \
         would leak Beauty here (dist 13 < max_radius=15). Got {oob_control}"
    );
}

// ── Assertion 9: idempotent_retick_pending_all_zero_after_5_empty_queue_ticks

/// Type A: pending[Warmth] all-zero after 5 idle ticks (ticks 2–6 from construction).
///
/// Setup (plan_attempt 3 explicit API):
///   (a) 3 FFI enqueues + 1 tick to drain (establishes post-tick-1 state)
///   (b) assert queue drained (pre-condition for idle run)
///   (c) write value 255 to all bytes of pending[Warmth] via pending_buf_mut(Warmth)
///       between tick 1 and tick 2
///   (d) run 5 idle ticks
///
/// Anti-gaming: the 255-seed ensures a stub IUS that skips clear_all_pending() is caught
/// (255 ≠ 0). Without pre-fill a hollow IUS (no-op) passes trivially (pending was already 0).
/// T7.8 assertion 2 confirms this pending_buf_mut API exists and works.
#[test]
fn harness_ffi_bridge_idempotent_retick_pending_all_zero_after_5_empty_queue_ticks() {
    let mut engine = fresh_phase2_engine();
    // (a) 3 FFI enqueues
    enqueue_building_placed(&mut engine.resources, 10, 10, 1);
    enqueue_building_placed(&mut engine.resources, 20, 20, 1);
    enqueue_building_placed(&mut engine.resources, 30, 30, 1);
    engine.tick(); // tick 1: BSS drains queue → dirty_regions = 3; IUS clears pending

    // (b) pre-condition guard
    assert_eq!(
        engine.resources.building_event_queue.len(),
        0,
        "pre-condition for idle run: queue must be empty after tick 1"
    );

    // T7.10.A..F: All 6 stamped channels (Warmth/Light/Noise/Danger/Spiritual/Beauty)
    // now have persistence semantics (copy current → pending), not a hard clear.
    // Switch to FoodAroma (the only remaining unstamped dispatch-shell channel —
    // Social would work equivalently) where pending is cleared every tick.
    // (c) seeding step — write 255 to all pending[FoodAroma] bytes between tick 1 and tick 2
    engine
        .resources
        .influence_grid
        .pending_buf_mut(InfluenceChannel::FoodAroma)
        .iter_mut()
        .for_each(|byte| *byte = 255);

    // (d) 5 idle ticks (empty queue — BSS no-op on all 5 ticks)
    for _ in 0..5 {
        engine.tick();
    }

    // Type A: threshold == true (every byte in pending[FoodAroma] == 0)
    // IUS clears pending for unstamped dispatch-shell channels every tick.
    // A stub IUS skipping clear_pending() would leave 255 in pending.
    let pending_all_zero = engine
        .resources
        .influence_grid
        .pending[InfluenceChannel::FoodAroma as usize]
        .iter()
        .all(|&byte| byte == 0);
    assert!(
        pending_all_zero,
        "pending[FoodAroma] must be all-zero after 5 idle ticks. \
         Seeded with 255 (via pending_buf_mut) before idle run to prevent vacuous pass. \
         A stub IUS skipping clear_pending() for unstamped dispatch-shell channels would leave 255 in pending."
    );
}

// ── Assertion 10: idempotent_retick_beauty_propagation_stable_after_5_idle_ticks

/// Type A: current[Beauty] at each of the 3 source centers remains 200 after
/// 5 idle ticks following the initial propagation tick.
///
/// T7.10.F rotation: post-Beauty-wiring, IUS drains dirty_regions[Beauty] each
/// tick — the original dispatch-shell-stable count assertion is no longer
/// possible. Re-formulation: Cold-tier persistence — after tick 1 propagates
/// Beauty to current, the next 5 idle ticks must preserve current[Beauty] via
/// the persistence branch (copy current → pending → swap). Failure modes:
///   (a) Persistence branch missing: current[Beauty] flickers to 0 on idle
///       ticks → assertion fails
///   (b) Erroneous re-stamping: current[Beauty] grows beyond 200 (not possible
///       with Max aggregation but caught by exact-equality check)
#[test]
fn harness_ffi_bridge_idempotent_retick_dirty_regions_stable_at_3_after_5_idle_ticks() {
    let mut engine = fresh_phase2_engine();
    enqueue_building_placed(&mut engine.resources, 10, 10, 1);
    enqueue_building_placed(&mut engine.resources, 20, 20, 1);
    enqueue_building_placed(&mut engine.resources, 30, 30, 1);
    engine.tick(); // tick 1: BFS propagates Beauty at 3 source centers

    // 5 idle ticks (ticks 2–6, empty queue — BSS no-op, IUS persistence branch)
    for _ in 0..5 {
        engine.tick();
    }

    // Type A: threshold == 200 at each source center after 5 idle ticks.
    for (sx, sy) in [(10u32, 10u32), (20, 20), (30, 30)] {
        let v = engine
            .resources
            .influence_grid
            .sample(sx, sy, InfluenceChannel::Beauty);
        assert_eq!(
            v, 200,
            "current[Beauty] at source ({sx},{sy}) must remain 200 after 5 idle ticks. \
             Cold-tier persistence branch missing would flicker to 0; got {v}"
        );
    }
}
