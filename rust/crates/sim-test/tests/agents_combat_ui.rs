//! V7 Phase 9-δ — Combat UI Integration harness.
//!
//! plan: p9-delta-combat-ui (plan_attempt 1, seed 42, agent_count 20)
//! lane: --full
//!
//! Asserts the 9 invariants locked by plan_attempt 1:
//!   A1 — locale_compiled_en_five_keys_present
//!   A2 — locale_compiled_ko_five_keys_present
//!   A3 — locale_five_keys_pairwise_distinct_en
//!   A4 — combat_started_ffi_kind_string
//!   A5 — combat_completed_ffi_kind_string
//!   A6 — combat_ffi_defender_id_exposed
//!   A7 — combat_reason_decision_as_str
//!   A8 — combat_panel_no_hardcoded_english
//!   A9 — phase8_delta_recall_cue_regression

use sim_bridge::ffi::{event_view_to_owned_dict, CausalEventView, FfiFieldValue};
use sim_core::causal::{CausalEvent, DecisionReason};
use sim_core::components::AgentId;

// ─── helpers ────────────────────────────────────────────────────────────

// ─── A4 — combat_started_ffi_kind_string ────────────────────────────────
//
// Type A invariant. CombatStarted FFI dict["kind"] must equal byte-for-
// byte the literal "combat_started" — the discriminator GDScript's
// CausalPanel match arm depends on.
#[test]
fn harness_p9_delta_combat_started_ffi_kind_string() {
    let ev = CausalEvent::CombatStarted {
        id: 1,
        parent: None,
        attacker: 5,
        defender: 17,
        position: (4, 5),
        tick: 42,
    };
    let view = CausalEventView::from_event(&ev);
    let dict = event_view_to_owned_dict(&view);
    match dict.get("kind") {
        Some(FfiFieldValue::Str(k)) => assert_eq!(
            *k, "combat_started",
            "A4: dict[\"kind\"] must equal \"combat_started\", got {k:?}"
        ),
        other => panic!("A4: dict[\"kind\"] must be Str, got {other:?}"),
    }
}

// ─── A5 — combat_completed_ffi_kind_string ──────────────────────────────
//
// Type A invariant. Same as A4 but for CombatCompleted —
// dict["kind"] must equal "combat_completed".
#[test]
fn harness_p9_delta_combat_completed_ffi_kind_string() {
    let ev = CausalEvent::CombatCompleted {
        id: 2,
        parent: Some(1),
        attacker: 5,
        defender: 23,
        position: (4, 5),
        hp_after: 42.0,
        settlement_link: None,
        tick: 50,
    };
    let view = CausalEventView::from_event(&ev);
    let dict = event_view_to_owned_dict(&view);
    match dict.get("kind") {
        Some(FfiFieldValue::Str(k)) => assert_eq!(
            *k, "combat_completed",
            "A5: dict[\"kind\"] must equal \"combat_completed\", got {k:?}"
        ),
        other => panic!("A5: dict[\"kind\"] must be Str, got {other:?}"),
    }
}

// ─── A6 — combat_ffi_defender_id_exposed ────────────────────────────────
//
// Type A invariant. CombatStarted must surface `defender_id` in the FFI
// dict with the supplied numeric id (Section 2-A extends serialization).
#[test]
fn harness_p9_delta_combat_ffi_defender_id_exposed() {
    const K: AgentId = 17;
    let ev = CausalEvent::CombatStarted {
        id: 1,
        parent: None,
        attacker: 5,
        defender: K,
        position: (3, 4),
        tick: 10,
    };
    let view = CausalEventView::from_event(&ev);
    let dict = event_view_to_owned_dict(&view);
    let v = dict.get("defender_id").unwrap_or_else(|| {
        panic!(
            "A6: dict missing \"defender_id\"; keys present: {:?}",
            dict.keys().collect::<Vec<_>>()
        )
    });
    match v {
        FfiFieldValue::I64(n) => assert_eq!(
            *n, K as i64,
            "A6: dict[\"defender_id\"] must equal {K}, got {n}"
        ),
        other => panic!("A6: dict[\"defender_id\"] must be I64, got {other:?}"),
    }
}

// ─── A7 — combat_reason_decision_as_str ─────────────────────────────────
//
// Type A invariant. DecisionReason::CombatReason.as_str() must equal the
// byte-exact wire literal "combat_reason".
#[test]
fn harness_p9_delta_combat_reason_decision_as_str() {
    assert_eq!(
        DecisionReason::CombatReason.as_str(),
        "combat_reason",
        "A7: CombatReason discriminator must be \"combat_reason\""
    );
}
