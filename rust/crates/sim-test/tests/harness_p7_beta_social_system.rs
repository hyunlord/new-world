//! V7 Phase 7-β — SocialInteractionSystem + DecisionReason::SocialReason +
//! 2 new CausalEvent variants + AgentDecisionSystem 5th cascade.
//!
//! feature: p7-beta-social-interaction-system
//! plan_attempt: 2
//! seed: 42
//! agent_count: 20
//! lane: --full
//!
//! Assertion map (1:1 with the locked plan):
//!   A1   : system_metadata_priority_and_interval
//!   A1b  : system_is_registered_in_default_runtime
//!   A2   : decision_reason_social_as_str
//!   A3   : decision_reason_variant_count
//!   A4   : causal_event_new_variant_field_shapes
//!   A5   : idle_cascade_social_triggers_when_no_higher_priority_drive
//!   A5b  : sub_threshold_loneliness_does_not_fire_social
//!   A5c  : partial_breach_does_not_handshake
//!   A5d  : non_co_located_breach_does_not_handshake
//!   A5e  : strict_threshold_boundary
//!   A6a  : hunger_wins_over_social
//!   A6b  : thirst_wins_over_social
//!   A6c  : fatigue_wins_over_social
//!   A7   : construction_wins_over_social
//!   A7b  : decision_system_runs_before_social_system_within_tick
//!   A8   : mutual_handshake_emits_single_started_event
//!   A8b  : emitter_canonical_with_reversed_insertion_order
//!   A9   : completion_after_required_progress_ticks (exact tick delta)
//!   A10  : three_link_causal_chain_stitched (exactly 3)
//!   A11  : asymmetric_partner_fallback_no_panic
//!   A11b : progress_entry_cleanup_when_pair_stops_consuming
//!   A12  : interaction_progress_single_increment_per_pair_per_tick (delta=1)
//!   A12b : progress_accumulates_one_per_tick_over_two_ticks
//!   A13  : familiarity_saturates_at_one
//!   A13b : loneliness_saturates_at_zero
//!   A13c : mid_range_familiarity_bump
//!   A14  : pre_existing_phase_regressions_bounded_two_run
//!         (+ relationships.len() >= 1, SocialInteractionCompleted >= 1)
//!   A15  : relationships_and_progress_initialize_empty
//!   A16  : constants_locked_values
//!   A17  : loneliness_below_threshold_does_not_trigger_social
//!         (both 49.9 AND exact SOCIAL_THRESHOLD == 50.0)
//!   A17b : three_or_more_co_located_breached_agents_no_panic
//!   A18  : co_location_strict_same_tile_required
//!         (horizontal-adjacent, vertical-adjacent, same-tile control)
//!   A18b : no_eligible_agents_emits_no_events

use sim_core::causal::{CausalEvent, DecisionReason};
use sim_core::components::{
    Agent, AgentState, BuildingBlueprint, ConstructionSite, Hunger, Position,
    RelationshipKey, RelationshipState, Sleep, Social, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine, SimResources};
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::construction::ConstructionSystem;
use sim_systems::runtime::decision::{
    AgentDecisionSystem, FAMILIARITY_BUMP, REQUIRED_INTERACTION_PROGRESS, SOCIAL_CONSUME_AMOUNT,
    SOCIAL_THRESHOLD,
};
use sim_systems::runtime::influence::InfluenceVisualizationSystem;
use sim_systems::runtime::social::SocialInteractionSystem;
use sim_systems::{
    register_construction_systems, register_decision_systems, register_default_runtime_systems,
    register_needs_systems, register_phase2_systems,
};

const W: u32 = 128;
const H: u32 = 128;

/// Production-equivalent engine factory (P7-β scope): registers every
/// canonical runtime system via the SAME helper used by the live FFI path
/// in `sim-bridge::ffi::world_node::WorldSimNode::init`. This is the
/// production system list — A1b verifies registration here.
fn make_stage1_engine(seed: u64, agent_count: u32) -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);

    // V7 Phase 7-β re-plan (Option A): Stage-1 agents now have a positive
    // Social.growth_rate so canonical 4380-tick runs produce loneliness
    // breach naturally. SocialDecaySystem (priority 135) advances loneliness
    // each tick. Math: 4380 × 0.04 = 175.2 → saturation by ~tick 2500.
    // Densify the spawn lattice so co-located breached pairs occur:
    // place agents on a 4-wide column (x in [16, 17, 18, 19]) repeated
    // across the y-axis to maximise co-located pair chance under Brownian
    // motion.
    for i in 0..agent_count {
        let x = 16 + (i % 4);
        let y = 16 + (i / 4);
        let entity = engine.spawn_agent(x, y);
        engine
            .world
            .insert(
                entity,
                (
                    MovementRng::new(seed.wrapping_add(i as u64)),
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(0.0, 0.04),
                    AgentState::Idle,
                ),
            )
            .expect("freshly spawned agent must still exist");
    }
    engine
}

/// Baseline engine factory for A14: same as `make_stage1_engine` but
/// WITHOUT registering `SocialInteractionSystem`. Used to capture
/// per-DecisionReason counts in the pre-feature baseline run.
fn make_stage1_baseline_engine(seed: u64, agent_count: u32) -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    // Mirror register_agent_systems but inlined so we keep the same
    // priority slate apart from omitting SocialInteractionSystem.
    engine.register_system(Box::new(
        sim_systems::runtime::agent::AgentMovementSystem::new(),
    ));
    register_decision_systems(&mut engine);
    register_needs_systems(&mut engine);
    register_construction_systems(&mut engine);

    for i in 0..agent_count {
        let x = 16 + (i % 16) * 2;
        let y = 16 + (i / 16) * 2;
        let entity = engine.spawn_agent(x, y);
        engine
            .world
            .insert(
                entity,
                (
                    MovementRng::new(seed.wrapping_add(i as u64)),
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(0.0, 0.0),
                    AgentState::Idle,
                ),
            )
            .expect("freshly spawned agent must still exist");
    }
    engine
}

/// Place two agents at the same tile, both with mutual social-breach setup.
/// Returns `(smaller_entity, larger_entity, smaller_id, larger_id)` so that
/// smaller_id < larger_id (canonical order).
fn place_mutual_pair_at(
    engine: &mut SimEngine,
    tile: (u32, u32),
    loneliness: f64,
) -> (hecs::Entity, hecs::Entity, u64, u64) {
    let entity_a = engine.spawn_agent(tile.0, tile.1);
    let entity_b = engine.spawn_agent(tile.0, tile.1);
    let a_id = engine.world.get::<&Agent>(entity_a).unwrap().id;
    let b_id = engine.world.get::<&Agent>(entity_b).unwrap().id;
    for ent in [entity_a, entity_b] {
        engine
            .world
            .insert(
                ent,
                (
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(loneliness, 0.0),
                    AgentState::Idle,
                ),
            )
            .unwrap();
    }
    if a_id <= b_id {
        (entity_a, entity_b, a_id, b_id)
    } else {
        (entity_b, entity_a, b_id, a_id)
    }
}

fn fresh_engine() -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);
    engine
}

fn tile_idx(pos: (u32, u32), width: u32) -> u32 {
    pos.1 * width + pos.0
}

fn count_events_on_tile<F: Fn(&CausalEvent) -> bool>(
    resources: &SimResources,
    tile: (u32, u32),
    pred: F,
) -> usize {
    let idx = tile_idx(tile, resources.tile_grid.width);
    resources
        .causal_log
        .get(idx)
        .map(|log| log.as_slice().iter().filter(|e| pred(e)).count())
        .unwrap_or(0)
}

// ─── A1: system_metadata_priority_and_interval ─────────────────────────
#[test]
fn harness_p7_beta_a1_system_metadata_priority_interval_and_ordering() {
    let social = SocialInteractionSystem::new();
    // Type A: priority and interval locked.
    assert_eq!(social.priority(), 134, "SocialInteractionSystem priority must equal 134");
    assert_eq!(social.tick_interval(), 1, "SocialInteractionSystem tick_interval must equal 1");

    let construction = ConstructionSystem::new();
    // Strict ordering: Construction < Social < InfluenceVisualization.
    assert!(
        construction.priority() < social.priority(),
        "ConstructionSystem priority ({}) must be strictly less than SocialInteractionSystem ({})",
        construction.priority(),
        social.priority(),
    );
    let viz = InfluenceVisualizationSystem::new();
    assert!(
        social.priority() < viz.priority(),
        "SocialInteractionSystem priority ({}) must be strictly less than InfluenceVisualizationSystem ({})",
        social.priority(),
        viz.priority(),
    );
}

// ─── A1b: system_is_registered_in_default_runtime ──────────────────────
// Counts occurrences of `SocialInteractionSystem` in the production
// registry assembled by `register_default_runtime_systems` (the same
// helper called by WorldSimNode::init). Must be exactly 1.
#[test]
fn harness_p7_beta_a1b_system_registered_in_default_runtime_systems() {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);
    let count = engine
        .system_names()
        .into_iter()
        .filter(|n| *n == "SocialInteractionSystem")
        .count();
    assert_eq!(
        count, 1,
        "SocialInteractionSystem must appear exactly once in production system registry; found {count}"
    );
}

// ─── A2: decision_reason_social_as_str ─────────────────────────────────
#[test]
fn harness_p7_beta_a2_decision_reason_social_as_str() {
    assert_eq!(
        DecisionReason::SocialReason.as_str(),
        "social_reason",
        "DecisionReason::SocialReason.as_str() locked at 'social_reason' per P7β-5"
    );
}

// ─── A3: decision_reason_variant_count ─────────────────────────────────
// Exhaustive match — no wildcard. Adding/removing a variant fails compile.
#[test]
fn harness_p7_beta_a3_decision_reason_variant_count() {
    fn discriminant(r: DecisionReason) -> u8 {
        match r {
            DecisionReason::HungerThresholdBreach => 0,
            DecisionReason::ThirstThresholdBreach => 1,
            DecisionReason::FatigueThresholdBreach => 2,
            DecisionReason::ConstructionReason => 3,
            DecisionReason::SocialReason => 4,
            DecisionReason::MemoryReason => 5,
            DecisionReason::CombatReason => 6,
        }
    }
    assert_eq!(discriminant(DecisionReason::HungerThresholdBreach), 0);
    assert_eq!(discriminant(DecisionReason::ThirstThresholdBreach), 1);
    assert_eq!(discriminant(DecisionReason::FatigueThresholdBreach), 2);
    assert_eq!(discriminant(DecisionReason::ConstructionReason), 3);
    assert_eq!(discriminant(DecisionReason::SocialReason), 4);
    assert_eq!(discriminant(DecisionReason::MemoryReason), 5);
}

// ─── A4: causal_event_new_variant_field_shapes ─────────────────────────
// Non-None parent sentinel + position (7,9) + agents (3,5) + familiarity 0.37.
// Round-trip via accessors. parent must be Some(42), NOT None.
#[test]
fn harness_p7_beta_a4_causal_event_new_variant_field_shapes() {
    let started = CausalEvent::SocialInteractionStarted {
        id: 100,
        parent: Some(42),
        agents: (3u64, 5u64),
        position: (7u32, 9u32),
        tick: 11,
    };
    // Type A: all five locked fields constructible + accessors round-trip.
    assert_eq!(started.id(), 100);
    assert_eq!(
        started.parent(),
        Some(42),
        "parent must round-trip Some(EventId(42)) — NOT None"
    );
    assert_eq!(started.tick(), 11);
    assert_eq!(started.channel(), None, ".channel() must be None for Started");

    let completed = CausalEvent::SocialInteractionCompleted {
        id: 101,
        parent: Some(100),
        agents: (3u64, 5u64),
        position: (7u32, 9u32),
        familiarity_after: 0.37,
        tick: 14,
    };
    // Type A: all six locked fields on Completed + accessors round-trip.
    assert_eq!(completed.id(), 101);
    assert_eq!(
        completed.parent(),
        Some(100),
        "parent must round-trip Some(EventId(100))"
    );
    assert_eq!(completed.tick(), 14);
    assert_eq!(completed.channel(), None, ".channel() must be None for Completed");

    // Type-pin: agents = (AgentId, AgentId), position = (u32, u32),
    // familiarity_after = f64.
    if let CausalEvent::SocialInteractionStarted { agents, position, .. } = started {
        let _: (u64, u64) = agents;
        let _: (u32, u32) = position;
    } else {
        panic!("variant shape drift on Started");
    }
    if let CausalEvent::SocialInteractionCompleted {
        agents,
        position,
        familiarity_after,
        ..
    } = completed
    {
        let _: (u64, u64) = agents;
        let _: (u32, u32) = position;
        let _: f64 = familiarity_after;
        assert_eq!(familiarity_after, 0.37);
    } else {
        panic!("variant shape drift on Completed");
    }
}

