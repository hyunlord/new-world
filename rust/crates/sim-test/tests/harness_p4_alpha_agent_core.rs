//! P4-α — Canonical Components + Agent Spawn (V7 Week 7-8 entry).
//!
//! V7 Phase 4 first deliverable: `sim_core::components::{Position, Agent}`
//! land as canonical ECS components, replacing the Phase 2 placeholder at
//! `sim_systems::runtime::influence::agent_sample::Position` per the
//! self-documenting landmark (`agent_sample.rs:9-15`).
//!
//! Threshold types follow `.harness/policy/evaluation_criteria.md`:
//!   - Type A — structural invariants (compile-time / shape / equality)
//!   - Type D — regression guards (pre-existing runtime paths stay green)
//!
//! Assertions:
//!   1.  Type A — `sim_core::components::{Position, Agent}` re-exports resolve
//!   2.  Type A — `Position { x: u32, y: u32 }` field layout (locked u32 per
//!       architecture invariant — tile coords, not pixels)
//!   3.  Type A — `Position::new` constructor produces the expected fields
//!   4.  Type A — `Agent` is a zero-sized marker (Copy + Default)
//!   5.  Type A — `SimEngine::spawn_agent(x: u32, y: u32) -> Entity` wrapper API
//!   6.  Type A — spawned entity has a `Position` component readable via ECS query
//!   7.  Type A — spawned entity has both `Position` and `Agent` (query tuple)
//!   8.  Type A — multiple `spawn_agent` calls produce distinct entity handles
//!   9.  Type A — canonical `Position` is the same type as
//!       `sim_systems::runtime::influence::agent_sample::Position`
//!       (compile-time landmark guarantee — "single-line rewire")
//!   10. Type D — `AgentInfluenceSampleSystem` reads canonical `Position`
//!       after the migration and still sees influence values stamped at the
//!       agent tile (runtime integration with priority 110 sampler)
//!   11. Type A — `Position::new(x, y)` equality / inequality holds
//!   12. Type A — pre-existing non-Agent entities survive `spawn_agent`,
//!       and the `Agent` marker query excludes them
//!
//! Run: `cargo test -p sim-test --test harness_p4_alpha_agent_core -- --nocapture`

use sim_core::components::{Agent, Position};
use sim_core::influence::InfluenceChannel;
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::influence::agent_sample::{
    AgentInfluenceSampleSystem, InfluenceSample,
};

const W: u32 = 32;
const H: u32 = 32;

fn fresh_engine() -> SimEngine {
    SimEngine::new(W, H, MaterialRegistry::new())
}

// ─── Assertion 1: components module re-exports ─────────────────────────────

/// Type A — compile-time re-export resolution.
/// `sim_core::components::{Position, Agent}` are visible re-exports.
/// Compile-time check via type aliases — failure would be a build error.
#[test]
fn harness_p4_alpha_components_mod_exports() {
    type _P = sim_core::components::Position;
    type _A = sim_core::components::Agent;
    type _PT = sim_core::Position;
    type _AT = sim_core::Agent;
    // Run-time witness so the test reports a clear pass.
    let _ = Position::new(0, 0);
    let _ = Agent { id: 0 };
}

// ─── Assertion 2: Position {x: u32, y: u32} field layout ────────────────────

/// Type A — structural invariant on Position field types.
/// Position fields are u32 (tile coordinates per CLAUDE.md architecture
/// invariant — pixel conversion lives in GDScript renderer).
#[test]
fn harness_p4_alpha_position_struct_fields_u32() {
    let p = Position { x: 7_u32, y: 11_u32 };
    let _x_check: u32 = p.x;
    let _y_check: u32 = p.y;
    assert_eq!(p.x, 7);
    assert_eq!(p.y, 11);
}

// ─── Assertion 3: Position::new constructor ────────────────────────────────

/// Type A — Position constructor round-trip (input → field readback).
#[test]
fn harness_p4_alpha_position_new_stores_fields() {
    let p = Position::new(5, 9);
    assert_eq!(p.x, 5);
    assert_eq!(p.y, 9);
}

// ─── Assertion 4: Agent identity struct layout (P5α migration) ──────────────

/// Type A — Agent layout contract.
///
/// Phase 4-α landed `Agent` as a ZST marker. V7 Phase 5-α (P5α-1) upgraded
/// it to `Agent { id: AgentId }`. The contract is now:
///   - `size_of::<Agent>() == size_of::<AgentId>()` (no padding)
///   - Copy semantics retained
///   - `Default` derive was dropped (a zero-id default would collide with
///     the first id minted by `SimResources::issue_agent_id`)
#[test]
fn harness_p4_alpha_agent_marker_zero_sized() {
    use sim_core::components::AgentId;
    assert_eq!(std::mem::size_of::<Agent>(), std::mem::size_of::<AgentId>());
    let a: Agent = Agent { id: 0 };
    let b: Agent = a; // Copy
    assert_eq!(a.id, b.id);
}

// ─── Assertion 5: SimEngine::spawn_agent API ───────────────────────────────

/// Type A — spawn_agent contract: returns a live Entity.
#[test]
fn harness_p4_alpha_spawn_agent_returns_entity() {
    let mut engine = fresh_engine();
    let entity = engine.spawn_agent(3, 4);
    assert!(engine.world.contains(entity));
}

// ─── Assertion 6: spawned entity has Position ──────────────────────────────

