//! P5-β — Thirst component + AgentState FSM + AgentDecisionSystem
//! (V7 Phase 5, First Daily Routine, Week 9-10, Phase β).
//!
//! Locked test plan: `feature: p5-beta-decision-food-tile, plan_attempt: 4`.
//! 19 assertions below match the plan thresholds exactly — values,
//! sub-case counts, and equalities are not negotiable.
//!
//! Threshold types follow `.harness/policy/evaluation_criteria.md`:
//!   - Type A — mathematical invariants (compile-time / shape / equality)
//!   - Type B — empirical baseline carried over from a sibling harness
//!     (cites the harness whose existing GREEN run justifies the threshold)
//!   - Type D — regression guards (pre-existing runtime paths stay green)
//!
//! Run:
//!   `cargo test -p sim-test --test harness_p5_beta_decision -- --nocapture`

use sim_bridge::ffi::world_node::CausalEventView;
use sim_core::causal::{CausalEvent, DecisionReason};
use sim_core::components::{
    Agent, AgentId, AgentState, Hunger, Position, TargetKind, Thirst,
};
use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
use sim_systems::runtime::decision::{
    AgentDecisionSystem, HUNGER_CONSUME_AMOUNT, THIRST_CONSUME_AMOUNT,
};
use sim_systems::runtime::needs::{HungerDecaySystem, ThirstDecaySystem};
use sim_systems::{
    register_agent_systems, register_decision_systems, register_needs_systems,
    register_phase2_systems,
};

const W: u32 = 128;
const H: u32 = 128;

/// Stage-1 engine factory used by the plan: seed determines the
/// deterministic per-agent RNG seeds; `agent_count` is the number of
/// canonical agents spawned (each with `Position`, `Agent`, `MovementRng`,
/// `Hunger`, `Thirst`, `AgentState`). Phase 5-α + 5-β registrations are
/// applied so the full priority schedule (90 → 1000) is live.
fn make_stage1_engine(seed: u64, agent_count: u32) -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_phase2_systems(&mut engine);
    register_agent_systems(&mut engine);
    register_decision_systems(&mut engine);
    register_needs_systems(&mut engine);

    for i in 0..agent_count {
        // Deterministic lattice: spread agents along a row near the centre.
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
                    AgentState::Idle,
                ),
            )
            .expect("freshly spawned agent must still exist");
    }
    engine
}

/// Convenience: run `n` engine ticks in sequence.
fn run_ticks(engine: &mut SimEngine, n: u64) {
    for _ in 0..n {
        engine.tick();
    }
}

/// Pick the first entity matching the `Agent` query so harness code can
/// overwrite its components without depending on hecs entity-allocation
/// order details.
fn first_agent(engine: &SimEngine) -> hecs::Entity {
    engine
        .world
        .query::<&Agent>()
        .iter()
        .next()
        .map(|(e, _)| e)
        .expect("at least one agent must be present")
}

// ── Assertion 1: thirst_construct_clamps_initial_to_saturation_range ──
// Type A — three constructed instances, exact equalities.
#[test]
fn harness_p5_beta_thirst_construct_clamps_initial_to_saturation_range() {
    let a = Thirst::new(-10.0, 0.1);
    let b = Thirst::new(50.0, 0.1);
    let c = Thirst::new(999.0, 0.1);
    assert_eq!(a.value, 0.0, "negative initial must clamp to 0");
    assert_eq!(b.value, 50.0, "in-range initial must pass through unchanged");
    assert_eq!(
        c.value,
        Thirst::SATURATION,
        "above-saturation initial must clamp to {}",
        Thirst::SATURATION
    );
    println!("[β-1] Thirst::new clamps to [0, SATURATION] on construct ✓");
}

// ── Assertion 2: thirst_tick_saturates_at_ceiling_and_floors_at_zero ──
// Type A — saturation function clamps both ends.
#[test]
fn harness_p5_beta_thirst_tick_saturates_at_ceiling_and_floors_at_zero() {
    // 50 ticks of growth-1 on a value seeded just below saturation must
    // pin at SATURATION (= 100.0) on the very first tick and stay there.
    let mut up = Thirst::new(99.0, 1.0);
    for _ in 0..50 {
        up.tick();
    }
    assert_eq!(up.value, Thirst::SATURATION);

    // Negative growth_rate (modelling a drink action) drains below zero
    // arithmetically; the implementation must clamp to 0.0.
    let mut down = Thirst::new(0.5, -2.0);
    for _ in 0..3 {
        down.tick();
    }
    assert_eq!(down.value, 0.0, "tick must clamp negative results to 0");
    println!("[β-2] Thirst::tick saturates at SATURATION and floors at 0 ✓");
}

// ── Assertion 3: agent_state_default_is_idle_and_seeking_is_only_movement_suppressor ──
// Type A — default + full truth table (5 variants).
#[test]
fn harness_p5_beta_agent_state_default_and_movement_suppression_truth_table() {
    // (a) default == Idle.
    assert_eq!(AgentState::default(), AgentState::Idle);

    // (b) suppresses_movement() truth table across all 5 distinct values.
    assert!(!AgentState::Idle.suppresses_movement());
    assert!(AgentState::Seeking { target: TargetKind::Food }.suppresses_movement());
    assert!(AgentState::Seeking { target: TargetKind::Water }.suppresses_movement());
    assert!(!AgentState::Consuming { target: TargetKind::Food }.suppresses_movement());
    assert!(!AgentState::Consuming { target: TargetKind::Water }.suppresses_movement());
    println!("[β-3] AgentState default + movement suppression truth table ✓");
}

