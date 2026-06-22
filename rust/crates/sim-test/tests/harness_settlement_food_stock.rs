//! Covering harness for Direction-2 slice 2-5a — settlement stockpile Food
//! surfaced additively through the settlement snapshot FFI.
//!
//! The ONLY headless-testable surface in this slice is the pure-Rust collector
//! `collect_settlement_snapshot(world, settlements) -> Vec<SettlementSnapshotRow>`
//! (sim-bridge `ffi::world_node`). The new `food_stock` field reads exactly
//! `stockpile.get(&ResourceKind::Food).copied().unwrap_or(0) as i32`. The FFI
//! `Dictionary` build (`settlement_rows_to_dict`, `food_stocks` key) and the
//! GDScript `Label` require a Godot runtime and are OUT of cargo scope — covered
//! by Visual/VLM + the strict GDScript check.
//!
//! `food_stock` is a pure function of the settlement's `stockpile` BTreeMap at
//! collection time — NOT time-dependent. Tick count is therefore only a SETUP
//! concern (a settlement needs >= 1 resolvable member so the collector does not
//! skip it via `if count == 0 { continue; }`). We satisfy that requirement by
//! constructing settlements directly in `resources.settlements` whose members
//! are real bootstrap agents — the plan's explicitly-sanctioned alternative to
//! running the production formation scan. Mirrors the import surface of
//! `harness_settlement_marker_fixed.rs`.
//!
//! Run:
//!   cargo test -p sim-test --test harness_settlement_food_stock -- --nocapture

use std::collections::HashMap;

use sim_bridge::ffi::collect_settlement_snapshot;
use sim_bridge::ffi::world_node::bootstrap_spawn_agents;
use sim_core::components::{Agent, ResourceKind, Settlement, SettlementId};
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::register_default_runtime_systems;

const W: u32 = 64;
const H: u32 = 64;

// ── helpers ─────────────────────────────────────────────────────────────────

/// Build a world that contains real `(Agent, Position)` members WITHOUT
/// advancing time: `SimEngine::new` → `register_default_runtime_systems` →
/// `bootstrap_spawn_agents`. No `tick()` runs, so `resources.settlements` is
/// empty and the test owns the settlement map completely. The collector only
/// needs the live agents (to resolve member positions) plus the settlement map.
fn scene() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    bootstrap_spawn_agents(&mut e);
    e
}

