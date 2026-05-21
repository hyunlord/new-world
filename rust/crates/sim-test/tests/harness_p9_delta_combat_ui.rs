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

use std::fs;

use sim_bridge::ffi::world_node::{event_view_to_owned_dict, CausalEventView, FfiFieldValue};
use sim_core::causal::{CausalEvent, DecisionReason};
use sim_core::components::AgentId;

const LOCALE_KEYS: [&str; 5] = [
    "UI_CAUSAL_REASON_COMBAT",
    "UI_CAUSAL_EVENT_COMBAT_STARTED",
    "UI_CAUSAL_EVENT_COMBAT_COMPLETED",
    "UI_AGENT_STATE_IN_COMBAT",
    "UI_COMBAT_HP_AFTER",
];

// ─── helpers ────────────────────────────────────────────────────────────

fn project_root() -> std::path::PathBuf {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .expect("project root above sim-test crate")
}

fn read_locale_json(locale: &str) -> serde_json::Value {
    let path = project_root()
        .join("localization")
        .join("compiled")
        .join(format!("{locale}.json"));
    let raw =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("read locale file {path:?}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse locale file {path:?}: {e}"))
}

fn strings_lookup<'a>(json: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    json.get("strings")
        .and_then(|s| s.get(key))
        .and_then(|v| v.as_str())
}

