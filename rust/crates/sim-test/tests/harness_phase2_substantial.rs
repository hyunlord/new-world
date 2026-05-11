//! T7.8 — Substantial Phase 2 harness: full-pipeline integration, long-run,
//! 1K-agent, and edge-case scenarios.
//!
//! Complements T7.7.B mechanism tests (`harness_phase2_ffi.rs`) by exercising
//! the complete 4-system pipeline end-to-end via `engine.tick()`.
//!
//! Key architecture facts tested here (NOT T7.7.B BSS-only ticks):
//!   - BSS (priority  90): drains queue, marks dirty_regions, does NOT touch pending
//!   - IUS (priority 100): calls clear_all_pending() + swap(), does NOT call clear_dirty()
//!   - AIS (priority 110): reads current buffer, writes to InfluenceSample
//!   - Viz (priority 1000): fires every 6 ticks, captures digest
//!
//! dirty_regions ACCUMULATE across full engine ticks because IUS never calls
//! clear_dirty(). This is the defining invariant of Phase 2 (Phase 3 will consume
//! dirty_regions via BFS propagation).
//!
//! PHASE 2 SCOPE NOTE: any assertion about current[] being zero is a Phase 2
//! architectural invariant only. When Phase 3 BFS propagation writes to pending
//! before IUS swaps, those assertions will legitimately fail.
//!
//! Run: `cargo test -p sim-test harness_substantial_ -- --nocapture`

use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, SimEngine};
use sim_systems::register_phase2_systems;
use sim_systems::runtime::influence::agent_sample::{InfluenceSample, Position};

const W: u32 = 64;
const H: u32 = 64;

/// Create a fresh 64×64 engine with all 4 Phase 2 systems registered.
fn fresh_phase2_engine() -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    engine
}

// ── Plan Assertion 1: causal_chain_queue_drains_and_warmth_dirty_marked ───────

/// Type A: (a) building_event_queue.len() == 0  AND
///         (b) dirty_regions[Warmth].len() == 1
/// after 1 full engine.tick() with 1 building event at (20,20) r=3.
///
/// Anti-gaming: (a) alone could mean BSS ran with an already-empty queue;
/// (b) alone could mean dirty state was left by a prior test (ruled out by
/// construction invariant T7.7.B A19). Both together prove BSS ran AND
/// processed this specific event in this full-pipeline tick.
///
/// ticks: 1 (engine.tick() — all 4 Phase 2 systems)
/// components_read: SimResources.building_event_queue, influence_grid.dirty_regions
#[test]
fn harness_substantial_causal_chain_queue_drains_and_warmth_dirty_marked() {
    let mut engine = fresh_phase2_engine();
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (20, 20),
        radius: 3,
    });
    engine.tick();

    // Type A: (a) threshold == 0 — BSS drains entire queue via while-let-pop_front
    let queue_len = engine.resources.building_event_queue.len();
    assert_eq!(
        queue_len,
        0,
        "building_event_queue must be empty after 1 tick (BSS drains via while-let-pop_front)"
    );

    // Type A: (b) threshold == 1 — BSS marked Warmth dirty; IUS does NOT call clear_dirty()
    let warmth_dirty =
        engine.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize].len();
    assert_eq!(
        warmth_dirty,
        1,
        "dirty_regions[Warmth].len() must be 1 after 1 tick with 1 in-bounds event; \
         IUS does not call clear_dirty()"
    );
}

// ── Plan Assertion 2: phase2_shell_pending_all_zero_after_full_pipeline_tick ──