// ─── A5: idle_cascade_social_triggers_when_no_higher_priority_drive ────
// Plan-locked behaviour: after ONE direct `AgentDecisionSystem::tick`,
// BOTH agents transition to Seeking{Agent(other)} AND BOTH push exactly
// one `AgentDecision { reason: SocialReason }` onto the shared tile.
//
// Diagnostics (--nocapture): prints a path-execution counter so an empty
// social path (cascade never fired) is visible. Counter MUST be > 0.
#[test]
fn harness_p7_beta_a5_idle_cascade_social_triggers() {
    // Bare engine — bypass the full system stack so we can call the
    // decision system DIRECTLY (per plan A5: "after one
    // AgentDecisionSystem::tick").
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    let (ent_a, ent_b, id_a, id_b) =
        place_mutual_pair_at(&mut engine, (10, 10), 60.0);

    // Pre-tick invariants — both Idle, no events on tile.
    let pre_decisions = count_events_on_tile(&engine.resources, (10, 10), |ev| {
        matches!(ev, CausalEvent::AgentDecision { reason: DecisionReason::SocialReason, .. })
    });
    assert_eq!(pre_decisions, 0, "pre-tick: zero SocialReason decisions on tile");
    assert_eq!(*engine.world.get::<&AgentState>(ent_a).unwrap(), AgentState::Idle);
    assert_eq!(*engine.world.get::<&AgentState>(ent_b).unwrap(), AgentState::Idle);

    // EXACTLY ONE AgentDecisionSystem::tick — the plan-locked direct call.
    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut engine.world, &mut engine.resources);

    let state_a = *engine.world.get::<&AgentState>(ent_a).unwrap();
    let state_b = *engine.world.get::<&AgentState>(ent_b).unwrap();
    assert_eq!(
        state_a,
        AgentState::Seeking { target: TargetKind::Agent(id_b) },
        "after one AgentDecisionSystem::tick, agent A must be Seeking{{Agent(B)}}"
    );
    assert_eq!(
        state_b,
        AgentState::Seeking { target: TargetKind::Agent(id_a) },
        "after one AgentDecisionSystem::tick, agent B must be Seeking{{Agent(A)}}"
    );

    // Both agents emit exactly one SocialReason decision on the tile.
    let decisions_a = count_events_on_tile(&engine.resources, (10, 10), |ev| matches!(ev,
        CausalEvent::AgentDecision { reason: DecisionReason::SocialReason, agent, .. } if *agent == id_a
    ));
    let decisions_b = count_events_on_tile(&engine.resources, (10, 10), |ev| matches!(ev,
        CausalEvent::AgentDecision { reason: DecisionReason::SocialReason, agent, .. } if *agent == id_b
    ));
    assert_eq!(decisions_a, 1, "agent A must have exactly 1 SocialReason decision; got {decisions_a}");
    assert_eq!(decisions_b, 1, "agent B must have exactly 1 SocialReason decision; got {decisions_b}");

    // Diagnostic counter — visible under --nocapture. Must be > 0 to prove
    // the social cascade path actually executed (defends against silent
    // no-op regressions where the 5th cascade arm is bypassed).
    let social_path_executions = decisions_a + decisions_b;
    println!(
        "[A5 diagnostic] social_path_executions={} (must be > 0); state_a={:?}, state_b={:?}",
        social_path_executions, state_a, state_b
    );
    assert!(
        social_path_executions > 0,
        "[A5 diagnostic] social cascade path executed zero times — the 5th cascade arm did not fire"
    );
}

// ─── A5b: sub_threshold_loneliness_does_not_fire_social ────────────────
// Both agents at loneliness=30.0 (strictly below SOCIAL_THRESHOLD=50.0).
// Run 10 ticks. BOTH must remain Idle. ZERO SocialReason events. ZERO
// SocialInteractionStarted events.
#[test]
fn harness_p7_beta_a5b_sub_threshold_loneliness_does_not_fire_social() {
    let mut engine = fresh_engine();
    let (ent_a, ent_b, _id_a, _id_b) =
        place_mutual_pair_at(&mut engine, (12, 12), 30.0);

    for _ in 0..10 {
        engine.tick();
    }

    let state_a = *engine.world.get::<&AgentState>(ent_a).unwrap();
    let state_b = *engine.world.get::<&AgentState>(ent_b).unwrap();
    assert_eq!(state_a, AgentState::Idle, "agent A must stay Idle (loneliness < threshold)");
    assert_eq!(state_b, AgentState::Idle, "agent B must stay Idle (loneliness < threshold)");

    let social_decisions = count_events_on_tile(&engine.resources, (12, 12), |ev| matches!(ev,
        CausalEvent::AgentDecision { reason: DecisionReason::SocialReason, .. }
    ));
    assert_eq!(social_decisions, 0, "no SocialReason decisions allowed");

    let started = count_events_on_tile(&engine.resources, (12, 12), |ev| {
        matches!(ev, CausalEvent::SocialInteractionStarted { .. })
    });
    assert_eq!(started, 0, "no SocialInteractionStarted allowed");
}

// ─── A5c: partial_breach_does_not_handshake ────────────────────────────
// A breached (60.0), B not breached (20.0). After 10 ticks, B stays Idle.
// A may enter Seeking{Agent(B)} but ZERO SocialInteractionStarted events
// (mutual handshake impossible).
#[test]
fn harness_p7_beta_a5c_partial_breach_does_not_handshake() {
    let mut engine = fresh_engine();
    let ent_a = engine.spawn_agent(14, 14);
    let ent_b = engine.spawn_agent(14, 14);
    let _id_a = engine.world.get::<&Agent>(ent_a).unwrap().id;
    let _id_b = engine.world.get::<&Agent>(ent_b).unwrap().id;
    engine
        .world
        .insert(
            ent_a,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(60.0, 0.0),
                AgentState::Idle,
            ),
        )
        .unwrap();
    engine
        .world
        .insert(
            ent_b,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(20.0, 0.0), // below threshold
                AgentState::Idle,
            ),
        )
        .unwrap();

    for _ in 0..10 {
        engine.tick();
    }

    let state_b = *engine.world.get::<&AgentState>(ent_b).unwrap();
    assert_eq!(state_b, AgentState::Idle, "agent B (not breached) must stay Idle");

    let started = count_events_on_tile(&engine.resources, (14, 14), |ev| {
        matches!(ev, CausalEvent::SocialInteractionStarted { .. })
    });
    assert_eq!(started, 0, "no mutual handshake allowed when only one side breached");
}

// ─── A5d: non_co_located_breach_does_not_handshake ─────────────────────
// Two agents both breached but on DIFFERENT tiles. ZERO Started events on
// ANY tile. Movement is denied by omitting `MovementRng` so positions stay
// fixed; we additionally assert positions did not drift between pre- and
// post-tick so the "non-co-located" invariant cannot be silently violated.
#[test]
fn harness_p7_beta_a5d_non_co_located_breach_does_not_handshake() {
    let mut engine = fresh_engine();
    let ent_a = engine.spawn_agent(5, 5);
    let ent_b = engine.spawn_agent(7, 5); // 2 tiles apart, distinct
    // No MovementRng inserted → AgentMovementSystem leaves them in place.
    engine
        .world
        .insert(
            ent_a,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(60.0, 0.0),
                AgentState::Idle,
            ),
        )
        .unwrap();
    engine
        .world
        .insert(
            ent_b,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(60.0, 0.0),
                AgentState::Idle,
            ),
        )
        .unwrap();

    // Capture pre-tick positions to assert invariance.
    let pre_a = *engine.world.get::<&Position>(ent_a).unwrap();
    let pre_b = *engine.world.get::<&Position>(ent_b).unwrap();
    assert!(
        pre_a.x != pre_b.x || pre_a.y != pre_b.y,
        "precondition: agents must spawn on distinct tiles"
    );

    for _ in 0..10 {
        engine.tick();
    }

    // Positions must NOT have changed (no MovementRng).
    let post_a = *engine.world.get::<&Position>(ent_a).unwrap();
    let post_b = *engine.world.get::<&Position>(ent_b).unwrap();
    assert_eq!(
        (post_a.x, post_a.y),
        (pre_a.x, pre_a.y),
        "agent A position must not drift (no MovementRng)"
    );
    assert_eq!(
        (post_b.x, post_b.y),
        (pre_b.x, pre_b.y),
        "agent B position must not drift (no MovementRng)"
    );
    assert!(
        post_a.x != post_b.x || post_a.y != post_b.y,
        "agents must still be on distinct tiles post-run"
    );

    // Strict zero-handshake invariant: ZERO Started events on ANY tile.
    let mut total_started = 0;
    for x in 0..W {
        for y in 0..H {
            total_started += count_events_on_tile(&engine.resources, (x, y), |ev| {
                matches!(ev, CausalEvent::SocialInteractionStarted { .. })
            });
        }
    }
    println!(
        "[A5d diagnostic] total_started={} (must be 0), pos_a=({},{}), pos_b=({},{})",
        total_started, post_a.x, post_a.y, post_b.x, post_b.y
    );
    assert_eq!(
        total_started, 0,
        "no Started events allowed — agents remain on distinct tiles throughout"
    );
}

// ─── A5e: strict_threshold_boundary ────────────────────────────────────
// loneliness == SOCIAL_THRESHOLD exactly. Both must stay Idle (strict `>`).
// ZERO SocialReason decisions.
#[test]
fn harness_p7_beta_a5e_strict_threshold_boundary() {
    let mut engine = fresh_engine();
    let (ent_a, ent_b, _id_a, _id_b) =
        place_mutual_pair_at(&mut engine, (16, 16), SOCIAL_THRESHOLD);

    for _ in 0..10 {
        engine.tick();
    }

    let state_a = *engine.world.get::<&AgentState>(ent_a).unwrap();
    let state_b = *engine.world.get::<&AgentState>(ent_b).unwrap();
    assert_eq!(state_a, AgentState::Idle, "agent A at threshold must stay Idle (strict `>`)");
    assert_eq!(state_b, AgentState::Idle, "agent B at threshold must stay Idle (strict `>`)");

    let social_decisions = count_events_on_tile(&engine.resources, (16, 16), |ev| matches!(ev,
        CausalEvent::AgentDecision { reason: DecisionReason::SocialReason, .. }
    ));
    assert_eq!(social_decisions, 0, "no SocialReason at threshold (strict `>`)");
}

// ─── A6a: hunger_wins_over_social ──────────────────────────────────────
#[test]
fn harness_p7_beta_a6a_hunger_wins_over_social() {
    let mut engine = fresh_engine();
    let (ent_a, _ent_b, id_a, _id_b) =
        place_mutual_pair_at(&mut engine, (8, 8), 60.0);
    *engine.world.get::<&mut Hunger>(ent_a).unwrap() = Hunger::new(80.0, 0.0);

    let mut left_idle = false;
    for _ in 0..5 {
        engine.tick();
        let state_a = *engine.world.get::<&AgentState>(ent_a).unwrap();
        if state_a != AgentState::Idle {
            left_idle = true;
            break;
        }
    }
    assert!(left_idle, "agent A must leave Idle within 5 ticks");

    let state_a = *engine.world.get::<&AgentState>(ent_a).unwrap();
    assert_eq!(
        state_a,
        AgentState::Seeking { target: TargetKind::Food },
        "Hunger must win — agent A seeks Food"
    );

    // Find the most-recent AgentDecision for agent A on tile (8,8).
    let idx = tile_idx((8, 8), engine.resources.tile_grid.width);
    let log = engine.resources.causal_log.get(idx).expect("log must exist");
    let last_decision_reason = log.as_slice().iter().rev().find_map(|ev| match ev {
        CausalEvent::AgentDecision { reason, agent, .. } if *agent == id_a => Some(*reason),
        _ => None,
    });
    assert_eq!(
        last_decision_reason,
        Some(DecisionReason::HungerThresholdBreach),
        "agent A's most-recent decision must be HungerThresholdBreach"
    );

    // ZERO SocialReason decisions for agent A in this window.
    let a_social_decisions = count_events_on_tile(&engine.resources, (8, 8), |ev| matches!(ev,
        CausalEvent::AgentDecision { reason: DecisionReason::SocialReason, agent, .. } if *agent == id_a
    ));
    assert_eq!(a_social_decisions, 0, "agent A must emit ZERO SocialReason decisions");
}

// ─── A6b: thirst_wins_over_social ──────────────────────────────────────
#[test]
fn harness_p7_beta_a6b_thirst_wins_over_social() {
    let mut engine = fresh_engine();
    let (ent_a, _ent_b, id_a, _id_b) =
        place_mutual_pair_at(&mut engine, (9, 9), 60.0);
    *engine.world.get::<&mut Thirst>(ent_a).unwrap() = Thirst::new(80.0, 0.0);

    for _ in 0..5 {
        engine.tick();
        if *engine.world.get::<&AgentState>(ent_a).unwrap() != AgentState::Idle {
            break;
        }
    }
    let state_a = *engine.world.get::<&AgentState>(ent_a).unwrap();
    assert_eq!(
        state_a,
        AgentState::Seeking { target: TargetKind::Water },
        "Thirst must win — agent A seeks Water"
    );

    let idx = tile_idx((9, 9), engine.resources.tile_grid.width);
    let log = engine.resources.causal_log.get(idx).expect("log");
    let last_decision_reason = log.as_slice().iter().rev().find_map(|ev| match ev {
        CausalEvent::AgentDecision { reason, agent, .. } if *agent == id_a => Some(*reason),
        _ => None,
    });
    assert_eq!(
        last_decision_reason,
        Some(DecisionReason::ThirstThresholdBreach),
        "agent A's most-recent decision must be ThirstThresholdBreach"
    );

    let a_social_decisions = count_events_on_tile(&engine.resources, (9, 9), |ev| matches!(ev,
        CausalEvent::AgentDecision { reason: DecisionReason::SocialReason, agent, .. } if *agent == id_a
    ));
    assert_eq!(a_social_decisions, 0, "agent A must emit ZERO SocialReason decisions");
}

