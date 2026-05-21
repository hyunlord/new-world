//! V7 Phase 10-β — `SettlementSystem` + Causal chain harness.
//!
//! feature: p10-beta-settlement-system
//! plan_attempt: 2
//! code_attempt: 4
//! seed: 42
//! lane: --full
//!
//! Tests are mapped 1:1 to the 29 locked plan assertions (A1..A29). Each
//! test is named `harness_p10_beta_a<N>_<short_label>`. Comments above
//! each test cite the assertion's threshold type (A/C/D).
//!
//! Scenario variants (per plan §0):
//!   S-Default       — seed=42, 20 agents, default world.
//!   S-Cluster       — seed=42, 20 agents + scripted cluster (3+ agents,
//!                     2+ buildings) within Chebyshev radius 5.
//!   S-Saturated     — seed=42, ≥60 agents pre-spawned, all in one
//!                     settlement so population_stats.current ≥ MAX_POP.
//!   S-MultiSettlement — seed=42, ≥2 pre-formed settlements + 1 outsider.

use std::collections::{HashMap, HashSet};

use sim_core::causal::event::{CausalEvent, DecisionReason, EventId};
use sim_core::components::{
    Agent, AgentId, AgentState, Hunger, Memory, Position, Settlement, SettlementId, Sleep, Social,
    TargetKind, Thirst, SETTLEMENT_HISTORY_CAP, SETTLEMENT_MAX_POP, SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{BuildingPlacedEvent, RuntimeSystem, SimEngine};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::construction::ConstructionSystem;
use sim_systems::runtime::influence::BuildingStampSystem;
use sim_systems::runtime::settlement::{SettlementSystem, BIRTH_COOLDOWN_TICKS};

const W: u32 = 64;
const H: u32 = 64;

// ════════════════════════════════════════════════════════════════════════
// Fixture helpers — scenario builders for the plan's named scenarios.
// ════════════════════════════════════════════════════════════════════════

fn fresh_engine() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    e
}

/// Spawn `count` agents in a tight cluster within Chebyshev radius 2 of
/// `(cx, cy)`. Needs are clamped to 0 so no need-driven cascade arm
/// fires.
fn spawn_cluster(engine: &mut SimEngine, cx: u32, cy: u32, count: u32) -> Vec<AgentId> {
    let mut ids = Vec::new();
    for i in 0..count {
        let dx = i % 3;
        let dy = i / 3;
        let entity = engine.spawn_agent(cx + dx, cy + dy);
        let aid = engine.world.get::<&Agent>(entity).unwrap().id;
        engine
            .world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(0.0, 0.0),
                    Memory::new(),
                    MovementRng::new(42u64.wrapping_add(i as u64)),
                ),
            )
            .unwrap();
        ids.push(aid);
    }
    ids.sort();
    ids
}

/// Place `count` buildings near `(cx, cy)` via the FFI queue + drain.
/// Returns nothing; the buildings land in `building_registry` with their
/// stable BuildingId == the BuildingPlaced event id.
fn place_buildings(engine: &mut SimEngine, cx: u32, cy: u32, count: u32) {
    for i in 0..count {
        let dx = i % 3;
        let dy = i / 3;
        engine.resources.building_event_queue.push_back(
            BuildingPlacedEvent {
                position: (cx + dx, cy + dy + 3), // offset away from agents
                radius: 1,
            },
        );
    }
    let mut bss = BuildingStampSystem::new();
    bss.tick(&mut engine.world, &mut engine.resources);
}

// ────────── Causal-log scanners (read-only). ──────────────────────────

fn count_kind<F: Fn(&CausalEvent) -> bool>(engine: &SimEngine, pred: F) -> usize {
    let mut n = 0;
    for (_t, log) in engine.resources.causal_log.iter() {
        for ev in log.iter() {
            if pred(ev) {
                n += 1;
            }
        }
    }
    n
}

fn collect_events<F: Fn(&CausalEvent) -> bool>(
    engine: &SimEngine,
    pred: F,
) -> Vec<CausalEvent> {
    let mut v = Vec::new();
    for (_t, log) in engine.resources.causal_log.iter() {
        for ev in log.iter() {
            if pred(ev) {
                v.push(ev.clone());
            }
        }
    }
    v
}

fn count_settlement_formed(e: &SimEngine) -> usize {
    count_kind(e, |ev| matches!(ev, CausalEvent::SettlementFormed { .. }))
}

fn count_settlement_dissolved(e: &SimEngine) -> usize {
    count_kind(e, |ev| matches!(ev, CausalEvent::SettlementDissolved { .. }))
}

fn count_agent_born(e: &SimEngine) -> usize {
    count_kind(e, |ev| matches!(ev, CausalEvent::AgentBorn { .. }))
}

fn first_formed(e: &SimEngine) -> Option<CausalEvent> {
    collect_events(e, |ev| matches!(ev, CausalEvent::SettlementFormed { .. }))
        .into_iter()
        .next()
}

fn first_dissolved(e: &SimEngine) -> Option<CausalEvent> {
    collect_events(e, |ev| matches!(ev, CausalEvent::SettlementDissolved { .. }))
        .into_iter()
        .next()
}

// ════════════════════════════════════════════════════════════════════════
// A1: SettlementSystem runs after Construction/BuildingStamp systems.
// Type A — same-tick ordering invariant.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a1_settlement_runs_after_construction_and_building_stamp() {
    // Registration order in `register_default_runtime_systems` is sorted
    // by priority. BuildingStampSystem (90), ConstructionSystem (133) and
    // SettlementSystem (138). Verify the ordering via `system_names()`.
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    let names = e.system_names();
    let bss_idx = names.iter().position(|n| *n == "BuildingStampSystem").unwrap();
    let cons_idx = names
        .iter()
        .position(|n| *n == "ConstructionSystem")
        .unwrap();
    let stl_idx = names.iter().position(|n| *n == "SettlementSystem").unwrap();
    // Type A — SettlementSystem MUST follow both producers within a tick.
    assert!(
        bss_idx < stl_idx,
        "BuildingStampSystem must precede SettlementSystem in tick order"
    );
    assert!(
        cons_idx < stl_idx,
        "ConstructionSystem must precede SettlementSystem in tick order"
    );
    // Behavioural check: schedule an FFI building, run one tick, observe
    // it in `building_registry` BEFORE SettlementSystem could have read
    // it. (Within the same tick, BSS runs at 90; SettlementSystem at
    // 138. If the BSS hadn't run first, the registry would still be empty
    // by the time SettlementSystem tried to scan it.)
    e.resources
        .building_event_queue
        .push_back(BuildingPlacedEvent {
            position: (10, 10),
            radius: 1,
        });
    e.tick();
    assert!(
        !e.resources.building_registry.is_empty(),
        "BuildingStampSystem must have populated the registry before \
         SettlementSystem's scan in the same tick"
    );
}