/// Type A: pending[Warmth].iter().all(|&b| b == 0) after 1 full engine.tick()
/// with 1 building event at (20,20) r=3.
///
/// Setup: pending[Warmth] is pre-filled with 255 to make the assertion non-trivial.
/// Without pre-fill a hollow IUS (no-op) would trivially pass (pending was already 0).
///
/// Two-fact proof:
///   (1) BSS does not write pending buffers (isolation invariant, T7.6 A7, T7.7.B A18)
///   (2) IUS calls clear_all_pending() every tick
/// → after 1 tick, pending[Warmth] must be all-zero regardless of queued events.
///
/// ticks: 1 (engine.tick() — all 4 Phase 2 systems)
/// components_read: SimResources.influence_grid.pending[Warmth]
#[test]
fn harness_substantial_phase2_shell_pending_all_zero_after_full_pipeline_tick() {
    let mut engine = fresh_phase2_engine();

    // Pre-fill pending[Warmth] with 255 so a hollow IUS (no clear) is detectable
    engine
        .resources
        .influence_grid
        .pending_buf_mut(InfluenceChannel::Warmth)
        .iter_mut()
        .for_each(|b| *b = 255);

    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (20, 20),
        radius: 3,
    });
    engine.tick();

    // Type A: threshold == true — IUS called clear_all_pending() before swap()
    // If IUS skipped clear, the 255s would survive into current (caught below).
    let pending_all_zero =
        engine.resources.influence_grid.pending[InfluenceChannel::Warmth as usize]
            .iter()
            .all(|&b| b == 0);
    assert!(
        pending_all_zero,
        "pending[Warmth] must be all-zero after full pipeline tick \
         (IUS must call clear_all_pending() before swap())"
    );

    // Type A: ALSO assert current_buf(Warmth) is all-zero.
    // Discrimination proof: a swap-only IUS (swap() without clear_all_pending())
    // moves the 255-seeded pending INTO current after the swap, leaving
    // current[Warmth] == 255 everywhere — this assertion catches that case.
    //
    //   Correct IUS: clear_all_pending() [pending → 0], swap() [current ← 0] → current == 0 ✓
    //   Swap-only:   no clear, swap() [current ← 255]                          → current == 255 ✗
    //   No-op IUS:   neither clears nor swaps                                  → current == 0 (start value, passes)
    //     (no-op is caught by A3's non-zero current pre-seed)
    let current_all_zero = engine
        .resources
        .influence_grid
        .current_buf(InfluenceChannel::Warmth)
        .iter()
        .all(|&b| b == 0);
    assert!(
        current_all_zero,
        "current[Warmth] must also be all-zero after full pipeline tick \
         (a swap-only IUS would move the 255-seeded pending into current, \
         revealing the missing clear_all_pending())"
    );
}

// ── Plan Assertion 3: phase2_shell_current_warmth_zero_inside_stamp_radius ────

/// Type A: influence_grid.sample(20, 20, Warmth) == 0 after 1 full engine.tick()
/// with building event at (20,20) r=3; 64×64 grid.
///
/// Phase 2 architectural invariant:
///   BSS writes dirty_regions only (not pending) + IUS clears pending before swap
///   → current[Warmth][idx(20,20)] == 0 even inside the stamp radius.
///
/// PHASE 2 SCOPE: fails by design when Phase 3 BFS propagation writes to pending
/// before IUS swaps. It is NOT a physical invariant — it is a Phase 2 shell invariant.
///
/// ticks: 1 (engine.tick() — all 4 Phase 2 systems)
/// components_read: SimResources.influence_grid.current
#[test]
fn harness_substantial_phase2_shell_current_warmth_zero_inside_stamp_radius() {
    let mut engine = fresh_phase2_engine();

    // Pre-seed ALL of current[Warmth] with 128 to make a no-op/missing IUS detectable.
    //
    // Without pre-seeding current starts at 0 (construction invariant, T7.7.B A3).
    // A no-op IUS leaves current at 0 → sample == 0 → assertion passes VACUOUSLY.
    // With 128 pre-seed the three IUS failure modes are:
    //
    //   Missing / no-op IUS: current stays at 128 → sample == 128 ≠ 0 → FAILS ✓
    //   no-swap IUS (clear only, no swap): pending cleared, current stays at 128 → FAILS ✓
    //   Correct IUS: clear_all_pending() [pending→0], swap() [current←0] → sample == 0 → passes
    //
    // Note: swap-only IUS (no clear) is caught by A2's stronger assertion on current.
    engine.resources.influence_grid.current[InfluenceChannel::Warmth as usize]
        .iter_mut()
        .for_each(|b| *b = 128);

    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (20, 20),
        radius: 3,
    });
    engine.tick();

    // Type A: threshold == 0 (Phase 2 shell: pending cleared → swapped to current)
    // current[Warmth] was pre-seeded with 128; IUS must overwrite it with 0 via clear+swap.
    let val = engine
        .resources
        .influence_grid
        .sample(20, 20, InfluenceChannel::Warmth);
    assert_eq!(
        val,
        0,
        "sample(20,20,Warmth) must be 0 in Phase 2 shell even when current was \
         pre-seeded with 128 (IUS clears pending then swaps; no-op IUS returns 128)"
    );
}