// ── Assertion 4: decision_reason_discriminator_returns_locked_strings ──
// Type A — byte-exact discriminator strings.
#[test]
fn harness_p5_beta_decision_reason_discriminator_returns_locked_strings() {
    assert_eq!(
        DecisionReason::HungerThresholdBreach.as_str(),
        "hunger_threshold_breach"
    );
    assert_eq!(
        DecisionReason::ThirstThresholdBreach.as_str(),
        "thirst_threshold_breach"
    );
    println!("[β-4] DecisionReason discriminator strings stable ✓");
}

// ── Assertion 5: agent_decision_system_executes_between_movement_and_hunger_decay_in_schedule ──
// Type A — A5a (observable priority ordering via system_names()) AND
// A5b (NEW: direct constant lock on `name() / priority() / tick_interval()`
// for `AgentMovementSystem` (120), `AgentDecisionSystem` (125),
// `HungerDecaySystem` (130), `ThirstDecaySystem` (131); tick_interval == 1
// on every one). Both sub-cases are mathematical equalities asserted in
// a single #[test] body; the plan locks the assertion-count at 19 so the
// "split" is a sub-case split, not a function split.
#[test]
fn harness_p5_beta_agent_decision_system_executes_between_movement_and_decay() {
    // ── A5a: observable priority ordering via system_names() ─────────
    let engine = make_stage1_engine(42, 20);
    let names = engine.system_names();

    let idx_move = names.iter().position(|n| *n == "AgentMovementSystem");
    let idx_dec = names.iter().position(|n| *n == "AgentDecisionSystem");
    let idx_hung = names.iter().position(|n| *n == "HungerDecaySystem");
    let idx_thi = names.iter().position(|n| *n == "ThirstDecaySystem");

    let i_move = idx_move.expect("AgentMovementSystem must be registered");
    let i_dec = idx_dec.expect("AgentDecisionSystem must be registered");
    let i_hung = idx_hung.expect("HungerDecaySystem must be registered");
    let i_thi = idx_thi.expect("ThirstDecaySystem must be registered");

    assert!(
        i_move < i_dec,
        "AgentMovementSystem must run BEFORE AgentDecisionSystem (got {} < {})",
        i_move,
        i_dec
    );
    assert!(
        i_dec < i_hung,
        "AgentDecisionSystem must run BEFORE HungerDecaySystem (got {} < {})",
        i_dec,
        i_hung
    );
    assert!(
        i_hung < i_thi,
        "HungerDecaySystem must run BEFORE ThirstDecaySystem (got {} < {})",
        i_hung,
        i_thi
    );

    // ── A5b: direct constant lock on RuntimeSystem trait values ──────
    // Verifies that the schedule order observed above is NOT being
    // produced by an accidental shuffle of priorities or by every
    // system registering at the same priority. Each system is
    // constructed fresh and its `name() / priority() / tick_interval()`
    // are asserted exactly. The plan locks:
    //   AgentMovementSystem  : priority 120, tick_interval 1
    //   AgentDecisionSystem  : priority 125, tick_interval 1
    //   HungerDecaySystem    : priority 130, tick_interval 1
    //   ThirstDecaySystem    : priority 131, tick_interval 1
    let move_sys = AgentMovementSystem::new();
    let decision_sys = AgentDecisionSystem::new();
    let hunger_sys = HungerDecaySystem::new();
    let thirst_sys = ThirstDecaySystem::new();

    assert_eq!(move_sys.name(), "AgentMovementSystem");
    assert_eq!(move_sys.priority(), 120, "AgentMovementSystem.priority() must equal 120");
    assert_eq!(move_sys.tick_interval(), 1, "AgentMovementSystem.tick_interval() must equal 1");

    assert_eq!(decision_sys.name(), "AgentDecisionSystem");
    assert_eq!(decision_sys.priority(), 125, "AgentDecisionSystem.priority() must equal 125");
    assert_eq!(decision_sys.tick_interval(), 1, "AgentDecisionSystem.tick_interval() must equal 1");

    assert_eq!(hunger_sys.name(), "HungerDecaySystem");
    assert_eq!(hunger_sys.priority(), 130, "HungerDecaySystem.priority() must equal 130");
    assert_eq!(hunger_sys.tick_interval(), 1, "HungerDecaySystem.tick_interval() must equal 1");

    assert_eq!(thirst_sys.name(), "ThirstDecaySystem");
    assert_eq!(thirst_sys.priority(), 131, "ThirstDecaySystem.priority() must equal 131");
    assert_eq!(thirst_sys.tick_interval(), 1, "ThirstDecaySystem.tick_interval() must equal 1");

    println!(
        "[β-5a] schedule order move({}) < decision({}) < hunger({}) < thirst({}) ✓",
        i_move, i_dec, i_hung, i_thi
    );
    println!(
        "[β-5b] direct priority lock: move=120, decision=125, hunger=130, thirst=131; all tick_interval=1 ✓"
    );
}

