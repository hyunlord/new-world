//! V7 Phase 7-δ — Social UI Integration harness.
//!
//! Verifies the new SimBridge collectors `collect_agent_snapshot` (extended
//! with `state_tag: u8`) and `collect_relationship_snapshot` plus the
//! localization keys gated by §4 of the feature prompt.
//!
//! plan: p7-delta-social-ui (plan_attempt 2, seed 42, agent_count 20)
//! lane: --full
//!
//! Run:
//!   `cargo test -p sim-test --test harness_p7_delta_social_ui -- --nocapture`

use std::collections::BTreeSet;

use sim_bridge::ffi::{
    collect_agent_snapshot, collect_relationship_snapshot, RelationshipSnapshotRow,
};
use sim_core::components::{
    Agent, AgentId, AgentState, Hunger, Position, RelationshipKey, RelationshipState, Sleep,
    Social, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{SimEngine, SimResources};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;

// ────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────

const W: u32 = 128;
const H: u32 = 128;

/// Mirror of the production Stage-1 engine factory used by the harness
/// plan. Same lattice, same component bag as `harness_p7_beta_social_system`.
fn make_stage1_engine(seed: u64, agent_count: u32) -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);
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

/// Build a tiny world with exactly one agent in `state`. Returns the
/// `state_tag` observed in the single snapshot row for that agent.
fn snapshot_single_state_tag(state: AgentState) -> u8 {
    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
    let entity = engine.spawn_agent(4, 4);
    engine
        .world
        .insert_one(entity, state)
        .expect("insert AgentState");
    let rows = collect_agent_snapshot(&engine.world);
    assert_eq!(rows.len(), 1, "expected exactly one snapshot row");
    rows[0].state_tag
}

/// Compute the §2-A-1 expected state_tag mapping directly from an
/// `AgentState` value. Used by A7 to ground the "same world, same call
/// site" comparison.
fn expected_tag(state: AgentState) -> u8 {
    match state {
        AgentState::Idle => 0,
        AgentState::Seeking { .. } => 1,
        AgentState::Consuming {
            target: TargetKind::Agent(_),
        } => 2,
        AgentState::Consuming { .. } => 3,
    }
}

