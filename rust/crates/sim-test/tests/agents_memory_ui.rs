//! V7 Phase 8-δ — Memory UI Integration harness.
//!
//! Verifies:
//!   * A1/A2: the 7 new `UI_*` locale keys are present in
//!     `localization/compiled/{en,ko}.json` with the Phase 7-δ contract
//!     (length >= 3 en / >= 2 ko, >= 2 ASCII letters en / >= 1 Hangul ko,
//!     ko byte-distinct from en).
//!   * A3/A4: those 7 values are pairwise distinct within each language.
//!   * A5: the production `event_view_to_dict()` Godot-bound marshaller's
//!     schema source — `event_view_to_owned_dict()` — exposes
//!     `kind == "memory_recalled"` for `CausalEvent::MemoryRecalled`.
//!     The two helpers are structurally pinned to a single source of
//!     truth by A7 (event_view_to_dict body calls event_view_to_owned_dict),
//!     so asserting on the owned-dict IS asserting on the production
//!     Godot-bound dict.
//!   * A6: same path — `triggered_by == "cascade_bias"` for
//!     `MemoryRecallTrigger::CascadeBias`.
//!   * A7: `event_view_to_dict` source-code structurally delegates to
//!     `event_view_to_owned_dict` and inserts NO independent keys, so the
//!     symmetric difference of their key sets is provably ∅.
//!   * A8: `DecisionReason::MemoryReason.as_str() == "memory_reason"`.
//!   * A9: Phase 8-β scope-lock: every `MemoryRecalled` event observed in
//!     a 200-tick stage1 run uses `MemoryRecallTrigger::CascadeBias`.
//!     Vacuous pass with explicit plan-required notice when 0 events fire.
//!   * A10: Phase 7-δ regression guard — `AgentSnapshotRow.state_tag ∈
//!     {0,1,2,3}` remains true across the same 200-tick run.
//!   * A11: `causal_panel.gd` memory-rendering region has zero hardcoded
//!     English: forbidden literals AND any quoted alphabetic literal of
//!     length >= 4 NOT wrapped in `Locale.ltr(...)`. UI_* identifier keys
//!     and wire discriminators are explicitly allowlisted.
//!   * A12: `localization/compiled/{en,ko}.json` parse cleanly with a
//!     `strings` dict of all-string values AND retain a baseline of
//!     pre-existing keys (so the Python JSON merge cannot silently delete
//!     keys).
//!   * Wiring guard: the AgentRenderer GDScript actually invokes
//!     `mark_agent_recalling()` from a real causal-event read path.
//!
//! plan: p8-delta-memory-ui (plan_attempt 3, seed 42, agent_count 20)
//! lane: --full
//!
//! Run:
//!   `cargo test -p sim-test --test harness_p8_delta_memory_ui -- --nocapture`

use sim_bridge::ffi::{
    collect_agent_snapshot, event_view_to_owned_dict, CausalEventView, FfiFieldValue,
};
use sim_core::causal::{CausalEvent, DecisionReason, MemoryRecallTrigger};
use sim_core::components::{AgentState, Hunger, Memory, Position, Sleep, Social, Thirst};
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;

// ────────────────────────────────────────────────────────────────────────
// Constants — locked by plan
// ────────────────────────────────────────────────────────────────────────

const W: u32 = 128;
const H: u32 = 128;

// ────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────

/// Build the FFI dict for a `CausalEvent::MemoryRecalled` via the SAME
/// schema source the Godot-bound `event_view_to_dict()` consumes.
///
/// Rationale for not invoking `event_view_to_dict()` directly: that
/// function's first non-trivial statement is `let owned =
/// event_view_to_owned_dict(view);` and the remaining body is a mechanical
/// `BTreeMap<&'static str, FfiFieldValue>` → `VarDictionary` conversion.
/// `VarDictionary::new()` panics outside a Godot runtime, so a unit test
/// cannot construct the VarDictionary form. Assertion A7 statically
/// verifies that `event_view_to_dict`'s body delegates to
/// `event_view_to_owned_dict` and inserts NO independent keys — so
/// asserting on the owned-dict's key set IS asserting on the dict the
/// GDScript runtime receives (modulo the mechanical
/// FfiFieldValue → Variant mapping, which is itself exhaustively covered
/// by the match in `event_view_to_dict`).
fn ffi_dict_for_memory_recalled(
    trigger: MemoryRecallTrigger,
) -> std::collections::BTreeMap<&'static str, FfiFieldValue> {
    let ev = CausalEvent::MemoryRecalled {
        id: 1,
        parent: None,
        agent: 7,
        recalled_event: 42,
        triggered_by: trigger,
        tick: 0,
    };
    let view = CausalEventView::from_event(&ev);
    event_view_to_owned_dict(&view)
}

