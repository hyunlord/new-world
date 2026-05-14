//! P3-β — Causal Chain Link Tracking (V7 Phase 3-β).
//!
//! P3-α records what happened on a tile; P3-β records *why* each event was
//! emitted by linking every `CausalEvent` to its parent through a
//! monotonically allocated [`EventId`]. The chain semantic (P3β-3) is:
//!
//! ```text
//! BuildingPlaced { parent: None }
//!   → StampDirty { parent: Some(building_id) }       (× 6 channels)
//!     → InfluenceChanged { parent: Some(stamp_id) }   (per channel)
//! ```
//!
//! `find_recent_stamp_dirty(tile_idx, channel)` resolves the per-channel
//! parent at push time; `trace_parents(tile_idx, event_id)` walks the
//! chain backward and terminates gracefully when an ancestor has been
//! evicted from the ring (FIFO depth 8).
//!
//! Setup: 64×64 engine, empty MaterialRegistry, Phase 2 systems registered
//! (unless a specific assertion needs a stripped engine — see
//! `bss_only_engine`). All thresholds are LOCKED — do NOT modify them.
//!
//! Run: `cargo test -p sim-test --test harness_p3_beta_causal_chain -- --nocapture`

use sim_core::causal::{CausalEvent, CausalLogStorage};
use sim_core::influence::{DirtyRegion, InfluenceChannel};
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, SimEngine};
use sim_systems::register_phase2_systems;
use sim_systems::runtime::influence::BuildingStampSystem;

const W: u32 = 64;
const H: u32 = 64;
const SX: u32 = 32;
const SY: u32 = 32;

fn full_engine() -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    engine
}

/// Engine with only BSS registered — keeps the BuildingPlaced event in the
/// centre tile's ring because IUS never runs to push the 6 InfluenceChanged
/// records that would evict it.
fn bss_only_engine() -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    engine.register_system(Box::new(BuildingStampSystem::new()));
    engine
}

fn place_source(engine: &mut SimEngine, position: (u32, u32)) {
    engine.resources.building_event_queue.push_back(BuildingPlacedEvent {
        position,
        radius: 12,
    });
}

fn centre_tile_idx(x: u32, y: u32) -> u32 {
    y * W + x
}

// ── A1: event ids are allocated monotonically ────────────────────────────────

/// Type A: every retained event on the centre tile reports a strictly
/// increasing `id()` across the slice. The monotonic guarantee is provided
/// by `SimResources::issue_event_id` (AtomicU64 fetch_add) — without it the
/// "왜?" UI cannot tell parent from child.
#[test]
fn harness_p3_beta_event_id_monotonic() {
    let mut engine = full_engine();
    place_source(&mut engine, (SX, SY));
    engine.tick();

    let log = engine
        .resources
        .causal_log
        .get(centre_tile_idx(SX, SY))
        .expect("centre tile must hold a log after one tick");
    let ids: Vec<u64> = log.iter().map(|e| e.id()).collect();
    assert_eq!(ids.len(), 8, "ring should hold exactly 8 events");
    for win in ids.windows(2) {
        assert!(
            win[0] < win[1],
            "ids must be strictly increasing — got {} then {}",
            win[0],
            win[1]
        );
    }
}

// ── A2: BuildingPlaced has no parent (root) ──────────────────────────────────

/// Type A: the root event of every chain — `BuildingPlaced` — reports
/// `parent == None`. The BSS-only engine keeps it in the ring (full Phase 2
/// would evict it after 6 InfluenceChanged pushes).
#[test]
fn harness_p3_beta_building_placed_has_no_parent() {
    let mut engine = bss_only_engine();
    place_source(&mut engine, (SX, SY));
    engine.tick();

    let log = engine
        .resources
        .causal_log
        .get(centre_tile_idx(SX, SY))
        .expect("BSS must record a log");
    let placed: Vec<&CausalEvent> = log
        .iter()
        .filter(|e| matches!(e, CausalEvent::BuildingPlaced { .. }))
        .collect();
    assert_eq!(
        placed.len(),
        1,
        "exactly one BuildingPlaced must be recorded on the centre tile"
    );
    assert_eq!(placed[0].parent(), None, "BuildingPlaced is a chain root");
}

