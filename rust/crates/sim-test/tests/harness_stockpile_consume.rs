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
                BodyHealth { hp: DEFAULT_MAX_HP, max_hp: DEFAULT_MAX_HP },
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
        AgentState::Seeking { target: TargetKind::Food },
        AgentState::Consuming { target: TargetKind::Food },
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
        assert_eq!(got, stock0, "A2: ground-eating member must not drain the reserve");
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
    assert_eq!(got, stock0, "A3: far/below-threshold members must not drain the reserve");
    assert_eq!(hunger_of(&e, far), Some(60.0), "A3: far member hunger unchanged");
    assert_eq!(hunger_of(&e, sated), Some(THRESH - 1.0), "A3: sated member hunger unchanged");
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
    println!("[consume A4] empty reserve: hunger {h0}->{h1} stock={}", stock_food(&e, 1));
    // Type A — got==0 ⇒ no relief (consume_only engine: only relief writes Hunger).
    assert_eq!(h1, h0, "A4: empty reserve must NOT relieve hunger (got>0 guard)");
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
    assert_eq!(stock0 - stock1, PER_MEAL, "A5: stockpile must drop by one meal at the boundary");
    assert!(hunger_drop >= 25.0, "A5: hunger must be relieved (>=25) at the exact threshold");
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
    fn run_famine(ft: (u32, u32), offsets: &[(i32, i32)], food: u32, span: u64) -> (usize, usize) {
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
    assert_eq!(stocked_dehy, 0, "A6: STOCKED must have zero Dehydration deaths (water held abundant)");
    assert_eq!(control_dehy, 0, "A6: CONTROL must have zero Dehydration deaths (water held abundant)");
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
    assert!(with_relief >= 100, "A7(i): with-relief stockpile Food must be a non-trivial >= 100");
    assert!(
        with_relief >= control - RELIEF_TOLERANCE,
        "A7(i): relief must not net-drain the reserve in plenty (with={with_relief} control={control} tol={RELIEF_TOLERANCE})"
    );
    // (ii) settlements persist.
    assert!(settlement_count >= 1, "A7(ii): at least one settlement must exist");
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
    println!("[consume A8] total1={} live1={} | total2={} live2={}", d1.0, d1.1, d2.0, d2.1);
    // Type A — bit-exact replay.
    assert_eq!(d1, d2, "A8: production digests must be byte-identical across runs");
}