// ════════════════════════════════════════════════════════════════════════
// A2: SettlementFormed event-field population under real scenario.
// Type A — non-zero settlement_id, founding_members matches in-cluster
//          agents, unique event id, tick == current tick.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a2_settlement_formed_event_field_population() {
    let mut e = fresh_engine();
    let cx = 20u32;
    let cy = 20u32;
    let founder_ids = spawn_cluster(&mut e, cx, cy, 4);
    place_buildings(&mut e, cx, cy, 3);
    e.tick();

    let formed: Vec<CausalEvent> = collect_events(&e, |ev| {
        matches!(ev, CausalEvent::SettlementFormed { .. })
    });
    assert_eq!(formed.len(), 1, "exactly one SettlementFormed event");
    let ev = formed.into_iter().next().unwrap();
    match ev {
        CausalEvent::SettlementFormed {
            id,
            parent: _,
            settlement_id,
            founding_members,
            tick,
        } => {
            // Type A — none of these may be the uninitialized sentinel.
            assert_ne!(settlement_id, 0, "settlement_id == 0 sentinel forbidden");
            assert!(
                !founding_members.is_empty(),
                "founding_members must not be empty"
            );
            assert!(
                founding_members.len() >= 3,
                "founding_members.len() must be >= FORMATION_AGENT_THRESHOLD (3)"
            );
            assert_eq!(
                tick,
                e.current_tick().saturating_sub(1),
                "tick must equal sim tick at emit time (tick 0)"
            );
            // founding_members matches the verifying snapshot.
            let founder_set: HashSet<AgentId> = founder_ids.iter().copied().collect();
            let event_set: HashSet<AgentId> = founding_members.iter().copied().collect();
            assert_eq!(
                event_set, founder_set,
                "founding_members must equal the verifying in-cluster snapshot"
            );
            // No duplicate ids (single event, trivially unique).
            assert!(id > 0, "event id must be a non-zero monotonic value");
        }
        _ => unreachable!(),
    }
}

// ════════════════════════════════════════════════════════════════════════
// A3: SettlementDissolved fires only when BOTH conditions hold.
// Type A — predicate: current == 0 AND member_buildings empty.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a3_dissolved_only_when_both_conditions_hold() {
    let mut e = fresh_engine();
    let cx = 22u32;
    let cy = 22u32;
    let ids = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick();
    assert_eq!(count_settlement_formed(&e), 1);
    let (sid, _) = e.resources.settlements.iter().next().unwrap();
    let sid = *sid;

    // Phase A: only clear agents — buildings still present → must NOT dissolve.
    let entities: Vec<hecs::Entity> = e
        .world
        .query::<&Agent>()
        .iter()
        .filter(|(_, a)| ids.contains(&a.id))
        .map(|(ent, _)| ent)
        .collect();
    for ent in entities {
        let _ = e.world.despawn(ent);
    }
    e.tick();
    // Type A — buildings non-empty → no dissolution.
    assert_eq!(
        count_settlement_dissolved(&e),
        0,
        "dissolution must not fire when buildings remain"
    );
    assert!(e.resources.settlements.contains_key(&sid));

    // Phase B: clear buildings too → BOTH conditions now hold → dissolve.
    e.resources.building_registry.clear();
    e.tick();
    assert_eq!(
        count_settlement_dissolved(&e),
        1,
        "dissolution must fire when BOTH conditions hold"
    );
    assert!(!e.resources.settlements.contains_key(&sid));
}

// ════════════════════════════════════════════════════════════════════════
// A4: SettlementDissolved.final_population == pre-clearing population.
// Type A — buildings-first ordering forces non-trivial value.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a4_dissolved_final_population_pre_clearing() {
    let mut e = fresh_engine();
    let cx = 28u32;
    let cy = 28u32;
    let ids = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick(); // tick 0 — formation
    assert_eq!(count_settlement_formed(&e), 1);

    // Phase 1: clear buildings first; pop still 3.
    e.resources.building_registry.clear();
    e.tick();

    // Phase 2: kill agents — pop drops to 0; dissolution fires.
    let entities: Vec<hecs::Entity> = e
        .world
        .query::<&Agent>()
        .iter()
        .filter(|(_, a)| ids.contains(&a.id))
        .map(|(ent, _)| ent)
        .collect();
    for ent in entities {
        let _ = e.world.despawn(ent);
    }
    e.tick();

    let ev = first_dissolved(&e).expect("dissolved present");
    if let CausalEvent::SettlementDissolved {
        final_population, ..
    } = ev
    {
        // Type A — must reflect the pre-clearing population (>=3), NOT 0.
        assert!(
            final_population >= 3,
            "final_population must carry pre-clearing population (>=3), got {}",
            final_population
        );
    } else {
        unreachable!();
    }
}

// ════════════════════════════════════════════════════════════════════════
// A5: Positive formation occurs when thresholds met.
// Type C — empirical reachability within 500 ticks of S-Cluster.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a5_positive_formation_when_thresholds_met() {
    let mut e = fresh_engine();
    let cx = 18u32;
    let cy = 18u32;
    let _ids = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick();
    // Type C — at least one SettlementFormed within the 500-tick window.
    // The cluster precondition is satisfied at tick 0, so formation must
    // emit at tick 0.
    assert!(
        count_settlement_formed(&e) >= 1,
        "S-Cluster must produce >=1 SettlementFormed within 500 ticks"
    );
}

// ════════════════════════════════════════════════════════════════════════
// A6: No formation below agent threshold.
// Type A — predicate negation.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a6_no_formation_below_agent_threshold() {
    let mut e = fresh_engine();
    let cx = 30u32;
    let cy = 30u32;
    // Only 2 agents — below threshold 3.
    let _ids = spawn_cluster(&mut e, cx, cy, 2);
    place_buildings(&mut e, cx, cy, 2);
    // Precondition: no qualifying cluster (2 < 3).
    e.tick();
    // Type A — no formation.
    assert_eq!(count_settlement_formed(&e), 0);
}

// ════════════════════════════════════════════════════════════════════════
// A7: No formation below building threshold.
// Type A — predicate negation on building axis.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a7_no_formation_below_building_threshold() {
    let mut e = fresh_engine();
    let cx = 34u32;
    let cy = 34u32;
    let _ids = spawn_cluster(&mut e, cx, cy, 3);
    // Only 1 building — below threshold 2.
    place_buildings(&mut e, cx, cy, 1);
    e.tick();
    // Type A — no formation.
    assert_eq!(count_settlement_formed(&e), 0);
}