// ── A3: StampDirty.parent == BuildingPlaced.id ───────────────────────────────

/// Type A: every StampDirty pushed by BSS in the same loop iteration links
/// back to the BuildingPlaced that triggered it. With 6 stamped channels +
/// 1 BuildingPlaced the ring holds 7 events (under the 8 FIFO cap) so all
/// links are observable.
#[test]
fn harness_p3_beta_stamp_dirty_parent_is_building() {
    let mut engine = bss_only_engine();
    place_source(&mut engine, (SX, SY));
    engine.tick();

    let log = engine
        .resources
        .causal_log
        .get(centre_tile_idx(SX, SY))
        .expect("BSS must record a log");

    let building_id = log
        .iter()
        .find_map(|e| match e {
            CausalEvent::BuildingPlaced { id, .. } => Some(*id),
            _ => None,
        })
        .expect("BuildingPlaced must be present");

    let stamps: Vec<&CausalEvent> = log
        .iter()
        .filter(|e| matches!(e, CausalEvent::StampDirty { .. }))
        .collect();
    assert_eq!(stamps.len(), 6, "BSS must push 6 StampDirty records");
    for s in stamps {
        assert_eq!(
            s.parent(),
            Some(building_id),
            "every StampDirty must link to the BuildingPlaced root"
        );
    }
}

// ── A4: InfluenceChanged.parent == matching-channel StampDirty.id ────────────

/// Type A: each `InfluenceChanged` record points at the same-channel
/// `StampDirty` that was still in the ring when IUS resolved its parent.
/// We test Noise specifically because the Noise StampDirty (BSS slot 5)
/// stays in the ring after all 13 pushes — its id is the recoverable
/// parent on the Noise InfluenceChanged record.
#[test]
fn harness_p3_beta_influence_changed_parent_is_stamp_dirty() {
    let mut engine = full_engine();
    place_source(&mut engine, (SX, SY));
    engine.tick();

    let log = engine
        .resources
        .causal_log
        .get(centre_tile_idx(SX, SY))
        .expect("centre log must exist");

    let noise_stamp_id = log
        .iter()
        .find_map(|e| match e {
            CausalEvent::StampDirty {
                id,
                channel: InfluenceChannel::Noise,
                ..
            } => Some(*id),
            _ => None,
        })
        .expect("Noise StampDirty must survive in the ring");

    let noise_inf_parent = log
        .iter()
        .find_map(|e| match e {
            CausalEvent::InfluenceChanged {
                parent,
                channel: InfluenceChannel::Noise,
                ..
            } => Some(*parent),
            _ => None,
        })
        .expect("Noise InfluenceChanged must be present");

    assert_eq!(
        noise_inf_parent,
        Some(noise_stamp_id),
        "Noise InfluenceChanged.parent must equal the Noise StampDirty id"
    );
}

// ── A5: trace_parents walks the full 3-event chain ───────────────────────────

