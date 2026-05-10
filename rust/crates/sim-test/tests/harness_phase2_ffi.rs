//! T7.7.B harness — 20 plan assertions for sim-bridge FFI mechanism.
//!
//! Assertions 1–4:   InfluenceGrid + queue construction invariants (pure Rust, no bridge call)
//! Assertions 5–6:   Bridge Identity Contract — calls `sim_bridge::ffi::enqueue_building_placed`
//!                   (the `pub fn` that `WorldSimNode::on_building_placed` MUST delegate to;
//!                    Evaluator verifies delegation is non-stub via Completeness code review)
//! Assertions 7–16:  BuildingStampSystem drain mechanics (raw push_back + manual BSS tick)
//! Assertion 17:     Empty-queue tick accumulation invariant
//! Assertion 18:     Type D — isolation regression guard (T7.6 A7 pattern)
//!
//! "BuildingStampSystem-only tick" means: call
//!   `BuildingStampSystem::new().tick(&mut e.world, &mut e.resources)`
//! directly. InfluenceUpdateSystem is EXCLUDED from all tests here to prevent
//! it consuming dirty_regions before assertions read them.
//!
//! Run: `cargo test -p sim-test harness_ffi_ -- --nocapture`

use sim_bridge::ffi::enqueue_building_placed;
use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, RuntimeSystem, SimEngine};
use sim_systems::runtime::influence::BuildingStampSystem;

/// Create a fresh 64×64 engine with NO systems registered.
/// Systems are NOT registered because tests need BuildingStampSystem-only ticks.
fn fresh_64() -> SimEngine {
    SimEngine::new(64, 64, MaterialRegistry::new())
}

/// Manually run one BuildingStampSystem tick on the engine (BSS-only, no other systems).
fn bss_tick(e: &mut SimEngine) {
    BuildingStampSystem::new().tick(&mut e.world, &mut e.resources);
}

// ─── A1: queue_field_init_empty ──────────────────────────────────────────────

/// Type A: resources.building_event_queue.len() == 0 on fresh 64×64 engine (0 ticks)
/// ticks: 0 | components_read: SimResources.building_event_queue
#[test]
fn harness_ffi_queue_field_init_empty() {
    let e = fresh_64();
    // Type A: threshold == 0
    assert_eq!(
        e.resources.building_event_queue.len(),
        0,
        "building_event_queue must start empty (VecDeque::new)"
    );
}

// ─── A2: all_8_channels_overlay_buffer_length_correct ────────────────────────

/// Type A: current_buf(ch).len() == 4096 for every InfluenceChannel on fresh 64×64 engine
/// ticks: 0 | components_read: SimResources.influence_grid
#[test]
fn harness_ffi_all_8_channels_overlay_buffer_length_correct() {
    let e = fresh_64();
    for ch in InfluenceChannel::all() {
        let len = e.resources.influence_grid.current_buf(*ch).len();
        // Type A: threshold == 4096 (64 × 64) for every channel
        assert_eq!(
            len, 4096,
            "current_buf({ch:?}).len() must be 4096 (64×64), got {len}"
        );
    }
}

// ─── A3: all_8_channels_overlay_buffer_zeros_on_fresh_engine ─────────────────

/// Type A: current_buf(ch).iter().all(|&b| b == 0) for all 8 channels on fresh 64×64 engine
/// ticks: 0 | components_read: SimResources.influence_grid
#[test]
fn harness_ffi_all_8_channels_overlay_buffer_zeros_on_fresh_engine() {
    let e = fresh_64();
    for ch in InfluenceChannel::all() {
        let all_zero = e
            .resources
            .influence_grid
            .current_buf(*ch)
            .iter()
            .all(|&b| b == 0);
        // Type A: threshold == true (zero non-zero bytes in every channel's current buffer)
        assert!(
            all_zero,
            "current_buf({ch:?}) must be all-zero on a fresh engine"
        );
    }
}

