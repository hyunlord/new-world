//! Harness — Direction-2 slice 2-3a: deposit carried Food into the settlement
//! stockpile (`StockpileDepositSystem`, passive).
//!
//! Plan: stockpile-deposit-2-3a (plan_attempt 2, seed 42, agents 20). Thresholds
//! are LOCKED by the plan; this file transcribes them verbatim.
//!
//! Assertions:
//!   A1  stockpile_food_zero_at_baseline                 (Type A)
//!   A2  controlled_fixture_exact_transfer               (Type A)
//!   A3  member_outside_radius_no_deposit                (Type A)
//!   A4  empty_inventory_member_noop                     (Type A)
//!   A5  missing_member_entity_no_panic                  (Type A)
//!   A6  non_member_within_radius_no_deposit             (Type A)
//!   A7  deposit_conserves_total_food_over_real_transfer (Type A)
//!   A8  only_food_is_deposited                          (Type A)
//!   A9  own_settlement_attribution                      (Type A)
//!   A10 production_scene_stockpile_accumulates          (Type C)
//!   A11 lockstep_determinism_production                 (Type A)
//!   A12 no_agent_freeze_regression                      (Type D)
//!   A13 settlement_and_membership_unaffected            (Type D)
//!
//! Run:
//!   cargo test -p sim-test --test harness_resource_deposit_to_stockpile -- --nocapture

use std::collections::HashSet;

use sim_bridge::ffi::world_node::{bootstrap_spawn_agents, enqueue_building_placed};
use sim_core::components::{
    Agent, AgentId, AgentState, Inventory, ResourceKind, SeekTarget, Settlement, SettlementId,
    INVENTORY_CAPACITY,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::influence::BuildingStampSystem;
use sim_systems::runtime::settlement::StockpileDepositSystem;

const W: u32 = 64;
const H: u32 = 64;
const YEAR_TICKS: u64 = 4380;

// ── controlled-fixture helpers ───────────────────────────────────────────────

/// Engine running ONLY the `StockpileDepositSystem` — no `SettlementSystem`, so
/// manually-inserted settlements + rosters are NOT mutated by membership sync.
/// This lets the proximity/membership gates be probed in isolation (A2-A9).
fn fixture_engine() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    e.register_system(Box::new(StockpileDepositSystem::new()));
    e
}

/// Spawn an agent at `(x, y)` and load its `Inventory` with the given amounts.
/// Returns the agent's stable `AgentId`.
fn spawn_member(e: &mut SimEngine, x: u32, y: u32, loadout: &[(ResourceKind, u32)]) -> AgentId {
    let entity = e.spawn_agent(x, y);
    let aid = e
        .world
        .get::<&Agent>(entity)
        .expect("freshly spawned agent must have Agent component")
        .id;
    let mut inv = Inventory::default();
    for &(kind, n) in loadout {
        inv.add(kind, n);
    }
    e.world
        .insert_one(entity, inv)
        .expect("freshly spawned agent must still exist for Inventory insert");
    aid
}

/// Insert a settlement with a fixed non-origin `formation_tile` and the given
/// member roster directly into `resources.settlements`.
fn insert_settlement(e: &mut SimEngine, id: SettlementId, ft: (u32, u32), members: &[AgentId]) {
    let mut s = Settlement::new_with_id(id, 0);
    s.formation_tile = ft;
    for &m in members {
        s.member_agents.insert(m);
    }
    s.population_stats.current = s.member_agents.len() as u32;
    e.resources.settlements.insert(id, s);
}

/// Stockpile Food count for settlement `sid` (0 if settlement/key absent).
fn stock_food(e: &SimEngine, sid: SettlementId) -> u32 {
    e.resources
        .settlements
        .get(&sid)
        .and_then(|s| s.stockpile.get(&ResourceKind::Food).copied())
        .unwrap_or(0)
}

/// `Inventory` count of `kind` carried by the agent whose id is `aid`.
fn inv_count(e: &SimEngine, aid: AgentId, kind: ResourceKind) -> u32 {
    for (_, (a, inv)) in e.world.query::<(&Agent, &Inventory)>().iter() {
        if a.id == aid {
            return inv.get(kind);
        }
    }
    0
}

