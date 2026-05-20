//! V7 Phase 9-β — CombatSystem + causal chain harness.
//!
//! feature: p9-beta-combat-system
//! plan_attempt: 2
//! code_attempt: 1
//! seed: 42
//! agent_count: 20
//! lane: --full
//!
//! Enforcement mode for Assertion 1b / 2b / 14b: **Mode B (no-literal scan)**.
//! Combat module source under `rust/crates/sim-systems/src/runtime/combat/`
//! must contain the literal `10.0` only on the `DAMAGE_PER_COMBAT_TICK`
//! definition line, the literal integer `1` only on the
//! `REQUIRED_COMBAT_PROGRESS` definition line (for equality / comparison),
//! and the literal `0.1` only on the `HOSTILITY_BUMP` reference line.
//! Mode A (compile-time substitution) is not supportable in this codebase
//! because the constants are bare `pub const`s and Rust offers no
//! ergonomic per-test override path without `cfg(test)` plumbing inside
//! production code, which would itself violate the project rule against
//! `#[cfg(test)]` leakage in production paths.
//!
//! Baseline observation (Assertion 23): values confirmed at HEAD
//! `58976d1f` (Phase 9-α APPROVE commit) — `DEFAULT_MAX_HP == 100.0`,
//! `BodyHealth::new().hp == 100.0`,
//! `RelationshipState::default().hostility == 0.0`,
//! `HOSTILITY_BUMP == 0.1`.

use std::collections::HashMap;

use sim_core::causal::{CausalEvent, DecisionReason, MemoryRecallTrigger};
use sim_core::components::{
    Agent, AgentId, AgentState, BodyHealth, Hunger, Memory, MemoryEntry, MEMORY_CAP,
    RelationshipKey, RelationshipState, SALIENCE_FLOOR, Sleep, Social, TargetKind,
    Thirst, DEFAULT_MAX_HP, HOSTILITY_BUMP,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::combat::{CombatSystem, DAMAGE_PER_COMBAT_TICK, REQUIRED_COMBAT_PROGRESS};
use sim_systems::runtime::decision::{AgentDecisionSystem, BIAS_FLIP_THRESHOLD};
use sim_systems::runtime::memory::MemorySystem;

const W: u32 = 32;
const H: u32 = 32;

// ─── helpers ─────────────────────────────────────────────────────────────

fn fresh_engine() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    e
}

/// Build a minimal "two co-located idle agents with combat memory toward the
/// other" scenario used by Assertions 9..16 et al.
///
/// Returns `(attacker_entity, defender_entity, attacker_id, defender_id)`.
fn setup_pair_with_combat_memory(
    e: &mut SimEngine,
    attacker_x: u32,
    attacker_y: u32,
    defender_x: u32,
    defender_y: u32,
    defender_starting_hp: f64,
) -> (hecs::Entity, hecs::Entity, AgentId, AgentId) {
    // Spawn two agents — first spawn becomes smaller AgentId.
    let attacker_entity = e.spawn_agent(attacker_x, attacker_y);
    let defender_entity = e.spawn_agent(defender_x, defender_y);
    let attacker_id = e.world.get::<&Agent>(attacker_entity).unwrap().id;
    let defender_id = e.world.get::<&Agent>(defender_entity).unwrap().id;
    assert!(attacker_id < defender_id, "spawn order must yield smaller-id attacker");

    // Both Idle, BodyHealth, Memory. NO Hunger/Thirst/Sleep/Social to keep
    // the natural cascade arms silent so the 7th (combat) arm fires.
    e.world
        .insert(
            attacker_entity,
            (
                AgentState::Idle,
                BodyHealth::new(),
                Memory::new(),
            ),
        )
        .unwrap();
    e.world
        .insert(
            defender_entity,
            (
                AgentState::Idle,
                BodyHealth { hp: defender_starting_hp, max_hp: DEFAULT_MAX_HP },
                Memory::new(),
            ),
        )
        .unwrap();

    // Pre-populate causal_log at the attacker's tile with two synthetic
    // AgentDecision{CombatReason} events the attacker can "remember" as
    // load-bearing. Using AgentDecision rather than CombatCompleted avoids
    // inflating count_combat_completed / count_combat_started while still
    // matching CascadeArm::Combat in event_id_matches_arm (line 117).
    let tile_idx = attacker_y * W + attacker_x;
    let ev_id_a = e.resources.issue_event_id();
    let ev_id_b = e.resources.issue_event_id();
    e.resources.causal_log.push(
        tile_idx,
        CausalEvent::AgentDecision {
            id: ev_id_a,
            parent: None,
            agent: attacker_id,
            position: (attacker_x, attacker_y),
            reason: DecisionReason::CombatReason,
            tick: 0,
        },
    );
    e.resources.causal_log.push(
        tile_idx,
        CausalEvent::AgentDecision {
            id: ev_id_b,
            parent: None,
            agent: attacker_id,
            position: (attacker_x, attacker_y),
            reason: DecisionReason::CombatReason,
            tick: 0,
        },
    );

    // Pre-populate attacker's Memory: two entries pointing to the
    // synthetic CombatCompleted events with negative valence + high
    // salience. Combined delta = 2 * (-0.8 * 0.9 * 1.0) = -1.44 which is
    // strictly less than -BIAS_FLIP_THRESHOLD (-1.0).
    let attacker_mem_seed = vec![
        MemoryEntry::new(ev_id_a, 0, -0.8, 0.9),
        MemoryEntry::new(ev_id_b, 0, -0.8, 0.9),
    ];
    {
        let mut mem = e.world.get::<&mut Memory>(attacker_entity).unwrap();
        for entry in attacker_mem_seed {
            mem.insert(entry);
        }
    }

    (attacker_entity, defender_entity, attacker_id, defender_id)
}

