//! Regression harness for the "moving settlement marker" bug
//! (fix-settlement-marker-fixed-position).
//!
//! HEAD before fix: `collect_settlement_snapshot` exposed ONLY the centroid
//! (live mean of member positions), and `world_renderer._update_settlement_furniture`
//! drew the marker sprite at that centroid. Members wander (Brownian / Seeking
//! / migration), so the centroid — and the on-screen marker — drifts every
//! tick, producing the "why is the building moving?" illusion.
//!
//! The fix promotes the formation tile onto `Settlement.formation_tile`
//! (set once at `run_formation_scan`, fixed for the settlement's lifetime),
//! exposes it additively as `formation_x/formation_y` through the FFI, and
//! re-points the GDScript marker to read it. This harness proves the
//! formation anchor is (1) written to a real tile, (2) faithfully marshalled,
//! (3) FIXED while members move, (4) distinct from the moving centroid,
//! (5) in-bounds, (6) pinned to the static building cluster, and
//! (7) deterministic across runs.
//!
//! The production bootstrap scene is reproduced exactly (mirrors
//! `world_renderer.gd::_ready()` and the settlements-zero regression): 64
//! bootstrap agents + the 3 startup buildings at (32,32)/(24,32)/(40,32)
//! radius 8. `make_stage1_engine` is NOT used — the bug only reproduces with
//! the clustered-building production scene that forms settlements.
//!
//! Run:
//!   cargo test -p sim-test --test harness_settlement_marker_fixed -- --nocapture

use std::collections::HashMap;

use sim_bridge::ffi::world_node::{bootstrap_spawn_agents, enqueue_building_placed};
use sim_bridge::ffi::{collect_settlement_snapshot, SettlementSnapshotRow};
use sim_core::components::{Agent, Position, SettlementId, SETTLEMENT_PROXIMITY_RADIUS};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::influence::BuildingStampSystem;

const W: u32 = 64;
const H: u32 = 64;
/// Adaptive cap (plan A1): FAIL if no settlement forms within this many ticks.
const MAX_FORM_TICKS: u64 = 2000;
/// Fixedness observation window (plan A3 / A4).
const FIXEDNESS_TICKS: u64 = 200;
/// The 3 static bootstrap building tiles placed by the production scene
/// (mirrors `world_renderer.gd::_ready()`). They never move — the formation
/// predicate requires >= 2 of these within Chebyshev 5 of the candidate, so
/// the anchor's correctness (plan A6) is keyed to this static set.
const BUILDING_TILES: [(u32, u32); 3] = [(32, 32), (24, 32), (40, 32)];

/// `(agent_id, (tile_x, tile_y))` member-position record captured at T for the
/// A3 movement precondition. Aliased to keep clippy's `type_complexity` lint
/// quiet on the per-settlement map below.
type MemberPos = (u64, (u32, u32));

// ── helpers ─────────────────────────────────────────────────────────────────

/// Build the REAL production scene WITHOUT advancing time: `SimEngine::new` →
/// `register_default_runtime_systems` → `bootstrap_spawn_agents` (64 agents) →
/// enqueue the 3 startup buildings → `BuildingStampSystem::tick`. Each call
/// constructs an INDEPENDENT engine instance (no shared resources / RNG state)
/// — required for the A7 two-run determinism precondition.
fn build_scene() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    bootstrap_spawn_agents(&mut e);
    for (x, y) in BUILDING_TILES {
        let ok = enqueue_building_placed(&mut e.resources, x as i32, y as i32, 8);
        assert!(ok, "startup building ({x},{y}) must enqueue within bounds");
    }
    let mut bss = BuildingStampSystem::new();
    bss.tick(&mut e.world, &mut e.resources);
    e
}

/// Tick `e` adaptively until `resources.settlements` is non-empty. Returns the
/// achieved tick count `T` (1-based) on success, or `None` if no settlement
/// forms within `cap` ticks.
fn run_until_settlement(e: &mut SimEngine, cap: u64) -> Option<u64> {
    for t in 1..=cap {
        e.tick();
        if !e.resources.settlements.is_empty() {
            return Some(t);
        }
    }
    None
}

/// Independent `Agent.id → (x, y)` lookup over the live world (recomputed here
/// so movement checks do not trust the collector's own arithmetic).
fn agent_pos_map(e: &SimEngine) -> HashMap<u64, (u32, u32)> {
    let mut map = HashMap::new();
    for (_, (agent, pos)) in e.world.query::<(&Agent, &Position)>().iter() {
        map.insert(agent.id, (pos.x, pos.y));
    }
    map
}

