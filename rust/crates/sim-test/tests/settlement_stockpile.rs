//! Merged in V8-0 (8.4) from `harness_resource_deposit_to_stockpile.rs`,
//! `harness_stockpile_consume.rs`, `harness_settlement_food_stock.rs`; each
//! source is one module below, unchanged.

// From `harness_resource_deposit_to_stockpile.rs`.
mod deposit {
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
        assert!(
            f > 0 && f <= INVENTORY_CAPACITY,
            "A2: F must be in (0, cap]"
        );
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
        assert_eq!(
            stock_food(&e, 1),
            0,
            "A3: out-of-radius member must not deposit"
        );
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
        println!(
            "[deposit A4] stockpile_food={} has_food_key={has_food_key}",
            stock_food(&e, 1)
        );
        // Type A — no spurious zero key.
        assert_eq!(
            stock_food(&e, 1),
            0,
            "A4: empty-handed member must deposit nothing"
        );
        assert!(
            !has_food_key,
            "A4: no Food key must be created in the stockpile map"
        );
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

        let total_before: u64 = members
            .iter()
            .map(|&m| inv_count(&e, m, ResourceKind::Food) as u64)
            .sum::<u64>()
            + stock_food(&e, 1) as u64;

        e.tick();

        let total_after: u64 = members
            .iter()
            .map(|&m| inv_count(&e, m, ResourceKind::Food) as u64)
            .sum::<u64>()
            + stock_food(&e, 1) as u64;

        println!(
            "[deposit A7] total_before={total_before} total_after={total_after} stockpile={}",
            stock_food(&e, 1)
        );
        // Type A — conservation + non-vacuity.
        assert_eq!(
            total_after, total_before,
            "A7: total Food must be conserved"
        );
        assert!(
            stock_food(&e, 1) > 0,
            "A7: a real transfer must have occurred"
        );
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
        let m = spawn_member(
            &mut e,
            30,
            30,
            &[(ResourceKind::Food, f), (ResourceKind::Wood, w)],
        );
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
        assert_eq!(
            inv_count(&e, m, ResourceKind::Food),
            0,
            "A8: member Food drained"
        );
        assert_eq!(
            inv_count(&e, m, ResourceKind::Wood),
            w,
            "A8: member Wood unchanged"
        );
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
        assert_eq!(
            stock_food(&e, 1),
            f,
            "A9: member must deposit into its OWN settlement S1"
        );
        assert_eq!(
            stock_food(&e, 2),
            0,
            "A9: S2 (not the member's) must receive nothing"
        );
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
        assert_eq!(
            d1, d2,
            "A11: production digests must be byte-identical across runs"
        );
        assert!(
            t1 > 0 && t2 > 0,
            "A11: deposits must actually occur in both runs"
        );
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
            for (_, (state, seek)) in e.world.query::<(&AgentState, Option<&SeekTarget>)>().iter() {
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
        assert!(
            settlement_count >= 1,
            "A13: at least one settlement must exist"
        );
        assert!(
            violations.is_empty(),
            "A13: {} alive member(s) dropped from all rosters — membership collapse regression",
            violations.len()
        );
    }
}

// From `harness_stockpile_consume.rs`.
mod consume {
    //! Harness — Direction-2 slice 2-4: famine-fallback stockpile consumption
    //! (`StockpileConsumeSystem`, passive, priority 142).
    //!
    //! Plan: stockpile-consume-2-4 (plan_attempt 2, seed 42, agents 20). Thresholds
    //! are LOCKED by the plan; this file transcribes them verbatim.
    //!
    //! Assertions:
    //!   A1  controlled_relief_bounded_drain_and_relief        (Type A)
    //!   A2  no_double_dip_ground_eater_skipped                (Type A)
    //!   A3  proximity_and_threshold_gating_noop               (Type A)
    //!   A4  empty_reserve_no_relieve_without_consume          (Type A)
    //!   A5  at_threshold_boundary_relief_fires                (Type A)
    //!   A6  famine_ab_full_reserve_prevents_starvation (PRIMARY, harness_stockpile_famine_relief) (Type A)
    //!   A7  plenty_state_non_interference                     (Type C)
    //!   A8  determinism_lockstep                              (Type A)
    //!   A9  regression (cross-phase) — verified by the workspace gate, not here.
    //!
    //! Run:
    //!   cargo test -p sim-test --test harness_stockpile_consume -- --nocapture

    use std::collections::BTreeSet;