// ── Plan Assertion 4: AIS sentinel overwrite (warmth 99→0) ────────────────────

/// Type A: count of agents where influence_sample.warmth != 0 == 0
/// after 1 full tick, with 20 agents each initialized to warmth sentinel 99.
///
/// Anti-gaming setup:
///   (1) Agent count verified == 20 before tick (anti-vacuous guard).
///   (2) Sentinel warmth=99 distinguishes "AIS ran" from "AIS never ran".
///   (3) ..Default::default() guards against InfluenceSample gaining extra fields.
///
/// 4-link causal chain proven by this test (in conjunction with A1–A3):
///   BSS marks dirty (A1b) → BSS doesn't write pending (A2) →
///   IUS clears+swaps → current[tile]==0 (A3) →
///   AIS writes 0 into InfluenceSample.warmth, overwriting sentinel 99 (A4).
///
/// ticks: 1 (engine.tick() — all 4 Phase 2 systems)
/// components_read: Position, InfluenceSample, influence_grid.current
#[test]
fn harness_substantial_agent_warmth_sentinel_overwritten_to_zero_by_ais() {
    let mut engine = fresh_phase2_engine();
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (20, 20),
        radius: 3,
    });

    // Spawn 20 agents each with sentinel warmth=99 (NOT default 0) to catch a stub AIS.
    // Use ..Default::default() to guard against InfluenceSample gaining extra fields
    // beyond warmth and danger.
    for i in 0u32..20 {
        let x = (i * 3) % W;
        let y = (i * 5) % H;
        engine.world.spawn((
            Position { x, y },
            InfluenceSample { warmth: 99, ..Default::default() },
        ));
    }

    // Anti-vacuous guard: confirm agent count == 20 before the tick.
    // Ensures the warmth-count assertion cannot pass trivially with 0 agents.
    let before_count = engine.world.query::<&InfluenceSample>().iter().count();
    assert_eq!(
        before_count,
        20,
        "anti-vacuous: must have exactly 20 agents before tick, got {before_count}"
    );

    engine.tick();

    // Type A: threshold == 0
    // AIS ran and wrote current[Warmth][tile]==0 into each agent's warmth,
    // overwriting sentinel 99. An unregistered or stub AIS leaves warmth at 99.
    let non_zero_count = engine
        .world
        .query::<&InfluenceSample>()
        .iter()
        .filter(|(_, s)| s.warmth != 0)
        .count();
    assert_eq!(
        non_zero_count,
        0,
        "count of agents with warmth != 0 must be 0 after tick; \
         AIS must overwrite sentinel 99 with current buffer value 0 (Phase 2 shell)"
    );
}

// ── Plan Assertion 5: all_4_stamped_channels_dirty_1_non_stamped_0 ────────────