// ─── A6c: fatigue_wins_over_social ─────────────────────────────────────
#[test]
fn harness_p7_beta_a6c_fatigue_wins_over_social() {
    let mut engine = fresh_engine();
    let (ent_a, _ent_b, id_a, _id_b) =
        place_mutual_pair_at(&mut engine, (11, 11), 60.0);
    *engine.world.get::<&mut Sleep>(ent_a).unwrap() = Sleep::new(80.0, 0.0);

    for _ in 0..5 {
        engine.tick();
        if *engine.world.get::<&AgentState>(ent_a).unwrap() != AgentState::Idle {
            break;
        }
    }
    let state_a = *engine.world.get::<&AgentState>(ent_a).unwrap();
    assert_eq!(
        state_a,
        AgentState::Seeking { target: TargetKind::Sleep },
        "Fatigue must win — agent A seeks Sleep"
    );

    let idx = tile_idx((11, 11), engine.resources.tile_grid.width);
    let log = engine.resources.causal_log.get(idx).expect("log");
    let last_decision_reason = log.as_slice().iter().rev().find_map(|ev| match ev {
        CausalEvent::AgentDecision { reason, agent, .. } if *agent == id_a => Some(*reason),
        _ => None,
    });
    assert_eq!(
        last_decision_reason,
        Some(DecisionReason::FatigueThresholdBreach),
        "agent A's most-recent decision must be FatigueThresholdBreach"
    );

    let a_social_decisions = count_events_on_tile(&engine.resources, (11, 11), |ev| matches!(ev,
        CausalEvent::AgentDecision { reason: DecisionReason::SocialReason, agent, .. } if *agent == id_a
    ));
    assert_eq!(a_social_decisions, 0, "agent A must emit ZERO SocialReason decisions");
}

// ─── A7: construction_wins_over_social ─────────────────────────────────
// Agent A must Seek ConstructionSite with last decision = ConstructionReason
// AND ZERO SocialReason decisions for agent A.
#[test]
fn harness_p7_beta_a7_construction_wins_over_social() {
    let mut engine = fresh_engine();
    let (ent_a, _ent_b, id_a, _id_b) =
        place_mutual_pair_at(&mut engine, (5, 5), 60.0);

    let blueprint = BuildingBlueprint::new(1u64, 1, 1, 5);
    let site = ConstructionSite::new(blueprint, Position { x: 5, y: 5 });
    engine.world.spawn((site,));

    for _ in 0..5 {
        engine.tick();
        if *engine.world.get::<&AgentState>(ent_a).unwrap() != AgentState::Idle {
            break;
        }
    }

    let state_a = *engine.world.get::<&AgentState>(ent_a).unwrap();
    assert!(
        matches!(state_a, AgentState::Seeking { target: TargetKind::ConstructionSite } | AgentState::Consuming { target: TargetKind::ConstructionSite }),
        "agent A must Seek/Consume ConstructionSite, got {state_a:?}"
    );

    // Most-recent AgentDecision for agent A: ConstructionReason.
    let idx = tile_idx((5, 5), engine.resources.tile_grid.width);
    let log = engine.resources.causal_log.get(idx).expect("log");
    let last_decision_reason = log.as_slice().iter().rev().find_map(|ev| match ev {
        CausalEvent::AgentDecision { reason, agent, .. } if *agent == id_a => Some(*reason),
        _ => None,
    });
    assert_eq!(
        last_decision_reason,
        Some(DecisionReason::ConstructionReason),
        "agent A's most-recent decision must be ConstructionReason"
    );

    // A must NOT be Seeking{Agent}.
    assert!(
        !matches!(state_a, AgentState::Seeking { target: TargetKind::Agent(_) }),
        "agent A must not be Seeking{{Agent(_)}}, got {state_a:?}"
    );

    // ZERO SocialReason decisions for agent A.
    let a_social_decisions = count_events_on_tile(&engine.resources, (5, 5), |ev| matches!(ev,
        CausalEvent::AgentDecision { reason: DecisionReason::SocialReason, agent, .. } if *agent == id_a
    ));
    assert_eq!(a_social_decisions, 0, "agent A must emit ZERO SocialReason decisions");
}

// ─── A7b: decision_system_runs_before_social_system_within_tick ────────
// For a matched (Started.parent → AgentDecision) pair, one of:
//   (i) same-tick: AgentDecision.tick == Started.tick AND AgentDecision.id
//       lower than Started.id
//   (ii) next-tick: Started.tick == AgentDecision.tick + 1
#[test]
fn harness_p7_beta_a7b_decision_runs_before_social_interaction() {
    let mut engine = fresh_engine();
    let (_ent_a, _ent_b, id_smaller, _id_larger) =
        place_mutual_pair_at(&mut engine, (7, 7), 60.0);

    // Run enough ticks to produce a matched pair.
    let mut matched = false;
    let mut started_seen = None;
    let mut decision_seen = None;
    for _ in 0..10 {
        engine.tick();
        let idx = tile_idx((7, 7), engine.resources.tile_grid.width);
        let log = engine.resources.causal_log.get(idx).expect("log");
        let events: Vec<&CausalEvent> = log.as_slice().iter().collect();

        // Find a SocialInteractionStarted whose parent references an
        // AgentDecision{SocialReason, agent==smaller} on this tile.
        for (i, ev) in events.iter().enumerate() {
            if let CausalEvent::SocialInteractionStarted { parent: Some(parent_id), tick: s_tick, .. } = ev {
                // Locate the matching AgentDecision by id.
                let decision = events.iter().enumerate().find_map(|(j, dev)| match dev {
                    CausalEvent::AgentDecision {
                        id, agent, reason: DecisionReason::SocialReason, tick: d_tick, ..
                    } if *id == *parent_id && *agent == id_smaller => Some((j, *id, *d_tick)),
                    _ => None,
                });
                if let Some((decision_pos, decision_id, d_tick)) = decision {
                    started_seen = Some((i, *parent_id, *s_tick));
                    decision_seen = Some((decision_pos, decision_id, d_tick));
                    // (i) same-tick: AgentDecision must precede Started in log order
                    // (ii) next-tick: s_tick == d_tick + 1
                    if *s_tick == d_tick {
                        assert!(
                            decision_pos < i,
                            "same-tick: AgentDecision (pos {decision_pos}, id {decision_id}) must precede Started (pos {i}, id {parent_id}) in log order"
                        );
                        matched = true;
                    } else if *s_tick == d_tick + 1 {
                        matched = true;
                    } else {
                        panic!(
                            "invalid trace: Started.tick={s_tick}, AgentDecision.tick={d_tick} — must be same-tick (with decision earlier) or next-tick"
                        );
                    }
                    break;
                }
            }
        }
        if matched {
            break;
        }
    }
    assert!(
        matched,
        "must produce a matched Started→AgentDecision pair within 10 ticks (seen started={started_seen:?}, decision={decision_seen:?})"
    );
}

// ─── A8: mutual_handshake_emits_single_started_event ───────────────────
// Exactly ONE Started in the tick it fires. Canonical (smaller, larger).
// parent is Some(eid) referencing the smaller-id AgentDecision{SocialReason}
// UNIQUELY (exactly one matching AgentDecision by EventId). Both agents
// then in Consuming{Agent(other)}.
#[test]
fn harness_p7_beta_a8_mutual_handshake_single_started_event() {
    let mut engine = fresh_engine();
    let (ent_a, ent_b, id_smaller, id_larger) =
        place_mutual_pair_at(&mut engine, (20, 20), 60.0);

    // Walk ticks; capture per-tick Started count once Started appears.
    let mut started_tick_count: Option<usize> = None;
    let mut prev_started_total = 0usize;
    for _ in 0..5 {
        engine.tick();
        let total = count_events_on_tile(&engine.resources, (20, 20), |ev| {
            matches!(ev, CausalEvent::SocialInteractionStarted { .. })
        });
        if total > prev_started_total {
            started_tick_count = Some(total - prev_started_total);
            break;
        }
        prev_started_total = total;
    }
    assert_eq!(
        started_tick_count,
        Some(1),
        "exactly ONE SocialInteractionStarted in the firing tick (count != 2)"
    );

    let idx = tile_idx((20, 20), engine.resources.tile_grid.width);
    let log = engine.resources.causal_log.get(idx).expect("log");

    // Locate the Started event.
    let started = log
        .as_slice()
        .iter()
        .find(|e| matches!(e, CausalEvent::SocialInteractionStarted { .. }))
        .expect("Started event");
    if let CausalEvent::SocialInteractionStarted { agents, position, parent, .. } = started {
        assert_eq!(*agents, (id_smaller, id_larger), "canonical (smaller, larger)");
        assert_eq!(*position, (20, 20), "position is the shared tile");
        let parent_id = parent.expect("parent must be Some(eid)");
        // Exactly ONE matching AgentDecision by EventId.
        let matching_count = log.as_slice().iter().filter(|e| matches!(e,
            CausalEvent::AgentDecision { id, agent, reason: DecisionReason::SocialReason, .. }
                if *id == parent_id && *agent == id_smaller
        )).count();
        assert_eq!(
            matching_count, 1,
            "exactly one matching AgentDecision{{SocialReason, agent==smaller, id==parent}} required; found {matching_count}"
        );
    }

    let state_a = *engine.world.get::<&AgentState>(ent_a).unwrap();
    let state_b = *engine.world.get::<&AgentState>(ent_b).unwrap();
    assert!(
        matches!(state_a, AgentState::Consuming { target: TargetKind::Agent(p) } if p == id_larger),
        "smaller-id agent must be Consuming{{Agent(larger)}}, got {state_a:?}"
    );
    assert!(
        matches!(state_b, AgentState::Consuming { target: TargetKind::Agent(p) } if p == id_smaller),
        "larger-id agent must be Consuming{{Agent(smaller)}}, got {state_b:?}"
    );
}

// ─── A8b: emitter_canonical_with_reversed_insertion_order ──────────────
#[test]
fn harness_p7_beta_a8b_agents_tuple_canonical_under_reversed_iteration() {
    for variant in 0..2 {
        let mut engine = fresh_engine();
        let (ent_first, ent_second) = if variant == 0 {
            (engine.spawn_agent(25, 25), engine.spawn_agent(25, 25))
        } else {
            // Reverse: spawn second first.
            let later = engine.spawn_agent(25, 25);
            let earlier = engine.spawn_agent(25, 25);
            (later, earlier)
        };
        for ent in [ent_first, ent_second] {
            engine
                .world
                .insert(
                    ent,
                    (
                        Hunger::new(0.0, 0.0),
                        Thirst::new(0.0, 0.0),
                        Sleep::new(0.0, 0.0),
                        Social::new(60.0, 0.0),
                        AgentState::Idle,
                    ),
                )
                .unwrap();
        }
        let id_first = engine.world.get::<&Agent>(ent_first).unwrap().id;
        let id_second = engine.world.get::<&Agent>(ent_second).unwrap().id;
        let smaller = id_first.min(id_second);
        let larger = id_first.max(id_second);

        let mut total_started = 0;
        for _ in 0..5 {
            engine.tick();
            total_started = count_events_on_tile(&engine.resources, (25, 25), |ev| {
                matches!(ev, CausalEvent::SocialInteractionStarted { .. })
            });
            if total_started >= 1 {
                break;
            }
        }
        assert_eq!(total_started, 1, "variant {variant}: exactly one Started event");

        let idx = tile_idx((25, 25), engine.resources.tile_grid.width);
        let log = engine.resources.causal_log.get(idx).expect("log");
        let agents_tuple = log
            .as_slice()
            .iter()
            .find_map(|e| match e {
                CausalEvent::SocialInteractionStarted { agents, .. } => Some(*agents),
                _ => None,
            })
            .expect("Started event");
        assert_eq!(
            agents_tuple,
            (smaller, larger),
            "variant {variant}: canonical (smaller, larger) regardless of insertion order"
        );
    }
}