    use sim_bridge::ffi::world_node::{bootstrap_spawn_agents, enqueue_building_placed};
    use sim_core::causal::event::{CausalEvent, DeathReason};
    use sim_core::components::{
        Agent, AgentId, AgentState, BodyHealth, Hunger, ResourceKind, Settlement, SettlementId,
        TargetKind, Thirst, DEFAULT_MAX_HP,
    };
    use sim_core::material::MaterialRegistry;
    use sim_engine::{RuntimeSystem, SimEngine};
    use sim_systems::runtime::influence::BuildingStampSystem;
    use sim_systems::runtime::needs::HungerDecaySystem;
    use sim_systems::runtime::settlement::stockpile_consume::{
        StockpileConsumeSystem, STOCKPILE_RELIEF_FOOD_PER_MEAL, STOCKPILE_RELIEF_HUNGER_THRESHOLD,
    };
    use sim_systems::runtime::survival::StarvationSystem;
    use sim_systems::{
        register_agent_systems, register_combat_systems, register_construction_systems,
        register_decision_systems, register_default_runtime_systems, register_memory_systems,
        register_needs_systems, register_phase2_systems, register_resource_systems,
        register_settlement_systems, register_social_systems, register_stockpile_systems,
        register_survival_systems,
    };

    const W: u32 = 64;
    const H: u32 = 64;
    const THRESH: f32 = STOCKPILE_RELIEF_HUNGER_THRESHOLD; // 50.0
    const PER_MEAL: u32 = STOCKPILE_RELIEF_FOOD_PER_MEAL; // 1

    // ── controlled-fixture helpers ───────────────────────────────────────────────

    /// Engine running ONLY `StockpileConsumeSystem` (interval 5) — no HungerDecay,
    /// no SettlementSystem. A single `e.tick()` from `current_tick == 0` fires
    /// EXACTLY ONE prio-142 relief pass (0 % 5 == 0), and no other system perturbs
    /// `Hunger`/`stockpile`, so the per-event arithmetic is fully isolated.
    fn consume_only_engine() -> SimEngine {
        let mut e = SimEngine::new(W, H, MaterialRegistry::new());
        e.register_system(Box::new(StockpileConsumeSystem::new()));
        e
    }

    /// Famine engine: HungerDecay (130, grows hunger) + Starvation (139, kills at
    /// saturation) + StockpileConsume (142, relief). NO movement/decision/thirst
    /// systems → positions stay at home, `AgentState` stays `Idle` (not eating
    /// ground), and `Thirst` stays at its seeded 0.0 (water held abundant). This
    /// isolates HUNGER as the only lethal variable for the A6 A/B control.
    fn famine_engine() -> SimEngine {
        let mut e = SimEngine::new(W, H, MaterialRegistry::new());
        e.register_system(Box::new(HungerDecaySystem::new()));
        e.register_system(Box::new(StarvationSystem::new()));
        e.register_system(Box::new(StockpileConsumeSystem::new()));
        e
    }

    /// Spawn an agent at `(x,y)` with a fixed `Hunger.value` (growth 0) and an
    /// explicit `AgentState`. Returns the stable `AgentId`.
    fn spawn_hungry(e: &mut SimEngine, x: u32, y: u32, hunger: f32, state: AgentState) -> AgentId {
        let ent = e.spawn_agent(x, y);
        let aid = e.world.get::<&Agent>(ent).expect("spawned agent").id;
        e.world
            .insert(ent, (Hunger::new(hunger, 0.0), state))
            .expect("seed hunger/state");
        aid
    }

    /// Spawn a famine member: full hp, growing hunger (`growth`), thirst pinned at 0
    /// (no decay system runs), `AgentState::Idle`.
    fn spawn_famine_member(e: &mut SimEngine, x: u32, y: u32, growth: f32) -> AgentId {
        let ent = e.spawn_agent(x, y);
        let aid = e.world.get::<&Agent>(ent).expect("spawned agent").id;
        e.world
            .insert(
                ent,
                (
                    AgentState::Idle,
                    BodyHealth {
                        hp: DEFAULT_MAX_HP,
                        max_hp: DEFAULT_MAX_HP,
                    },
                    Hunger::new(0.0, growth),
                    Thirst::new(0.0, 0.0),
                ),
            )
            .expect("seed famine member");
        aid
    }

