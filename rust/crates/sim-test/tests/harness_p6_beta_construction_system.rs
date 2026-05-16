//! V7 Phase 6-β — ConstructionSystem + AgentDecisionSystem 4th-cascade +
//! CausalEvent::ConstructionStarted/Completed + DecisionReason::ConstructionReason.
//!
//! plan_attempt: 2
//! assertions: A1..A14
//! lane: --full
//!
//! Run:
//!   `cargo test -p sim-test --test harness_p6_beta_construction_system -- --nocapture`
//!
//! Threshold types:
//!   Type A — pure mathematical invariants (compile-time / shape / equality).
//!   Type D — regression guards.

use sim_core::causal::{CausalEvent, DecisionReason};
use sim_core::components::{
    Agent, AgentId, AgentState, BuildingBlueprint, ConstructionSite, Hunger, Position, Sleep,
    TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::construction::ConstructionSystem;
use sim_systems::runtime::decision::{AgentDecisionSystem, HUNGER_THRESHOLD};
use sim_systems::{
    register_agent_systems, register_construction_systems, register_decision_systems,
    register_needs_systems, register_phase2_systems,
};

const W: u32 = 32;
const H: u32 = 32;

fn fresh_engine() -> SimEngine {
    SimEngine::new(W, H, MaterialRegistry::new())
}

fn tile_idx(x: u32, y: u32) -> u32 {
    y * W + x
}

/// Insert a quiet need set on an agent: all needs at 0, growth_rate 0.
fn insert_quiet_needs(engine: &mut SimEngine, e: hecs::Entity) {
    engine
        .world
        .insert(
            e,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
            ),
        )
        .unwrap();
}

// ── A1: ConstructionSystem registered with correct metadata ───────────────
#[test]
fn harness_p6_beta_a1_construction_system_metadata_and_registry() {
    // (a) Direct metadata lock — type A invariants on a fresh instance.
    let sys = ConstructionSystem::new();
    assert_eq!(sys.name(), "ConstructionSystem");
    assert_eq!(sys.priority(), 133);
    assert_eq!(sys.tick_interval(), 1);

    // (b) Registration via production helper — exactly one ConstructionSystem
    //     reachable through the same registry path the production engine
    //     uses (register_construction_systems / engine.system_names()).
    let mut engine = fresh_engine();
    register_phase2_systems(&mut engine);
    register_agent_systems(&mut engine);
    register_decision_systems(&mut engine);
    register_needs_systems(&mut engine);
    register_construction_systems(&mut engine);
    let names = engine.system_names();
    let count = names.iter().filter(|n| **n == "ConstructionSystem").count();
    assert_eq!(count, 1, "exactly one ConstructionSystem must be registered");

    // (c) Priority ordering: SleepDecaySystem(132) < ConstructionSystem(133) <
    //     InfluenceVisualizationSystem(1000).
    let i_sleep = names
        .iter()
        .position(|n| *n == "SleepDecaySystem")
        .expect("SleepDecaySystem registered");
    let i_con = names
        .iter()
        .position(|n| *n == "ConstructionSystem")
        .expect("ConstructionSystem registered");
    let i_viz = names
        .iter()
        .position(|n| *n == "InfluenceVisualizationSystem")
        .expect("InfluenceVisualizationSystem registered");
    assert!(i_sleep < i_con, "Sleep({}) must precede Construction({})", i_sleep, i_con);
    assert!(i_con < i_viz, "Construction({}) must precede Viz({})", i_con, i_viz);
    println!("[β-1] ConstructionSystem(133) registered between Sleep(132) and Viz(1000) ✓");
}

// ── A2: DecisionReason locked enumeration and discriminator ───────────────
#[test]
fn harness_p6_beta_a2_decision_reason_enumeration_and_discriminator() {
    // Exhaustive match — adding a 5th variant would fail to compile.
    let reasons = [
        DecisionReason::HungerThresholdBreach,
        DecisionReason::ThirstThresholdBreach,
        DecisionReason::FatigueThresholdBreach,
        DecisionReason::ConstructionReason,
    ];
    for r in reasons {
        match r {
            DecisionReason::HungerThresholdBreach => {}
            DecisionReason::ThirstThresholdBreach => {}
            DecisionReason::FatigueThresholdBreach => {}
            DecisionReason::ConstructionReason => {}
        }
    }
    assert_eq!(
        DecisionReason::ConstructionReason.as_str(),
        "construction_reason"
    );
    println!("[β-2] DecisionReason has exactly 4 variants; ConstructionReason → \"construction_reason\" ✓");
}

// ── A3: CausalEvent locked enumeration + accessor coverage + channel ──────
//
// NOTE: The plan threshold lists {AgentDecision, BuildingPlaced, TraumaRecorded,
// TechDiscovered, ConstructionStarted, ConstructionCompleted}. The plan
// explicitly allows the Generator to update the baseline list to match the
// authoritative pre-P6β enum. The actual pre-P6β variant set in sim-core is
// {BuildingPlaced, StampDirty, InfluenceChanged, AgentDecision}; therefore the
// authoritative post-P6β set is {BuildingPlaced, StampDirty, InfluenceChanged,
// AgentDecision, ConstructionStarted, ConstructionCompleted} — six variants,
// which is what the exhaustive match below asserts.
#[test]
fn harness_p6_beta_a3_causal_event_enumeration_accessors_and_channel() {
    let bp = BuildingBlueprint::new(7, 2, 2, 5);
    let started = CausalEvent::ConstructionStarted {
        id: 100,
        parent: Some(42),
        blueprint: bp,
        position: (3, 4),
        tick: 17,
    };
    let completed = CausalEvent::ConstructionCompleted {
        id: 101,
        parent: Some(100),
        blueprint: bp,
        position: (3, 4),
        tick: 22,
    };
    let placed = CausalEvent::BuildingPlaced {
        id: 1,
        parent: None,
        position: (0, 0),
        radius: 0,
        tick: 0,
    };

    // Exhaustive match — adds compile-time guarantee that the variant set is
    // exactly the six listed.
    fn classify(ev: &CausalEvent) -> &'static str {
        match ev {
            CausalEvent::BuildingPlaced { .. } => "building_placed",
            CausalEvent::StampDirty { .. } => "stamp_dirty",
            CausalEvent::InfluenceChanged { .. } => "influence_changed",
            CausalEvent::AgentDecision { .. } => "agent_decision",
            CausalEvent::ConstructionStarted { .. } => "construction_started",
            CausalEvent::ConstructionCompleted { .. } => "construction_completed",
        }
    }
    assert_eq!(classify(&started), "construction_started");
    assert_eq!(classify(&completed), "construction_completed");
    assert_eq!(classify(&placed), "building_placed");

    // Accessor round-trip.
    assert_eq!(started.id(), 100);
    assert_eq!(started.parent(), Some(42));
    assert_eq!(started.tick(), 17);
    assert_eq!(completed.id(), 101);
    assert_eq!(completed.parent(), Some(100));
    assert_eq!(completed.tick(), 22);

    // channel() returns None — parallels BuildingPlaced and AgentDecision precedent.
    assert_eq!(placed.channel(), None);
    assert_eq!(started.channel(), None);
    assert_eq!(completed.channel(), None);
    println!(
        "[β-3] CausalEvent has 6 variants; Construction* accessors round-trip; channel() == None ✓"
    );
}