/// Type A: dirty_regions[Warmth/Spiritual/Beauty/Light].len() == 1 each,
///         dirty_regions[Danger/Noise/FoodAroma/Social].len() == 0 each
/// after 1 full engine.tick() with 1 building event at (20,20) r=3.
///
/// Verifies STAMPED_CHANNELS = {Warmth, Spiritual, Beauty, Light} is wired
/// correctly in the full pipeline (vs. BSS-only ticks in T7.7.B A7–A11).
/// IUS does not affect dirty_regions, so counts survive the full tick.
///
/// ticks: 1 (engine.tick() — all 4 Phase 2 systems)
/// components_read: SimResources.influence_grid.dirty_regions (all 8 channels)
#[test]
fn harness_substantial_all_4_stamped_channels_dirty_1_non_stamped_0_full_pipeline() {
    let mut engine = fresh_phase2_engine();
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (20, 20),
        radius: 3,
    });
    engine.tick();

    // Type A: stamped channels — threshold == 1 each
    for ch in [
        InfluenceChannel::Warmth,
        InfluenceChannel::Spiritual,
        InfluenceChannel::Beauty,
        InfluenceChannel::Light,
    ] {
        let len = engine.resources.influence_grid.dirty_regions[ch as usize].len();
        assert_eq!(
            len,
            1,
            "stamped channel {ch:?} dirty_regions.len() must be 1 after 1 event, got {len}"
        );
    }

    // Type A: non-stamped channels — threshold == 0 each
    for ch in [
        InfluenceChannel::Danger,
        InfluenceChannel::Noise,
        InfluenceChannel::FoodAroma,
        InfluenceChannel::Social,
    ] {
        let len = engine.resources.influence_grid.dirty_regions[ch as usize].len();
        assert_eq!(
            len,
            0,
            "non-stamped channel {ch:?} dirty_regions.len() must be 0, got {len}"
        );
    }
}

// ── Plan Assertion 6: dirty_regions accumulate across 3 events (1 tick) ───────

/// Type A: dirty_regions[Warmth].len() == 3 after 1 full engine.tick() with
/// 3 building events pre-queued at distinct in-bounds positions (10,10), (30,30), (50,50).
///
/// Proves: (1) BSS while-loop drains all 3 events in a single tick.
///         (2) IUS Phase 2 shell does NOT call clear_dirty().
///         → dirty_regions[Warmth] accumulates exactly 3 entries.
///
/// A Generator who mistakenly adds clear_dirty() to IUS Phase 2 produces count == 0.
/// A Generator who processes only 1 event per tick produces count == 1.
///
/// ticks: 1 (engine.tick() — all 4 Phase 2 systems)
/// components_read: SimResources.influence_grid.dirty_regions[Warmth]
#[test]
fn harness_substantial_dirty_regions_3_events_1_tick_count_3() {
    let mut engine = fresh_phase2_engine();

    // Queue 3 distinct in-bounds events
    for (cx, cy) in [(10u32, 10u32), (30, 30), (50, 50)] {
        engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
            position: (cx, cy),
            radius: 2,
        });
    }
    engine.tick();

    // Type A: threshold == 3
    let warmth_dirty =
        engine.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize].len();
    assert_eq!(
        warmth_dirty,
        3,
        "dirty_regions[Warmth].len() must be 3 after 1 tick with 3 queued events; \
         BSS while-loop drains all + IUS does not call clear_dirty() (got {warmth_dirty})"
    );
}

// ── Plan Assertion 7: dual-event single-tick causal stability ─────────────────