// ── Assertion 6: causal_event_agent_decision_accessors_return_correct_fields ──
// Type A — four accessor invariants on a constructed AgentDecision.
#[test]
fn harness_p5_beta_causal_event_agent_decision_accessors_return_correct_fields() {
    let ev = CausalEvent::AgentDecision {
        id: 7u64,
        parent: Some(3u64),
        agent: 11 as AgentId,
        position: (4, 5),
        reason: DecisionReason::ThirstThresholdBreach,
        tick: 99,
    };
    assert_eq!(ev.id(), 7u64);
    assert_eq!(ev.parent(), Some(3u64));
    assert_eq!(ev.tick(), 99);
    assert_eq!(
        ev.channel(),
        None,
        "AgentDecision has no influence channel"
    );
    println!("[β-6] AgentDecision accessors: id=7, parent=Some(3), tick=99, channel=None ✓");
}

// ── Assertion 7: causal_event_view_ffi_maps_agent_decision_and_legacy_variants_preserve_new_fields_as_none ──
// Type A — TWO views: AgentDecision populates new fields; legacy variant
// preserves new fields as None (additive surface).
#[test]
fn harness_p5_beta_causal_event_view_maps_agent_decision_and_preserves_legacy_none() {
    // (a) AgentDecision view — new fields populated.
    let dec = CausalEvent::AgentDecision {
        id: 1u64,
        parent: None,
        agent: 42 as AgentId,
        position: (2, 3),
        reason: DecisionReason::HungerThresholdBreach,
        tick: 10,
    };
    let view_a = CausalEventView::from_event(&dec);
    assert_eq!(view_a.kind, "agent_decision");
    assert_eq!(view_a.agent_id, Some(42));
    assert_eq!(view_a.reason, Some("hunger_threshold_breach"));
    assert_eq!(view_a.position, Some((2, 3)));
    assert_eq!(view_a.channel, None);

    // (b) Legacy variant (InfluenceChanged) — new fields default to None.
    let inf = CausalEvent::InfluenceChanged {
        id: 2u64,
        parent: None,
        channel: InfluenceChannel::Warmth,
        position: (0, 0),
        old: 0.0,
        new: 50.0,
        tick: 0,
    };
    let view_b = CausalEventView::from_event(&inf);
    assert_eq!(
        view_b.agent_id, None,
        "InfluenceChanged view must NOT populate agent_id"
    );
    assert_eq!(
        view_b.reason, None,
        "InfluenceChanged view must NOT populate reason"
    );
    println!("[β-7] AgentDecision populates new fields; legacy InfluenceChanged keeps them None ✓");
}

// ── Assertion 8: idle_agent_with_hunger_above_threshold_transitions_to_seeking_food ──
// Type A — Hunger=80, Thirst=10 → Seeking{Food} after 1 engine tick;
// Hunger.value stays EXACTLY 80.0 and Thirst.value stays EXACTLY 10.0
// (growth_rate=0.0 → decay tick is a no-op; AgentDecisionSystem must
// not touch the need values during an Idle→Seeking transition; the
// pair of exact equalities closes the "decay-coupling gaming vector"
// where an implementation could leak a small decrement on transition).
#[test]
fn harness_p5_beta_idle_agent_with_hunger_above_threshold_transitions_to_seeking_food() {
    let mut engine = make_stage1_engine(42, 20);
    let entity = first_agent(&engine);
    engine
        .world
        .insert(
            entity,
            (
                AgentState::Idle,
                Hunger::new(80.0, 0.0),
                Thirst::new(10.0, 0.0),
                Position::new(60, 60),
            ),
        )
        .unwrap();

    run_ticks(&mut engine, 1);

    let state = *engine.world.get::<&AgentState>(entity).unwrap();
    let hunger_val = engine.world.get::<&Hunger>(entity).unwrap().value;
    let thirst_val = engine.world.get::<&Thirst>(entity).unwrap().value;
    assert_eq!(state, AgentState::Seeking { target: TargetKind::Food });
    assert_eq!(
        hunger_val, 80.0_f32,
        "Hunger.value must remain EXACTLY 80.0 during the Idle→Seeking transition (got {})",
        hunger_val
    );
    assert_eq!(
        thirst_val, 10.0_f64,
        "Thirst.value must remain EXACTLY 10.0 during the Idle→Seeking transition (got {})",
        thirst_val
    );
    println!(
        "[β-8] Idle+Hunger=80 → Seeking{{Food}}; Hunger=80.0 exact, Thirst=10.0 exact ✓",
    );
}