    /// Insert a settlement with fixed `formation_tile`, member roster, and a
    /// pre-stocked Food reserve (`food == 0` ⇒ empty reserve, no key created).
    fn insert_settlement(
        e: &mut SimEngine,
        id: SettlementId,
        ft: (u32, u32),
        members: &[AgentId],
        food: u32,
    ) {
        let mut s = Settlement::new_with_id(id, 0);
        s.formation_tile = ft;
        for &m in members {
            s.member_agents.insert(m);
        }
        s.population_stats.current = s.member_agents.len() as u32;
        if food > 0 {
            s.store(ResourceKind::Food, food);
        }
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

    /// Current `Hunger.value` of the member whose id is `aid` (None if despawned).
    fn hunger_of(e: &SimEngine, aid: AgentId) -> Option<f32> {
        for (_, (a, h)) in e.world.query::<(&Agent, &Hunger)>().iter() {
            if a.id == aid {
                return Some(h.value);
            }
        }
        None
    }

    /// Count `AgentDied` events whose `reason` matches `want`, across the causal log.
    fn count_deaths_by_reason(e: &SimEngine, want: DeathReason) -> usize {
        let mut n = 0;
        for (_t, log) in e.resources.causal_log.iter() {
            for ev in log.iter() {
                if let CausalEvent::AgentDied { reason, .. } = ev {
                    if *reason == want {
                        n += 1;
                    }
                }
            }
        }
        n
    }

    // ── production-scene helpers (mirror the 2-3a deposit harness) ────────────────

    /// Production-like scene: full default runtime (now incl. `StockpileConsume`) +
    /// 64 bootstrap agents + 3 startup buildings → forms settlements and drives the
    /// gather/deposit/eat loop.
    fn make_production_engine() -> SimEngine {
        let mut e = SimEngine::new(W, H, MaterialRegistry::new());
        register_default_runtime_systems(&mut e);
        seed_production_scene(&mut e);
        e
    }

    /// Production scene WITHOUT `StockpileConsumeSystem` — the relief-disabled
    /// control for A7 leg (i). Registers every default register-fn except the new
    /// consume registration.
    fn make_no_relief_engine() -> SimEngine {
        let mut e = SimEngine::new(W, H, MaterialRegistry::new());
        register_phase2_systems(&mut e);
        register_agent_systems(&mut e);
        register_decision_systems(&mut e);
        register_needs_systems(&mut e);
        register_construction_systems(&mut e);
        register_social_systems(&mut e);
        register_memory_systems(&mut e);
        register_combat_systems(&mut e);
        register_settlement_systems(&mut e);
        register_survival_systems(&mut e);
        register_resource_systems(&mut e);
        register_stockpile_systems(&mut e); // 2-3a deposit (NOT 2-4 consume)
        seed_production_scene(&mut e);
        e
    }

    fn seed_production_scene(e: &mut SimEngine) {
        bootstrap_spawn_agents(e);
        for (x, y) in [(32i32, 32i32), (24, 32), (40, 32)] {
            let ok = enqueue_building_placed(&mut e.resources, x, y, 8);
            assert!(ok, "startup building ({x},{y}) must enqueue");
        }
        let mut bss = BuildingStampSystem::new();
        bss.tick(&mut e.world, &mut e.resources);
    }

    fn total_stock_food(e: &SimEngine) -> u64 {
        e.resources
            .settlements
            .values()
            .map(|s| s.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0) as u64)
            .sum()
    }

