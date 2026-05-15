//! P5-α — AgentId infrastructure + Hunger component + HungerDecaySystem
//! (V7 Phase 5, First Daily Routine, Week 9-10).
//!
//! Locked test plan: `feature: p5-alpha-agent-id-hunger, plan_attempt: 2`.
//! All 12 assertions below match the plan thresholds exactly — values,
//! sub-case counts, and equalities are not negotiable.
//!
//! Threshold types follow `.harness/policy/evaluation_criteria.md`:
//!   - Type A — structural invariants (compile-time / shape / equality)
//!   - Type D — regression guards (pre-existing runtime paths stay green)
//!
//! Run: `cargo test -p sim-test --test harness_p5_alpha_agent_id_hunger -- --nocapture`

use sim_core::components::{Agent, AgentId, Hunger, Position};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::needs::HungerDecaySystem;
use sim_systems::{register_agent_systems, register_needs_systems, register_phase2_systems};

const W: u32 = 64;
const H: u32 = 64;

fn fresh_engine() -> SimEngine {
    SimEngine::new(W, H, MaterialRegistry::new())
}

// ── Assertion 1: agent_id_is_u64_alias ─────────────────────────────────
// Type A — direct bindings between `AgentId` and `u64` with NO `as` cast,
// NO `From`/`Into`, NO `.into()`. A newtype-shaped `AgentId` would force
// at least one of those, so a plain `let _: AgentId = 0u64;` round-trip
// is the only spelling that pins the transparent type-alias contract.
#[test]
fn harness_p5_alpha_agent_id_is_u64_alias() {
    // Direct binding from `u64` literal to `AgentId` (no cast, no into).
    let _: AgentId = 0u64;

    // Round-trip both directions with no conversion at all.
    let x: AgentId = 7u64;
    let y: u64 = x;
    assert_eq!(y, 7u64);
    println!("[α-1] AgentId is a transparent u64 alias (no cast / no into) ✓");
}

// ── Assertion 2: agent_struct_exposes_public_id_field ──────────────────
// Type A — `Agent { id: 42 }` (struct-literal, named field, pub id)
// constructs and round-trips. The tuple form `Agent(42)` MUST fail to
// compile; this is documented in the doc-comment below for completeness
// (Rust rejects `Agent(42)` for a named-field struct at compile time —
// no runtime test can express that without trybuild, but the inverse —
// the brace form succeeding — is itself proof of named-field shape,
// because brace syntax requires named fields).
//
// ```compile_fail
// # use sim_core::components::Agent;
// // Tuple-construction must NOT compile — Agent is a named-field struct.
// let _ = Agent(42);
// ```
#[test]
fn harness_p5_alpha_agent_struct_exposes_public_id_field() {
    let a = Agent { id: 42 };
    // Exact equality to `42u64` — proves no transformation on field assign.
    assert_eq!(a.id, 42u64);
    println!("[α-2] Agent {{ id: 42 }}.id == 42u64 (named-field, pub id) ✓");
}

// ── Assertion 3: agent_default_derive_is_absent ────────────────────────
// Type A — `Agent::default()` MUST fail to compile (Default not derived).
// Verified by a method-priority trait probe: an inherent method exists
// only for types that satisfy `T: Default`. For non-Default types, only
// the fallback trait method exists, and method resolution picks it.
//
// Positive control: `u64` (known Default) resolves to the "is_default"
// branch. Negative test: `Agent` resolves to the "not_default" branch.
//
// (A direct `Agent::default()` call would force a compile failure,
// which is not expressible inside a passing test without trybuild —
// hence the trait-probe per the plan's "OR" clause.)
mod default_probe {
    use std::marker::PhantomData;

    /// Phantom wrapper used as the method receiver.
    pub struct DefaultProbe<T>(pub PhantomData<T>);

    impl<T> DefaultProbe<T> {
        pub fn new() -> Self {
            Self(PhantomData)
        }
    }