/// Sorted, deduplicated list of live `Agent.id`s in the world — the pool of
/// resolvable member ids the collector can join against.
fn agent_ids(e: &SimEngine) -> Vec<u64> {
    let mut v: Vec<u64> = e.world.query::<&Agent>().iter().map(|(_, a)| a.id).collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// Construct a settlement with the given members, formation anchor, and an
/// optional Food count injected via the real `Settlement::store` path.
fn make_settlement(
    id: SettlementId,
    formation: (u32, u32),
    members: &[u64],
    food: Option<u32>,
) -> Settlement {
    let mut s = Settlement::new_with_id(id, 0);
    for &m in members {
        s.add_member_agent(m);
    }
    s.formation_tile = formation;
    if let Some(f) = food {
        s.store(ResourceKind::Food, f);
    }
    s
}

// ─── Assertion 1: food_stock_equals_injected_food_count ────────────────────
#[test]
fn harness_food_stock_a1_food_stock_equals_injected_food_count() {
    // Type A — the collector copies `stockpile[Food]` into `food_stock` with no
    // scaling or truncation. Inject the distinctive value 42 and read it back.
    let mut e = scene();
    let ids = agent_ids(&e);
    assert!(!ids.is_empty(), "A1: bootstrap must spawn >= 1 agent");

    let s = make_settlement(1, (10, 10), &[ids[0]], Some(42));
    e.resources.settlements.insert(1, s);

    let rows = collect_settlement_snapshot(&e.world, &e.resources.settlements);
    let row = rows
        .iter()
        .find(|r| r.settlement_id == 1)
        .expect("A1: settlement 1 (1 resolvable member) must be emitted");

    // Type A: exact equality with the injected u32 cast to i32.
    assert_eq!(
        row.food_stock, 42,
        "A1: food_stock must equal the injected Food count exactly; got {}",
        row.food_stock
    );
    println!("[food-stock A1] food_stock == 42 (injected) ✓");
}

// ─── Assertion 2: food_stock_is_zero_when_no_food_stocked ──────────────────
#[test]
fn harness_food_stock_a2_food_stock_is_zero_when_no_food_stocked() {
    // Type A — the `unwrap_or(0)` default branch. A settlement whose stockpile
    // has NO Food key must yield exactly 0, never garbage.
    let mut e = scene();
    let ids = agent_ids(&e);
    assert!(!ids.is_empty(), "A2: bootstrap must spawn >= 1 agent");

    let s = make_settlement(1, (10, 10), &[ids[0]], None);
    assert!(
        !s.stockpile.contains_key(&ResourceKind::Food),
        "A2: precondition — settlement must have no Food key stocked"
    );
    e.resources.settlements.insert(1, s);

    let rows = collect_settlement_snapshot(&e.world, &e.resources.settlements);
    let row = rows
        .iter()
        .find(|r| r.settlement_id == 1)
        .expect("A2: settlement 1 must be emitted");

    // Type A: exact 0.
    assert_eq!(
        row.food_stock, 0,
        "A2: absent Food key must yield food_stock == 0; got {}",
        row.food_stock
    );
    println!("[food-stock A2] food_stock == 0 (no Food stocked) ✓");
}

// ─── Assertion 3: food_stock_counts_food_only_not_other_kinds ──────────────
#[test]
fn harness_food_stock_a3_food_stock_counts_food_only_not_other_kinds() {
    // Type A — the collector keys solely on ResourceKind::Food. Distinct
    // discriminating values make the likely errors detectable: sum → 67,
    // wrong key → 7/13/5, correct → 42.
    let mut e = scene();
    let ids = agent_ids(&e);
    assert!(!ids.is_empty(), "A3: bootstrap must spawn >= 1 agent");

    let mut s = make_settlement(1, (10, 10), &[ids[0]], Some(42));
    s.store(ResourceKind::Water, 7);
    s.store(ResourceKind::Wood, 13);
    s.store(ResourceKind::Stone, 5);
    e.resources.settlements.insert(1, s);

    let rows = collect_settlement_snapshot(&e.world, &e.resources.settlements);
    let row = rows
        .iter()
        .find(|r| r.settlement_id == 1)
        .expect("A3: settlement 1 must be emitted");

    // Type A: exactly the Food count — NOT the sum (67), NOT a wrong key.
    assert_eq!(
        row.food_stock, 42,
        "A3: food_stock must reflect ONLY Food (42); not the sum (67) nor a \
         wrong-key read (7/13/5); got {}",
        row.food_stock
    );
    println!("[food-stock A3] food_stock == 42 with Water/Wood/Stone also present ✓");
}

// ─── Assertion 4: rows_sorted_and_food_stock_parity ────────────────────────
#[test]
fn harness_food_stock_a4_rows_sorted_and_food_stock_parity() {
    // Type A — with >= 2 settlements carrying DIFFERENT Food counts:
    //   (a) rows are strictly ascending by settlement_id,
    //   (b) rows.len() == count of >= 1-resolvable-member settlements,
    //   (c) each row's food_stock reflects ITS OWN settlement's injected count.
    let mut e = scene();
    let ids = agent_ids(&e);
    assert!(ids.len() >= 3, "A4: need >= 3 distinct agents for 3 settlements");

    // Distinct Food per settlement; insert in NON-ascending id order so the
    // collector's sort is genuinely exercised.
    let source: HashMap<SettlementId, u32> =
        [(3u32, 7u32), (1u32, 42u32), (2u32, 99u32)].into_iter().collect();
    e.resources
        .settlements
        .insert(3, make_settlement(3, (30, 30), &[ids[2]], Some(7)));
    e.resources
        .settlements
        .insert(1, make_settlement(1, (10, 10), &[ids[0]], Some(42)));
    e.resources
        .settlements
        .insert(2, make_settlement(2, (20, 20), &[ids[1]], Some(99)));

    let rows = collect_settlement_snapshot(&e.world, &e.resources.settlements);

    // (b) length parity — every settlement has 1 resolvable member.
    assert_eq!(
        rows.len(),
        source.len(),
        "A4(b): rows.len() must equal the count of >= 1-member settlements"
    );

    // (a) strictly ascending settlement_id order.
    let mut sort_violations = 0usize;
    for w in rows.windows(2) {
        if w[0].settlement_id >= w[1].settlement_id {
            sort_violations += 1;
        }
    }
    assert_eq!(
        sort_violations, 0,
        "A4(a): rows must be strictly ascending by settlement_id; {sort_violations} violation(s)"
    );

    // (c) per-row cross-check against the source map (defeats constant-value gaming).
    let mut mismatches = 0usize;
    for row in &rows {
        let expected = *source.get(&row.settlement_id).unwrap_or_else(|| {
            panic!("A4: unexpected settlement_id {} in rows", row.settlement_id)
        });
        if row.food_stock != expected as i32 {
            mismatches += 1;
        }
    }
    assert_eq!(
        mismatches, 0,
        "A4(c): every row must carry its OWN settlement's Food count; {mismatches} mismatch(es)"
    );
    println!("[food-stock A4] {} rows sorted, length-parity, per-row food_stock correct ✓", rows.len());
}

// ─── Assertion 5: additive_change_preserves_existing_row_fields ────────────
#[test]
fn harness_food_stock_a5_additive_change_preserves_existing_row_fields() {
    // Type D — regression guard for the additive-field change: the 7 pre-existing
    // fields must still populate correctly alongside the new food_stock.
    let mut e = scene();
    let ids = agent_ids(&e);
    assert!(ids.len() >= 2, "A5: need >= 2 distinct agents");

    let formation = (12u32, 34u32);
    let members = [ids[0], ids[1]];
    let s = make_settlement(7, formation, &members, Some(55));
    e.resources.settlements.insert(7, s);

    let rows = collect_settlement_snapshot(&e.world, &e.resources.settlements);
    let row = rows
        .iter()
        .find(|r| r.settlement_id == 7)
        .expect("A5: settlement 7 must be emitted");

    // Existing fields preserved (Type D exact checks).
    assert_eq!(row.settlement_id, 7, "A5: settlement_id preserved");
    assert_eq!(
        row.member_count, 2,
        "A5: member_count must equal resolvable-member tally (2)"
    );
    assert_eq!(
        row.formation_x, formation.0 as i32,
        "A5: formation_x must equal Settlement::formation_tile.0"
    );
    assert_eq!(
        row.formation_y, formation.1 as i32,
        "A5: formation_y must equal Settlement::formation_tile.1"
    );
    // New field present and populated.
    assert_eq!(row.food_stock, 55, "A5: new food_stock field populated");
    println!("[food-stock A5] existing fields preserved alongside food_stock ✓");
}

// ─── Assertion 6: zero_member_settlement_with_food_still_skipped ───────────
#[test]
fn harness_food_stock_a6_zero_member_settlement_with_food_still_skipped() {
    // Type A — stocking Food must NOT override the `if count == 0 { continue; }`
    // skip. The 0-resolvable-member settlement (its only member id is bogus /
    // unresolvable) is absent from rows; the valid settlement is present.
    let mut e = scene();
    let ids = agent_ids(&e);
    assert!(!ids.is_empty(), "A6: bootstrap must spawn >= 1 agent");

    // Valid: 1 real resolvable member, Food stocked.
    e.resources
        .settlements
        .insert(1, make_settlement(1, (10, 10), &[ids[0]], Some(10)));
    // 0-resolvable-member: a bogus member id that no live agent matches, yet
    // Food is stocked. Must STILL be skipped (count resolves to 0).
    let bogus: u64 = 9_999_999;
    assert!(!ids.contains(&bogus), "A6: bogus member id must not collide with a real agent");
    e.resources
        .settlements
        .insert(2, make_settlement(2, (20, 20), &[bogus], Some(99)));

    let rows = collect_settlement_snapshot(&e.world, &e.resources.settlements);

    let has_valid = rows.iter().any(|r| r.settlement_id == 1);
    let has_skipped = rows.iter().filter(|r| r.settlement_id == 2).count();
    assert!(has_valid, "A6: the valid 1-member settlement must be present");
    assert_eq!(
        has_skipped, 0,
        "A6: a 0-resolvable-member settlement must be skipped even with Food stocked; \
         found {has_skipped} row(s) for it"
    );
    println!("[food-stock A6] 0-member settlement with Food skipped; valid present ✓");
}