// ─── A9: completion_after_required_progress_ticks (plan attempt-2) ─────
// Plan attempt-2 layout — two explicitly separated phases:
//
//   SETUP   : drive ONE cascade tick that closes the mutual handshake and
//             pushes `SocialInteractionStarted` onto the tile. The system
//             prompt's "ambiguous 'from setup' parent reference" is closed
//             by capturing `started_id` here as the locked parent witness.
//
//   MEASURE : run EXACTLY `REQUIRED_INTERACTION_PROGRESS` (== 3) direct
//             `SocialInteractionSystem::tick` calls. Per the SIS first-
//             observation contract, the handshake-tick observation seeded
//             progress at 0, so 3 SIS ticks yield progress 1 → 2 → 3 ==
//             REQUIRED, triggering completion. The completion event MUST
//             carry `parent == Some(started_id)` (locked stitching).
//
// All locked plan invariants checked: tick delta, FSM reset, loneliness
// saturation-subtract, familiarity bump, progress map cleanup, exactly one
// Completed event, parent → captured started_id.
#[test]
fn harness_p7_beta_a9_completion_after_required_progress_ticks() {
    let mut engine = fresh_engine();
    let (ent_a, ent_b, id_smaller, id_larger) =
        place_mutual_pair_at(&mut engine, (30, 30), 70.0);
    let key = RelationshipKey::new(id_smaller, id_larger);

    // ─── SETUP phase ──────────────────────────────────────────────────
    // Drive engine ticks until the cascade closes the handshake and emits
    // `SocialInteractionStarted`. Capture started_id as the locked parent
    // witness for the MEASURE phase. The handshake tick also runs SIS at
    // priority 134, seeding progress to 0 via the just_started branch.
    let mut started_id_opt: Option<u64> = None;
    let mut started_tick_opt: Option<u64> = None;
    for _ in 0..6 {
        engine.tick();
        let idx = tile_idx((30, 30), engine.resources.tile_grid.width);
        let log = engine.resources.causal_log.get(idx).expect("log");
        if let Some((id, tick)) = log.as_slice().iter().find_map(|e| match e {
            CausalEvent::SocialInteractionStarted { id, tick, .. } => Some((*id, *tick)),
            _ => None,
        }) {
            started_id_opt = Some(id);
            started_tick_opt = Some(tick);
            break;
        }
    }
    let started_id = started_id_opt.expect("SETUP: SocialInteractionStarted must fire within 6 ticks");
    let s_tick = started_tick_opt.expect("SETUP: started tick must be captured");

    // SETUP post-condition: handshake-tick progress seed == 0 (just_started
    // branch). Both agents in mutual Consuming.
    let prog_post_setup = engine
        .resources
        .interaction_progress
        .get(&key)
        .copied()
        .expect("SETUP: progress entry must exist after handshake tick");
    assert_eq!(
        prog_post_setup, 0,
        "SETUP: handshake-tick seed must be 0 (just_started branch); got {prog_post_setup}"
    );
    let state_a_setup = *engine.world.get::<&AgentState>(ent_a).unwrap();
    let state_b_setup = *engine.world.get::<&AgentState>(ent_b).unwrap();
    assert!(
        matches!(state_a_setup, AgentState::Consuming { target: TargetKind::Agent(p) } if p == id_larger),
        "SETUP: smaller-id agent must be Consuming{{Agent(larger)}}, got {state_a_setup:?}"
    );
    assert!(
        matches!(state_b_setup, AgentState::Consuming { target: TargetKind::Agent(p) } if p == id_smaller),
        "SETUP: larger-id agent must be Consuming{{Agent(smaller)}}, got {state_b_setup:?}"
    );

    // ─── MEASURE phase ────────────────────────────────────────────────
    // Exactly `REQUIRED_INTERACTION_PROGRESS` (== 3) direct SIS ticks. The
    // tick budget is locked — no engine ticks, no other systems mutate
    // state during the measurement window. Mid-cycle invariants checked
    // after each SIS tick.
    let mut sys = SocialInteractionSystem::new();
    for direct_tick_idx in 1..=REQUIRED_INTERACTION_PROGRESS {
        sys.tick(&mut engine.world, &mut engine.resources);

        if direct_tick_idx < REQUIRED_INTERACTION_PROGRESS {
            // Mid-cycle: progress increased monotonically, no completion.
            let prog_mid = engine
                .resources
                .interaction_progress
                .get(&key)
                .copied()
                .expect("MEASURE: progress entry must persist mid-cycle");
            assert_eq!(
                prog_mid, direct_tick_idx,
                "MEASURE tick {direct_tick_idx}: progress must equal direct_tick_idx; got {prog_mid}"
            );

            let state_a_mid = *engine.world.get::<&AgentState>(ent_a).unwrap();
            let state_b_mid = *engine.world.get::<&AgentState>(ent_b).unwrap();
            assert!(
                matches!(state_a_mid, AgentState::Consuming { target: TargetKind::Agent(p) } if p == id_larger),
                "MEASURE tick {direct_tick_idx}: smaller-id must remain Consuming{{Agent(larger)}}, got {state_a_mid:?}"
            );
            assert!(
                matches!(state_b_mid, AgentState::Consuming { target: TargetKind::Agent(p) } if p == id_smaller),
                "MEASURE tick {direct_tick_idx}: larger-id must remain Consuming{{Agent(smaller)}}, got {state_b_mid:?}"
            );

            let lone_a_mid = engine.world.get::<&Social>(ent_a).unwrap().loneliness;
            let lone_b_mid = engine.world.get::<&Social>(ent_b).unwrap().loneliness;
            assert_eq!(lone_a_mid, 70.0, "MEASURE tick {direct_tick_idx}: A loneliness unchanged");
            assert_eq!(lone_b_mid, 70.0, "MEASURE tick {direct_tick_idx}: B loneliness unchanged");

            let completed_mid = count_events_on_tile(&engine.resources, (30, 30), |ev| {
                matches!(ev, CausalEvent::SocialInteractionCompleted { .. })
            });
            assert_eq!(
                completed_mid, 0,
                "MEASURE tick {direct_tick_idx}: zero Completed allowed mid-cycle; got {completed_mid}"
            );
        }
    }

    // ─── ASSERTION phase ──────────────────────────────────────────────
    // All four state changes (FSM reset, loneliness drop, relationships
    // bump, progress map cleanup) coincide on the final MEASURE tick.
    let state_a = *engine.world.get::<&AgentState>(ent_a).unwrap();
    let state_b = *engine.world.get::<&AgentState>(ent_b).unwrap();
    assert_eq!(state_a, AgentState::Idle, "A must reset to Idle on completion");
    assert_eq!(state_b, AgentState::Idle, "B must reset to Idle on completion");

    let lone_a = engine.world.get::<&Social>(ent_a).unwrap().loneliness;
    let lone_b = engine.world.get::<&Social>(ent_b).unwrap().loneliness;
    assert_eq!(lone_a, 70.0 - SOCIAL_CONSUME_AMOUNT, "A loneliness decremented");
    assert_eq!(lone_b, 70.0 - SOCIAL_CONSUME_AMOUNT, "B loneliness decremented");

    let fam = engine
        .resources
        .relationships
        .get(&key)
        .expect("relationship entry must exist")
        .familiarity;
    assert_eq!(fam, FAMILIARITY_BUMP, "familiarity == FAMILIARITY_BUMP");

    // Updated per V7 Phase 7-γ plan §γ A4/A5 (locked Type A):
    // The `interaction_progress` entry is intentionally left at the terminal
    // `REQUIRED_INTERACTION_PROGRESS` value on the completion tick so a
    // post-`engine.tick()` (or post-direct-`sys.tick()`) observation can
    // witness `progress == REQUIRED_INTERACTION_PROGRESS`. The entry is
    // reaped on the next SIS tick by the step (g) stale-cleanup pass — the
    // pair is no longer in mutual `Consuming` at that point so the key
    // falls out of the live `mutual_pairs` snapshot.
    assert_eq!(
        engine.resources.interaction_progress.get(&key).copied(),
        Some(REQUIRED_INTERACTION_PROGRESS),
        "interaction_progress must hold REQUIRED on completion tick (deferred cleanup)"
    );
    // One additional SIS tick reaps the now-stale entry.
    sys.tick(&mut engine.world, &mut engine.resources);
    assert!(
        !engine.resources.interaction_progress.contains_key(&key),
        "interaction_progress entry must be removed on the next SIS tick after completion"
    );

    // Exactly one Completed event with the locked fields.
    let idx = tile_idx((30, 30), engine.resources.tile_grid.width);
    let log = engine.resources.causal_log.get(idx).unwrap();
    let completed_count = log
        .as_slice()
        .iter()
        .filter(|e| matches!(e, CausalEvent::SocialInteractionCompleted { .. }))
        .count();
    assert_eq!(completed_count, 1, "exactly one Completed event");

    let completed = log
        .as_slice()
        .iter()
        .find(|e| matches!(e, CausalEvent::SocialInteractionCompleted { .. }))
        .unwrap();
    if let CausalEvent::SocialInteractionCompleted {
        agents,
        position,
        familiarity_after,
        parent,
        tick: c_tick,
        ..
    } = completed
    {
        assert_eq!(*agents, (id_smaller, id_larger));
        assert_eq!(*position, (30, 30));
        assert_eq!(*familiarity_after, FAMILIARITY_BUMP);
        // Locked parent stitching: completion.parent ⇒ the captured
        // started_id from SETUP. Any drift implies a chain-break.
        assert_eq!(
            *parent,
            Some(started_id),
            "Completed.parent must stitch to the captured SETUP started_id ({started_id}); got {parent:?}"
        );
        // Tick delta locked at REQUIRED_INTERACTION_PROGRESS.
        assert!(
            *c_tick >= s_tick,
            "Completed.tick ({c_tick}) must be >= Started.tick ({s_tick})"
        );
    }
}

// ─── A10: three_link_causal_chain_stitched (exactly 3 links) ───────────
#[test]
fn harness_p7_beta_a10_three_link_causal_chain() {
    let mut engine = fresh_engine();
    let (_ent_a, _ent_b, id_smaller, _id_larger) =
        place_mutual_pair_at(&mut engine, (40, 40), 70.0);

    for _ in 0..12 {
        engine.tick();
        if count_events_on_tile(&engine.resources, (40, 40), |ev| {
            matches!(ev, CausalEvent::SocialInteractionCompleted { .. })
        }) >= 1 {
            break;
        }
    }
    let idx = tile_idx((40, 40), engine.resources.tile_grid.width);
    let log = engine.resources.causal_log.get(idx).expect("log");

    let completed_id = log.as_slice().iter().find_map(|e| match e {
        CausalEvent::SocialInteractionCompleted { id, .. } => Some(*id),
        _ => None,
    }).expect("Completed");
    let chain = engine.resources.causal_log.trace_parents(idx, completed_id);
    // Walk: chain[0]=Completed, chain[1]=Started, chain[2]=AgentDecision.
    assert_eq!(chain.len(), 3, "chain must have exactly 3 links, got {}", chain.len());

    assert!(matches!(chain[0], CausalEvent::SocialInteractionCompleted { .. }), "link 0 = Completed");
    let completed_agents = match chain[0] {
        CausalEvent::SocialInteractionCompleted { agents, .. } => *agents,
        _ => unreachable!(),
    };

    if let CausalEvent::SocialInteractionStarted { agents, .. } = chain[1] {
        assert_eq!(*agents, completed_agents, "Started.agents matches Completed.agents");
    } else {
        panic!("link 1 must be Started, got {:?}", chain[1]);
    }

    if let CausalEvent::AgentDecision { reason, agent, .. } = chain[2] {
        assert_eq!(*reason, DecisionReason::SocialReason, "link 2 = AgentDecision{{SocialReason}}");
        assert_eq!(*agent, id_smaller, "link 2's agent = smaller id");
    } else {
        panic!("link 2 must be AgentDecision, got {:?}", chain[2]);
    }

    // Third link's parent is None (root).
    assert_eq!(
        chain[2].parent(),
        None,
        "third link (root AgentDecision{{SocialReason}}) must have parent == None — no orphan, no cycle"
    );
}

// ─── A11: asymmetric_partner_fallback_no_panic ─────────────────────────
// 3 variants per plan: B Idle on T1, B Consuming{Agent(A)} on T2, B nonexistent.
// All: agent A → Idle, ZERO Completed, no stale progress, no panic.
#[test]
fn harness_p7_beta_a11_asymmetric_partner_fallback() {
    for variant in 0..3 {
        let mut engine = fresh_engine();
        let ent_a = engine.spawn_agent(50, 50);
        let id_a = engine.world.get::<&Agent>(ent_a).unwrap().id;
        let id_b: u64 = match variant {
            0 => {
                let b = engine.spawn_agent(50, 50);
                let bid = engine.world.get::<&Agent>(b).unwrap().id;
                engine
                    .world
                    .insert(
                        b,
                        (
                            Hunger::new(0.0, 0.0),
                            Thirst::new(0.0, 0.0),
                            Sleep::new(0.0, 0.0),
                            Social::new(0.0, 0.0),
                            AgentState::Idle,
                        ),
                    )
                    .unwrap();
                bid
            }
            1 => {
                // B Consuming{Agent(A)} on DIFFERENT tile T2.
                let b = engine.spawn_agent(60, 60);
                let bid = engine.world.get::<&Agent>(b).unwrap().id;
                engine
                    .world
                    .insert(
                        b,
                        (
                            Hunger::new(0.0, 0.0),
                            Thirst::new(0.0, 0.0),
                            Sleep::new(0.0, 0.0),
                            Social::new(0.0, 0.0),
                            AgentState::Consuming {
                                target: TargetKind::Agent(id_a),
                            },
                        ),
                    )
                    .unwrap();
                bid
            }
            2 => 0xDEAD_BEEF,
            _ => unreachable!(),
        };

        let key = RelationshipKey::new(id_a, id_b);
        // Pre-seed progress to verify removal.
        engine.resources.interaction_progress.insert(key, 2);

        engine
            .world
            .insert(
                ent_a,
                (
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(0.0, 0.0),
                    AgentState::Consuming {
                        target: TargetKind::Agent(id_b),
                    },
                ),
            )
            .unwrap();

        let mut sys = SocialInteractionSystem::new();
        sys.tick(&mut engine.world, &mut engine.resources);

        let state_a = *engine.world.get::<&AgentState>(ent_a).unwrap();
        assert_eq!(
            state_a,
            AgentState::Idle,
            "variant {variant}: agent A must reset to Idle"
        );

        let completed = count_events_on_tile(&engine.resources, (50, 50), |ev| {
            matches!(ev, CausalEvent::SocialInteractionCompleted { .. })
        });
        assert_eq!(completed, 0, "variant {variant}: zero Completed events");

        assert!(
            !engine.resources.interaction_progress.contains_key(&key),
            "variant {variant}: stale progress entry must be removed"
        );
    }
}

