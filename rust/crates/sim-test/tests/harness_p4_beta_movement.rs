//! P4-β — Agent Movement System + MovementRng plumbing (V7 Week 7-8).
//!
//! Builds on P4-α canonical components (`sim_core::components::{Position,
//! Agent}` + `SimEngine::spawn_agent`). Adds priority-120 `AgentMovementSystem`
//! and per-agent `MovementRng` so agents take deterministic Brownian steps
//! on every tick.
//!
//! Per planning §2.2 (`.harness/plans/phase4.md` lines 120-153):
//!   - Movement model: Brownian (deterministic seeded RNG)
//!   - Priority: ~120, after AgentInfluenceSampleSystem (priority 110)
//!   - tick_interval: 1
//!   - AgentDecision causal variant: DEFERRED (planning permits default
//!     deferral; AgentId infrastructure not yet in sim-core)
//!
//! Threshold types follow `.harness/policy/evaluation_criteria.md`:
//!   - Type A — structural invariants (compile-time / shape / equality)
//!   - Type D — regression guards (pre-existing runtime paths stay green)
//!
//! Assertions:
//!   1.  Type A — `AgentMovementSystem` re-exports resolve at
//!       `sim_systems::runtime::agent::AgentMovementSystem`.
//!   2.  Type A — system metadata (name / priority=120 / tick_interval=1).
//!   3.  Type A — `MovementRng` re-exports resolve at
//!       `sim_systems::runtime::agent::MovementRng`.
//!   4.  Type A — `MovementRng::new(seed)` produces a working PRNG that
//!       escapes the all-zero state (splitmix64 invariant).
//!   5.  Type A — single agent at (10,10) with seed=42 moves on the first
//!       tick by at most 1 cell on each axis (Brownian neighborhood).
//!   6.  Type A — full determinism: replaying with identical seeds produces
//!       byte-identical trajectories across 16 ticks.
//!   7.  Type A — distinct seeds produce distinct trajectories (within 32
//!       ticks the two walks diverge at least once).
//!   8.  Type D — multi-agent independence: agent with seed=7 produces the
//!       same 16-tick trajectory solo and alongside a seed-13 co-tenant.
//!   9.  Type A — boundary clamp: an agent spawned at (0,0) and another at
//!       (W-1,H-1) survive 200 ticks without ever leaving the grid (u32
//!       underflow / out-of-bounds protection).
//!   10. Type D — `register_agent_systems(&mut engine)` registers the
//!       movement system and 8 subsequent `engine.tick()` calls advance the
//!       single spawned agent away from its initial position.
//!   11. Type D — `register_phase2_systems` + `register_agent_systems`
//!       coexist: after one full `engine.tick()` an agent that started at
//!       a freshly-stamped warmth source reads `InfluenceSample::warmth > 0`
//!       (IUS@100 propagates → AIS@110 samples → AMS@120 moves).
//!
//! Run: `cargo test -p sim-test --test harness_p4_beta_movement -- --nocapture`

use sim_core::components::{Agent, Position};
use sim_engine::{BuildingPlacedEvent, RuntimeSystem, SimEngine};
use sim_core::material::MaterialRegistry;
use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
use sim_systems::runtime::influence::agent_sample::InfluenceSample;
use sim_systems::{register_agent_systems, register_phase2_systems};

const W: u32 = 32;
const H: u32 = 32;

fn fresh_engine() -> SimEngine {
    SimEngine::new(W, H, MaterialRegistry::new())
}

// ─── Assertion 1: AgentMovementSystem re-export ────────────────────────────

#[test]
fn harness_p4_beta_movement_system_export_resolves() {
    type _S = sim_systems::runtime::agent::AgentMovementSystem;
    let s = AgentMovementSystem::new();
    assert_eq!(s.name(), "AgentMovementSystem");
}

// ─── Assertion 2: system metadata ──────────────────────────────────────────

#[test]
fn harness_p4_beta_system_metadata() {
    let s = AgentMovementSystem::new();
    assert_eq!(s.name(), "AgentMovementSystem");
    assert_eq!(s.priority(), 120, "priority must be 120 per planning §2.2");
    assert_eq!(s.tick_interval(), 1, "tick_interval must be 1 (every tick)");
}

// ─── Assertion 3: MovementRng re-export ────────────────────────────────────

#[test]
fn harness_p4_beta_rng_export_resolves() {
    type _R = sim_systems::runtime::agent::MovementRng;
    let _r = MovementRng::new(0);
}

// ─── Assertion 4: splitmix64 escapes zero seed ─────────────────────────────

#[test]
fn harness_p4_beta_rng_escapes_zero_seed() {
    let mut r = MovementRng::new(0);
    // splitmix64 with a non-zero additive constant must produce a non-zero
    // first sample even when seeded with zero.
    let first = r.next_u64();
    assert_ne!(first, 0, "splitmix64 must escape the all-zero state");
}

// ─── Assertion 5: one tick → Brownian neighborhood ─────────────────────────

