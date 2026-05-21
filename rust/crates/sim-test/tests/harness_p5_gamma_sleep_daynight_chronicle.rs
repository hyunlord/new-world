//! V7 Phase 5-γ — Sleep + day/night cycle + chronicle harness
//! (closure milestone for Phase 5: First Daily Routine).
//!
//! plan_attempt: 2
//! assertions: 13 (1, 2, 3, 4, 5, 5b, 6, 7, 8, 8b, 9, 9b, 9c, 10, 10b, 10c, 11, 12, 13)
//! lane: --full
//!
//! Type A — invariants (clamp / serde / enum surface / metadata).
//! Type D — regression guards (Phase 4 / Phase 5-α / Phase 5-β stay green).
//!
//! Run:
//!   `cargo test -p sim-test --test harness_p5_gamma_sleep_daynight_chronicle -- --nocapture`

use sim_core::causal::{CausalEvent, DecisionReason};
use sim_core::components::{
    Agent, AgentState, Hunger, Position, Sleep, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
use sim_systems::runtime::decision::{
    AgentDecisionSystem, FATIGUE_CONSUME_AMOUNT,
};
use sim_systems::runtime::needs::{HungerDecaySystem, SleepDecaySystem, ThirstDecaySystem};

fn fresh_engine() -> SimEngine {
    SimEngine::new(32, 32, MaterialRegistry::new())
}

fn fresh_engine_sized(w: u32, h: u32) -> SimEngine {
    SimEngine::new(w, h, MaterialRegistry::new())
}

// ─── Assertion 1: sleep_component_clamp_and_tick_invariants ─────────────
// Type A — pure component invariants.
#[test]
fn harness_p5_gamma_sleep_clamps_and_ticks() {
    // (a) lower clamp on construction
    // Type: f64 exact integer arithmetic
    assert_eq!(Sleep::new(-10.0, 0.0).fatigue, 0.0);

    // (b) upper clamp on construction
    assert_eq!(Sleep::new(150.0, 0.0).fatigue, 100.0);

    // (c) additive growth using the field (NOT an argument)
    let mut s = Sleep::new(10.0, 5.0);
    s.tick();
    assert_eq!(s.fatigue, 15.0);
    s.tick();
    assert_eq!(s.fatigue, 20.0);

    // (d) saturation clamp on growth
    let mut s = Sleep::new(99.0, 5.0);
    s.tick();
    assert_eq!(s.fatigue, 100.0);

    // (e) monotonic non-decreasing + bit-exact accumulation at 0.5
    let mut s = Sleep::new(0.5, 0.5);
    let mut prev = s.fatigue;
    for step in 0..200 {
        s.tick();
        let curr = s.fatigue;
        assert!(
            prev <= curr,
            "fatigue must be monotonically non-decreasing (step={}, prev={}, curr={})",
            step,
            prev,
            curr
        );
        prev = curr;
    }
    assert_eq!(s.fatigue, 100.0);

    // (f) negative growth_rate decrements; lower clamp at 0 applies
    let mut s = Sleep::new(20.0, -3.0);
    s.tick();
    assert_eq!(s.fatigue, 17.0);
    let mut s = Sleep::new(2.0, -5.0);
    s.tick();
    assert_eq!(s.fatigue, 0.0);

    // (g) named constant exists and equals 100
    assert_eq!(Sleep::SATURATION, 100.0_f64);
}

// ─── Assertion 2: sleep_serde_round_trip ─────────────────────────────────
// Type A — serde round-trip invariant.
#[test]
fn harness_p5_gamma_sleep_serde_round_trip() {
    let s = Sleep::new(42.5, 0.7);
    // Type: Sleep PartialEq via derived f64-component compare
    let encoded = ron::to_string(&s).unwrap();
    let decoded: Sleep = ron::from_str(&encoded).unwrap();
    assert_eq!(s, decoded);
}

// ─── Assertion 3: target_kind_sleep_variant_surface ──────────────────────
// Type A — enum surface checks.
#[test]
fn harness_p5_gamma_target_kind_sleep_variant_exists() {
    // (a) TargetKind::Sleep constructible
    let _k = TargetKind::Sleep;

    // (b) serde round-trip for Seeking and Consuming variants
    let cases = [
        AgentState::Seeking { target: TargetKind::Sleep },
        AgentState::Consuming { target: TargetKind::Sleep },
    ];
    for state in cases {
        let encoded = ron::to_string(&state).unwrap();
        let decoded: AgentState = ron::from_str(&encoded).unwrap();
        assert_eq!(state, decoded);
    }

    // (c) Seeking{Sleep}.suppresses_movement == true
    assert!(AgentState::Seeking { target: TargetKind::Sleep }.suppresses_movement());

    // (d) Consuming{Sleep}.suppresses_movement == false
    assert!(!AgentState::Consuming { target: TargetKind::Sleep }.suppresses_movement());
}

// ─── Assertion 4: decision_reason_fatigue_discriminator ──────────────────
// Type A — discriminator strings part of FFI contract.
#[test]
fn harness_p5_gamma_decision_reason_fatigue_discriminator() {
    assert_eq!(
        DecisionReason::FatigueThresholdBreach.as_str(),
        "fatigue_threshold_breach"
    );
    assert_eq!(
        DecisionReason::HungerThresholdBreach.as_str(),
        "hunger_threshold_breach"
    );
    assert_eq!(
        DecisionReason::ThirstThresholdBreach.as_str(),
        "thirst_threshold_breach"
    );
}

// ─── Assertion 5: sleep_decay_system_metadata ────────────────────────────
// Type A — SimSystem trait surface.
#[test]
fn harness_p5_gamma_sleep_decay_system_metadata() {
    let s = SleepDecaySystem::new();
    assert_eq!(s.priority(), 132);
    assert_eq!(s.tick_interval(), 1);
    assert_eq!(s.name(), "SleepDecaySystem");
}

// ─── Assertion 5b: sleep_decay_system_behavior ───────────────────────────
// Type A — isolated decay system behavior across 100 ticks.
#[test]
fn harness_p5_gamma_sleep_decay_system_behavior_isolated() {
    let mut e = fresh_engine();
    // Register ONLY the decay systems; deliberately exclude decision + movement.
    e.register_system(Box::new(SleepDecaySystem::new()));

    let a = e.spawn_agent(0, 0);
    e.world.insert_one(a, Sleep::new(0.0, 1.0)).unwrap();
    let b = e.spawn_agent(1, 0);
    e.world.insert_one(b, Sleep::new(0.0, 0.5)).unwrap();
    let c = e.spawn_agent(2, 0);
    e.world.insert_one(c, Sleep::new(98.0, 5.0)).unwrap();

    // Record Agent B's value per tick to verify monotonic + bit-exact.
    let mut series: Vec<f64> = Vec::with_capacity(101);
    series.push(e.world.get::<&Sleep>(b).unwrap().fatigue);
    for k in 1..=100 {
        e.tick();
        let val = e.world.get::<&Sleep>(b).unwrap().fatigue;
        series.push(val);
        // Type: f64 (k * 0.5) — bit-exact since 0.5 is exactly representable
        assert_eq!(
            val,
            0.5 * (k as f64),
            "Agent B per-tick mismatch at tick {}: got {}",
            k,
            val
        );
    }

    // (a) Agent A — 1.0 × 100 = 100.0 (saturated, value matches either way)
    assert_eq!(e.world.get::<&Sleep>(a).unwrap().fatigue, 100.0);
    // (b) Agent B — 0.5 × 100 = 50.0
    assert_eq!(e.world.get::<&Sleep>(b).unwrap().fatigue, 50.0);
    // (c) Agent C — saturated at 100.0
    assert_eq!(e.world.get::<&Sleep>(c).unwrap().fatigue, 100.0);

    // monotonic non-decreasing for Agent B
    for w in series.windows(2) {
        assert!(w[0] <= w[1], "Agent B non-monotonic: {} -> {}", w[0], w[1]);
    }
}

// ─── Assertion 6: time_of_day_initial_state ──────────────────────────────
// Type A — initial state on fresh engine.
#[test]
fn harness_p5_gamma_time_of_day_starts_at_zero() {
    let e = fresh_engine();
    // Type: f64
    assert_eq!(e.resources.time_of_day, 0.0);
    // Type: u64
    assert_eq!(e.resources.ticks_per_day, 1440);
}

// ─── Assertion 7: time_of_day_advances_linearly_within_one_day ───────────
// Type A — pure modulo + division on u64 tick counter.
#[test]
fn harness_p5_gamma_time_of_day_advances_linearly() {
    let mut e = fresh_engine();
    let checkpoints = [1u64, 359, 360, 361, 720, 1080, 1437, 1438, 1439];
    let mut current = 0u64;
    for &n in &checkpoints {
        while current < n {
            e.tick();
            current += 1;
        }
        let expected = (n % 1440) as f64 * 24.0 / 1440.0;
        let observed = e.resources.time_of_day;
        // Type: f64 (within 1e-9)
        assert!(
            (observed - expected).abs() < 1e-9,
            "time_of_day mismatch at tick {}: expected {} got {}",
            n,
            expected,
            observed
        );
        if n == 1437 || n == 1438 || n == 1439 {
            assert!(observed < 24.0, "pre-wrap sample must be < 24.0");
        }
    }
}

// ─── Assertion 8: time_of_day_wraps_at_day_boundary ──────────────────────
// Type A — modulo wrap on u64 cannot drift.
#[test]
fn harness_p5_gamma_time_of_day_wraps_at_boundary() {
    let mut e = fresh_engine();
    // Run to tick 1440
    for _ in 0..1440 {
        e.tick();
    }
    assert!(
        (e.resources.time_of_day - 0.0).abs() < 1e-9,
        "time_of_day must wrap to 0.0 at tick 1440, got {}",
        e.resources.time_of_day
    );

    // Tick 1441
    e.tick();
    let expected = 24.0 / 1440.0;
    assert!(
        (e.resources.time_of_day - expected).abs() < 1e-9,
        "time_of_day at tick 1441 mismatch: expected {} got {}",
        expected,
        e.resources.time_of_day
    );

    // Run to tick 2879 (one tick before second-day wrap — multi-day off-by-one guard)
    for _ in 1441..2879 {
        e.tick();
    }
    // Pre-wrap value: (2879 % 1440) = 1439; 1439 * 24 / 1440 = 23.98333...
    let pre_wrap_expected = 1439.0 * 24.0 / 1440.0;
    assert!(
        (e.resources.time_of_day - pre_wrap_expected).abs() < 1e-9,
        "time_of_day at tick 2879 (pre-wrap) mismatch: expected {} got {}",
        pre_wrap_expected,
        e.resources.time_of_day
    );
    assert!(
        e.resources.time_of_day < 24.0,
        "pre-wrap sample at tick 2879 must be strictly < 24.0"
    );

    // Tick 2880 — second day wrap
    e.tick();
    assert!(
        (e.resources.time_of_day - 0.0).abs() < 1e-9,
        "time_of_day must wrap to 0.0 at tick 2880, got {}",
        e.resources.time_of_day
    );

    // Tick 2881
    e.tick();
    assert!(
        (e.resources.time_of_day - expected).abs() < 1e-9,
        "time_of_day at tick 2881 mismatch: expected {} got {}",
        expected,
        e.resources.time_of_day
    );
}

// ─── Assertion 8b: ticks_per_day_zero_guard ──────────────────────────────
// Type A — zero-guard prevents division panic.
#[test]
fn harness_p5_gamma_ticks_per_day_zero_guard() {
    let mut e = fresh_engine();
    e.resources.ticks_per_day = 0;
    // (a) No panic occurs across 5 ticks
    for _ in 0..5 {
        e.tick();
    }
    // (b) time_of_day stays 0.0
    assert_eq!(e.resources.time_of_day, 0.0);
}

// ─── Assertion 9: idle_to_seeking_sleep_on_fatigue_breach ────────────────
// Type A — single-need FSM rule.
#[test]
fn harness_p5_gamma_idle_to_seeking_sleep_on_fatigue_breach() {
    let mut e = fresh_engine();
    let entity = e.spawn_agent(3, 3);
    e.world
        .insert(
            entity,
            (
                AgentState::Idle,
                Hunger::new(10.0, 0.0),
                Thirst::new(10.0, 0.0),
                Sleep::new(60.0, 0.0),
            ),
        )
        .unwrap();

    let mut sys = AgentDecisionSystem::new();
    sys.tick(&mut e.world, &mut e.resources);

    // (a) state == Seeking { Sleep }
    let state = *e.world.get::<&AgentState>(entity).unwrap();
    assert_eq!(state, AgentState::Seeking { target: TargetKind::Sleep });

    // (b) exactly one AgentDecision event with FatigueThresholdBreach
    let width = e.resources.tile_grid.width;
    let tile_idx = (3 * width + 3) as u32;
    let log = e.resources.causal_log.get(tile_idx).expect("log present");
    let decisions: Vec<&CausalEvent> = log
        .as_slice()
        .iter()
        .filter(|ev| matches!(ev, CausalEvent::AgentDecision { .. }))
        .collect();
    assert_eq!(
        decisions.len(),
        1,
        "exactly one AgentDecision event must be recorded for a single fatigue breach"
    );
    match decisions[0] {
        CausalEvent::AgentDecision { reason, .. } => {
            assert_eq!(
                *reason,
                DecisionReason::FatigueThresholdBreach,
                "the recorded event must be FatigueThresholdBreach"
            );
        }
        _ => unreachable!("filter guarantees AgentDecision variant"),
    }
}

// ─── Assertion 9b: need_priority_order_locked ────────────────────────────
// Type A — strict priority chain Hunger > Thirst > Fatigue.
#[test]
fn harness_p5_gamma_need_priority_order_locked() {
    // Setup A: Hunger=60, Thirst=60, Fatigue=60 → Food / hunger
    {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(3, 3);
        e.world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(60.0, 0.0),
                    Thirst::new(60.0, 0.0),
                    Sleep::new(60.0, 0.0),
                ),
            )
            .unwrap();
        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        let state = *e.world.get::<&AgentState>(entity).unwrap();
        assert_eq!(state, AgentState::Seeking { target: TargetKind::Food });

        let width = e.resources.tile_grid.width;
        let tile_idx = (3 * width + 3) as u32;
        let log = e.resources.causal_log.get(tile_idx).expect("log");
        let decisions: Vec<&CausalEvent> = log
            .as_slice()
            .iter()
            .filter(|ev| matches!(ev, CausalEvent::AgentDecision { .. }))
            .collect();
        assert_eq!(decisions.len(), 1, "exactly one AgentDecision event per tick");
        match decisions[0] {
            CausalEvent::AgentDecision { reason, .. } => {
                assert_eq!(*reason, DecisionReason::HungerThresholdBreach);
            }
            _ => unreachable!(),
        }
    }

    // Setup B: Hunger=10, Thirst=60, Fatigue=60 → Water
    {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(3, 3);
        e.world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(10.0, 0.0),
                    Thirst::new(60.0, 0.0),
                    Sleep::new(60.0, 0.0),
                ),
            )
            .unwrap();
        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        let state = *e.world.get::<&AgentState>(entity).unwrap();
        assert_eq!(state, AgentState::Seeking { target: TargetKind::Water });

        let width = e.resources.tile_grid.width;
        let tile_idx = (3 * width + 3) as u32;
        let log = e.resources.causal_log.get(tile_idx).expect("log");
        let decisions: Vec<&CausalEvent> = log
            .as_slice()
            .iter()
            .filter(|ev| matches!(ev, CausalEvent::AgentDecision { .. }))
            .collect();
        assert_eq!(decisions.len(), 1);
        match decisions[0] {
            CausalEvent::AgentDecision { reason, .. } => {
                assert_eq!(*reason, DecisionReason::ThirstThresholdBreach);
            }
            _ => unreachable!(),
        }
    }

    // Setup C: Hunger=10, Thirst=10, Fatigue=60 → Sleep
    {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(3, 3);
        e.world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(10.0, 0.0),
                    Thirst::new(10.0, 0.0),
                    Sleep::new(60.0, 0.0),
                ),
            )
            .unwrap();
        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        let state = *e.world.get::<&AgentState>(entity).unwrap();
        assert_eq!(state, AgentState::Seeking { target: TargetKind::Sleep });

        let width = e.resources.tile_grid.width;
        let tile_idx = (3 * width + 3) as u32;
        let log = e.resources.causal_log.get(tile_idx).expect("log");
        let decisions: Vec<&CausalEvent> = log
            .as_slice()
            .iter()
            .filter(|ev| matches!(ev, CausalEvent::AgentDecision { .. }))
            .collect();
        assert_eq!(decisions.len(), 1);
        match decisions[0] {
            CausalEvent::AgentDecision { reason, .. } => {
                assert_eq!(*reason, DecisionReason::FatigueThresholdBreach);
            }
            _ => unreachable!(),
        }
    }
}