/// Type A — spawn_agent attaches Position with the caller's coordinates.
#[test]
fn harness_p4_alpha_spawned_agent_has_position() {
    let mut engine = fresh_engine();
    let entity = engine.spawn_agent(12, 17);
    let pos = *engine.world.get::<&Position>(entity).unwrap();
    assert_eq!(pos, Position::new(12, 17));
}

// ─── Assertion 7: spawned entity has Position + Agent ───────────────────────

/// Type A — spawn_agent attaches the (Position, Agent) tuple.
#[test]
fn harness_p4_alpha_spawned_agent_has_marker() {
    let mut engine = fresh_engine();
    let entity = engine.spawn_agent(1, 2);
    let mut q = engine.world.query_one::<(&Position, &Agent)>(entity).unwrap();
    let (pos, _agent) = q.get().expect("agent missing Position+Agent tuple");
    assert_eq!(pos.x, 1);
    assert_eq!(pos.y, 2);
}

// ─── Assertion 8: distinct entities ────────────────────────────────────────

/// Type A — distinct spawn_agent calls yield distinct Entity handles,
/// even at identical coordinates (co-location edge case).
#[test]
fn harness_p4_alpha_multiple_spawns_unique_entities() {
    let mut engine = fresh_engine();
    let a = engine.spawn_agent(0, 0);
    let b = engine.spawn_agent(0, 0);
    let c = engine.spawn_agent(1, 1);
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
    let count = engine.world.query::<&Agent>().iter().count();
    assert_eq!(count, 3);
}

// ─── Assertion 9: agent_sample::Position == canonical Position ──────────────

/// Type A — landmark contract: `agent_sample::Position` is the canonical
/// type (single-line rewire via `pub use sim_core::components::Position`).
#[test]
fn harness_p4_alpha_agent_sample_re_exports_canonical() {
    fn assert_same<T>(_: &T, _: &T) {}
    let a: Position = Position::new(0, 0);
    let b: sim_systems::runtime::influence::agent_sample::Position = Position::new(0, 0);
    assert_same(&a, &b);
    assert_eq!(a, b);
}

// ─── Assertion 10: AgentInfluenceSampleSystem reads canonical Position ─────

/// Type D — regression guard for the Phase 2 IUS runtime path.
/// Post-migration runtime integration: agent at (5, 5) sees a manually
/// stamped warmth=99 value after one sampler tick.
#[test]
fn harness_p4_alpha_agent_sample_runtime_integration() {
    let mut engine = fresh_engine();
    // Stamp warmth=99 at tile (5,5) via pending → swap.
    {
        let buf = engine
            .resources
            .influence_grid
            .pending_buf_mut(InfluenceChannel::Warmth);
        buf[5 * W as usize + 5] = 99;
    }
    engine.resources.influence_grid.swap();

    let agent = engine.spawn_agent(5, 5);
    // InfluenceSample must be present for the sampler to write into.
    engine
        .world
        .insert_one(agent, InfluenceSample::default())
        .unwrap();

    let mut sampler = AgentInfluenceSampleSystem::new();
    assert_eq!(sampler.priority(), 110);
    sampler.tick(&mut engine.world, &mut engine.resources);

    let sample = *engine.world.get::<&InfluenceSample>(agent).unwrap();
    assert_eq!(sample.warmth, 99);
}

// ─── Assertion 11: Position equality ────────────────────────────────────────

/// Type A — Position equality / inequality semantics.
#[test]
fn harness_p4_alpha_position_equality() {
    assert_eq!(Position::new(8, 8), Position::new(8, 8));
    assert_ne!(Position::new(8, 8), Position::new(8, 9));
}

// ─── Assertion 12: pre-existing non-Agent entity edge case ──────────────────

/// Type A — pre-existing entity edge case.
/// A non-Agent entity that exists in the world before any `spawn_agent`
/// call must:
///   (a) remain present after subsequent spawn_agent calls, and
///   (b) be excluded from the `Agent` marker query (count == number of
///       spawned agents, not number of entities).
#[test]
fn harness_p4_alpha_pre_existing_entity_excluded_from_agent_query() {
    let mut engine = fresh_engine();

    // Pre-existing non-Agent entity: Position only, no Agent marker.
    let pre = engine.world.spawn((Position::new(20, 20),));
    assert!(engine.world.contains(pre));

    // Spawn 2 agents.
    let a1 = engine.spawn_agent(1, 1);
    let a2 = engine.spawn_agent(2, 2);

    // (a) The pre-existing entity still exists.
    assert!(engine.world.contains(pre), "pre-existing entity vanished");
    assert!(engine.world.contains(a1));
    assert!(engine.world.contains(a2));
    // It still has its Position and no Agent marker.
    let pre_pos = *engine.world.get::<&Position>(pre).unwrap();
    assert_eq!(pre_pos, Position::new(20, 20));
    assert!(
        engine.world.get::<&Agent>(pre).is_err(),
        "pre-existing entity must NOT have the Agent marker"
    );

    // (b) Agent-marker query count equals the number of spawned agents (2),
    //     not the total entity count (3).
    let agent_count = engine.world.query::<&Agent>().iter().count();
    assert_eq!(agent_count, 2);

    // Sanity: Position query sees all 3 entities (pre + 2 agents).
    let position_count = engine.world.query::<&Position>().iter().count();
    assert_eq!(position_count, 3);
}