/// Type A: simultaneously (a) queue.len() == 0 AND (b) dirty_regions[Warmth].len() == 2
/// after 1 full engine.tick() with 2 building events pre-queued.
///
/// Causal pair variant for multi-event batch:
///   (a) BSS must drain BOTH events (while-loop, not if-let or single pop)
///   (b) BSS must produce two distinct dirty marks (one per in-bounds event)
///   IUS must not clear dirty_regions or (b) collapses to 0.
///
/// ticks: 1 (engine.tick() — all 4 Phase 2 systems)
/// components_read: SimResources.building_event_queue, influence_grid.dirty_regions[Warmth]
#[test]
fn harness_substantial_two_events_same_tick_yields_two_dirty_regions_per_stamped_channel() {
    let mut engine = fresh_phase2_engine();
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (10, 10),
        radius: 2,
    });
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (40, 40),
        radius: 1,
    });
    engine.tick();

    // Type A: (a) threshold == 0 — BSS while-loop drains both events in one tick
    let queue_len = engine.resources.building_event_queue.len();
    assert_eq!(
        queue_len,
        0,
        "building_event_queue must be empty after 2-event tick (BSS while-loop drains all)"
    );

    // Type A: (b) threshold == 2 — one dirty region per event; IUS does not clear
    let warmth_len =
        engine.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize].len();
    assert_eq!(
        warmth_len,
        2,
        "Warmth dirty_regions.len() must be 2 after 1 tick with 2 queued events \
         (BSS while-loop drains all; if-let or single pop would yield 1)"
    );
}

// ── Plan Assertion 8: dirty_region_bounds_preserved_through_full_pipeline_tick ─

/// Type A: dirty_regions[Warmth][0] == {min_x:17, max_x:23, min_y:17, max_y:23}
/// after 1 full engine.tick() with building event at (20,20) r=3; 64×64 grid.
///
/// Arithmetic: cx=20, r=3 → x1=20.saturating_sub(3)=17; x2=(20+3).min(63)=23.
/// IUS does not modify dirty_regions, so coordinate data produced by BSS is
/// preserved exactly through the full pipeline tick.
///
/// ticks: 1 (engine.tick() — all 4 Phase 2 systems)
/// components_read: SimResources.influence_grid.dirty_regions[Warmth]
#[test]
fn harness_substantial_dirty_region_bounds_preserved_through_full_pipeline_tick() {
    let mut engine = fresh_phase2_engine();
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (20, 20),
        radius: 3,
    });
    engine.tick();

    let regs =
        &engine.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize];
    assert_eq!(regs.len(), 1, "expected exactly 1 dirty region for Warmth");
    let r = &regs[0];

    // Type A: exact spatial bounds {17,17,23,23} — Chebyshev box at (20,20)±3 in 64×64
    assert_eq!(r.min_x, 17, "min_x must be 17 (20-3=17), got {}", r.min_x);
    assert_eq!(r.max_x, 23, "max_x must be 23 (20+3=23), got {}", r.max_x);
    assert_eq!(r.min_y, 17, "min_y must be 17 (20-3=17), got {}", r.min_y);
    assert_eq!(r.max_y, 23, "max_y must be 23 (20+3=23), got {}", r.max_y);
}

// ── Plan Assertion 9: OOB guard in full pipeline (mixed 5 events) ─────────────

/// Type A: no panic AND dirty_regions[Warmth].len() == 3
/// after 1 full engine.tick() with 5 events: 3 in-bounds + 2 OOB.
///
/// Setup: (10,10), (30,30), (50,50) in-bounds; (70,70), (100,100) OOB on 64×64 grid.
/// OOB guard uses `continue` (not break/return) → all 3 in-bounds events still processed.
/// count == 3 proves OOB events were skipped (not processed).
/// count < 3 would indicate in-bounds events were dropped by an early `return` or `break`.
///
/// ticks: 1 (engine.tick() — all 4 Phase 2 systems)
/// components_read: SimResources.influence_grid.dirty_regions[Warmth]
#[test]
fn harness_substantial_oob_guard_mixed_5events_3inbounds_dirty3() {
    let mut engine = fresh_phase2_engine();

    // 3 in-bounds events — these should produce dirty regions
    for (cx, cy) in [(10u32, 10u32), (30, 30), (50, 50)] {
        engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
            position: (cx, cy),
            radius: 2,
        });
    }
    // 2 OOB events (70 >= 64 and 100 >= 64 exceed the 64-tile grid)
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (70, 70),
        radius: 1,
    });
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position: (100, 100),
        radius: 1,
    });

    // Type A: no panic — test reaching this assertion proves the continue path ran safely
    engine.tick();

    // Type A: threshold == 3
    // OOB events skipped via `continue`; in-bounds events produced dirty regions.
    let warmth_dirty =
        engine.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize].len();
    assert_eq!(
        warmth_dirty,
        3,
        "dirty_regions[Warmth].len() must be 3 (3 in-bounds stamped, 2 OOB skipped), got {warmth_dirty}"
    );
}