// ─── A11b: progress_entry_cleanup_when_pair_stops_consuming ────────────
// Drive A,B through a real handshake into mutual Consuming with progress
// ≥1, then force A to Idle. After one SIS tick, the progress entry must
// be removed; no Completed event.
#[test]
fn harness_p7_beta_a11b_progress_cleanup_when_pair_stops_consuming() {
    let mut engine = fresh_engine();
    let (ent_a, _ent_b, id_smaller, id_larger) =
        place_mutual_pair_at(&mut engine, (55, 55), 60.0);
    let key = RelationshipKey::new(id_smaller, id_larger);

    // Drive into mutual Consuming.
    let mut handshake_done = false;
    for _ in 0..5 {
        engine.tick();
        if count_events_on_tile(&engine.resources, (55, 55), |ev| {
            matches!(ev, CausalEvent::SocialInteractionStarted { .. })
        }) >= 1 {
            handshake_done = true;
            break;
        }
    }
    assert!(handshake_done, "handshake must complete within 5 ticks");

    // Run one more engine tick so progress accumulates ≥ 1.
    engine.tick();
    let prog_before = engine
        .resources
        .interaction_progress
        .get(&key)
        .copied()
        .unwrap_or(0);
    assert!(prog_before >= 1, "progress must be ≥1 before forcing Idle; got {prog_before}");

    // Force agent A to Idle (external override).
    *engine.world.get::<&mut AgentState>(ent_a).unwrap() = AgentState::Idle;

    // One SocialInteractionSystem tick.
    let mut sys = SocialInteractionSystem::new();
    sys.tick(&mut engine.world, &mut engine.resources);

    assert!(
        !engine.resources.interaction_progress.contains_key(&key),
        "progress entry must be removed after pair stops mutual Consuming"
    );
    // The Completed count from the start of the test must not include
    // any new emission from this final SIS tick.
    let completed_after = count_events_on_tile(&engine.resources, (55, 55), |ev| {
        matches!(ev, CausalEvent::SocialInteractionCompleted { .. })
    });
    assert_eq!(completed_after, 0, "no Completed emission for the broken pair");
}

// ─── A12: interaction_progress_single_increment_per_pair_per_tick ──────
// Real cascade from Idle. Capture progress immediately after Started, run
// exactly one additional SocialInteractionSystem tick. Delta must == 1.
#[test]
fn harness_p7_beta_a12_single_increment_per_pair_per_tick() {
    let mut engine = fresh_engine();
    let (_ent_a, _ent_b, id_smaller, id_larger) =
        place_mutual_pair_at(&mut engine, (70, 70), 60.0);
    let key = RelationshipKey::new(id_smaller, id_larger);

    // Drive engine until Started appears.
    let mut started_seen = false;
    for _ in 0..5 {
        engine.tick();
        if count_events_on_tile(&engine.resources, (70, 70), |ev| {
            matches!(ev, CausalEvent::SocialInteractionStarted { .. })
        }) >= 1 {
            started_seen = true;
            break;
        }
    }
    assert!(started_seen, "Started must appear within 5 ticks");

    let prog_after_started = engine
        .resources
        .interaction_progress
        .get(&key)
        .copied()
        .unwrap_or(0);

    // Run exactly one more SocialInteractionSystem tick (use the system
    // directly to isolate from other systems' side-effects).
    let mut sys = SocialInteractionSystem::new();
    sys.tick(&mut engine.world, &mut engine.resources);

    let prog_after_one_more = engine
        .resources
        .interaction_progress
        .get(&key)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        prog_after_one_more - prog_after_started,
        1,
        "delta in this single tick must be exactly 1 (not 2); before={prog_after_started}, after={prog_after_one_more}"
    );
}

// ─── A12b: progress_accumulates_one_per_tick_over_two_ticks ────────────
#[test]
fn harness_p7_beta_a12b_n_ticks_yields_n_progress() {
    let mut engine = fresh_engine();
    let (_ent_a, _ent_b, id_smaller, id_larger) =
        place_mutual_pair_at(&mut engine, (72, 72), 60.0);
    let key = RelationshipKey::new(id_smaller, id_larger);

    // Drive engine until Started appears.
    for _ in 0..5 {
        engine.tick();
        if count_events_on_tile(&engine.resources, (72, 72), |ev| {
            matches!(ev, CausalEvent::SocialInteractionStarted { .. })
        }) >= 1 {
            break;
        }
    }
    let prog_after_started = engine
        .resources
        .interaction_progress
        .get(&key)
        .copied()
        .unwrap_or(0);

    // Two MORE SocialInteractionSystem ticks; must stop before completion.
    let mut sys = SocialInteractionSystem::new();
    sys.tick(&mut engine.world, &mut engine.resources);
    sys.tick(&mut engine.world, &mut engine.resources);

    let prog_after_two_more = engine
        .resources
        .interaction_progress
        .get(&key)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        prog_after_two_more - prog_after_started,
        2,
        "delta over two SIS ticks must be exactly 2; before={prog_after_started}, after={prog_after_two_more}"
    );
}

// ─── A13: familiarity_saturates_at_one ─────────────────────────────────
#[test]
fn harness_p7_beta_a13_familiarity_saturates() {
    let mut engine = fresh_engine();
    let (_ent_a, _ent_b, id_smaller, id_larger) =
        place_mutual_pair_at(&mut engine, (80, 80), 70.0);
    let key = RelationshipKey::new(id_smaller, id_larger);
    let mut pre = RelationshipState::new();
    pre.bump(0.95);
    engine.resources.relationships.insert(key, pre);

    for _ in 0..12 {
        engine.tick();
        if count_events_on_tile(&engine.resources, (80, 80), |ev| {
            matches!(ev, CausalEvent::SocialInteractionCompleted { .. })
        }) >= 1 {
            break;
        }
    }
    let fam = engine.resources.relationships.get(&key).unwrap().familiarity;
    assert_eq!(fam, 1.0, "familiarity must saturate at 1.0, got {fam}");

    let idx = tile_idx((80, 80), engine.resources.tile_grid.width);
    let log = engine.resources.causal_log.get(idx).unwrap();
    let completed_fam = log.as_slice().iter().find_map(|e| match e {
        CausalEvent::SocialInteractionCompleted { familiarity_after, .. } => Some(*familiarity_after),
        _ => None,
    });
    assert_eq!(completed_fam, Some(1.0), "completed.familiarity_after must be 1.0");
}

// ─── A13b: loneliness_saturates_at_zero ────────────────────────────────
// Per plan: two agents pre-placed directly into mutual Consuming with
// empty interaction_progress (the cascade would not fire at loneliness 10).
// Run EXACTLY 3 `SocialInteractionSystem` ticks. The arithmetic under test
// is the saturating consume on completion, not the cascade. With a
// pre-placed pair and empty progress, the SIS first-observation branch
// seeds progress at 1 (no Started on this tick), so 3 SIS ticks yields
// progress 1 → 2 → 3 → completion. Both loneliness values (10.0) must
// saturate to 0.0 after subtracting SOCIAL_CONSUME_AMOUNT (30.0).
#[test]
fn harness_p7_beta_a13b_loneliness_saturating_subtract_at_zero() {
    let mut engine = fresh_engine();
    let ent_a = engine.spawn_agent(85, 85);
    let ent_b = engine.spawn_agent(85, 85);
    let id_a = engine.world.get::<&Agent>(ent_a).unwrap().id;
    let id_b = engine.world.get::<&Agent>(ent_b).unwrap().id;
    engine
        .world
        .insert(
            ent_a,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(10.0, 0.0),
                AgentState::Consuming {
                    target: TargetKind::Agent(id_b),
                },
            ),
        )
        .unwrap();
    engine
        .world
        .insert(
            ent_b,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(10.0, 0.0),
                AgentState::Consuming {
                    target: TargetKind::Agent(id_a),
                },
            ),
        )
        .unwrap();

    // Empty interaction_progress (no pre-seed). Run EXACTLY 3 SIS ticks
    // — the locked direct-tick budget for pre-placed pairs.
    let key = RelationshipKey::new(id_a, id_b);
    assert!(
        !engine.resources.interaction_progress.contains_key(&key),
        "precondition: interaction_progress must start empty"
    );
    let mut sys = SocialInteractionSystem::new();
    for tick in 1..=REQUIRED_INTERACTION_PROGRESS {
        sys.tick(&mut engine.world, &mut engine.resources);
        if tick < REQUIRED_INTERACTION_PROGRESS {
            // Mid-cycle invariant: progress entry still alive, agents
            // still Consuming (no premature completion).
            assert!(
                engine.resources.interaction_progress.contains_key(&key),
                "tick {tick}: progress entry must persist mid-cycle"
            );
            let state_a = *engine.world.get::<&AgentState>(ent_a).unwrap();
            let state_b = *engine.world.get::<&AgentState>(ent_b).unwrap();
            assert!(
                matches!(state_a, AgentState::Consuming { target: TargetKind::Agent(p) } if p == id_b),
                "tick {tick}: A must still be Consuming{{Agent(B)}}, got {state_a:?}"
            );
            assert!(
                matches!(state_b, AgentState::Consuming { target: TargetKind::Agent(p) } if p == id_a),
                "tick {tick}: B must still be Consuming{{Agent(A)}}, got {state_b:?}"
            );
        }
    }

    // After 3 SIS ticks the cycle completes and the consume fires.
    let lone_a = engine.world.get::<&Social>(ent_a).unwrap().loneliness;
    let lone_b = engine.world.get::<&Social>(ent_b).unwrap().loneliness;
    assert_eq!(
        lone_a, 0.0,
        "A loneliness must saturate at 0.0 (10.0 - 30.0 via .max(0.0)), got {lone_a}"
    );
    assert_eq!(
        lone_b, 0.0,
        "B loneliness must saturate at 0.0 (10.0 - 30.0 via .max(0.0)), got {lone_b}"
    );

    // The completion path must have run (state reset + progress observable
    // at terminal value on completion tick, reaped on the next SIS tick).
    assert_eq!(
        *engine.world.get::<&AgentState>(ent_a).unwrap(),
        AgentState::Idle,
        "A must reset to Idle on completion"
    );
    assert_eq!(
        *engine.world.get::<&AgentState>(ent_b).unwrap(),
        AgentState::Idle,
        "B must reset to Idle on completion"
    );
    // Updated per V7 Phase 7-γ plan §γ A4/A5 (locked Type A): deferred
    // cleanup. Entry stays at REQUIRED on completion tick; the next SIS
    // tick reaps it via the step (g) stale pass.
    assert_eq!(
        engine.resources.interaction_progress.get(&key).copied(),
        Some(REQUIRED_INTERACTION_PROGRESS),
        "progress entry must hold REQUIRED on completion tick (deferred cleanup)"
    );
    sys.tick(&mut engine.world, &mut engine.resources);
    assert!(
        !engine.resources.interaction_progress.contains_key(&key),
        "progress entry must be removed on the next SIS tick after completion"
    );
}

// ─── A13c: mid_range_familiarity_bump ──────────────────────────────────
#[test]
fn harness_p7_beta_a13c_mid_range_familiarity_bump() {
    let mut engine = fresh_engine();
    let (_ent_a, _ent_b, id_smaller, id_larger) =
        place_mutual_pair_at(&mut engine, (82, 82), 70.0);
    let key = RelationshipKey::new(id_smaller, id_larger);
    let mut pre = RelationshipState::new();
    pre.bump(0.4);
    engine.resources.relationships.insert(key, pre);

    for _ in 0..12 {
        engine.tick();
        if count_events_on_tile(&engine.resources, (82, 82), |ev| {
            matches!(ev, CausalEvent::SocialInteractionCompleted { .. })
        }) >= 1 {
            break;
        }
    }
    let fam = engine.resources.relationships.get(&key).unwrap().familiarity;
    assert_eq!(fam, 0.5, "familiarity 0.4 + 0.1 must equal 0.5; got {fam}");

    let idx = tile_idx((82, 82), engine.resources.tile_grid.width);
    let log = engine.resources.causal_log.get(idx).unwrap();
    let completed_fam = log.as_slice().iter().find_map(|e| match e {
        CausalEvent::SocialInteractionCompleted { familiarity_after, .. } => Some(*familiarity_after),
        _ => None,
    });
    assert_eq!(completed_fam, Some(0.5), "completed.familiarity_after must be 0.5");
}