    // Inherent method exists only when `T: Default` — Rust's method
    // resolution filters this out for non-Default types.
    impl<T: Default> DefaultProbe<T> {
        pub fn branch(&self) -> &'static str {
            "is_default"
        }
    }

    /// Catch-all fallback trait — method resolution picks this when the
    /// inherent method's `T: Default` bound fails.
    pub trait DefaultProbeFallback {
        fn branch(&self) -> &'static str {
            "not_default"
        }
    }
    impl<T> DefaultProbeFallback for DefaultProbe<T> {}
}

#[test]
fn harness_p5_alpha_agent_default_derive_is_absent() {
    use default_probe::{DefaultProbe, DefaultProbeFallback as _};

    let agent_probe: DefaultProbe<Agent> = DefaultProbe::new();
    let u64_probe: DefaultProbe<u64> = DefaultProbe::new();

    // Positive control: u64 is Default → inherent method wins.
    assert_eq!(
        u64_probe.branch(),
        "is_default",
        "positive control failed: u64 must resolve to is_default"
    );
    // Negative: Agent has no Default → fallback trait method wins.
    assert_eq!(
        agent_probe.branch(),
        "not_default",
        "Agent must NOT impl Default — fallback branch must win"
    );
    println!("[α-3] Agent does NOT implement Default (positive ctrl: u64 does) ✓");
}

// ── Assertion 4: agent_layout_matches_agent_id ─────────────────────────
// Type A — exact size AND align equality. A larger size would imply
// stray fields or repr corruption; a smaller size is impossible.
#[test]
fn harness_p5_alpha_agent_layout_matches_agent_id() {
    use std::mem::{align_of, size_of};
    assert_eq!(
        size_of::<Agent>(),
        size_of::<AgentId>(),
        "size_of::<Agent>() must equal size_of::<AgentId>()"
    );
    assert_eq!(
        align_of::<Agent>(),
        align_of::<AgentId>(),
        "align_of::<Agent>() must equal align_of::<AgentId>()"
    );
    println!(
        "[α-4] layout: size={} align={} (Agent ≡ AgentId) ✓",
        size_of::<Agent>(),
        align_of::<Agent>()
    );
}

// ── Assertion 5: spawn_mints_ids_strictly_monotonic_from_zero ──────────
// Type A — exact sequence `[0, 1, 2, 3, 4]` on a fresh engine.
#[test]
fn harness_p5_alpha_spawn_mints_ids_strictly_monotonic_from_zero() {
    let mut engine = fresh_engine();
    // Spawn 5 agents at distinct positions.
    let entities = [
        engine.spawn_agent(0, 0),
        engine.spawn_agent(1, 1),
        engine.spawn_agent(2, 2),
        engine.spawn_agent(3, 3),
        engine.spawn_agent(4, 4),
    ];

    let mut ids = [0u64; 5];
    for (idx, ent) in entities.iter().enumerate() {
        ids[idx] = engine
            .world
            .get::<&Agent>(*ent)
            .expect("entity must carry Agent")
            .id;
    }

    assert_eq!(
        ids,
        [0u64, 1, 2, 3, 4],
        "fresh engine must mint AgentIds [0,1,2,3,4] in spawn order"
    );
    println!("[α-5] spawn-id sequence: {ids:?} ✓");
}

// ── Assertion 6: counter_advance_and_spawn_share_state ─────────────────
// Type A — the shared-counter contract. `issue_agent_id` advances the
// SAME `AtomicU64` that `spawn_agent` consumes next. If both returned 0
// the test FAILS (would indicate two separate counters).
#[test]
fn harness_p5_alpha_counter_advance_and_spawn_share_state() {
    let mut engine = fresh_engine();
    // On a fresh engine the counter sits at 0. issue_agent_id reads 0
    // and bumps the counter to 1; the next spawn must mint id = 1.
    let predicted = engine.resources.issue_agent_id();
    let spawned = engine.spawn_agent(7, 7);
    let actual = engine
        .world
        .get::<&Agent>(spawned)
        .expect("entity must carry Agent")
        .id;

    // The "two separate counters" failure mode: predicted=0 AND actual=0.
    assert!(
        !(predicted == 0 && actual == 0),
        "predicted=0 with actual=0 would mean issue_agent_id and spawn_agent \
         draw from separate counters — contract violation"
    );
    // The shared-counter contract: actual is exactly predicted + 1.
    assert_eq!(
        actual,
        predicted + 1,
        "spawn_agent must mint the id immediately after issue_agent_id's read \
         (shared AtomicU64); got predicted={predicted}, actual={actual}"
    );
    // And on a truly fresh engine, predicted is 0 → actual is 1.
    assert_eq!(predicted, 0, "fresh engine counter must start at 0");
    assert_eq!(actual, 1, "first post-issue spawn must mint id 1");
    println!("[α-6] shared counter: predicted={predicted}, spawned={actual} ✓");
}

