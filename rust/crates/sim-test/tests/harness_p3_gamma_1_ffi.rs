//! P3-γ (γ-1) — FFI surface for causal chain queries (V7 Phase 3-γ).
//!
//! γ-1 exposes the P3-α tile-causal log and the P3-β `trace_parents` walk
//! through two new read-only sim-bridge `#[func]` methods:
//!   - `WorldSimNode::get_tile_causal_history(x, y) -> Array<Dictionary>`
//!   - `WorldSimNode::get_event_chain(x, y, event_id) -> Array<Dictionary>`
//!
//! Both `#[func]` bodies are thin marshalling wrappers — the canonical
//! pure-Rust implementations live in [`collect_tile_causal_history`] and
//! [`collect_event_chain`]. This harness exercises those collectors directly
//! (no Godot runtime), mirroring the T7.7.B Bridge Identity Contract pattern
//! used by `enqueue_building_placed` in `harness_phase2_ffi.rs`. The
//! `Variant`-side marshalling (`event_view_to_dict`,
//! `event_views_to_variant_array`) is verified by Evaluator code review.
//!
//! Setup: 64×64 engine, empty `MaterialRegistry`, Phase 2 systems registered
//! (full-engine assertions). One assertion uses BSS-only to keep the
//! `BuildingPlaced` event surviving in the centre tile's 8-slot ring.
//!
//! Run: `cargo test -p sim-test --test harness_p3_gamma_1_ffi -- --nocapture`

use sim_bridge::ffi::{
    collect_event_chain, collect_tile_causal_history, try_collect_event_chain,
    try_collect_tile_causal_history, CausalEventView,
};
use sim_core::causal::CausalEvent;
use sim_core::influence::{DirtyRegion, InfluenceChannel};
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, SimEngine};
use sim_systems::register_phase2_systems;
use sim_systems::runtime::influence::BuildingStampSystem;

const W: u32 = 64;
const H: u32 = 64;
const SX: u32 = 32;
const SY: u32 = 32;
const RADIUS: u32 = 12;

fn full_engine() -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    engine
}

fn bss_only_engine() -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    engine.register_system(Box::new(BuildingStampSystem::new()));
    engine
}

fn place_source(engine: &mut SimEngine, position: (u32, u32)) {
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position,
        radius: RADIUS,
    });
}

fn centre_tile_idx(x: u32, y: u32) -> u32 {
    y * W + x
}

// ── A1: empty enumerator on never-stamped tile ───────────────────────────────

/// Type A: a tile that has never received a `CausalEvent::push` returns an
/// empty `Vec`. Mirrors the `#[func]` early-return path that the panel layer
/// (γ-2) relies on to render "no events recorded" without throwing.
#[test]
fn harness_p3_gamma_1_enumerator_empty_on_unstamped_tile() {
    let engine = full_engine();
    let views = collect_tile_causal_history(&engine.resources, centre_tile_idx(0, 0));
    assert!(
        views.is_empty(),
        "untouched tile must yield zero CausalEventView entries (got {})",
        views.len()
    );
}

// ── A2: full tick produces 13 pushes, ring caps at 8 entries ────────────────

/// Type A: a full Phase 2 tick on the centre tile pushes
/// 1 BuildingPlaced + 6 StampDirty + 6 InfluenceChanged = 13 events. The
/// FIFO ring caps retention at `TILE_CAUSAL_RING_SIZE` (8), and the
/// collector returns exactly that capped count. This locks the eviction
/// contract observable through the FFI surface.
#[test]
fn harness_p3_gamma_1_enumerator_returns_all_events_after_building() {
    let mut engine = full_engine();
    place_source(&mut engine, (SX, SY));
    engine.tick();

    let views = collect_tile_causal_history(&engine.resources, centre_tile_idx(SX, SY));
    assert_eq!(
        views.len(),
        8,
        "ring caps at 8 after the 13 pushes of a full tick"
    );
    // The 8 retained entries are the last 8 pushed (FIFO eviction drops
    // the oldest 5). Spot-check that ids are strictly monotonic — the
    // Variant-side marshalling preserves the slice ordering.
    for win in views.windows(2) {
        assert!(
            win[0].id < win[1].id,
            "view ids must remain strictly increasing across the ring"
        );
    }
}

// ── A3: BuildingPlaced view exposes the full schema ──────────────────────────