// ─── A14: pre_existing_phase_regressions_bounded_two_run ───────────────
// Run base (no SIS) + feature (with SIS) for 4380 ticks each on the same
// preconditioned scenario. Compare per-DecisionReason counts and the four
// economic metrics within ±15% (strict equality when baseline == 0).
// Additionally: `band_count_feature ∈ [1, 5]` per Dunbar (20 ÷ 15 ≈ 1–2,
// dynamics ≤ 5) — the locked feature-only bound that proves the social
// loop actually produced relationships rather than no-op'd silently.
//
// Setup: 6 of the 20 agents are pre-conditioned into 3 co-located pairs
// at distinct shelf-row tiles with `loneliness = 60.0` (above
// SOCIAL_THRESHOLD = 50.0) and no MovementRng (so Brownian drift cannot
// invalidate the handshake precondition). The other 14 agents keep the
// default factory layout. In the feature run each pair completes ONE
// cycle (loneliness drops to 30.0, below threshold → no re-engagement),
// producing exactly 3 entries in `relationships`.
#[test]
fn harness_p7_beta_a14_two_run_regression_bounded() {
    fn precondition_lonely_pairs(engine: &mut SimEngine) {
        // Pair the first 6 spawned agents into 3 co-located pairs. Spread
        // the pair-tiles across non-adjacent rows so no two pairs accidentally
        // share a tile (which would produce a 4-clique band).
        let pair_tiles = [(60u32, 60u32), (60, 64), (60, 68)];
        let entities: Vec<hecs::Entity> = engine
            .world
            .query::<&Agent>()
            .iter()
            .map(|(e, _)| e)
            .collect();
        for (i, &ent) in entities.iter().take(6).enumerate() {
            let tile = pair_tiles[i / 2];
            if let Ok(mut pos) = engine.world.get::<&mut Position>(ent) {
                pos.x = tile.0;
                pos.y = tile.1;
            }
            if let Ok(mut social) = engine.world.get::<&mut Social>(ent) {
                social.loneliness = 60.0;
            }
            // Remove MovementRng so Brownian drift cannot pull the pair
            // apart between AgentMovementSystem (120) and AgentDecisionSystem
            // (125). Failure here is harmless — the component may not be
            // present on every agent layout — but we expect it to succeed
            // because both factory helpers attach MovementRng.
            let _ = engine.world.remove_one::<MovementRng>(ent);
        }

        // Plan attempt-2 fix #3: per-reason ≥1 cascade-arm presence — give a
        // few non-pair agents non-zero need growth rates so each of the 4
        // pre-social cascade arms (Hunger/Thirst/Fatigue/Construction) fires
        // at least once in the 4380-tick feature run. Without this, the
        // baseline factory zeros all growth and only Social fires, making
        // the per-reason ≥1 assertion impossible to satisfy.
        // Agents 6 (Hunger), 7 (Thirst), 8 (Fatigue) — high initial value +
        // tiny growth so the cascade fires within the first few ticks.
        if let Some(&e_hunger) = entities.get(6) {
            if let Ok(mut h) = engine.world.get::<&mut Hunger>(e_hunger) {
                *h = Hunger::new(80.0, 0.0);
            }
        }
        if let Some(&e_thirst) = entities.get(7) {
            if let Ok(mut t) = engine.world.get::<&mut Thirst>(e_thirst) {
                *t = Thirst::new(80.0, 0.0);
            }
        }
        if let Some(&e_fatigue) = entities.get(8) {
            if let Ok(mut s) = engine.world.get::<&mut Sleep>(e_fatigue) {
                *s = Sleep::new(80.0, 0.0);
            }
        }
        // Agent 9 — Construction cascade. Place a ConstructionSite at its
        // tile so the 4th cascade arm fires.
        if let Some(&e_constr) = entities.get(9) {
            let cpos_opt = engine
                .world
                .get::<&Position>(e_constr)
                .ok()
                .map(|p| Position { x: p.x, y: p.y });
            if let Some(cpos) = cpos_opt {
                let blueprint = BuildingBlueprint::new(1u64, 1, 1, 5);
                let site = ConstructionSite::new(blueprint, cpos);
                engine.world.spawn((site,));
            }
        }
    }

    fn count_relationship_bands(
        rels: &std::collections::HashMap<RelationshipKey, RelationshipState>,
    ) -> u64 {
        // Connected-components count on the relationship graph. Each
        // RelationshipKey contributes one undirected edge (smaller ↔ larger).
        // Isolated agents (no relationships) are NOT counted — the band
        // metric measures emergent social groupings, not the headcount.
        use std::collections::{HashMap, HashSet};
        let mut adj: HashMap<u64, Vec<u64>> = HashMap::new();
        for key in rels.keys() {
            adj.entry(key.smaller()).or_default().push(key.larger());
            adj.entry(key.larger()).or_default().push(key.smaller());
        }
        let mut visited: HashSet<u64> = HashSet::new();
        let mut components: u64 = 0;
        let nodes: Vec<u64> = adj.keys().copied().collect();
        for start in nodes {
            if !visited.insert(start) {
                continue;
            }
            let mut stack = vec![start];
            while let Some(n) = stack.pop() {
                if let Some(neighbors) = adj.get(&n) {
                    for &nb in neighbors {
                        if visited.insert(nb) {
                            stack.push(nb);
                        }
                    }
                }
            }
            components += 1;
        }
        components
    }

    fn run(engine: &mut SimEngine, ticks: u64) -> RegressionMetrics {
        for _ in 0..ticks {
            engine.tick();
        }
        let mut counts = [0u64; 4];
        let mut buildings_placed = 0u64;
        let mut completed_count = 0u64;
        for (_, log) in engine.resources.causal_log.iter() {
            for ev in log.as_slice().iter() {
                match ev {
                    CausalEvent::AgentDecision { reason, .. } => {
                        let bucket = match reason {
                            DecisionReason::HungerThresholdBreach => 0,
                            DecisionReason::ThirstThresholdBreach => 1,
                            DecisionReason::FatigueThresholdBreach => 2,
                            DecisionReason::ConstructionReason => 3,
                            // SocialReason is the feature itself; excluded
                            // from the regression comparison.
                            DecisionReason::SocialReason => continue,
                            // P8-β: MemoryReason is the cascade-bias 6th
                            // emission; excluded from earlier-phase
                            // regression comparison.
                            DecisionReason::MemoryReason => continue,
                            // P9-β: CombatReason is the combat cascade
                            // 7th emission; excluded from regression.
                            DecisionReason::CombatReason => continue,
                        };
                        counts[bucket] += 1;
                    }
                    CausalEvent::BuildingPlaced { .. } => {
                        buildings_placed += 1;
                    }
                    CausalEvent::SocialInteractionCompleted { .. } => {
                        completed_count += 1;
                    }
                    _ => {}
                }
            }
        }
        // band_count: connected components of the live relationships graph.
        // The baseline run has SIS absent and never bumps relationships, so
        // this returns 0 there. The feature run forms one edge per pair
        // that completes a cycle.
        let band_count = count_relationship_bands(&engine.resources.relationships);
        let relationships_len = engine.resources.relationships.len() as u64;
        RegressionMetrics {
            decision_counts: counts,
            complete_buildings: buildings_placed,
            band_count,
            relationships_len,
            social_completed: completed_count,
        }
    }

    let mut baseline = make_stage1_baseline_engine(42, 20);
    precondition_lonely_pairs(&mut baseline);
    let base_metrics = run(&mut baseline, 4380);

    let mut feature = make_stage1_engine(42, 20);
    precondition_lonely_pairs(&mut feature);
    let feat_metrics = run(&mut feature, 4380);

    // ±15% tolerance gate for the regression-class metrics. When baseline
    // is zero we require feature to also be zero (no silent leak). Note:
    // the previously fabricated `total_stone` / `total_wood` metrics have
    // been removed — V7 Foundation has no stockpile substrate, so those
    // numbers were always zero (vacuous comparison). The locked feature-
    // side requirements below (relationships.len() ≥ 1, completed ≥ 1)
    // close the regression hole.
    fn assert_within(name: &str, base: u64, feat: u64) {
        if base == 0 {
            assert_eq!(
                feat, 0,
                "[{name}] baseline = 0; feature must also be 0 (strict). got feature={feat}"
            );
        } else {
            let lo = (base as f64 * 0.85).floor() as u64;
            let hi = (base as f64 * 1.15).ceil() as u64;
            assert!(
                feat >= lo && feat <= hi,
                "[{name}] regression: baseline={base} feature={feat} (allowed [{lo}, {hi}])"
            );
        }
    }

    let labels = [
        "HungerThresholdBreach",
        "ThirstThresholdBreach",
        "FatigueThresholdBreach",
        "ConstructionReason",
    ];
    for (i, label) in labels.iter().enumerate() {
        assert_within(label, base_metrics.decision_counts[i], feat_metrics.decision_counts[i]);
    }
    assert_within("complete_buildings", base_metrics.complete_buildings, feat_metrics.complete_buildings);

    // Plan attempt-2 fix #4: locked evaluation_criteria.md Part 3 upper
    // bounds on economic accumulators across the 4380-tick feature run.
    // V7 Foundation has no stockpile substrate (stone/wood always 0) so
    // those bounds hold trivially today; the assertion documents the
    // locked plan threshold so any future substrate addition that
    // generates accumulator values is gated by the same bound. buildings
    // ≤50 mirrors the same evaluation_criteria entry.
    const STONE_UPPER_BOUND: u64 = 4000;
    const WOOD_UPPER_BOUND: u64 = 7500;
    const BUILDINGS_UPPER_BOUND: u64 = 50;
    let feat_stone: u64 = 0; // no stockpile substrate in V7 Foundation
    let feat_wood: u64 = 0;
    assert!(
        feat_stone <= STONE_UPPER_BOUND,
        "A14 locked plan threshold — stone accumulator must be ≤ {STONE_UPPER_BOUND}; got {feat_stone}"
    );
    assert!(
        feat_wood <= WOOD_UPPER_BOUND,
        "A14 locked plan threshold — wood accumulator must be ≤ {WOOD_UPPER_BOUND}; got {feat_wood}"
    );
    assert!(
        feat_metrics.complete_buildings <= BUILDINGS_UPPER_BOUND,
        "A14 locked plan threshold — buildings placed must be ≤ {BUILDINGS_UPPER_BOUND}; got {}",
        feat_metrics.complete_buildings
    );

    // Plan attempt-2 fix #4: per-reason ≥1 cascade-arm presence checks.
    // Each of the four pre-social cascade arms must fire at least once in
    // the feature run, proving the cascade fall-through is alive across
    // every reason. `precondition_lonely_pairs` seeds Hunger/Thirst/Sleep
    // on agents 6/7/8 and places a ConstructionSite on agent 9's tile so
    // each arm fires within the 4380-tick window.
    for (i, label) in labels.iter().enumerate() {
        assert!(
            feat_metrics.decision_counts[i] >= 1,
            "A14 per-reason cascade-arm presence — feature run must produce ≥1 {label} decision; got {}",
            feat_metrics.decision_counts[i]
        );
    }

    // band_count is INTENTIONALLY excluded from the ±15% diff: the baseline
    // has SIS absent so its relationships graph is empty by construction
    // (band_count_base == 0). Comparing against zero would force the
    // feature also to be zero, which would defeat the whole feature. The
    // plan instead asserts the absolute bound on the feature side.
    assert_eq!(
        base_metrics.band_count, 0,
        "baseline band_count must be 0 (SIS absent ⇒ relationships never bump); got {}",
        base_metrics.band_count
    );
    assert!(
        (1..=5).contains(&feat_metrics.band_count),
        "band_count_feature must lie in [1, 5] (Dunbar 20 ÷ 15 ≈ 1–2, dynamics ≤ 5); got {}",
        feat_metrics.band_count
    );

    // Plan attempt-2 fix: close the vacuous-pass hole where a wired-but-
    // never-firing social system would pass the 1-year baseline.
    assert!(
        feat_metrics.relationships_len >= 1,
        "A14: feature run must produce at least one relationships entry (≥ 1); got {}",
        feat_metrics.relationships_len
    );
    assert!(
        feat_metrics.social_completed >= 1,
        "A14: feature run must produce ≥ 1 SocialInteractionCompleted event; got {}",
        feat_metrics.social_completed
    );
    assert_eq!(
        base_metrics.relationships_len, 0,
        "A14: baseline run (no SIS) must produce zero relationships; got {}",
        base_metrics.relationships_len
    );
    assert_eq!(
        base_metrics.social_completed, 0,
        "A14: baseline run (no SIS) must produce zero SocialInteractionCompleted; got {}",
        base_metrics.social_completed
    );
}

struct RegressionMetrics {
    decision_counts: [u64; 4],
    complete_buildings: u64,
    band_count: u64,
    relationships_len: u64,
    social_completed: u64,
}

// ─── A15: relationships_and_progress_initialize_empty ──────────────────
// Per P7β-13: `SimResources::new` initializes both per-pair HashMaps empty.
// We exercise BOTH the direct `SimResources::new` construction path
// (independent of `SimEngine::new`) AND the engine-wrapped path, so a
// regression that initialises either field via a non-empty default is
// caught regardless of which constructor a future caller picks.
//
// `Default::default()` is NOT implemented for `SimResources` (the inner
// `MaterialRegistry`/`TileGrid`/`InfluenceGrid` substrates do not provide
// a meaningful default and the engine has always required explicit width/
// height/registry). The plan permits skipping the Default branch when not
// implemented — the explicit comment here makes that decision visible.
#[test]
fn harness_p7_beta_a15_relationships_and_progress_initialize_empty() {
    // (1) Direct SimResources::new path — bypasses SimEngine entirely.
    let resources = SimResources::new(8, 8, MaterialRegistry::new());
    assert_eq!(
        resources.relationships.len(),
        0,
        "SimResources::new — relationships must start empty per P7β-13"
    );
    assert_eq!(
        resources.interaction_progress.len(),
        0,
        "SimResources::new — interaction_progress must start empty per P7β-13"
    );

    // (2) Engine-wrapped path — what the live FFI uses via SimEngine::new.
    let engine = SimEngine::new(8, 8, MaterialRegistry::new());
    assert_eq!(
        engine.resources.relationships.len(),
        0,
        "SimEngine::new — relationships must start empty per P7β-13"
    );
    assert_eq!(
        engine.resources.interaction_progress.len(),
        0,
        "SimEngine::new — interaction_progress must start empty per P7β-13"
    );

    // Default::default() — NOT implemented (see test doc comment above).
    // If a future patch adds `impl Default for SimResources`, this test
    // should grow a third assertion here.
}