// ── production-scene helper (A15 pattern: 64 agents + 3 startup buildings) ─────

/// Production-like scene that forms settlements AND drives gathering — the same
/// shape the 2-2 gather harness A15 uses. `register_default_runtime_systems`
/// now includes `StockpileDepositSystem`, so deposits run in this scene.
fn make_production_engine() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    bootstrap_spawn_agents(&mut e);
    for (x, y) in [(32i32, 32i32), (24, 32), (40, 32)] {
        let ok = enqueue_building_placed(&mut e.resources, x, y, 8);
        assert!(ok, "startup building ({x},{y}) must enqueue");
    }
    let mut bss = BuildingStampSystem::new();
    bss.tick(&mut e.world, &mut e.resources);
    e
}

/// Total stockpile Food summed across all settlements.
fn total_stock_food(e: &SimEngine) -> u64 {
    e.resources
        .settlements
        .values()
        .map(|s| s.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0) as u64)
        .sum()
}

// ═══════════════════════════════════════════════════════════════════════════
// A1 — stockpile_food_zero_at_baseline. Type A: sum == 0 AND per-settlement
// max == 0 at tick 0 BEFORE any deposit pass runs.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_resource_deposit_a1_stockpile_food_zero_at_baseline() {
    let e = make_production_engine();
    // BEFORE any engine.tick(): the deposit pass has not fired.
    let sum: u64 = total_stock_food(&e);
    let per_settlement_max: u32 = e
        .resources
        .settlements
        .values()
        .map(|s| s.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0))
        .max()
        .unwrap_or(0);
    println!("[deposit A1] baseline sum={sum}, per_settlement_max={per_settlement_max}");
    // Type A — exact physical invariant.
    assert_eq!(sum, 0, "A1: stockpile Food sum must be 0 at baseline");
    assert_eq!(
        per_settlement_max, 0,
        "A1: per-settlement stockpile Food max must be 0 at baseline"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A2 — controlled_fixture_exact_transfer. Type A: stockpile Food == K*F AND
// every member Inventory Food == 0. Boundary member at exactly radius pins `<=`.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_resource_deposit_a2_controlled_fixture_exact_transfer() {
    let mut e = fixture_engine();
    let ft = (30u32, 30u32);
    let f: u32 = 4; // 0 < F <= INVENTORY_CAPACITY
    assert!(f > 0 && f <= INVENTORY_CAPACITY, "A2: F must be in (0, cap]");
    // K = 3 members within radius; one at the EXACT boundary tile (cheb == 5).
    let m0 = spawn_member(&mut e, 30, 30, &[(ResourceKind::Food, f)]); // dist 0
    let m1 = spawn_member(&mut e, 33, 32, &[(ResourceKind::Food, f)]); // dist 3
    let m2 = spawn_member(&mut e, 35, 30, &[(ResourceKind::Food, f)]); // dist 5 == radius
    let members = [m0, m1, m2];
    let k = members.len() as u32;
    insert_settlement(&mut e, 1, ft, &members);

    e.tick(); // current_tick 0 → interval-10 deposit pass fires once

    let got = stock_food(&e, 1);
    println!("[deposit A2] K={k} F={f} stockpile={got}");
    // Type A — exact sum deposited.
    assert_eq!(got, k * f, "A2: stockpile Food must equal K*F");
    for m in members {
        assert_eq!(
            inv_count(&e, m, ResourceKind::Food),
            0,
            "A2: member {m} Inventory Food must be fully drained"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// A3 — member_outside_radius_no_deposit. Type A: stockpile == 0 AND inv == F.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_resource_deposit_a3_member_outside_radius_no_deposit() {
    let mut e = fixture_engine();
    let ft = (30u32, 30u32);
    let f: u32 = 5;
    // (36,30): Chebyshev dist 6 > radius 5.
    let m = spawn_member(&mut e, 36, 30, &[(ResourceKind::Food, f)]);
    insert_settlement(&mut e, 1, ft, &[m]);

    for _ in 0..30 {
        e.tick(); // passes at current_tick 0, 10, 20 → ≥3 passes
    }

    println!(
        "[deposit A3] stockpile={} inv={}",
        stock_food(&e, 1),
        inv_count(&e, m, ResourceKind::Food)
    );
    // Type A — proximity gate.
    assert_eq!(stock_food(&e, 1), 0, "A3: out-of-radius member must not deposit");
    assert_eq!(
        inv_count(&e, m, ResourceKind::Food),
        f,
        "A3: out-of-radius member Inventory must be unchanged"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A4 — empty_inventory_member_noop. Type A: stockpile == 0 AND no Food key.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_resource_deposit_a4_empty_inventory_member_noop() {
    let mut e = fixture_engine();
    let ft = (30u32, 30u32);
    // Member within radius carrying 0 Food.
    let m = spawn_member(&mut e, 30, 30, &[]);
    insert_settlement(&mut e, 1, ft, &[m]);

    for _ in 0..30 {
        e.tick();
    }

    let s = e.resources.settlements.get(&1).expect("settlement 1");
    let has_food_key = s.stockpile.contains_key(&ResourceKind::Food);
    println!("[deposit A4] stockpile_food={} has_food_key={has_food_key}", stock_food(&e, 1));
    // Type A — no spurious zero key.
    assert_eq!(stock_food(&e, 1), 0, "A4: empty-handed member must deposit nothing");
    assert!(!has_food_key, "A4: no Food key must be created in the stockpile map");
}

// ═══════════════════════════════════════════════════════════════════════════
// A5 — missing_member_entity_no_panic. Type A: no panic AND valid member's F
// still deposits (stockpile == F).
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_resource_deposit_a5_missing_member_entity_no_panic() {
    let mut e = fixture_engine();
    let ft = (30u32, 30u32);
    let f: u32 = 6;
    let valid = spawn_member(&mut e, 30, 30, &[(ResourceKind::Food, f)]);
    // Roster includes a stale id with NO live entity (never spawned).
    let stale: AgentId = 9_999_999;
    insert_settlement(&mut e, 1, ft, &[valid, stale]);

    e.tick(); // must not panic on the stale id

    println!("[deposit A5] stockpile={}", stock_food(&e, 1));
    // Type A — robustness: valid member still deposits.
    assert_eq!(
        stock_food(&e, 1),
        f,
        "A5: stale roster id must be skipped without blocking the valid member"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A6 — non_member_within_radius_no_deposit. Type A: stockpile == 0 AND inv == F.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_resource_deposit_a6_non_member_within_radius_no_deposit() {
    let mut e = fixture_engine();
    let ft = (30u32, 30u32);
    let f: u32 = 7;
    // Agent WITHIN radius (dist 0) but NOT in any roster.
    let non_member = spawn_member(&mut e, 30, 30, &[(ResourceKind::Food, f)]);
    insert_settlement(&mut e, 1, ft, &[]); // empty roster

    e.tick(); // exactly ONE deposit pass; no SettlementSystem to auto-enroll

    println!(
        "[deposit A6] stockpile={} inv={}",
        stock_food(&e, 1),
        inv_count(&e, non_member, ResourceKind::Food)
    );
    // Type A — membership gate.
    assert_eq!(stock_food(&e, 1), 0, "A6: non-member must not deposit");
    assert_eq!(
        inv_count(&e, non_member, ResourceKind::Food),
        f,
        "A6: non-member Inventory must be unchanged"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A7 — deposit_conserves_total_food_over_real_transfer. Type A: total Food
// after == before AND stockpile after > 0.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_resource_deposit_a7_conserves_total_food() {
    let mut e = fixture_engine();
    let ft = (30u32, 30u32);
    let f: u32 = 4;
    let m0 = spawn_member(&mut e, 30, 30, &[(ResourceKind::Food, f)]);
    let m1 = spawn_member(&mut e, 31, 31, &[(ResourceKind::Food, f)]);
    let m2 = spawn_member(&mut e, 32, 30, &[(ResourceKind::Food, f)]);
    let members = [m0, m1, m2];
    insert_settlement(&mut e, 1, ft, &members);

    let total_before: u64 =
        members.iter().map(|&m| inv_count(&e, m, ResourceKind::Food) as u64).sum::<u64>()
            + stock_food(&e, 1) as u64;

    e.tick();

    let total_after: u64 =
        members.iter().map(|&m| inv_count(&e, m, ResourceKind::Food) as u64).sum::<u64>()
            + stock_food(&e, 1) as u64;

    println!(
        "[deposit A7] total_before={total_before} total_after={total_after} stockpile={}",
        stock_food(&e, 1)
    );
    // Type A — conservation + non-vacuity.
    assert_eq!(total_after, total_before, "A7: total Food must be conserved");
    assert!(stock_food(&e, 1) > 0, "A7: a real transfer must have occurred");
}

// ═══════════════════════════════════════════════════════════════════════════
// A8 — only_food_is_deposited. Type A: stockpile Food == F, no Wood key,
// member Food == 0, member Wood == W (unchanged).
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_resource_deposit_a8_only_food_is_deposited() {
    let mut e = fixture_engine();
    let ft = (30u32, 30u32);
    let f: u32 = 3;
    let w: u32 = 4; // F + W <= INVENTORY_CAPACITY (10)
    let m = spawn_member(&mut e, 30, 30, &[(ResourceKind::Food, f), (ResourceKind::Wood, w)]);
    insert_settlement(&mut e, 1, ft, &[m]);

    e.tick();

    let s = e.resources.settlements.get(&1).expect("settlement 1");
    let stock_wood = s.stockpile.get(&ResourceKind::Wood).copied().unwrap_or(0);
    let has_wood_key = s.stockpile.contains_key(&ResourceKind::Wood);
    println!(
        "[deposit A8] stock_food={} stock_wood={stock_wood} has_wood_key={has_wood_key} inv_food={} inv_wood={}",
        stock_food(&e, 1),
        inv_count(&e, m, ResourceKind::Food),
        inv_count(&e, m, ResourceKind::Wood)
    );
    // Type A — resource selectivity.
    assert_eq!(stock_food(&e, 1), f, "A8: stockpile Food must equal F");
    assert_eq!(stock_wood, 0, "A8: stockpile Wood must be 0");
    assert!(!has_wood_key, "A8: no Wood key must be created");
    assert_eq!(inv_count(&e, m, ResourceKind::Food), 0, "A8: member Food drained");
    assert_eq!(inv_count(&e, m, ResourceKind::Wood), w, "A8: member Wood unchanged");
}

// ═══════════════════════════════════════════════════════════════════════════
// A9 — own_settlement_attribution. Type A: S1 Food == F AND S2 Food == 0.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_resource_deposit_a9_own_settlement_attribution() {
    let mut e = fixture_engine();
    let f: u32 = 5;
    // Two settlements far apart — radii (5) do NOT overlap (dist 30).
    let s1_ft = (15u32, 15u32);
    let s2_ft = (45u32, 45u32);
    // Member of S1, WITHIN S1 radius (dist 0), OUTSIDE S2 radius.
    let m = spawn_member(&mut e, 15, 15, &[(ResourceKind::Food, f)]);
    insert_settlement(&mut e, 1, s1_ft, &[m]);
    insert_settlement(&mut e, 2, s2_ft, &[]); // m is NOT a member of S2

    e.tick();

    println!(
        "[deposit A9] s1={} s2={}",
        stock_food(&e, 1),
        stock_food(&e, 2)
    );
    // Type A — attribution.
    assert_eq!(stock_food(&e, 1), f, "A9: member must deposit into its OWN settlement S1");
    assert_eq!(stock_food(&e, 2), 0, "A9: S2 (not the member's) must receive nothing");
}

// ═══════════════════════════════════════════════════════════════════════════
// A10 — production_scene_stockpile_accumulates. Type C: 100 <= total <= 100000.
// FLOOR is plan-locked (NOT derived from the GREEN run). The GREEN-measured
// total is recorded below for documentation only.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_resource_deposit_to_stockpile() {
    let mut e = make_production_engine();
    for _ in 0..YEAR_TICKS {
        e.tick();
    }
    let total = total_stock_food(&e);
    println!("[deposit A10] total stockpile Food after {YEAR_TICKS} ticks = {total}");
    // Type C — plan-locked floor + gross-runaway ceiling. Do NOT relax.
    assert!(
        total >= 100,
        "A10: total stockpile Food ({total}) must be >= 100 (plan floor)"
    );
    assert!(
        total <= 100_000,
        "A10: total stockpile Food ({total}) must be <= 100000 (gross-runaway ceiling)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A11 — lockstep_determinism_production. Type A: digest byte-identical across
// two runs AND total stockpile Food > 0 in both.
// ═══════════════════════════════════════════════════════════════════════════
type DepositDigest = (Vec<u32>, Vec<(u64, u32)>);

#[test]
fn harness_resource_deposit_a11_lockstep_determinism() {
    fn digest(ticks: u64) -> (DepositDigest, u64) {
        let mut e = make_production_engine();
        for _ in 0..ticks {
            e.tick();
        }
        let mut stock: Vec<u32> = e
            .resources
            .settlements
            .values()
            .map(|s| s.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0))
            .collect();
        stock.sort_unstable();
        let mut inv: Vec<(u64, u32)> = e
            .world
            .query::<(&Agent, &Inventory)>()
            .iter()
            .map(|(_, (a, i))| (a.id, i.get(ResourceKind::Food)))
            .collect();
        inv.sort_unstable();
        let total = total_stock_food(&e);
        ((stock, inv), total)
    }

    let (d1, t1) = digest(2000);
    let (d2, t2) = digest(2000);
    println!("[deposit A11] total1={t1} total2={t2}");
    // Type A — determinism + non-vacuity.
    assert_eq!(d1, d2, "A11: production digests must be byte-identical across runs");
    assert!(t1 > 0 && t2 > 0, "A11: deposits must actually occur in both runs");
}

// ═══════════════════════════════════════════════════════════════════════════
// A12 — no_agent_freeze_regression. Type D: targetless-Seeking observations == 0
// over the full run.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_resource_deposit_a12_no_agent_freeze_regression() {
    let mut e = make_production_engine();
    let mut targetless_seeking = 0u64;
    for _ in 0..YEAR_TICKS {
        e.tick();
        for (_, (state, seek)) in e
            .world
            .query::<(&AgentState, Option<&SeekTarget>)>()
            .iter()
        {
            if matches!(state, AgentState::Seeking { .. }) && seek.is_none() {
                targetless_seeking += 1;
            }
        }
    }
    println!("[deposit A12] targetless_seeking={targetless_seeking}");
    // Type D — freeze regression guard.
    assert_eq!(
        targetless_seeking, 0,
        "A12: a Seeking state must always carry a SeekTarget (freeze regression)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// A13 — settlement_and_membership_unaffected. Type D: settlement count >= 1 AND
// belonging-collapse violations == 0.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn harness_resource_deposit_a13_settlement_and_membership_unaffected() {
    let mut e = make_production_engine();

    let mut ever_member: HashSet<AgentId> = HashSet::new();
    let sample_every = 20u64;
    for t in 1..=YEAR_TICKS {
        e.tick();
        if !t.is_multiple_of(sample_every) {
            continue;
        }
        for s in e.resources.settlements.values() {
            for mid in s.member_agents.iter().copied() {
                ever_member.insert(mid);
            }
        }
    }

    let settlement_count = e.resources.settlements.len();
    let live_ids: HashSet<AgentId> = e
        .world
        .query::<&Agent>()
        .iter()
        .map(|(_, a)| a.id)
        .collect();
    let in_any_roster: HashSet<AgentId> = e
        .resources
        .settlements
        .values()
        .flat_map(|s| s.member_agents.iter().copied())
        .collect();
    let violations: Vec<AgentId> = ever_member
        .iter()
        .copied()
        .filter(|id| live_ids.contains(id) && !in_any_roster.contains(id))
        .collect();

    println!(
        "[deposit A13] settlements={settlement_count} ever_member={} violations={}",
        ever_member.len(),
        violations.len()
    );
    // Type D — regression guards on UNCHANGED systems.
    assert!(settlement_count >= 1, "A13: at least one settlement must exist");
    assert!(
        violations.is_empty(),
        "A13: {} alive member(s) dropped from all rosters — membership collapse regression",
        violations.len()
    );
}
