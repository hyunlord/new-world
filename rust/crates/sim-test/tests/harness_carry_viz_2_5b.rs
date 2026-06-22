//! V7 Direction-2 slice 2-5b — carry viz: `AgentSnapshotRow.carried_food` collector.
//!
//! Bridge + GDScript only (no sim-core/systems/engine change), so the sole
//! Rust-side coverage is this pure-collector test on
//! `sim_bridge::ffi::collect_agent_snapshot` — mirroring the
//! `harness_p4_gamma_rendering` pattern (build a `SimEngine` world, spawn
//! agents, assert on the returned rows; no Godot runtime needed).
//!
//! The `carried_foods` `PackedInt32Array` in `agent_rows_to_dict` is built by
//! `carried_foods[i] = rows[i].carried_food` with `resize(rows.len())` —
//! identical in shape to the proven `seek_kinds`/`hungers` additive arrays — so
//! the FFI-marshalling length invariant (`carried_foods.len() == rows.len()`)
//! is structural; this test pins the `carried_food` ROW value the array copies.
//!
//! Assertions track the locked plan (stockpile-viz-2-5b):
//!   A1: Food=7 → carried_food == 7 (sub-capacity exact, not a default/`> 0`).
//!   A2: Water=4, Food=0 → carried_food == 0 (anti-gaming: a `total()`-based
//!       bug would return 4; proves the field reads `get(Food)` specifically).
//!   A3: no Inventory → carried_food == 0 (the `Option<&Inventory>` None branch).
//!   A4: rows.len() == count of (Agent, Position) entities (parallel-array
//!       contract — `Option` must NOT filter rows out).
//!   A5: live Inventory mutation — 7 → 0 on deposit (reads live ECS per call).
//!
//! Run: `cargo test -p sim-test --test harness_carry_viz_2_5b -- --nocapture`

use sim_bridge::ffi::collect_agent_snapshot;
use sim_core::components::{Agent, Inventory, Position, ResourceKind};
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;

const W: u32 = 64;
const H: u32 = 64;

fn fresh_engine() -> SimEngine {
    SimEngine::new(W, H, MaterialRegistry::new())
}

/// Look up the `carried_food` for a specific entity in the snapshot rows.
fn carried_food_for(rows: &[sim_bridge::ffi::AgentSnapshotRow], ent: hecs::Entity) -> i32 {
    rows.iter()
        .find(|r| r.entity_bits == ent.to_bits().get())
        .unwrap_or_else(|| panic!("snapshot row for {ent:?} missing"))
        .carried_food
}

#[test]
fn harness_carry_viz_agent_snapshot_carried_food() {
    let mut e = fresh_engine();

    // Agent A — carries Food = 7 (plan A1: non-trivial, < INVENTORY_CAPACITY=10
    // so add() does not clamp; defeats any default / `> 0` shortcut).
    let a = e.spawn_agent(5, 5);
    let mut inv_a = Inventory::default();
    inv_a.add(ResourceKind::Food, 7);
    let _ = e.world.insert_one(a, inv_a);

    // Agent B — Inventory holds Water = 4 but Food = 0 (plan A2 anti-gaming:
    // an implementation that read inv.total() or the wrong kind returns 4 here).
    let b = e.spawn_agent(6, 6);
    let mut inv_b = Inventory::default();
    inv_b.add(ResourceKind::Water, 4);
    let _ = e.world.insert_one(b, inv_b);

    // Agent C — NO Inventory component at all (plan A3: the None branch → 0).
    let c = e.spawn_agent(7, 7);

    let rows = collect_agent_snapshot(&e.world);

    // Plan A4 — parallel-array contract: one row per (Agent, Position) entity.
    // The Option<&Inventory> in the query tuple must NOT drop the no-inventory
    // agent. Compare against the independently-counted (Agent, Position) set.
    let agent_pos_count = e
        .world
        .query::<(&Agent, &Position)>()
        .iter()
        .count();
    // Type A: rows.len() == count of (Agent, Position) entities (exact).
    assert_eq!(
        rows.len(),
        agent_pos_count,
        "rows.len() must equal the (Agent, Position) entity count (Option non-filtering)"
    );
    // Type A: with three spawned agents, that count is exactly 3.
    assert_eq!(rows.len(), 3, "exactly one snapshot row per spawned agent (3)");

    // Type A: agent carrying Food=7 reports carried_food == 7 (exact).
    assert_eq!(carried_food_for(&rows, a), 7, "Food=7 → carried_food=7 (plan A1)");
    // Type A: Water=4 / Food=0 reports carried_food == 0 (anti-total() guard).
    assert_eq!(
        carried_food_for(&rows, b),
        0,
        "Water=4, Food=0 → carried_food=0 (plan A2: reads get(Food), not total())"
    );
    // Type A: agent with no Inventory reports carried_food == 0 (None branch).
    assert_eq!(carried_food_for(&rows, c), 0, "no Inventory → carried_food=0 (plan A3)");

    println!(
        "[2-5b] carried_food: A(Food=7)=7, B(Water=4,Food=0)=0, C(no-inv)=0; rows={} ✓",
        rows.len()
    );
}

#[test]
fn harness_carry_viz_carried_food_tracks_inventory_change() {
    // Plan A5 — carried_food reflects the LIVE Inventory each call: it drops to
    // 0 when a deposit drains Food. A constant-return stub cannot satisfy 7 → 0.
    let mut e = fresh_engine();
    let a = e.spawn_agent(4, 4);
    let mut inv = Inventory::default();
    inv.add(ResourceKind::Food, 7);
    let _ = e.world.insert_one(a, inv);

    let before = collect_agent_snapshot(&e.world);
    // Type A: first collection carried_food == 7.
    assert_eq!(before[0].carried_food, 7, "before deposit: carried_food=7 (plan A5)");

    // Simulate the real deposit event (2-3a drains Inventory Food → stockpile).
    if let Ok(mut inv) = e.world.get::<&mut Inventory>(a) {
        let n = inv.get(ResourceKind::Food);
        inv.remove(ResourceKind::Food, n);
    }

    let after = collect_agent_snapshot(&e.world);
    // Type A: second collection carried_food == 0.
    assert_eq!(after[0].carried_food, 0, "after deposit: carried_food drops to 0 (plan A5)");
    println!("[2-5b] carried_food tracks live Inventory: 7 → 0 on deposit ✓");
}