// ─── A4: tile_detail_all_8_channels_zero_on_fresh_engine ─────────────────────

/// Type A: influence_grid.sample(10, 10, ch) == 0 for all 8 channels on fresh 64×64 engine
/// ticks: 0 | components_read: SimResources.influence_grid
#[test]
fn harness_ffi_tile_detail_all_8_channels_zero_on_fresh_engine() {
    let e = fresh_64();
    for ch in InfluenceChannel::all() {
        let val = e.resources.influence_grid.sample(10, 10, *ch);
        // Type A: threshold == 0
        assert_eq!(
            val, 0,
            "sample(10, 10, {ch:?}) must be 0 on a fresh engine, got {val}"
        );
    }
}

// ─── A5: on_building_placed_inbounds_enqueues_and_returns_true ───────────────
// [Bridge Identity Contract]: calls enqueue_building_placed — the same pub fn that
// WorldSimNode::on_building_placed MUST consist solely of a forwarding call to.
// Evaluator verifies via Completeness code review that on_building_placed's #[func]
// body calls this exact symbol and is not a stub.

/// Type A: (a) enqueue_building_placed returns true for in-bounds (x=20,y=20,radius=3)
///         (b) building_event_queue.len() == 1
/// ticks: 0 | components_read: SimResources.building_event_queue
#[test]
fn harness_ffi_on_building_placed_inbounds_enqueues_and_returns_true() {
    let mut e = fresh_64();
    let ok = enqueue_building_placed(&mut e.resources, 20, 20, 3);
    // Type A: (a) return value == true
    assert!(
        ok,
        "enqueue_building_placed(20,20,3) on 64×64 grid must return true (in-bounds)"
    );
    // Type A: (b) building_event_queue.len() == 1
    assert_eq!(
        e.resources.building_event_queue.len(),
        1,
        "queue must contain exactly 1 event after one in-bounds enqueue"
    );
}

// ─── A6: on_building_placed_oob_does_not_enqueue_and_returns_false ───────────
// [Bridge Identity Contract]: same enqueue_building_placed pub fn.

/// Type A: (a) enqueue_building_placed returns false for OOB (x=999,y=999,radius=1)
///         (b) building_event_queue.len() == 0
/// ticks: 0 | components_read: SimResources.building_event_queue
#[test]
fn harness_ffi_on_building_placed_oob_does_not_enqueue_and_returns_false() {
    let mut e = fresh_64();
    let ok = enqueue_building_placed(&mut e.resources, 999, 999, 1);
    // Type A: (a) return value == false
    assert!(
        !ok,
        "enqueue_building_placed(999,999,1) on 64×64 grid must return false (OOB)"
    );
    // Type A: (b) building_event_queue.len() == 0
    assert_eq!(
        e.resources.building_event_queue.len(),
        0,
        "queue must remain empty after OOB enqueue attempt"
    );
}

// ─── A7: stamp_marks_exactly_warmth_dirty ────────────────────────────────────

/// Type A: dirty_regions[Warmth].len() == 1 after raw push_back {pos:(20,20),r:3} + 1 BSS-only tick
/// ticks: 1 (BuildingStampSystem only) | components_read: SimResources.influence_grid
#[test]
fn harness_ffi_stamp_marks_exactly_warmth_dirty() {
    let mut e = fresh_64();
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (20, 20),
        radius: 3,
    });
    bss_tick(&mut e);
    let len =
        e.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize].len();
    // Type A: threshold == 1
    assert_eq!(len, 1, "Warmth dirty_regions.len() must be 1, got {len}");
}

// ─── A8: stamp_marks_exactly_spiritual_dirty ─────────────────────────────────