/// Chebyshev (chessboard / L∞) distance between two tiles.
fn chebyshev(a: (u32, u32), b: (u32, u32)) -> u32 {
    let dx = a.0.abs_diff(b.0);
    let dy = a.1.abs_diff(b.1);
    dx.max(dy)
}

/// Settlement-keyed snapshot index: `settlement_id → row`.
fn snapshot_by_id(e: &SimEngine) -> HashMap<SettlementId, SettlementSnapshotRow> {
    collect_settlement_snapshot(&e.world, &e.resources.settlements)
        .into_iter()
        .map(|r| (r.settlement_id, r))
        .collect()
}

// ─── Assertion 1: formation_tile_written_not_sentinel ──────────────────────
#[test]
fn harness_marker_fixed_a1_formation_tile_written_not_sentinel() {
    // Type A — adaptive: run until >= 1 settlement, then assert ZERO formed
    // settlements still hold the (0,0) `new_with_id` sentinel. A formed
    // settlement at (0,0) means the set-at-formation step never ran.
    let mut e = build_scene();
    let t = run_until_settlement(&mut e, MAX_FORM_TICKS)
        .unwrap_or_else(|| panic!("A1: no settlement formed within {MAX_FORM_TICKS} ticks"));

    let n = e.resources.settlements.len();
    assert!(n >= 1, "A1: expected >= 1 formed settlement; got {n}");

    let sentinel_violations = e
        .resources
        .settlements
        .values()
        .filter(|s| s.formation_tile == (0, 0))
        .count();
    assert_eq!(
        sentinel_violations, 0,
        "A1: every formed settlement must hold a non-sentinel formation_tile; \
         {sentinel_violations} of {n} still at (0,0)"
    );
    println!("[marker-fixed A1] {n} settlement(s) formed by tick {t}, 0 at (0,0) sentinel ✓");
}

// ─── Assertion 2: snapshot_formation_xy_equals_component_field ─────────────
#[test]
fn harness_marker_fixed_a2_snapshot_formation_xy_equals_component_field() {
    // Type A — the FFI must copy `formation_tile` into `formation_x/y` with no
    // recomputation or offset. Exact equality for every row.
    let mut e = build_scene();
    run_until_settlement(&mut e, MAX_FORM_TICKS)
        .unwrap_or_else(|| panic!("A2: no settlement formed within {MAX_FORM_TICKS} ticks"));

    let rows = collect_settlement_snapshot(&e.world, &e.resources.settlements);
    assert!(!rows.is_empty(), "A2: snapshot must be non-empty");

    let mut mismatches = 0usize;
    for row in &rows {
        let settlement = e
            .resources
            .settlements
            .get(&row.settlement_id)
            .unwrap_or_else(|| panic!("A2: row.settlement_id {} must be live", row.settlement_id));
        if row.formation_x != settlement.formation_tile.0 as i32
            || row.formation_y != settlement.formation_tile.1 as i32
        {
            mismatches += 1;
        }
    }
    assert_eq!(
        mismatches, 0,
        "A2: every row's (formation_x, formation_y) must equal the component \
         formation_tile; {mismatches} mismatch(es)"
    );
    println!("[marker-fixed A2] all {} rows' formation_xy == component field ✓", rows.len());
}