// ── Plan Assertion 10: empty-queue stability (no spurious dirty marks) ─────────

/// Type A: dirty_regions[ch].len() == 0 for all 8 channels after 1 tick with 0 events.
///
/// Without events, BSS has nothing to stamp and must produce no dirty marks.
/// A BSS stub that marks dirty unconditionally fails this assertion.
/// Complements A6 (dirty persists with events) — the two assertions together prove
/// dirty_regions reflects event volume exactly, neither under- nor over-counting.
///
/// ticks: 1 (engine.tick() — all 4 Phase 2 systems)
/// components_read: SimResources.influence_grid.dirty_regions (all 8 channels)
#[test]
fn harness_substantial_empty_queue_1tick_all_channels_zero_dirty() {
    let mut engine = fresh_phase2_engine();
    // Queue 0 events — building_event_queue starts empty per T7.7.B A1 regression guard
    engine.tick(); // 1 tick with empty queue

    // Type A: threshold == 0 for all 8 channels
    for ch in InfluenceChannel::all() {
        let len = engine.resources.influence_grid.dirty_regions[*ch as usize].len();
        assert_eq!(
            len,
            0,
            "dirty_regions[{ch:?}].len() must be 0 after 1 tick with empty queue; \
             BSS without events marks no dirty regions (got {len})"
        );
    }
}

// ── Plan Assertion 11 (S1): 4380-tick idle baseline ───────────────────────────

/// Type A: all 8 channel current buffers remain zero after 4380 ticks on a
/// 64×64 engine with NO Phase 2 systems registered.
///
/// Long-run baseline (1 sim-year @ 12 ticks/day × 365). Existing max was 20 ticks.
/// Verifies no phantom influence sources are created by the engine itself.
///
/// ticks: 4380 (no systems — raw engine.tick())
/// components_read: influence_grid.current (8 channels), dirty_regions (8 channels)
#[test]
fn harness_substantial_idle_4380_no_systems_all_channels_zero() {
    // Deliberately NO register_phase2_systems — bare engine
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    for _ in 0..4380 {
        engine.tick();
    }

    // Type A: all channels, current buffer all-zero after 4380 idle ticks
    for ch in InfluenceChannel::all() {
        let all_zero = engine
            .resources
            .influence_grid
            .current_buf(*ch)
            .iter()
            .all(|&v| v == 0);
        assert!(
            all_zero,
            "channel {ch:?} current buffer must be all-zero after 4380 idle ticks (no systems)"
        );
        // dirty_regions also stay empty with no systems
        assert!(
            engine.resources.influence_grid.dirty_regions[*ch as usize].is_empty(),
            "dirty_regions[{ch:?}] must be empty after 4380 idle ticks (no BSS registered)"
        );
    }
    assert!(
        engine.resources.building_event_queue.is_empty(),
        "building_event_queue must be empty after 4380 idle ticks (no FFI calls made)"
    );
}

// ── Plan Assertion 12 (S4): 1000-agent AIS grid indexing, Phase 2 all-zero ────