/// Type A: dirty_regions[Spiritual].len() == 1 after raw push_back {pos:(20,20),r:3} + 1 BSS-only tick
/// ticks: 1 (BuildingStampSystem only) | components_read: SimResources.influence_grid
#[test]
fn harness_ffi_stamp_marks_exactly_spiritual_dirty() {
    let mut e = fresh_64();
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (20, 20),
        radius: 3,
    });
    bss_tick(&mut e);
    let len =
        e.resources.influence_grid.dirty_regions[InfluenceChannel::Spiritual as usize].len();
    // Type A: threshold == 1
    assert_eq!(len, 1, "Spiritual dirty_regions.len() must be 1, got {len}");
}

// ─── A9: stamp_marks_exactly_beauty_dirty ────────────────────────────────────

/// Type A: dirty_regions[Beauty].len() == 1 after raw push_back {pos:(20,20),r:3} + 1 BSS-only tick
/// ticks: 1 (BuildingStampSystem only) | components_read: SimResources.influence_grid
#[test]
fn harness_ffi_stamp_marks_exactly_beauty_dirty() {
    let mut e = fresh_64();
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (20, 20),
        radius: 3,
    });
    bss_tick(&mut e);
    let len =
        e.resources.influence_grid.dirty_regions[InfluenceChannel::Beauty as usize].len();
    // Type A: threshold == 1
    assert_eq!(len, 1, "Beauty dirty_regions.len() must be 1, got {len}");
}

// ─── A10: stamp_marks_exactly_light_dirty ────────────────────────────────────

/// Type A: dirty_regions[Light].len() == 1 after raw push_back {pos:(20,20),r:3} + 1 BSS-only tick
/// ticks: 1 (BuildingStampSystem only) | components_read: SimResources.influence_grid
#[test]
fn harness_ffi_stamp_marks_exactly_light_dirty() {
    let mut e = fresh_64();
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (20, 20),
        radius: 3,
    });
    bss_tick(&mut e);
    let len =
        e.resources.influence_grid.dirty_regions[InfluenceChannel::Light as usize].len();
    // Type A: threshold == 1
    assert_eq!(len, 1, "Light dirty_regions.len() must be 1, got {len}");
}

// ─── A11: non_stamped_channels_produce_zero_dirty_regions ────────────────────

/// Type A: dirty_regions[{Danger,Noise,FoodAroma,Social}].len() == 0 after same setup
/// STAMPED_CHANNELS = {Warmth, Spiritual, Beauty, Light} exactly (4 channels, locked per spec)
/// ticks: 1 (BuildingStampSystem only) | components_read: SimResources.influence_grid
#[test]
fn harness_ffi_non_stamped_channels_produce_zero_dirty_regions() {
    let mut e = fresh_64();
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (20, 20),
        radius: 3,
    });
    bss_tick(&mut e);
    for ch in [
        InfluenceChannel::Danger,
        InfluenceChannel::Noise,
        InfluenceChannel::FoodAroma,
        InfluenceChannel::Social,
    ] {
        let len = e.resources.influence_grid.dirty_regions[ch as usize].len();
        // Type A: threshold == 0 for all four non-stamped channels
        assert_eq!(
            len, 0,
            "non-stamped channel {ch:?} dirty_regions.len() must be 0, got {len}"
        );
    }
}

// ─── A12: oob_event_discarded_no_dirty_regions_queue_drained ─────────────────

/// Type A: (a) all 8 dirty_regions.len() == 0; (b) queue.len() == 0
///         after raw push_back {pos:(999,999),r:1} + 1 BSS-only tick on 64×64 engine
/// ticks: 1 (BuildingStampSystem only)
/// components_read: SimResources.building_event_queue, SimResources.influence_grid
#[test]
fn harness_ffi_oob_event_discarded_no_dirty_regions_queue_drained() {
    let mut e = fresh_64();
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (999, 999),
        radius: 1,
    });
    bss_tick(&mut e);
    // Type A: (b) queue.len() == 0 (event consumed via pop_front before OOB check)
    assert_eq!(
        e.resources.building_event_queue.len(),
        0,
        "queue must be empty after OOB event tick (pop_front before bounds check)"
    );
    // Type A: (a) all 8 dirty_regions.len() == 0
    for ch in InfluenceChannel::all() {
        let len = e.resources.influence_grid.dirty_regions[*ch as usize].len();
        assert_eq!(
            len, 0,
            "dirty_regions[{ch:?}].len() must be 0 after OOB event, got {len}"
        );
    }
}