// ─── Assertion 3: marker_position_is_fixed_while_members_move (CORE) ───────
#[test]
fn harness_marker_fixed_a3_marker_position_is_fixed_while_members_move() {
    // Type A — THE feature. Capture each settlement's formation anchor at T,
    // run +200 ticks, recapture. Three conditions ALL required:
    //   (1) non-vacuity: intersection I of settlement_ids present at BOTH
    //       captures has |I| >= 1 (else explicit FAIL — not a vacuous pass).
    //   (2) movement precondition: >= 1 member of a settlement in I moved.
    //   (3) fixedness: for every id in I, (formation_x, formation_y) IDENTICAL
    //       at T and T+200. member_count MAY differ (roster churn).
    let mut e = build_scene();
    run_until_settlement(&mut e, MAX_FORM_TICKS)
        .unwrap_or_else(|| panic!("A3: no settlement formed within {MAX_FORM_TICKS} ticks"));

    let cap_t = snapshot_by_id(&e);
    // Per-settlement member positions at T (for the movement precondition).
    let positions_t = agent_pos_map(&e);
    let members_t: HashMap<SettlementId, Vec<MemberPos>> = e
        .resources
        .settlements
        .iter()
        .map(|(id, s)| {
            let members: Vec<MemberPos> = s
                .member_agents
                .iter()
                .filter_map(|aid| positions_t.get(aid).map(|p| (*aid, *p)))
                .collect();
            (*id, members)
        })
        .collect();

    for _ in 0..FIXEDNESS_TICKS {
        e.tick();
    }

    let cap_t2 = snapshot_by_id(&e);
    let pos_t2 = agent_pos_map(&e);

    // (1) non-vacuity / empty-intersection guard.
    let intersection: Vec<SettlementId> = cap_t
        .keys()
        .filter(|id| cap_t2.contains_key(id))
        .copied()
        .collect();
    assert!(
        !intersection.is_empty(),
        "A3(1): intersection of settlement_ids at T and T+200 must be non-empty \
         (every settlement dissolved/reformed → explicit FAIL, not vacuous pass)"
    );

    // (2) movement precondition — >= 1 member of a settlement in I moved.
    let mut any_member_moved = false;
    for id in &intersection {
        if let Some(members) = members_t.get(id) {
            for (aid, old_pos) in members {
                if let Some(new_pos) = pos_t2.get(aid) {
                    if new_pos != old_pos {
                        any_member_moved = true;
                        break;
                    }
                }
            }
        }
        if any_member_moved {
            break;
        }
    }
    assert!(
        any_member_moved,
        "A3(2): >= 1 member of a settlement in the intersection must have moved \
         over {FIXEDNESS_TICKS} ticks (else fixedness is vacuous AND it flags a freeze regression)"
    );

    // (3) fixedness — anchors identical for every id in I.
    let mut drifted = 0usize;
    for id in &intersection {
        let a = cap_t.get(id).unwrap();
        let b = cap_t2.get(id).unwrap();
        if a.formation_x != b.formation_x || a.formation_y != b.formation_y {
            drifted += 1;
            eprintln!(
                "[marker-fixed A3] settlement {id} anchor DRIFTED: \
                 ({},{}) → ({},{})",
                a.formation_x, a.formation_y, b.formation_x, b.formation_y
            );
        }
    }
    assert_eq!(
        drifted, 0,
        "A3(3): formation anchor must be IDENTICAL at T and T+200 for every \
         settlement in the intersection; {drifted} anchor(s) moved"
    );
    println!(
        "[marker-fixed A3] {} settlement(s) fixed across {FIXEDNESS_TICKS} ticks while members moved ✓",
        intersection.len()
    );
}

// ─── Assertion 4: centroid_moves_contrast_proof (soft) ─────────────────────
#[test]
fn harness_marker_fixed_a4_centroid_moves_contrast_proof() {
    // Type E (soft) — contrast: the centroid (live mean) is expected to shift
    // within 200 ticks because members wander. Demonstrates centroid !=
    // formation_tile (the root cause being fixed). Floor-rounding could mask a
    // small net drift for a given seed — a miss warrants investigation, not an
    // automatic bug verdict. Both captures are eprintln'd regardless.
    let mut e = build_scene();
    run_until_settlement(&mut e, MAX_FORM_TICKS)
        .unwrap_or_else(|| panic!("A4: no settlement formed within {MAX_FORM_TICKS} ticks"));

    let cap_t = snapshot_by_id(&e);
    for _ in 0..FIXEDNESS_TICKS {
        e.tick();
    }
    let cap_t2 = snapshot_by_id(&e);

    let mut changed = 0usize;
    for (id, a) in &cap_t {
        if let Some(b) = cap_t2.get(id) {
            eprintln!(
                "[marker-fixed A4] settlement {id} centroid: ({},{}) → ({},{}); \
                 anchor: ({},{}) → ({},{})",
                a.centroid_x, a.centroid_y, b.centroid_x, b.centroid_y,
                a.formation_x, a.formation_y, b.formation_x, b.formation_y
            );
            if a.centroid_x != b.centroid_x || a.centroid_y != b.centroid_y {
                changed += 1;
            }
        }
    }
    assert!(
        changed >= 1,
        "A4 (soft): >= 1 settlement's centroid should differ between T and T+200 \
         (members move). 0 changed — investigate (floor-rounding may mask a small \
         net drift for this seed), not an automatic bug verdict"
    );
    println!("[marker-fixed A4] {changed} settlement centroid(s) moved (contrast vs fixed anchor) ✓");
}