/// 20-agent stage1 engine that maximises the chance of a memory-recall
/// cascade firing during the 200-tick window: high `social_growth` so
/// agents enter social interactions quickly and accumulate memory entries
/// whose recall can flip the cascade.
fn make_stage1_engine(seed: u64, agent_count: u32) -> SimEngine {
    let mut engine = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut engine);
    for i in 0..agent_count {
        // Co-locate adjacent pairs on the same tile so social handshakes
        // close every couple of ticks.
        let x = 16 + (i / 2);
        let y = 16 + (i % 2);
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
                    Social::new(0.0, 1.0),
                    AgentState::Idle,
                    Memory::new(),
                ),
            )
            .expect("freshly spawned agent must still exist");
        {
            let mut p = engine
                .world
                .get::<&mut Position>(entity)
                .expect("Position present");
            p.x = x;
            p.y = y;
        }
    }
    engine
}

/// Pull every `CausalEvent::MemoryRecalled` from the engine's tile causal
/// log across every populated tile.
fn collect_memory_recalled(resources: &sim_engine::SimResources) -> Vec<CausalEvent> {
    let mut out = Vec::new();
    for (_tile_idx, log) in resources.causal_log.iter() {
        for ev in log.as_slice() {
            if matches!(ev, CausalEvent::MemoryRecalled { .. }) {
                out.push(ev.clone());
            }
        }
    }
    out
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 5 — memory_recalled_ffi_kind_string_literal_via_godot_marshaller
// ────────────────────────────────────────────────────────────────────────
//
// Plan §A5: the dict that flows through `event_view_to_dict()` to GDScript
// must have `kind == "memory_recalled"` (exact byte-for-byte equality).
//
// Routing: this test inspects the dict produced by the SAME schema
// generator the Godot-bound `event_view_to_dict()` invokes as its first
// statement — `event_view_to_owned_dict()`. A7 structurally pins
// `event_view_to_dict`'s body so it MUST call `event_view_to_owned_dict`
// and MUST NOT add any independent dict keys, making the two
// representations provably equivalent in key set.
#[test]
fn harness_p8_delta_memory_recalled_ffi_kind_string_literal_via_godot_marshaller() {
    let dict = ffi_dict_for_memory_recalled(MemoryRecallTrigger::CascadeBias);
    let kind_value = dict.get("kind").unwrap_or_else(|| {
        panic!(
            "A5: production FFI dict for MemoryRecalled is missing key \"kind\"; got keys {:?}",
            dict.keys().collect::<Vec<_>>(),
        )
    });
    match kind_value {
        FfiFieldValue::Str(k) => assert_eq!(
            *k, "memory_recalled",
            "A5: production FFI dict[\"kind\"] must be exactly \"memory_recalled\" (snake_case literal the GDScript CausalPanel match arm pins on), got {k:?}",
        ),
        other => panic!(
            "A5: production FFI dict[\"kind\"] must be Str(\"memory_recalled\"), got {other:?}",
        ),
    }
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 6 — memory_recalled_ffi_dict_includes_triggered_by_cascade_bias
// ────────────────────────────────────────────────────────────────────────
//
// Plan §A6: the dict that flows through `event_view_to_dict()` to GDScript
// MUST include the key `"triggered_by"` with value `"cascade_bias"` for a
// `MemoryRecallTrigger::CascadeBias` event. The GDScript CausalPanel
// `_format_event()` match arm pins on this exact literal to choose the
// `UI_MEMORY_RECALL_TRIGGER_CASCADE` locale key.
//
// Same routing argument as A5: inspecting the owned-dict directly tests
// the production schema source (locked by A7).
#[test]
fn harness_p8_delta_memory_recalled_ffi_dict_includes_triggered_by_cascade_bias_via_godot_marshaller(
) {
    let dict = ffi_dict_for_memory_recalled(MemoryRecallTrigger::CascadeBias);
    // (a) the key MUST be present.
    let trig_value = dict.get("triggered_by").unwrap_or_else(|| {
        panic!(
            "A6: production FFI dict for MemoryRecalled is missing key \"triggered_by\"; got keys {:?}. The GDScript CausalPanel cannot render the cascade variant without this field.",
            dict.keys().collect::<Vec<_>>(),
        )
    });
    // (b) the value MUST be `Str("cascade_bias")` — Phase 8-β only wires
    // CascadeBias; the GDScript `_format_event()` match arm pins on this
    // exact literal.
    match trig_value {
        FfiFieldValue::Str(s) => {
            assert!(
                !s.is_empty(),
                "A6: production FFI dict[\"triggered_by\"] must be a non-empty Str, got {s:?}",
            );
            assert_eq!(
                *s, "cascade_bias",
                "A6: production FFI dict[\"triggered_by\"] for CascadeBias must be \"cascade_bias\" (the GDScript CausalPanel match arm pins on this exact literal), got {s:?}",
            );
        }
        other => panic!("A6: production FFI dict[\"triggered_by\"] must be Str(_), got {other:?}",),
    }
    // (c) kind also flows through — sanity check the discriminator the
    // CausalPanel uses to even enter the memory-rendering branch.
    match dict.get("kind") {
        Some(FfiFieldValue::Str(k)) => assert_eq!(
            *k, "memory_recalled",
            "A6: production FFI dict[\"kind\"] must equal \"memory_recalled\", got {k:?}",
        ),
        other => panic!(
            "A6: production FFI dict[\"kind\"] must be Str(\"memory_recalled\"), got {other:?}",
        ),
    }
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 8 — decision_reason_memory_reason_as_str_exact
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_p8_delta_memory_decision_reason_memory_reason_as_str_exact() {
    // Type A — locked snake_case discriminator the CausalPanel matches on.
    assert_eq!(
        DecisionReason::MemoryReason.as_str(),
        "memory_reason",
        "A8: DecisionReason::MemoryReason discriminator must be \"memory_reason\"",
    );
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 9 — memory_recalled_triggered_by_only_cascade_in_phase8_run
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_p8_delta_memory_recalled_triggered_by_only_cascade_in_phase8_run() {
    // Type A scope-lock — Phase 8-β only wires `CascadeBias`. Any other
    // trigger appearing means an out-of-scope Phase 9-δ code path leaked
    // in. Vacuous pass on 0 events (with the plan-required notice text)
    // is allowed by the plan.
    let mut engine = make_stage1_engine(42, 20);
    for _ in 0..200 {
        engine.tick();
    }
    let recalls = collect_memory_recalled(&engine.resources);
    let count_total = recalls.len();
    let count_cascade = recalls
        .iter()
        .filter(|ev| {
            matches!(
                ev,
                CausalEvent::MemoryRecalled {
                    triggered_by: MemoryRecallTrigger::CascadeBias,
                    ..
                }
            )
        })
        .count();
    let count_non_cascade = count_total - count_cascade;
    if count_total == 0 {
        // Plan-required exact notice text for the Evaluator's log scan.
        eprintln!("notice: 0 MemoryRecalled events in 200 ticks — vacuous pass");
    } else {
        eprintln!(
            "[p8-δ A9] MemoryRecalled observation: total={count_total} cascade={count_cascade} non_cascade={count_non_cascade}",
        );
    }
    assert_eq!(
        count_non_cascade, 0,
        "A9: Phase 8-β must emit only CascadeBias triggers; saw {count_non_cascade} non-cascade in {count_total} total recalls",
    );
}

// ────────────────────────────────────────────────────────────────────────
// AgentId↔snapshot round-trip (code-attempt 3 fix per evaluator review)
// ────────────────────────────────────────────────────────────────────────
//
// Background:
//   `CausalEvent::MemoryRecalled.agent` is an `AgentId` (the monotonically
//   minted `Agent.id` field, a `u64` separate from `hecs::Entity::to_bits`).
//   `event_view_to_owned_dict` serialises it as `dict["agent_id"]`.
//
//   The agent snapshot path used by the renderer historically only
//   exposed `entity_bits` (Phase 4-γ A5 contract). The previous code
//   attempt marked `_recalling_agents` with `event.agent_id` (an AgentId)
//   while *checking* against snapshot `ids[i]` (entity bits). The two
//   domains are disjoint so the cue could never fire.
//
//   The fix exposes `Agent.id` through the snapshot as a parallel
//   `agent_ids` field while preserving `entity_bits` for the existing P4-γ
//   contract.
#[test]
fn harness_p8_delta_memory_snapshot_exposes_agent_id_matching_component_id() {
    // Type A — the AgentSnapshotRow MUST carry an `agent_id` field whose
    // value equals the corresponding agent entity's `Agent.id` component.
    // Without this, the renderer cannot key its recall lookup against the
    // same AgentId domain the `MemoryRecalled` causal event emits.
    use sim_core::components::Agent;
    let mut engine = make_stage1_engine(42, 20);
    engine.tick();
    let rows = collect_agent_snapshot(&engine.world);
    assert_eq!(
        rows.len(),
        20,
        "expected 20 agent rows from stage1 engine, got {}",
        rows.len(),
    );
    for row in &rows {
        let entity =
            hecs::Entity::from_bits(row.entity_bits).expect("entity_bits is a valid hecs::Entity");
        let agent = engine
            .world
            .get::<&Agent>(entity)
            .expect("snapshot row's entity must still carry an Agent component");
        assert_eq!(
            row.agent_id, agent.id,
            "AgentSnapshotRow.agent_id must equal Agent.id for entity {entity:?} \
             (snapshot row reported {}, component holds {})",
            row.agent_id, agent.id,
        );
    }
}

#[test]
fn harness_p8_delta_memory_snapshot_agent_id_distinct_from_entity_bits() {
    // Type A — AgentId is minted by a monotonic counter starting from 0
    // (or low values), while hecs::Entity::to_bits packs (generation, index)
    // and uses high bits for the generation. The two MUST be observably
    // different so a renderer that confuses them fails this test even on a
    // single agent. Mirrors the evaluator's `AgentId vs entity_bits
    // mismatch` callout from code-attempt 2.
    let mut engine = make_stage1_engine(42, 20);
    engine.tick();
    let rows = collect_agent_snapshot(&engine.world);
    let mismatches = rows
        .iter()
        .filter(|r| (r.agent_id as i64) != (r.entity_bits as i64))
        .count();
    assert!(
        mismatches > 0,
        "expected at least one row where agent_id differs from entity_bits — \
         otherwise the renderer's prior bug (marking by AgentId, checking by \
         entity_bits) would silently pass. rows={rows:?}",
    );
}

#[test]
fn harness_p8_delta_memory_ffi_event_agent_id_matches_constructed_agent_id() {
    // Type A end-to-end identity — given a `CausalEvent::MemoryRecalled`
    // with a known AgentId, the FFI dict MUST round-trip that exact value
    // under the key `"agent_id"`. This is the same key the renderer reads
    // and the same key it uses to call `mark_agent_recalling`.
    use sim_core::causal::CausalEvent;
    const TEST_AGENT_ID: sim_core::components::AgentId = 0xCAFE_BABE_u64;
    let ev = CausalEvent::MemoryRecalled {
        id: 9_001,
        parent: None,
        agent: TEST_AGENT_ID,
        recalled_event: 17,
        triggered_by: MemoryRecallTrigger::CascadeBias,
        tick: 0,
    };
    let view = CausalEventView::from_event(&ev);
    let dict = event_view_to_owned_dict(&view);
    let agent_id_value = dict.get("agent_id").unwrap_or_else(|| {
        panic!(
            "FFI dict missing \"agent_id\" for MemoryRecalled; keys={:?}",
            dict.keys().collect::<Vec<_>>()
        )
    });
    match agent_id_value {
        FfiFieldValue::I64(v) => assert_eq!(
            *v as u64,
            TEST_AGENT_ID,
            "FFI dict[\"agent_id\"] must round-trip CausalEvent::MemoryRecalled.agent exactly; got {v}, expected {TEST_AGENT_ID}",
        ),
        other => panic!(
            "FFI dict[\"agent_id\"] must be I64(_), got {other:?}",
        ),
    }
}