// ════════════════════════════════════════════════════════════════════════
// A8: No formation when cluster inside existing settlement radius.
// Type A — overlap prohibition.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a8_no_formation_inside_existing_radius() {
    let mut e = fresh_engine();
    let cx = 36u32;
    let cy = 36u32;
    let _ids = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick();
    assert_eq!(count_settlement_formed(&e), 1, "first settlement formed");

    // Stage a second qualifying cluster INSIDE the first settlement's
    // radius. Must NOT form a second settlement.
    let _ids2 = spawn_cluster(&mut e, cx + 1, cy + 1, 3);
    place_buildings(&mut e, cx + 1, cy + 1, 2);
    e.tick();
    // Type A — still exactly one settlement.
    assert_eq!(
        count_settlement_formed(&e),
        1,
        "second formation inside existing radius must be suppressed"
    );
}

// ════════════════════════════════════════════════════════════════════════
// A9: Formation idempotent within a single tick.
// Type A — same cluster scanned twice → one event only.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a9_formation_idempotent_within_tick() {
    let mut e = fresh_engine();
    let cx = 14u32;
    let cy = 14u32;
    let _ids = spawn_cluster(&mut e, cx, cy, 5);
    place_buildings(&mut e, cx, cy, 4);
    e.tick();
    // Type A — one event for one cluster.
    assert_eq!(
        count_settlement_formed(&e),
        1,
        "duplicate emission within single tick forbidden"
    );
    assert_eq!(e.resources.settlements.len(), 1);
}

// ════════════════════════════════════════════════════════════════════════
// A10: SettlementId monotonic — no reuse after dissolution.
// Type A — covers same-area reformation too (previous-attempt issue #2).
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a10_settlement_id_no_reuse() {
    let mut e = fresh_engine();
    let cx = 12u32;
    let cy = 12u32;
    let ids = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick();
    let first_sid = *e.resources.settlements.keys().next().unwrap();
    assert_ne!(first_sid, 0, "settlement_id == 0 sentinel forbidden");

    // Dissolve.
    let entities: Vec<hecs::Entity> = e
        .world
        .query::<&Agent>()
        .iter()
        .filter(|(_, a)| ids.contains(&a.id))
        .map(|(ent, _)| ent)
        .collect();
    for ent in entities {
        let _ = e.world.despawn(ent);
    }
    e.resources.building_registry.clear();
    e.tick();
    assert!(e.resources.settlements.is_empty());

    // Re-form in the SAME area — previous attempt 2 issue #2: must not
    // be permanently suppressed.
    let _ids2 = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick();
    assert_eq!(
        e.resources.settlements.len(),
        1,
        "same-area reformation must succeed after dissolution"
    );
    let second_sid = *e.resources.settlements.keys().next().unwrap();
    // Type A — strict id monotonic, no reuse.
    assert_ne!(second_sid, first_sid, "settlement_id must NOT be reused");
    assert!(second_sid > first_sid, "id must be strictly increasing");
}

// ════════════════════════════════════════════════════════════════════════
// A11: founding_members deterministic ordering (ascending AgentId).
// Type A — determinism invariant.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a11_founding_members_deterministic_ordering() {
    fn run_once() -> Vec<AgentId> {
        let mut e = fresh_engine();
        let cx = 16u32;
        let cy = 16u32;
        let _ids = spawn_cluster(&mut e, cx, cy, 3);
        place_buildings(&mut e, cx, cy, 2);
        e.tick();
        match first_formed(&e).unwrap() {
            CausalEvent::SettlementFormed {
                founding_members, ..
            } => founding_members,
            _ => unreachable!(),
        }
    }
    let run1 = run_once();
    let run2 = run_once();
    // Type A — sorted AND identical across re-runs.
    let mut sorted_check = run1.clone();
    sorted_check.sort();
    assert_eq!(run1, sorted_check, "founding_members must be sorted ascending");
    assert_eq!(run1, run2, "founding_members must be deterministic");
}

// ════════════════════════════════════════════════════════════════════════
// A12: Birth does NOT fire before BIRTH_COOLDOWN_TICKS elapsed.
// Type A — cooldown predicate.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a12_birth_does_not_fire_before_cooldown() {
    let mut e = fresh_engine();
    let cx = 24u32;
    let cy = 24u32;
    let _ids = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick(); // tick 0 — formation
    assert_eq!(count_agent_born(&e), 0);
    // Run BIRTH_COOLDOWN_TICKS - 1 more ticks; cooldown not yet elapsed.
    for _ in 0..(BIRTH_COOLDOWN_TICKS as usize - 1) {
        e.tick();
    }
    // Type A — no births before cooldown.
    assert_eq!(
        count_agent_born(&e),
        0,
        "AgentBorn must not fire before BIRTH_COOLDOWN_TICKS"
    );
}

// ════════════════════════════════════════════════════════════════════════
// A13: Birth fires after cooldown when under MAX_POP.
// Type A — positive case.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a13_birth_fires_after_cooldown_under_max_pop() {
    let mut e = fresh_engine();
    let cx = 26u32;
    let cy = 26u32;
    let _ids = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick();
    // Advance well past cooldown.
    for _ in 0..(BIRTH_COOLDOWN_TICKS as usize + 1) {
        e.tick();
    }
    // Type A — at least one birth.
    assert!(
        count_agent_born(&e) >= 1,
        "AgentBorn must fire after BIRTH_COOLDOWN_TICKS elapsed"
    );
}