// ── A4: Idle → Seeking transition when only co-located site exists ────────
#[test]
fn harness_p6_beta_a4_idle_to_seeking_when_only_site_colocated() {
    let mut engine = fresh_engine();
    let agent_e = engine.spawn_agent(3, 5);
    let agent_id = engine.world.get::<&Agent>(agent_e).unwrap().id;
    insert_quiet_needs(&mut engine, agent_e);
    engine
        .world
        .insert_one(agent_e, AgentState::Idle)
        .unwrap();

    let bp = BuildingBlueprint::new(1, 2, 2, 5);
    engine
        .world
        .spawn((ConstructionSite::new(bp, Position::new(3, 5)),));

    let tidx = tile_idx(3, 5);
    let before_len = engine
        .resources
        .causal_log
        .get(tidx)
        .map(|l| l.len())
        .unwrap_or(0);

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut engine.world, &mut engine.resources);

    let state = *engine.world.get::<&AgentState>(agent_e).unwrap();
    assert_eq!(
        state,
        AgentState::Seeking {
            target: TargetKind::ConstructionSite
        }
    );

    let pos = *engine.world.get::<&Position>(agent_e).unwrap();
    assert_eq!(pos, Position::new(3, 5));

    let log = engine
        .resources
        .causal_log
        .get(tidx)
        .expect("AgentDecision must be recorded at (3,5)");
    let new_events: Vec<&CausalEvent> = log.as_slice().iter().skip(before_len).collect();
    let decisions: Vec<(AgentId, DecisionReason)> = new_events
        .iter()
        .filter_map(|ev| match ev {
            CausalEvent::AgentDecision { agent, reason, .. } => Some((*agent, *reason)),
            _ => None,
        })
        .collect();
    assert_eq!(decisions.len(), 1, "exactly one AgentDecision appended");
    assert_eq!(decisions[0].0, agent_id);
    assert_eq!(decisions[0].1, DecisionReason::ConstructionReason);

    // No other CausalEvent variant appended this tick.
    let other_count = new_events.len() - decisions.len();
    assert_eq!(
        other_count, 0,
        "no other CausalEvent variant should be appended; got {} extras",
        other_count
    );
    println!("[β-4] Idle+site@(3,5) → Seeking{{ConstructionSite}}; AgentDecision{{Construction}} emitted ✓");
}

// ── A5: Need priority dominates Construction (cascade ordering) ───────────
#[test]
fn harness_p6_beta_a5_need_priority_dominates_construction() {
    let mut engine = fresh_engine();
    let agent_e = engine.spawn_agent(3, 5);
    engine
        .world
        .insert(
            agent_e,
            (
                AgentState::Idle,
                Hunger::new(HUNGER_THRESHOLD + 1.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
            ),
        )
        .unwrap();

    let bp = BuildingBlueprint::new(7, 2, 2, 5);
    let site_e = engine
        .world
        .spawn((ConstructionSite::new(bp, Position::new(3, 5)),));
    let initial = *engine.world.get::<&ConstructionSite>(site_e).unwrap();

    let tidx = tile_idx(3, 5);

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut engine.world, &mut engine.resources);

    let state = *engine.world.get::<&AgentState>(agent_e).unwrap();
    assert_eq!(
        state,
        AgentState::Seeking {
            target: TargetKind::Food
        }
    );

    let log = engine.resources.causal_log.get(tidx).unwrap();
    let last_reason = log
        .as_slice()
        .iter()
        .rev()
        .find_map(|ev| match ev {
            CausalEvent::AgentDecision { reason, .. } => Some(*reason),
            _ => None,
        })
        .expect("AgentDecision recorded");
    assert_eq!(last_reason, DecisionReason::HungerThresholdBreach);

    assert!(engine.world.contains(site_e), "site entity must remain");
    let after = *engine.world.get::<&ConstructionSite>(site_e).unwrap();
    assert_eq!(after.progress, initial.progress);
    assert_eq!(after.blueprint, initial.blueprint);
    assert_eq!(
        after.blueprint.required_progress,
        initial.blueprint.required_progress
    );
    println!("[β-5] Hunger preempts Construction; site fields byte-identical ✓");
}