// ─── Assertion 9c: idle_stability_below_thresholds ───────────────────────
// Type A — sub-threshold stability across 20 ticks.
#[test]
fn harness_p5_gamma_idle_stability_below_thresholds() {
    let mut e = fresh_engine();
    let entity = e.spawn_agent(3, 3);
    e.world
        .insert(
            entity,
            (
                AgentState::Idle,
                Hunger::new(10.0, 0.0),
                Thirst::new(10.0, 0.0),
                Sleep::new(10.0, 0.0),
            ),
        )
        .unwrap();

    let mut sys = AgentDecisionSystem::new();
    for _ in 0..20 {
        sys.tick(&mut e.world, &mut e.resources);
        let state = *e.world.get::<&AgentState>(entity).unwrap();
        // Type: AgentState
        assert_eq!(state, AgentState::Idle);
    }

    let width = e.resources.tile_grid.width;
    let tile_idx = (3 * width + 3) as u32;
    let count = e
        .resources
        .causal_log
        .get(tile_idx)
        .map(|log| {
            log.as_slice()
                .iter()
                .filter(|ev| matches!(ev, CausalEvent::AgentDecision { .. }))
                .count()
        })
        .unwrap_or(0);
    assert_eq!(count, 0, "no AgentDecision events should fire below thresholds");
}