/// Type A: count of agents where influence_sample.warmth != 0 == 0
/// after 1 tick with 1000 agents spawned at deterministic positions.
///
/// AIS must correctly index each agent's tile and write warmth=0 (current all-zero
/// in Phase 2 shell). An AIS that panics on large agent counts fails at runtime.
/// Anti-vacuous guard: verify total agent count == 1000 before the warmth check.
///
/// Position formula: x = (i*7) % 64, y = ((i*13) / 64) % 64 for i in 0..1000.
/// All positions in [0,63]×[0,63] — outer %64 ensures y<64.
///
/// Phase 2 limitation acknowledged: since current is all-zero, this cannot
/// distinguish correct tile-indexed reads from a hardcoded 0.0.
/// Phase 3 non-zero grid tests close that gap.
///
/// ticks: 1
/// components_read: InfluenceSample, InfluenceGrid (current buffer)
#[test]
fn harness_substantial_1k_agents_ais_warmth_all_zero_phase2_baseline() {
    let mut engine = fresh_phase2_engine();

    // Spawn 1000 agents at deterministic positions per plan assertion 12 spec.
    //
    // ANTI-CIRCULAR SETUP: each agent is initialized with warmth sentinel 99
    // (NOT default 0) so a no-op AIS is distinguishable from a correct AIS.
    //
    //   No-op AIS:     warmth stays at 99 → non_zero count > 0 → FAILS ✓
    //   Correct AIS:   writes current[Warmth][tile]==0 → overwrites 99 with 0 → passes ✓
    //
    // `..Default::default()` guards against InfluenceSample gaining extra fields
    // beyond warmth and danger (compilation would break `{ warmth: 99, danger: 0 }`).
    for i in 0..1000usize {
        let x = (i as u32 * 7) % W;
        let y = ((i as u32 * 13) / W) % H; // ((i*13)/64)%64
        engine.world.spawn((
            Position { x, y },
            InfluenceSample { warmth: 99, ..Default::default() },
        ));
    }

    // Anti-vacuous guard: verify 1000 agents exist BEFORE the tick.
    // Guards against the assertion passing with 0 agents (count==0 → non_zero==0 trivially).
    let total_before = engine.world.query::<&InfluenceSample>().iter().count();
    assert_eq!(
        total_before,
        1000,
        "anti-vacuous: must have exactly 1000 agents with warmth=99 before tick, got {total_before}"
    );

    engine.tick(); // 1 tick — AIS runs for all 1000 agents

    // Type A: threshold == 0
    // AIS writes current[Warmth][tile]==0 to all agents (Phase 2 all-zero current),
    // overwriting sentinel 99. An unregistered or stub AIS leaves warmth at 99 → fails.
    let non_zero = engine
        .world
        .query::<&InfluenceSample>()
        .iter()
        .filter(|(_, s)| s.warmth != 0)
        .count();
    assert_eq!(
        non_zero,
        0,
        "all 1000 agents must have warmth == 0 after 1 tick (AIS overwrites sentinel 99 \
         with current[Warmth][tile]==0 in Phase 2 shell); got {non_zero} with non-zero warmth"
    );
}

// ── Plan Assertion 13 (S6): 100-event dirty_regions accumulation ──────────────

/// Type A: dirty_regions[Warmth].len() == 100 after 1 full engine.tick()
/// with 100 events pre-queued at positions ((i*7)%64, (i*11)%64) for i in 0..100.
///
/// Scales BSS drain to high-throughput burst. All 100 positions are in-bounds
/// (both formulas produce values in [0,63] for i<100 with W=H=64).
/// A BSS with fixed-size buffer that truncates produces count < 100.
/// A BSS that deduplicates overlapping regions produces count < 100.
///
/// ticks: 1 (engine.tick() — all 4 Phase 2 systems)
/// components_read: SimResources.influence_grid.dirty_regions[Warmth]
#[test]
fn harness_substantial_burst_100_events_single_tick_drain() {
    let mut engine = fresh_phase2_engine();

    // Enqueue 100 events at positions per plan assertion 13 spec: (i*7)%64, (i*11)%64
    for i in 0u32..100 {
        let x = (i * 7) % W;
        let y = (i * 11) % H; // plan spec: (i*11)%64
        engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
            position: (x, y),
            radius: 1,
        });
    }

    // Type A: no panic — test returning normally = pass
    engine.tick();

    // Type A: (a) threshold == 0 — BSS while-loop drains all 100 events
    let queue_len = engine.resources.building_event_queue.len();
    assert_eq!(
        queue_len,
        0,
        "queue must be empty after 100-event tick (BSS while-loop drains all), got {queue_len}"
    );

    // Type A: (b) threshold == 100 — one dirty region per event per stamped channel
    let warmth_dirty =
        engine.resources.influence_grid.dirty_regions[InfluenceChannel::Warmth as usize].len();
    assert_eq!(
        warmth_dirty,
        100,
        "Warmth dirty_regions must have 100 entries after 100-event burst, got {warmth_dirty}"
    );
}