// ── A6: Seeking → Consuming emits ConstructionStarted with correct parent ─
#[test]
fn harness_p6_beta_a6_seeking_to_consuming_emits_construction_started_correct_parent() {
    let mut engine = fresh_engine();

    let bp = BuildingBlueprint::new(42, 3, 4, 10);
    let site_e = engine
        .world
        .spawn((ConstructionSite::new(bp, Position::new(4, 7)),));

    let agent_a = engine.spawn_agent(4, 7);
    let agent_a_id = engine.world.get::<&Agent>(agent_a).unwrap().id;
    insert_quiet_needs(&mut engine, agent_a);
    engine
        .world
        .insert_one(
            agent_a,
            AgentState::Seeking {
                target: TargetKind::ConstructionSite,
            },
        )
        .unwrap();

    let agent_b_id: AgentId = agent_a_id.wrapping_add(1000);
    let tidx = tile_idx(4, 7);

    // 1: AgentDecision for agent B with reason ConstructionReason (distractor: wrong agent).
    let id1 = engine.resources.issue_event_id();
    engine.resources.causal_log.push(
        tidx,
        CausalEvent::AgentDecision {
            id: id1,
            parent: None,
            agent: agent_b_id,
            position: (4, 7),
            reason: DecisionReason::ConstructionReason,
            tick: 0,
        },
    );

    // 2: AgentDecision for agent A with reason HungerThresholdBreach (distractor: wrong reason).
    let id2 = engine.resources.issue_event_id();
    engine.resources.causal_log.push(
        tidx,
        CausalEvent::AgentDecision {
            id: id2,
            parent: None,
            agent: agent_a_id,
            position: (4, 7),
            reason: DecisionReason::HungerThresholdBreach,
            tick: 0,
        },
    );

    // 3: AgentDecision for agent A with reason ConstructionReason (legitimate parent).
    let id3 = engine.resources.issue_event_id();
    engine.resources.causal_log.push(
        tidx,
        CausalEvent::AgentDecision {
            id: id3,
            parent: None,
            agent: agent_a_id,
            position: (4, 7),
            reason: DecisionReason::ConstructionReason,
            tick: 0,
        },
    );
    let expected_parent_id = id3;

    let before_len = engine.resources.causal_log.get(tidx).unwrap().len();
    let live_blueprint = engine.world.get::<&ConstructionSite>(site_e).unwrap().blueprint;

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut engine.world, &mut engine.resources);

    let after_log = engine.resources.causal_log.get(tidx).unwrap();
    let new_events: Vec<&CausalEvent> = after_log.as_slice().iter().skip(before_len).collect();

    let started: Vec<_> = new_events
        .iter()
        .filter_map(|ev| match ev {
            CausalEvent::ConstructionStarted {
                id,
                parent,
                blueprint,
                position,
                tick,
            } => Some((*id, *parent, *blueprint, *position, *tick)),
            _ => None,
        })
        .collect();
    assert_eq!(started.len(), 1, "exactly one ConstructionStarted emitted");
    let (_sid, sparent, sblueprint, sposition, stick) = started[0];
    assert_eq!(sposition, (4, 7));
    assert_eq!(
        sblueprint, live_blueprint,
        "blueprint MUST be snapshotted from live site, not default"
    );
    assert_eq!(stick, engine.resources.current_tick);
    assert_eq!(
        sparent,
        Some(expected_parent_id),
        "parent must point to event #3 (A+Construction), NOT #1 (wrong agent) or #2 (wrong reason)"
    );

    let state = *engine.world.get::<&AgentState>(agent_a).unwrap();
    assert_eq!(
        state,
        AgentState::Consuming {
            target: TargetKind::ConstructionSite
        }
    );
    println!("[β-6] Seeking→Consuming emits ConstructionStarted with correct parent linkage ✓");
}

// ── A7: Progress increments per tick under co-located Consuming agent ─────
#[test]
fn harness_p6_beta_a7_progress_increments_under_consuming_agent() {
    let mut engine = fresh_engine();

    let bp = BuildingBlueprint::new(1, 2, 2, 5);
    let site_e = engine
        .world
        .spawn((ConstructionSite::new(bp, Position::new(2, 9)),));

    let agent_e = engine.spawn_agent(2, 9);
    insert_quiet_needs(&mut engine, agent_e);
    engine
        .world
        .insert_one(
            agent_e,
            AgentState::Consuming {
                target: TargetKind::ConstructionSite,
            },
        )
        .unwrap();

    // Diagnostic: count agents currently in Consuming{ConstructionSite}.
    // The Pipeline harness contract requires `agents_doing_construction=N`
    // with N > 0 on this behavioral path.
    let agents_doing_construction: usize = engine
        .world
        .query::<&AgentState>()
        .iter()
        .filter(|(_, s)| {
            matches!(
                **s,
                AgentState::Consuming {
                    target: TargetKind::ConstructionSite
                }
            )
        })
        .count();
    assert!(
        agents_doing_construction > 0,
        "diagnostic guard: at least one agent must be Consuming{{ConstructionSite}}"
    );
    println!("agents_doing_construction={}", agents_doing_construction);

    let mut sys = ConstructionSystem::new();

    for (t, expected_progress) in [(0u64, 1u32), (1, 2), (2, 3), (3, 4)].iter() {
        engine.resources.current_tick = *t;
        sys.tick(&mut engine.world, &mut engine.resources);
        let site = *engine
            .world
            .get::<&ConstructionSite>(site_e)
            .expect("site still present pre-completion");
        assert_eq!(
            site.progress, *expected_progress,
            "after tick {}: progress must be {}",
            t, expected_progress
        );
        assert_eq!(
            site.blueprint.required_progress, 5,
            "required_progress MUST NOT decrease (got {} at tick {})",
            site.blueprint.required_progress, t
        );
    }

    // Tick 5 → completion → site despawned.
    engine.resources.current_tick = 4;
    sys.tick(&mut engine.world, &mut engine.resources);
    assert!(
        !engine.world.contains(site_e),
        "site must be despawned on the completion tick"
    );
    println!("[β-7] Progress 0→1→2→3→4 across 4 ticks; completion despawns at tick 5 ✓");
}