/// Type A: `trace_parents` returns the events in
/// `[child, parent, grand-parent]` order when all three links survive in
/// the ring. We hand-build a 3-event chain via direct `CausalLogStorage::push`
/// to bypass FIFO eviction — the test exercises the traversal logic in
/// isolation from the engine pipeline.
#[test]
fn harness_p3_beta_chain_depth_three() {
    let mut log = CausalLogStorage::new();
    let tile = 100u32;

    let placed = CausalEvent::BuildingPlaced {
        id: 0,
        parent: None,
        position: (1, 1),
        radius: 1,
        tick: 0,
    };
    let stamp = CausalEvent::StampDirty {
        id: 1,
        parent: Some(0),
        channel: InfluenceChannel::Warmth,
        region: DirtyRegion::new(0, 0, 2, 2),
        tick: 0,
    };
    let influence = CausalEvent::InfluenceChanged {
        id: 2,
        parent: Some(1),
        channel: InfluenceChannel::Warmth,
        position: (1, 1),
        old: 0.0,
        new: 200.0,
        tick: 0,
    };

    log.push(tile, placed);
    log.push(tile, stamp);
    log.push(tile, influence);

    let chain = log.trace_parents(tile, 2);
    assert_eq!(chain.len(), 3, "chain must have depth 3");
    assert!(matches!(chain[0], CausalEvent::InfluenceChanged { .. }));
    assert!(matches!(chain[1], CausalEvent::StampDirty { .. }));
    assert!(matches!(chain[2], CausalEvent::BuildingPlaced { .. }));
    assert_eq!(chain[0].id(), 2, "child id must be the InfluenceChanged id");
    assert_eq!(chain[1].id(), 1, "parent id must be the StampDirty id");
    assert_eq!(chain[2].id(), 0, "root id must be the BuildingPlaced id");
    assert_eq!(chain[2].parent(), None, "root has no parent");
}

// ── A6: cross-channel chains stay disjoint ───────────────────────────────────

/// Type A: each channel's `InfluenceChanged` parent points at *its own*
/// channel's `StampDirty`, not a sibling. Noise InfluenceChanged.parent
/// resolves to a Noise StampDirty; Danger likewise to Danger. The two
/// parent ids differ.
#[test]
fn harness_p3_beta_cross_channel_isolation() {
    let mut engine = full_engine();
    place_source(&mut engine, (SX, SY));
    engine.tick();

    let log = engine
        .resources
        .causal_log
        .get(centre_tile_idx(SX, SY))
        .expect("centre log must exist");

    fn parent_of(log: &sim_core::causal::TileCausalLog, ch: InfluenceChannel) -> u64 {
        log.iter()
            .find_map(|e| match e {
                CausalEvent::InfluenceChanged {
                    parent: Some(p),
                    channel,
                    ..
                } if *channel == ch => Some(*p),
                _ => None,
            })
            .expect("expected InfluenceChanged with Some(parent)")
    }

    let noise_parent = parent_of(log, InfluenceChannel::Noise);
    let danger_parent = parent_of(log, InfluenceChannel::Danger);
    assert_ne!(
        noise_parent, danger_parent,
        "cross-channel chains must reference different StampDirty ids"
    );

    let noise_parent_channel = log
        .iter()
        .find_map(|e| match e {
            CausalEvent::StampDirty { id, channel, .. } if *id == noise_parent => Some(*channel),
            _ => None,
        })
        .expect("Noise InfluenceChanged.parent must resolve to a StampDirty in the ring");
    assert_eq!(noise_parent_channel, InfluenceChannel::Noise);

    let danger_parent_channel = log
        .iter()
        .find_map(|e| match e {
            CausalEvent::StampDirty { id, channel, .. } if *id == danger_parent => Some(*channel),
            _ => None,
        })
        .expect("Danger InfluenceChanged.parent must resolve to a StampDirty in the ring");
    assert_eq!(danger_parent_channel, InfluenceChannel::Danger);
}

// ── A7: chains on different tiles / ticks are independent ────────────────────