    fn live_count(e: &SimEngine) -> usize {
        e.world.query::<&Agent>().iter().count()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // A1 — controlled relief: reserve drains by EXACTLY one meal AND hunger is
    // relieved by [25,35]. Single-firing window (one e.tick() from current_tick 0).
    // ═══════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_stockpile_consume_a1_controlled_relief_bounded() {
        let mut e = consume_only_engine();
        let ft = (30u32, 30u32);
        let h0: f32 = 60.0; // >= THRESH (50)
        let stock0: u32 = 10;
        let m = spawn_hungry(&mut e, 30, 30, h0, AgentState::Idle);
        insert_settlement(&mut e, 1, ft, &[m], stock0);

        e.tick(); // current_tick 0 → exactly ONE prio-142 relief pass

        let stock1 = stock_food(&e, 1);
        let h1 = hunger_of(&e, m).expect("member alive");
        let stock_drop = stock0 - stock1;
        let hunger_drop = h0 - h1;
        println!(
        "[consume A1] stock {stock0}->{stock1} (Δ={stock_drop}), hunger {h0}->{h1} (Δ={hunger_drop})"
    );
        // Type A — mass balance: exactly one meal withdrawn (NOT >= 1).
        assert_eq!(
            stock_drop, PER_MEAL,
            "A1: stockpile must drop by EXACTLY one meal per firing"
        );
        // Type A — relief delivered, centred on HUNGER_CONSUME_AMOUNT (30) ± slack.
        assert!(
            (25.0..=35.0).contains(&hunger_drop),
            "A1: hunger relief must be in [25,35]; got {hunger_drop}"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // A2 — no double-dip: a ground-eating member (AgentState targets Food) is
    // skipped; stockpile Δ == 0 over a multi-firing window.
    // ═══════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_stockpile_consume_a2_no_double_dip() {
        for state in [
            AgentState::Seeking {
                target: TargetKind::Food,
            },
            AgentState::Consuming {
                target: TargetKind::Food,
            },
        ] {
            let mut e = consume_only_engine();
            let ft = (30u32, 30u32);
            let stock0: u32 = 10;
            let m = spawn_hungry(&mut e, 30, 30, 60.0, state);
            insert_settlement(&mut e, 1, ft, &[m], stock0);

            for _ in 0..10 {
                e.tick(); // passes fire at current_tick 0 & 5 → ≥2 chances
            }

            let got = stock_food(&e, 1);
            println!("[consume A2] state={state:?} stockpile {stock0}->{got}");
            // Type A — hard gate: ground eater never drains the reserve.
            assert_eq!(
                got, stock0,
                "A2: ground-eating member must not drain the reserve"
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // A3 — proximity / threshold gating: far member AND below-threshold member are
    // both no-ops. Stockpile Δ == 0.
    // ═══════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_stockpile_consume_a3_gating_noop() {
        let mut e = consume_only_engine();
        let ft = (30u32, 30u32);
        let stock0: u32 = 10;
        // (a) far member: Chebyshev 10 > RADIUS 5, hunger above threshold, Idle.
        let far = spawn_hungry(&mut e, 40, 30, 60.0, AgentState::Idle);
        // (b) at-home member but hunger strictly below threshold.
        let sated = spawn_hungry(&mut e, 30, 30, THRESH - 1.0, AgentState::Idle);
        insert_settlement(&mut e, 1, ft, &[far, sated], stock0);

        for _ in 0..10 {
            e.tick();
        }

        let got = stock_food(&e, 1);
        println!(
            "[consume A3] stockpile {stock0}->{got} far_h={:?} sated_h={:?}",
            hunger_of(&e, far),
            hunger_of(&e, sated)
        );
        // Type A — both predicates hard-gate relief.
        assert_eq!(
            got, stock0,
            "A3: far/below-threshold members must not drain the reserve"
        );
        assert_eq!(
            hunger_of(&e, far),
            Some(60.0),
            "A3: far member hunger unchanged"
        );
        assert_eq!(
            hunger_of(&e, sated),
            Some(THRESH - 1.0),
            "A3: sated member hunger unchanged"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // A4 — empty reserve: with nothing to withdraw (got==0), hunger is NOT relieved
    // (no relieve-without-consume). Stockpile stays 0.
    // ═══════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_stockpile_consume_a4_empty_reserve_no_phantom_relief() {
        let mut e = consume_only_engine();
        let ft = (30u32, 30u32);
        let h0: f32 = 60.0;
        let m = spawn_hungry(&mut e, 30, 30, h0, AgentState::Idle); // at home, hungry, Idle
        insert_settlement(&mut e, 1, ft, &[m], 0); // EMPTY reserve

        for _ in 0..10 {
            e.tick(); // ≥2 relief passes attempt to fire
        }

        let h1 = hunger_of(&e, m).expect("member alive");
        println!(
            "[consume A4] empty reserve: hunger {h0}->{h1} stock={}",
            stock_food(&e, 1)
        );
        // Type A — got==0 ⇒ no relief (consume_only engine: only relief writes Hunger).
        assert_eq!(
            h1, h0,
            "A4: empty reserve must NOT relieve hunger (got>0 guard)"
        );
        assert_eq!(stock_food(&e, 1), 0, "A4: stockpile remains 0");
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // A5 — at-threshold boundary: hunger EXACTLY == THRESH fires relief (inclusive
    // `>=`). Single-firing window.
    // ═══════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_stockpile_consume_a5_at_threshold_fires() {
        let mut e = consume_only_engine();
        let ft = (30u32, 30u32);
        let stock0: u32 = 10;
        let m = spawn_hungry(&mut e, 30, 30, THRESH, AgentState::Idle); // exactly at threshold
        insert_settlement(&mut e, 1, ft, &[m], stock0);

        e.tick(); // single firing

        let stock1 = stock_food(&e, 1);
        let h1 = hunger_of(&e, m).expect("member alive");
        let hunger_drop = THRESH - h1;
        println!("[consume A5] at-threshold: stock {stock0}->{stock1} hunger {THRESH}->{h1}");
        // Type A — inclusive comparison: relief fires AT the boundary.
        assert_eq!(
            stock0 - stock1,
            PER_MEAL,
            "A5: stockpile must drop by one meal at the boundary"
        );
        assert!(
            hunger_drop >= 25.0,
            "A5: hunger must be relieved (>=25) at the exact threshold"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // A6 (PRIMARY) — famine A/B: a full reserve prevents starvation deaths that the
    // empty-reserve control suffers. Water held abundant ⇒ Dehydration == 0 in both.
    // ═══════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_stockpile_famine_relief() {
        const N: usize = 6;
        const GROWTH: f32 = 0.5; // saturates from 0 at ~tick 200; lethal under StarvationSystem
        const SPAN: u64 = 3000;
        const STOCK: u32 = 1_000_000; // ≫ FOOD_PER_MEAL × N × firings-in-span
        let ft = (30u32, 30u32);
        // Cohort offsets — all Chebyshev <= 1 of `ft` (held within proximity, no
        // movement system runs so they never wander out).
        let offsets = [(0i32, 0i32), (1, 0), (0, 1), (1, 1), (-1, 0), (0, -1)];

        // Run one famine condition; return (starvation_deaths, dehydration_deaths).
        fn run_famine(
            ft: (u32, u32),
            offsets: &[(i32, i32)],
            food: u32,
            span: u64,
        ) -> (usize, usize) {
            let mut e = famine_engine();
            let mut members = Vec::new();
            for &(dx, dy) in offsets {
                let x = (ft.0 as i32 + dx) as u32;
                let y = (ft.1 as i32 + dy) as u32;
                members.push(spawn_famine_member(&mut e, x, y, GROWTH));
            }
            insert_settlement(&mut e, 1, ft, &members, food);
            for _ in 0..span {
                e.tick();
            }
            (
                count_deaths_by_reason(&e, DeathReason::Starvation),
                count_deaths_by_reason(&e, DeathReason::Dehydration),
            )
        }

        let (stocked_starv, stocked_dehy) = run_famine(ft, &offsets, STOCK, SPAN);
        let (control_starv, control_dehy) = run_famine(ft, &offsets, 0, SPAN);
        let half = N.div_ceil(2);
        println!(
        "[consume A6] STOCKED starv={stocked_starv} dehy={stocked_dehy} | CONTROL starv={control_starv} dehy={control_dehy} | ceil(N/2)={half}"
    );
        // Water confirmed constant: no dehydration deaths in EITHER arm.
        assert_eq!(
            stocked_dehy, 0,
            "A6: STOCKED must have zero Dehydration deaths (water held abundant)"
        );
        assert_eq!(
            control_dehy, 0,
            "A6: CONTROL must have zero Dehydration deaths (water held abundant)"
        );
        // Non-vacuity: famine IS lethal without the reserve.
        assert!(
            control_starv >= half,
            "A6: CONTROL must lose >= ceil(N/2)={half} to Starvation; got {control_starv}"
        );
        // Load-bearing behavioural proof: a full reserve keeps the cohort alive.
        assert_eq!(
            stocked_starv, 0,
            "A6: STOCKED must have zero Starvation deaths (the reserve feeds the famine cohort)"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // A7 — plenty-state non-interference: relief does not net-drain the reserve, no
    // destabilisation. (i) with-relief stockpile >= no-relief control − tolerance;
    // (ii) settlements >= 1; (iii) live pop in [20,400], not frozen.
    // ═══════════════════════════════════════════════════════════════════════════
    #[test]
    fn harness_stockpile_consume_a7_plenty_non_interference() {
        const SPAN: u64 = 3000;
        // Observed-baseline tolerance for rare plenty-state firings (Type C). The
        // GREEN net diff (control − with-relief) is ~72 on ~1100; this 300 margin
        // keeps teeth (rejects a gross net-drain) while absorbing deterministic
        // trajectory divergence between the with/without-relief system sets.
        const RELIEF_TOLERANCE: i64 = 300;

        // With-relief run + population-freeze sampling.
        let mut e = make_production_engine();
        let mut pop_samples: BTreeSet<usize> = BTreeSet::new();
        for t in 1..=SPAN {
            e.tick();
            if t.is_multiple_of(500) {
                pop_samples.insert(live_count(&e));
            }
        }
        let with_relief = total_stock_food(&e) as i64;
        let settlement_count = e.resources.settlements.len();
        let final_pop = live_count(&e);

        // Relief-disabled control on the identical seeded scene.
        let mut c = make_no_relief_engine();
        for _ in 0..SPAN {
            c.tick();
        }
        let control = total_stock_food(&c) as i64;

        println!(
        "[consume A7] with_relief={with_relief} control={control} (tol={RELIEF_TOLERANCE}) settlements={settlement_count} final_pop={final_pop} pop_samples={pop_samples:?}"
    );
        // (i) reserve not materially net-drained in plenty.
        assert!(
            with_relief >= 100,
            "A7(i): with-relief stockpile Food must be a non-trivial >= 100"
        );
        assert!(
        with_relief >= control - RELIEF_TOLERANCE,
        "A7(i): relief must not net-drain the reserve in plenty (with={with_relief} control={control} tol={RELIEF_TOLERANCE})"
    );
        // (ii) settlements persist.
        assert!(
            settlement_count >= 1,
            "A7(ii): at least one settlement must exist"
        );
        // (iii) population in the structural sanity band, not frozen.
        assert!(
            (20..=400).contains(&final_pop),
            "A7(iii): final live pop must be in [20,400]; got {final_pop}"
        );
        assert!(
        pop_samples.len() >= 2,
        "A7(iii): population must not be pinned to a single value across the span (no freeze); samples={pop_samples:?}"
    );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // A8 — determinism: lockstep ×2 byte-identical (stockpile, live count, per-
    // settlement member sets).
    // ═══════════════════════════════════════════════════════════════════════════
    type ConsumeDigest = (u64, usize, Vec<Vec<AgentId>>);

    #[test]
    fn harness_stockpile_consume_a8_determinism() {
        fn digest(ticks: u64) -> ConsumeDigest {
            let mut e = make_production_engine();
            for _ in 0..ticks {
                e.tick();
            }
            let total = total_stock_food(&e);
            let live = live_count(&e);
            let mut sets: Vec<Vec<AgentId>> = e
                .resources
                .settlements
                .values()
                .map(|s| {
                    let mut m: Vec<AgentId> = s.member_agents.iter().copied().collect();
                    m.sort_unstable();
                    m
                })
                .collect();
            sets.sort_unstable();
            (total, live, sets)
        }

        let d1 = digest(2000);
        let d2 = digest(2000);
        println!(
            "[consume A8] total1={} live1={} | total2={} live2={}",
            d1.0, d1.1, d2.0, d2.1
        );
        // Type A — bit-exact replay.
        assert_eq!(
            d1, d2,
            "A8: production digests must be byte-identical across runs"
        );
    }
}

// From `harness_settlement_food_stock.rs`.
mod food_stock {
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
        let mut v: Vec<u64> = e
            .world
            .query::<&Agent>()
            .iter()
            .map(|(_, a)| a.id)
            .collect();
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
        assert!(
            ids.len() >= 3,
            "A4: need >= 3 distinct agents for 3 settlements"
        );

        // Distinct Food per settlement; insert in NON-ascending id order so the
        // collector's sort is genuinely exercised.
        let source: HashMap<SettlementId, u32> = [(3u32, 7u32), (1u32, 42u32), (2u32, 99u32)]
            .into_iter()
            .collect();
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
        println!(
            "[food-stock A4] {} rows sorted, length-parity, per-row food_stock correct ✓",
            rows.len()
        );
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
        assert!(
            !ids.contains(&bogus),
            "A6: bogus member id must not collide with a real agent"
        );
        e.resources
            .settlements
            .insert(2, make_settlement(2, (20, 20), &[bogus], Some(99)));

        let rows = collect_settlement_snapshot(&e.world, &e.resources.settlements);

        let has_valid = rows.iter().any(|r| r.settlement_id == 1);
        let has_skipped = rows.iter().filter(|r| r.settlement_id == 2).count();
        assert!(
            has_valid,
            "A6: the valid 1-member settlement must be present"
        );
        assert_eq!(
            has_skipped, 0,
            "A6: a 0-resolvable-member settlement must be skipped even with Food stocked; \
         found {has_skipped} row(s) for it"
        );
        println!("[food-stock A6] 0-member settlement with Food skipped; valid present ✓");
    }
}