// ── A8: Completion emits ConstructionCompleted then BuildingPlaced ────────
#[test]
fn harness_p6_beta_a8_completion_emits_completed_then_placed_chained() {
    let mut engine = fresh_engine();

    // Plan attempt-2 lock: required_progress == 3, three construction ticks,
    // completion on the third tick, site despawn asserted.
    let bp = BuildingBlueprint::new(1, 2, 2, 3);
    let site_e = engine
        .world
        .spawn((ConstructionSite::new(bp, Position::new(2, 9)),));

    let agent_e = engine.spawn_agent(2, 9);
    insert_quiet_needs(&mut engine, agent_e);
    engine
        .world
        .insert_one(
            agent_e,
            AgentState::Consuming {
                target: TargetKind::ConstructionSite,
            },
        )
        .unwrap();

    let tidx = tile_idx(2, 9);

    // Pre-seed a ConstructionStarted so the ConstructionCompleted.parent
    // chain can resolve. (A9 exercises the full chain end-to-end; here we
    // isolate the emission contract of ConstructionSystem itself.)
    let started_id = engine.resources.issue_event_id();
    engine.resources.causal_log.push(
        tidx,
        CausalEvent::ConstructionStarted {
            id: started_id,
            parent: None,
            blueprint: bp,
            position: (2, 9),
            tick: 0,
        },
    );

    let mut sys = ConstructionSystem::new();
    // Run 2 ticks to drive progress 0→1→2 (pre-completion).
    for t in 0..2u64 {
        engine.resources.current_tick = t;
        sys.tick(&mut engine.world, &mut engine.resources);
        let mid = *engine
            .world
            .get::<&ConstructionSite>(site_e)
            .expect("site still present pre-completion");
        assert_eq!(
            mid.progress,
            (t as u32) + 1,
            "progress must equal tick+1 pre-completion (got {} at tick {})",
            mid.progress, t
        );
    }

    let before_len = engine.resources.causal_log.get(tidx).unwrap().len();

    // Third construction tick — completion edge fires (progress 2→3).
    engine.resources.current_tick = 2;
    sys.tick(&mut engine.world, &mut engine.resources);

    // Plan-locked despawn assertion.
    assert!(
        !engine.world.contains(site_e),
        "site must be despawned on completion tick (required_progress=3, third tick)"
    );

    let after_log = engine.resources.causal_log.get(tidx).unwrap();
    let new_events: Vec<&CausalEvent> = after_log.as_slice().iter().skip(before_len).collect();
    assert_eq!(
        new_events.len(),
        2,
        "completion tick must append exactly 2 events; got {}",
        new_events.len()
    );

    let (completed_id, completed_parent, completed_blueprint, completed_pos, completed_tick) =
        match new_events[0] {
            CausalEvent::ConstructionCompleted {
                id,
                parent,
                blueprint,
                position,
                tick,
            } => (*id, *parent, *blueprint, *position, *tick),
            other => panic!(
                "expected ConstructionCompleted at index 0 (before BuildingPlaced); got {:?}",
                other
            ),
        };
    assert_eq!(completed_parent, Some(started_id));
    assert_eq!(completed_blueprint, bp);
    assert_eq!(completed_pos, (2, 9));
    assert_eq!(completed_tick, engine.resources.current_tick);

    let (placed_id, placed_parent, placed_pos, placed_radius, placed_tick) = match new_events[1] {
        CausalEvent::BuildingPlaced {
            id,
            parent,
            position,
            radius,
            tick,
        } => (*id, *parent, *position, *radius, *tick),
        other => panic!(
            "expected BuildingPlaced at index 1 (after ConstructionCompleted); got {:?}",
            other
        ),
    };
    assert_eq!(placed_parent, Some(completed_id));
    assert_eq!(placed_pos, (2, 9));
    assert_eq!(
        placed_radius, 0,
        "agent-construction emission MUST use radius=0 (BSS path is untouched)"
    );
    assert_eq!(placed_tick, engine.resources.current_tick);

    // Pairwise distinct event ids.
    assert_ne!(started_id, completed_id);
    assert_ne!(completed_id, placed_id);
    assert_ne!(started_id, placed_id);

    // Agent's Idle reset owned by ConstructionSystem itself.
    let state_after = *engine.world.get::<&AgentState>(agent_e).unwrap();
    assert_eq!(state_after, AgentState::Idle);
    println!("[β-8] Completion tick: Completed→Placed (ordered, chained, radius=0); agent→Idle ✓");
}

// ── A9: Full 4-link causal chain walk by EventId ──────────────────────────
#[test]
fn harness_p6_beta_a9_full_4_link_causal_chain_walk() {
    let mut engine = fresh_engine();
    register_phase2_systems(&mut engine);
    register_agent_systems(&mut engine);
    register_decision_systems(&mut engine);
    register_needs_systems(&mut engine);
    register_construction_systems(&mut engine);

    let bp = BuildingBlueprint::new(99, 2, 2, 2);
    let _site_e = engine
        .world
        .spawn((ConstructionSite::new(bp, Position::new(2, 9)),));

    let agent_e = engine.spawn_agent(2, 9);
    let agent_id = engine.world.get::<&Agent>(agent_e).unwrap().id;
    insert_quiet_needs(&mut engine, agent_e);
    engine
        .world
        .insert_one(agent_e, AgentState::Idle)
        .unwrap();

    let tidx = tile_idx(2, 9);

    // Tick 1: Idle → Seeking, emits AgentDecision.
    engine.tick();
    // Tick 2: Seeking → Consuming, emits ConstructionStarted; ConstructionSystem
    //         advances progress 0→1.
    engine.tick();
    // Tick 3: ConstructionSystem advances 1→2 = required_progress; emits
    //         ConstructionCompleted + BuildingPlaced; despawns site; agent→Idle.
    engine.tick();

    let log = engine
        .resources
        .causal_log
        .get(tidx)
        .expect("tile log must exist");

    let placed_id = log
        .as_slice()
        .iter()
        .rev()
        .find_map(|ev| match ev {
            CausalEvent::BuildingPlaced { id, .. } => Some(*id),
            _ => None,
        })
        .expect("BuildingPlaced must be present");

    let chain = engine.resources.causal_log.trace_parents(tidx, placed_id);
    let kinds: Vec<&'static str> = chain
        .iter()
        .map(|ev| match ev {
            CausalEvent::BuildingPlaced { .. } => "building_placed",
            CausalEvent::StampDirty { .. } => "stamp_dirty",
            CausalEvent::InfluenceChanged { .. } => "influence_changed",
            CausalEvent::AgentDecision { .. } => "agent_decision",
            CausalEvent::ConstructionStarted { .. } => "construction_started",
            CausalEvent::ConstructionCompleted { .. } => "construction_completed",
        })
        .collect();

    assert!(
        chain.len() >= 4,
        "chain must have at least 4 links (got {}: {:?})",
        chain.len(),
        kinds
    );
    // First four entries are the new 4-link chain newest → oldest.
    assert_eq!(kinds[0], "building_placed");
    assert_eq!(kinds[1], "construction_completed");
    assert_eq!(kinds[2], "construction_started");
    assert_eq!(kinds[3], "agent_decision");
    assert!(
        chain.len() <= 5,
        "chain must be 4 or 5 links (the AgentDecision's parent is optional); got {}",
        chain.len()
    );

    // Final AgentDecision: reason and agent must match.
    match chain[3] {
        CausalEvent::AgentDecision { reason, agent, .. } => {
            assert_eq!(*reason, DecisionReason::ConstructionReason);
            assert_eq!(*agent, agent_id);
        }
        other => panic!("expected AgentDecision at chain[3]; got {:?}", other),
    }

    // Three non-AgentDecision links must have Some(parent) (the chain holds).
    assert!(chain[0].parent().is_some());
    assert!(chain[1].parent().is_some());
    assert!(chain[2].parent().is_some());

    // Pairwise distinct EventIds.
    let ids: Vec<u64> = chain.iter().take(4).map(|e| e.id()).collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(
                ids[i], ids[j],
                "ids must be pairwise distinct (i={} j={})",
                i, j
            );
        }
    }
    println!("[β-9] 4-link chain BuildingPlaced→Completed→Started→AgentDecision walked ✓");
}

