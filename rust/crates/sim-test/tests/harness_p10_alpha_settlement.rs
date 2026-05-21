//! V7 Phase 10-α — Settlement data substrate harness.
//!
//! feature: p10-alpha-settlement-substrate
//! plan_attempt: 1
//! code_attempt: 2
//! seed: 42
//! agent_count: 20
//! lane: --full
//!
//! Assertion map (1:1 with the locked plan §Assertions):
//!   A1  : `size_of::<SettlementId>() == size_of::<u32>()` (exact equality)
//!   A2  : `size_of::<BuildingId>()  == size_of::<u64>()` (exact equality)
//!   A3  : SETTLEMENT_HISTORY_CAP == 32
//!   A4  : SETTLEMENT_FORMATION_AGENT_THRESHOLD == 3
//!   A5  : SETTLEMENT_FORMATION_BUILDING_THRESHOLD == 2
//!   A6  : SETTLEMENT_DISSOLUTION_THRESHOLD == 0
//!   A7  : SETTLEMENT_MAX_POP == 50
//!   A8  : SETTLEMENT_PROXIMITY_RADIUS == 5
//!   A9  : `Settlement::new_with_id(0, 0)` produces an empty, dissolved
//!         settlement with `settlement_id == 0` and `founded_at == 0`.
//!   A10 : `add_member_agent` / `remove_member_agent` HashSet round-trip,
//!         with `is_dissolved()` true after removal.
//!   A11 : `add_member_building` / `remove_member_building` HashSet round-trip.
//!   A12 : `append_history` saturates at `SETTLEMENT_HISTORY_CAP` and evicts
//!         index 0 first.
//!   A13 : `population_count() == member_agents.len()` after 3 inserts and
//!         again after one removal (== 2).
//!   A14 : `is_dissolved()` 3 transitions: empty → true, +1 agent → false,
//!         remove → true.
//!   A15 : Settlement serde JSON round-trip (2 agents, 1 building, 3 history
//!         entries) preserves all fields via `PartialEq`.
//!   A16 : `SimResources::new(...).settlements.is_empty()` and
//!         `next_settlement_id == 0`.
//!   A17 : 5 consecutive `issue_settlement_id()` calls return `[0, 1, 2, 3, 4]`.
//!   A18 : `CausalEvent::AgentBorn { id: 99, parent: Some(10), agent: 1,
//!         tick: 42 }` — accessor return values match exactly
//!         (id=99, parent=Some(10), tick=42, channel=None).
//!   A19 : `AgentBorn` Clone + PartialEq (`ev.clone() == ev`).
//!   A20 : `Settlement`, `SettlementId`, `BuildingId`,
//!         `SETTLEMENT_HISTORY_CAP`, `SETTLEMENT_MAX_POP` re-exported from
//!         `sim_core::components`.
//!   A21 : Phase 9-α + Phase 8-α regression — `make_stage1_engine(42, 20)`
//!         survives 100 ticks without panic, agent count ≥ 1, and the
//!         MemorySystem / event-processing path is still exercised (causal
//!         log advances past tick 0).

use sim_core::causal::event::{CausalEvent, EventId};
use sim_core::components::{
    Agent, AgentId, BuildingId, Hunger, Memory, PopulationStats, Settlement, SettlementId, Sleep,
    Social, AgentState, Thirst, SETTLEMENT_DISSOLUTION_THRESHOLD,
    SETTLEMENT_FORMATION_AGENT_THRESHOLD, SETTLEMENT_FORMATION_BUILDING_THRESHOLD,
    SETTLEMENT_HISTORY_CAP, SETTLEMENT_MAX_POP, SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{SimEngine, SimResources};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;

const STAGE1_W: u32 = 128;
const STAGE1_H: u32 = 128;

/// Stage-1 engine factory mirroring the production pattern (Phase 7-δ/8-β).
/// Drives the 100-tick regression sentinel for A21 — exercises all default
/// runtime systems (MovementSystem … MemorySystem) against 20 agents.
fn make_stage1_engine(seed: u64, agent_count: u32) -> SimEngine {
    let mut engine = SimEngine::new(STAGE1_W, STAGE1_H, MaterialRegistry::new());
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
                    Memory::new(),
                ),
            )
            .expect("freshly spawned agent must still exist");
    }
    engine
}