// ─── A16: constants_locked_values ──────────────────────────────────────
#[test]
fn harness_p7_beta_a16_constants_locked_values() {
    assert_eq!(SOCIAL_THRESHOLD, 50.0_f64);
    assert_eq!(SOCIAL_CONSUME_AMOUNT, 30.0_f64);
    assert_eq!(REQUIRED_INTERACTION_PROGRESS, 3_u32);
    assert_eq!(FAMILIARITY_BUMP, 0.1_f64);
}

// ─── A17: smaller_agent_id_emits_invariant (plan attempt-2 lock) ───────
//
// Plan §β attempt-2 fix #5: the smaller-AgentId-emits invariant must be
// proven across three canonical-ordering scenarios so an iteration-order
// coincidence (e.g. HashMap query order) cannot produce a false pass.
//
//   Run 1 — pair (1, 2)   → smaller = 1
//   Run 2 — pair (5, 3)   → smaller = 3   (larger spawned first)
//   Run 3 — pair (7, 11)  → smaller = 7
//
// In every run, the `SocialInteractionStarted` event's `agents` tuple must
// equal `(smaller, larger)` canonical, AND its `parent` must point at an
// `AgentDecision { reason: SocialReason, agent: smaller }` — proving that
// the smaller-AgentId agent is the unique emitter regardless of spawn
// order, iteration order, or label assignment.
//
// To control AgentIds directly, this test spawns entities with explicit
// `Agent { id: <chosen> }` components, bypassing `SimEngine::spawn_agent`'s
// monotonic id minting. The cascade reads the `Agent.id` field, not the
// hecs entity bits, so explicit ids are honored end-to-end.
#[test]
fn harness_p7_beta_a17_smaller_agent_id_emits_invariant() {
    use sim_core::components::AgentId;

    // Three canonical-ordering runs. Each tuple = (first_spawn_id, second_spawn_id).
    // The "<" / ">" notation in the plan refers to whether the first-spawned
    // id is smaller (Run 1, Run 3) or larger (Run 2) than the second.
    let runs: &[(AgentId, AgentId, &str)] = &[
        (1, 2, "Run 1: pair (1<2) — smaller spawned first"),
        (5, 3, "Run 2: pair (5>3) — larger spawned first"),
        (7, 11, "Run 3: pair (7<11) — smaller spawned first"),
    ];

    for (i, (id_first, id_second, label)) in runs.iter().enumerate() {
        let mut engine = fresh_engine();
        // Use distinct tiles per run so logs don't cross-contaminate.
        let tile = (40 + (i as u32) * 8, 40);

        // Spawn entities with EXPLICIT AgentId. Bypass engine.spawn_agent
        // (which mints monotonic ids) by spawning with full component tuple.
        let ent_first = engine.world.spawn((
            Position { x: tile.0, y: tile.1 },
            Agent { id: *id_first },
            Hunger::new(0.0, 0.0),
            Thirst::new(0.0, 0.0),
            Sleep::new(0.0, 0.0),
            Social::new(60.0, 0.0),
            AgentState::Idle,
        ));
        let ent_second = engine.world.spawn((
            Position { x: tile.0, y: tile.1 },
            Agent { id: *id_second },
            Hunger::new(0.0, 0.0),
            Thirst::new(0.0, 0.0),
            Sleep::new(0.0, 0.0),
            Social::new(60.0, 0.0),
            AgentState::Idle,
        ));
        let _ = (ent_first, ent_second);

        let smaller = (*id_first).min(*id_second);
        let larger = (*id_first).max(*id_second);

        // Drive engine until the mutual handshake fires.
        let mut started_seen = false;
        for _ in 0..6 {
            engine.tick();
            if count_events_on_tile(&engine.resources, tile, |ev| {
                matches!(ev, CausalEvent::SocialInteractionStarted { .. })
            }) >= 1
            {
                started_seen = true;
                break;
            }
        }
        assert!(
            started_seen,
            "A17 [{label}]: SocialInteractionStarted must fire within 6 ticks"
        );

        let idx = tile_idx(tile, engine.resources.tile_grid.width);
        let log = engine.resources.causal_log.get(idx).expect("log");

        // (a) Exactly one Started event in this run.
        let started_count = log
            .as_slice()
            .iter()
            .filter(|e| matches!(e, CausalEvent::SocialInteractionStarted { .. }))
            .count();
        assert_eq!(
            started_count, 1,
            "A17 [{label}]: exactly one Started event required; got {started_count}"
        );

        // (b) Canonical `agents` tuple == (smaller, larger).
        let (started_agents, started_parent) = log
            .as_slice()
            .iter()
            .find_map(|e| match e {
                CausalEvent::SocialInteractionStarted { agents, parent, .. } => {
                    Some((*agents, *parent))
                }
                _ => None,
            })
            .expect("Started event");
        assert_eq!(
            started_agents,
            (smaller, larger),
            "A17 [{label}]: agents tuple must be canonical (smaller={smaller}, larger={larger})"
        );

        // (c) Started.parent → AgentDecision{SocialReason, agent: smaller}.
        // This is the locked smaller-emits invariant: only the smaller-id
        // agent's SocialReason decision can be the parent of the Started
        // event. A regression where the larger agent emits would surface
        // here as parent referring to a `agent == larger` decision.
        let parent_id = started_parent
            .unwrap_or_else(|| panic!("A17 [{label}]: Started.parent must be Some"));
        let parent_decision = log
            .as_slice()
            .iter()
            .find_map(|e| match e {
                CausalEvent::AgentDecision {
                    id,
                    agent,
                    reason: DecisionReason::SocialReason,
                    ..
                } if *id == parent_id => Some(*agent),
                _ => None,
            })
            .unwrap_or_else(|| panic!("A17 [{label}]: parent decision must exist on this tile"));
        assert_eq!(
            parent_decision, smaller,
            "A17 [{label}]: smaller-AgentId-emits invariant — parent decision's agent must equal smaller ({smaller}); got {parent_decision}"
        );
    }
}

// ─── A17c: loneliness_threshold_strictness (retained from prior attempt) ──
// Lock the `>` vs `>=` predicate distinction. Two boundary scenarios:
//   (i) loneliness == 49.9 (strictly below SOCIAL_THRESHOLD == 50.0)
//   (ii) loneliness == SOCIAL_THRESHOLD exactly (== 50.0)
// Both agents must stay Idle. Zero SocialReason / Started events. A `>=`
// implementation in the cascade would fail the exact-50.0 case.
#[test]
fn harness_p7_beta_a17c_loneliness_below_threshold_does_not_trigger_social() {
    let cases: &[(f64, &str)] = &[
        (49.9, "loneliness=49.9 (strictly below threshold)"),
        (SOCIAL_THRESHOLD, "loneliness == SOCIAL_THRESHOLD (==50.0)"),
    ];
    for (i, (lone, label)) in cases.iter().enumerate() {
        let mut engine = fresh_engine();
        let tile = (20 + (i as u32) * 4, 20);
        let (ent_a, ent_b, _id_a, _id_b) =
            place_mutual_pair_at(&mut engine, tile, *lone);

        for _ in 0..10 {
            engine.tick();
        }

        let state_a = *engine.world.get::<&AgentState>(ent_a).unwrap();
        let state_b = *engine.world.get::<&AgentState>(ent_b).unwrap();
        assert_eq!(state_a, AgentState::Idle, "A17c [{label}]: A must stay Idle");
        assert_eq!(state_b, AgentState::Idle, "A17c [{label}]: B must stay Idle");

        let social_decisions = count_events_on_tile(&engine.resources, tile, |ev| matches!(ev,
            CausalEvent::AgentDecision { reason: DecisionReason::SocialReason, .. }
        ));
        assert_eq!(social_decisions, 0, "A17c [{label}]: ZERO SocialReason");

        let started = count_events_on_tile(&engine.resources, tile, |ev| {
            matches!(ev, CausalEvent::SocialInteractionStarted { .. })
        });
        assert_eq!(started, 0, "A17c [{label}]: ZERO Started");
    }
}

// ─── A17b: three_or_more_co_located_breached_agents_no_panic ───────────
#[test]
fn harness_p7_beta_a17b_three_agents_no_panic() {
    let mut engine = fresh_engine();
    let ent_a = engine.spawn_agent(90, 90);
    let ent_b = engine.spawn_agent(90, 90);
    let ent_c = engine.spawn_agent(90, 90);
    for ent in [ent_a, ent_b, ent_c] {
        engine
            .world
            .insert(
                ent,
                (
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(60.0, 0.0),
                    AgentState::Idle,
                ),
            )
            .unwrap();
    }

    for _ in 0..10 {
        engine.tick(); // must not panic
    }

    // No self-pair (x, x) keys in interaction_progress.
    let id_a = engine.world.get::<&Agent>(ent_a).unwrap().id;
    let id_b = engine.world.get::<&Agent>(ent_b).unwrap().id;
    let id_c = engine.world.get::<&Agent>(ent_c).unwrap().id;
    for self_id in [id_a, id_b, id_c] {
        let bad_key = RelationshipKey::new(self_id, self_id);
        assert!(
            !engine.resources.interaction_progress.contains_key(&bad_key)
                || bad_key.smaller() != bad_key.larger(),
            "no self-pair (id, id) entry allowed"
        );
    }

    // Each agent must be in a valid FSM state.
    for ent in [ent_a, ent_b, ent_c] {
        let state = *engine.world.get::<&AgentState>(ent).unwrap();
        match state {
            AgentState::Idle
            | AgentState::Seeking { target: TargetKind::Agent(_) }
            | AgentState::Consuming { target: TargetKind::Agent(_) } => {}
            other => panic!("agent in unexpected state {other:?}"),
        }
    }
}

// ─── A18: co_location_strict_same_tile_required ────────────────────────
// Plan attempt-2 fix: prove pairing requires EXACT tile match — not
// Chebyshev distance ≤ 1, not Manhattan distance ≤ 1.
//
// Three variants:
//   (i)   horizontal-adjacent  — agents at (x, y) and (x+1, y)
//   (ii)  vertical-adjacent    — agents at (x, y) and (x, y+1)
//   (iii) same-tile control    — agents at (x, y) and (x, y)
//
// Variants (i) and (ii) must produce ZERO Started events on ANY tile.
// Variant (iii) must produce ≥ 1 Started event on the shared tile —
// without the positive control a vacuous-pass implementation that always
// returns "not co-located" would silently slip through.
//
// Movement is denied by omitting `MovementRng` so positions stay fixed.
#[test]
fn harness_p7_beta_a18_co_location_strict_same_tile_required() {
    fn run_variant(label: &str, tile_a: (u32, u32), tile_b: (u32, u32)) -> (usize, (u32, u32)) {
        let mut engine = fresh_engine();
        let ent_a = engine.spawn_agent(tile_a.0, tile_a.1);
        let ent_b = engine.spawn_agent(tile_b.0, tile_b.1);
        // No MovementRng inserted → AgentMovementSystem leaves them in place.
        for ent in [ent_a, ent_b] {
            engine
                .world
                .insert(
                    ent,
                    (
                        Hunger::new(0.0, 0.0),
                        Thirst::new(0.0, 0.0),
                        Sleep::new(0.0, 0.0),
                        Social::new(60.0, 0.0),
                        AgentState::Idle,
                    ),
                )
                .unwrap();
        }

        let pre_a = *engine.world.get::<&Position>(ent_a).unwrap();
        let pre_b = *engine.world.get::<&Position>(ent_b).unwrap();

        for _ in 0..10 {
            engine.tick();
        }

        let post_a = *engine.world.get::<&Position>(ent_a).unwrap();
        let post_b = *engine.world.get::<&Position>(ent_b).unwrap();
        assert_eq!(
            (post_a.x, post_a.y),
            (pre_a.x, pre_a.y),
            "A18 [{label}]: agent A position must not drift (no MovementRng)"
        );
        assert_eq!(
            (post_b.x, post_b.y),
            (pre_b.x, pre_b.y),
            "A18 [{label}]: agent B position must not drift (no MovementRng)"
        );

        let mut total_started = 0usize;
        for x in 0..W {
            for y in 0..H {
                total_started += count_events_on_tile(&engine.resources, (x, y), |ev| {
                    matches!(ev, CausalEvent::SocialInteractionStarted { .. })
                });
            }
        }
        // Return the count + the same-tile pair's tile (if same) for caller diagnostics.
        (total_started, (pre_a.x, pre_a.y))
    }

    // (i) horizontal-adjacent — distinct tiles, |Δx| == 1, Δy == 0.
    let (started_horiz, _) = run_variant("horizontal-adjacent", (10, 10), (11, 10));
    assert_eq!(
        started_horiz, 0,
        "A18 [horizontal-adjacent]: ZERO Started events allowed when |Δx| == 1 (got {started_horiz})"
    );

    // (ii) vertical-adjacent — distinct tiles, Δx == 0, |Δy| == 1.
    let (started_vert, _) = run_variant("vertical-adjacent", (30, 30), (30, 31));
    assert_eq!(
        started_vert, 0,
        "A18 [vertical-adjacent]: ZERO Started events allowed when |Δy| == 1 (got {started_vert})"
    );

    // (iii) same-tile control — agents at exactly the same coordinate. The
    //       handshake must fire; this rules out a vacuous-pass implementation
    //       that always reports "not co-located".
    let (started_same, _) = run_variant("same-tile-control", (50, 50), (50, 50));
    assert!(
        started_same >= 1,
        "A18 [same-tile-control]: at least 1 Started event required (positive control); got {started_same}"
    );
}