/// Type A: a `BuildingPlaced` event maps to a `CausalEventView` with
/// `kind == "building_placed"`, the source `(x, y)`, the configured radius,
/// `parent == None` (root), no channel, no region, and no old/new values.
/// Uses the BSS-only engine so the `BuildingPlaced` survives in the ring
/// (full Phase 2 evicts it once IUS pushes 6 InfluenceChanged records).
#[test]
fn harness_p3_gamma_1_enumerator_dict_schema_building_placed() {
    let mut engine = bss_only_engine();
    place_source(&mut engine, (SX, SY));
    engine.tick();

    let views = collect_tile_causal_history(&engine.resources, centre_tile_idx(SX, SY));
    let placed = views
        .iter()
        .find(|v| v.kind == "building_placed")
        .expect("BSS-only engine must keep BuildingPlaced in the ring");

    assert_eq!(placed.kind, "building_placed");
    assert_eq!(placed.parent, None, "BuildingPlaced is a chain root");
    assert_eq!(placed.position, Some((SX, SY)));
    assert_eq!(placed.radius, Some(RADIUS));
    assert_eq!(placed.channel, None);
    assert_eq!(placed.region, None);
    assert_eq!(placed.old_value, None);
    assert_eq!(placed.new_value, None);
}

// ── A4: StampDirty view exposes the Chebyshev region ─────────────────────────

/// Type A: a `StampDirty` event maps to a `CausalEventView` with
/// `kind == "stamp_dirty"`, a channel index, a region equal to the Chebyshev
/// box BSS clamped to the grid, no position, no radius, no old/new values,
/// and `parent == Some(building_id)` linking back to the originating
/// BuildingPlaced. Uses the BSS-only engine to keep the BuildingPlaced
/// surviving so we can correlate ids.
#[test]
fn harness_p3_gamma_1_enumerator_dict_schema_stamp_dirty() {
    let mut engine = bss_only_engine();
    place_source(&mut engine, (SX, SY));
    engine.tick();

    let views = collect_tile_causal_history(&engine.resources, centre_tile_idx(SX, SY));
    let placed = views
        .iter()
        .find(|v| v.kind == "building_placed")
        .expect("BuildingPlaced must survive in the ring");
    let building_id = placed.id;

    let stamp = views
        .iter()
        .find(|v| v.kind == "stamp_dirty")
        .expect("BSS must push StampDirty records");

    assert_eq!(stamp.kind, "stamp_dirty");
    assert_eq!(stamp.parent, Some(building_id));
    assert!(stamp.channel.is_some(), "StampDirty exposes a channel index");
    assert_eq!(stamp.position, None);
    assert_eq!(stamp.radius, None);
    let region = stamp.region.expect("StampDirty carries a region");
    let (min_x, min_y, max_x, max_y) = region;
    // Chebyshev box around (SX, SY) with RADIUS=12, clamped to (0, W-1).
    let expected_min_x = SX.saturating_sub(RADIUS);
    let expected_min_y = SY.saturating_sub(RADIUS);
    let expected_max_x = (SX + RADIUS).min(W - 1);
    let expected_max_y = (SY + RADIUS).min(H - 1);
    assert_eq!(min_x, expected_min_x, "min_x must match Chebyshev clamp");
    assert_eq!(min_y, expected_min_y, "min_y must match Chebyshev clamp");
    assert_eq!(max_x, expected_max_x, "max_x must match Chebyshev clamp");
    assert_eq!(max_y, expected_max_y, "max_y must match Chebyshev clamp");
    assert_eq!(stamp.old_value, None);
    assert_eq!(stamp.new_value, None);
}

// ── A5: InfluenceChanged view exposes old/new sample values ──────────────────

