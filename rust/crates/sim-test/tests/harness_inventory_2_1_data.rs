//! Direction-2 slice 2-1 — Resource carry/store data structures harness.
//!
//! feature: inventory-2-1-data
//! plan_attempt: 2
//! code_attempt: 1
//! seed: 42
//! agent_count: 20
//! lane: --quick (pure sim-core data structures; no engine run)
//!
//! This is a unit-style harness: every assertion is a deterministic Type A
//! logical/mathematical invariant (or one Type D additive-field regression
//! guard, A7) constructed directly from the structs at ticks=0. There is NO
//! `make_stage1_engine(42, 20)` run — the slice adds only data structures with
//! no behaviour to converge (see plan §"NOT in Scope").
//!
//! Assertion map (1:1 with the locked plan §Assertions):
//!   A1  : ResourceKind `Ord` total order == hardcoded `Food<Water<Wood<Stone`;
//!         scrambled-insertion BTreeMap iterates in Ord order.
//!   A2  : Inventory::add enforces INVENTORY_CAPACITY (total-count cap),
//!         accumulates per-kind, and returns exact overflow.
//!   A3  : Inventory::remove returns actual-removed and drops zeroed keys.
//!   A4  : total()/get() exact across kinds, built via public add().
//!   A5  : Settlement.stockpile store/withdraw symmetry, zero-drop,
//!         zero-quantity no-op, empty default.
//!   A6  : serde RON round-trip preserves Inventory and Settlement.
//!   A7  : Settlement new field is additive (Type D regression guard).
//!   A8  : Settlement::store uses saturating_add (no wrap/panic at u32 ceiling).
//!   A9  : Inventory empty-default boundary.
//!   A10 : ResourceKind has exactly the 4 in-scope variants (scope guard).

use std::collections::BTreeMap;

use sim_core::components::{Inventory, ResourceKind, Settlement, INVENTORY_CAPACITY};

// ---------------------------------------------------------------------------
// A1 — ResourceKind Ord total order is the hardcoded declared sequence.
// Type A: exact logical invariant (no derivation from the declaration).
// ---------------------------------------------------------------------------
#[test]
fn harness_resource_kind_ord_total_order() {
    // (a) Three adjacent strict-less-than relations as HARDCODED LITERALS —
    //     not derived by reading resource_kind.rs. Pins declaration order
    //     against an accidental future reorder.
    assert!(ResourceKind::Food < ResourceKind::Water);
    assert!(ResourceKind::Water < ResourceKind::Wood);
    assert!(ResourceKind::Wood < ResourceKind::Stone);

    // (b) BTreeMap built by SCRAMBLED insertion (Stone, Food, Wood, Water)
    //     must iterate in Ord order, not insertion order.
    let mut m: BTreeMap<ResourceKind, u32> = BTreeMap::new();
    m.insert(ResourceKind::Stone, 1);
    m.insert(ResourceKind::Food, 1);
    m.insert(ResourceKind::Wood, 1);
    m.insert(ResourceKind::Water, 1);
    let keys: Vec<ResourceKind> = m.keys().copied().collect();
    assert_eq!(
        keys,
        vec![
            ResourceKind::Food,
            ResourceKind::Water,
            ResourceKind::Wood,
            ResourceKind::Stone
        ],
        "BTreeMap iteration must follow Ord, not insertion order"
    );
}

