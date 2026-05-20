//! V7 Phase 8-γ — MemorySystem complete lifecycle harness.
//!
//! feature: p8-gamma-memory-chronicle
//! plan_attempt: 2
//! code_attempt: 1
//! seed: 42
//! lane: --full
//!
//! 16 assertions (A1-A16) proving: encode → persist & decay → cascade-flip
//! → reinforce → causal traceability.

use sim_core::causal::{CausalEvent, DecisionReason, EventId, MemoryRecallTrigger};
use sim_core::components::{
    Agent, AgentId, AgentState, BuildingBlueprint, ConstructionSite, Hunger, Memory, MemoryEntry,
    Position, Sleep, Social, TargetKind, Thirst, SALIENCE_FLOOR,
};
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::memory::{DECAY_RATE, MAX_RECENCY_TICKS, REINFORCEMENT_BOOST};

const W: u32 = 128;
const H: u32 = 128;
const SHARED_X: u32 = 6;
const SHARED_Y: u32 = 5;
const CTRL_X: u32 = 10;
const CTRL_Y: u32 = 10;
const N_DECAY_TICKS: u64 = 100;
const PHASE1_MAX: u64 = 80;
const PHASE3_MAX: u64 = 30;
const N_TICKS_TOTAL: u64 = 250;

fn fresh_engine() -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);
    engine
}

fn spawn_chronicle_agent(engine: &mut SimEngine, x: u32, y: u32) -> (hecs::Entity, AgentId) {
    let entity = engine.spawn_agent(x, y);
    let agent_id = engine.world.get::<&Agent>(entity).unwrap().id;
    engine
        .world
        .insert(
            entity,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 1.0),
                AgentState::Idle,
                Memory::new(),
            ),
        )
        .unwrap();
    (entity, agent_id)
}

fn spawn_construction_site(engine: &mut SimEngine, x: u32, y: u32, required_progress: u32) {
    let bp = BuildingBlueprint::new(1, 1, 1, required_progress);
    let site = ConstructionSite {
        blueprint: bp,
        position: Position::new(x, y),
        progress: 0,
    };
    engine.world.spawn((site,));
}

/// Returns true if `ev` is classified to the Social cascade arm.
fn is_social_arm(ev: &CausalEvent) -> bool {
    matches!(
        ev,
        CausalEvent::SocialInteractionStarted { .. }
            | CausalEvent::SocialInteractionCompleted { .. }
            | CausalEvent::AgentDecision {
                reason: DecisionReason::SocialReason,
                ..
            }
    )
}

/// Avoid unused-import warnings for MemoryRecallTrigger (used in doc/plan context).
#[allow(dead_code)]
const _MRT_CHECK: MemoryRecallTrigger = MemoryRecallTrigger::CascadeBias;

// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn harness_p8_gamma_a_complete_memory_chronicle() {
    let mut engine = fresh_engine();

    // ── Spawn 4 agents ────────────────────────────────────────────────────
    let (ent_1, id_1) = spawn_chronicle_agent(&mut engine, SHARED_X, SHARED_Y);
    let (ent_2, id_2) = spawn_chronicle_agent(&mut engine, SHARED_X, SHARED_Y);
    let (_ent_3, id_3) = spawn_chronicle_agent(&mut engine, CTRL_X, CTRL_Y);
    let (_ent_4, id_4) = spawn_chronicle_agent(&mut engine, CTRL_X, CTRL_Y);

    let shared_tile_idx = SHARED_Y * W + SHARED_X;
    let (min12, max12) = if id_1 < id_2 { (id_1, id_2) } else { (id_2, id_1) };

    // Accumulators for A12/A13 (collected across ALL phases).
    let mut agent2_recalled_total: u32 = 0;
    let mut ctrl_completed_total: u32 = 0;
    let mut ctrl_mr_total: u32 = 0;

    // ── Phase 1: run until SocialInteractionCompleted for (id_1, id_2) ───
    let mut t1: u64 = 0;
    let mut natural_completed_id: EventId = 0;

    for _ in 0..PHASE1_MAX {
        engine.tick();
        let cur = engine.resources.current_tick;

        for (_, log) in engine.resources.causal_log.iter() {
            for ev in log.iter() {
                if ev.tick() != cur {
                    continue;
                }
                match ev {
                    CausalEvent::SocialInteractionCompleted { id, agents, .. }
                        if *agents == (min12, max12) && t1 == 0 =>
                    {
                        t1 = cur;
                        natural_completed_id = *id;
                    }
                    CausalEvent::MemoryRecalled { agent, .. } if *agent == id_2 => {
                        agent2_recalled_total += 1;
                    }
                    CausalEvent::SocialInteractionCompleted { agents, .. }
                        if agents.0 == id_3
                            || agents.1 == id_3
                            || agents.0 == id_4
                            || agents.1 == id_4 =>
                    {
                        ctrl_completed_total += 1;
                    }
                    CausalEvent::AgentDecision {
                        agent,
                        reason: DecisionReason::MemoryReason,
                        ..
                    } if *agent == id_3 || *agent == id_4 => {
                        ctrl_mr_total += 1;
                    }
                    _ => {}
                }
            }
        }

        if t1 > 0 {
            break;
        }
    }
    assert!(
        t1 > 0,
        "Phase 1: SocialInteractionCompleted for (id_1,id_2) must fire within {PHASE1_MAX} ticks"
    );

    // ── A1: encode_completed_entry_valence_and_salience ───────────────────
    let entry_a1 = engine
        .world
        .get::<&Memory>(ent_1)
        .unwrap()
        .entries
        .iter()
        .find(|e| e.event_id == natural_completed_id)
        .copied()
        .expect("A1: natural_completed_id entry must be in agent_1 Memory");
    assert!(
        (entry_a1.valence - 0.7).abs() < 1e-9,
        "A1: valence={} must be 0.7",
        entry_a1.valence
    );
    assert!(
        (entry_a1.salience - 0.8).abs() < 1e-9,
        "A1: salience={} must be 0.8",
        entry_a1.salience
    );

    // ── POST-T1 seed injection ────────────────────────────────────────────
    let seed_started_id = engine.resources.issue_event_id();
    let seed_completed_id = engine.resources.issue_event_id();
    engine.resources.causal_log.push(
        shared_tile_idx,
        CausalEvent::SocialInteractionStarted {
            id: seed_started_id,
            parent: None,
            agents: (min12, max12),
            position: (SHARED_X, SHARED_Y),
            tick: t1,
        },
    );
    engine.resources.causal_log.push(
        shared_tile_idx,
        CausalEvent::SocialInteractionCompleted {
            id: seed_completed_id,
            parent: Some(seed_started_id),
            agents: (min12, max12),
            position: (SHARED_X, SHARED_Y),
            familiarity_after: 0.5,
            tick: t1,
        },
    );
    engine
        .world
        .get::<&mut Memory>(ent_1)
        .unwrap()
        .insert(MemoryEntry::new(seed_completed_id, t1, 0.9, 1.0));

    // ── A14: seed entry injected correctly (precondition guard) ───────────
    let seed_at_inject = engine
        .world
        .get::<&Memory>(ent_1)
        .unwrap()
        .entries
        .iter()
        .find(|e| e.event_id == seed_completed_id)
        .copied()
        .expect("A14: seed entry must be present after injection");
    assert!(
        (seed_at_inject.valence - 0.9).abs() < 1e-9,
        "A14: seed valence={} must be 0.9",
        seed_at_inject.valence
    );
    assert!(
        (seed_at_inject.salience - 1.0).abs() < 1e-9,
        "A14: seed salience={} must be 1.0",
        seed_at_inject.salience
    );

    // Move agent_2 to isolation tile.
    {
        let mut pos = engine.world.get::<&mut Position>(ent_2).unwrap();
        pos.x = 0;
        pos.y = 0;
    }

    // ── Phase 2: N_DECAY_TICKS ticks — watching for A15 ──────────────────
    let mut phase2_recalled_count: u32 = 0;
    for _ in 0..N_DECAY_TICKS {
        engine.tick();
        let cur = engine.resources.current_tick;
        for (_, log) in engine.resources.causal_log.iter() {
            for ev in log.iter() {
                if ev.tick() != cur {
                    continue;
                }
                match ev {
                    CausalEvent::MemoryRecalled { agent, .. } if *agent == id_1 => {
                        phase2_recalled_count += 1;
                    }
                    CausalEvent::MemoryRecalled { agent, .. } if *agent == id_2 => {
                        agent2_recalled_total += 1;
                    }
                    CausalEvent::SocialInteractionCompleted { agents, .. }
                        if agents.0 == id_3
                            || agents.1 == id_3
                            || agents.0 == id_4
                            || agents.1 == id_4 =>
                    {
                        ctrl_completed_total += 1;
                    }
                    CausalEvent::AgentDecision {
                        agent,
                        reason: DecisionReason::MemoryReason,
                        ..
                    } if *agent == id_3 || *agent == id_4 => {
                        ctrl_mr_total += 1;
                    }
                    _ => {}
                }
            }
        }
    }
    let t2 = engine.resources.current_tick;
    assert_eq!(t2, t1 + N_DECAY_TICKS, "t2 must equal t1 + N_DECAY_TICKS");

    // ── A15: no MemoryRecalled for agent_1 during Phase 2 ─────────────────
    assert_eq!(
        phase2_recalled_count, 0,
        "A15: MemoryRecalled count for agent_1 during Phase 2 must be 0, got {phase2_recalled_count}"
    );

    // ── A2: completed entry survives decay window ─────────────────────────
    let entry_a2 = engine
        .world
        .get::<&Memory>(ent_1)
        .unwrap()
        .entries
        .iter()
        .find(|e| e.event_id == natural_completed_id)
        .copied()
        .expect("A2: natural_completed_id entry must still be present at T2");

    // ── A3: exact linear decay of natural_completed entry ─────────────────
    let expected_sal_a3 = 0.8 - N_DECAY_TICKS as f64 * DECAY_RATE;
    assert!(
        (entry_a2.salience - expected_sal_a3).abs() < 1e-9,
        "A3: salience={} expected={expected_sal_a3} (0.8 - {}*{})",
        entry_a2.salience,
        N_DECAY_TICKS,
        DECAY_RATE
    );

    // ── A16: seed entry decays exactly ────────────────────────────────────
    let seed_at_t2 = engine
        .world
        .get::<&Memory>(ent_1)
        .unwrap()
        .entries
        .iter()
        .find(|e| e.event_id == seed_completed_id)
        .copied()
        .expect("A16: seed entry must be present at T2");
    let expected_sal_a16 = 1.0 - N_DECAY_TICKS as f64 * DECAY_RATE;
    assert!(
        (seed_at_t2.salience - expected_sal_a16).abs() < 1e-9,
        "A16: seed salience={} expected={expected_sal_a16} (1.0 - {}*{})",
        seed_at_t2.salience,
        N_DECAY_TICKS,
        DECAY_RATE
    );

    // ── POST-T2 setup ─────────────────────────────────────────────────────
    {
        let mut pos = engine.world.get::<&mut Position>(ent_2).unwrap();
        pos.x = SHARED_X;
        pos.y = SHARED_Y;
    }
    spawn_construction_site(&mut engine, SHARED_X, SHARED_Y, 5);

    // ── Phase 3: up to PHASE3_MAX ticks, looking for memory flip ─────────
    let mut t_flip: u64 = 0;
    let mut phase3_recalled_count: u32 = 0;
    let mut phase3_mr_count: u32 = 0;
    let mut sal_before_flip: f64 = 0.0;

    for _ in 0..PHASE3_MAX {
        // Capture seed salience BEFORE this tick (for A9 formula).
        let sal_now = engine
            .world
            .get::<&Memory>(ent_1)
            .unwrap()
            .entries
            .iter()
            .find(|e| e.event_id == seed_completed_id)
            .map(|e| e.salience)
            .unwrap_or(0.0);

        engine.tick();
        let cur = engine.resources.current_tick;

        for (_, log) in engine.resources.causal_log.iter() {
            for ev in log.iter() {
                if ev.tick() != cur {
                    continue;
                }
                match ev {
                    CausalEvent::MemoryRecalled { agent, .. } if *agent == id_1 => {
                        phase3_recalled_count += 1;
                    }
                    CausalEvent::MemoryRecalled { agent, .. } if *agent == id_2 => {
                        agent2_recalled_total += 1;
                    }
                    CausalEvent::AgentDecision {
                        agent,
                        reason: DecisionReason::MemoryReason,
                        ..
                    } if *agent == id_1 => {
                        phase3_mr_count += 1;
                        if t_flip == 0 {
                            t_flip = cur;
                            sal_before_flip = sal_now;
                        }
                    }
                    CausalEvent::AgentDecision {
                        agent,
                        reason: DecisionReason::MemoryReason,
                        ..
                    } if *agent == id_3 || *agent == id_4 => {
                        ctrl_mr_total += 1;
                    }
                    CausalEvent::SocialInteractionCompleted { agents, .. }
                        if agents.0 == id_3
                            || agents.1 == id_3
                            || agents.0 == id_4
                            || agents.1 == id_4 =>
                    {
                        ctrl_completed_total += 1;
                    }
                    _ => {}
                }
            }
        }

        if t_flip > 0 {
            break;
        }
    }
    assert!(
        t_flip > 0,
        "Phase 3: AgentDecision{{MemoryReason}} for agent_1 must fire within {PHASE3_MAX} ticks"
    );

    // ── A4: CS progress == 0 at T_flip (natural winner pre-empted) ────────
    let cs_progress = engine
        .world
        .query::<&ConstructionSite>()
        .iter()
        .find(|(_, cs)| cs.position.x == SHARED_X && cs.position.y == SHARED_Y)
        .map(|(_, cs)| cs.progress)
        .expect("A4: ConstructionSite at (SHARED_X, SHARED_Y) must still exist at T_flip");
    assert_eq!(
        cs_progress, 0,
        "A4: CS progress must be 0 at T_flip — Construction was pre-empted before any work"
    );

    // ── A5: social_delta > BIAS_FLIP_THRESHOLD=1.0 ───────────────────────
    let social_delta: f64 = {
        let mem = engine.world.get::<&Memory>(ent_1).unwrap();
        mem.entries
            .iter()
            .filter(|e| e.salience > SALIENCE_FLOOR)
            .filter_map(|e| {
                let ev = engine.resources.causal_log.lookup(e.event_id)?;
                if is_social_arm(ev) {
                    let elapsed = t_flip.saturating_sub(e.encoded_tick);
                    let recency =
                        (1.0_f64 - elapsed as f64 / MAX_RECENCY_TICKS as f64).max(0.0);
                    Some(e.valence * e.salience * recency)
                } else {
                    None
                }
            })
            .sum()
    };
    assert!(
        social_delta > 0.0,
        "A5: social_delta={social_delta:.6} must be > 0 (seed/natural entries must be in causal_log)"
    );
    assert!(
        social_delta > 1.0,
        "A5: social_delta={social_delta:.6} must exceed BIAS_FLIP_THRESHOLD=1.0"
    );

    // ── A6: exactly 1 MemoryRecalled for agent_1 in Phase 3 (to T_flip) ──
    assert_eq!(
        phase3_recalled_count, 1,
        "A6: MemoryRecalled count for agent_1 must be 1, got {phase3_recalled_count}"
    );

    // ── A7: exactly 1 AgentDecision{{MemoryReason}} for agent_1 ──────────
    assert_eq!(
        phase3_mr_count, 1,
        "A7: AgentDecision{{MemoryReason}} count for agent_1 must be 1, got {phase3_mr_count}"
    );

    // ── A8: agent_1 transitions to Seeking{Agent(id_2)} ──────────────────
    let state_a1 = *engine.world.get::<&AgentState>(ent_1).unwrap();
    assert_eq!(
        state_a1,
        AgentState::Seeking {
            target: TargetKind::Agent(id_2)
        },
        "A8: agent_1 must be Seeking{{Agent(id_2)}} at T_flip, got {state_a1:?}"
    );

    // ── A9: reinforcement boosts seed salience by REINFORCEMENT_BOOST ─────
    let seed_at_flip = engine
        .world
        .get::<&Memory>(ent_1)
        .unwrap()
        .entries
        .iter()
        .find(|e| e.event_id == seed_completed_id)
        .copied()
        .expect("A9: seed entry must still be present at T_flip");
    let expected_sal_a9 = (sal_before_flip + REINFORCEMENT_BOOST).min(1.0);
    assert!(
        (seed_at_flip.salience - expected_sal_a9).abs() < 1e-9,
        "A9: sal_after={} expected=({sal_before_flip}+{REINFORCEMENT_BOOST}).min(1.0)={expected_sal_a9}",
        seed_at_flip.salience
    );

    // ── A10: reinforcement_count == 1 ─────────────────────────────────────
    assert_eq!(
        seed_at_flip.reinforcement_count, 1,
        "A10: seed reinforcement_count must be 1"
    );

    // ── A11: 5-hop causal chain (checked BEFORE remaining ticks) ──────────
    // Hop 1: AgentDecision{MemoryReason} for id_1 at T_flip → parent = P1
    let mr_decision_parent = engine
        .resources
        .causal_log
        .iter()
        .flat_map(|(_, log)| log.iter())
        .find(|ev| {
            ev.tick() == t_flip
                && matches!(
                    ev,
                    CausalEvent::AgentDecision {
                        agent,
                        reason: DecisionReason::MemoryReason,
                        ..
                    } if *agent == id_1
                )
        })
        .and_then(|ev| ev.parent())
        .expect("A11 hop1: AgentDecision{MemoryReason} for id_1 must have parent P1");

    // Hop 2: lookup P1 → MemoryRecalled, extract recalled_event
    let recalled_event_id = engine
        .resources
        .causal_log
        .lookup(mr_decision_parent)
        .and_then(|ev| {
            if let CausalEvent::MemoryRecalled { recalled_event, .. } = ev {
                Some(*recalled_event)
            } else {
                None
            }
        })
        .expect("A11 hop2: P1 must resolve to MemoryRecalled with recalled_event");

    // Hop 3: recalled_event == seed_completed_id
    assert_eq!(
        recalled_event_id, seed_completed_id,
        "A11 hop3: recalled_event must == seed_completed_id"
    );

    // Hop 4: SocialInteractionCompleted(seed_completed_id).parent == Some(seed_started_id)
    let seed_completed_parent = engine
        .resources
        .causal_log
        .lookup(seed_completed_id)
        .and_then(|ev| ev.parent())
        .expect(
            "A11 hop4: SocialInteractionCompleted(seed_completed_id) must be in causal_log with parent",
        );
    assert_eq!(
        seed_completed_parent, seed_started_id,
        "A11 hop4: seed_completed.parent must == seed_started_id"
    );

    // Hop 5: SocialInteractionStarted(seed_started_id) exists in causal_log
    let seed_started_present = engine
        .resources
        .causal_log
        .lookup(seed_started_id)
        .map(|ev| {
            matches!(ev, CausalEvent::SocialInteractionStarted { id, .. } if *id == seed_started_id)
        })
        .unwrap_or(false);
    assert!(
        seed_started_present,
        "A11 hop5: SocialInteractionStarted(seed_started_id={seed_started_id}) must be in causal_log"
    );

    // ── Remaining ticks (T_flip+1 … N_TICKS_TOTAL) ───────────────────────
    let remaining = N_TICKS_TOTAL.saturating_sub(t_flip);
    for _ in 0..remaining {
        engine.tick();
        let cur = engine.resources.current_tick;
        for (_, log) in engine.resources.causal_log.iter() {
            for ev in log.iter() {
                if ev.tick() != cur {
                    continue;
                }
                match ev {
                    CausalEvent::MemoryRecalled { agent, .. } if *agent == id_2 => {
                        agent2_recalled_total += 1;
                    }
                    CausalEvent::AgentDecision {
                        agent,
                        reason: DecisionReason::MemoryReason,
                        ..
                    } if *agent == id_3 || *agent == id_4 => {
                        ctrl_mr_total += 1;
                    }
                    CausalEvent::SocialInteractionCompleted { agents, .. }
                        if agents.0 == id_3
                            || agents.1 == id_3
                            || agents.0 == id_4
                            || agents.1 == id_4 =>
                    {
                        ctrl_completed_total += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    // ── A12: no MemoryRecalled for agent_2 across full run ────────────────
    assert_eq!(
        agent2_recalled_total, 0,
        "A12: MemoryRecalled for agent_2 must be 0 across full run, got {agent2_recalled_total}"
    );

    // ── A13: control agents complete >= 1 social cycle, zero MemoryReason ─
    assert!(
        ctrl_completed_total >= 1,
        "A13: control agents must complete >= 1 SocialInteractionCompleted, got {ctrl_completed_total}"
    );
    assert_eq!(
        ctrl_mr_total, 0,
        "A13: MemoryReason for control agents must be 0, got {ctrl_mr_total}"
    );
}
