//! V7 Section 16-β — directed movement (Seeking → goal) harness.
//!
//! β closes the gathering loop laid by α0 (`53075aff`, resource substrate)
//! and α (`335ca145`, `SeekTarget` nearest-tile targeting): a
//! `Seeking{Food/Water/Sleep}` agent that carries a `SeekTarget` now takes a
//! one-tile **directed signum step** toward `SeekTarget.tile` in
//! `AgentMovementSystem`, instead of being frozen. The full path is:
//!
//!   breach → Seeking + SeekTarget (α) → directed walk (β) →
//!   Seeking→Consuming (decision, on arrival) → satisfied → Idle.
//!
//! Non-circular rule: every trajectory assertion compares against a
//! HAND-COMPUTED signum result derived from the start/target coordinates,
//! never a value re-read from the production step. Directed math is
//! `dx = signum(target.x - pos.x)`, `dy = signum(target.y - pos.y)` — pure
//! coordinate arithmetic, RNG-free, so it is inherently deterministic.
//!
//! Run:
//!   cargo test -p sim-test --test harness_s16_beta_movement -- --nocapture

use sim_core::components::{AgentState, Hunger, Position, SeekTarget, TargetKind};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};

// ── helpers ─────────────────────────────────────────────────────────────────

/// Fresh 64×64 engine — large enough for every hand-placed tile below
/// (clamp bounds are `max_x = max_y = 63`).
fn engine() -> SimEngine {
    SimEngine::new(64, 64, MaterialRegistry::new())
}

/// Current `(x, y)` of `ent`.
fn pos(e: &SimEngine, ent: hecs::Entity) -> (u32, u32) {
    let p = e.world.get::<&Position>(ent).expect("Position present");
    (p.x, p.y)
}

/// State of `ent`.
fn state(e: &SimEngine, ent: hecs::Entity) -> AgentState {
    *e.world.get::<&AgentState>(ent).expect("AgentState present")
}

/// Spawn a movement-eligible probe: Position + MovementRng + AgentState
/// (+ optional SeekTarget). `MovementRng` is REQUIRED — the movement query
/// is `(&mut Position, &mut MovementRng, Option<&AgentState>, Option<&SeekTarget>)`,
/// so an agent without it is never visited by the system.
fn spawn_probe(
    e: &mut SimEngine,
    at: (u32, u32),
    seed: u64,
    st: AgentState,
    seek: Option<SeekTarget>,
) -> hecs::Entity {
    let ent = e.spawn_agent(at.0, at.1);
    e.world
        .insert(ent, (MovementRng::new(seed), st))
        .expect("insert rng + state");
    if let Some(s) = seek {
        e.world.insert_one(ent, s).expect("insert seek target");
    }
    ent
}

/// Run `n` movement-system ticks on `e`.
fn run_movement(e: &mut SimEngine, n: usize) {
    let mut sys = AgentMovementSystem::new();
    for _ in 0..n {
        sys.tick(&mut e.world, &mut e.resources);
    }
}

// ── shared end-to-end gathering-loop run (Assertions 4, 9, 16) ──────────────

/// Result of one full-engine gathering-loop run.
struct LoopResult {
    final_hunger: f32,
    final_state: AgentState,
    reached_source: bool,
    source_value: Option<u8>,
}

/// Full `engine.tick()` run with all default runtime systems registered.
/// Seeds ONE infinite food source at `(20,20)`, spawns ONE agent at
/// `(16,16)` (DIAGONAL Chebyshev-4 path, both axes non-zero each step),
/// `Idle` + `Hunger{value=51.0 (>THRESHOLD), growth=0}` + a `MovementRng`,
/// then runs 30 ticks sampling the agent position each tick.
fn run_gathering_loop() -> LoopResult {
    let mut e = engine();
    register_default_runtime_systems(&mut e);
    e.resources.set_food_tile(20, 20, RESOURCE_SOURCE_INFINITE);

    let agent = e.spawn_agent(16, 16);
    e.world
        .insert(
            agent,
            (
                AgentState::Idle,
                Hunger::new(51.0, 0.0),
                MovementRng::new(42),
            ),
        )
        .expect("seed end-to-end agent");

    let mut reached = false;
    for _ in 0..30 {
        e.tick();
        if pos(&e, agent) == (20, 20) {
            reached = true;
        }
    }

    let final_hunger = e.world.get::<&Hunger>(agent).expect("hunger").value;
    let final_state = state(&e, agent);
    let source_value = e.resources.food_tiles.get(&(20, 20)).copied();
    LoopResult {
        final_hunger,
        final_state,
        reached_source: reached,
        source_value,
    }
}