// ─── A13: dirty_region_coordinate_bounds_match_input ─────────────────────────

/// Type A: Warmth dirty_regions[0] bounding box for {pos:(20,20),r:3}:
///   min_x == 17 ∧ max_x == 23 ∧ min_y == 17 ∧ max_y == 23 (exact Chebyshev box)
///
/// Arithmetic: x1 = 20.saturating_sub(3) = 17, x2 = 20.saturating_add(3).min(63) = 23
///             y1 = 20.saturating_sub(3) = 17, y2 = 20.saturating_add(3).min(63) = 23
/// ticks: 1 (BuildingStampSystem only) | components_read: SimResources.influence_grid
#[test]
fn harness_ffi_dirty_region_coordinate_bounds_match_input() {
    let mut e = fresh_64();
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (20, 20),
        radius: 3,
    });
    bss_tick(&mut e);
    let regs =
        &e.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize];
    assert_eq!(regs.len(), 1, "expected exactly 1 dirty region for Warmth");
    let r = &regs[0];
    // Type A: exact spatial bounds {17,17,23,23} — Chebyshev box 20±3
    assert_eq!(r.min_x, 17, "min_x must be exactly 17 (20−3=17), got {}", r.min_x);
    assert_eq!(r.max_x, 23, "max_x must be exactly 23 (20+3=23), got {}", r.max_x);
    assert_eq!(r.min_y, 17, "min_y must be exactly 17 (20−3=17), got {}", r.min_y);
    assert_eq!(r.max_y, 23, "max_y must be exactly 23 (20+3=23), got {}", r.max_y);
}

// ─── A14: three_events_all_drained_in_single_tick ────────────────────────────

/// Type A: (a) queue.len() == 0; (b) dirty_regions[Warmth].len() == 3
///         after 3 raw push_backs + 1 BSS-only tick
/// ticks: 1 (BuildingStampSystem only)
/// components_read: SimResources.building_event_queue, SimResources.influence_grid
#[test]
fn harness_ffi_three_events_all_drained_in_single_tick() {
    let mut e = fresh_64();
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (10, 10),
        radius: 2,
    });
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (30, 30),
        radius: 1,
    });
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (50, 50),
        radius: 5,
    });
    bss_tick(&mut e);
    // Type A: (a) queue.len() == 0 (while-loop drains all, not just first)
    assert_eq!(
        e.resources.building_event_queue.len(),
        0,
        "queue must be empty after 3-event drain tick (while-loop required, not single pop)"
    );
    let warmth_len =
        e.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize].len();
    // Type A: (b) dirty_regions[Warmth].len() == 3
    assert_eq!(
        warmth_len, 3,
        "Warmth dirty_regions.len() must be 3 after 3-event drain, got {warmth_len}"
    );
}

// ─── A15: three_events_all_4_stamped_channels_three_dirty_regions ─────────────

/// Type A: dirty_regions[{Warmth,Spiritual,Beauty,Light}].len() == 3 each
///         after 3 raw push_backs + 1 BSS-only tick
/// ticks: 1 (BuildingStampSystem only) | components_read: SimResources.influence_grid
#[test]
fn harness_ffi_three_events_all_4_stamped_channels_three_dirty_regions() {
    let mut e = fresh_64();
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (10, 10),
        radius: 2,
    });
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (30, 30),
        radius: 1,
    });
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (50, 50),
        radius: 5,
    });
    bss_tick(&mut e);
    for ch in [
        InfluenceChannel::Warmth,
        InfluenceChannel::Spiritual,
        InfluenceChannel::Beauty,
        InfluenceChannel::Light,
    ] {
        let len = e.resources.influence_grid.dirty_regions[ch as usize].len();
        // Type A: threshold == 3 for each of the 4 stamped channels
        assert_eq!(
            len, 3,
            "dirty_regions[{ch:?}].len() must be 3 for a 3-event batch, got {len}"
        );
    }
}