// ─── Assertion 10: consuming_sleep_decrements_tile_need_and_returns_to_idle
// Type A — conditional-mutation contract + underflow guard.
#[test]
fn harness_p5_gamma_consuming_sleep_decrements_then_idles() {
    // Setup 1: tile counter > 1
    {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(5, 5);
        e.world
            .insert(
                entity,
                (
                    AgentState::Consuming { target: TargetKind::Sleep },
                    Sleep::new(50.0, 0.0),
                ),
            )
            .unwrap();
        e.resources.set_sleep_tile(5, 5, 2);
        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        assert_eq!(e.resources.sleep_tiles.get(&(5, 5)), Some(&1));
        let s = *e.world.get::<&Sleep>(entity).unwrap();
        assert_eq!(s.fatigue, 50.0 - FATIGUE_CONSUME_AMOUNT);
        assert_eq!(s.fatigue, 20.0);
        assert_eq!(*e.world.get::<&AgentState>(entity).unwrap(), AgentState::Idle);
    }

    // Setup 2: tile counter == 1 → removed
    {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(5, 5);
        e.world
            .insert(
                entity,
                (
                    AgentState::Consuming { target: TargetKind::Sleep },
                    Sleep::new(50.0, 0.0),
                ),
            )
            .unwrap();
        e.resources.set_sleep_tile(5, 5, 1);
        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        assert!(!e.resources.sleep_tiles.contains_key(&(5, 5)));
        let s = *e.world.get::<&Sleep>(entity).unwrap();
        assert_eq!(s.fatigue, 20.0);
        assert_eq!(*e.world.get::<&AgentState>(entity).unwrap(), AgentState::Idle);
    }

    // Setup 3: no tile present (conditional-mutation contract)
    {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(5, 5);
        e.world
            .insert(
                entity,
                (
                    AgentState::Consuming { target: TargetKind::Sleep },
                    Sleep::new(50.0, 0.0),
                ),
            )
            .unwrap();
        // sleep_tiles is empty — no insertion should occur
        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        assert!(!e.resources.sleep_tiles.contains_key(&(5, 5)),
            "no phantom 0-counter entry should be inserted");
        let s = *e.world.get::<&Sleep>(entity).unwrap();
        assert_eq!(s.fatigue, 20.0, "need decrement is unconditional");
        assert_eq!(*e.world.get::<&AgentState>(entity).unwrap(), AgentState::Idle);
    }

    // Setup 4: underflow guard — Sleep clamped at 0
    {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(5, 5);
        e.world
            .insert(
                entity,
                (
                    AgentState::Consuming { target: TargetKind::Sleep },
                    Sleep::new(10.0, 0.0),
                ),
            )
            .unwrap();
        e.resources.set_sleep_tile(5, 5, 2);
        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);

        let s = *e.world.get::<&Sleep>(entity).unwrap();
        assert_eq!(s.fatigue, 0.0, "underflow must clamp to 0, not negative");
        assert_eq!(*e.world.get::<&AgentState>(entity).unwrap(), AgentState::Idle);
    }
}