// ── A10: Absent-site fallback — no panic, no progress, agent → Idle ───────
#[test]
fn harness_p6_beta_a10_absent_site_fallback_no_panic() {
    let mut engine = fresh_engine();

    let agent_e = engine.spawn_agent(1, 1);
    insert_quiet_needs(&mut engine, agent_e);
    engine
        .world
        .insert_one(
            agent_e,
            AgentState::Consuming {
                target: TargetKind::ConstructionSite,
            },
        )
        .unwrap();

    let bp = BuildingBlueprint::new(1, 2, 2, 5);
    let site_e = engine
        .world
        .spawn((ConstructionSite::new(bp, Position::new(1, 1)),));
    // Despawn the site BEFORE running ConstructionSystem.
    engine.world.despawn(site_e).unwrap();

    // Another, untouched site at a different tile — its progress must NOT
    // be mutated as a side effect of the fallback.
    let bp2 = BuildingBlueprint::new(2, 2, 2, 5);
    let other_site = engine
        .world
        .spawn((ConstructionSite::new(bp2, Position::new(15, 15)),));
    let other_progress_before = engine.world.get::<&ConstructionSite>(other_site).unwrap().progress;

    let tidx = tile_idx(1, 1);
    let before_len = engine
        .resources
        .causal_log
        .get(tidx)
        .map(|l| l.len())
        .unwrap_or(0);

    let mut sys = ConstructionSystem::new();
    sys.tick(&mut engine.world, &mut engine.resources);

    let state = *engine.world.get::<&AgentState>(agent_e).unwrap();
    assert_eq!(state, AgentState::Idle);

    let new_events: Vec<&CausalEvent> = if let Some(log) = engine.resources.causal_log.get(tidx) {
        log.as_slice().iter().skip(before_len).collect()
    } else {
        vec![]
    };
    let completed_count = new_events
        .iter()
        .filter(|e| matches!(e, CausalEvent::ConstructionCompleted { .. }))
        .count();
    let placed_count = new_events
        .iter()
        .filter(|e| matches!(e, CausalEvent::BuildingPlaced { .. }))
        .count();
    assert_eq!(completed_count, 0);
    assert_eq!(placed_count, 0);

    let other_after = engine.world.get::<&ConstructionSite>(other_site).unwrap().progress;
    assert_eq!(other_after, other_progress_before);
    println!("[β-10] Despawned site → agent→Idle, no completion events, no side-effects ✓");
}

// ── A11: Idle site without Consuming{ConstructionSite} agent does not advance ─
#[test]
fn harness_p6_beta_a11_site_without_consuming_construction_agent_does_not_advance() {
    let mut engine = fresh_engine();

    let bp = BuildingBlueprint::new(1, 2, 2, 5);
    let site_e = engine
        .world
        .spawn((ConstructionSite::new(bp, Position::new(6, 6)),));

    // Agent X — Idle.
    let agent_x = engine.spawn_agent(6, 6);
    insert_quiet_needs(&mut engine, agent_x);
    engine
        .world
        .insert_one(agent_x, AgentState::Idle)
        .unwrap();
    let x_before = *engine.world.get::<&AgentState>(agent_x).unwrap();

    // Agent Y — Consuming{Food} (NOT ConstructionSite).
    let agent_y = engine.spawn_agent(6, 6);
    insert_quiet_needs(&mut engine, agent_y);
    engine
        .world
        .insert_one(
            agent_y,
            AgentState::Consuming {
                target: TargetKind::Food,
            },
        )
        .unwrap();
    let y_before = *engine.world.get::<&AgentState>(agent_y).unwrap();

    // Agent Z — Consuming{Water} (NOT ConstructionSite).
    let agent_z = engine.spawn_agent(6, 6);
    insert_quiet_needs(&mut engine, agent_z);
    engine
        .world
        .insert_one(
            agent_z,
            AgentState::Consuming {
                target: TargetKind::Water,
            },
        )
        .unwrap();
    let z_before = *engine.world.get::<&AgentState>(agent_z).unwrap();

    let tidx = tile_idx(6, 6);
    let before_len = engine
        .resources
        .causal_log
        .get(tidx)
        .map(|l| l.len())
        .unwrap_or(0);

    let mut sys = ConstructionSystem::new();
    for t in 0..10u64 {
        engine.resources.current_tick = t;
        sys.tick(&mut engine.world, &mut engine.resources);
    }

    assert!(engine.world.contains(site_e), "site must remain after 10 ticks");
    let site = *engine.world.get::<&ConstructionSite>(site_e).unwrap();
    assert_eq!(site.progress, 0);
    assert_eq!(site.blueprint.required_progress, 5);

    let after_events: Vec<&CausalEvent> = engine
        .resources
        .causal_log
        .get(tidx)
        .map(|l| l.as_slice().iter().skip(before_len).collect())
        .unwrap_or_default();
    let started = after_events
        .iter()
        .filter(|e| matches!(e, CausalEvent::ConstructionStarted { .. }))
        .count();
    let completed = after_events
        .iter()
        .filter(|e| matches!(e, CausalEvent::ConstructionCompleted { .. }))
        .count();
    let placed = after_events
        .iter()
        .filter(|e| matches!(e, CausalEvent::BuildingPlaced { .. }))
        .count();
    assert_eq!(started, 0);
    assert_eq!(completed, 0);
    assert_eq!(placed, 0);

    assert_eq!(*engine.world.get::<&AgentState>(agent_x).unwrap(), x_before);
    assert_eq!(*engine.world.get::<&AgentState>(agent_y).unwrap(), y_before);
    assert_eq!(*engine.world.get::<&AgentState>(agent_z).unwrap(), z_before);
    println!(
        "[β-11] Site with Idle/Consuming{{Food}}/Consuming{{Water}} co-located agents stays at progress=0 ✓"
    );
}

