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

// ── Assertion 4: ffi_dirty_regions_warmth_count_3_after_ffi_enqueue_path ────

/// Type C: dirty_regions[Spiritual].len() == 3 after 3 FFI enqueues + 1 full tick.
///
/// T7.10.A relaxation: switched from Warmth to Spiritual.
/// T7.10.A wires Warmth: IUS drains dirty_regions[Warmth] via std::mem::take each tick,
/// so dirty_regions[Warmth].len() == 0 after any full tick. Spiritual remains on the
/// Phase 2 dispatch shell (IUS never drains it) → dirty_regions[Spiritual] accumulates.
/// Baseline: T7.7.B A14(b) + T7.8 assertion 6 both observed 3 for BSS stamping all
/// stamped channels. This assertion extends that observation to the FFI path.
/// Re-calibrate for Spiritual when T7.10.B wires the Spiritual channel.
#[test]
fn harness_ffi_bridge_ffi_dirty_regions_warmth_count_3_after_ffi_enqueue_path() {
    let mut engine = fresh_phase2_engine();
    enqueue_building_placed(&mut engine.resources, 10, 10, 1);
    enqueue_building_placed(&mut engine.resources, 20, 20, 1);
    enqueue_building_placed(&mut engine.resources, 30, 30, 1);
    engine.tick();
    // Type C: threshold == 3 (Spiritual accumulates; Warmth was drained by T7.10.A IUS)
    let len =
        engine.resources.influence_grid.dirty_regions[InfluenceChannel::Spiritual as usize].len();
    assert_eq!(
        len,
        3,
        "dirty_regions[Spiritual].len() must be 3 after 3 FFI events + 1 full tick. \
         Spiritual remains on Phase 2 dispatch shell (IUS does not drain it). \
         T7.10.A drains Warmth instead — re-calibrate for Spiritual when T7.10.B wired. Got {len}"
    );
}

// ── Assertion 5: ffi_dirty_regions_exact_bounds_match_enqueue_coordinates ───