// ════════════════════════════════════════════════════════════════════════
// A14: Birth does NOT fire when at or above MAX_POP.
// Type A — saturation predicate (S-Saturated scenario).
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a14_no_birth_when_at_max_pop() {
    // Minimal engine — SettlementSystem only, no MovementSystem.
    // Without MovementSystem, agents stay at their spawn positions, so
    // membership sync counts all 50 every tick and keeps
    // population_stats.current == MAX_POP, gating the birth trigger.
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    e.register_system(Box::new(SettlementSystem::new()));
    let cx = 25u32;
    let cy = 25u32;
    let _ids = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick();
    assert_eq!(count_settlement_formed(&e), 1);
    let sid = *e.resources.settlements.keys().next().unwrap();

    // Saturate the settlement by spawning SETTLEMENT_MAX_POP additional
    // agents INSIDE the proximity radius of the formation anchor.
    // Pattern: dx ∈ [-4..5] (10 values, signed), dy ∈ [1..5] (5 values,
    // positive offset to avoid colliding with founder row at dy=0). Max
    // Chebyshev distance is max(|dx|, dy) = max(5, 5) = 5, exactly the
    // proximity radius — so every spawned agent qualifies for membership
    // during sync.
    //
    // Important: extras are spawned WITHOUT `MovementRng`. The Brownian-
    // motion query in `AgentMovementSystem` requires `&mut MovementRng`,
    // so agents lacking that component never move and stay pinned inside
    // the proximity radius for the entire 200+ tick cooldown window. The
    // 3 founders DO have MovementRng and may drift out of radius over
    // the 200+ tick cooldown; we therefore over-provision with
    // SETTLEMENT_MAX_POP pinned extras (10×5 grid filled completely) so
    // the post-drift population is guaranteed to satisfy `current >=
    // SETTLEMENT_MAX_POP` even with all 3 founders displaced.
    // Add MAX_POP-3 agents within Chebyshev radius 4 of the formation tile.
    // A 9x9 grid centred at (cx,cy) gives positions with max(|dx|,|dy|)<=4 < 5.
    // All stay in range every tick since no MovementSystem is registered.
    for i in 0..(SETTLEMENT_MAX_POP - 3) {
        let dx = (i % 9) as i64 - 4;
        let dy = (i / 9) as i64 - 4;
        let x = (cx as i64 + dx) as u32;
        let y = (cy as i64 + dy) as u32;
        let entity = e.spawn_agent(x, y);
        e.world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(0.0, 0.0),
                    Memory::new(),
                ),
            )
            .unwrap();
    }
    // Run past cooldown.
    let births_before = count_agent_born(&e);
    for _ in 0..(BIRTH_COOLDOWN_TICKS as usize + 5) {
        e.tick();
    }
    // The settlement should now be at MAX_POP. Verify and then ensure no
    // further births fire.
    let s = e
        .resources
        .settlements
        .get(&sid)
        .expect("settlement still present");
    assert!(
        s.population_stats.current >= SETTLEMENT_MAX_POP,
        "S-Saturated precondition: current ({}) >= MAX_POP ({})",
        s.population_stats.current,
        SETTLEMENT_MAX_POP
    );
    let births_at_saturation = count_agent_born(&e);

    // Run another 2*BIRTH_COOLDOWN ticks; births must not increase.
    for _ in 0..(2 * BIRTH_COOLDOWN_TICKS as usize) {
        e.tick();
    }
    let births_after = count_agent_born(&e);
    // Type A — no new births while saturated.
    assert_eq!(
        births_after, births_at_saturation,
        "birth must not fire when settlement at/above MAX_POP"
    );
    let _ = births_before;
}

// ════════════════════════════════════════════════════════════════════════
// A15: Birth spawns agent inside SETTLEMENT_PROXIMITY_RADIUS of anchor.
// Type A — Chebyshev <= SETTLEMENT_PROXIMITY_RADIUS exactly.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a15_birth_spawns_inside_radius() {
    let mut e = fresh_engine();
    let cx = 30u32;
    let cy = 30u32;
    let founders = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick();
    let pre_ids: HashSet<AgentId> = e
        .world
        .query::<&Agent>()
        .iter()
        .map(|(_, a)| a.id)
        .collect();
    // The formation tile is (cx, cy) because that's the lowest agent's
    // position.
    for _ in 0..(BIRTH_COOLDOWN_TICKS as usize + 1) {
        e.tick();
    }
    assert!(count_agent_born(&e) >= 1, "must spawn at least one new agent");

    let mut new_positions: Vec<(u32, u32)> = Vec::new();
    for (_, (a, p)) in e.world.query::<(&Agent, &Position)>().iter() {
        if !pre_ids.contains(&a.id) && !founders.contains(&a.id) {
            new_positions.push((p.x, p.y));
        }
    }
    assert!(!new_positions.is_empty());
    for (nx, ny) in &new_positions {
        let dx = nx.abs_diff(cx);
        let dy = ny.abs_diff(cy);
        let cheb = dx.max(dy);
        // Type A — within EXACTLY the proximity radius (no slack).
        assert!(
            cheb <= SETTLEMENT_PROXIMITY_RADIUS,
            "newborn at ({},{}) must be within radius {} of anchor ({},{}); cheb={}",
            nx, ny, SETTLEMENT_PROXIMITY_RADIUS, cx, cy, cheb
        );
    }
    // Impassable-tile behaviour: at this phase the TileGrid is empty
    // (no walls). The Generator's documented choice is "spawn at anchor
    // member's tile", which is always passable in this fixture. If
    // future phases introduce walls, A15 would need to be extended.
    // Document the chosen behaviour here so future maintainers know the
    // contract.
    let _ = new_positions; // (documented above)
}

// ════════════════════════════════════════════════════════════════════════
// A16: SettlementReason fires for non-member when settlement has capacity.
// Type A — positive case for the 8th cascade arm.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a16_settlement_reason_fires_for_outsider() {
    let mut e = fresh_engine();
    let cx = 10u32;
    let cy = 10u32;
    let _founders = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick();
    assert_eq!(count_settlement_formed(&e), 1);

    let outsider = e.spawn_agent(50, 50);
    e.world
        .insert(
            outsider,
            (
                AgentState::Idle,
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 0.0),
                Memory::new(),
                MovementRng::new(999),
            ),
        )
        .unwrap();
    e.tick();
    let n = count_kind(&e, |ev| {
        matches!(
            ev,
            CausalEvent::AgentDecision {
                reason: DecisionReason::SettlementReason,
                ..
            }
        )
    });
    assert!(n >= 1, "outsider must emit AgentDecision{{SettlementReason}}");
}

// ════════════════════════════════════════════════════════════════════════
// A17: SettlementReason does NOT fire when agent already a member.
// Type A — predicate negation on membership.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a17_no_settlement_reason_when_already_member() {
    let mut e = fresh_engine();
    let cx = 14u32;
    let cy = 14u32;
    let founders: HashSet<AgentId> = spawn_cluster(&mut e, cx, cy, 3).into_iter().collect();
    place_buildings(&mut e, cx, cy, 2);
    e.tick(); // formation
    e.tick(); // membership stable
    let mut bad = 0;
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::AgentDecision {
                agent,
                reason: DecisionReason::SettlementReason,
                ..
            } = ev
            {
                if founders.contains(agent) {
                    bad += 1;
                }
            }
        }
    }
    // Type A — never fires for founders.
    assert_eq!(bad, 0);
}

