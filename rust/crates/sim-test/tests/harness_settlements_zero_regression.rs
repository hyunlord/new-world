//! Regression harness for the "Settlements 0" HUD bug (fix-settlements-zero-hud).
//!
//! HEAD before fix: `collect_settlement_snapshot` iterated the ECS world's
//! `query::<&Settlement>()`, but no `Settlement` is ever `world.spawn`ed — the
//! `SettlementSystem` stores settlements in `resources.settlements`
//! (`HashMap<SettlementId, Settlement>`). The world query was therefore always
//! empty → the FFI snapshot returned 0 rows → `hud_topbar.gd` printed
//! "Settlements 0" forever, even though `resources.settlements` held real
//! settlements.
//!
//! This harness reproduces the EXACT production scene (bootstrap 64 agents +
//! the 3 startup buildings at (32,32)/(24,32)/(40,32) radius 8) and asserts the
//! snapshot fed by `resources.settlements` matches the live settlement store.
//! The standard `make_stage1_engine(42, 20)` is NOT used because the bug only
//! reproduces with the clustered-building production scene that forms
//! settlements.
//!
//! Run:
//!   cargo test -p sim-test --test harness_settlements_zero_regression -- --nocapture

use std::collections::HashMap;

use sim_bridge::ffi::world_node::{bootstrap_spawn_agents, enqueue_building_placed};
use sim_bridge::ffi::{collect_settlement_snapshot, SettlementSnapshotRow};
use sim_core::components::{Agent, Position, Settlement, SettlementId};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::influence::BuildingStampSystem;

const W: u32 = 64;
const H: u32 = 64;
const RUN_TICKS: u64 = 300;

// ── helpers ─────────────────────────────────────────────────────────────────

/// The REAL production scene: `SimEngine::new` → `register_default_runtime_systems`
/// → `bootstrap_spawn_agents` (64 agents) → enqueue the 3 startup buildings at
/// (32,32)/(24,32)/(40,32) radius 8 → `BuildingStampSystem::tick` → run
/// `RUN_TICKS` ticks. Mirrors `world_renderer.gd::_ready()`.
fn production_scene() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    bootstrap_spawn_agents(&mut e);
    for (x, y) in [(32i32, 32i32), (24, 32), (40, 32)] {
        let ok = enqueue_building_placed(&mut e.resources, x, y, 8);
        assert!(ok, "startup building ({x},{y}) must enqueue within bounds");
    }
    let mut bss = BuildingStampSystem::new();
    bss.tick(&mut e.world, &mut e.resources);
    for _ in 0..RUN_TICKS {
        e.tick();
    }
    e
}

/// A fresh engine with NO buildings — bootstrap agents only, `RUN_TICKS` ticks.
fn no_building_scene() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    bootstrap_spawn_agents(&mut e);
    for _ in 0..RUN_TICKS {
        e.tick();
    }
    e
}

/// Independent `Agent.id → (x, y)` lookup over the live world (mirrors the
/// collector's first pass, recomputed here so assertions do not trust the
/// collector's own arithmetic).
fn agent_pos_map(e: &SimEngine) -> HashMap<u64, (u32, u32)> {
    let mut map = HashMap::new();
    for (_, (agent, pos)) in e.world.query::<(&Agent, &Position)>().iter() {
        map.insert(agent.id, (pos.x, pos.y));
    }
    map
}

/// Number of a settlement's `member_agents` resolvable to a live world Position.
fn resolvable_member_count(settlement: &Settlement, positions: &HashMap<u64, (u32, u32)>) -> u32 {
    settlement
        .member_agents
        .iter()
        .filter(|id| positions.contains_key(*id))
        .count() as u32
}

/// Count of live settlements that have ≥1 member resolvable to a world Position
/// (i.e. the settlements the collector will NOT skip).
fn settlements_with_resolvable_members(e: &SimEngine) -> usize {
    let positions = agent_pos_map(e);
    e.resources
        .settlements
        .values()
        .filter(|s| resolvable_member_count(s, &positions) >= 1)
        .count()
}

// ─── Assertion 1: production_scene_forms_settlements ───────────────────────
#[test]
fn harness_settlements_zero_a1_production_scene_forms_settlements() {
    // Type C — anti-regression for the simulation half. Threshold >= 1
    // (observed 3 by tick 250). If this drops to 0 the test premise is void
    // and Assertion 2's match would be vacuously satisfied by 0 == 0.
    let e = production_scene();
    let n = e.resources.settlements.len();
    assert!(
        n >= 1,
        "A1: production scene must form >= 1 settlement after {RUN_TICKS} ticks; got {n}"
    );
    println!("[settlements-zero A1] {n} settlement(s) formed in the production scene ✓");
}