/// Type A: exact DirtyRegion field values (min_x/min_y/max_x/max_y: u32) for
/// enqueue coordinates (10,10), (20,20), (30,30) all r=1 on 64×64 grid.
///
/// Expected tuple set {(min_x, min_y, max_x, max_y)} per plan_attempt 3:
///   (10,10) r=1: min_x=9,  min_y=9,  max_x=11, max_y=11 → (9,9,11,11)
///   (20,20) r=1: min_x=19, min_y=19, max_x=21, max_y=21 → (19,19,21,21)
///   (30,30) r=1: min_x=29, min_y=29, max_x=31, max_y=31 → (29,29,31,31)
///
/// BSS formula (confirmed from building_stamp.rs):
///   min_x = cx.saturating_sub(r),  max_x = (cx + r).min(w - 1)
///   min_y = cy.saturating_sub(r),  max_y = (cy + r).min(h - 1)
///
/// Order-independent search: each expected region must appear in dirty_regions exactly once.
/// Three distinct indices (no duplicates) proves position field is propagated, not ignored.
#[test]
fn harness_ffi_bridge_ffi_dirty_regions_exact_bounds_match_enqueue_coordinates() {
    let mut engine = fresh_phase2_engine();
    enqueue_building_placed(&mut engine.resources, 10, 10, 1);
    enqueue_building_placed(&mut engine.resources, 20, 20, 1);
    enqueue_building_placed(&mut engine.resources, 30, 30, 1);
    engine.tick();

    // T7.10.A: IUS now drains dirty_regions[Warmth] via std::mem::take.
    // Spiritual remains in dispatch shell, preserves original dispatch-shell coverage.
    let regs =
        &engine.resources.influence_grid.dirty_regions[InfluenceChannel::Spiritual as usize];
    assert_eq!(
        regs.len(),
        3,
        "must have exactly 3 dirty regions (one per FFI enqueue coordinate)"
    );

    // Collect actual (min_x, min_y, max_x, max_y) tuples for readable error output.
    let actual: Vec<(u32, u32, u32, u32)> =
        regs.iter().map(|r| (r.min_x, r.min_y, r.max_x, r.max_y)).collect();

    // Expected exact DirtyRegion bounds per plan_attempt 3 — derived from BSS formula.
    let expected: [(u32, u32, u32, u32); 3] = [
        (9, 9, 11, 11),   // (10,10) r=1: 10-1=9, 10+1=11
        (19, 19, 21, 21), // (20,20) r=1: 20-1=19, 20+1=21
        (29, 29, 31, 31), // (30,30) r=1: 30-1=29, 30+1=31
    ];

    // (a) each expected bound is covered by exactly 1 dirty region
    let mut covering_indices: [usize; 3] = [usize::MAX; 3];
    for (coord_idx, (ex_min_x, ex_min_y, ex_max_x, ex_max_y)) in expected.iter().enumerate() {
        let mut found_idx: Option<usize> = None;
        let mut found_count: usize = 0;
        for (ridx, &(ax, ay, bx, by)) in actual.iter().enumerate() {
            if ax == *ex_min_x && ay == *ex_min_y && bx == *ex_max_x && by == *ex_max_y {
                found_count += 1;
                if found_idx.is_none() {
                    found_idx = Some(ridx);
                }
            }
        }
        assert_eq!(
            found_count,
            1,
            "Type A: expected DirtyRegion ({},{},{},{}) must appear exactly once in dirty_regions. \
             Found {found_count} times. Actual regions: {actual:?}",
            ex_min_x, ex_min_y, ex_max_x, ex_max_y
        );
        covering_indices[coord_idx] = found_idx.unwrap();
    }

    // (b) all three covering indices must be distinct — proves one-to-one mapping
    let (i0, i1, i2) = (covering_indices[0], covering_indices[1], covering_indices[2]);
    assert!(
        i0 != i1 && i0 != i2 && i1 != i2,
        "Type A: covering region indices ({i0},{i1},{i2}) must all be distinct. \
         BSS producing 3 identical dirty regions (e.g., all at (0,0)) would map all \
         three expected coords to the same index, revealing position field not propagated. \
         Actual regions: {actual:?}"
    );
}

// ── Assertion 6: ffi_dirty_regions_non_warmth_channel_spot_check_spiritual ──