// ---------------------------------------------------------------------------
// A2 — Inventory::add enforces INVENTORY_CAPACITY, accumulates per-kind,
//      returns exact overflow.
// Type A: closed-form invariant taken = min(n, CAP - total), return = n - taken.
// ---------------------------------------------------------------------------
#[test]
fn harness_inventory_add_capacity_accumulate_overflow() {
    // (a) read the const (do not hardcode 10).
    let cap = INVENTORY_CAPACITY;
    assert!(cap >= 5, "test math assumes capacity >= 5; locked const is 10");

    // (b) NORMAL path: room >= n → return 0, total grows by exactly n.
    let mut inv = Inventory::default();
    let overflow = inv.add(ResourceKind::Food, 3);
    assert_eq!(overflow, 0, "no overflow when room >= n");
    assert_eq!(inv.total(), 3, "total grows by exactly n");

    // (c) PER-KIND ACCUMULATION: adding into an existing key accumulates.
    let mut inv = Inventory::default();
    inv.add(ResourceKind::Food, 2);
    inv.add(ResourceKind::Food, 3);
    assert_eq!(inv.get(ResourceKind::Food), 5, "add accumulates, no overwrite");

    // (d) fill to exactly capacity.
    let mut inv = Inventory::default();
    let of = inv.add(ResourceKind::Food, cap);
    assert_eq!(of, 0);
    assert_eq!(inv.total(), cap, "total == INVENTORY_CAPACITY at cap");

    // (e) a further add on a full inventory returns exactly n (all overflow).
    let of_full = inv.add(ResourceKind::Water, 4);
    assert_eq!(of_full, 4, "full inventory: all of n overflows");
    assert_eq!(inv.total(), cap, "total unchanged on full add");

    // (f) PARTIAL fill: room < n → return n - room, total reaches cap.
    let mut inv = Inventory::default();
    let room = 2;
    inv.add(ResourceKind::Food, cap - room); // room slots remain
    assert_eq!(inv.total(), cap - room);
    let n = room + 3; // 5
    let of_partial = inv.add(ResourceKind::Wood, n);
    assert_eq!(of_partial, n - room, "partial add returns n - room");
    assert_eq!(inv.total(), cap, "partial add fills to exactly cap");

    // (g) CROSS-KIND cap: total-count cap applies to cross-kind sum.
    let mut inv = Inventory::default();
    let half = cap / 2;
    let rest = cap - half;
    inv.add(ResourceKind::Food, half);
    inv.add(ResourceKind::Water, rest);
    assert_eq!(inv.total(), cap, "two kinds sum to cap");
    let of_cross = inv.add(ResourceKind::Wood, 1);
    assert_eq!(of_cross, 1, "cap is a shared total-count budget, not per-kind");
    assert_eq!(inv.total(), cap, "cross-kind total never exceeds cap");
}

// ---------------------------------------------------------------------------
// A3 — Inventory::remove returns actual-removed and drops zeroed keys.
// Type A: saturating subtraction returning the delta; zero-entry hygiene.
// ---------------------------------------------------------------------------
#[test]
fn harness_inventory_remove_actual_and_drops_zero() {
    // (a) remove-all drops the key.
    let mut inv = Inventory::default();
    inv.add(ResourceKind::Food, 5);
    let r = inv.remove(ResourceKind::Food, 5);
    assert_eq!(r, 5);
    assert!(
        !inv.items.contains_key(&ResourceKind::Food),
        "zeroed key must be dropped"
    );

    // (b) over-remove returns only the held amount, key dropped.
    let mut inv = Inventory::default();
    inv.add(ResourceKind::Water, 3);
    let r = inv.remove(ResourceKind::Water, 10);
    assert_eq!(r, 3, "over-remove returns only held");
    assert!(!inv.items.contains_key(&ResourceKind::Water));

    // (c) remove from absent key returns 0, map unchanged.
    let mut inv = Inventory::default();
    let r = inv.remove(ResourceKind::Stone, 4);
    assert_eq!(r, 0);
    assert!(inv.items.is_empty());

    // (d) partial remove leaving a positive remainder keeps the key.
    let mut inv = Inventory::default();
    inv.add(ResourceKind::Wood, 5);
    let r = inv.remove(ResourceKind::Wood, 2);
    assert_eq!(r, 2);
    assert_eq!(inv.get(ResourceKind::Wood), 3);
    assert!(inv.items.contains_key(&ResourceKind::Wood));

    // (e) zero-quantity remove is a no-op returning 0, key still present.
    let mut inv = Inventory::default();
    inv.add(ResourceKind::Food, 4);
    let r = inv.remove(ResourceKind::Food, 0);
    assert_eq!(r, 0);
    assert_eq!(inv.get(ResourceKind::Food), 4);
    assert!(
        inv.items.contains_key(&ResourceKind::Food),
        "zero-quantity remove must not drop a still-positive key"
    );
}