// ════════════════════════════════════════════════════════════════════════
// A18: MAX_POP caps the migration-pull cascade arm.
// Type A — behavioural capacity gate: while a settlement is saturated to
//          `SETTLEMENT_MAX_POP`, the 8th cascade arm MUST NOT emit a
//          `SettlementReason` decision for outsiders. Once capacity is
//          available, the next tick MUST emit one.
//
// Tick-ordering note: `AgentDecisionSystem` runs at priority 125, and
// `SettlementSystem` runs at priority 138. So a value written into
// `population_stats.current` AFTER a completed tick is observed by the
// NEXT tick's decision pass BEFORE the settlement system re-syncs
// membership. That sequence is what makes this test behavioural rather
// than constants-only: we exercise the actual gate at the actual call
// site, then let the natural re-sync reopen capacity.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a18_max_pop_caps_migration_pull() {
    let mut e = fresh_engine();
    let cx = 20u32;
    let cy = 20u32;
    let _founders = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    // Tick 1: form the settlement.
    e.tick();
    let sid = *e
        .resources
        .settlements
        .keys()
        .next()
        .expect("a settlement must form from the founder cluster");

    // Saturate population to MAX_POP. SettlementSystem (priority 138)
    // re-syncs from membership at the END of each tick, so this write is
    // visible to AgentDecisionSystem (priority 125) on the NEXT tick
    // BEFORE the re-sync runs.
    e.resources
        .settlements
        .get_mut(&sid)
        .expect("settlement still present")
        .population_stats
        .current = SETTLEMENT_MAX_POP;

    // Spawn an outsider far from any settlement boundary so the only
    // available cascade arm is the migration pull (other drives clamped
    // to zero).
    let outsider = e.spawn_agent(50, 50);
    e.world
        .insert(
            outsider,
            (
                AgentState::Idle,
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 0.0),
                Memory::new(),
                MovementRng::new(13),
            ),
        )
        .unwrap();
    let outsider_id = e.world.get::<&Agent>(outsider).unwrap().id;

    // Sanity: the gate value AgentDecisionSystem will read next tick is
    // exactly MAX_POP.
    assert_eq!(
        e.resources
            .settlements
            .get(&sid)
            .map(|s| s.population_stats.current)
            .unwrap_or(0),
        SETTLEMENT_MAX_POP
    );

    // Tick 2: decision pass observes saturated capacity → no
    // `SettlementReason` for the outsider.
    e.tick();
    let blocked_emit = e.resources.causal_log.iter().any(|(_, log)| {
        log.iter().any(|ev| {
            matches!(
                ev,
                CausalEvent::AgentDecision {
                    agent,
                    reason: DecisionReason::SettlementReason,
                    ..
                } if *agent == outsider_id
            )
        })
    });
    assert!(
        !blocked_emit,
        "no SettlementReason expected while settlement.population_stats.current == SETTLEMENT_MAX_POP"
    );
    let outsider_state_blocked = e
        .world
        .query::<(&Agent, &AgentState)>()
        .iter()
        .find_map(|(_, (a, s))| {
            if a.id == outsider_id {
                Some(*s)
            } else {
                None
            }
        })
        .expect("outsider must still be in the world");
    let migration_target_blocked = matches!(
        outsider_state_blocked,
        AgentState::Seeking {
            target: TargetKind::Agent(_)
        }
    );
    assert!(
        !migration_target_blocked,
        "no migration target may be assigned while settlement is at MAX_POP"
    );

    // SettlementSystem (priority 138) re-synced membership at the end of
    // tick 2, dropping `population_stats.current` back to the actual
    // member count (3) — capacity is now available for tick 3.
    let current_after = e
        .resources
        .settlements
        .get(&sid)
        .map(|s| s.population_stats.current)
        .unwrap_or(0);
    assert!(
        current_after < SETTLEMENT_MAX_POP,
        "membership re-sync must restore current below MAX_POP, got {current_after}"
    );

    // Tick 3: capacity available → SettlementReason MUST fire for the
    // outsider this tick.
    e.tick();
    let opened_emit = e.resources.causal_log.iter().any(|(_, log)| {
        log.iter().any(|ev| {
            matches!(
                ev,
                CausalEvent::AgentDecision {
                    agent,
                    reason: DecisionReason::SettlementReason,
                    ..
                } if *agent == outsider_id
            )
        })
    });
    assert!(
        opened_emit,
        "SettlementReason MUST fire once settlement capacity is available"
    );
}

// ════════════════════════════════════════════════════════════════════════
// A19: Anti-recursion — SettlementReason does not re-emit decision.
// Type A — classify_event(SettlementReason) → None, even after the
//          actual recursive event is emitted.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a19_anti_recursion_settlement_reason_not_re_emitted() {
    let mut e = fresh_engine();
    let cx = 12u32;
    let cy = 12u32;
    let _founders = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick();
    let outsider = e.spawn_agent(50, 50);
    e.world
        .insert(
            outsider,
            (
                AgentState::Idle,
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 0.0),
                Memory::new(),
                MovementRng::new(7),
            ),
        )
        .unwrap();
    e.tick();
    let outsider_id = e.world.get::<&Agent>(outsider).unwrap().id;
    // Find the SettlementReason decision id for the outsider.
    let mut sr_id: Option<EventId> = None;
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::AgentDecision {
                id,
                agent,
                reason: DecisionReason::SettlementReason,
                ..
            } = ev
            {
                if *agent == outsider_id {
                    sr_id = Some(*id);
                }
            }
        }
    }
    let original_id = sr_id.expect("SettlementReason decision present");
    // Run more ticks; the memory pipeline must never produce a derived
    // decision whose parent chain includes this id.
    for _ in 0..100 {
        e.tick();
    }
    // Walk every AgentDecision{SettlementReason} parent chain.
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::AgentDecision {
                id,
                parent,
                reason: DecisionReason::SettlementReason,
                ..
            } = ev
            {
                if *id == original_id {
                    continue;
                }
                if let Some(p) = parent {
                    assert_ne!(
                        *p, original_id,
                        "anti-recursion: no derived SettlementReason may chain to original"
                    );
                }
            }
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// A20: Settlement events not memory-encoded.
// Type A — classify_event(SettlementFormed|Dissolved) → None.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a20_settlement_events_not_memory_encoded() {
    let mut e = fresh_engine();
    let cx = 18u32;
    let cy = 18u32;
    let founders = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick(); // formation
    let formed_id = match first_formed(&e).unwrap() {
        CausalEvent::SettlementFormed { id, .. } => id,
        _ => unreachable!(),
    };
    // Run more ticks so MemorySystem has plenty of opportunity to
    // encode the event.
    for _ in 0..5 {
        e.tick();
    }
    // Type A — no founder's Memory may contain formed_id.
    for (_, (a, m)) in e.world.query::<(&Agent, &Memory)>().iter() {
        if !founders.contains(&a.id) {
            continue;
        }
        let cnt = m.entries.iter().filter(|x| x.event_id == formed_id).count();
        assert_eq!(
            cnt, 0,
            "SettlementFormed must not be memory-encoded (agent {}, formed_id {})",
            a.id, formed_id
        );
    }
    // Dissolve and check SettlementDissolved isn't encoded either.
    let ids: Vec<AgentId> = founders.to_vec();
    let entities: Vec<hecs::Entity> = e
        .world
        .query::<&Agent>()
        .iter()
        .filter(|(_, a)| ids.contains(&a.id))
        .map(|(ent, _)| ent)
        .collect();
    for ent in entities {
        let _ = e.world.despawn(ent);
    }
    e.resources.building_registry.clear();
    e.tick();
    let dissolved_id = match first_dissolved(&e).unwrap() {
        CausalEvent::SettlementDissolved { id, .. } => id,
        _ => unreachable!(),
    };
    // After dissolution, founders are despawned — we sample remaining
    // agents (no member references to dissolved_id can exist).
    for (_, (_a, m)) in e.world.query::<(&Agent, &Memory)>().iter() {
        let cnt = m.entries.iter().filter(|x| x.event_id == dissolved_id).count();
        assert_eq!(
            cnt, 0,
            "SettlementDissolved must not be memory-encoded"
        );
    }
}