/// Type C: dirty_regions[Spiritual].len() == 3 after FFI path + 1 tick.
/// Spiritual is the representative non-Warmth stamped channel.
/// A FFI bug that only marks Warmth and drops Spiritual/Beauty/Light would
/// leave dirty_regions[Spiritual].len()==0 while Assertion 4's Warmth=3 still passes.
/// Re-calibrate when Phase 3 BFS is wired.
#[test]
fn harness_ffi_bridge_ffi_dirty_regions_non_warmth_channel_spot_check_spiritual() {
    let mut engine = fresh_phase2_engine();
    enqueue_building_placed(&mut engine.resources, 10, 10, 1);
    enqueue_building_placed(&mut engine.resources, 20, 20, 1);
    enqueue_building_placed(&mut engine.resources, 30, 30, 1);
    engine.tick();
    // Type C: threshold == 3
    let len =
        engine.resources.influence_grid.dirty_regions[InfluenceChannel::Spiritual as usize].len();
    assert_eq!(
        len,
        3,
        "dirty_regions[Spiritual].len() must be 3 after FFI path + 1 tick \
         (all 4 stamped channels must be marked; FFI channel mis-mapping would yield 0 here). \
         Got {len}"
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

// ── Assertion 8: oob_clamp_at_enqueue_time_dirty_count_equals_2 ─────────────

/// Type C: dirty_regions[Warmth].len() == 2 after 2 in-bounds + 1 OOB + 1 full tick.
/// OOB event rejected at enqueue time (not at BSS drain time) → BSS only stamps 2 events.
/// Confirms FFI OOB guard and pipeline OOB guard are consistent:
///   neither double-filters (count=0) nor neither-filters (count=3).
#[test]
fn harness_ffi_bridge_oob_clamp_at_enqueue_time_dirty_count_equals_2() {
    let mut engine = fresh_phase2_engine();
    enqueue_building_placed(&mut engine.resources, 5, 5, 1);
    enqueue_building_placed(&mut engine.resources, 15, 15, 1);
    enqueue_building_placed(&mut engine.resources, 64, 0, 1); // OOB: rejected at enqueue
    engine.tick();
    // T7.10.A: IUS now drains dirty_regions[Warmth]; switch to Spiritual (dispatch shell intact).
    // Type C: threshold == 2
    let len =
        engine.resources.influence_grid.dirty_regions[InfluenceChannel::Spiritual as usize].len();
    assert_eq!(
        len,
        2,
        "dirty_regions[Spiritual].len() must be 2 (2 in-bounds accepted, 1 OOB rejected at enqueue). \
         Got {len}"
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

    // T7.10.A: Warmth pending now reflects Cold-tier persistence (copy current → pending),
    // not a hard clear. Switch to Spiritual (dispatch shell intact, pending cleared every tick)
    // to preserve the anti-stub-IUS coverage this assertion provides.
    // (c) seeding step — write 255 to all pending[Spiritual] bytes between tick 1 and tick 2
    engine
        .resources
        .influence_grid
        .pending_buf_mut(InfluenceChannel::Spiritual)
        .iter_mut()
        .for_each(|byte| *byte = 255);

    // (d) 5 idle ticks (empty queue — BSS no-op on all 5 ticks)
    for _ in 0..5 {
        engine.tick();
    }

    // Type A: threshold == true (every byte in pending[Spiritual] == 0)
    // IUS clears pending for non-Warmth stamped channels every tick (dispatch shell).
    // A stub IUS skipping clear_all_pending() would leave 255 in pending.
    let pending_all_zero = engine
        .resources
        .influence_grid
        .pending[InfluenceChannel::Spiritual as usize]
        .iter()
        .all(|&byte| byte == 0);
    assert!(
        pending_all_zero,
        "pending[Spiritual] must be all-zero after 5 idle ticks. \
         Seeded with 255 (via pending_buf_mut) before idle run to prevent vacuous pass. \
         A stub IUS skipping clear_all_pending() for non-Warmth would leave 255 in pending."
    );
}

// ── Assertion 10: idempotent_retick_dirty_regions_stable_at_3_after_5_idle_ticks

/// Type C: dirty_regions[Warmth].len() remains 3 after 5 idle ticks (unchanged from Assertion 4).
/// With empty queue on idle ticks (ticks 2–6), BSS adds no new dirty regions.
/// IUS Phase 2 shell does NOT call clear_dirty() → count stays at 3.
/// Failure modes:
///   (a) erroneous idle clear_dirty() drops count to 0
///   (b) phantom BSS stamping grows count above 3
/// Re-calibrate when Phase 3 BFS is wired.
#[test]
fn harness_ffi_bridge_idempotent_retick_dirty_regions_stable_at_3_after_5_idle_ticks() {
    let mut engine = fresh_phase2_engine();
    enqueue_building_placed(&mut engine.resources, 10, 10, 1);
    enqueue_building_placed(&mut engine.resources, 20, 20, 1);
    enqueue_building_placed(&mut engine.resources, 30, 30, 1);
    engine.tick(); // tick 1: establishes 3 dirty regions

    // 5 idle ticks (ticks 2–6, empty queue — BSS no-op)
    for _ in 0..5 {
        engine.tick();
    }

    // T7.10.A: IUS now drains dirty_regions[Warmth] during BFS propagation.
    // Switch to Spiritual (still dispatch shell, IUS does not clear its dirty regions).
    // Type C: threshold == 3 (unchanged from Assertion 4's post-tick-1 count)
    let len =
        engine.resources.influence_grid.dirty_regions[InfluenceChannel::Spiritual as usize].len();
    assert_eq!(
        len,
        3,
        "dirty_regions[Spiritual].len() must remain 3 after 5 idle ticks \
         (BSS no-op on empty queue; IUS Phase 2 dispatch shell does not call clear_dirty() for non-Warmth). \
         Erroneous idle clear_dirty() would drop to 0. Got {len}"
    );
}