// ---------------------------------------------------------------------------
// A4 — total() and get() exact across multiple kinds (built via public add()).
// Type A: deterministic sum and lookup.
// ---------------------------------------------------------------------------
#[test]
fn harness_inventory_total_get_exact_multi_kind() {
    let mut inv = Inventory::default();
    inv.add(ResourceKind::Food, 2);
    inv.add(ResourceKind::Water, 1);
    inv.add(ResourceKind::Wood, 4);

    assert_eq!(inv.get(ResourceKind::Food), 2);
    assert_eq!(inv.get(ResourceKind::Water), 1);
    assert_eq!(inv.get(ResourceKind::Wood), 4);
    assert_eq!(inv.get(ResourceKind::Stone), 0, "absent kind reads 0");
    assert_eq!(inv.total(), 7, "total == exact arithmetic sum");
}

// ---------------------------------------------------------------------------
// A5 — Settlement.stockpile store/withdraw symmetry, zero-drop, zero no-op,
//      empty default.
// Type A: store (uncapped saturating_add) + withdraw (mirror of remove).
// ---------------------------------------------------------------------------
#[test]
fn harness_settlement_stockpile_store_withdraw_symmetry() {
    // (a) fresh settlement has an EMPTY stockpile.
    let s = Settlement::new_with_id(1, 0);
    assert!(s.stockpile.is_empty());
    assert_eq!(s.stockpile.len(), 0);

    // (b) store accumulates uncapped.
    let mut s = Settlement::new_with_id(1, 0);
    s.store(ResourceKind::Stone, 5);
    s.store(ResourceKind::Stone, 3);
    assert_eq!(s.stockpile[&ResourceKind::Stone], 8, "uncapped accumulation");

    // (c) withdraw-all returns held and drops the key.
    let r = s.withdraw(ResourceKind::Stone, 8);
    assert_eq!(r, 8);
    assert!(!s.stockpile.contains_key(&ResourceKind::Stone));

    // (d) over-withdraw returns held only, key dropped.
    let mut s = Settlement::new_with_id(1, 0);
    s.store(ResourceKind::Wood, 2);
    let r = s.withdraw(ResourceKind::Wood, 10);
    assert_eq!(r, 2);
    assert!(!s.stockpile.contains_key(&ResourceKind::Wood));

    // (e) withdraw from empty/absent returns 0.
    let mut s = Settlement::new_with_id(1, 0);
    let r = s.withdraw(ResourceKind::Stone, 5);
    assert_eq!(r, 0);

    // (f) zero-quantity no-ops.
    let mut s = Settlement::new_with_id(1, 0);
    s.store(ResourceKind::Wood, 3);
    let r = s.withdraw(ResourceKind::Wood, 0);
    assert_eq!(r, 0, "withdraw(k, 0) returns 0");
    assert_eq!(s.stockpile[&ResourceKind::Wood], 3, "withdraw(k, 0) leaves count");
    assert!(s.stockpile.contains_key(&ResourceKind::Wood));

    let mut s2 = Settlement::new_with_id(2, 0);
    s2.store(ResourceKind::Food, 0);
    assert!(
        s2.stockpile.is_empty(),
        "store(k, 0) must not create a key on an empty stockpile"
    );
}