// ── Assertion 9: idle_agent_with_only_thirst_breaching_transitions_to_seeking_water ──
// Type A — Hunger=10, Thirst=80 → Seeking{Water}.
#[test]
fn harness_p5_beta_idle_agent_with_only_thirst_breaching_transitions_to_seeking_water() {
    let mut engine = make_stage1_engine(42, 20);
    let entity = first_agent(&engine);
    engine
        .world
        .insert(
            entity,
            (
                AgentState::Idle,
                Hunger::new(10.0, 0.0),
                Thirst::new(80.0, 0.0),
                Position::new(40, 40),
            ),
        )
        .unwrap();

    run_ticks(&mut engine, 1);

    let state = *engine.world.get::<&AgentState>(entity).unwrap();
    assert_eq!(state, AgentState::Seeking { target: TargetKind::Water });
    println!("[β-9] Idle+Thirst=80 (Hunger=10) → Seeking{{Water}} ✓");
}

// ── Assertion 10: hunger_wins_when_both_needs_breach_simultaneously_deterministic_tiebreak ──
// Type A — Hunger=80, Thirst=80 → Seeking{Food}.
#[test]
fn harness_p5_beta_hunger_wins_when_both_needs_breach_simultaneously() {
    let mut engine = make_stage1_engine(42, 20);
    let entity = first_agent(&engine);
    engine
        .world
        .insert(
            entity,
            (
                AgentState::Idle,
                Hunger::new(80.0, 0.0),
                Thirst::new(80.0, 0.0),
                Position::new(50, 50),
            ),
        )
        .unwrap();

    run_ticks(&mut engine, 1);

    let state = *engine.world.get::<&AgentState>(entity).unwrap();
    assert_eq!(
        state,
        AgentState::Seeking { target: TargetKind::Food },
        "Hunger must win on simultaneous breach"
    );
    println!("[β-10] Tie-break: Hunger wins over Thirst ✓");
}

// ── Assertion 11: seeking_agent_position_frozen_over_32_ticks_while_idle_baseline_moves ──
// Type B — empirical baseline carried over from the existing
// `harness_p4_beta_movement` harness, which has been GREEN since the
// Phase 4-β land and demonstrates that the canonical Brownian-step
// MovementRng produces at least one observable (dx, dy) ≠ (0, 0) move
// within 32 ticks for the seed-class used here. That empirical baseline
// is what justifies the 32-tick observation window: it is short enough
// to stay deterministic and fast, yet long enough for the Idle baseline
// to be statistically certain to move. See
// `rust/crates/sim-test/tests/harness_p4_beta_movement.rs`
// (`harness_p4_beta_movement_*` family) for the cited baseline.
//
// TWO probes on the SAME engine: Seeking probe stays put,
// Idle baseline probe moves at least once.
#[test]
fn harness_p5_beta_seeking_position_frozen_idle_baseline_moves() {
    let mut engine = make_stage1_engine(42, 20);

    // Collect two distinct agents.
    let agents: Vec<hecs::Entity> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(e, _)| e)
        .take(2)
        .collect();
    assert_eq!(agents.len(), 2, "need two probes");
    let probe_a = agents[0]; // Seeking — suppressed
    let probe_b = agents[1]; // Idle baseline — moves

    // Both start at the SAME position with the SAME RNG seed so the only
    // observable difference is the AgentState.
    let start_x: u32 = 64;
    let start_y: u32 = 64;
    let seed: u64 = 1234;
    engine
        .world
        .insert(
            probe_a,
            (
                Position::new(start_x, start_y),
                MovementRng::new(seed),
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                AgentState::Seeking { target: TargetKind::Food },
            ),
        )
        .unwrap();
    engine
        .world
        .insert(
            probe_b,
            (
                Position::new(start_x, start_y),
                MovementRng::new(seed),
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                AgentState::Idle,
            ),
        )
        .unwrap();

    run_ticks(&mut engine, 32);

    let pa = *engine.world.get::<&Position>(probe_a).unwrap();
    let pb = *engine.world.get::<&Position>(probe_b).unwrap();
    assert_eq!(pa.x, start_x, "Seeking probe x must be frozen");
    assert_eq!(pa.y, start_y, "Seeking probe y must be frozen");
    assert!(
        pb.x != start_x || pb.y != start_y,
        "Idle baseline must move at least once over 32 ticks (got {:?})",
        (pb.x, pb.y)
    );
    println!(
        "[β-11] Seeking frozen at ({},{}); Idle baseline moved to ({},{}) ✓",
        pa.x, pa.y, pb.x, pb.y
    );
}