// ── Plan Assertion 14 (S11): system double-registration count ─────────────────

/// Type C: engine.system_count() == 8 after register_phase2_systems called twice.
/// No panic on subsequent tick with doubled Phase 2 stack.
///
/// Type C (observed behavior): double-registering 4 systems produces count==8 under
/// the current additive (non-idempotent) registration semantics. If the contract
/// changes to idempotent dedup (count stays 4), update threshold to == 4 in that commit.
///
/// ticks: 1 (engine.tick() — 8 systems, 2× Phase 2 stack)
/// components_read: SimEngine.system_count()
#[test]
fn harness_substantial_register_phase2_twice_no_panic() {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    register_phase2_systems(&mut engine);

    // Type C: registration is additive — system count doubles
    assert_eq!(
        engine.system_count(),
        8,
        "register_phase2_systems called twice must yield 8 systems (additive, not idempotent)"
    );

    // Type A: no panic on tick with doubled Phase 2 stack
    engine.tick(); // second BSS finds empty queue after first BSS drains it — no panic
}

// ── Plan Assertion 15 (S12): four-corner boundary clamping ────────────────────

/// Type A: all dirty region coordinates stay within [0, 63] bounds after
/// stamping all 4 corners with r=3 in 1 full engine.tick(); 64×64 grid.
///
/// Extends T7.7.B A16 (one corner) to all 4 corners simultaneously.
/// Verifies saturating_sub and .min(w−1)/.min(h−1) clamps at all grid edges.
///
/// ticks: 1 (engine.tick() — all 4 Phase 2 systems)
/// components_read: SimResources.influence_grid.dirty_regions (4 stamped channels)
#[test]
fn harness_substantial_four_corner_stamps_clamp_no_oob_dirty() {
    let mut engine = fresh_phase2_engine();

    // Stamp all 4 corners with r=3 (would extend 3 tiles past each edge without clamp)
    for (cx, cy) in [(0u32, 0u32), (63, 0), (0, 63), (63, 63)] {
        engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
            position: (cx, cy),
            radius: 3,
        });
    }
    engine.tick();

    // Type A: no dirty region coordinate may exceed grid bounds
    for ch in [
        InfluenceChannel::Warmth,
        InfluenceChannel::Spiritual,
        InfluenceChannel::Beauty,
        InfluenceChannel::Light,
    ] {
        let regs = &engine.resources.influence_grid.dirty_regions[ch as usize];
        assert_eq!(
            regs.len(),
            4,
            "{ch:?} must have 4 dirty regions (one per corner event), got {}",
            regs.len()
        );
        for r in regs {
            assert!(
                r.min_x < W,
                "{ch:?} dirty region min_x={} must be < {W}",
                r.min_x
            );
            assert!(
                r.max_x < W,
                "{ch:?} dirty region max_x={} must be < {W} (clamped to w-1)",
                r.max_x
            );
            assert!(
                r.min_y < H,
                "{ch:?} dirty region min_y={} must be < {H}",
                r.min_y
            );
            assert!(
                r.max_y < H,
                "{ch:?} dirty region max_y={} must be < {H} (clamped to h-1)",
                r.max_y
            );
        }
    }
}