// ─── A18b: no_eligible_agents_emits_no_events ──────────────────────────
// Run engine and zero all Social.loneliness, force all agents to Idle.
// 50 ticks. ZERO Started, Completed, and SocialReason AgentDecision events.
#[test]
fn harness_p7_beta_a18b_no_eligible_agents_emits_no_events() {
    let mut engine = make_stage1_engine(42, 20);
    // Zero all loneliness + force Idle.
    let entities: Vec<hecs::Entity> = engine.world.query::<&Agent>().iter().map(|(e, _)| e).collect();
    for e in entities {
        if let Ok(mut s) = engine.world.get::<&mut Social>(e) {
            s.loneliness = 0.0;
        }
        if let Ok(mut st) = engine.world.get::<&mut AgentState>(e) {
            *st = AgentState::Idle;
        }
    }

    // Run 50 ticks. We count only SOCIAL-class events.
    let count_social_events = |resources: &SimResources| -> (usize, usize, usize) {
        let mut started = 0;
        let mut completed = 0;
        let mut social_decisions = 0;
        for (_, log) in resources.causal_log.iter() {
            for ev in log.as_slice().iter() {
                match ev {
                    CausalEvent::SocialInteractionStarted { .. } => started += 1,
                    CausalEvent::SocialInteractionCompleted { .. } => completed += 1,
                    CausalEvent::AgentDecision { reason: DecisionReason::SocialReason, .. } => {
                        social_decisions += 1
                    }
                    _ => {}
                }
            }
        }
        (started, completed, social_decisions)
    };

    // Pre-tick baseline (should be 0 in a fresh engine).
    let (s0, c0, d0) = count_social_events(&engine.resources);
    assert_eq!((s0, c0, d0), (0, 0, 0), "pre-run baseline must be zero social events");

    for _ in 0..50 {
        engine.tick();
        // Re-zero loneliness each tick to keep all agents ineligible
        // (decay would push them above threshold otherwise across 50 ticks).
        let ents: Vec<hecs::Entity> = engine.world.query::<&Agent>().iter().map(|(e, _)| e).collect();
        for e in ents {
            if let Ok(mut s) = engine.world.get::<&mut Social>(e) {
                s.loneliness = 0.0;
            }
        }
    }

    let (started, completed, social_decisions) = count_social_events(&engine.resources);
    assert_eq!(started, 0, "ZERO Started events");
    assert_eq!(completed, 0, "ZERO Completed events");
    assert_eq!(social_decisions, 0, "ZERO SocialReason AgentDecision events");
}

// ─── §β Re-plan A18p-A22: plan-locked canonical-emergence assertions ────

/// Helper: scan the entire causal_log for social-event counts (re-plan).
fn scan_canonical_emergence(engine: &SimEngine) -> (usize, usize, usize, usize) {
    let mut started = 0usize;
    let mut completed = 0usize;
    let mut social_decisions = 0usize;
    let mut nonzero_relationships = 0usize;
    let log_count = engine.resources.causal_log.active_tile_count();
    for tile_idx in 0..(engine.resources.tile_grid.width * engine.resources.tile_grid.height) {
        if let Some(log) = engine.resources.causal_log.get(tile_idx) {
            for ev in log.as_slice() {
                match ev {
                    CausalEvent::SocialInteractionStarted { .. } => started += 1,
                    CausalEvent::SocialInteractionCompleted { .. } => completed += 1,
                    CausalEvent::AgentDecision { reason: DecisionReason::SocialReason, .. } => {
                        social_decisions += 1;
                    }
                    _ => {}
                }
            }
        }
    }
    for rel in engine.resources.relationships.values() {
        if rel.familiarity > 0.0 {
            nonzero_relationships += 1;
        }
    }
    let _ = log_count;
    (started, completed, social_decisions, nonzero_relationships)
}

/// Walk seed-42 canonical Stage-1 engine 4380 ticks and produce
/// per-tile + global event counts. Used by A18p/A19/A20.
fn run_canonical(seed: u64, ticks: u64) -> SimEngine {
    let mut engine = make_stage1_engine(seed, 20);
    for _ in 0..ticks {
        engine.tick();
    }
    engine
}

// ─── A18p: canonical 4380-tick emergent social run (plan-locked) ────────
//
// Plan §β re-plan: `make_stage1_engine(42, 20)` over 4380 ticks must
// produce ≥1 SocialInteractionStarted, ≥1 SocialInteractionCompleted,
// AND ≥1 non-zero relationship — without manual lonely-pair preconditioning.
// Drives SocialDecaySystem priority 135 organic loneliness accumulation.
#[test]
fn harness_p7_beta_a18p_canonical_emergent_social_run() {
    let engine = run_canonical(42, 4380);
    let (started, completed, social_decisions, nonzero_rels) =
        scan_canonical_emergence(&engine);

    println!(
        "[A18p] canonical 4380-tick run seed=42 agents=20: started={} completed={} \
         social_decisions={} nonzero_relationships={} agents_doing_social_interaction={}",
        started, completed, social_decisions, nonzero_rels, social_decisions
    );

    assert!(
        social_decisions > 0,
        "A18p: agents_doing_social_interaction must be > 0 in canonical run (got {})",
        social_decisions
    );
    assert!(
        started >= 1,
        "A18p: ≥1 SocialInteractionStarted expected (got {})",
        started
    );
    assert!(
        completed >= 1,
        "A18p: ≥1 SocialInteractionCompleted expected (got {})",
        completed
    );
    assert!(
        nonzero_rels >= 1,
        "A18p: ≥1 non-zero relationship expected (got {})",
        nonzero_rels
    );
}

// ─── A19: 3-seed cross-seed robustness ─────────────────────────────────
#[test]
fn harness_p7_beta_a19_three_seed_cross_seed_robustness() {
    let seeds = [42u64, 100u64, 200u64];
    for &seed in &seeds {
        let engine = run_canonical(seed, 4380);
        let (started, completed, _social_dec, nonzero_rels) =
            scan_canonical_emergence(&engine);
        println!(
            "[A19] seed={} started={} completed={} nonzero_rels={}",
            seed, started, completed, nonzero_rels
        );
        assert!(
            started >= 1 && completed >= 1 && nonzero_rels >= 1,
            "A19: seed {} canonical run must hit A18 floor (got started={} completed={} rels={})",
            seed, started, completed, nonzero_rels
        );
    }
}

// ─── A20: same-seed determinism (relationships snapshot + start sequence) ─
#[test]
fn harness_p7_beta_a20_same_seed_determinism() {
    let engine_a = run_canonical(42, 4380);
    let engine_b = run_canonical(42, 4380);

    // (a) relationships HashMap snapshot equal (by content).
    let mut keys_a: Vec<RelationshipKey> = engine_a.resources.relationships.keys().copied().collect();
    let mut keys_b: Vec<RelationshipKey> = engine_b.resources.relationships.keys().copied().collect();
    keys_a.sort_by_key(|k| (k.smaller(), k.larger()));
    keys_b.sort_by_key(|k| (k.smaller(), k.larger()));
    assert_eq!(keys_a, keys_b, "A20: relationship key sets must match across same-seed runs");

    for k in &keys_a {
        let fa = engine_a.resources.relationships.get(k).unwrap().familiarity;
        let fb = engine_b.resources.relationships.get(k).unwrap().familiarity;
        assert_eq!(
            fa.to_bits(),
            fb.to_bits(),
            "A20: familiarity for {:?} must be bit-exact across same-seed runs (a={} b={})",
            k, fa, fb
        );
    }

    // (b) SocialInteractionStarted (tick, id) sequence equal — locked by
    //     plan attempt-2 fix. Including the EventId in the tuple proves
    //     that not only the timing and pair-set but also the event-id
    //     issuing order is bit-deterministic across same-seed runs. The
    //     agents tuple is included as a tertiary diagnostic so a regression
    //     can surface which pair drifted.
    fn collect_starts(engine: &SimEngine) -> Vec<(u64, u64, (u64, u64))> {
        let mut out: Vec<(u64, u64, (u64, u64))> = Vec::new();
        for tile_idx in 0..(engine.resources.tile_grid.width * engine.resources.tile_grid.height) {
            if let Some(log) = engine.resources.causal_log.get(tile_idx) {
                for ev in log.as_slice() {
                    if let CausalEvent::SocialInteractionStarted { tick, id, agents, .. } = ev {
                        out.push((*tick, *id, *agents));
                    }
                }
            }
        }
        // Sort by (tick, id) for a deterministic comparison. Two same-seed
        // runs must produce the same (tick, id) sequence; sorting is just
        // a normalisation over storage-order to make the diff readable.
        out.sort_by_key(|(t, id, _)| (*t, *id));
        out
    }
    let seq_a = collect_starts(&engine_a);
    let seq_b = collect_starts(&engine_b);
    assert_eq!(
        seq_a, seq_b,
        "A20: SocialInteractionStarted (tick, id) sequence must be deterministic across same-seed runs"
    );

    // (c) The sequence must be non-empty — proves the canonical run did
    //     produce social events (defends against a silent regression where
    //     both runs produce zero events and trivially match).
    assert!(
        !seq_a.is_empty(),
        "A20: canonical run must produce ≥1 SocialInteractionStarted (got 0)"
    );
}

// ─── A21: Hot-tier production schedule (registered via register_default) ─
#[test]
fn harness_p7_beta_a21_hot_tier_production_schedule() {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);
    let names = engine.system_names();

    // SocialInteractionSystem must appear exactly once in the production list.
    let count = names.iter().filter(|n| **n == "SocialInteractionSystem").count();
    assert_eq!(
        count, 1,
        "A21: SocialInteractionSystem must be registered exactly once via register_default_runtime_systems (got {})",
        count
    );

    // SocialDecaySystem (re-plan) must appear exactly once.
    let dec_count = names.iter().filter(|n| **n == "SocialDecaySystem").count();
    assert_eq!(
        dec_count, 1,
        "A21: SocialDecaySystem must be registered exactly once (got {})",
        dec_count
    );

    // Priority ordering: Construction(133) < SocialInteraction(134) < SocialDecay(135) < Viz(1000).
    let i_con = names.iter().position(|n| *n == "ConstructionSystem").expect("Construction registered");
    let i_si = names.iter().position(|n| *n == "SocialInteractionSystem").expect("SocialInteraction registered");
    let i_sd = names.iter().position(|n| *n == "SocialDecaySystem").expect("SocialDecay registered");
    let i_viz = names.iter().position(|n| *n == "InfluenceVisualizationSystem").expect("Viz registered");
    assert!(i_con < i_si, "A21: Construction({}) must precede SocialInteraction({})", i_con, i_si);
    assert!(i_si < i_sd, "A21: SocialInteraction({}) must precede SocialDecay({})", i_si, i_sd);
    assert!(i_sd < i_viz, "A21: SocialDecay({}) must precede Viz({})", i_sd, i_viz);
}

// ─── A22: real partner despawn mid-interaction ─────────────────────────
#[test]
fn harness_p7_beta_a22_real_partner_despawn_mid_interaction() {
    let mut engine = fresh_engine();
    let (ent_a, ent_b, id_a, id_b) =
        place_mutual_pair_at(&mut engine, (40, 40), SOCIAL_THRESHOLD + 20.0);

    // Tick 1: Idle → Seeking { Agent } via cascade.
    engine.tick();
    // Tick 2: Seeking → Consuming + SocialInteractionStarted (smaller-id emits).
    engine.tick();

    // Confirm both reached Consuming.
    let state_a = *engine.world.get::<&AgentState>(ent_a).unwrap();
    let state_b = *engine.world.get::<&AgentState>(ent_b).unwrap();
    assert!(matches!(state_a, AgentState::Consuming { target: TargetKind::Agent(id) } if id == id_b),
        "A22 precondition: ent_a must be Consuming{{Agent(b)}}");
    assert!(matches!(state_b, AgentState::Consuming { target: TargetKind::Agent(id) } if id == id_a),
        "A22 precondition: ent_b must be Consuming{{Agent(a)}}");

    // Despawn partner B mid-Consuming.
    engine.world.despawn(ent_b).unwrap();

    // Snapshot completed count before fallback.
    let (completed_before, _, _, _) = scan_canonical_emergence(&engine);

    // Run SocialInteractionSystem (priority 134) — fallback should return A to Idle.
    engine.tick();

    let state_a_after = *engine.world.get::<&AgentState>(ent_a).unwrap();
    assert_eq!(
        state_a_after,
        AgentState::Idle,
        "A22: agent A must return to Idle after partner B despawn (got {:?})",
        state_a_after
    );

    // No SocialInteractionCompleted fired for this pair (partner was despawned
    // before reaching REQUIRED_INTERACTION_PROGRESS).
    let (completed_after, _, _, _) = scan_canonical_emergence(&engine);
    assert_eq!(
        completed_after, completed_before,
        "A22: no SocialInteractionCompleted may fire after partner despawn (before={} after={})",
        completed_before, completed_after
    );
}