// ─── Assertion 1: one directed signum step toward food target ──────────────
#[test]
fn harness_movement_directed_step_toward_food_target_one_tick() {
    // Type: A — hand-computed signum: dx=signum(15-10)=+1, dy=signum(10-10)=0
    // ⇒ exactly (11,10) after one movement tick. RNG-free.
    let mut e = engine();
    let a = spawn_probe(
        &mut e,
        (10, 10),
        7,
        AgentState::Seeking { target: TargetKind::Food },
        Some(SeekTarget::new((15, 10))),
    );
    run_movement(&mut e, 1);
    assert_eq!(pos(&e, a), (11, 10), "A1: one signum step → (11,10)");
    println!("[S16-β A1] (10,10)+target(15,10) → (11,10) after 1 tick ✓");
}

// ─── Assertion 2: directed path reaches then holds the target ──────────────
#[test]
fn harness_movement_directed_path_reaches_then_holds_target() {
    // Type: A — Chebyshev 5; one step/tick closes it in 5 ticks, then dx=0
    // holds for the remaining 3 ticks. x never overshoots past 15.
    let mut e = engine();
    let a = spawn_probe(
        &mut e,
        (10, 10),
        7,
        AgentState::Seeking { target: TargetKind::Food },
        Some(SeekTarget::new((15, 10))),
    );
    let mut sys = AgentMovementSystem::new();
    let mut prev_x = 10u32;
    for tick in 1..=8 {
        sys.tick(&mut e.world, &mut e.resources);
        let (x, y) = pos(&e, a);
        assert_eq!(y, 10, "A2: y must never drift (tick {tick})");
        assert!(x >= prev_x, "A2: x must be monotonically non-decreasing (tick {tick})");
        assert!(x <= 15, "A2: x must never overshoot past 15 (tick {tick}, x={x})");
        prev_x = x;
    }
    assert_eq!(pos(&e, a), (15, 10), "A2: final position must be the target (15,10)");
    println!("[S16-β A2] (10,10)→(15,10) in 5 steps, holds through tick 8 ✓");
}

// ─── Assertion 3: directed movement deterministic AND non-frozen ───────────
#[test]
fn harness_movement_directed_movement_deterministic_and_nonfrozen() {
    // Type: A — two independent engines, identical setup, 6 ticks each:
    // (a) cross-engine equal final position, (b) BOTH actually displaced
    // toward goal (pos.x > 10) — a no-op β would freeze at x=10 and fail (b).
    fn build_and_run() -> (u32, u32) {
        let mut e = engine();
        let a = spawn_probe(
            &mut e,
            (10, 10),
            42,
            AgentState::Seeking { target: TargetKind::Food },
            Some(SeekTarget::new((15, 10))),
        );
        run_movement(&mut e, 6);
        pos(&e, a)
    }
    let pa = build_and_run();
    let pb = build_and_run();
    assert_eq!(pa, pb, "A3a: identical setup must yield identical final position");
    assert!(pa.0 > 10, "A3b: engine_a agent must have moved toward goal (x>10)");
    assert!(pb.0 > 10, "A3b: engine_b agent must have moved toward goal (x>10)");
    println!("[S16-β A3] deterministic ({pa:?}=={pb:?}) AND non-frozen (x>10) ✓");
}

// ─── Assertion 4: end-to-end gathering loop satisfies hunger ───────────────
#[test]
fn harness_movement_end_to_end_gathering_loop_satisfies_hunger() {
    // Type: C — breach→Seeking(α)→directed-diagonal-walk(β)→arrive→Consuming
    // →satisfied→Idle fires end-to-end. Computed final Hunger 51-30=21.0 < 50.
    let r = run_gathering_loop();
    assert!(
        r.final_hunger < 50.0,
        "A4: final Hunger must drop below HUNGER_THRESHOLD(50.0); got {}",
        r.final_hunger
    );
    assert_eq!(r.final_state, AgentState::Idle, "A4: agent must return to Idle");
    println!(
        "[S16-β A4] end-to-end: Hunger {} < 50.0, state Idle ✓",
        r.final_hunger
    );
}