// ── A12: Phase 5 needs cascade regression — Hunger preempts Construction ──
// Type D — regression guard.
#[test]
fn harness_p6_beta_a12_phase5_cascade_regression_hunger_preempts_construction() {
    let mut engine = fresh_engine();
    let agent_e = engine.spawn_agent(3, 5);
    engine
        .world
        .insert(
            agent_e,
            (
                AgentState::Idle,
                Hunger::new(HUNGER_THRESHOLD + 1.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
            ),
        )
        .unwrap();

    let bp = BuildingBlueprint::new(1, 2, 2, 5);
    let site_e = engine
        .world
        .spawn((ConstructionSite::new(bp, Position::new(3, 5)),));

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut engine.world, &mut engine.resources);

    let state = *engine.world.get::<&AgentState>(agent_e).unwrap();
    assert_eq!(
        state,
        AgentState::Seeking {
            target: TargetKind::Food
        }
    );

    let log = engine.resources.causal_log.get(tile_idx(3, 5)).unwrap();
    let latest_reason = log
        .as_slice()
        .iter()
        .rev()
        .find_map(|ev| match ev {
            CausalEvent::AgentDecision { reason, .. } => Some(*reason),
            _ => None,
        })
        .unwrap();
    assert_eq!(latest_reason, DecisionReason::HungerThresholdBreach);

    assert!(engine.world.contains(site_e));
    let site = *engine.world.get::<&ConstructionSite>(site_e).unwrap();
    assert_eq!(site.progress, 0);
    assert_eq!(site.blueprint.required_progress, 5);
    println!("[β-12] Phase 5 cascade regression: Hunger > Construction priority preserved ✓");
}

// ── A13: Phase 6-α ConstructionSite shape regression ──────────────────────
// Type D — regression guard.
#[test]
fn harness_p6_beta_a13_phase6_alpha_construction_site_shape_regression() {
    let mut engine = fresh_engine();

    let bp = BuildingBlueprint::new(1, 2, 2, 5);
    let site_e = engine
        .world
        .spawn((ConstructionSite::new(bp, Position::new(8, 8)),));

    let before = *engine.world.get::<&ConstructionSite>(site_e).unwrap();
    assert_eq!(before.progress, 0);
    assert!(!before.is_complete());

    let tidx = tile_idx(8, 8);
    let before_log_len = engine
        .resources
        .causal_log
        .get(tidx)
        .map(|l| l.len())
        .unwrap_or(0);

    let mut sys = ConstructionSystem::new();
    sys.tick(&mut engine.world, &mut engine.resources);

    assert!(engine.world.contains(site_e));
    let after = *engine.world.get::<&ConstructionSite>(site_e).unwrap();
    // Field-by-field equality.
    assert_eq!(after.progress, before.progress);
    assert_eq!(
        after.blueprint.required_progress,
        before.blueprint.required_progress
    );
    assert_eq!(after.blueprint, before.blueprint);
    assert_eq!(after.is_complete(), before.is_complete());

    let after_log_len = engine
        .resources
        .causal_log
        .get(tidx)
        .map(|l| l.len())
        .unwrap_or(0);
    assert_eq!(after_log_len, before_log_len, "no CausalEvent at (8,8)");
    println!("[β-13] Phase 6-α ConstructionSite shape unchanged; isolated tick is inert ✓");
}