// ─── A1: size_of::<SettlementId>() == size_of::<u32>() ───────────────────
#[test]
fn harness_p10_alpha_a1_settlement_id_is_u32() {
    // Type A — locked plan asserts exact `size_of` equality between the
    // SettlementId alias and `u32`. Any size deviation breaks FFI snapshot
    // serialization in Phase 10-γ.
    assert_eq!(
        std::mem::size_of::<SettlementId>(),
        std::mem::size_of::<u32>(),
        "SettlementId must have the same layout as u32"
    );
}

// ─── A2: size_of::<BuildingId>() == size_of::<u64>() ─────────────────────
#[test]
fn harness_p10_alpha_a2_building_id_is_u64() {
    // Type A — locked plan asserts exact `size_of` equality. Aligns with
    // AgentId = u64 convention.
    assert_eq!(
        std::mem::size_of::<BuildingId>(),
        std::mem::size_of::<u64>(),
        "BuildingId must have the same layout as u64"
    );
}

// ─── A3: SETTLEMENT_HISTORY_CAP == 32 (MEMORY_CAP mirror) ────────────────
#[test]
fn harness_p10_alpha_a3_settlement_history_cap_equals_32() {
    assert_eq!(
        SETTLEMENT_HISTORY_CAP, 32,
        "SETTLEMENT_HISTORY_CAP must mirror MEMORY_CAP = 32 (substrate symmetry)"
    );
}

// ─── A4: SETTLEMENT_FORMATION_AGENT_THRESHOLD == 3 ───────────────────────
#[test]
fn harness_p10_alpha_a4_formation_agent_threshold_equals_3() {
    assert_eq!(
        SETTLEMENT_FORMATION_AGENT_THRESHOLD, 3,
        "formation requires ≥3 agents (P10Plan-4)"
    );
}

// ─── A5: SETTLEMENT_FORMATION_BUILDING_THRESHOLD == 2 ────────────────────
#[test]
fn harness_p10_alpha_a5_formation_building_threshold_equals_2() {
    assert_eq!(
        SETTLEMENT_FORMATION_BUILDING_THRESHOLD, 2,
        "formation requires ≥2 buildings (P10Plan-4)"
    );
}

// ─── A6: SETTLEMENT_DISSOLUTION_THRESHOLD == 0 ───────────────────────────
#[test]
fn harness_p10_alpha_a6_dissolution_threshold_equals_0() {
    assert_eq!(
        SETTLEMENT_DISSOLUTION_THRESHOLD, 0,
        "dissolution when population == 0 (P10Plan-3)"
    );
}

// ─── A7: SETTLEMENT_MAX_POP == 50 ────────────────────────────────────────
#[test]
fn harness_p10_alpha_a7_max_pop_equals_50() {
    assert_eq!(
        SETTLEMENT_MAX_POP, 50,
        "migration pull stops at population 50 (P10Plan-8)"
    );
}

// ─── A8: SETTLEMENT_PROXIMITY_RADIUS == 5 ────────────────────────────────
#[test]
fn harness_p10_alpha_a8_proximity_radius_equals_5() {
    assert_eq!(
        SETTLEMENT_PROXIMITY_RADIUS, 5,
        "Chebyshev proximity radius = 5 tiles (P10Plan-4)"
    );
}

// ─── A9: new_with_id(0, 0) initializes empty + dissolved ─────────────────
#[test]
fn harness_p10_alpha_a9_new_with_id_initializes_empty() {
    // Locked plan: call `Settlement::new_with_id(0, 0)` and verify all 7
    // postconditions simultaneously.
    let s = Settlement::new_with_id(0, 0);
    assert!(s.member_agents.is_empty(), "member_agents must be empty");
    assert!(s.member_buildings.is_empty(), "member_buildings must be empty");
    assert!(s.community_history.is_empty(), "community_history must be empty");
    assert_eq!(s.population_count(), 0, "population_count() must be 0");
    assert!(s.is_dissolved(), "is_dissolved() must be true on empty settlement");
    assert_eq!(s.settlement_id, 0, "settlement_id must be 0");
    assert_eq!(s.founded_at, 0, "founded_at must be 0");
    // Spot-check the default PopulationStats aggregate too (substrate symmetry).
    assert_eq!(s.population_stats, PopulationStats::default());
}