#[test]
fn harness_p4_beta_one_tick_brownian_step() {
    let mut e = fresh_engine();
    let id = e
        .world
        .spawn((Position::new(10, 10), Agent { id: 0 }, MovementRng::new(42)));
    let mut sys = AgentMovementSystem::new();
    sys.tick(&mut e.world, &mut e.resources);
    let p = *e.world.get::<&Position>(id).unwrap();
    assert!(
        p.x.abs_diff(10) <= 1,
        "x must move by at most 1 cell after one tick, got {} (was 10)",
        p.x
    );
    assert!(
        p.y.abs_diff(10) <= 1,
        "y must move by at most 1 cell after one tick, got {} (was 10)",
        p.y
    );
}

// ─── Assertion 6: full determinism across 16 ticks ─────────────────────────

#[test]
fn harness_p4_beta_determinism_full_trajectory() {
    fn trajectory(seed: u64) -> Vec<(u32, u32)> {
        let mut e = fresh_engine();
        let id = e
            .world
            .spawn((Position::new(16, 16), Agent { id: 0 }, MovementRng::new(seed)));
        let mut sys = AgentMovementSystem::new();
        let mut out = Vec::with_capacity(16);
        for _ in 0..16 {
            sys.tick(&mut e.world, &mut e.resources);
            let p = *e.world.get::<&Position>(id).unwrap();
            out.push((p.x, p.y));
        }
        out
    }
    let a = trajectory(0xC0FFEE);
    let b = trajectory(0xC0FFEE);
    assert_eq!(a, b, "identical seed must produce identical trajectory");
    assert_eq!(a.len(), 16);
}

// ─── Assertion 7: distinct seeds → distinct trajectories ───────────────────

#[test]
fn harness_p4_beta_distinct_seeds_diverge() {
    let mut e = fresh_engine();
    let a = e
        .world
        .spawn((Position::new(16, 16), Agent { id: 0 }, MovementRng::new(1)));
    let b = e
        .world
        .spawn((Position::new(16, 16), Agent { id: 0 }, MovementRng::new(2)));
    let mut sys = AgentMovementSystem::new();
    let mut diverged = false;
    for _ in 0..32 {
        sys.tick(&mut e.world, &mut e.resources);
        let pa = *e.world.get::<&Position>(a).unwrap();
        let pb = *e.world.get::<&Position>(b).unwrap();
        if pa != pb {
            diverged = true;
            break;
        }
    }
    assert!(
        diverged,
        "two agents with distinct seeds must diverge within 32 ticks"
    );
}

// ─── Assertion 8: multi-agent stream independence ──────────────────────────
//
// Plan-locked specification (`.harness/prompts/p4-beta-movement-decision.md`
// Section 3.5 row 8): "solo trajectory (seed 7, 16 ticks) == multi-tenant
// trajectory of same seed alongside seed 13".

#[test]
fn harness_p4_beta_multi_agent_independence() {
    // Run agent A (seed 7) solo for 16 ticks → expected trajectory.
    let mut solo_engine = fresh_engine();
    let solo = solo_engine
        .world
        .spawn((Position::new(16, 16), Agent { id: 0 }, MovementRng::new(7)));
    let mut solo_sys = AgentMovementSystem::new();
    let mut solo_traj = Vec::with_capacity(16);
    for _ in 0..16 {
        solo_sys.tick(&mut solo_engine.world, &mut solo_engine.resources);
        let p = *solo_engine.world.get::<&Position>(solo).unwrap();
        solo_traj.push((p.x, p.y));
    }

    // Run the same seed-7 agent alongside a seed-13 co-tenant → seed-7
    // trajectory must be byte-identical to the solo run.
    let mut multi_engine = fresh_engine();
    let a = multi_engine
        .world
        .spawn((Position::new(16, 16), Agent { id: 0 }, MovementRng::new(7)));
    let _b = multi_engine
        .world
        .spawn((Position::new(5, 5), Agent { id: 0 }, MovementRng::new(13)));
    let mut multi_sys = AgentMovementSystem::new();
    let mut multi_traj = Vec::with_capacity(16);
    for _ in 0..16 {
        multi_sys.tick(&mut multi_engine.world, &mut multi_engine.resources);
        let p = *multi_engine.world.get::<&Position>(a).unwrap();
        multi_traj.push((p.x, p.y));
    }

    // Diagnostic: how many of the 16 sampled positions actually changed
    // between consecutive ticks (Brownian zero-step is allowed, so this
    // is a soft signal, not an assertion target).
    let agents_doing_movement = solo_traj
        .windows(2)
        .filter(|w| w[0] != w[1])
        .count();
    eprintln!(
        "[p4-beta][multi_agent_independence] agents_doing_movement={} \
         (transitions where (x,y) changed across the 16-tick solo run)",
        agents_doing_movement
    );

    assert_eq!(
        solo_traj, multi_traj,
        "seed-7 trajectory must not depend on a co-tenant seed-13 agent"
    );
}