// ─── Assertion 10b: seeking_to_consuming_transition_on_colocated_tile ────
// Type A — path symmetry across Food/Water/Sleep.
#[test]
fn harness_p5_gamma_seeking_to_consuming_transition() {
    // Setup A: Food path
    {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(5, 5);
        e.world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(60.0, 0.0),
                    Thirst::new(10.0, 0.0),
                    Sleep::new(10.0, 0.0),
                ),
            )
            .unwrap();
        e.resources.set_food_tile(5, 5, 5);
        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            *e.world.get::<&AgentState>(entity).unwrap(),
            AgentState::Seeking { target: TargetKind::Food }
        );
        sys.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            *e.world.get::<&AgentState>(entity).unwrap(),
            AgentState::Consuming { target: TargetKind::Food }
        );
    }

    // Setup B: Water path
    {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(5, 5);
        e.world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(10.0, 0.0),
                    Thirst::new(60.0, 0.0),
                    Sleep::new(10.0, 0.0),
                ),
            )
            .unwrap();
        e.resources.set_water_tile(5, 5, 5);
        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        sys.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            *e.world.get::<&AgentState>(entity).unwrap(),
            AgentState::Consuming { target: TargetKind::Water }
        );
    }

    // Setup C: Sleep path
    {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(5, 5);
        e.world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(10.0, 0.0),
                    Thirst::new(10.0, 0.0),
                    Sleep::new(60.0, 0.0),
                ),
            )
            .unwrap();
        e.resources.set_sleep_tile(5, 5, 5);
        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        sys.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            *e.world.get::<&AgentState>(entity).unwrap(),
            AgentState::Consuming { target: TargetKind::Sleep }
        );
    }
}