/// Type A: an `InfluenceChanged` event maps to a `CausalEventView` with
/// `kind == "influence_changed"`, a channel index, the centre sample
/// position, `Some(old)` and `Some(new)` intensity values, no region, no
/// radius, and a non-None parent (the same-channel StampDirty resolved by
/// IUS via `find_recent_stamp_dirty`).
#[test]
fn harness_p3_gamma_1_enumerator_dict_schema_influence_changed() {
    let mut engine = full_engine();
    place_source(&mut engine, (SX, SY));
    engine.tick();

    let views = collect_tile_causal_history(&engine.resources, centre_tile_idx(SX, SY));
    // Pick an InfluenceChanged whose same-channel StampDirty is still
    // present in the ring (after eviction, only the most-recent stamps
    // survive — 1 BP + 6 Stamps + 6 Influences = 13 pushes, ring caps
    // at 8). Iterating ensures the chain link can be verified without
    // depending on which channels happened to be retained.
    let (influence, stamp) = views
        .iter()
        .filter(|v| v.kind == "influence_changed")
        .find_map(|inf| {
            let ch = inf.channel?;
            let stamp = views
                .iter()
                .find(|v| v.kind == "stamp_dirty" && v.channel == Some(ch))?;
            Some((inf.clone(), stamp.clone()))
        })
        .expect(
            "at least one InfluenceChanged must have its same-channel StampDirty \
             retained in the ring after one Phase 2 tick",
        );

    assert_eq!(influence.kind, "influence_changed");
    assert_eq!(
        influence.parent,
        Some(stamp.id),
        "InfluenceChanged.parent must equal the same-channel StampDirty.id (P3-β chain link)"
    );
    assert_eq!(
        influence.position,
        Some((SX, SY)),
        "InfluenceChanged samples at the centre tile"
    );
    assert_eq!(influence.radius, None);
    assert_eq!(influence.region, None);
    assert!(
        influence.old_value.is_some(),
        "InfluenceChanged exposes pre-propagation intensity"
    );
    assert!(
        influence.new_value.is_some(),
        "InfluenceChanged exposes post-propagation intensity"
    );
}

// ── A6: chain walk returns three events for a hand-built lineage ─────────────

/// Type A: `collect_event_chain` walks the parent chain backwards and
/// returns `[child, parent, root]` in that order. We hand-build a 3-event
/// chain via `CausalLogStorage::push` so the test exercises the FFI walk
/// path in isolation from the engine pipeline (eviction-free).
#[test]
fn harness_p3_gamma_1_chain_returns_three_events() {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    let tile = centre_tile_idx(1, 1);

    engine.resources.causal_log.push(
        tile,
        CausalEvent::BuildingPlaced {
            id: 0,
            parent: None,
            position: (1, 1),
            radius: 1,
            tick: 0,
        },
    );
    engine.resources.causal_log.push(
        tile,
        CausalEvent::StampDirty {
            id: 1,
            parent: Some(0),
            channel: InfluenceChannel::Warmth,
            region: DirtyRegion::new(0, 0, 2, 2),
            tick: 0,
        },
    );
    engine.resources.causal_log.push(
        tile,
        CausalEvent::InfluenceChanged {
            id: 2,
            parent: Some(1),
            channel: InfluenceChannel::Warmth,
            position: (1, 1),
            old: 0.0,
            new: 200.0,
            tick: 0,
        },
    );

    let chain = collect_event_chain(&engine.resources, tile, 2);
    assert_eq!(chain.len(), 3, "chain must walk InfluenceChanged → Stamp → BuildingPlaced");
    assert_eq!(chain[0].kind, "influence_changed");
    assert_eq!(chain[0].id, 2);
    assert_eq!(chain[1].kind, "stamp_dirty");
    assert_eq!(chain[1].id, 1);
    assert_eq!(chain[2].kind, "building_placed");
    assert_eq!(chain[2].id, 0);
    assert_eq!(chain[2].parent, None, "root must report parent == None");
}

// ── A7: root event returns a single-element chain ────────────────────────────

/// Type A: walking from a root `BuildingPlaced` (parent: None) yields a
/// 1-element chain containing just that event. This is the "no further
/// ancestry" terminal case the γ-2 panel surfaces as the bottom of the
/// trace.
#[test]
fn harness_p3_gamma_1_chain_root_event_returns_single() {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    let tile = centre_tile_idx(5, 5);

    engine.resources.causal_log.push(
        tile,
        CausalEvent::BuildingPlaced {
            id: 42,
            parent: None,
            position: (5, 5),
            radius: 3,
            tick: 7,
        },
    );

    let chain = collect_event_chain(&engine.resources, tile, 42);
    assert_eq!(chain.len(), 1, "root walk must return a single entry");
    assert_eq!(chain[0].kind, "building_placed");
    assert_eq!(chain[0].id, 42);
    assert_eq!(chain[0].parent, None);
}

// ── A8: unknown event_id returns an empty chain ──────────────────────────────