/// Phase 7-γ scenario builder: two co-located agents at (6, 5), both
/// `Social::new(0.0, 1.0)`, all other needs `growth_rate = 0.0`. Returns
/// `(engine, entity_1, entity_2, id_1, id_2, rel_key)`.
#[allow(clippy::type_complexity)]
fn build_phase7_gamma_scenario() -> (
    SimEngine,
    hecs::Entity,
    hecs::Entity,
    AgentId,
    AgentId,
    RelationshipKey,
) {
    const SHARED_X: u32 = 6;
    const SHARED_Y: u32 = 5;
    let mut engine = SimEngine::new(12, 12, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);

    let entity_1 = engine.spawn_agent(5, 5);
    engine
        .world
        .insert(
            entity_1,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 1.0),
                AgentState::Idle,
            ),
        )
        .expect("insert needs bag on agent_1");

    let entity_2 = engine.spawn_agent(SHARED_X, SHARED_Y);
    engine
        .world
        .insert(
            entity_2,
            (
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                Sleep::new(0.0, 0.0),
                Social::new(0.0, 1.0),
                AgentState::Idle,
            ),
        )
        .expect("insert needs bag on agent_2");

    {
        let mut p1 = engine
            .world
            .get::<&mut Position>(entity_1)
            .expect("agent_1 Position");
        p1.x = SHARED_X;
        p1.y = SHARED_Y;
    }

    let id_1 = engine.world.get::<&Agent>(entity_1).unwrap().id;
    let id_2 = engine.world.get::<&Agent>(entity_2).unwrap().id;
    let rel_key = RelationshipKey::new(id_1, id_2);
    (engine, entity_1, entity_2, id_1, id_2, rel_key)
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 5 — state_tag_row_count_matches_agent_count
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_state_tag_row_count_matches_agent_count() {
    // Type A — snapshot includes every (Agent, Position, AgentState) row.
    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
    let states = [
        AgentState::Idle,
        AgentState::Seeking {
            target: TargetKind::Agent(7),
        },
        AgentState::Consuming {
            target: TargetKind::Agent(8),
        },
        AgentState::Consuming {
            target: TargetKind::Food,
        },
    ];
    for (i, s) in states.iter().enumerate() {
        let entity = engine.spawn_agent(2 + i as u32, 2);
        engine
            .world
            .insert_one(entity, *s)
            .expect("insert AgentState");
    }
    let rows = collect_agent_snapshot(&engine.world);
    assert_eq!(rows.len(), 4, "A5: snapshot row count must equal 4");
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 6 — state_tag_seeking_non_agent_target
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_state_tag_seeking_non_agent_target() {
    // Type A — Seeking → 1 regardless of TargetKind (no branching).
    let tag = snapshot_single_state_tag(AgentState::Seeking {
        target: TargetKind::Food,
    });
    assert_eq!(
        tag, 1,
        "A6: Seeking{{Food}} must still produce state_tag == 1"
    );
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 7 — state_tag matches live AgentState (same query, no caching)
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_state_tag_matches_live_agentstate_same_query() {
    // Type A — derives tag from same world the rest of the snapshot reads.
    let mut engine = make_stage1_engine(42, 20);
    for _ in 0..50 {
        engine.tick();
    }
    let rows = collect_agent_snapshot(&engine.world);
    for row in &rows {
        // Re-construct the Entity from raw bits for the cross-check.
        let entity =
            hecs::Entity::from_bits(row.entity_bits).expect("entity_bits is a valid hecs::Entity");
        let live = *engine
            .world
            .get::<&AgentState>(entity)
            .expect("snapshot row entity must have AgentState");
        let exp = expected_tag(live);
        assert_eq!(
            row.state_tag, exp,
            "A7: state_tag {} does not match expected {} for live state {:?}",
            row.state_tag, exp, live,
        );
    }
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 8 — relationship_snapshot empty with no relationships
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_relationship_snapshot_empty_when_no_relationships() {
    let engine = SimEngine::new(16, 16, MaterialRegistry::new());
    let rows = collect_relationship_snapshot(&engine.resources);
    assert_eq!(
        rows.len(),
        0,
        "A8: empty relationships must yield empty rows"
    );
}

// ────────────────────────────────────────────────────────────────────────
// Helper to mutate `resources.relationships` for filter tests.
// ────────────────────────────────────────────────────────────────────────
fn insert_pair(res: &mut SimResources, a: AgentId, b: AgentId, fam: f64, hos: f64) {
    let key = RelationshipKey::new(a, b);
    let mut state = RelationshipState::new();
    state.familiarity = fam;
    state.hostility = hos;
    res.relationships.insert(key, state);
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 9 — zero pair filtered out
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_relationship_snapshot_filters_zero_pairs() {
    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
    insert_pair(&mut engine.resources, 1, 2, 0.0, 0.0);
    let rows = collect_relationship_snapshot(&engine.resources);
    assert_eq!(rows.len(), 0, "A9: zero-only pair must be filtered");
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 10 — familiarity-only pair included
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_relationship_snapshot_includes_familiarity_only_pair() {
    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
    insert_pair(&mut engine.resources, 1, 2, 0.1, 0.0);
    let rows = collect_relationship_snapshot(&engine.resources);
    assert_eq!(rows.len(), 1, "A10: length must be 1");
    assert!(
        (rows[0].familiarity - 0.1).abs() < 1e-9,
        "A10: familiarity must round-trip 0.1, got {}",
        rows[0].familiarity,
    );
    assert!(
        (rows[0].hostility - 0.0).abs() < 1e-9,
        "A10: hostility must round-trip 0.0, got {}",
        rows[0].hostility,
    );
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 11 — hostility-only pair included
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_relationship_snapshot_includes_hostility_only_pair() {
    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
    insert_pair(&mut engine.resources, 1, 2, 0.0, 0.2);
    let rows = collect_relationship_snapshot(&engine.resources);
    assert_eq!(rows.len(), 1, "A11: length must be 1");
    assert!(
        (rows[0].familiarity - 0.0).abs() < 1e-9,
        "A11: familiarity must round-trip 0.0, got {}",
        rows[0].familiarity,
    );
    assert!(
        (rows[0].hostility - 0.2).abs() < 1e-9,
        "A11: hostility must round-trip 0.2, got {}",
        rows[0].hostility,
    );
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 12 — mixed pair: both fields preserved (no swap)
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_relationship_snapshot_includes_mixed_pair_both_fields_preserved() {
    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
    insert_pair(&mut engine.resources, 1, 2, 0.1, 0.05);
    let rows = collect_relationship_snapshot(&engine.resources);
    assert_eq!(rows.len(), 1, "A12: length must be 1");
    assert!(
        (rows[0].familiarity - 0.1).abs() < 1e-9,
        "A12: familiarity must be 0.1 (no swap), got {}",
        rows[0].familiarity,
    );
    assert!(
        (rows[0].hostility - 0.05).abs() < 1e-9,
        "A12: hostility must be 0.05 (no swap), got {}",
        rows[0].hostility,
    );
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 13 — negative values excluded
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_relationship_snapshot_excludes_negative_values() {
    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
    insert_pair(&mut engine.resources, 1, 2, -0.1, -0.1);
    let rows = collect_relationship_snapshot(&engine.resources);
    assert_eq!(
        rows.len(),
        0,
        "A13: negative-only pair must be filtered by > 0"
    );
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 14 — id_a < id_b canonical ordering
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_relationship_snapshot_id_a_lt_id_b_canonical() {
    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
    // Insert via both orderings; canonical key collapses to one entry.
    let _k1 = RelationshipKey::new(5, 2);
    let _k2 = RelationshipKey::new(2, 5);
    insert_pair(&mut engine.resources, 5, 2, 0.1, 0.0);
    insert_pair(&mut engine.resources, 7, 3, 0.0, 0.2);
    let rows = collect_relationship_snapshot(&engine.resources);
    assert!(!rows.is_empty(), "A14: expected at least one row");
    for row in &rows {
        assert!(
            row.id_a < row.id_b,
            "A14: row {row:?} must satisfy id_a < id_b",
        );
    }
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 15 — one row per logical pair (canonical-key dedupe)
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_relationship_snapshot_one_row_per_pair_after_two_inserts_same_pair() {
    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
    insert_pair(&mut engine.resources, 4, 9, 0.1, 0.0);
    insert_pair(&mut engine.resources, 9, 4, 0.1, 0.0);
    let rows = collect_relationship_snapshot(&engine.resources);
    assert_eq!(rows.len(), 1, "A15: canonical key must dedupe to 1 row");
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 16 — end-to-end socializing pair → state_tag == 2
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_end_to_end_socializing_pair_produces_snapshot_state_tag_2() {
    use sim_systems::runtime::REQUIRED_INTERACTION_PROGRESS;

    let (mut engine, entity_1, entity_2, _id_1, _id_2, rel_key) = build_phase7_gamma_scenario();

    let mut found_tick: Option<u64> = None;
    for _ in 0..80 {
        engine.tick();
        let now = engine.resources.current_tick;
        let s1 = engine
            .world
            .get::<&AgentState>(entity_1)
            .map(|s| *s)
            .unwrap_or(AgentState::Idle);
        let progress = engine
            .resources
            .interaction_progress
            .get(&rel_key)
            .copied()
            .unwrap_or(0);
        if matches!(
            s1,
            AgentState::Consuming {
                target: TargetKind::Agent(_)
            }
        ) && progress < REQUIRED_INTERACTION_PROGRESS
        {
            found_tick = Some(now);
            break;
        }
    }
    let qualifying_tick = found_tick.expect(
        "A16: no tick in [0,80) observed agent_1 in Consuming{Agent} with progress < REQUIRED",
    );
    // At the qualifying tick, both agents must read tag 2 from the snapshot.
    let rows = collect_agent_snapshot(&engine.world);
    let mut tags_for_pair: Vec<u8> = Vec::new();
    for ent in [entity_1, entity_2] {
        let bits = ent.to_bits().get();
        let row = rows
            .iter()
            .find(|r| r.entity_bits == bits)
            .expect("A16: pair entity must appear in snapshot");
        tags_for_pair.push(row.state_tag);
    }
    for tag in &tags_for_pair {
        assert_eq!(
            *tag, 2,
            "A16: at qualifying tick {qualifying_tick}, both pair tags must be 2, got {tags_for_pair:?}",
        );
    }
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 17 — relationship snapshot after completed interaction
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_end_to_end_relationship_snapshot_after_completed_interaction() {
    let (mut engine, entity_1, entity_2, id_1, id_2, rel_key) = build_phase7_gamma_scenario();
    for _ in 0..80 {
        engine.tick();
    }
    let rows = collect_relationship_snapshot(&engine.resources);
    let matched: Vec<&RelationshipSnapshotRow> = rows
        .iter()
        .filter(|r| {
            let (lo, hi) = if id_1 <= id_2 {
                (id_1, id_2)
            } else {
                (id_2, id_1)
            };
            r.id_a as u64 == lo && r.id_b as u64 == hi
        })
        .collect();
    assert_eq!(
        matched.len(),
        1,
        "A17: exactly one row must match the (id_1,id_2) pair",
    );
    // Hardcoded 0.1 — DO NOT replace with FAMILIARITY_BUMP import.
    assert!(
        (matched[0].familiarity - 0.1_f64).abs() < 1e-9,
        "A17: familiarity must equal hardcoded 0.1, got {}",
        matched[0].familiarity,
    );
    // Both agents back to Idle.
    for ent in [entity_1, entity_2] {
        let st = *engine
            .world
            .get::<&AgentState>(ent)
            .expect("agent must still have AgentState");
        assert_eq!(
            st,
            AgentState::Idle,
            "A17: agents must return to Idle after the cycle",
        );
    }
    // interaction_progress for the pair: absent OR within 1e-9 of 0.0.
    match engine.resources.interaction_progress.get(&rel_key).copied() {
        None => {}
        Some(p) => assert_eq!(p, 0, "A17: interaction_progress must reset (absent or 0)"),
    }
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 18 — state_tag == 0 after interaction
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_end_to_end_state_tag_idle_after_interaction() {
    let (mut engine, entity_1, entity_2, _id_1, _id_2, _rel_key) = build_phase7_gamma_scenario();
    for _ in 0..80 {
        engine.tick();
    }
    let rows = collect_agent_snapshot(&engine.world);
    for ent in [entity_1, entity_2] {
        let bits = ent.to_bits().get();
        let row = rows
            .iter()
            .find(|r| r.entity_bits == bits)
            .expect("agent must appear in final snapshot");
        assert_eq!(
            row.state_tag, 0,
            "A18: post-cycle state_tag must be 0 (Idle), got {}",
            row.state_tag,
        );
    }
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 22 — state_tag in {0,1,2,3} across a stage1 200-tick run
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_agent_snapshot_state_tag_byte_range() {
    let mut engine = make_stage1_engine(42, 20);
    let mut observed: BTreeSet<u8> = BTreeSet::new();
    for _ in 0..200 {
        engine.tick();
        let rows = collect_agent_snapshot(&engine.world);
        for row in &rows {
            assert!(
                matches!(row.state_tag, 0..=3),
                "A22: state_tag must be in {{0,1,2,3}}, got {}",
                row.state_tag,
            );
            observed.insert(row.state_tag);
        }
    }
    // Plan note: no requirement that all four values appear at runtime.
    assert!(
        !observed.is_empty(),
        "A22: at least one row must have been observed",
    );
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 23 — deterministic stream across two runs with the same seed
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_state_tag_stream_deterministic_across_two_runs_same_seed() {
    fn collect_stream(seed: u64) -> Vec<Vec<(u64, u8)>> {
        let mut engine = make_stage1_engine(seed, 20);
        let mut frames = Vec::with_capacity(100);
        for _ in 0..100 {
            engine.tick();
            let mut rows: Vec<(u64, u8)> = collect_agent_snapshot(&engine.world)
                .iter()
                .map(|r| (r.entity_bits, r.state_tag))
                .collect();
            rows.sort_by_key(|r| r.0);
            frames.push(rows);
        }
        frames
    }
    let a = collect_stream(42);
    let b = collect_stream(42);
    assert_eq!(a.len(), b.len(), "A23: tick counts must match");
    for (t, (fa, fb)) in a.iter().zip(b.iter()).enumerate() {
        assert_eq!(
            fa, fb,
            "A23: two runs diverged at tick {t}: lhs={fa:?}, rhs={fb:?}",
        );
    }
}