// ─── Assertion 2: snapshot_count_matches_live_settlements (CORE BUG GUARD) ──
#[test]
fn harness_settlements_zero_a2_snapshot_count_matches_live_settlements() {
    // Type D — THE regression guard for the FFI source bug. Pre-fix the
    // collector queried the always-empty world Settlement table → snapshot
    // len 0 while resources held 3. Post-fix the collector reads
    // resources.settlements → lengths match. The `>= 1` half is essential:
    // a bare `==` passes vacuously when both are 0.
    let e = production_scene();
    let snapshot = collect_settlement_snapshot(&e.world, &e.resources.settlements);
    let expected = settlements_with_resolvable_members(&e);

    assert!(
        !snapshot.is_empty(),
        "A2: snapshot must be NON-EMPTY in a scene with live settlements; got {}",
        snapshot.len()
    );
    assert_eq!(
        snapshot.len(),
        expected,
        "A2: snapshot length ({}) must equal the count of settlements with \
         resolvable members ({expected})",
        snapshot.len()
    );
    println!(
        "[settlements-zero A2] snapshot_len {} == resolvable-settlement count {} (>=1) ✓",
        snapshot.len(),
        expected
    );
}

// ─── Assertion 3: every_snapshot_row_wellformed ────────────────────────────
#[test]
fn harness_settlements_zero_a3_every_snapshot_row_wellformed() {
    // Type A — invariant. Collector skips count==0 settlements, so every
    // emitted row necessarily has member_count >= 1; its settlement_id must
    // be a live settlement key (no fabricated/garbage rows).
    let e = production_scene();
    let snapshot = collect_settlement_snapshot(&e.world, &e.resources.settlements);
    let live_ids: std::collections::HashSet<SettlementId> =
        e.resources.settlements.keys().copied().collect();

    let mut violations = 0usize;
    for row in &snapshot {
        if row.member_count == 0 {
            violations += 1;
        }
        if !live_ids.contains(&row.settlement_id) {
            violations += 1;
        }
    }
    assert_eq!(
        violations, 0,
        "A3: every row must have member_count >= 1 AND settlement_id ∈ live ids; \
         {violations} violation(s) across {} rows",
        snapshot.len()
    );
    println!(
        "[settlements-zero A3] all {} rows well-formed (member_count >= 1, id live) ✓",
        snapshot.len()
    );
}

// ─── Assertion 4: member_count_equals_resolvable_members ───────────────────
#[test]
fn harness_settlements_zero_a4_member_count_equals_resolvable_members() {
    // Type A — invariant / centroid-correctness proxy. For each row,
    // row.member_count must equal the independently-recomputed count of that
    // settlement's members resolvable to a Position.
    let e = production_scene();
    let snapshot = collect_settlement_snapshot(&e.world, &e.resources.settlements);
    let positions = agent_pos_map(&e);

    let mut mismatches = 0usize;
    for row in &snapshot {
        let settlement = e
            .resources
            .settlements
            .get(&row.settlement_id)
            .unwrap_or_else(|| panic!("A4: row.settlement_id {} must be live", row.settlement_id));
        let recomputed = resolvable_member_count(settlement, &positions);
        if row.member_count != recomputed {
            mismatches += 1;
        }
    }
    assert_eq!(
        mismatches, 0,
        "A4: every row's member_count must equal the resolvable-member count; \
         {mismatches} mismatch(es)"
    );
    println!(
        "[settlements-zero A4] member_count == resolvable members for all {} rows ✓",
        snapshot.len()
    );
}

// ─── Assertion 5: anti_circular_no_buildings_no_rows ───────────────────────
#[test]
fn harness_settlements_zero_a5_anti_circular_no_buildings_no_rows() {
    // Type A (b) + Type C (a). With NO buildings, formation never fires:
    // (a) settlements.len() == 0 [observed], and (b) the collector must emit
    // 0 rows from the empty store — it must NOT fabricate rows. This is the
    // anti-circular proof that Assertion 2 is not passing by coincidence.
    let e = no_building_scene();
    let n = e.resources.settlements.len();
    assert_eq!(
        n, 0,
        "A5(a): no-building scene must form 0 settlements; got {n}"
    );
    let snapshot = collect_settlement_snapshot(&e.world, &e.resources.settlements);
    assert_eq!(
        snapshot.len(),
        0,
        "A5(b): collector must emit 0 rows from an empty settlement store; got {}",
        snapshot.len()
    );
    println!("[settlements-zero A5] no buildings → 0 settlements → 0 snapshot rows ✓");
}

// ─── Assertion 6: snapshot_determinism ─────────────────────────────────────
#[test]
fn harness_settlements_zero_a6_snapshot_determinism() {
    // Type A — deterministic simulation invariant (seed 42, fixed bootstrap).
    // Two identical runs must produce identical settlement stores and therefore
    // identical snapshots. The collector returns rows sorted by settlement_id,
    // so the full (settlement_id, member_count) sequence is also compared.
    fn run() -> Vec<(SettlementId, u32)> {
        let e = production_scene();
        let snapshot: Vec<SettlementSnapshotRow> =
            collect_settlement_snapshot(&e.world, &e.resources.settlements);
        snapshot
            .iter()
            .map(|r| (r.settlement_id, r.member_count))
            .collect()
    }
    let a = run();
    let b = run();
    assert!(!a.is_empty(), "A6: non-empty floor — settlements must exist");
    assert_eq!(
        a.len(),
        b.len(),
        "A6: two identical runs must yield identical snapshot lengths"
    );
    assert_eq!(
        a, b,
        "A6: two identical runs must yield identical (settlement_id, member_count) sequences \
         (collector returns rows sorted by settlement_id)"
    );
    println!(
        "[settlements-zero A6] determinism: {} rows identical across two runs ✓",
        a.len()
    );
}