// ─── Assertion 10c: seeking_with_no_tile_remains_seeking ─────────────────
// Type A — Seeking with no matching tile must stall.
#[test]
fn harness_p5_gamma_seeking_with_no_tile_remains_seeking() {
    let mut e = fresh_engine();
    let entity = e.spawn_agent(5, 5);
    e.world
        .insert(
            entity,
            (
                AgentState::Seeking { target: TargetKind::Sleep },
                Sleep::new(60.0, 0.0),
            ),
        )
        .unwrap();
    // sleep_tiles is empty
    let mut sys = AgentDecisionSystem::new();
    for _ in 0..5 {
        sys.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            *e.world.get::<&AgentState>(entity).unwrap(),
            AgentState::Seeking { target: TargetKind::Sleep }
        );
        let s = *e.world.get::<&Sleep>(entity).unwrap();
        assert_eq!(s.fatigue, 60.0, "no consume should have happened");
    }

    let width = e.resources.tile_grid.width;
    let tile_idx = (5 * width + 5) as u32;
    let count = e
        .resources
        .causal_log
        .get(tile_idx)
        .map(|log| {
            log.as_slice()
                .iter()
                .filter(|ev| matches!(ev, CausalEvent::AgentDecision { .. }))
                .count()
        })
        .unwrap_or(0);
    assert_eq!(count, 0, "no AgentDecision events emitted in stalled Seeking");
}