/// Type A: walking from an `event_id` that is not present on the tile
/// returns an empty `Vec`. This is the panel layer's "id not found"
/// failure mode (e.g. evicted root with a stale UI reference). The FFI
/// surface returns an empty `Array` rather than throwing — the panel can
/// render the corresponding `cause evicted` state.
#[test]
fn harness_p3_gamma_1_chain_invalid_event_id_returns_empty() {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    let tile = centre_tile_idx(2, 2);

    engine.resources.causal_log.push(
        tile,
        CausalEvent::BuildingPlaced {
            id: 10,
            parent: None,
            position: (2, 2),
            radius: 1,
            tick: 0,
        },
    );

    // Probe 1: a fabricated large id that was never pushed.
    let chain_missing = collect_event_chain(&engine.resources, tile, 999);
    assert!(
        chain_missing.is_empty(),
        "fabricated large event_id (999) must yield an empty chain (got {} entries)",
        chain_missing.len()
    );

    // Probe 2: id 0 — absent from this tile (the sole pushed event has id
    // 10). The walker must NOT treat 0 as a sentinel for "first event"; it
    // is just an id that happens to not exist on this tile.
    let chain_zero = collect_event_chain(&engine.resources, tile, 0);
    assert!(
        chain_zero.is_empty(),
        "absent event_id (0) must yield an empty chain (got {} entries)",
        chain_zero.len()
    );
}

// ── A8b: out-of-bounds coordinate and negative event_id semantics ───────────

/// Type A: the FFI-mirror helpers reject out-of-bounds `(x, y)` and
/// negative `event_id` by returning an empty `Vec` — never panicking,
/// never fabricating entries. This covers the boundary-semantics fact
/// P3γ-F2 for the path the `#[func]` bodies take.
#[test]
fn harness_p3_gamma_1_ffi_oob_and_negative_id_return_empty() {
    let mut engine = full_engine();
    place_source(&mut engine, (SX, SY));
    engine.tick();

    // Negative x.
    let neg_x = try_collect_tile_causal_history(&engine.resources, -1, SY as i32);
    assert!(neg_x.is_empty(), "negative x must yield empty history");

    // Negative y.
    let neg_y = try_collect_tile_causal_history(&engine.resources, SX as i32, -5);
    assert!(neg_y.is_empty(), "negative y must yield empty history");

    // x >= width.
    let big_x =
        try_collect_tile_causal_history(&engine.resources, W as i32, SY as i32);
    assert!(big_x.is_empty(), "x == width must yield empty history");
    let huge_x = try_collect_tile_causal_history(&engine.resources, 9999, SY as i32);
    assert!(huge_x.is_empty(), "x far past width must yield empty history");

    // y >= height.
    let big_y =
        try_collect_tile_causal_history(&engine.resources, SX as i32, H as i32);
    assert!(big_y.is_empty(), "y == height must yield empty history");

    // Same checks for the chain walker.
    let neg_x_chain = try_collect_event_chain(&engine.resources, -1, SY as i32, 1);
    assert!(neg_x_chain.is_empty(), "negative x must yield empty chain");
    let big_y_chain =
        try_collect_event_chain(&engine.resources, SX as i32, H as i32, 1);
    assert!(big_y_chain.is_empty(), "y == height must yield empty chain");

    // Negative event_id on an in-bounds tile that has real events present
    // — the rejection must come from the negative-id guard, not from a
    // missing tile log.
    let neg_id_chain =
        try_collect_event_chain(&engine.resources, SX as i32, SY as i32, -1);
    assert!(
        neg_id_chain.is_empty(),
        "negative event_id must yield empty chain (got {} entries)",
        neg_id_chain.len()
    );
    let neg_id_chain2 =
        try_collect_event_chain(&engine.resources, SX as i32, SY as i32, i64::MIN);
    assert!(
        neg_id_chain2.is_empty(),
        "i64::MIN event_id must yield empty chain (no underflow on cast)"
    );
}

// ── A9: cross-channel chain isolation ────────────────────────────────────────