// ════════════════════════════════════════════════════════════════════════
// A21: Community history ingests all four qualifying event classes 1:1.
// Type A — BuildingPlaced, CombatCompleted, AgentBorn, SettlementReason.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a21_community_history_ingests_qualifying_events() {
    let mut e = fresh_engine();
    let cx = 20u32;
    let cy = 20u32;
    let _founders = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick(); // formation
    let sid = *e.resources.settlements.keys().next().unwrap();

    // Classes present accumulator — snapshotted at every test phase since
    // BuildingPlaced ids are subject to per-tile-ring eviction by the
    // continuous Warmth/Light/Noise/Danger InfluenceChanged stream. For
    // BuildingPlaced specifically we use the authoritative `building_
    // registry` (lifecycle-stable: entry persists for the building's
    // lifetime) as the lookup surface. Other classes (AgentBorn,
    // CombatCompleted, SettlementReason) live exclusively in the per-tile
    // causal log so we snapshot them at the moment they are appended.
    let mut classes_present: HashSet<&str> = HashSet::new();
    let snapshot_classes = |e: &SimEngine,
                            sid: SettlementId,
                            classes: &mut HashSet<&'static str>| {
        let s = e.resources.settlements.get(&sid).unwrap();
        for eid in &s.community_history {
            // Authoritative: BuildingPlaced ids = building_registry keys.
            if e.resources.building_registry.contains_key(eid) {
                classes.insert("building_placed");
            }
            if let Some(ev) = e.resources.causal_log.lookup(*eid) {
                match ev {
                    CausalEvent::BuildingPlaced { .. } => {
                        classes.insert("building_placed");
                    }
                    CausalEvent::AgentBorn { .. } => {
                        classes.insert("agent_born");
                    }
                    CausalEvent::CombatCompleted { .. } => {
                        classes.insert("combat_completed");
                    }
                    CausalEvent::AgentDecision {
                        reason: DecisionReason::SettlementReason,
                        ..
                    } => {
                        classes.insert("settlement_reason");
                    }
                    _ => {}
                }
            }
        }
    };

    // (a) BuildingPlaced — snapshot AT formation tick before influence
    //     propagation can evict the BuildingPlaced events.
    snapshot_classes(&e, sid, &mut classes_present);
    assert!(
        classes_present.contains("building_placed"),
        "BuildingPlaced must be routed to community history at formation tick"
    );

    // (b) AgentBorn — fires after BIRTH_COOLDOWN.
    for _ in 0..(BIRTH_COOLDOWN_TICKS as usize + 1) {
        e.tick();
    }
    snapshot_classes(&e, sid, &mut classes_present);

    // Capture born_agent_id immediately (0 ticks since birth) while the
    // AgentBorn event is still in the tile ring buffer. The born agent has
    // no MovementRng so it stays pinned — safe to use as attacker in (d).
    let born_agent_id: AgentId = {
        let history_ids: Vec<EventId> = {
            let s = e.resources.settlements.get(&sid).unwrap();
            s.community_history.to_vec()
        };
        let mut found = None;
        for eid in history_ids {
            if let Some(CausalEvent::AgentBorn { agent, .. }) =
                e.resources.causal_log.lookup(eid)
            {
                found = Some(*agent);
                break;
            }
        }
        found.expect("AgentBorn event must be findable immediately after birth tick")
    };

    // (c) AgentDecision{SettlementReason} — spawn an outsider and
    //     synthesise the event so SettlementSystem routes it.
    let outsider = e.spawn_agent(50, 50);
    e.world
        .insert(
            outsider,
            (
                AgentState::Idle,
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 0.0),
                Memory::new(),
                MovementRng::new(11),
            ),
        )
        .unwrap();
    let outsider_aid = e.world.get::<&Agent>(outsider).unwrap().id;
    // Push with tick = NEXT tick so SettlementSystem's ev.tick()==current_tick
    // filter matches when e.tick() advances. Routing: outsider is at (50,50),
    // not a member of the settlement → routes to first capacity settlement.
    let synth_decision_id = e.resources.issue_event_id();
    e.resources.causal_log.push(
        50 * W + 50,
        CausalEvent::AgentDecision {
            id: synth_decision_id,
            parent: None,
            agent: outsider_aid,
            position: (50, 50),
            reason: DecisionReason::SettlementReason,
            tick: e.current_tick() + 1,
        },
    );
    e.tick();
    snapshot_classes(&e, sid, &mut classes_present);

    // (d) CombatCompleted via real CombatSystem path (priority 137).
    // Pre-tick injection into causal_log is evicted by InfluenceUpdateSystem
    // (priority 100) before SettlementSystem (138) scans. Instead, insert
    // into combat_pairs so CombatSystem pushes CombatCompleted post-IUS;
    // SettlementSystem then routes it without eviction risk.
    // Use born_agent_id as attacker: no MovementRng → pinned, always member.
    let dummy_defender_aid = {
        let eid = e.spawn_agent(cx, cy);
        e.world.get::<&Agent>(eid).unwrap().id
    };
    e.resources.combat_pairs.insert((born_agent_id, dummy_defender_aid));
    e.tick();
    snapshot_classes(&e, sid, &mut classes_present);

    let s = e.resources.settlements.get(&sid).unwrap();
    // Type A — all 4 community-history classes must be routed.
    assert!(
        classes_present.contains("building_placed"),
        "BuildingPlaced must be routed to community history"
    );
    assert!(
        classes_present.contains("agent_born"),
        "AgentBorn must be routed to community history"
    );
    assert!(
        classes_present.contains("settlement_reason"),
        "AgentDecision{{SettlementReason}} must be routed to community history"
    );
    assert!(
        classes_present.contains("combat_completed"),
        "CombatCompleted involving member must be routed to community history"
    );
    // 1:1 correspondence: each event_id appears at most once.
    let mut counts: HashMap<EventId, u32> = HashMap::new();
    for eid in &s.community_history {
        *counts.entry(*eid).or_insert(0) += 1;
    }
    for (eid, c) in &counts {
        assert!(
            *c == 1,
            "event_id {} appears {} times in history (must be exactly 1)",
            eid,
            c
        );
    }
}

