//! V7 Phase 12-γ — Settlement Centroid Furniture Placeholder harness.
//!
//! Static-inspection + smoke-test harness for Phase 12-γ:
//!   - `rust/crates/sim-bridge/src/ffi/world_node.rs` — adds
//!     `SettlementSnapshotRow`, `collect_settlement_snapshot`,
//!     `settlement_rows_to_dict`, and a `#[func] get_settlement_snapshot`
//!     method on `WorldSimNode`.
//!   - `rust/crates/sim-bridge/src/ffi/mod.rs` — re-exports the new symbols.
//!   - `scripts/ui/world_renderer.gd` (modified) — polls the new snapshot,
//!     maintains a keyed dictionary of furniture Sprite2D nodes, reaps
//!     dissolved settlements.
//!   - Regression guards for Phase 4-γ, Phase 11-α + D1, Phase 12-α,
//!     12-β.1, and 12-β.2 A3 invariants.
//!
//! Plan locked thresholds: see plan_final.md for this feature.

use std::collections::HashMap;

use hecs::World;
use sim_bridge::ffi::{collect_settlement_snapshot, SettlementSnapshotRow};
use sim_core::components::{Agent, Position, Settlement, SettlementId};

// ── helpers ───────────────────────────────────────────────────────────────

// ─── Assertion 6: collector_returns_correct_centroid_for_known_membership ─
#[test]
fn harness_p12_gamma_a6_collector_returns_correct_centroid_for_known_membership() {
    // Type: A — spawn 3 agents at known positions + a Settlement holding
    // those 3 agent ids; assert centroid is floor mean and member_count==3.
    let mut world = World::new();

    // Three distinct positions: (10, 20), (12, 22), (14, 24).
    // sum_x = 36 → /3 = 12; sum_y = 66 → /3 = 22.
    let positions = [(10u32, 20u32), (12u32, 22u32), (14u32, 24u32)];
    let agent_ids = [101u64, 102u64, 103u64];

    for ((ax, ay), &aid) in positions.iter().zip(agent_ids.iter()) {
        world.spawn((Agent { id: aid }, Position { x: *ax, y: *ay }));
    }

    let mut settlement = Settlement::new_with_id(7u32, 0);
    for &aid in agent_ids.iter() {
        settlement.add_member_agent(aid);
    }
    // Re-pointed: settlements live in `resources.settlements`
    // (HashMap<SettlementId, Settlement>), NOT as ECS world entities.
    let mut settlements: HashMap<SettlementId, Settlement> = HashMap::new();
    settlements.insert(settlement.settlement_id, settlement);

    let rows: Vec<SettlementSnapshotRow> = collect_settlement_snapshot(&world, &settlements);

    assert_eq!(
        rows.len(),
        1,
        "A6.1: expected 1 settlement row; got {}",
        rows.len()
    );
    let row = rows[0];

    let expected_x: i32 = (10 + 12 + 14) / 3;
    let expected_y: i32 = (20 + 22 + 24) / 3;
    assert_eq!(
        row.centroid_x, expected_x,
        "A6.2: centroid_x must equal floor((10+12+14)/3)={expected_x}; got {}",
        row.centroid_x
    );
    assert_eq!(
        row.centroid_y, expected_y,
        "A6.3: centroid_y must equal floor((20+22+24)/3)={expected_y}; got {}",
        row.centroid_y
    );
    assert_eq!(
        row.member_count, 3,
        "A6.4: member_count must equal 3; got {}",
        row.member_count
    );
    assert_eq!(
        row.settlement_id, 7,
        "A6.5: settlement_id must equal 7; got {}",
        row.settlement_id
    );
    assert_eq!(
        row.entity_bits, 7u64,
        "A6.6: entity_bits must equal settlement_id (7) — no ECS entity exists for a \
         HashMap-sourced settlement; the field remains a stable unique downstream key"
    );
    println!("[P12-γ A6] centroid + member_count + ids correct ✓");
}

// ─── Assertion 7: zero_member_settlement_produces_no_row ──────────────────
#[test]
fn harness_p12_gamma_a7_zero_member_settlement_produces_no_row() {
    // Type: A — Settlement with empty member_agents must be skipped.
    // Re-pointed: settlement lives in the HashMap store; world is empty.
    let world = World::new();
    let empty_settlement = Settlement::new_with_id(99, 0);
    let mut settlements: HashMap<SettlementId, Settlement> = HashMap::new();
    settlements.insert(empty_settlement.settlement_id, empty_settlement);

    let rows = collect_settlement_snapshot(&world, &settlements);
    assert_eq!(
        rows.len(),
        0,
        "A7: empty-member Settlement must produce 0 rows; got {}",
        rows.len()
    );
    println!("[P12-γ A7] empty-member Settlement skipped ✓");
}

// ─── Assertion 8: settlement_with_missing_member_position_is_handled ──────
#[test]
fn harness_p12_gamma_a8_settlement_with_missing_member_position_is_handled() {
    // Type: A — Settlement references one resolvable agent + one stale id.
    // member_count must reflect only the resolvable member; centroid must
    // be only that member's position.
    let mut world = World::new();
    world.spawn((Agent { id: 501 }, Position { x: 5, y: 9 }));

    let mut settlement = Settlement::new_with_id(11, 0);
    settlement.add_member_agent(501); // resolvable
    settlement.add_member_agent(9999); // stale — no Agent with this id
                                       // Re-pointed: settlement lives in the HashMap store; agents stay in world.
    let mut settlements: HashMap<SettlementId, Settlement> = HashMap::new();
    settlements.insert(settlement.settlement_id, settlement);

    let rows = collect_settlement_snapshot(&world, &settlements);
    assert_eq!(rows.len(), 1, "A8.1: expected 1 row; got {}", rows.len());
    let row = rows[0];
    assert_eq!(
        row.member_count, 1,
        "A8.2: member_count must equal resolvable count (1); got {}",
        row.member_count
    );
    assert_eq!(
        row.centroid_x, 5,
        "A8.3: centroid_x must equal only resolvable member's x=5; got {}",
        row.centroid_x
    );
    assert_eq!(
        row.centroid_y, 9,
        "A8.4: centroid_y must equal only resolvable member's y=9; got {}",
        row.centroid_y
    );
    println!("[P12-γ A8] stale member references handled without contamination ✓");
}