// ─── Assertion 11: full_day_chronicle_visits_all_three_needs ─────────────
// Type A — full-day chronicle, derived invariant from design intent.
// Also Assertion 12 lives here since it shares the same run.
#[test]
fn harness_p5_gamma_full_day_chronicle_visits_all_three_needs() {
    let mut e = fresh_engine_sized(16, 16);
    e.register_system(Box::new(AgentDecisionSystem::new()));
    e.register_system(Box::new(HungerDecaySystem::new()));
    e.register_system(Box::new(ThirstDecaySystem::new()));
    e.register_system(Box::new(SleepDecaySystem::new()));
    // No movement system needed since tiles are co-located.

    let entity = e.spawn_agent(8, 8);
    e.world
        .insert(
            entity,
            (
                AgentState::Idle,
                Hunger::new(0.0, 0.1),
                Thirst::new(0.0, 0.08),
                Sleep::new(0.0, 0.06),
            ),
        )
        .unwrap();
    e.resources.set_food_tile(8, 8, u8::MAX);
    e.resources.set_water_tile(8, 8, u8::MAX);
    e.resources.set_sleep_tile(8, 8, u8::MAX);

    // Trajectory recording: per-tick (prev_state, curr_state)
    let mut state_trajectory: Vec<AgentState> = Vec::with_capacity(1441);
    state_trajectory.push(*e.world.get::<&AgentState>(entity).unwrap());

    for _ in 0..1440 {
        e.tick();
        state_trajectory.push(*e.world.get::<&AgentState>(entity).unwrap());
    }

    let width = e.resources.tile_grid.width;
    let tile_idx = (8 * width + 8) as u32;
    let log = e
        .resources
        .causal_log
        .get(tile_idx)
        .expect("agent-tile causal log present");

    // Collect (id, reason) pairs from AgentDecision events in tile-log order.
    // NOTE: TILE_CAUSAL_RING_SIZE = 8, and over 1440 ticks the chronicle
    // emits ~9 AgentDecision events, so the very first Hunger event is
    // evicted by ring overflow. Set-membership is verified from the log;
    // first-appearance ordering is verified from the AgentState
    // trajectory (every transition recorded, never evicted), which is
    // a strictly stronger guarantee than the log-only ring-clipped view.
    let mut decisions: Vec<(u64, DecisionReason)> = Vec::new();
    for ev in log.as_slice() {
        if let CausalEvent::AgentDecision { id, reason, .. } = ev {
            decisions.push((*id, *reason));
        }
    }

    // Set membership from log.
    let mut seen_hunger_log = false;
    let mut seen_thirst_log = false;
    let mut seen_fatigue_log = false;
    for (_id, r) in &decisions {
        match r {
            DecisionReason::HungerThresholdBreach => seen_hunger_log = true,
            DecisionReason::ThirstThresholdBreach => seen_thirst_log = true,
            DecisionReason::FatigueThresholdBreach => seen_fatigue_log = true,
            // V7 Phase 6-β / 7-β: ConstructionReason (4th) and SocialReason
            // (5th) cascade steps. This chronicle scenario seeds no
            // construction sites and no co-located social partners, so
            // neither variant is expected to appear — but the exhaustive
            // match must still cover both.
            DecisionReason::ConstructionReason
            | DecisionReason::SocialReason
            | DecisionReason::MemoryReason
            | DecisionReason::CombatReason
            | DecisionReason::SettlementReason => {}
        }
    }
    assert!(
        seen_hunger_log,
        "Hunger discriminator must appear in log over full day"
    );
    assert!(
        seen_thirst_log,
        "Thirst discriminator must appear in log over full day"
    );
    assert!(
        seen_fatigue_log,
        "Fatigue discriminator must appear in log over full day"
    );

    // First-appearance ordering from FSM trajectory (Idle→Seeking{T} edge).
    // The trajectory is exhaustive (1441 states; never evicted), so it
    // captures first-appearance of each reason with full fidelity.
    let mut first_hunger_tick: Option<usize> = None;
    let mut first_thirst_tick: Option<usize> = None;
    let mut first_fatigue_tick: Option<usize> = None;
    for (i, w) in state_trajectory.windows(2).enumerate() {
        let prev = w[0];
        let curr = w[1];
        if prev == AgentState::Idle {
            if let AgentState::Seeking { target } = curr {
                match target {
                    TargetKind::Food => {
                        if first_hunger_tick.is_none() {
                            first_hunger_tick = Some(i);
                        }
                    }
                    TargetKind::Water => {
                        if first_thirst_tick.is_none() {
                            first_thirst_tick = Some(i);
                        }
                    }
                    TargetKind::Sleep => {
                        if first_fatigue_tick.is_none() {
                            first_fatigue_tick = Some(i);
                        }
                    }
                    // V7 Phase 6-α / 7-α: ConstructionSite + Agent are
                    // unreachable in the γ chronicle (no decision logic
                    // routes to either).
                    TargetKind::ConstructionSite | TargetKind::Agent(_) => {}
                }
            }
        }
    }
    let fh = first_hunger_tick.expect("trajectory must contain first Hunger breach");
    let ft = first_thirst_tick.expect("trajectory must contain first Thirst breach");
    let ff = first_fatigue_tick.expect("trajectory must contain first Fatigue breach");
    assert!(
        fh < ft,
        "Hunger first-appearance must precede Thirst (fh={}, ft={})",
        fh,
        ft
    );
    assert!(
        ft < ff,
        "Thirst first-appearance must precede Fatigue (ft={}, ff={})",
        ft,
        ff
    );

    // ─── P5-γ DIAGNOSTIC COUNTERS — chronicle execution evidence ─────────
    // Counts each reason across the trajectory (every transition, never
    // evicted) and prints first-occurrence ticks. This proves that all
    // three needs actually executed (counts > 0), not just that the log
    // ring captured a subset.
    let mut count_hunger = 0u32;
    let mut count_thirst = 0u32;
    let mut count_fatigue = 0u32;
    for w in state_trajectory.windows(2) {
        let prev = w[0];
        let curr = w[1];
        if prev == AgentState::Idle {
            if let AgentState::Seeking { target } = curr {
                match target {
                    TargetKind::Food => count_hunger += 1,
                    TargetKind::Water => count_thirst += 1,
                    TargetKind::Sleep => count_fatigue += 1,
                    // V7 Phase 6-α / 7-α: ConstructionSite + Agent unreachable in γ chronicle.
                    TargetKind::ConstructionSite | TargetKind::Agent(_) => {}
                }
            }
        }
    }
    println!(
        "[P5-γ chronicle] per-reason transition counts (Idle→Seeking{{T}}): hunger={}, thirst={}, fatigue={}",
        count_hunger, count_thirst, count_fatigue
    );
    println!(
        "[P5-γ chronicle] first-occurrence ticks: hunger@{}, thirst@{}, fatigue@{}",
        fh, ft, ff
    );
    println!(
        "[P5-γ chronicle] log-ring AgentDecision events captured: {} (TILE_CAUSAL_RING_SIZE clipping expected)",
        decisions.len()
    );
    assert!(count_hunger > 0, "hunger executed at least once (count > 0)");
    assert!(count_thirst > 0, "thirst executed at least once (count > 0)");
    assert!(count_fatigue > 0, "fatigue executed at least once (count > 0)");

    // ─── Assertion 12 part — distinct event ids + AgentState transition trace
    // (a)(b) already implied by seen_* + decisions.len() ≥ 3
    assert!(decisions.len() >= 3, "at least 3 distinct AgentDecision events");

    // (c) all event ids unique
    let mut ids: Vec<u64> = decisions.iter().map(|(id, _)| *id).collect();
    ids.sort_unstable();
    let dedup_len = {
        let mut d = ids.clone();
        d.dedup();
        d.len()
    };
    assert_eq!(ids.len(), dedup_len, "all event ids must be unique");

    // (d) trajectory contains at least one transition into Consuming{Food/Water/Sleep}
    let mut consumed_food = false;
    let mut consumed_water = false;
    let mut consumed_sleep = false;
    for w in state_trajectory.windows(2) {
        let prev = w[0];
        let curr = w[1];
        if let AgentState::Seeking { target: prev_target } = prev {
            if let AgentState::Consuming { target: curr_target } = curr {
                if prev_target == curr_target {
                    match curr_target {
                        TargetKind::Food => consumed_food = true,
                        TargetKind::Water => consumed_water = true,
                        TargetKind::Sleep => consumed_sleep = true,
                        // V7 Phase 6-α / 7-α: ConstructionSite + Agent unreachable in γ chronicle.
                        TargetKind::ConstructionSite | TargetKind::Agent(_) => {}
                    }
                }
            }
        }
    }
    assert!(consumed_food, "trajectory must include Seeking→Consuming{{Food}}");
    assert!(consumed_water, "trajectory must include Seeking→Consuming{{Water}}");
    assert!(consumed_sleep, "trajectory must include Seeking→Consuming{{Sleep}}");
}