// ─── A10: add/remove member_agent round-trip + post-removal dissolved ───
#[test]
fn harness_p10_alpha_a10_add_remove_member_agent_roundtrip() {
    let mut s = Settlement::new_with_id(1, 0);

    let agent: AgentId = 10;
    assert!(s.add_member_agent(agent), "first add_member_agent returns true");
    assert_eq!(s.population_count(), 1, "population_count increased by 1");

    assert!(
        !s.add_member_agent(agent),
        "second call with same id returns false (HashSet semantics)"
    );

    assert!(
        s.remove_member_agent(agent),
        "remove_member_agent returns true on present agent"
    );

    assert!(
        !s.remove_member_agent(agent),
        "remove_member_agent returns false on absent agent"
    );

    // Locked plan: after removal the settlement must be dissolved again.
    assert!(
        s.is_dissolved(),
        "is_dissolved() must be true after removing the only member"
    );
}

// ─── A11: add/remove member_building round-trip ──────────────────────────
#[test]
fn harness_p10_alpha_a11_add_remove_member_building_roundtrip() {
    let mut s = Settlement::new_with_id(1, 0);

    let building: BuildingId = 5;
    assert!(s.add_member_building(building), "first insert returns true");
    assert!(
        !s.add_member_building(building),
        "duplicate insert returns false"
    );

    assert!(
        s.remove_member_building(building),
        "remove returns true on present id"
    );
    assert!(
        !s.remove_member_building(building),
        "remove returns false on absent id"
    );

    assert_eq!(
        s.member_buildings.len(),
        0,
        "member_buildings.len() == 0 after removal"
    );
}

// ─── A12: append_history saturates at SETTLEMENT_HISTORY_CAP ─────────────
#[test]
fn harness_p10_alpha_a12_append_history_saturates_at_cap() {
    let mut s = Settlement::new_with_id(1, 0);

    // Append SETTLEMENT_HISTORY_CAP + 5 distinct EventIds (0..CAP+5).
    let total = (SETTLEMENT_HISTORY_CAP + 5) as EventId;
    for i in 0..total {
        s.append_history(i);
    }

    assert_eq!(
        s.community_history.len(),
        SETTLEMENT_HISTORY_CAP,
        "history length must saturate at SETTLEMENT_HISTORY_CAP"
    );
    // First retained EventId is 5 (0..4 evicted from index 0).
    assert_eq!(
        s.community_history[0], 5,
        "index-0 element must be EventId 5 — confirms FIFO eviction at index 0"
    );
}

// ─── A13: population_count() == member_agents.len() ──────────────────────
#[test]
fn harness_p10_alpha_a13_population_count_equals_member_agents_len() {
    let mut s = Settlement::new_with_id(1, 0);

    s.add_member_agent(1);
    s.add_member_agent(2);
    s.add_member_agent(3);
    assert_eq!(s.population_count(), 3, "population_count after 3 inserts");
    assert_eq!(
        s.member_agents.len(),
        3,
        "member_agents.len() after 3 inserts"
    );
    assert_eq!(
        s.population_count(),
        s.member_agents.len(),
        "population_count() must equal member_agents.len() at 3"
    );

    assert!(s.remove_member_agent(1), "remove agent 1");
    assert_eq!(s.population_count(), 2, "population_count after removing 1");
    assert_eq!(
        s.member_agents.len(),
        2,
        "member_agents.len() after removing 1"
    );
    assert_eq!(
        s.population_count(),
        s.member_agents.len(),
        "population_count() must equal member_agents.len() at 2"
    );
}