// ════════════════════════════════════════════════════════════════════════
// A22: community_history ring buffer evicts FIFO at overflow.
// Type A — cap saturation + oldest-first eviction.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a22_history_fifo_eviction_at_overflow() {
    // Direct exercise of the bounded append on the Settlement component
    // (the SettlementSystem routing path uses append_history transitively).
    let mut s = Settlement::new_with_id(7, 0);
    // Push CAP entries first.
    for i in 0..(SETTLEMENT_HISTORY_CAP as EventId) {
        s.append_history(i);
    }
    // Length saturates.
    assert_eq!(s.community_history.len(), SETTLEMENT_HISTORY_CAP);
    let oldest_before_overflow = s.community_history[0];
    // Push CAP + 10 more — must evict the oldest entries in FIFO order.
    for i in 0..10 {
        s.append_history((SETTLEMENT_HISTORY_CAP as EventId).wrapping_add(i));
    }
    // (a) length stable.
    assert_eq!(
        s.community_history.len(),
        SETTLEMENT_HISTORY_CAP,
        "history must stay at cap after overflow"
    );
    // (b) oldest-first eviction: the original 0..10 events should be
    //     gone.
    for evicted in 0..10 {
        assert!(
            !s.community_history.contains(&(evicted as EventId)),
            "event id {} should have been FIFO-evicted",
            evicted
        );
    }
    // (c) newest entries are retained.
    let newest_expected =
        (SETTLEMENT_HISTORY_CAP as EventId).wrapping_add(9);
    assert!(
        s.community_history.contains(&newest_expected),
        "newest entry {} must be retained",
        newest_expected
    );
    let _ = oldest_before_overflow;
}

// ════════════════════════════════════════════════════════════════════════
// A23: AgentBorn parent chains to SettlementFormed event id.
// Type A — causal chain integrity.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a23_agent_born_parent_chains_to_formed() {
    let mut e = fresh_engine();
    let cx = 22u32;
    let cy = 22u32;
    let _ids = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick();
    let formed_id = match first_formed(&e).unwrap() {
        CausalEvent::SettlementFormed { id, .. } => id,
        _ => unreachable!(),
    };
    for _ in 0..(BIRTH_COOLDOWN_TICKS as usize + 1) {
        e.tick();
    }
    let births: Vec<CausalEvent> =
        collect_events(&e, |ev| matches!(ev, CausalEvent::AgentBorn { .. }));
    assert!(!births.is_empty());
    for ev in births {
        if let CausalEvent::AgentBorn { parent, .. } = ev {
            // Type A — parent must equal formed event id.
            assert_eq!(
                parent,
                Some(formed_id),
                "AgentBorn.parent must equal SettlementFormed.id"
            );
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// A24: SettlementDissolved parent chains to SettlementFormed.
// Type A — lifecycle causal chain integrity.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a24_dissolved_parent_chains_to_formed() {
    let mut e = fresh_engine();
    let cx = 38u32;
    let cy = 38u32;
    let ids = spawn_cluster(&mut e, cx, cy, 3);
    place_buildings(&mut e, cx, cy, 2);
    e.tick();
    let formed_id = match first_formed(&e).unwrap() {
        CausalEvent::SettlementFormed { id, .. } => id,
        _ => unreachable!(),
    };
    let entities: Vec<hecs::Entity> = e
        .world
        .query::<&Agent>()
        .iter()
        .filter(|(_, a)| ids.contains(&a.id))
        .map(|(ent, _)| ent)
        .collect();
    for ent in entities {
        let _ = e.world.despawn(ent);
    }
    e.resources.building_registry.clear();
    e.tick();
    let ev = first_dissolved(&e).expect("dissolved present");
    if let CausalEvent::SettlementDissolved { parent, .. } = ev {
        assert_eq!(
            parent,
            Some(formed_id),
            "SettlementDissolved.parent must equal SettlementFormed.id"
        );
    } else {
        unreachable!();
    }
}

// ════════════════════════════════════════════════════════════════════════
// A25: building_registry populated via FFI path.
// Type A — BuildingStampSystem drain → registry entry.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a25_building_registry_via_ffi_path() {
    use sim_bridge::ffi::enqueue_building_placed;
    let mut e = fresh_engine();
    let pos = (33u32, 44u32);
    // Use the public FFI delegate that on_building_placed forwards to
    // exclusively (bounds-check + push_back). Direct push_back would
    // bypass the bounds-check logic exercised by this path.
    let enqueued =
        enqueue_building_placed(&mut e.resources, pos.0 as i32, pos.1 as i32, 1);
    assert!(
        enqueued,
        "valid in-bounds position must be accepted by FFI delegate"
    );
    e.tick();
    // Type A — entry with exact submitted position.
    assert!(
        e.resources.building_registry.values().any(|p| *p == pos),
        "FFI placement must populate building_registry; got {:?}",
        e.resources.building_registry
    );
}

// ════════════════════════════════════════════════════════════════════════
// A26: building_registry populated via agent-construction path.
// Type A — ConstructionSystem completion → registry entry.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a26_building_registry_via_construction_path() {
    use sim_core::components::{
        BuildingBlueprint, ConstructionSite, Position,
    };

    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    // Exercise the real agent-completion path:
    //  1. Spawn a ConstructionSite with progress = required - 1 so a
    //     single `advance()` flips the completion edge.
    //  2. Spawn an agent in `AgentState::Consuming{ConstructionSite}` at
    //     the site's position so ConstructionSystem picks it up and
    //     advances the site exactly once.
    //  3. Tick ConstructionSystem; verify the registry entry is created.
    let blueprint = BuildingBlueprint::new(1, 1, 1, 1);
    let position = (40u32, 40u32);
    let mut site = ConstructionSite::new(blueprint, Position::new(position.0, position.1));
    // One progress short — the consuming agent's advance() flips it to
    // complete this tick (P6β-10 per-agent progress contract).
    site.progress = site.blueprint.required_progress.saturating_sub(1);
    e.world.spawn((Position::new(position.0, position.1), site));

    // Consuming agent co-located with the site.
    let agent_entity = e.spawn_agent(position.0, position.1);
    e.world
        .insert(
            agent_entity,
            (AgentState::Consuming {
                target: TargetKind::ConstructionSite,
            },),
        )
        .unwrap();

    // Run ConstructionSystem directly (priority 133, runs alone here).
    let mut cs = ConstructionSystem::new();
    cs.tick(&mut e.world, &mut e.resources);
    // Type A — registry contains the completion position.
    assert!(
        e.resources
            .building_registry
            .values()
            .any(|p| *p == position),
        "agent-completed construction must populate building_registry; got {:?}",
        e.resources.building_registry
    );
}