// ─── A16: large_radius_clamped_no_panic_yields_gte1_dirty_region_within_bounds ─

/// Type A: (a) no panic; (b) dirty_regions[Warmth].len() == 1;
///         (c) exact clamped bounds {min_x:0, min_y:0, max_x:63, max_y:63}
///         after push_back {pos:(1,1),r:100} + 1 BSS-only tick on 64×64 engine
///
/// Arithmetic: x1 = 1.saturating_sub(100) = 0, y1 = 0
///             x2 = 1.saturating_add(100).min(63) = 63, y2 = 63
/// ticks: 1 (BuildingStampSystem only) | components_read: SimResources.influence_grid
#[test]
fn harness_ffi_large_radius_clamped_no_panic_yields_gte1_dirty_region_within_bounds() {
    let mut e = fresh_64();
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (1, 1),
        radius: 100, // saturating arithmetic + grid-clamp required
    });
    // Type A: (a) no panic — test returning normally = pass
    bss_tick(&mut e);
    let regs =
        &e.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize];
    // Type A: (b) len == 1 (one event → one dirty region)
    assert_eq!(
        regs.len(),
        1,
        "dirty_regions[Warmth] must have exactly 1 entry after large-radius event at (1,1)"
    );
    // Type A: (c) exact clamped bounds {0, 0, 63, 63}
    let r = &regs[0];
    assert_eq!(r.min_x, 0, "min_x must be exactly 0 (1.saturating_sub(100)=0), got {}", r.min_x);
    assert_eq!(r.min_y, 0, "min_y must be exactly 0 (1.saturating_sub(100)=0), got {}", r.min_y);
    assert_eq!(r.max_x, 63, "max_x must be exactly 63 (clamped to w−1=63), got {}", r.max_x);
    assert_eq!(r.max_y, 63, "max_y must be exactly 63 (clamped to h−1=63), got {}", r.max_y);
}

// ─── A17-mixed: mixed_oob_then_valid_event_proves_continue_not_break ─────────

/// Type A: OOB event followed by valid event →
///   (a) queue.len() == 0 after 1 BSS-only tick
///   (b) dirty_regions[{Warmth,Spiritual,Beauty,Light}].len() == 1 each
///   (c) dirty_regions[{Danger,Noise,FoodAroma,Social}].len() == 0 each
///
/// Proves the OOB guard uses `continue` — not `return` or `break`:
///   - `return`  would exit tick() entirely, leaving queue non-empty.
///   - `break`   would exit the while-loop, leaving queue non-empty.
///   - `continue` skips only the OOB event; valid event is then processed.
///
/// ticks: 1 (BuildingStampSystem only)
/// components_read: SimResources.building_event_queue, SimResources.influence_grid
#[test]
fn harness_ffi_mixed_oob_then_valid_event_proves_continue_not_break() {
    let mut e = fresh_64();
    // First: OOB event — must be discarded via `continue`, NOT via `return`/`break`
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (999, 999),
        radius: 1,
    });
    // Second: valid event — must be processed despite the preceding OOB event
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (30, 30),
        radius: 2,
    });
    bss_tick(&mut e);
    // Type A: (a) queue fully drained — both events consumed via pop_front
    assert_eq!(
        e.resources.building_event_queue.len(),
        0,
        "queue must be empty: OOB event discarded via `continue`, valid event processed"
    );
    // Type A: (b) 4 stamped channels have exactly 1 dirty region (from the valid event)
    for ch in [
        InfluenceChannel::Warmth,
        InfluenceChannel::Spiritual,
        InfluenceChannel::Beauty,
        InfluenceChannel::Light,
    ] {
        let len = e.resources.influence_grid.dirty_regions[ch as usize].len();
        assert_eq!(
            len,
            1,
            "{ch:?} must have exactly 1 dirty region (valid event processed after OOB via `continue`), got {len}"
        );
    }
    // Type A: (c) 4 non-stamped channels remain clean
    for ch in [
        InfluenceChannel::Danger,
        InfluenceChannel::Noise,
        InfluenceChannel::FoodAroma,
        InfluenceChannel::Social,
    ] {
        let len = e.resources.influence_grid.dirty_regions[ch as usize].len();
        assert_eq!(
            len,
            0,
            "non-stamped {ch:?} must have 0 dirty regions, got {len}"
        );
    }
}