// ── Assertion 7: hunger_constructor_clamps_initial_value ───────────────
// Type A — three constructor inputs exercise lower-clamp, upper-clamp,
// and pass-through branches. Exact f32 equality (all three target values
// are bit-representable).
#[test]
fn harness_p5_alpha_hunger_constructor_clamps_initial_value() {
    let lo = Hunger::new(-5.0, 1.0);
    assert_eq!(lo.value, 0.0_f32, "negative initial must clamp to 0.0");

    let hi = Hunger::new(999.0, 1.0);
    assert_eq!(
        hi.value,
        Hunger::SATURATION,
        "above-SATURATION initial must clamp to SATURATION (100.0)"
    );
    assert_eq!(hi.value, 100.0_f32);

    let mid = Hunger::new(37.5, 1.0);
    assert_eq!(mid.value, 37.5_f32, "in-range initial must pass through");
    println!("[α-7] Hunger::new clamps: -5→0.0, 999→100.0, 37.5→37.5 ✓");
}

// ── Assertion 8: hunger_tick_saturates_at_upper_bound_and_is_stable ────
// Type A — saturate-and-hold. One tick caps at 100, then 1000 more
// ticks must leave it at 100 (stable fixed point, not a one-shot cap).
#[test]
fn harness_p5_alpha_hunger_tick_saturates_at_upper_bound_and_is_stable() {
    let mut h = Hunger::new(95.0, 10.0);
    // 95 + 10 = 105 → saturates to 100.
    h.tick();
    assert_eq!(
        h.value,
        Hunger::SATURATION,
        "first tick must saturate at SATURATION (100.0)"
    );
    assert_eq!(h.value, 100.0_f32);

    // 1000 more ticks at SATURATION — must remain exactly 100.0.
    for _ in 0..1000 {
        h.tick();
    }
    assert_eq!(
        h.value,
        Hunger::SATURATION,
        "value must stay at SATURATION across 1000 repeated ticks (stable cap)"
    );
    println!("[α-8] saturation is a stable fixed point (1001 ticks ⇒ 100.0) ✓");
}

// ── Assertion 9: hunger_tick_floors_at_zero_under_negative_rate ────────
// Type A — negative growth_rate exercises the lower floor. After two
// ticks at the floor, value must remain exactly 0.0 (stable floor).
#[test]
fn harness_p5_alpha_hunger_tick_floors_at_zero_under_negative_rate() {
    let mut h = Hunger::new(2.0, -5.0);
    // 2 - 5 = -3 → floor at 0.
    h.tick();
    assert_eq!(h.value, 0.0_f32, "first tick under negative rate must floor at 0");

    // Second tick from 0 with -5 rate: -5 → must still floor at 0.
    h.tick();
    assert_eq!(
        h.value, 0.0_f32,
        "second tick must stay at 0 — floor is stable, not transient"
    );
    println!("[α-9] zero-floor is stable under negative growth rate ✓");
}

// ── Assertion 10: hunger_system_metadata_matches_locked_priority_band ──
// Type A — exact name, priority, tick_interval (LOCKED facts P5α-5).
#[test]
fn harness_p5_alpha_hunger_system_metadata_matches_locked_priority_band() {
    let sys = HungerDecaySystem::new();
    assert_eq!(sys.name(), "HungerDecaySystem");
    assert_eq!(sys.priority(), 130u32, "priority MUST be exactly 130");
    assert_eq!(sys.tick_interval(), 1u64, "tick_interval MUST be exactly 1");
    println!("[α-10] HungerDecaySystem name='HungerDecaySystem' prio=130 interval=1 ✓");
}