// ─── Assertion 5: formation_anchor_in_bounds ───────────────────────────────
#[test]
fn harness_marker_fixed_a5_formation_anchor_in_bounds() {
    // Type A — a formation tile must be a valid map coordinate. Hard invariant,
    // independent of member movement or sampling-tick lag.
    let mut e = build_scene();
    run_until_settlement(&mut e, MAX_FORM_TICKS)
        .unwrap_or_else(|| panic!("A5: no settlement formed within {MAX_FORM_TICKS} ticks"));

    let width = e.resources.influence_grid.width as i32;
    let height = e.resources.influence_grid.height as i32;
    let rows = collect_settlement_snapshot(&e.world, &e.resources.settlements);
    assert!(!rows.is_empty(), "A5: snapshot must be non-empty");

    let mut out_of_bounds = 0usize;
    for row in &rows {
        if row.formation_x < 0
            || row.formation_x >= width
            || row.formation_y < 0
            || row.formation_y >= height
        {
            out_of_bounds += 1;
        }
    }
    assert_eq!(
        out_of_bounds, 0,
        "A5: every formation_x/y must be within [0,{width}) × [0,{height}); \
         {out_of_bounds} out-of-bounds row(s)"
    );
    println!("[marker-fixed A5] all {} anchors in-bounds ✓", rows.len());
}

// ─── Assertion 6: formation_anchor_pinned_to_static_building_cluster ───────
#[test]
fn harness_marker_fixed_a6_formation_anchor_pinned_to_static_building_cluster() {
    // Type A — CORRECTNESS PIN. The formation predicate requires >= 2 buildings
    // co-located within Chebyshev 5 of the candidate, and the anchor IS that
    // candidate. Buildings are STATIC, so ">= 2 buildings within Chebyshev 5 of
    // the anchor" holds REGARDLESS of member drift or snapshot-vs-formation-tick
    // lag. Defeats the constant-anchor gaming vector (a hardcoded (1,1) fails
    // unless it genuinely sits within 5 of >= 2 buildings).
    let mut e = build_scene();
    run_until_settlement(&mut e, MAX_FORM_TICKS)
        .unwrap_or_else(|| panic!("A6: no settlement formed within {MAX_FORM_TICKS} ticks"));

    let rows = collect_settlement_snapshot(&e.world, &e.resources.settlements);
    assert!(!rows.is_empty(), "A6: snapshot must be non-empty");

    let mut underpinned = 0usize;
    for row in &rows {
        let anchor = (row.formation_x as u32, row.formation_y as u32);
        let nearby = BUILDING_TILES
            .iter()
            .filter(|&&b| chebyshev(anchor, b) <= SETTLEMENT_PROXIMITY_RADIUS)
            .count();
        if nearby < 2 {
            underpinned += 1;
            eprintln!(
                "[marker-fixed A6] settlement {} anchor ({},{}) has only {nearby} \
                 building(s) within Chebyshev {SETTLEMENT_PROXIMITY_RADIUS}",
                row.settlement_id, anchor.0, anchor.1
            );
        }
    }
    assert_eq!(
        underpinned, 0,
        "A6: every formed settlement's anchor must have >= 2 static building \
         tiles within Chebyshev {SETTLEMENT_PROXIMITY_RADIUS}; {underpinned} underpinned"
    );
    println!("[marker-fixed A6] all {} anchors pinned to >= 2 static buildings ✓", rows.len());
}

// ─── Assertion 7: formation_xy_deterministic_across_runs ───────────────────
#[test]
fn harness_marker_fixed_a7_formation_xy_deterministic_across_runs() {
    // Type A — seed 42 is deterministic; two independent engine instances must
    // produce identical anchors. Run 1 establishes the achieved adaptive
    // T_form; run 2 (a freshly-constructed engine, no shared state) runs to that
    // SAME T_form. Compare (settlement_id → (formation_x, formation_y)).
    let mut e1 = build_scene();
    let t_form = run_until_settlement(&mut e1, MAX_FORM_TICKS)
        .unwrap_or_else(|| panic!("A7: run 1 formed no settlement within {MAX_FORM_TICKS} ticks"));

    let mut e2 = build_scene();
    for _ in 0..t_form {
        e2.tick();
    }

    let map1: HashMap<SettlementId, (i32, i32)> = snapshot_by_id(&e1)
        .into_iter()
        .map(|(id, r)| (id, (r.formation_x, r.formation_y)))
        .collect();
    let map2: HashMap<SettlementId, (i32, i32)> = snapshot_by_id(&e2)
        .into_iter()
        .map(|(id, r)| (id, (r.formation_x, r.formation_y)))
        .collect();

    assert!(!map1.is_empty(), "A7: run 1 snapshot must be non-empty");
    assert_eq!(
        map1, map2,
        "A7: two independent seed-42 runs to T_form={t_form} must yield identical \
         settlement_ids AND identical formation_x/y; run1={map1:?} run2={map2:?}"
    );
    println!(
        "[marker-fixed A7] {} settlement anchor(s) identical across two independent runs (T_form={t_form}) ✓",
        map1.len()
    );
}