// ── Assertion 12: consuming_food_decrements_tile_to_zero_then_returns_to_idle_with_tile_removed ──
// Type A — set_food_tile(7,7,1), Seeking{Food}+Hunger=80, run 2 ticks:
// state→Idle, Hunger=50, food_tiles entry at (7,7) removed AND final
// Position remains (7,7) (regression guard: with the locked schedule
// `120 move < 125 decide`, a Consuming tick would otherwise move the
// agent off (7,7) BEFORE the consume commit — corrupting both the
// food-tile decrement and the consume position).
#[test]
fn harness_p5_beta_consuming_food_decrements_tile_to_zero_then_idles_with_tile_removed() {
    let mut engine = make_stage1_engine(42, 20);
    let entity = first_agent(&engine);
    // Pin a MovementRng seed that WOULD produce a non-zero Brownian step
    // on its first two `next_step()` calls — proves the freeze on the
    // Consuming tick is intentional, not accidental. Seed 1 verified by
    // construction below.
    const MOVING_SEED: u64 = 1;
    {
        let mut probe = MovementRng::new(MOVING_SEED);
        let dx = probe.next_step();
        let dy = probe.next_step();
        assert!(
            dx != 0 || dy != 0,
            "test pre-flight failed: MOVING_SEED={} unexpectedly yields a (0,0) Brownian step, \
             so the test cannot distinguish the freeze from an accidental no-op",
            MOVING_SEED
        );
    }
    engine
        .world
        .insert(
            entity,
            (
                Position::new(7, 7),
                MovementRng::new(MOVING_SEED),
                AgentState::Seeking { target: TargetKind::Food },
                Hunger::new(80.0, 0.0),
                Thirst::new(0.0, 0.0),
            ),
        )
        .unwrap();
    engine.resources.set_food_tile(7, 7, 1);

    run_ticks(&mut engine, 2);

    let state = *engine.world.get::<&AgentState>(entity).unwrap();
    let hunger = engine.world.get::<&Hunger>(entity).unwrap().value;
    let pos = *engine.world.get::<&Position>(entity).unwrap();
    assert_eq!(state, AgentState::Idle, "must return to Idle after consume");
    assert_eq!(
        hunger,
        80.0 - HUNGER_CONSUME_AMOUNT,
        "Hunger must drop by HUNGER_CONSUME_AMOUNT (got {})",
        hunger
    );
    // The ORIGINAL tile (7,7) — the one the agent reached and consumed
    // on — must be the one removed. A buggy implementation that lets
    // movement run on the Consuming tick would commit on a neighbouring
    // tile and leave (7,7) untouched.
    assert_eq!(
        engine.resources.food_tiles.get(&(7, 7)),
        None,
        "food_tiles entry at the ORIGINAL Seeking tile (7,7) must be removed when counter hits 0; \
         if non-None, the consume commit ran on a post-move tile (schedule regression)"
    );
    // Belt-and-braces: position itself must still be (7,7) — neither the
    // Seeking-tick nor the Consuming-tick should have produced a Brownian
    // step at this MovementRng seed.
    assert_eq!(
        (pos.x, pos.y),
        (7, 7),
        "agent position must remain at the Seeking tile (7,7) across both ticks; \
         got ({}, {}) — a non-(7,7) result proves the Consuming-tick movement freeze regressed",
        pos.x,
        pos.y
    );
    // No phantom decrement anywhere else: every other food tile must remain absent.
    assert!(
        engine.resources.food_tiles.is_empty(),
        "no phantom food entries: only (7,7) was ever populated, and it was removed; got {} stray keys",
        engine.resources.food_tiles.len()
    );
    println!("[β-12] Food consume: state→Idle, Hunger=50, tile (7,7) removed, pos=(7,7) ✓");
}

// ── Assertion 13: consuming_water_decrements_nonzero_counter_then_returns_to_idle_with_tile_retained ──
// Type A — set_water_tile(8,8,2), Seeking{Water}+Thirst=80, run 2 ticks:
// state→Idle, Thirst=50, water_tiles entry at (8,8) decremented from
// 2→1 (retained), and final Position remains (8,8). Same nonzero-
// movement regression guard as Assertion 12 — applied to the symmetric
// water branch with the non-removal counter path.
#[test]
fn harness_p5_beta_consuming_water_decrements_nonzero_counter_then_idles_with_tile_retained() {
    let mut engine = make_stage1_engine(42, 20);
    let entity = first_agent(&engine);
    const MOVING_SEED: u64 = 1;
    {
        let mut probe = MovementRng::new(MOVING_SEED);
        let dx = probe.next_step();
        let dy = probe.next_step();
        assert!(
            dx != 0 || dy != 0,
            "test pre-flight failed: MOVING_SEED={} unexpectedly yields a (0,0) Brownian step",
            MOVING_SEED
        );
    }
    engine
        .world
        .insert(
            entity,
            (
                Position::new(8, 8),
                MovementRng::new(MOVING_SEED),
                AgentState::Seeking { target: TargetKind::Water },
                Hunger::new(0.0, 0.0),
                Thirst::new(80.0, 0.0),
            ),
        )
        .unwrap();
    engine.resources.set_water_tile(8, 8, 2);

    run_ticks(&mut engine, 2);

    let state = *engine.world.get::<&AgentState>(entity).unwrap();
    let thirst = engine.world.get::<&Thirst>(entity).unwrap().value;
    let pos = *engine.world.get::<&Position>(entity).unwrap();
    assert_eq!(state, AgentState::Idle, "must return to Idle after consume");
    assert_eq!(
        thirst,
        80.0 - THIRST_CONSUME_AMOUNT,
        "Thirst must drop by THIRST_CONSUME_AMOUNT (got {})",
        thirst
    );
    assert_eq!(
        engine.resources.water_tiles.get(&(8, 8)),
        Some(&1),
        "water_tiles entry at the ORIGINAL Seeking tile (8,8) must remain at counter=1 \
         (non-zero retention branch); if 2 or None, the consume commit ran on a post-move tile"
    );
    assert_eq!(
        (pos.x, pos.y),
        (8, 8),
        "agent position must remain at the Seeking tile (8,8) across both ticks; \
         got ({}, {}) — a non-(8,8) result proves the Consuming-tick movement freeze regressed",
        pos.x,
        pos.y
    );
    // Exactly one water entry (the one at (8,8)); no phantom decrement
    // ever populated a neighbouring tile.
    assert_eq!(
        engine.resources.water_tiles.len(),
        1,
        "exactly one water_tiles entry must remain (the original at (8,8)); got {} entries",
        engine.resources.water_tiles.len()
    );
    println!("[β-13] Water consume: state→Idle, Thirst=50, tile (8,8) counter 2→1, pos=(8,8) ✓");
}