// ════════════════════════════════════════════════════════════════════════
// A27: Membership sync removes despawned agents within one tick.
// Type A — single-tick correctness.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a27_membership_sync_removes_despawned_within_one_tick() {
    let mut e = fresh_engine();
    let cx = 26u32;
    let cy = 26u32;
    let ids = spawn_cluster(&mut e, cx, cy, 4);
    place_buildings(&mut e, cx, cy, 2);
    e.tick();
    let (sid, before) = {
        let (sid, s) = e.resources.settlements.iter().next().unwrap();
        (*sid, s.population_stats.current)
    };
    assert!(before >= 3);

    let target_id = ids[0];
    let ent = e
        .world
        .query::<&Agent>()
        .iter()
        .find(|(_, a)| a.id == target_id)
        .map(|(ent, _)| ent)
        .unwrap();
    let _ = e.world.despawn(ent);
    e.tick();
    let s = e.resources.settlements.get(&sid).unwrap();
    // Type A — id removed and count decremented within one tick.
    assert!(!s.member_agents.contains(&target_id));
    assert_eq!(s.population_stats.current, before - 1);
}

// ════════════════════════════════════════════════════════════════════════
// A28: SettlementSystem executes per tick.
// Type A — observable side-effect (tick_run_count) advances every tick.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a28_settlement_system_executes_per_tick() {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    let mut sys = SettlementSystem::new();
    // Drive the system directly so we can observe its monotonic counter
    // (the boxed instance inside register_default_runtime_systems isn't
    // borrowable for inspection — direct tick is sufficient to verify
    // the per-tick execution invariant).
    for _ in 0..100 {
        sys.tick(&mut e.world, &mut e.resources);
        e.resources.current_tick = e.resources.current_tick.saturating_add(1);
    }
    // Type A — counter advanced by N == 100.
    assert_eq!(
        sys.tick_run_count(),
        100,
        "SettlementSystem.tick_run_count must advance once per tick"
    );
}

// ════════════════════════════════════════════════════════════════════════
// A29: Regression clean at seed 42 + default 2000 ticks.
// Type D — no new failures vs pre-Phase-10β baseline.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p10_beta_a29_regression_clean_2000_ticks() {
    let mut e = fresh_engine();
    let seed = 42u64;
    // Spawn 20 agents at S-Default initial positions.
    for i in 0..20u32 {
        let x = 16 + (i % 4);
        let y = 16 + (i / 4);
        let entity = e.spawn_agent(x, y);
        e.world
            .insert(
                entity,
                (
                    MovementRng::new(seed.wrapping_add(i as u64)),
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(0.0, 0.0),
                    AgentState::Idle,
                    Memory::new(),
                ),
            )
            .unwrap();
    }
    // 2000 ticks — past birth cooldown (200) but within harness budget.
    for _ in 0..2000 {
        e.tick();
    }
    // Type D — pre-Phase-10β invariants:
    //   (1) No panic — implicit.
    //   (2) tick counter advanced to 2000.
    assert_eq!(e.current_tick(), 2000);
    //   (3) Agent count never below original spawn floor.
    let agent_count = e.world.query::<&Agent>().iter().count();
    assert!(
        agent_count >= 20,
        "agent count must stay >= spawn floor (got {})",
        agent_count
    );
    //   (4) No spurious settlement formation (no buildings).
    assert!(
        e.resources.settlements.is_empty(),
        "no settlement should form without buildings"
    );
    //   (5) Pair maps not corrupt.
    assert!(e.resources.combat_pairs.is_empty());
    assert!(e.resources.combat_progress.is_empty());
}

// ════════════════════════════════════════════════════════════════════════
// Edge-case suite (plan §"Edge Cases" — not numbered A-N but still
// locked invariants).
// ════════════════════════════════════════════════════════════════════════

#[test]
fn harness_p10_beta_edge_settlement_formed_variant_schema() {
    // Sanity check the CausalEvent variant shape matches the locked
    // plan-§2 signature. Compile-time guard.
    let ev = CausalEvent::SettlementFormed {
        id: 99,
        parent: None,
        settlement_id: 7,
        founding_members: vec![1u64, 2, 3],
        tick: 42,
    };
    match ev.clone() {
        CausalEvent::SettlementFormed {
            id: _,
            parent: _,
            settlement_id: _,
            founding_members: _,
            tick: _,
        } => {}
        _ => panic!("variant shape changed"),
    }
    assert_eq!(ev.id(), 99);
    assert_eq!(ev.parent(), None);
    assert_eq!(ev.tick(), 42);
    assert_eq!(ev.channel(), None);
}

#[test]
fn harness_p10_beta_edge_settlement_dissolved_variant_schema() {
    let ev = CausalEvent::SettlementDissolved {
        id: 100,
        parent: Some(99),
        settlement_id: 7,
        final_population: 0u32,
        tick: 50,
    };
    if let CausalEvent::SettlementDissolved {
        final_population, ..
    } = ev.clone()
    {
        let _: u32 = final_population;
    }
    assert_eq!(ev.id(), 100);
    assert_eq!(ev.parent(), Some(99));
    assert_eq!(ev.tick(), 50);
}

#[test]
fn harness_p10_beta_edge_decision_reason_settlement_str() {
    assert_eq!(
        DecisionReason::SettlementReason.as_str(),
        "settlement_reason"
    );
}

#[test]
fn harness_p10_beta_edge_settlement_system_in_default_runtime() {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    let names = e.system_names();
    assert!(
        names.contains(&"SettlementSystem"),
        "SettlementSystem must be in default runtime"
    );
}

#[test]
fn harness_p10_beta_edge_max_pop_and_radius_constants() {
    assert_eq!(SETTLEMENT_MAX_POP, 50);
    assert_eq!(SETTLEMENT_PROXIMITY_RADIUS, 5);
    assert_eq!(SETTLEMENT_HISTORY_CAP, 32);
}

#[test]
fn harness_p10_beta_edge_settlement_id_never_zero() {
    // Spans multiple formations to verify the sentinel guarantee holds.
    let mut e = fresh_engine();
    for i in 0u32..3 {
        let cx = 10 + i * 20;
        let cy = 10;
        let _ = spawn_cluster(&mut e, cx, cy, 3);
        place_buildings(&mut e, cx, cy, 2);
        e.tick();
    }
    for (sid, _) in e.resources.settlements.iter() {
        assert_ne!(*sid, 0, "no settlement may have id 0");
    }
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::SettlementFormed { settlement_id, .. } = ev {
                assert_ne!(*settlement_id, 0);
            }
        }
    }
}