// ─── Assertion 5: Seeking without a target freezes ─────────────────────────
#[test]
fn harness_movement_seeking_without_target_freezes() {
    // Type: A — Seeking with NO SeekTarget + empty food_tiles must freeze
    // (no Brownian, no directed step). Position drift = no-target guard bug.
    let mut e = engine();
    let a = spawn_probe(
        &mut e,
        (10, 10),
        42,
        AgentState::Seeking { target: TargetKind::Food },
        None,
    );
    run_movement(&mut e, 32);
    assert_eq!(pos(&e, a), (10, 10), "A5: no-target Seeking agent must stay frozen");
    println!("[S16-β A5] Seeking + no SeekTarget → frozen at (10,10) ✓");
}

// ─── Assertion 6: Consuming state freezes ──────────────────────────────────
#[test]
fn harness_movement_consuming_state_freezes() {
    // Type: D — regression guard. Consuming must freeze so the 2-tick consume
    // commit reads the correct tile. Seed 1 produces a non-zero step when not
    // suppressed, so a regressed freeze would surface observably.
    let mut e = engine();
    let a = spawn_probe(
        &mut e,
        (10, 10),
        1,
        AgentState::Consuming { target: TargetKind::Food },
        None,
    );
    run_movement(&mut e, 32);
    assert_eq!(pos(&e, a), (10, 10), "A6: Consuming agent must stay frozen");
    // Sanity: seed 1 does produce movement when not suppressed.
    let mut probe = MovementRng::new(1);
    let dx = probe.next_step();
    let dy = probe.next_step();
    assert!(dx != 0 || dy != 0, "A6: seed 1 must yield a non-zero step (else vacuous)");
    println!("[S16-β A6] Consuming{{Food}} → frozen at (10,10) ✓");
}

// ─── Assertion 7: Idle Brownian preserved, bounded ─────────────────────────
#[test]
fn harness_movement_idle_brownian_preserved_bounded() {
    // Type: A — Idle agents must still Brownian-walk: (a) net displacement
    // non-zero over 32 ticks (HARD assert — never vacuously passes), and
    // (b) every per-axis step ≤ 1. Seed 42 verified to move under current
    // code (matches the movement.rs `idle_state_still_moves` seed).
    let mut e = engine();
    let a = spawn_probe(&mut e, (16, 16), 42, AgentState::Idle, None);
    let mut sys = AgentMovementSystem::new();
    let mut prev = (16u32, 16u32);
    let mut moved = false;
    for tick in 1..=32 {
        sys.tick(&mut e.world, &mut e.resources);
        let (x, y) = pos(&e, a);
        assert!(x.abs_diff(prev.0) <= 1, "A7: |Δx| ≤ 1 per tick (tick {tick})");
        assert!(y.abs_diff(prev.1) <= 1, "A7: |Δy| ≤ 1 per tick (tick {tick})");
        if (x, y) != (16, 16) {
            moved = true;
        }
        prev = (x, y);
    }
    assert!(moved, "A7: Idle agent must execute Brownian motion (net displacement ≠ 0)");
    println!("[S16-β A7] Idle Brownian preserved, |Δ|≤1/axis, moved=true ✓");
}

// ─── Assertion 8: Water + Sleep directed steps (kind-agnostic) ─────────────
#[test]
fn harness_movement_water_and_sleep_directed_steps() {
    // Type: A — directed branch keys on Seeking{..} regardless of TargetKind.
    // (a) Water target (10,15): dy=+1 → (10,11). (b) Sleep target (5,10):
    // dx=-1 → (9,10).
    let mut e = engine();
    let water = spawn_probe(
        &mut e,
        (10, 10),
        7,
        AgentState::Seeking { target: TargetKind::Water },
        Some(SeekTarget::new((10, 15))),
    );
    let sleep = spawn_probe(
        &mut e,
        (10, 10),
        7,
        AgentState::Seeking { target: TargetKind::Sleep },
        Some(SeekTarget::new((5, 10))),
    );
    run_movement(&mut e, 1);
    assert_eq!(pos(&e, water), (10, 11), "A8a: Water seek → (10,11)");
    assert_eq!(pos(&e, sleep), (9, 10), "A8b: Sleep seek → (9,10)");
    println!("[S16-β A8] Water→(10,11), Sleep→(9,10) — kind-agnostic ✓");
}