// ─── A17: no_dirty_regions_accumulate_on_empty_queue_tick ────────────────────

/// Type A: dirty_regions[Warmth].len() == 1 after tick1 (1 event),
///                                        == 1 after tick2 (empty queue, unchanged)
/// ticks: 2 (both BSS-only; InfluenceUpdateSystem excluded from both)
/// components_read: SimResources.influence_grid
#[test]
fn harness_ffi_no_dirty_regions_accumulate_on_empty_queue_tick() {
    let mut e = fresh_64();
    // Tick 1: one event → BSS drains and marks dirty
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (20, 20),
        radius: 3,
    });
    bss_tick(&mut e);
    let after_tick1 =
        e.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize].len();
    // Type A: (a) after tick1 == 1
    assert_eq!(
        after_tick1, 1,
        "after tick1 Warmth dirty_regions.len() must be 1, got {after_tick1}"
    );

    // Tick 2: empty queue → BSS is a no-op (neither adds nor removes dirty regions)
    bss_tick(&mut e);
    let after_tick2 =
        e.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize].len();
    // Type A: (b) after tick2 == 1 (unchanged — BSS neither adds nor removes)
    assert_eq!(
        after_tick2, 1,
        "after tick2 (empty queue) Warmth dirty_regions.len() must still be 1, got {after_tick2}"
    );
}

// ─── A18: isolation_invariant_survives_empty_queue_tick ──────────────────────
// Type D: regression guard for T7.6 harness_building_stamp_isolation (A7 in harness_phase2.rs).
// Ensures T7.7.B's new drain code does NOT regress T7.6's isolation invariant.

/// Type D: pending_buf_mut(Warmth)[5*32+5]=200 survives empty-queue BSS tick + swap
///         → sample(5,5,Warmth) == 200
/// ticks: 1 (manual BuildingStampSystem.tick() only, NOT engine.tick())
/// components_read: SimResources.influence_grid
#[test]
fn harness_ffi_isolation_invariant_survives_empty_queue_tick() {
    // 32×32 engine (matches T7.6 A7 pattern)
    let mut e = SimEngine::new(32, 32, MaterialRegistry::new());
    // (1) Write 200 into pending buffer at tile (5, 5): idx = y*width + x = 5*32+5 = 165
    let idx = 5 * 32 + 5;
    e.resources
        .influence_grid
        .pending_buf_mut(InfluenceChannel::Warmth)[idx] = 200;
    // (2) BSS tick with empty queue — MUST NOT touch pending (isolation invariant)
    BuildingStampSystem::new().tick(&mut e.world, &mut e.resources);
    // (3) Swap pending → current
    e.resources.influence_grid.swap();
    // Type D: value 200 must survive (BSS empty-queue tick must not clobber pending)
    let val = e.resources.influence_grid.sample(5, 5, InfluenceChannel::Warmth);
    assert_eq!(
        val, 200,
        "sample(5,5,Warmth) must be 200 — BSS isolation invariant must survive T7.7.B drain code"
    );
}

// ─── A19: dirty_regions_all_channels_empty_at_construction ───────────────────