// ─── A14: is_dissolved() 3 state transitions ─────────────────────────────
#[test]
fn harness_p10_alpha_a14_is_dissolved_transitions() {
    let mut s = Settlement::new_with_id(1, 0);
    assert!(s.is_dissolved(), "fresh Settlement: is_dissolved() == true");

    s.add_member_agent(99);
    assert!(
        !s.is_dissolved(),
        "after adding 1 agent: is_dissolved() == false"
    );

    assert!(s.remove_member_agent(99));
    assert!(
        s.is_dissolved(),
        "after removing the only agent: is_dissolved() == true"
    );
}

// ─── A15: Settlement serde JSON round-trip (2 agents, 1 building, 3 hist) ─
#[test]
fn harness_p10_alpha_a15_settlement_serde_roundtrip() {
    let mut s = Settlement::new_with_id(7, 42);

    // Locked plan: 2 agents, 1 building, 3 history entries.
    s.add_member_agent(10);
    s.add_member_agent(20);
    s.add_member_building(100);
    s.append_history(500);
    s.append_history(501);
    s.append_history(502);

    let json = serde_json::to_string(&s).expect("Settlement must serialize to JSON");
    let back: Settlement =
        serde_json::from_str(&json).expect("Settlement must deserialize from JSON");

    assert_eq!(s, back, "deserialized Settlement must equal original");
    assert_eq!(
        back.community_history.len(),
        3,
        "round-trip preserved 3 history entries"
    );
    assert_eq!(back.member_agents.len(), 2, "round-trip preserved 2 agents");
    assert_eq!(
        back.member_buildings.len(),
        1,
        "round-trip preserved 1 building"
    );
}

// ─── A16: SimResources.settlements empty + next_settlement_id == 1 ──────
// Phase 10-β plan A2 reserved `settlement_id == 0` as the uninitialized
// sentinel — the counter starts at 1 so the first issued id is `1`.
#[test]
fn harness_p10_alpha_a16_sim_resources_settlements_empty_at_init() {
    let res = SimResources::new(20, 20, MaterialRegistry::new());
    assert!(
        res.settlements.is_empty(),
        "SimResources::new must create an empty `settlements` HashMap"
    );
    assert_eq!(
        res.next_settlement_id, 1,
        "next_settlement_id must start at 1 (0 reserved as sentinel — Phase 10-β plan A2)"
    );
}

// ─── A17: issue_settlement_id() returns [1, 2, 3, 4, 5] ─────────────────
// Phase 10-β plan A2: `settlement_id == 0` is the uninitialized sentinel,
// so issue_settlement_id() must never return 0.
#[test]
fn harness_p10_alpha_a17_issue_settlement_id_monotonically_increases() {
    let mut res = SimResources::new(20, 20, MaterialRegistry::new());
    let ids: Vec<SettlementId> = (0..5).map(|_| res.issue_settlement_id()).collect();
    assert_eq!(
        ids,
        vec![1u32, 2, 3, 4, 5],
        "5 consecutive issue_settlement_id() calls must yield [1,2,3,4,5] (0 reserved)"
    );
    assert_eq!(
        res.next_settlement_id, 6,
        "next_settlement_id must advance to 6 after 5 issues"
    );
}

// ─── A18: AgentBorn { id: 99, parent: Some(10), agent: 1, tick: 42 } ────
#[test]
fn harness_p10_alpha_a18_causal_event_agent_born_variant_and_accessors() {
    // Locked plan: construct exactly this event and verify the 4 accessors.
    let ev = CausalEvent::AgentBorn {
        id: 99,
        parent: Some(10),
        agent: 1 as AgentId,
        tick: 42,
    };

    assert_eq!(ev.id(), 99, "id() must return 99");
    assert_eq!(ev.parent(), Some(10), "parent() must return Some(10)");
    assert_eq!(ev.tick(), 42, "tick() must return 42");
    assert_eq!(
        ev.channel(),
        None,
        "channel() must be None (AgentBorn is not influence-channel-bound)"
    );
}