fn read_panel_src() -> String {
    let path = project_root()
        .join("scripts")
        .join("ui")
        .join("panels")
        .join("causal_panel.gd");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

fn read_agent_renderer_src() -> String {
    let path = project_root()
        .join("scripts")
        .join("ui")
        .join("agent_renderer.gd");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

// ─── A1 — locale_compiled_en_five_keys_present ──────────────────────────
//
// Type A invariant. Each of the 5 keys must exist under `strings` in
// `localization/compiled/en.json`, value char-length >= 2, with >= 2 ASCII
// alphabetic chars in [A-Za-z].
#[test]
fn harness_p9_delta_combat_locale_en_five_keys() {
    let json = read_locale_json("en");
    for key in LOCALE_KEYS {
        let value = strings_lookup(&json, key)
            .unwrap_or_else(|| panic!("A1: en.json missing key {key}"));
        let len = value.chars().count();
        assert!(
            len >= 2,
            "A1: en[{key}] length must be >= 2, got {len} ({value:?})"
        );
        let alpha = value.chars().filter(|c| c.is_ascii_alphabetic()).count();
        assert!(
            alpha >= 2,
            "A1: en[{key}] must contain >= 2 ASCII letters, got {alpha} ({value:?})"
        );
    }
}

// ─── A2 — locale_compiled_ko_five_keys_present ──────────────────────────
//
// Type A invariant. Each of the 5 keys must exist under `strings` in
// `localization/compiled/ko.json`, value char-length >= 2, with >= 1
// Hangul syllable (U+AC00..=U+D7A3), and the ko value must differ
// byte-wise from the corresponding en value.
#[test]
fn harness_p9_delta_combat_locale_ko_five_keys() {
    let en = read_locale_json("en");
    let ko = read_locale_json("ko");
    for key in LOCALE_KEYS {
        let en_val = strings_lookup(&en, key)
            .unwrap_or_else(|| panic!("A2: en.json missing key {key}"));
        let ko_val = strings_lookup(&ko, key)
            .unwrap_or_else(|| panic!("A2: ko.json missing key {key}"));
        let len = ko_val.chars().count();
        assert!(
            len >= 2,
            "A2: ko[{key}] length must be >= 2, got {len} ({ko_val:?})"
        );
        let hangul = ko_val
            .chars()
            .filter(|c| ('\u{AC00}'..='\u{D7A3}').contains(c))
            .count();
        assert!(
            hangul >= 1,
            "A2: ko[{key}] must contain >= 1 Hangul syllable, got {hangul} ({ko_val:?})"
        );
        assert_ne!(
            en_val.as_bytes(),
            ko_val.as_bytes(),
            "A2: ko[{key}] must differ byte-wise from en[{key}]"
        );
    }
}

// ─── A3 — locale_five_keys_pairwise_distinct_en ─────────────────────────
//
// Type A invariant. The 5 en values must be pairwise distinct — zero
// collisions across the C(5,2) = 10 unordered pairs.
#[test]
fn harness_p9_delta_combat_locale_pairwise_distinct_en() {
    let en = read_locale_json("en");
    let vals: Vec<&str> = LOCALE_KEYS
        .iter()
        .map(|k| strings_lookup(&en, k).unwrap_or_else(|| panic!("A3: en.json missing {k}")))
        .collect();
    let mut collisions = 0usize;
    for i in 0..vals.len() {
        for j in (i + 1)..vals.len() {
            if vals[i] == vals[j] {
                collisions += 1;
            }
        }
    }
    assert_eq!(
        collisions, 0,
        "A3: en values must be pairwise distinct, got {collisions} colliding pair(s) in {vals:?}"
    );
}

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

// ─── A8 — combat_panel_no_hardcoded_english ─────────────────────────────
//
// Type A invariant. Within the combat match-arm bodies of causal_panel.gd
// (`"combat_started"`, `"combat_completed"`, `"combat_reason"`), every
// double- or single-quoted string literal consisting only of [A-Za-z]
// characters with length >= 4 must be on the allowlist:
//   * any UI_-prefixed identifier
//   * snake_case wire literals: combat_started, combat_completed,
//     combat_reason, new_value, defender_id, agent_id, kind, reason, tick
// Underscored / non-alphabetic / short literals are filtered out by the
// predicate before the allowlist check. violation_count must be 0.
#[test]
fn harness_p9_delta_combat_panel_no_hardcoded_english() {
    let src = read_panel_src();

    // Allowlist set (used only when a literal slips through the pure-
    // alpha predicate, which it cannot under the plan — kept defensively).
    let allowlist = [
        "combat_started",
        "combat_completed",
        "combat_reason",
        "new_value",
        "defender_id",
        "agent_id",
        "kind",
        "reason",
        "tick",
    ];

    // Locate each wire arm by exact substring; include lines from the
    // occurrence until the next match-arm boundary OR method end. A
    // "next match-arm boundary" is a trimmed line beginning with `"` —
    // mirroring the GDScript indentation idiom for match arms. Method
    // end is the next line starting with `func `.
    let wires = ["combat_started", "combat_completed", "combat_reason"];
    let mut violations: Vec<String> = Vec::new();

    for wire in wires {
        let quoted = format!("\"{wire}\"");
        let mut search_from = 0usize;
        let mut arms = 0usize;
        while let Some(off) = src[search_from..].find(&quoted) {
            let arm_start = search_from + off;
            arms += 1;

            // Walk forward line-by-line collecting body lines until the
            // next arm boundary or method end.
            let tail = &src[arm_start..];
            let mut body = String::new();
            let mut saw_first_line = false;
            for line in tail.lines() {
                if !saw_first_line {
                    // Include the line containing the wire literal.
                    body.push_str(line);
                    body.push('\n');
                    saw_first_line = true;
                    continue;
                }
                let trim = line.trim_start();
                if trim.starts_with("func ") {
                    break;
                }
                if trim.starts_with('"') {
                    // Next match arm — boundary reached.
                    break;
                }
                body.push_str(line);
                body.push('\n');
            }

            // Scan body for pure-alpha quoted literals of length >= 4.
            for raw_line in body.lines() {
                // Strip GDScript line comment (heuristic: `#` outside
                // of a "…" string starts a comment).
                let mut in_str = false;
                let mut comment_at: Option<usize> = None;
                for (i, ch) in raw_line.char_indices() {
                    match ch {
                        '"' => in_str = !in_str,
                        '#' if !in_str => {
                            comment_at = Some(i);
                            break;
                        }
                        _ => {}
                    }
                }
                let code = match comment_at {
                    Some(i) => &raw_line[..i],
                    None => raw_line,
                };

                // Extract every "…" and '…' literal on the code portion.
                for quote in ['"', '\''] {
                    let bytes = code.as_bytes();
                    let mut i = 0;
                    while i < bytes.len() {
                        if bytes[i] == quote as u8 {
                            let start = i + 1;
                            let mut j = start;
                            while j < bytes.len() && bytes[j] != quote as u8 {
                                j += 1;
                            }
                            if j >= bytes.len() {
                                break;
                            }
                            let literal = &code[start..j];
                            i = j + 1;

                            // Plan predicate: only [A-Za-z], length >= 4.
                            if literal.len() < 4 {
                                continue;
                            }
                            if !literal.chars().all(|c| c.is_ascii_alphabetic()) {
                                continue;
                            }
                            // Allowlist: UI_-prefixed OR allowlisted wire.
                            if literal.starts_with("UI_") {
                                continue;
                            }
                            if allowlist.contains(&literal) {
                                continue;
                            }
                            violations.push(format!(
                                "arm \"{wire}\": literal {literal:?}"
                            ));
                        } else {
                            i += 1;
                        }
                    }
                }
            }

            search_from = arm_start + quoted.len();
            // Defensive cap on per-wire arm count.
            if arms > 8 {
                break;
            }
        }
        assert!(
            arms >= 1,
            "A8: expected at least one occurrence of arm key \"{wire}\" in causal_panel.gd"
        );
    }

    assert_eq!(
        violations.len(),
        0,
        "A8: hardcoded English in combat arms — {} violation(s):\n  {}",
        violations.len(),
        violations.join("\n  ")
    );
}

// ─── A9 — phase8_delta_recall_cue_regression ────────────────────────────
//
// Type D regression guard. agent_renderer.gd must still contain all four
// Phase 8-δ recall-cue tokens as substrings. If a Generator removes or
// renames any of them while mirroring the recall pattern for combat,
// this assertion fails.
#[test]
fn harness_p9_delta_combat_phase8_recall_regression() {
    let src = read_agent_renderer_src();
    let tokens = [
        "_recalling_agents",
        "mark_agent_recalling",
        "_ingest_memory_recalls",
        "RECALL_CUE_FRAMES",
    ];
    for tok in tokens {
        assert!(
            src.contains(tok),
            "A9: agent_renderer.gd must contain Phase 8-δ token {tok:?} (regression guard)"
        );
    }
}