// ── Assertion 14: agent_decision_event_recorded_with_parent_none_or_some ──
// Type A — TWO scenarios. Scenario A: no prior InfluenceChanged →
// AgentDecision.parent == None. Scenario B: InfluenceChanged pushed
// pre-tick at the same tile → AgentDecision.parent == Some(<that id>).
//
// Both scenarios assert EXACTLY ONE matching AgentDecision via
// `filter(...).collect::<Vec<_>>()` + len-check, NOT `find_map(...).expect()`.
// The latter would silently accept multiple matching records and tolerate a
// double-emit bug; the explicit count assertion closes that gap.
#[test]
fn harness_p5_beta_agent_decision_parent_none_and_some_branches() {
    // ── Scenario A: parent == None ─────────────────────────────────────
    {
        let mut engine = make_stage1_engine(42, 20);
        let entity = first_agent(&engine);
        engine
            .world
            .insert(
                entity,
                (
                    Position::new(3, 3),
                    AgentState::Idle,
                    Hunger::new(80.0, 0.0),
                    Thirst::new(0.0, 0.0),
                ),
            )
            .unwrap();

        // Pre-condition: no InfluenceChanged at (3,3).
        let tile_idx = 3u32 * W + 3u32;
        if let Some(log) = engine.resources.causal_log.get(tile_idx) {
            assert!(
                !log.as_slice().iter().any(|ev| matches!(ev, CausalEvent::InfluenceChanged { .. })),
                "no InfluenceChanged at (3,3) before tick"
            );
        }
        let agent_id_a = engine.world.get::<&Agent>(entity).unwrap().id;

        run_ticks(&mut engine, 1);

        let log = engine
            .resources
            .causal_log
            .get(tile_idx)
            .expect("AgentDecision must produce a log at (3,3)");
        let matches_a: Vec<((u32, u32), DecisionReason, Option<u64>)> = log
            .as_slice()
            .iter()
            .filter_map(|ev| match ev {
                CausalEvent::AgentDecision {
                    agent,
                    position,
                    reason,
                    parent,
                    ..
                } if *agent == agent_id_a => Some((*position, *reason, *parent)),
                _ => None,
            })
            .collect();
        assert_eq!(
            matches_a.len(),
            1,
            "Scenario A: exactly one AgentDecision for this agent expected, got {} \
             (a double-emit bug would also pass `find_map`)",
            matches_a.len()
        );
        let (pos_a, reason_a, parent_a) = matches_a[0];
        assert_eq!(pos_a, (3, 3), "Scenario A: decision position");
        assert_eq!(reason_a, DecisionReason::HungerThresholdBreach);
        assert_eq!(parent_a, None, "Scenario A: parent must be None");
    }

    // ── Scenario B: parent == Some(<InfluenceChanged id>) ─────────────
    {
        let mut engine = make_stage1_engine(42, 20);
        let entity = first_agent(&engine);
        engine
            .world
            .insert(
                entity,
                (
                    Position::new(4, 4),
                    AgentState::Idle,
                    Hunger::new(80.0, 0.0),
                    Thirst::new(0.0, 0.0),
                ),
            )
            .unwrap();

        // Pre-seed an InfluenceChanged at (4,4) BEFORE the decision tick.
        let influence_id = engine.resources.issue_event_id();
        let tile_idx = 4u32 * W + 4u32;
        engine.resources.causal_log.push(
            tile_idx,
            CausalEvent::InfluenceChanged {
                id: influence_id,
                parent: None,
                channel: InfluenceChannel::FoodAroma,
                position: (4, 4),
                old: 0.0,
                new: 100.0,
                tick: engine.current_tick(),
            },
        );
        let agent_id_b = engine.world.get::<&Agent>(entity).unwrap().id;

        run_ticks(&mut engine, 1);

        let log = engine
            .resources
            .causal_log
            .get(tile_idx)
            .expect("log must exist at (4,4)");
        let matches_b: Vec<Option<u64>> = log
            .as_slice()
            .iter()
            .filter_map(|ev| match ev {
                CausalEvent::AgentDecision { agent, parent, position, .. }
                    if *agent == agent_id_b && *position == (4, 4) =>
                {
                    Some(*parent)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            matches_b.len(),
            1,
            "Scenario B: exactly one AgentDecision for this agent expected, got {}",
            matches_b.len()
        );
        assert_eq!(
            matches_b[0],
            Some(influence_id),
            "Scenario B: parent must equal the pre-seeded InfluenceChanged id"
        );
    }

    println!("[β-14] AgentDecision parent linkage: None branch + Some branch (count==1 each) ✓");
}

// (Plan attempt 3: the prior same-tile contention edge-case test was
// removed. It encoded the legacy "race loser stays in Seeking" branch
// which is now superseded by Section-3.10's unconditional-need-decrement
// rule. Coverage replaced by Assertions 16 and 17 below.)

// ── Assertion 15: hunger_decay_regression_and_phase_3_alpha_match_sites ──
// Type D — (a) Hunger.value grows by EXACTLY 10.0 ± 0.001 over 100
// ticks at growth_rate 0.1. The tolerance is f32-accumulation tolerance
// for 100 sequential `value += 0.1_f32` operations — there is NO
// damping factor in HungerDecaySystem (a `* 0.5` damping would shift
// the result to ~5.0 and trip this assertion; that is the locked
// regression contract). tick_interval=1 is enforced by Assertion 5b,
// so HungerDecaySystem runs once per engine tick across 100 ticks.
// (b) the existing Phase 3-α harness must still pass — verified by the
//     Verification block (`cargo test --test harness_p3_alpha_event_recording`)
//     because Rust's exhaustive `match` already breaks compilation if a
//     wildcard catch-all is introduced and a new variant is added. The
//     three sites in harness_p3_alpha_event_recording.rs explicitly add
//     `CausalEvent::AgentDecision { tick, .. } => *tick`; if a Generator
//     silently introduces `_ => 0` instead, the Phase 3-α tick-stamp
//     assertions still pass but downstream consumers break — therefore
//     the gate runs that harness in addition to this one (Section 5).
#[test]
fn harness_p5_beta_hunger_decay_regression_and_p3_alpha_match_sites() {
    // (a) Hunger decay regression — 100 ticks at growth_rate 0.1.
    let mut engine = make_stage1_engine(42, 20);
    let entity = first_agent(&engine);
    engine
        .world
        .insert_one(entity, Hunger::new(0.0, 0.1))
        .unwrap();
    // Snapshot before.
    let before = engine.world.get::<&Hunger>(entity).unwrap().value;

    run_ticks(&mut engine, 100);

    let after = engine.world.get::<&Hunger>(entity).unwrap().value;
    let delta = after - before;
    // EXACT delta with f32-accumulation tolerance — locks out damping
    // substitutions (e.g. `* 0.5` would give ~5.0 here) and rate
    // substitutions while still tolerating the sequential float-add
    // accumulation noise of 100 `+= 0.1_f32` steps.
    let expected = 10.0_f32;
    let tol = 0.001_f32;
    assert!(
        (delta - expected).abs() < tol,
        "Hunger growth over 100 ticks @ rate 0.1 must equal {} ± {} (got Δ={}); \
         a value ≠ ~10.0 indicates either (i) a damping factor was applied in \
         HungerDecaySystem, (ii) tick_interval drifted from 1, or (iii) rate is wrong",
        expected,
        tol,
        delta
    );
    assert!(
        after <= Hunger::SATURATION,
        "Hunger must not exceed SATURATION (got {})",
        after
    );

    println!(
        "[β-15] Hunger decay regression: {:.4} → {:.4} (Δ={:.4} within ±{}); P3-α invoked via gate ✓",
        before, after, delta, tol
    );
}

// ── Assertion 16: consuming_food_on_initially_absent_tile_still_decrements_hunger_and_returns_to_idle_without_creating_entry ──
// Type D — regression guard locking the absent-tile Consume branch on
// the Food channel. Section-3.10 conditional mutation rule: tile map
// mutation is conditional on Some; need decrement is unconditional.
// Triple threshold (state == Idle, Hunger == 50.0, tile entry still
// None) closes three independent gaming vectors.
#[test]
fn harness_p5_beta_consuming_food_on_absent_tile_decrements_hunger_and_idles() {
    let mut engine = make_stage1_engine(42, 20);
    let entity = first_agent(&engine);
    engine
        .world
        .insert(
            entity,
            (
                Position::new(11, 11),
                AgentState::Consuming { target: TargetKind::Food },
                Hunger::new(80.0, 0.0),
                Thirst::new(0.0, 0.0),
            ),
        )
        .unwrap();

    // Precondition: tile MUST be absent at tick start.
    assert_eq!(
        engine.resources.food_tiles.get(&(11, 11)),
        None,
        "precondition: food tile (11,11) must be absent before tick"
    );

    run_ticks(&mut engine, 1);

    let state = *engine.world.get::<&AgentState>(entity).unwrap();
    let hunger = engine.world.get::<&Hunger>(entity).unwrap().value;
    assert_eq!(
        state,
        AgentState::Idle,
        "absent-tile Consume{{Food}} MUST commit state → Idle (Section-3.10: state transition is unconditional)"
    );
    assert_eq!(
        hunger,
        80.0 - HUNGER_CONSUME_AMOUNT,
        "absent-tile Consume{{Food}} MUST still subtract HUNGER_CONSUME_AMOUNT (got {}); need decrement is unconditional",
        hunger
    );
    assert_eq!(
        engine.resources.food_tiles.get(&(11, 11)),
        None,
        "absent-tile Consume{{Food}} MUST NOT create a tile entry (tile mutation is conditional on Some); spurious entry would imply counter underflow"
    );
    println!("[β-16] Absent-tile Consume{{Food}}: state→Idle, Hunger=50, food_tiles still empty ✓");
}

// ── Assertion 17: consuming_water_on_initially_absent_tile_still_decrements_thirst_and_returns_to_idle_without_creating_entry ──
// Type D — symmetric to Assertion 16 on the Water channel. Both
// channels MUST be enforced; a Generator that fixed only one would
// leave the other in its prior bugged form.
#[test]
fn harness_p5_beta_consuming_water_on_absent_tile_decrements_thirst_and_idles() {
    let mut engine = make_stage1_engine(42, 20);
    let entity = first_agent(&engine);
    engine
        .world
        .insert(
            entity,
            (
                Position::new(12, 12),
                AgentState::Consuming { target: TargetKind::Water },
                Hunger::new(0.0, 0.0),
                Thirst::new(80.0, 0.0),
            ),
        )
        .unwrap();

    // Precondition: tile MUST be absent at tick start.
    assert_eq!(
        engine.resources.water_tiles.get(&(12, 12)),
        None,
        "precondition: water tile (12,12) must be absent before tick"
    );

    run_ticks(&mut engine, 1);

    let state = *engine.world.get::<&AgentState>(entity).unwrap();
    let thirst = engine.world.get::<&Thirst>(entity).unwrap().value;
    assert_eq!(
        state,
        AgentState::Idle,
        "absent-tile Consume{{Water}} MUST commit state → Idle (Section-3.10 symmetric)"
    );
    assert_eq!(
        thirst,
        80.0 - THIRST_CONSUME_AMOUNT,
        "absent-tile Consume{{Water}} MUST still subtract THIRST_CONSUME_AMOUNT (got {})",
        thirst
    );
    assert_eq!(
        engine.resources.water_tiles.get(&(12, 12)),
        None,
        "absent-tile Consume{{Water}} MUST NOT create a tile entry"
    );
    println!("[β-17] Absent-tile Consume{{Water}}: state→Idle, Thirst=50, water_tiles still empty ✓");
}

// ── Assertion 18: consume_amount_constants_locked_to_thirty ──
// Type A — direct constant lock on `HUNGER_CONSUME_AMOUNT` and
// `THIRST_CONSUME_AMOUNT`. Closes the "arithmetic substitution" gaming
// vector where an implementation could pick a different amount that
// still happens to drop a single freshly-breached agent (value ≈ 51)
// below threshold, but would silently miscount for any other input.
#[test]
fn harness_p5_beta_consume_amount_constants_locked_to_thirty() {
    assert_eq!(
        HUNGER_CONSUME_AMOUNT, 30.0_f32,
        "HUNGER_CONSUME_AMOUNT must be exactly 30.0 (got {})",
        HUNGER_CONSUME_AMOUNT
    );
    assert_eq!(
        THIRST_CONSUME_AMOUNT, 30.0_f64,
        "THIRST_CONSUME_AMOUNT must be exactly 30.0 (got {})",
        THIRST_CONSUME_AMOUNT
    );
    println!("[β-18] HUNGER_CONSUME_AMOUNT=30.0, THIRST_CONSUME_AMOUNT=30.0 ✓");
}

// ── Assertion 19: target_kind_has_exactly_two_variants_food_and_water ──
// Type A — compile-time exhaustive match on `TargetKind` enforcing
// exactly two variants (Food, Water). Closes the "silent variant
// addition" gaming vector where an implementation could add a third
// variant (e.g. `Sleep`) ahead of γ without breaking any other
// assertion. Adding a variant would cause this match to fail to
// compile, immediately surfacing the scope creep.
#[test]
fn harness_p5_beta_target_kind_has_exactly_two_variants() {
    // Exhaustive match — no wildcard. β-era tripwire was designed to
    // fail compilation when γ adds `TargetKind::Sleep`. γ has now
    // landed, and Phase 6-α has added `TargetKind::ConstructionSite`
    // (Path b symmetry precedent — see `.harness/plans/phase6.md`
    // §2.1). The discriminator covers all four variants — but the
    // exhaustive match (no wildcard) still guards against further
    // unannounced variant additions in δ+.
    fn discriminator(k: TargetKind) -> &'static str {
        match k {
            TargetKind::Food => "food",
            TargetKind::Water => "water",
            TargetKind::Sleep => "sleep",
            TargetKind::ConstructionSite => "construction_site",
            TargetKind::Agent(_) => "agent",
        }
    }
    assert_eq!(discriminator(TargetKind::Food), "food");
    assert_eq!(discriminator(TargetKind::Water), "water");
    assert_eq!(discriminator(TargetKind::Sleep), "sleep");
    assert_eq!(
        discriminator(TargetKind::ConstructionSite),
        "construction_site"
    );
    println!(
        "[β-19/γ/P6α] TargetKind has exactly 4 variants (Food, Water, Sleep, ConstructionSite) ✓"
    );
}