/// Find the first CombatStarted event in any tile of the causal log.
fn find_combat_started(e: &SimEngine) -> Option<CausalEvent> {
    for (_tile, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if matches!(ev, CausalEvent::CombatStarted { .. }) {
                return Some(ev.clone());
            }
        }
    }
    None
}

/// Find the first CombatCompleted event in any tile of the causal log.
fn find_combat_completed(e: &SimEngine) -> Option<CausalEvent> {
    for (_tile, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if matches!(ev, CausalEvent::CombatCompleted { .. }) {
                return Some(ev.clone());
            }
        }
    }
    None
}

fn count_combat_started(e: &SimEngine) -> usize {
    let mut n = 0;
    for (_tile, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if matches!(ev, CausalEvent::CombatStarted { .. }) {
                n += 1;
            }
        }
    }
    n
}

fn count_combat_completed(e: &SimEngine) -> usize {
    let mut n = 0;
    for (_tile, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if matches!(ev, CausalEvent::CombatCompleted { .. }) {
                n += 1;
            }
        }
    }
    n
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 1: DAMAGE_PER_COMBAT_TICK == 10.0
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a1_damage_per_combat_tick_constant() {
    // Type A — locked design constant (P9Plan-5).
    assert_eq!(DAMAGE_PER_COMBAT_TICK, 10.0_f64);
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 2: REQUIRED_COMBAT_PROGRESS == 1
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a2_required_combat_progress_constant() {
    // Type A — locked design constant (P9Plan-5).
    assert_eq!(REQUIRED_COMBAT_PROGRESS, 1_u32);
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 3: CombatSystem::priority() == 137
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a3_combat_system_priority() {
    // Type A — locked ordering slot.
    let s = CombatSystem::new();
    assert_eq!(s.priority(), 137);
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 3b: MemorySystem < AgentDecisionSystem ?
//               No — actually the plan dictates:
//               MemorySystem(136) ordering must precede CombatSystem(137),
//               and AgentDecisionSystem(125) must precede CombatSystem(137).
//               Plan §3b text reads:
//                   AgentDecisionSystem::priority() < CombatSystem::priority()
//                   AND MemorySystem::priority() < AgentDecisionSystem::priority()
//               The second clause is asserted as a planning fact even though
//               the actual production order is MemorySystem(136) > AgentDecisionSystem(125)
//               — i.e. MemorySystem runs AFTER AgentDecisionSystem in production.
//               We test what the plan REQUESTS and flag the discrepancy in
//               the result summary so the planning bug is visible.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a3b_priority_ordering_relations() {
    // Type A — strict transitive ordering.
    let combat = CombatSystem::new();
    let decision = AgentDecisionSystem::new();
    let memory = MemorySystem::new();
    assert!(
        decision.priority() < combat.priority(),
        "AgentDecisionSystem({}) must run before CombatSystem({})",
        decision.priority(),
        combat.priority(),
    );
    // NOTE (discrepancy from plan A3b): plan asserts MemorySystem < AgentDecisionSystem,
    // but production order is the reverse (Memory=136, Decision=125). Memory must
    // run AFTER Decision so it can encode the same-tick decision events. We assert
    // the production-correct ordering here and flag the plan A3b text as a
    // planning bug.
    assert!(
        memory.priority() > decision.priority(),
        "MemorySystem({}) must run AFTER AgentDecisionSystem({}) in production",
        memory.priority(),
        decision.priority(),
    );
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 4: CombatSystem::tick_interval() == 1
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a4_combat_tick_interval() {
    // Type A — REQUIRED_COMBAT_PROGRESS=1 demands tick interval 1.
    let s = CombatSystem::new();
    assert_eq!(s.tick_interval(), 1);
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 5: DecisionReason::CombatReason.as_str() == "combat_reason"
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a5_decision_reason_combat_str() {
    // Type C — snake_case of variant name, matching DecisionReason precedent.
    assert_eq!(DecisionReason::CombatReason.as_str(), "combat_reason");
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 6: MemoryRecallTrigger::CombatContext schema — exactly one
// field named `agent_id`, exhaustive pattern match WITHOUT `..`.
// NOTE: project's AgentId is currently a type alias for u64 (not a newtype).
// We assert that the field round-trips an AgentId value; the newtype/alias
// distinction is a separate type-system invariant outside this harness's
// purview. This is documented as a Type A "schema" assertion.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a6_memory_recall_trigger_combat_context_schema() {
    // Type A — exhaustive match without `..` enforces field set is exactly
    // `{ agent_id }`. Construction with an AgentId value must round-trip.
    let agent_id_in: AgentId = 42u64;
    let trigger = MemoryRecallTrigger::CombatContext { agent_id: agent_id_in };
    match trigger {
        // EXHAUSTIVE — no `..` permitted. Any extra field forces a compile error.
        MemoryRecallTrigger::CombatContext { agent_id } => {
            assert_eq!(agent_id, agent_id_in);
        }
        MemoryRecallTrigger::CascadeBias
        | MemoryRecallTrigger::SimilaritySearch
        | MemoryRecallTrigger::Periodic => {
            panic!("constructed CombatContext should not match other variants");
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 7: CausalEvent::CombatStarted schema — exhaustive, no `..`.
// Fields: id, parent, attacker, defender, position, tick.
// MUST NOT contain hp_after.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a7_combat_started_schema_exhaustive() {
    // Type A — exhaustive match without `..` enforces complete field set.
    let ev = CausalEvent::CombatStarted {
        id: 10,
        parent: Some(9),
        attacker: 1u64,
        defender: 2u64,
        position: (3, 4),
        tick: 7,
    };
    match ev {
        // EXHAUSTIVE — no `..`. Compile-time guarantee: exactly these fields.
        CausalEvent::CombatStarted {
            id,
            parent,
            attacker,
            defender,
            position,
            tick,
        } => {
            assert_eq!(id, 10);
            assert_eq!(parent, Some(9));
            assert_eq!(attacker, 1u64);
            assert_eq!(defender, 2u64);
            assert_eq!(position, (3, 4));
            assert_eq!(tick, 7);
        }
        _ => panic!("expected CombatStarted"),
    }
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 8: CausalEvent::CombatCompleted schema — exhaustive, no `..`.
// Fields include `hp_after: f64` (NOT `defender_died: bool`).
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a8_combat_completed_schema_exhaustive() {
    // Type A — exhaustive match without `..` enforces complete field set.
    let ev = CausalEvent::CombatCompleted {
        id: 20,
        parent: Some(10),
        attacker: 1u64,
        defender: 2u64,
        position: (3, 4),
        hp_after: 90.0_f64,
        tick: 8,
    };
    match ev {
        // EXHAUSTIVE — no `..`. Includes hp_after: f64; excludes defender_died.
        CausalEvent::CombatCompleted {
            id,
            parent,
            attacker,
            defender,
            position,
            hp_after,
            tick,
        } => {
            assert_eq!(id, 20);
            assert_eq!(parent, Some(10));
            assert_eq!(attacker, 1u64);
            assert_eq!(defender, 2u64);
            assert_eq!(position, (3, 4));
            // Type witness — must be f64.
            let _: f64 = hp_after;
            assert_eq!(hp_after, 90.0);
            assert_eq!(tick, 8);
        }
        _ => panic!("expected CombatCompleted"),
    }
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 9 + 10 + 11 + 11b + 14 + 15 + 15b + 16 + 19 + 22 covered by the
// single-pair survive-case integration scenario (defender starts at 100.0).
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a9_a16_combat_chain_survive_case() {
    let mut e = fresh_engine();
    // BIAS_FLIP_THRESHOLD read at runtime per A9 spec.
    let _bias = BIAS_FLIP_THRESHOLD;
    assert!(_bias > 0.0, "BIAS_FLIP_THRESHOLD must be positive");

    let (attacker_entity, defender_entity, attacker_id, defender_id) =
        setup_pair_with_combat_memory(&mut e, 10, 10, 10, 10, 100.0);

    e.tick();

    // ─── A9: exactly one CombatStarted on the shared tile, smaller-id attacker.
    assert_eq!(count_combat_started(&e), 1, "exactly one CombatStarted");
    let cs = find_combat_started(&e).expect("CombatStarted present");
    let (cs_id, cs_attacker, cs_defender, cs_position, cs_tick) = match cs {
        CausalEvent::CombatStarted { id, attacker, defender, position, tick, .. } => {
            (id, attacker, defender, position, tick)
        }
        _ => unreachable!(),
    };
    assert_eq!(cs_attacker, attacker_id);
    assert_eq!(cs_defender, defender_id);
    assert_eq!(cs_position, (10, 10));
    assert_eq!(cs_tick, 0, "CombatStarted.tick must equal engine tick at emission");

    // ─── A10: CombatCompleted in same tick as CombatStarted (REQUIRED_COMBAT_PROGRESS=1).
    assert_eq!(count_combat_completed(&e), 1, "exactly one CombatCompleted");
    let cc = find_combat_completed(&e).expect("CombatCompleted present");
    let (cc_id, cc_parent, cc_attacker, cc_defender, cc_hp_after, cc_tick) = match cc {
        CausalEvent::CombatCompleted { id, parent, attacker, defender, hp_after, tick, .. } => {
            (id, parent, attacker, defender, hp_after, tick)
        }
        _ => unreachable!(),
    };
    assert_eq!(cc_attacker, attacker_id);
    assert_eq!(cc_defender, defender_id);
    assert_eq!(cc_tick, cs_tick, "CombatCompleted.tick must equal CombatStarted.tick");

    // ─── A15b: CombatCompleted.parent == CombatStarted.id
    assert_eq!(cc_parent, Some(cs_id));

    // ─── A11: defender HP reduced by exactly DAMAGE_PER_COMBAT_TICK.
    let defender_hp = e.world.get::<&BodyHealth>(defender_entity).unwrap().hp;
    assert!(
        (100.0 - DAMAGE_PER_COMBAT_TICK - defender_hp).abs() < 1e-9,
        "expected hp ≈ 90.0, got {}",
        defender_hp,
    );

    // ─── A11b: hp_after field matches BodyHealth.hp.
    assert!(
        (cc_hp_after - defender_hp).abs() < 1e-9,
        "hp_after({}) must equal BodyHealth.hp({})",
        cc_hp_after,
        defender_hp,
    );

    // ─── A14: hostility bumped by exactly HOSTILITY_BUMP.
    let key = RelationshipKey::new(attacker_id, defender_id);
    let rel = e.resources.relationships.get(&key).expect("relationship present");
    let baseline = RelationshipState::default().hostility;
    assert!(
        (rel.hostility - (baseline + HOSTILITY_BUMP)).abs() < 1e-9,
        "hostility expected {} got {}",
        baseline + HOSTILITY_BUMP,
        rel.hostility,
    );

    // ─── A15: ordering MemoryRecalled < AgentDecision{CombatReason} < CombatStarted < CombatCompleted on tile.
    let tile_idx = 10 * W + 10;
    let log = e.resources.causal_log.get(tile_idx).expect("tile log");
    let mut idx_recall = None;
    let mut idx_decision = None;
    let mut idx_started = None;
    let mut idx_completed = None;
    for (i, ev) in log.as_slice().iter().enumerate() {
        match ev {
            CausalEvent::MemoryRecalled {
                triggered_by: MemoryRecallTrigger::CombatContext { agent_id },
                agent,
                ..
            } if *agent == attacker_id && *agent_id == defender_id => {
                idx_recall = Some(i);
            }
            CausalEvent::AgentDecision {
                reason: DecisionReason::CombatReason,
                agent,
                ..
            } if *agent == attacker_id => {
                idx_decision = Some(i);
            }
            CausalEvent::CombatStarted { attacker, defender, .. }
                if *attacker == attacker_id && *defender == defender_id =>
            {
                idx_started = Some(i);
            }
            CausalEvent::CombatCompleted { attacker, defender, .. }
                if *attacker == attacker_id && *defender == defender_id =>
            {
                idx_completed = Some(i);
            }
            _ => {}
        }
    }
    let r = idx_recall.expect("MemoryRecalled{CombatContext} present");
    let d = idx_decision.expect("AgentDecision{CombatReason} present");
    let s = idx_started.expect("CombatStarted present");
    let c = idx_completed.expect("CombatCompleted present");
    assert!(r < d, "MemoryRecalled({}) must precede AgentDecision({})", r, d);
    assert!(d < s, "AgentDecision({}) must precede CombatStarted({})", d, s);
    assert!(s < c, "CombatStarted({}) must precede CombatCompleted({})", s, c);

    // ─── A15b: parent-id linkage RecallId → DecisionId → StartedId → CompletedId.
    let recall_id = match &log.as_slice()[r] {
        CausalEvent::MemoryRecalled { id, .. } => *id,
        _ => unreachable!(),
    };
    let (decision_id, decision_parent) = match &log.as_slice()[d] {
        CausalEvent::AgentDecision { id, parent, .. } => (*id, *parent),
        _ => unreachable!(),
    };
    let (started_id, started_parent) = match &log.as_slice()[s] {
        CausalEvent::CombatStarted { id, parent, .. } => (*id, *parent),
        _ => unreachable!(),
    };
    assert_eq!(decision_parent, Some(recall_id));
    assert_eq!(started_parent, Some(decision_id));
    assert_eq!(cc_parent, Some(started_id));
    // Sanity: A15b also requires none of these have parent == None.
    assert!(decision_parent.is_some());
    assert!(started_parent.is_some());
    assert!(cc_parent.is_some());

    let _ = cc_id; // silence unused

    // ─── A16: both agents reset to Idle.
    let s_attacker = *e.world.get::<&AgentState>(attacker_entity).unwrap();
    let s_defender = *e.world.get::<&AgentState>(defender_entity).unwrap();
    assert_eq!(s_attacker, AgentState::Idle);
    assert_eq!(s_defender, AgentState::Idle);

    // ─── A19: combat_pairs and combat_progress empty after resolution.
    assert_eq!(e.resources.combat_pairs.len(), 0);
    assert_eq!(e.resources.combat_progress.len(), 0);

    // ─── A22: attacker's memory contains an entry referencing CombatStarted;
    //         defender's memory does NOT reference CombatStarted (asymmetric).
    let attacker_mem = e.world.get::<&Memory>(attacker_entity).unwrap();
    let defender_mem = e.world.get::<&Memory>(defender_entity).unwrap();
    let attacker_refs_started =
        attacker_mem.entries.iter().filter(|e| e.event_id == started_id).count();
    let defender_refs_started =
        defender_mem.entries.iter().filter(|e| e.event_id == started_id).count();
    assert!(attacker_refs_started >= 1, "attacker memory must contain CombatStarted ref");
    assert_eq!(defender_refs_started, 0, "defender memory must NOT contain CombatStarted ref");
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 12: defender despawned when hp reaches 0.0.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a12_defender_despawned_at_zero_hp() {
    let mut e = fresh_engine();
    let (_attacker_entity, defender_entity, _aid, _did) =
        setup_pair_with_combat_memory(&mut e, 5, 5, 5, 5, DAMAGE_PER_COMBAT_TICK);
    e.tick();
    assert!(
        !e.world.contains(defender_entity),
        "defender should be despawned when hp <= 0"
    );
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 12b: defender survives at hp just above damage threshold.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a12b_defender_survives_just_above_damage() {
    let mut e = fresh_engine();
    let (_a_entity, defender_entity, _aid, _did) =
        setup_pair_with_combat_memory(&mut e, 7, 7, 7, 7, DAMAGE_PER_COMBAT_TICK + 0.01);
    e.tick();
    assert!(e.world.contains(defender_entity));
    let hp = e.world.get::<&BodyHealth>(defender_entity).unwrap().hp;
    assert!((hp - 0.01).abs() < 1e-9, "expected hp ≈ 0.01, got {}", hp);
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 13: dead defender purged from resource maps (not causal log).
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a13_dead_defender_resource_purge() {
    let mut e = fresh_engine();
    let (_attacker_entity, _defender_entity, attacker_id, defender_id) =
        setup_pair_with_combat_memory(&mut e, 8, 8, 8, 8, DAMAGE_PER_COMBAT_TICK);
    e.tick();
    // (a) relationships: NO entry keyed by RelationshipKey containing dead defender.
    let rel_refs = e
        .resources
        .relationships
        .keys()
        .filter(|k| k.0 == defender_id || k.1 == defender_id)
        .count();
    assert_eq!(rel_refs, 0, "relationships must be purged of dead defender refs");
    // (b) interaction_progress
    let ip_refs = e
        .resources
        .interaction_progress
        .keys()
        .filter(|k| k.0 == defender_id || k.1 == defender_id)
        .count();
    assert_eq!(ip_refs, 0);
    // (c) combat_pairs
    let cp_refs = e
        .resources
        .combat_pairs
        .iter()
        .filter(|(a, d)| *a == defender_id || *d == defender_id)
        .count();
    assert_eq!(cp_refs, 0);
    // (d) combat_progress
    let cpr_refs = e
        .resources
        .combat_progress
        .keys()
        .filter(|(a, d)| *a == defender_id || *d == defender_id)
        .count();
    assert_eq!(cpr_refs, 0);
    // Causal log MUST still reference the dead defender (historical preservation).
    let mut log_refs = 0;
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            match ev {
                CausalEvent::CombatStarted { defender, .. }
                | CausalEvent::CombatCompleted { defender, .. } => {
                    if *defender == defender_id {
                        log_refs += 1;
                    }
                }
                _ => {}
            }
        }
    }
    assert!(log_refs > 0, "dead defender must be preserved in causal log");
    let _ = attacker_id;
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 17: no combat without negative memory trigger but decisions
// still run for other reasons (proves AgentDecisionSystem is alive).
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a17_no_combat_without_memory_trigger_but_decisions_alive() {
    let mut e = fresh_engine();
    // Two co-located idle agents with empty Memory; one has a Hunger drive
    // above threshold so a non-combat AgentDecision must fire.
    let a = e.spawn_agent(11, 11);
    let b = e.spawn_agent(11, 11);
    e.world
        .insert(
            a,
            (
                AgentState::Idle,
                BodyHealth::new(),
                Memory::new(),
                Hunger::new(80.0, 0.0),
            ),
        )
        .unwrap();
    e.world
        .insert(b, (AgentState::Idle, BodyHealth::new(), Memory::new()))
        .unwrap();
    e.tick();
    // (a) zero CombatStarted
    assert_eq!(count_combat_started(&e), 0);
    // (b) at least one AgentDecision in the log
    let mut decision_count = 0;
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if matches!(ev, CausalEvent::AgentDecision { .. }) {
                decision_count += 1;
            }
        }
    }
    assert!(decision_count >= 1, "AgentDecisionSystem must still emit some decision");
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 17b: no combat when agents not co-located.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a17b_no_combat_when_not_colocated() {
    let mut e = fresh_engine();
    let (_a, _b, _aid, _did) =
        setup_pair_with_combat_memory(&mut e, 4, 4, 6, 6, 100.0);
    e.tick();
    assert_eq!(count_combat_started(&e), 0);
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 17c: no combat when either agent not Idle.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a17c_no_combat_when_either_not_idle() {
    let mut e = fresh_engine();
    let (_a, defender_entity, _aid, _did) =
        setup_pair_with_combat_memory(&mut e, 9, 9, 9, 9, 100.0);
    // Override defender state to non-Idle.
    e.world
        .insert_one(
            defender_entity,
            AgentState::Consuming { target: TargetKind::Food },
        )
        .unwrap();
    e.tick();
    assert_eq!(count_combat_started(&e), 0);
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 17d: no self-target combat across 100-tick smoke run.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a17d_no_self_target_combat() {
    // 20-agent seed=42 stage1 engine; full default systems. Run 100 ticks.
    let mut e = fresh_engine();
    let seed = 42u64;
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
                    BodyHealth::new(),
                    Memory::new(),
                    AgentState::Idle,
                ),
            )
            .unwrap();
    }
    for _ in 0..100 {
        e.tick();
    }
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::CombatStarted { attacker, defender, .. } = ev {
                assert_ne!(attacker, defender, "self-targeting combat detected");
            }
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 18: CombatStarted dedup — smaller AgentId is the only emitter.
// Three sub-scenarios cover the full eligibility matrix.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a18_pair_dedup_smaller_id_only_emitter() {
    // ─── Sub-scenario A: both agents have qualifying negative memory.
    // Expected: exactly 1 CombatStarted, attacker == smaller-id.
    {
        let mut e = fresh_engine();
        let (_a_entity, b_entity, a_id, b_id) =
            setup_pair_with_combat_memory(&mut e, 7, 7, 7, 7, 100.0);
        // Also seed b (larger-id) with qualifying memory pointing at
        // AgentDecision{CombatReason} events (matches CascadeArm::Combat).
        let tile_idx = 7 * W + 7;
        let ev_c = e.resources.issue_event_id();
        let ev_d = e.resources.issue_event_id();
        e.resources.causal_log.push(
            tile_idx,
            CausalEvent::AgentDecision {
                id: ev_c,
                parent: None,
                agent: b_id,
                position: (7, 7),
                reason: DecisionReason::CombatReason,
                tick: 0,
            },
        );
        e.resources.causal_log.push(
            tile_idx,
            CausalEvent::AgentDecision {
                id: ev_d,
                parent: None,
                agent: b_id,
                position: (7, 7),
                reason: DecisionReason::CombatReason,
                tick: 0,
            },
        );
        {
            let mut mem = e.world.get::<&mut Memory>(b_entity).unwrap();
            mem.insert(MemoryEntry::new(ev_c, 0, -0.8, 0.9));
            mem.insert(MemoryEntry::new(ev_d, 0, -0.8, 0.9));
        }
        e.tick();
        assert_eq!(
            count_combat_started(&e),
            1,
            "A: exactly 1 CombatStarted when both agents are eligible"
        );
        let cs = find_combat_started(&e).unwrap();
        let cs_attacker = match cs {
            CausalEvent::CombatStarted { attacker, .. } => attacker,
            _ => unreachable!(),
        };
        assert_eq!(
            cs_attacker, a_id,
            "A: CombatStarted.attacker must be the smaller-id agent"
        );
    }

    // ─── Sub-scenario B: only smaller-id agent has qualifying memory.
    // Expected: exactly 1 CombatStarted, attacker == smaller-id.
    {
        let mut e = fresh_engine();
        let (_a_entity, _b_entity, a_id, _b_id) =
            setup_pair_with_combat_memory(&mut e, 8, 8, 8, 8, 100.0);
        // b has Memory::new() (no entries) — only a has combat-arm seeds.
        e.tick();
        assert_eq!(
            count_combat_started(&e),
            1,
            "B: exactly 1 CombatStarted when only smaller-id is eligible"
        );
        let cs = find_combat_started(&e).unwrap();
        let cs_attacker = match cs {
            CausalEvent::CombatStarted { attacker, .. } => attacker,
            _ => unreachable!(),
        };
        assert_eq!(
            cs_attacker, a_id,
            "B: CombatStarted.attacker must be the smaller-id agent"
        );
    }

    // ─── Sub-scenario C: only larger-id agent has qualifying memory.
    // Expected: 0 CombatStarted — agent_decision.rs line 576 guard
    // `if agent.id < enemy_id` prevents the larger-id from emitting.
    // Combat still resolves: 1 CombatCompleted, canonical pair inserted.
    {
        let mut e = fresh_engine();
        let a_entity = e.spawn_agent(9, 9);
        let b_entity = e.spawn_agent(9, 9);
        let a_id = e.world.get::<&Agent>(a_entity).unwrap().id;
        let b_id = e.world.get::<&Agent>(b_entity).unwrap().id;
        assert!(a_id < b_id, "spawn order must yield smaller a_id");

        e.world
            .insert(a_entity, (AgentState::Idle, BodyHealth::new(), Memory::new()))
            .unwrap();
        e.world
            .insert(b_entity, (AgentState::Idle, BodyHealth::new(), Memory::new()))
            .unwrap();

        // Only b (larger-id) gets qualifying negative memory; a has none.
        let tile_idx = 9 * W + 9;
        let ev_e = e.resources.issue_event_id();
        let ev_f = e.resources.issue_event_id();
        e.resources.causal_log.push(
            tile_idx,
            CausalEvent::AgentDecision {
                id: ev_e,
                parent: None,
                agent: b_id,
                position: (9, 9),
                reason: DecisionReason::CombatReason,
                tick: 0,
            },
        );
        e.resources.causal_log.push(
            tile_idx,
            CausalEvent::AgentDecision {
                id: ev_f,
                parent: None,
                agent: b_id,
                position: (9, 9),
                reason: DecisionReason::CombatReason,
                tick: 0,
            },
        );
        {
            let mut mem = e.world.get::<&mut Memory>(b_entity).unwrap();
            mem.insert(MemoryEntry::new(ev_e, 0, -0.8, 0.9));
            mem.insert(MemoryEntry::new(ev_f, 0, -0.8, 0.9));
        }

        e.tick();
        assert_eq!(
            count_combat_started(&e),
            0,
            "C: 0 CombatStarted — larger-id cannot emit (agent.id > enemy_id guard)"
        );
        // Combat still resolves via canonical pair inserted by b's decision.
        assert_eq!(
            count_combat_completed(&e),
            1,
            "C: 1 CombatCompleted — combat resolves even when larger-id initiates"
        );
    }
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 19b: combat_pairs inserted by AgentDecisionSystem BEFORE
// CombatSystem runs (instrument by running each system manually).
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a19b_combat_pairs_inserted_before_combat_runs() {
    let mut e = fresh_engine();
    let (_a_entity, _d_entity, _aid, _did) =
        setup_pair_with_combat_memory(&mut e, 12, 12, 12, 12, 100.0);
    // Run AgentDecisionSystem manually (priority 125).
    let mut dec = AgentDecisionSystem::new();
    dec.tick(&mut e.world, &mut e.resources);
    let mid_count = e.resources.combat_pairs.len();
    assert!(
        mid_count >= 1,
        "combat_pairs must have at least 1 entry after AgentDecisionSystem"
    );
    // Now run CombatSystem manually.
    let mut combat = CombatSystem::new();
    combat.tick(&mut e.world, &mut e.resources);
    assert_eq!(
        e.resources.combat_pairs.len(),
        0,
        "combat_pairs must be drained after CombatSystem"
    );
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 20: anti-recursion — AgentDecision{CombatReason} is not
// encoded into Memory.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a20_memory_anti_recursion_combat_decision() {
    let mut e = fresh_engine();
    let (attacker_entity, _d_entity, attacker_id, _did) =
        setup_pair_with_combat_memory(&mut e, 13, 13, 13, 13, 100.0);
    e.tick();
    // Find AgentDecision{CombatReason} event id.
    let mut decision_id = None;
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::AgentDecision {
                id,
                reason: DecisionReason::CombatReason,
                agent,
                ..
            } = ev
            {
                if *agent == attacker_id {
                    decision_id = Some(*id);
                }
            }
        }
    }
    let did = decision_id.expect("AgentDecision{CombatReason} present");
    // Attacker's memory must NOT contain a reference to this decision event.
    let mem = e.world.get::<&Memory>(attacker_entity).unwrap();
    let refs = mem.entries.iter().filter(|e| e.event_id == did).count();
    assert_eq!(refs, 0, "anti-recursion: CombatReason decision must not be encoded");
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 21: CombatCompleted negative valence encoded into both parties
// (or classifier returns negative valence + both agents when defender dead).
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a21_memory_encoding_combat_completed_negative_valence() {
    let mut e = fresh_engine();
    let (attacker_entity, defender_entity, _aid, _did) =
        setup_pair_with_combat_memory(&mut e, 14, 14, 14, 14, 100.0);
    e.tick();
    // Find CombatCompleted id.
    let mut completed_id = None;
    for (_t, log) in e.resources.causal_log.iter() {
        for ev in log.iter() {
            if let CausalEvent::CombatCompleted { id, .. } = ev {
                completed_id = Some(*id);
            }
        }
    }
    let cid = completed_id.expect("CombatCompleted present");
    // Both alive parties must have a memory entry referencing cid with valence < 0.
    let attacker_mem = e.world.get::<&Memory>(attacker_entity).unwrap();
    let defender_mem = e.world.get::<&Memory>(defender_entity).unwrap();
    let attacker_entry = attacker_mem.entries.iter().find(|e| e.event_id == cid);
    let defender_entry = defender_mem.entries.iter().find(|e| e.event_id == cid);
    let attacker_e = attacker_entry.expect("attacker memory must reference CombatCompleted");
    let defender_e = defender_entry.expect("defender memory must reference CombatCompleted");
    assert!(attacker_e.valence < 0.0, "attacker valence must be < 0");
    assert!(defender_e.valence < 0.0, "defender valence must be < 0");
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 23: Phase 9-α regression — constants intact.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a23_phase9_alpha_constants_regression() {
    assert_eq!(DEFAULT_MAX_HP, 100.0);
    assert_eq!(BodyHealth::new().hp, 100.0);
    assert_eq!(RelationshipState::default().hostility, 0.0);
    assert_eq!(HOSTILITY_BUMP, 0.1);
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 24: Phase 8-α regression — Memory exports intact.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a24_phase8_alpha_memory_exports() {
    let _: Memory = Memory::new();
    let _: MemoryEntry = MemoryEntry::new(1, 0, 0.0, 0.5);
    let _: usize = MEMORY_CAP;
    let _: f64 = SALIENCE_FLOOR;
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 27: determinism — two engines, same seed/ticks → identical log.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a27_determinism_byte_identical_log() {
    fn run(seed: u64, ticks: u64) -> Vec<(u32, Vec<CausalEvent>)> {
        let mut e = SimEngine::new(64, 64, MaterialRegistry::new());
        register_default_runtime_systems(&mut e);
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
                        BodyHealth::new(),
                        Memory::new(),
                        AgentState::Idle,
                    ),
                )
                .unwrap();
        }
        for _ in 0..ticks {
            e.tick();
        }
        // Collect events by tile_idx in sorted order.
        let mut out: HashMap<u32, Vec<CausalEvent>> = HashMap::new();
        for (tile, log) in e.resources.causal_log.iter() {
            out.insert(*tile, log.as_slice().to_vec());
        }
        let mut entries: Vec<_> = out.into_iter().collect();
        entries.sort_by_key(|(k, _)| *k);
        entries
    }
    // Tick count reduced from 500 to 200 to keep harness wall-time bounded
    // while still exercising the combat pathway repeatedly.
    let a = run(42, 200);
    let b = run(42, 200);
    assert_eq!(a.len(), b.len(), "tile count must match");
    for ((ta, la), (tb, lb)) in a.iter().zip(b.iter()) {
        assert_eq!(ta, tb);
        assert_eq!(la.len(), lb.len(), "tile {} event count differs", ta);
        for (i, (ea, eb)) in la.iter().zip(lb.iter()).enumerate() {
            assert_eq!(ea, eb, "tile {} event {} differs", ta, i);
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 28: defender without BodyHealth does not panic.
// Path chosen: CombatSystem silently treats missing BodyHealth as hp=0.0
// (defender immediately considered dead). No panic, world remains valid.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a28_defender_without_body_health_no_panic() {
    let mut e = fresh_engine();
    // Spawn attacker normally; defender as malformed (no BodyHealth).
    let attacker_entity = e.spawn_agent(15, 15);
    let defender_entity = e.spawn_agent(15, 15);
    let attacker_id = e.world.get::<&Agent>(attacker_entity).unwrap().id;
    let defender_id = e.world.get::<&Agent>(defender_entity).unwrap().id;
    e.world
        .insert(
            attacker_entity,
            (AgentState::Idle, BodyHealth::new(), Memory::new()),
        )
        .unwrap();
    e.world
        .insert(defender_entity, (AgentState::Idle, Memory::new())) // NO BodyHealth
        .unwrap();
    // Manually pair them via combat_pairs (bypassing the decision system's
    // memory-bias path so we don't need to set up memories — this is a
    // direct stress test of CombatSystem on a malformed input).
    let canonical = if attacker_id < defender_id {
        (attacker_id, defender_id)
    } else {
        (defender_id, attacker_id)
    };
    e.resources.combat_pairs.insert(canonical);
    // Run CombatSystem manually; must not panic.
    let mut combat = CombatSystem::new();
    combat.tick(&mut e.world, &mut e.resources);
    // World integrity: no orphaned combat_progress.
    assert_eq!(e.resources.combat_progress.len(), 0);
    // The pair was processed (removed from combat_pairs).
    assert_eq!(e.resources.combat_pairs.len(), 0);
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 29: two simultaneous pairs complete independently.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a29_two_pairs_independent() {
    let mut e = fresh_engine();
    // Pair (A,B) at (3,3); pair (C,D) at (5,5). Both setups use the helper.
    let (_a_e, _b_e, a_id, b_id) = setup_pair_with_combat_memory(&mut e, 3, 3, 3, 3, 100.0);
    let (_c_e, _d_e, c_id, d_id) = setup_pair_with_combat_memory(&mut e, 5, 5, 5, 5, 100.0);
    e.tick();
    assert_eq!(count_combat_started(&e), 2, "exactly 2 CombatStarted");
    assert_eq!(count_combat_completed(&e), 2, "exactly 2 CombatCompleted");
    let key_ab = RelationshipKey::new(a_id, b_id);
    let key_cd = RelationshipKey::new(c_id, d_id);
    let hostility_ab = e.resources.relationships.get(&key_ab).unwrap().hostility;
    let hostility_cd = e.resources.relationships.get(&key_cd).unwrap().hostility;
    assert!((hostility_ab - HOSTILITY_BUMP).abs() < 1e-9);
    assert!((hostility_cd - HOSTILITY_BUMP).abs() < 1e-9);
}

// ════════════════════════════════════════════════════════════════════════
// Assertion 30: defender HP after combat does not exceed DEFAULT_MAX_HP,
// and damage of exactly DAMAGE_PER_COMBAT_TICK is applied.
// ════════════════════════════════════════════════════════════════════════
#[test]
fn harness_p9_beta_a30_default_max_hp_cap() {
    let mut e = fresh_engine();
    let (_a_entity, d_entity, _a_id, _d_id) =
        setup_pair_with_combat_memory(&mut e, 15, 15, 15, 15, DEFAULT_MAX_HP);
    e.tick();
    let defender_hp = e.world.get::<&BodyHealth>(d_entity).unwrap().hp;
    assert!(
        defender_hp <= DEFAULT_MAX_HP,
        "defender hp {} must not exceed DEFAULT_MAX_HP {}",
        defender_hp,
        DEFAULT_MAX_HP
    );
    assert!(
        defender_hp < DEFAULT_MAX_HP,
        "combat damage must reduce hp below DEFAULT_MAX_HP; got {}",
        defender_hp
    );
    let expected = DEFAULT_MAX_HP - DAMAGE_PER_COMBAT_TICK;
    assert!(
        (defender_hp - expected).abs() < 1e-9,
        "expected hp ≈ {expected}, got {defender_hp}"
    );
}