// ─── Assertion 9: boundary clamp at both corners ───────────────────────────

#[test]
fn harness_p4_beta_boundary_clamp() {
    let mut e = fresh_engine();
    let nw = e
        .world
        .spawn((Position::new(0, 0), Agent { id: 0 }, MovementRng::new(7)));
    let se = e.world.spawn((
        Position::new(W - 1, H - 1),
        Agent { id: 1 },
        MovementRng::new(13),
    ));
    let mut sys = AgentMovementSystem::new();
    for _ in 0..200 {
        sys.tick(&mut e.world, &mut e.resources);
        let p_nw = *e.world.get::<&Position>(nw).unwrap();
        let p_se = *e.world.get::<&Position>(se).unwrap();
        assert!(p_nw.x < W, "NW agent left grid on x: {}", p_nw.x);
        assert!(p_nw.y < H, "NW agent left grid on y: {}", p_nw.y);
        assert!(p_se.x < W, "SE agent left grid on x: {}", p_se.x);
        assert!(p_se.y < H, "SE agent left grid on y: {}", p_se.y);
    }
}

// ─── Assertion 10: register_agent_systems + tick progress ──────────────────
//
// Plan-locked specification (`.harness/prompts/p4-beta-movement-decision.md`
// Section 3.5 row 10): "register_agent_systems(&mut engine) + 1 spawn +
// 8 advance ticks → at least one position changed".

#[test]
fn harness_p4_beta_register_and_progress() {
    let mut e = fresh_engine();
    let initial = Position::new(8, 8);
    let id = e
        .world
        .spawn((initial, Agent { id: 0 }, MovementRng::new(0x1000)));

    register_agent_systems(&mut e);

    for _ in 0..8 {
        e.tick();
    }

    let after = *e.world.get::<&Position>(id).unwrap();
    let moved = (after.x, after.y) != (initial.x, initial.y);
    let agents_moved = if moved { 1 } else { 0 };
    eprintln!(
        "[p4-beta][register_and_progress] agents_moved={} after 8 engine.tick()s \
         (initial=({},{}) final=({},{}))",
        agents_moved, initial.x, initial.y, after.x, after.y
    );
    assert!(
        moved,
        "spawned agent must have moved from initial=({},{}) after 8 engine.tick()s, \
         got final=({},{})",
        initial.x, initial.y, after.x, after.y
    );
}

// ─── Assertion 11: AgentInfluenceSampleSystem coexistence (regression) ─────
//
// Plan-locked specification (`.harness/prompts/p4-beta-movement-decision.md`
// Section 3.5 row 11): "stamp Warmth at agent's pre-move tile, register
// Phase 2 + agent stacks, 1 tick → `InfluenceSample::warmth > 0` (IUS@110
// still upstream of AMS@120)". We use the canonical BuildingStampSystem
// path (queue a BuildingPlacedEvent for BSS@90 to drain), so the warmth
// arrives via IUS propagation rather than a manual buffer poke. AIS@110
// then samples the canonical Position before AMS@120 moves the agent.

#[test]
fn harness_p4_beta_influence_sampler_still_reads_canonical_position() {
    let mut e = fresh_engine();

    // Queue a building placement at the agent's spawn tile so BSS@90
    // marks Warmth dirty → IUS@100 propagates → AIS@110 samples non-zero
    // warmth into the agent's InfluenceSample → AMS@120 moves the agent.
    e.resources
        .building_event_queue
        .push_back(BuildingPlacedEvent {
            position: (12, 12),
            radius: 2,
        });

    let id = e.world.spawn((
        Position::new(12, 12),
        Agent { id: 0 },
        InfluenceSample::default(),
        MovementRng::new(99),
    ));

    register_phase2_systems(&mut e);
    register_agent_systems(&mut e);

    e.tick();

    let sample = *e.world.get::<&InfluenceSample>(id).unwrap();
    let p_after = *e.world.get::<&Position>(id).unwrap();

    // Diagnostic: how many agents observed non-zero warmth this tick
    // (here N=1 by construction, but expose it for --nocapture audits).
    let agents_doing_movement = e
        .world
        .query::<(&Position, &MovementRng)>()
        .iter()
        .count();
    let agents_moved = if (p_after.x, p_after.y) != (12, 12) { 1 } else { 0 };
    eprintln!(
        "[p4-beta][sampler_coexist] agents_doing_movement={} agents_moved={} \
         warmth={} final=({},{})",
        agents_doing_movement, agents_moved, sample.warmth, p_after.x, p_after.y
    );

    assert!(
        sample.warmth > 0,
        "after one engine.tick() AIS@110 must have sampled warmth > 0 at \
         (12,12) — got {}",
        sample.warmth
    );
    // Movement system also ran (AMS@120) and must have kept the agent on-grid.
    assert!(p_after.x < W, "agent left grid on x after one engine.tick()");
    assert!(p_after.y < H, "agent left grid on y after one engine.tick()");
}