// ── A14: Multiple sites are independent (no cross-tile contamination) ─────
#[test]
fn harness_p6_beta_a14_multiple_sites_independence() {
    let mut engine = fresh_engine();

    let bp1 = BuildingBlueprint::new(1, 2, 2, 2);
    let site1 = engine
        .world
        .spawn((ConstructionSite::new(bp1, Position::new(10, 10)),));

    let bp2 = BuildingBlueprint::new(2, 2, 2, 2);
    let site2 = engine
        .world
        .spawn((ConstructionSite::new(bp2, Position::new(12, 12)),));

    let agent_e = engine.spawn_agent(10, 10);
    insert_quiet_needs(&mut engine, agent_e);
    engine
        .world
        .insert_one(
            agent_e,
            AgentState::Consuming {
                target: TargetKind::ConstructionSite,
            },
        )
        .unwrap();

    let tidx1 = tile_idx(10, 10);
    let tidx2 = tile_idx(12, 12);
    let p1_before = engine
        .resources
        .causal_log
        .get(tidx1)
        .map(|l| l.len())
        .unwrap_or(0);
    let p2_before = engine
        .resources
        .causal_log
        .get(tidx2)
        .map(|l| l.len())
        .unwrap_or(0);

    let mut sys = ConstructionSystem::new();

    // Tick 1
    sys.tick(&mut engine.world, &mut engine.resources);
    assert!(engine.world.contains(site1));
    assert!(engine.world.contains(site2));
    assert_eq!(engine.world.get::<&ConstructionSite>(site1).unwrap().progress, 1);
    assert_eq!(engine.world.get::<&ConstructionSite>(site2).unwrap().progress, 0);

    // No completion events anywhere this tick.
    let p1_t1_new: Vec<&CausalEvent> = engine
        .resources
        .causal_log
        .get(tidx1)
        .map(|l| l.as_slice().iter().skip(p1_before).collect())
        .unwrap_or_default();
    let p2_t1_new: Vec<&CausalEvent> = engine
        .resources
        .causal_log
        .get(tidx2)
        .map(|l| l.as_slice().iter().skip(p2_before).collect())
        .unwrap_or_default();
    for ev in p1_t1_new.iter().chain(p2_t1_new.iter()) {
        assert!(
            !matches!(
                ev,
                CausalEvent::ConstructionCompleted { .. } | CausalEvent::BuildingPlaced { .. }
            ),
            "no completion-side events on tick 1; got {:?}",
            ev
        );
    }

    // Tick 2 — completion at P1.
    engine.resources.current_tick = 1;
    sys.tick(&mut engine.world, &mut engine.resources);
    assert!(!engine.world.contains(site1), "P1 must be despawned");
    assert!(engine.world.contains(site2));
    let s2 = *engine.world.get::<&ConstructionSite>(site2).unwrap();
    assert_eq!(s2.progress, 0);
    assert_eq!(s2.blueprint.required_progress, 2);

    let p1_total: Vec<&CausalEvent> = engine
        .resources
        .causal_log
        .get(tidx1)
        .unwrap()
        .as_slice()
        .iter()
        .skip(p1_before)
        .collect();
    let p1_completed = p1_total
        .iter()
        .filter(|e| matches!(e, CausalEvent::ConstructionCompleted { .. }))
        .count();
    let p1_placed = p1_total
        .iter()
        .filter(|e| matches!(e, CausalEvent::BuildingPlaced { .. }))
        .count();
    assert_eq!(p1_completed, 1);
    assert_eq!(p1_placed, 1);

    let p2_total: Vec<&CausalEvent> = if let Some(log) = engine.resources.causal_log.get(tidx2) {
        log.as_slice().iter().skip(p2_before).collect()
    } else {
        vec![]
    };
    let p2_started = p2_total
        .iter()
        .filter(|e| matches!(e, CausalEvent::ConstructionStarted { .. }))
        .count();
    let p2_completed = p2_total
        .iter()
        .filter(|e| matches!(e, CausalEvent::ConstructionCompleted { .. }))
        .count();
    let p2_placed = p2_total
        .iter()
        .filter(|e| matches!(e, CausalEvent::BuildingPlaced { .. }))
        .count();
    assert_eq!(p2_started, 0);
    assert_eq!(p2_completed, 0);
    assert_eq!(p2_placed, 0);

    // Global newly-appended count check.
    assert_eq!(p1_completed + p2_completed, 1);
    assert_eq!(p1_placed + p2_placed, 1);
    println!("[β-14] Two-site independence: P1 completes, P2 stays at 0; global count == 1 each ✓");
}