// ─── Assertion 13: phase4_phase5alpha_phase5beta_regression_clean ────────
// Type D — regression guards.
#[test]
fn harness_p5_gamma_phase4_phase5alpha_phase5beta_regression_clean() {
    // (a) Hunger 60 (no Thirst/Sleep) → Seeking{Food} + exactly one HungerThresholdBreach
    {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(4, 4);
        e.world
            .insert(entity, (AgentState::Idle, Hunger::new(60.0, 0.0)))
            .unwrap();
        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            *e.world.get::<&AgentState>(entity).unwrap(),
            AgentState::Seeking { target: TargetKind::Food }
        );
        let width = e.resources.tile_grid.width;
        let tile_idx = (4 * width + 4) as u32;
        let log = e.resources.causal_log.get(tile_idx).expect("log");
        let count = log
            .as_slice()
            .iter()
            .filter(|ev| matches!(
                ev,
                CausalEvent::AgentDecision { reason: DecisionReason::HungerThresholdBreach, .. }
            ))
            .count();
        assert_eq!(count, 1, "exactly one HungerThresholdBreach for (a)");
    }

    // (b) Hunger 10, Thirst 60 (no Sleep) → Seeking{Water} + exactly one ThirstThresholdBreach
    {
        let mut e = fresh_engine();
        let entity = e.spawn_agent(4, 4);
        e.world
            .insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(10.0, 0.0),
                    Thirst::new(60.0, 0.0),
                ),
            )
            .unwrap();
        let mut sys = AgentDecisionSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        assert_eq!(
            *e.world.get::<&AgentState>(entity).unwrap(),
            AgentState::Seeking { target: TargetKind::Water }
        );
        let width = e.resources.tile_grid.width;
        let tile_idx = (4 * width + 4) as u32;
        let log = e.resources.causal_log.get(tile_idx).expect("log");
        let count = log
            .as_slice()
            .iter()
            .filter(|ev| matches!(
                ev,
                CausalEvent::AgentDecision { reason: DecisionReason::ThirstThresholdBreach, .. }
            ))
            .count();
        assert_eq!(count, 1, "exactly one ThirstThresholdBreach for (b)");
    }

    // (c) Brownian-only agent, 64 ticks — bit-exact trajectory.
    // Since no pre-γ snapshot is available, we run the same setup twice
    // (with identical seed and starting position) and assert determinism.
    // Two engines with the same setup must produce bit-identical sequences.
    {
        let mut engine_a = fresh_engine_sized(16, 16);
        engine_a.register_system(Box::new(AgentMovementSystem::new()));
        let id_a = engine_a.world.spawn((
            Position::new(5, 5),
            MovementRng::new(42),
        ));

        let mut engine_b = fresh_engine_sized(16, 16);
        engine_b.register_system(Box::new(AgentMovementSystem::new()));
        let id_b = engine_b.world.spawn((
            Position::new(5, 5),
            MovementRng::new(42),
        ));

        let mut trajectory_a: Vec<(u32, u32)> = Vec::with_capacity(65);
        let mut trajectory_b: Vec<(u32, u32)> = Vec::with_capacity(65);
        trajectory_a.push({
            let p = *engine_a.world.get::<&Position>(id_a).unwrap();
            (p.x, p.y)
        });
        trajectory_b.push({
            let p = *engine_b.world.get::<&Position>(id_b).unwrap();
            (p.x, p.y)
        });

        for _ in 0..64 {
            engine_a.tick();
            engine_b.tick();
            let pa = *engine_a.world.get::<&Position>(id_a).unwrap();
            let pb = *engine_b.world.get::<&Position>(id_b).unwrap();
            trajectory_a.push((pa.x, pa.y));
            trajectory_b.push((pb.x, pb.y));
        }
        assert_eq!(
            trajectory_a, trajectory_b,
            "Brownian trajectory must be bit-identical across two engines with same seed"
        );

        // Behavioral guard: at least one position change must have occurred
        // over 64 ticks. Same-seed equality alone is circular — a stationary
        // movement system would pass it. Asserting movement closes that hole.
        let start = trajectory_a[0];
        let any_change = trajectory_a.iter().any(|&p| p != start);
        assert!(
            any_change,
            "Brownian movement must produce at least one position change over 64 ticks (start={:?}, trajectory={:?})",
            start, trajectory_a
        );

        // Stronger guard: count distinct positions visited (> 1 implies movement).
        let mut distinct: Vec<(u32, u32)> = trajectory_a.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert!(
            distinct.len() > 1,
            "Brownian motion must visit > 1 distinct position over 64 ticks (got {} distinct)",
            distinct.len()
        );
    }
}

// ─── Helper: silence unused-import warning for Agent if needed ───────────
#[allow(dead_code)]
fn _agent_use(_: &Agent) {}