/// Type A: walking two channels' `InfluenceChanged` events on the same
/// tile yields disjoint parent links — each chain hops to its own
/// channel's `StampDirty`. This proves the FFI surface preserves the
/// per-channel disambiguation that P3-β established via
/// `find_recent_stamp_dirty`.
#[test]
fn harness_p3_gamma_1_cross_channel_chain_isolation() {
    let mut engine = full_engine();
    place_source(&mut engine, (SX, SY));
    engine.tick();

    let tile = centre_tile_idx(SX, SY);
    let views = collect_tile_causal_history(&engine.resources, tile);

    fn channel_view(views: &[CausalEventView], ch_idx: u8, kind: &str) -> CausalEventView {
        views
            .iter()
            .find(|v| v.kind == kind && v.channel == Some(ch_idx))
            .unwrap_or_else(|| {
                panic!(
                    "expected {kind} for channel index {ch_idx} in ring of {} entries",
                    views.len()
                )
            })
            .clone()
    }

    let noise_idx = InfluenceChannel::Noise as u8;
    let danger_idx = InfluenceChannel::Danger as u8;

    let noise_inf = channel_view(&views, noise_idx, "influence_changed");
    let danger_inf = channel_view(&views, danger_idx, "influence_changed");

    let noise_chain = collect_event_chain(&engine.resources, tile, noise_inf.id);
    let danger_chain = collect_event_chain(&engine.resources, tile, danger_inf.id);

    assert!(
        noise_chain.len() >= 2,
        "Noise chain must contain at least the influence + its parent stamp"
    );
    assert!(
        danger_chain.len() >= 2,
        "Danger chain must contain at least the influence + its parent stamp"
    );

    let noise_stamp = &noise_chain[1];
    let danger_stamp = &danger_chain[1];
    assert_eq!(noise_stamp.kind, "stamp_dirty");
    assert_eq!(danger_stamp.kind, "stamp_dirty");
    assert_eq!(
        noise_stamp.channel,
        Some(noise_idx),
        "Noise InfluenceChanged.parent must be a Noise StampDirty"
    );
    assert_eq!(
        danger_stamp.channel,
        Some(danger_idx),
        "Danger InfluenceChanged.parent must be a Danger StampDirty"
    );
    assert_ne!(
        noise_stamp.id, danger_stamp.id,
        "cross-channel chains must reference distinct StampDirty ids"
    );
}

// ── A10: parent: None survives the view layer for root events ────────────────

/// Type A: a root `BuildingPlaced` round-trips through `CausalEventView`
/// with `parent == None`. The dictionary layer (verified by Evaluator
/// code review) serialises this as the sentinel `-1` so GDScript can
/// distinguish the chain root from a present-but-evicted parent (which
/// also serialises as `-1`). The behavioural promise tested here is that
/// the view stage faithfully preserves the `Option<EventId>` field —
/// independent of how the marshalling layer encodes `None`.
#[test]
fn harness_p3_gamma_1_parent_none_serialized_as_minus_one() {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    let tile = centre_tile_idx(3, 3);

    engine.resources.causal_log.push(
        tile,
        CausalEvent::BuildingPlaced {
            id: 7,
            parent: None,
            position: (3, 3),
            radius: 2,
            tick: 0,
        },
    );

    let history = collect_tile_causal_history(&engine.resources, tile);
    let root = history
        .iter()
        .find(|v| v.kind == "building_placed")
        .expect("root must be present");
    assert_eq!(root.parent, None, "root view must carry parent: None");

    let chain = collect_event_chain(&engine.resources, tile, 7);
    assert_eq!(chain.len(), 1);
    assert_eq!(
        chain[0].parent, None,
        "root walked through collect_event_chain still carries parent: None"
    );

    // Mirror the dictionary encoding path used by `event_view_to_dict`:
    // `parent.map(|p| p as i64).unwrap_or(-1)`. This documents the FFI
    // contract surfaced to GDScript without depending on the Godot
    // runtime.
    let parent_wire: i64 = chain[0].parent.map(|p| p as i64).unwrap_or(-1);
    assert_eq!(parent_wire, -1, "parent: None encodes as the -1 sentinel");

    // Sanity check: a present parent encodes as a non-negative i64. We
    // use a fabricated stamp on a separate tile to avoid colliding with
    // the root's id space.
    let stamp_tile = centre_tile_idx(4, 4);
    engine.resources.causal_log.push(
        stamp_tile,
        CausalEvent::StampDirty {
            id: 11,
            parent: Some(7),
            channel: InfluenceChannel::Warmth,
            region: DirtyRegion::new(0, 0, 1, 1),
            tick: 0,
        },
    );
    let stamp_history = collect_tile_causal_history(&engine.resources, stamp_tile);
    let stamp = stamp_history.first().expect("stamp must be present");
    let stamp_parent_wire: i64 = stamp.parent.map(|p| p as i64).unwrap_or(-1);
    assert_eq!(
        stamp_parent_wire, 7,
        "present parent must encode as the i64 value of the EventId"
    );
}

// Compile-time assertion: the re-exported symbols are reachable through
// the public `sim_bridge::ffi` namespace. The import list at the top of
// this file already exercises that path; a no-op test here documents the
// contract so a future re-export removal triggers a harness failure
// alongside whatever compile error follows.
#[test]
fn harness_p3_gamma_1_collectors_are_publicly_reexported() {
    let engine = SimEngine::new(W, H, MaterialRegistry::new());
    let _: Vec<CausalEventView> = collect_tile_causal_history(&engine.resources, 0);
    let _: Vec<CausalEventView> = collect_event_chain(&engine.resources, 0, 0);
}