// ── A16: Already-complete site is NOT a Construction target (no stuck agent) ──
//
// Attempt-3 regression: filter on `ConstructionSite::is_complete()` so the
// Idle cascade does NOT select Construction when the only co-located site
// is already saturated, AND the ConstructionSystem unwedges any agent that
// already entered `Consuming{ConstructionSite}` on such a site.
#[test]
fn harness_p6_beta_a16_already_complete_site_does_not_trigger_construction_and_unwedges_agent() {
    // (a) Idle cascade: a co-located complete site must NOT be selected,
    //     and no AgentDecision{ConstructionReason} must be emitted.
    let mut engine = fresh_engine();
    let agent_e = engine.spawn_agent(3, 5);
    insert_quiet_needs(&mut engine, agent_e);
    engine
        .world
        .insert_one(agent_e, AgentState::Idle)
        .unwrap();

    // required_progress == 0 → born complete (is_complete() == true).
    let bp_zero = BuildingBlueprint::new(1, 2, 2, 0);
    engine
        .world
        .spawn((ConstructionSite::new(bp_zero, Position::new(3, 5)),));

    // A site whose progress was directly written to == required_progress.
    let bp_saturated = BuildingBlueprint::new(2, 2, 2, 4);
    let saturated_site = engine
        .world
        .spawn((ConstructionSite::new(bp_saturated, Position::new(20, 20)),));
    engine
        .world
        .get::<&mut ConstructionSite>(saturated_site)
        .unwrap()
        .progress = 4;
    assert!(engine
        .world
        .get::<&ConstructionSite>(saturated_site)
        .unwrap()
        .is_complete());

    let tidx = tile_idx(3, 5);
    let before_len = engine
        .resources
        .causal_log
        .get(tidx)
        .map(|l| l.len())
        .unwrap_or(0);

    let mut decision = AgentDecisionSystem::new();
    decision.tick(&mut engine.world, &mut engine.resources);

    // Agent stays Idle — no need breached, no active construction.
    let state = *engine.world.get::<&AgentState>(agent_e).unwrap();
    assert_eq!(
        state,
        AgentState::Idle,
        "agent must stay Idle when only co-located site is already complete"
    );
    let new_events: Vec<&CausalEvent> = engine
        .resources
        .causal_log
        .get(tidx)
        .map(|l| l.as_slice().iter().skip(before_len).collect())
        .unwrap_or_default();
    let construction_reason_count = new_events
        .iter()
        .filter(|e| {
            matches!(
                e,
                CausalEvent::AgentDecision {
                    reason: DecisionReason::ConstructionReason,
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        construction_reason_count, 0,
        "no AgentDecision{{ConstructionReason}} must be emitted for a complete co-located site"
    );

    // (b) ConstructionSystem unwedge: an agent that somehow entered
    //     Consuming{ConstructionSite} on an already-complete site must be
    //     reset to Idle on the next tick — no duplicate completion chain.
    let other_agent = engine.spawn_agent(20, 20);
    insert_quiet_needs(&mut engine, other_agent);
    engine
        .world
        .insert_one(
            other_agent,
            AgentState::Consuming {
                target: TargetKind::ConstructionSite,
            },
        )
        .unwrap();

    let saturated_tidx = tile_idx(20, 20);
    let before_saturated_len = engine
        .resources
        .causal_log
        .get(saturated_tidx)
        .map(|l| l.len())
        .unwrap_or(0);

    let mut sys = ConstructionSystem::new();
    sys.tick(&mut engine.world, &mut engine.resources);

    let state_after = *engine.world.get::<&AgentState>(other_agent).unwrap();
    assert_eq!(
        state_after,
        AgentState::Idle,
        "agent on an already-complete site must be unwedged to Idle (no stuck Consuming)"
    );

    // The already-complete site is NOT despawned and emits NO completion
    // chain — ConstructionSystem owns completion edges only, not idle
    // reaping of pre-saturated sites.
    assert!(
        engine.world.contains(saturated_site),
        "already-complete site must NOT be despawned by ConstructionSystem (no completion edge)"
    );
    let new_saturated_events: Vec<&CausalEvent> = engine
        .resources
        .causal_log
        .get(saturated_tidx)
        .map(|l| l.as_slice().iter().skip(before_saturated_len).collect())
        .unwrap_or_default();
    let completed_count = new_saturated_events
        .iter()
        .filter(|e| matches!(e, CausalEvent::ConstructionCompleted { .. }))
        .count();
    let placed_count = new_saturated_events
        .iter()
        .filter(|e| matches!(e, CausalEvent::BuildingPlaced { .. }))
        .count();
    assert_eq!(
        completed_count, 0,
        "no ConstructionCompleted for already-complete site; got {}",
        completed_count
    );
    assert_eq!(
        placed_count, 0,
        "no BuildingPlaced for already-complete site; got {}",
        placed_count
    );
    println!(
        "[β-16] Complete site never selected; pre-Consuming agent unwedged; no duplicate chain ✓"
    );
}

// ── A15: Per-agent progress — two co-located agents advance site by 2/tick ─
//
// Plan attempt-2 regression: removing the per-site dedup means each
// co-located Consuming{ConstructionSite} agent advances the site once per
// tick. Two agents on a required_progress==2 site complete it in a single
// tick; the completion chain is emitted exactly once.
#[test]
fn harness_p6_beta_a15_two_agents_complete_required_progress_two_in_one_tick() {
    let mut engine = fresh_engine();

    let bp = BuildingBlueprint::new(7, 2, 2, 2);
    let site_e = engine
        .world
        .spawn((ConstructionSite::new(bp, Position::new(5, 6)),));

    let agent_a = engine.spawn_agent(5, 6);
    insert_quiet_needs(&mut engine, agent_a);
    engine
        .world
        .insert_one(
            agent_a,
            AgentState::Consuming {
                target: TargetKind::ConstructionSite,
            },
        )
        .unwrap();

    let agent_b = engine.spawn_agent(5, 6);
    insert_quiet_needs(&mut engine, agent_b);
    engine
        .world
        .insert_one(
            agent_b,
            AgentState::Consuming {
                target: TargetKind::ConstructionSite,
            },
        )
        .unwrap();

    let tidx = tile_idx(5, 6);
    let before_len = engine
        .resources
        .causal_log
        .get(tidx)
        .map(|l| l.len())
        .unwrap_or(0);

    // Diagnostic counter — N==2 here proves multi-agent co-location.
    let agents_doing_construction: usize = engine
        .world
        .query::<&AgentState>()
        .iter()
        .filter(|(_, s)| {
            matches!(
                **s,
                AgentState::Consuming {
                    target: TargetKind::ConstructionSite
                }
            )
        })
        .count();
    assert_eq!(
        agents_doing_construction, 2,
        "two agents must be Consuming{{ConstructionSite}} pre-tick"
    );
    println!("agents_doing_construction={}", agents_doing_construction);

    let mut sys = ConstructionSystem::new();
    sys.tick(&mut engine.world, &mut engine.resources);

    // Per-agent progress: site advances by 2 in one tick → completion.
    assert!(
        !engine.world.contains(site_e),
        "site must be despawned (required_progress=2, two co-located agents → completion in one tick)"
    );

    // Exactly one completion chain (Completed + Placed) — duplicates are
    // forbidden even with multiple agents on the same completion edge.
    let new_events: Vec<&CausalEvent> = engine
        .resources
        .causal_log
        .get(tidx)
        .map(|l| l.as_slice().iter().skip(before_len).collect())
        .unwrap_or_default();
    let completed_count = new_events
        .iter()
        .filter(|e| matches!(e, CausalEvent::ConstructionCompleted { .. }))
        .count();
    let placed_count = new_events
        .iter()
        .filter(|e| matches!(e, CausalEvent::BuildingPlaced { .. }))
        .count();
    assert_eq!(
        completed_count, 1,
        "exactly one ConstructionCompleted per completion edge; got {}",
        completed_count
    );
    assert_eq!(
        placed_count, 1,
        "exactly one BuildingPlaced per completion edge; got {}",
        placed_count
    );

    // Both agents reset to Idle by ConstructionSystem.
    assert_eq!(*engine.world.get::<&AgentState>(agent_a).unwrap(), AgentState::Idle);
    assert_eq!(*engine.world.get::<&AgentState>(agent_b).unwrap(), AgentState::Idle);
    println!(
        "[β-15] Two co-located agents complete required_progress=2 in one tick; chain emitted exactly once ✓"
    );
}