// ─── Assertion 9: infinite source not depleted by the loop ─────────────────
#[test]
fn harness_movement_infinite_source_not_depleted_by_loop() {
    // Type: D — validity-conjoined with A4 (Hunger dropped) + A16 (reached).
    // The u8::MAX source must remain u8::MAX after a real co-located consume.
    let r = run_gathering_loop();
    // Validity precondition: only meaningful if the agent actually consumed.
    assert!(r.final_hunger < 50.0, "A9 precondition (A4): Hunger must have dropped");
    assert!(r.reached_source, "A9 precondition (A16): agent must have reached (20,20)");
    assert_eq!(
        r.source_value,
        Some(RESOURCE_SOURCE_INFINITE),
        "A9: source tile (20,20) must remain u8::MAX after a genuine consume"
    );
    println!("[S16-β A9] source (20,20) still u8::MAX after consumed-from loop ✓");
}

// ─── Assertion 10: co-located target → signum-0 → no move ──────────────────
#[test]
fn harness_movement_colocated_target_signum_zero_no_move() {
    // Type: A — agent already ON its SeekTarget: dx=signum(0)=0, dy=signum(0)=0
    // ⇒ zero displacement.
    let mut e = engine();
    let a = spawn_probe(
        &mut e,
        (12, 12),
        7,
        AgentState::Seeking { target: TargetKind::Food },
        Some(SeekTarget::new((12, 12))),
    );
    run_movement(&mut e, 1);
    assert_eq!(pos(&e, a), (12, 12), "A10: signum-0 on both axes → no move");
    println!("[S16-β A10] on-target agent holds station at (12,12) ✓");
}

// ─── Assertion 11: suppresses_movement() truth table unchanged ─────────────
#[test]
fn harness_movement_suppresses_movement_truth_table_unchanged() {
    // Type: D — cross-phase regression guard on the LOCKED predicate. β must
    // NOT alter it; 5 external harnesses depend on it.
    assert!(
        AgentState::Seeking { target: TargetKind::Food }.suppresses_movement(),
        "A11: Seeking{{Food}}.suppresses_movement() must remain true"
    );
    assert!(
        !AgentState::Idle.suppresses_movement(),
        "A11: Idle.suppresses_movement() must remain false"
    );
    println!("[S16-β A11] truth table intact: Seeking→true, Idle→false ✓");
}

// ─── Assertion 12: directed step does not consume RNG ──────────────────────
#[test]
fn harness_movement_directed_step_does_not_consume_rng() {
    // Type: A — Agent A runs 5 directed ticks (carrying SeekTarget) then goes
    // Idle; Agent B goes Idle immediately. Same seed S. If the directed step
    // consumed no RNG, A's stream is un-advanced, so A's first Idle Brownian
    // Δ must equal B's. Compare STEP VECTORS (cancels A's directed offset).
    const S: u64 = 12345;
    let mut e = engine();
    let a = spawn_probe(
        &mut e,
        (10, 10),
        S,
        AgentState::Seeking { target: TargetKind::Food },
        Some(SeekTarget::new((15, 10))),
    );
    let b = spawn_probe(&mut e, (10, 10), S, AgentState::Idle, None);

    // Preferred direct check: after 5 directed ticks A's RNG state must be
    // byte-identical to a fresh seed-S RNG (MovementRng derives PartialEq).
    run_movement(&mut e, 5);
    {
        let rng_a = *e.world.get::<&MovementRng>(a).expect("A rng");
        assert_eq!(
            rng_a,
            MovementRng::new(S),
            "A12: directed steps must NOT advance the RNG (state must equal fresh seed S)"
        );
    }

    // Observable proxy: transition A → Idle, take one Brownian step each, and
    // compare per-axis displacement vectors.
    e.world
        .insert_one(a, AgentState::Idle)
        .expect("A → Idle");
    let a_before = pos(&e, a);
    let b_before = pos(&e, b);
    run_movement(&mut e, 1);
    let a_after = pos(&e, a);
    let b_after = pos(&e, b);
    let da = (a_after.0 as i64 - a_before.0 as i64, a_after.1 as i64 - a_before.1 as i64);
    let db = (b_after.0 as i64 - b_before.0 as i64, b_after.1 as i64 - b_before.1 as i64);
    assert_eq!(da, db, "A12: A and B first-Idle Brownian Δ must match (RNG un-consumed)");
    println!("[S16-β A12] directed step consumes no RNG (Δ {da:?}=={db:?}) ✓");
}