// ---------------------------------------------------------------------------
// A6 — serde RON round-trip preserves Inventory and Settlement (value equality).
// Type A: round-trip identity over BTreeMap<ResourceKind, u32>.
// ---------------------------------------------------------------------------
#[test]
fn harness_serde_ron_roundtrip_inventory_and_settlement() {
    // (a) Inventory with >= 2 kinds.
    let mut inv = Inventory::default();
    inv.add(ResourceKind::Food, 3);
    inv.add(ResourceKind::Wood, 2);
    let encoded = ron::to_string(&inv).expect("Inventory must serialize to RON");
    let decoded: Inventory = ron::from_str(&encoded).expect("Inventory must deserialize from RON");
    assert_eq!(decoded, inv, "Inventory RON round-trip must be identity");

    // (b) Settlement with a non-empty stockpile (>= 2 kinds).
    let mut st = Settlement::new_with_id(7, 42);
    st.store(ResourceKind::Stone, 5);
    st.store(ResourceKind::Wood, 2);
    let encoded = ron::to_string(&st).expect("Settlement must serialize to RON");
    let decoded: Settlement = ron::from_str(&encoded).expect("Settlement must deserialize from RON");
    assert_eq!(decoded, st, "Settlement RON round-trip must be identity");
}

// ---------------------------------------------------------------------------
// A7 — Settlement new field is additive — PartialEq still derives, stockpile
//      defaults empty (Type D regression guard for the additive change).
// ---------------------------------------------------------------------------
#[test]
fn harness_settlement_additive_field_regression() {
    let a = Settlement::new_with_id(7, 0);
    let b = Settlement::new_with_id(7, 0);

    // (a) reflexive self-equality holds with the new field present.
    assert_eq!(a, b, "two fresh new_with_id(7, 0) instances compare ==");

    // (b) pre-existing collection fields at prior defaults.
    assert!(a.member_agents.is_empty(), "member_agents unchanged from pre-2-1");
    assert!(a.member_buildings.is_empty());
    assert!(a.community_history.is_empty());

    // (c) stockpile defaults empty (contributes nothing to equality).
    assert!(a.stockpile.is_empty(), "stockpile defaults empty");
}

// ---------------------------------------------------------------------------
// A8 — Settlement::store uses saturating_add — no wrap/panic at u32 boundary.
// Type A: saturation invariant at the reachable (uncapped) store path.
// ---------------------------------------------------------------------------
#[test]
fn harness_settlement_store_saturating_add_boundary() {
    let mut s = Settlement::new_with_id(1, 0);
    s.store(ResourceKind::Stone, u32::MAX - 2);
    s.store(ResourceKind::Stone, 10); // crosses the u32 ceiling
    assert_eq!(
        s.stockpile[&ResourceKind::Stone],
        u32::MAX,
        "store must saturate at u32::MAX, never wrap or panic"
    );
}

// ---------------------------------------------------------------------------
// A9 — Inventory empty-default boundary.
// Type A: fresh container is exactly empty (no pre-seeded garbage).
// ---------------------------------------------------------------------------
#[test]
fn harness_inventory_empty_default_boundary() {
    let inv = Inventory::default();
    assert_eq!(inv.total(), 0, "fresh Inventory total is 0");
    assert!(inv.items.is_empty(), "fresh Inventory map is empty");
    for kind in [
        ResourceKind::Food,
        ResourceKind::Water,
        ResourceKind::Wood,
        ResourceKind::Stone,
    ] {
        assert_eq!(inv.get(kind), 0, "fresh Inventory get(k) == 0 for every kind");
    }
}

// ---------------------------------------------------------------------------
// A10 — ResourceKind has exactly the 4 in-scope variants (scope guard).
// Type A: wildcard-free exhaustive match is compile-enforced; a 5th variant
//         breaks the build. The literal list pins the count at 4.
// ---------------------------------------------------------------------------
#[test]
fn harness_resource_kind_exactly_four_variants() {
    let v = ResourceKind::Food;
    // No `_` wildcard arm: a future 5th variant (e.g. Material(MaterialId))
    // would fail to compile here, turning scope creep into a build failure.
    let label = match v {
        ResourceKind::Food => "food",
        ResourceKind::Water => "water",
        ResourceKind::Wood => "wood",
        ResourceKind::Stone => "stone",
    };
    assert_eq!(label, "food");

    let all = [
        ResourceKind::Food,
        ResourceKind::Water,
        ResourceKind::Wood,
        ResourceKind::Stone,
    ];
    assert_eq!(all.len(), 4, "exactly 4 in-scope ResourceKind variants");
}