// ── Assertion 11: hunger_system_advances_value_at_unit_rate_per_engine_tick
// Type A — three sub-cases on independent fresh engines: 0, 7, 8 engine
// ticks. With growth_rate = 1.0 and starting value 0.0, post-tick value
// equals tick count exactly.
fn run_unit_rate_subcase(ticks: u32) -> f32 {
    let mut engine = fresh_engine();
    register_needs_systems(&mut engine);

    let agent = engine.spawn_agent(8, 8);
    engine
        .world
        .insert_one(agent, Hunger::new(0.0, 1.0))
        .expect("attach Hunger");

    for _ in 0..ticks {
        engine.tick();
    }

    let hunger = engine
        .world
        .get::<&Hunger>(agent)
        .expect("agent must carry Hunger");
    hunger.value
}

#[test]
fn harness_p5_alpha_hunger_system_advances_value_at_unit_rate_per_engine_tick() {
    let v0 = run_unit_rate_subcase(0);
    let v7 = run_unit_rate_subcase(7);
    let v8 = run_unit_rate_subcase(8);

    assert_eq!(v0, 0.0_f32, "after 0 engine ticks, value must be 0.0");
    assert_eq!(v7, 7.0_f32, "after 7 engine ticks, value must be 7.0");
    assert_eq!(v8, 8.0_f32, "after 8 engine ticks, value must be 8.0");
    println!("[α-11] unit-rate advancement: 0→{v0}, 7→{v7}, 8→{v8} ✓");
}

// ── Assertion 12: full_stack_phase2_agent_needs_coexist_without_interference
// Type D — cross-stack regression guard. Seed 42, three stacks, agent at
// (16, 16). After 8 ticks: id stable, position moved, hunger == 8.0,
// entity still exists.
#[test]
fn harness_p5_alpha_full_stack_phase2_agent_needs_coexist_without_interference() {
    // Seed 42 is the locked harness seed. The engine constructor takes
    // (width, height, registry) — no explicit seed slot today — so the
    // seed surfaces via MovementRng on the agent and the resources stay
    // at their default deterministic state.
    let mut engine = fresh_engine();
    register_phase2_systems(&mut engine);
    register_agent_systems(&mut engine);
    register_needs_systems(&mut engine);

    let agent = engine.spawn_agent(16, 16);
    engine
        .world
        .insert(agent, (MovementRng::new(42), Hunger::new(0.0, 1.0)))
        .expect("attach MovementRng + Hunger");

    let id_before = engine
        .world
        .get::<&Agent>(agent)
        .expect("agent must carry Agent")
        .id;

    for _ in 0..8 {
        engine.tick();
    }

    // (d) entity still exists.
    assert!(
        engine.world.contains(agent),
        "agent entity must still exist after 8 ticks"
    );

    // (a) Agent.id is stable across ticks.
    let id_after = engine
        .world
        .get::<&Agent>(agent)
        .expect("agent must still carry Agent")
        .id;
    assert_eq!(
        id_after, id_before,
        "Agent.id must be stable across ticks — no system re-stamps it"
    );

    // (b) Position has changed in at least one coordinate.
    let post_pos = *engine
        .world
        .get::<&Position>(agent)
        .expect("agent must still carry Position");
    assert!(
        (post_pos.x, post_pos.y) != (16, 16),
        "AgentMovementSystem must have moved the agent within 8 ticks; got {post_pos:?}"
    );

    // (c) Hunger.value advanced at unit rate (8 ticks × 1.0/tick).
    let post_hunger = engine
        .world
        .get::<&Hunger>(agent)
        .expect("agent must still carry Hunger")
        .value;
    assert_eq!(
        post_hunger, 8.0_f32,
        "HungerDecaySystem must have advanced value to exactly 8.0 over 8 ticks"
    );

    println!(
        "[α-12] full stack OK: id={id_before} (stable), pos {:?}, hunger {post_hunger} ✓",
        (post_pos.x, post_pos.y)
    );
}