// ─── Assertion 13: simultaneous positive two-axis step in one tick ─────────
#[test]
fn harness_movement_diagonal_two_axis_step_one_tick() {
    // Type: A — dx=signum(15-10)=+1, dy=signum(15-10)=+1 ⇒ (11,11). Catches
    // the update-x-then-early-return-before-y bug class (would yield (11,10)).
    let mut e = engine();
    let a = spawn_probe(
        &mut e,
        (10, 10),
        7,
        AgentState::Seeking { target: TargetKind::Food },
        Some(SeekTarget::new((15, 15))),
    );
    run_movement(&mut e, 1);
    assert_eq!(pos(&e, a), (11, 11), "A13: both axes step +1 → (11,11)");
    println!("[S16-β A13] diagonal +x/+y in one tick → (11,11) ✓");
}

// ─── Assertion 14: simultaneous negative two-axis step in one tick ─────────
#[test]
fn harness_movement_diagonal_negative_two_axis_step_one_tick() {
    // Type: A — dx=signum(5-10)=-1, dy=signum(5-10)=-1 ⇒ (9,9). Covers the
    // combined-negative direction entirely absent from prior plans.
    let mut e = engine();
    let a = spawn_probe(
        &mut e,
        (10, 10),
        7,
        AgentState::Seeking { target: TargetKind::Food },
        Some(SeekTarget::new((5, 5))),
    );
    run_movement(&mut e, 1);
    assert_eq!(pos(&e, a), (9, 9), "A14: both axes step -1 → (9,9)");
    println!("[S16-β A14] diagonal -x/-y in one tick → (9,9) ✓");
}

// ─── Assertion 15: staggered-axis arrival (one axis holds, other closes) ───
#[test]
fn harness_movement_diagonal_staggered_axis_arrival() {
    // Type: A — target (15,12): y closes 10→12 in 2 ticks then signum-0 holds;
    // x closes 10→15 in 5. A stop-when-EITHER-axis-arrives bug freezes x at 12.
    let mut e = engine();
    let a = spawn_probe(
        &mut e,
        (10, 10),
        7,
        AgentState::Seeking { target: TargetKind::Food },
        Some(SeekTarget::new((15, 12))),
    );
    let mut sys = AgentMovementSystem::new();
    let mut prev_x = 10u32;
    for tick in 1..=8 {
        sys.tick(&mut e.world, &mut e.resources);
        let (x, y) = pos(&e, a);
        if tick >= 2 {
            assert_eq!(y, 12, "A15: y must reach 12 by tick 2 and hold (tick {tick})");
        }
        assert!(x >= prev_x, "A15: x must be non-decreasing (tick {tick})");
        assert!(x <= 15, "A15: x must never overshoot 15 (tick {tick}, x={x})");
        prev_x = x;
    }
    assert_eq!(pos(&e, a), (15, 12), "A15: final position must be (15,12)");
    println!("[S16-β A15] staggered arrival: y holds at 12 while x closes to 15 ✓");
}

// ─── Assertion 16: agent reaches the source tile during the loop ───────────
#[test]
fn harness_movement_agent_reaches_source_tile_during_loop() {
    // Type: A — de-vacuums A9: the agent's Position must equal (20,20) on at
    // least one tick of the 30-tick run (walked the full diagonal path).
    let r = run_gathering_loop();
    assert!(
        r.reached_source,
        "A16: agent must reach source (20,20) on at least one tick"
    );
    // Anchor: the run is the same one A4 measured (Hunger dropped).
    assert!(r.final_hunger < 50.0, "A16: same run as A4 — Hunger must have dropped");
    println!("[S16-β A16] agent reached (20,20) during the 30-tick loop ✓");
}