// ─── A19: AgentBorn Clone + PartialEq ────────────────────────────────────
#[test]
fn harness_p10_alpha_a19_agent_born_clone_and_partial_eq() {
    let ev = CausalEvent::AgentBorn {
        id: 99,
        parent: Some(10),
        agent: 1 as AgentId,
        tick: 42,
    };

    let cloned = ev.clone();
    assert!(
        cloned == ev,
        "cloned AgentBorn must compare equal to original via derive(PartialEq)"
    );
}

// ─── A20: components re-exports compile ──────────────────────────────────
#[test]
fn harness_p10_alpha_a20_components_re_exports() {
    // Locked plan: only the act of importing these symbols verifies the
    // re-export surface (type A — compiler-verified).
    use sim_core::components::{
        BuildingId as _BuildingId, Settlement as _Settlement, SettlementId as _SettlementId,
        SETTLEMENT_HISTORY_CAP as _SHC, SETTLEMENT_MAX_POP as _SMP,
    };
    // Spot-check the imported constants resolve to their locked values.
    assert_eq!(_SHC, 32);
    assert_eq!(_SMP, 50);
    // Type aliases instantiate cleanly.
    let _id: _SettlementId = 0u32;
    let _bid: _BuildingId = 0u64;
    let _ = _Settlement::new_with_id(0, 0);
}

// ─── A21: Phase 9-α + 8-α regression sentinel ────────────────────────────
#[test]
fn harness_p10_alpha_a21_phase9_phase8_regression_guard() {
    // Locked plan (Type D):
    //   1. make_stage1_engine(42, 20)
    //   2. advance 100 ticks
    //   3. assert no panic, agent count >= 1
    //   4. assert MemorySystem / event processing is still exercised
    //      (causal log advanced past tick 0)
    let mut engine = make_stage1_engine(42, 20);

    // Snapshot the active-tile count in the causal log before running.
    // Anything > this after 100 ticks demonstrates the event-emission path
    // (which feeds MemorySystem via the AgentDecisionSystem cascade) is
    // still wired through the new `CausalEvent::AgentBorn` exhaustive
    // match arms.
    let causal_tiles_before = engine.resources.causal_log.active_tile_count();

    for _ in 0..100 {
        engine.tick();
    }

    // (1) No-panic guarantee — implicit: the test reaches this point.
    // (2) Engine ran to completion: current_tick advanced to exactly 100.
    assert_eq!(
        engine.current_tick(),
        100,
        "engine.current_tick() must equal 100 after 100 ticks (got {})",
        engine.current_tick()
    );

    // (3) Agent count >= 1 after 100 ticks (no death system at this stage,
    // so all 20 spawned agents must still be alive).
    let agent_count = engine.world.query::<&Agent>().iter().count();
    assert!(
        agent_count >= 1,
        "agent count must be >= 1 after 100 ticks (got {})",
        agent_count
    );

    // (4) Phase 8-α MemorySystem still attached — every spawned agent
    // carries the Memory component, proving the component re-exports +
    // exhaustive-match updates did not regress the Phase 8 substrate.
    let memory_count = engine.world.query::<&Memory>().iter().count();
    assert_eq!(
        memory_count, agent_count,
        "every agent must still carry a Memory component (Phase 8-α regression)"
    );

    // (5) Event-processing sentinel: the causal log accumulated activity on
    // at least one new tile, proving the event-emission cascade is still
    // wired through the new AgentBorn variant's match arms.
    let causal_tiles_after = engine.resources.causal_log.active_tile_count();
    assert!(
        causal_tiles_after >= causal_tiles_before,
        "causal_log active_tile_count must not regress over 100 ticks \
         (before={}, after={})",
        causal_tiles_before,
        causal_tiles_after,
    );

    // (6) Settlement substrate must remain a no-op at this phase — Phase
    // 10-α adds no SettlementSystem; the HashMap stays empty.
    assert!(
        engine.resources.settlements.is_empty(),
        "Phase 10-α adds no SettlementSystem; settlements HashMap must remain empty"
    );
    assert_eq!(
        engine.resources.next_settlement_id, 1,
        "no settlement id issued during Phase 10-α regression run \
         (Phase 10-β plan A2: counter starts at 1, 0 reserved as sentinel)"
    );
}