/// Type A: all 8 channel dirty_regions.len() == 0 on a fresh 64×64 engine (0 ticks).
///
/// Construction invariant: `InfluenceGrid::new()` must produce empty dirty_regions
/// for every channel. This is the required baseline that makes Assertions 7–18
/// meaningful — if dirty_regions were non-empty at start, all drain/stamp tests
/// would be polluted.
///
/// ticks: 0 | components_read: SimResources.influence_grid
#[test]
fn harness_ffi_dirty_regions_all_channels_empty_at_construction() {
    let e = fresh_64();
    for ch in InfluenceChannel::all() {
        let len = e.resources.influence_grid.dirty_regions[*ch as usize].len();
        // Type A: threshold == 0 (construction invariant)
        assert_eq!(
            len,
            0,
            "dirty_regions[{ch:?}].len() must be 0 at construction, got {len}"
        );
    }
}

// ─── A20: radius_zero_produces_single_pixel_dirty_region ─────────────────────

/// Type A: `BuildingPlacedEvent { pos:(63,63), radius:0 }` produces exactly one
/// DirtyRegion per stamped channel with **exact** bounds (63,63,63,63).
///
/// Corner-case rationale (max grid tile):
/// - min_x == 63: 63.saturating_sub(0) == 63 — no underflow
/// - max_x == 63: 63.saturating_add(0).min(63) == 63 — no expansion, already at limit
/// - min_y == 63, max_y == 63: same logic on y axis
///
/// Catches implementations that treat radius=0 as "no stamp" (would yield len==0),
/// that expand radius=0 to a default minimum (would yield bounds != (63,63,63,63)),
/// or that mis-clamp the max grid corner (63,63) as OOB (would yield len==0).
///
/// ticks: 1 (BuildingStampSystem only) | components_read: SimResources.influence_grid
#[test]
fn harness_ffi_radius_zero_produces_single_pixel_dirty_region() {
    let mut e = fresh_64();
    e.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (63, 63), // max-corner of 64×64 grid (inclusive)
        radius: 0,
    });
    bss_tick(&mut e);

    for ch in [
        InfluenceChannel::Warmth,
        InfluenceChannel::Spiritual,
        InfluenceChannel::Beauty,
        InfluenceChannel::Light,
    ] {
        let regs = &e.resources.influence_grid.dirty_regions[ch as usize];
        let len = regs.len();
        // Type A: threshold == 1 (radius=0 must not be treated as "no stamp")
        assert_eq!(
            len,
            1,
            "{ch:?} dirty_regions.len() must be 1 for radius=0 corner event, got {len}"
        );
        let r = &regs[0];
        // Type A: exact spatial bounds (63,63,63,63) — single-pixel at grid corner
        assert_eq!(
            r.min_x, 63,
            "{ch:?} min_x={} must be exactly 63 (radius=0 corner at x=63)",
            r.min_x
        );
        assert_eq!(
            r.max_x, 63,
            "{ch:?} max_x={} must be exactly 63 (radius=0 corner at x=63)",
            r.max_x
        );
        assert_eq!(
            r.min_y, 63,
            "{ch:?} min_y={} must be exactly 63 (radius=0 corner at y=63)",
            r.min_y
        );
        assert_eq!(
            r.max_y, 63,
            "{ch:?} max_y={} must be exactly 63 (radius=0 corner at y=63)",
            r.max_y
        );
    }

    // Non-stamped channels must remain untouched.
    for ch in [
        InfluenceChannel::Danger,
        InfluenceChannel::Noise,
        InfluenceChannel::FoodAroma,
        InfluenceChannel::Social,
    ] {
        let len = e.resources.influence_grid.dirty_regions[ch as usize].len();
        // Type A: threshold == 0 (radius=0 event must not spill to non-stamped channels)
        assert_eq!(
            len,
            0,
            "non-stamped {ch:?} must have 0 dirty regions for radius=0 event, got {len}"
        );
    }

    // Queue must be fully drained.
    assert_eq!(
        e.resources.building_event_queue.len(),
        0,
        "queue must be empty after radius=0 event is processed"
    );
}