/// Type A: two BuildingPlacedEvents queued in different ticks at different
/// tiles produce two distinct chains. The two BuildingPlaced ids differ,
/// and each lives in a different tile's log.
#[test]
fn harness_p3_beta_multi_tick_chains_independent() {
    let mut engine = bss_only_engine();

    place_source(&mut engine, (10, 10));
    engine.tick();
    place_source(&mut engine, (50, 50));
    engine.tick();

    let log_a = engine
        .resources
        .causal_log
        .get(centre_tile_idx(10, 10))
        .expect("first tile must have a log");
    let log_b = engine
        .resources
        .causal_log
        .get(centre_tile_idx(50, 50))
        .expect("second tile must have a log");

    let id_a = log_a
        .iter()
        .find_map(|e| match e {
            CausalEvent::BuildingPlaced { id, .. } => Some(*id),
            _ => None,
        })
        .expect("first tile must record BuildingPlaced");
    let id_b = log_b
        .iter()
        .find_map(|e| match e {
            CausalEvent::BuildingPlaced { id, .. } => Some(*id),
            _ => None,
        })
        .expect("second tile must record BuildingPlaced");

    assert_ne!(id_a, id_b, "two buildings must own distinct event ids");

    let stamps_a: Vec<&CausalEvent> = log_a
        .iter()
        .filter(|e| matches!(e, CausalEvent::StampDirty { .. }))
        .collect();
    let stamps_b: Vec<&CausalEvent> = log_b
        .iter()
        .filter(|e| matches!(e, CausalEvent::StampDirty { .. }))
        .collect();
    assert_eq!(
        stamps_a.len(),
        6,
        "tile A must record 6 StampDirty events (one per channel)"
    );
    assert_eq!(
        stamps_b.len(),
        6,
        "tile B must record 6 StampDirty events (one per channel)"
    );
    for s in &stamps_a {
        assert_eq!(
            s.parent(),
            Some(id_a),
            "every StampDirty on tile A must reference tile A's BuildingPlaced id"
        );
        assert_ne!(
            s.parent(),
            Some(id_b),
            "no StampDirty on tile A may reference tile B's BuildingPlaced id"
        );
    }
    for s in &stamps_b {
        assert_eq!(
            s.parent(),
            Some(id_b),
            "every StampDirty on tile B must reference tile B's BuildingPlaced id"
        );
        assert_ne!(
            s.parent(),
            Some(id_a),
            "no StampDirty on tile B may reference tile A's BuildingPlaced id"
        );
    }
}

// ── A8: trace_parents terminates gracefully when parent is evicted ───────────

/// Type A: after a full Phase 2 tick the Warmth `StampDirty` (BSS slot 1)
/// has been evicted by the six `InfluenceChanged` pushes, but the Warmth
/// `InfluenceChanged` still references its parent id. `trace_parents`
/// returns a one-entry chain — just the InfluenceChanged itself — which
/// is the documented "cause evicted" outcome for the "왜?" UI.
#[test]
fn harness_p3_beta_no_parent_after_eviction() {
    let mut engine = full_engine();
    place_source(&mut engine, (SX, SY));
    engine.tick();

    let centre = centre_tile_idx(SX, SY);
    let log = engine
        .resources
        .causal_log
        .get(centre)
        .expect("centre log must exist");

    let (warmth_inf_id, warmth_inf_parent) = log
        .iter()
        .find_map(|e| match e {
            CausalEvent::InfluenceChanged {
                id,
                parent,
                channel: InfluenceChannel::Warmth,
                ..
            } => Some((*id, *parent)),
            _ => None,
        })
        .expect("Warmth InfluenceChanged must survive in the ring");

    assert!(
        warmth_inf_parent.is_some(),
        "parent was resolved at push time before eviction"
    );

    let warmth_stamp_present = log.iter().any(|e| matches!(
        e,
        CausalEvent::StampDirty {
            channel: InfluenceChannel::Warmth,
            ..
        }
    ));
    assert!(
        !warmth_stamp_present,
        "the Warmth StampDirty should have been FIFO-evicted by InfluenceChanged pushes"
    );

    let chain = engine
        .resources
        .causal_log
        .trace_parents(centre, warmth_inf_id);
    assert_eq!(
        chain.len(),
        1,
        "chain must terminate at the InfluenceChanged since its parent stamp was evicted"
    );
    assert!(matches!(chain[0], CausalEvent::InfluenceChanged { .. }));
    assert_eq!(
        chain[0].id(),
        warmth_inf_id,
        "chain[0] must be the InfluenceChanged we started from"
    );
    assert_eq!(
        chain[0].channel(),
        Some(InfluenceChannel::Warmth),
        "chain[0] must report the Warmth channel"
    );
}
