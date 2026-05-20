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

use std::collections::BTreeSet;
use std::fs;

use sim_bridge::ffi::world_node::{
    collect_agent_snapshot, event_view_to_owned_dict, CausalEventView, FfiFieldValue,
};
use sim_core::causal::{CausalEvent, DecisionReason, MemoryRecallTrigger};
use sim_core::components::{
    AgentState, Hunger, Memory, Position, Sleep, Social, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;

// ────────────────────────────────────────────────────────────────────────
// Constants — locked by plan
// ────────────────────────────────────────────────────────────────────────

const W: u32 = 128;
const H: u32 = 128;

const LOCALE_KEYS: [&str; 7] = [
    "UI_CAUSAL_REASON_MEMORY",
    "UI_CAUSAL_EVENT_MEMORY_RECALLED",
    "UI_CAUSAL_EVENT_MEMORY_RECALLED_CASCADE",
    "UI_AGENT_STATE_RECALLING",
    "UI_MEMORY_RECALL_TRIGGER_CASCADE",
    "UI_MEMORY_RECALL_TRIGGER_SIMILARITY",
    "UI_MEMORY_RECALL_TRIGGER_PERIODIC",
];

/// A small set of well-known pre-existing locale keys that MUST be retained
/// across Phase 8-δ's Python JSON merge. If the merge accidentally deletes
/// any of these, A12 fails. Sampled across the categories listed in the
/// `meta.categories_order` array so a partial-deletion bug shows up
/// regardless of which slice gets clobbered.
const A12_PREEXISTING_KEY_BASELINE: [&str; 8] = [
    // ui — Phase 7-δ baseline (immediate predecessor of Phase 8-δ)
    "UI_CAUSAL_EVENT_AGENT_DECISION",
    "UI_CAUSAL_EVENT_SOCIAL_INTERACTION_STARTED",
    "UI_AGENT_STATE_SOCIALIZING",
    "UI_RELATIONSHIP_PANEL_TITLE",
    // game-side legacy categories
    "ACE_DOMESTIC_VIOLENCE",
    "ACTION_BUILD",
    "ACTION_CELEBRATE",
    "ACTION_COMBAT",
];

// ────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────

/// Locate the project root (the workspace root, which is also the repo
/// root for WorldSim) by walking up from `CARGO_MANIFEST_DIR`. Mirrors the
/// Phase 7-δ harness helper.
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
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read locale file {path:?}: {e}"));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("parse locale file {path:?}: {e}"))
}

/// Pull a locale value strictly from `strings.<KEY>` (the Phase 8-δ plan
/// pins the lookup surface to the compiled JSON's `strings` dict).
fn strings_lookup<'a>(json: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    json.get("strings")
        .and_then(|s| s.get(key))
        .and_then(|v| v.as_str())
}

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

/// Strip a `#`-prefixed line comment from a GDScript code line. We approximate
/// by walking the line and treating any `#` *outside* a double-quoted string
/// as the start of a comment. Good enough for the static scan in Assertion 11
/// (we never use it to evaluate code semantics).
fn strip_gdscript_line_comment(line: &str) -> &str {
    let mut in_str = false;
    for (i, ch) in line.char_indices() {
        match ch {
            '"' => in_str = !in_str,
            '#' if !in_str => return &line[..i],
            _ => {}
        }
    }
    line
}

/// Extract every double-quoted string literal from a code line, returning
/// `(literal_text, byte_index_in_original_line_of_open_quote)` pairs.
/// Does not attempt to interpret escapes; the heuristic is good enough for
/// the static scan in Assertion 11 (the file in question contains no `\"`).
fn extract_double_quoted_literals(line: &str) -> Vec<(&str, usize)> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() && bytes[j] != b'"' {
                j += 1;
            }
            if j < bytes.len() {
                out.push((&line[start..j], i));
                i = j + 1;
            } else {
                break;
            }
        } else {
            i += 1;
        }
    }
    out
}

/// Returns true when the literal at `quote_open_idx` in `code_line` is
/// syntactically the *direct* argument of a `Locale.ltr(...)` or `_ltr(...)`
/// call (or `ltr(...)` in case of unqualified call). The heuristic walks
/// backwards from the open-quote to skip whitespace and `(`, then matches
/// the identifier immediately preceding.
fn literal_is_localized(code_line: &str, quote_open_idx: usize) -> bool {
    let prefix = &code_line[..quote_open_idx];
    let prefix_trimmed = prefix.trim_end();
    if !prefix_trimmed.ends_with('(') {
        return false;
    }
    // Strip the trailing `(` and trailing whitespace.
    let before_paren = &prefix_trimmed[..prefix_trimmed.len() - 1];
    let before_trimmed = before_paren.trim_end();
    // Pull the trailing identifier (alphanumeric + `_` + `.`).
    let ident_start = before_trimmed
        .rfind(|c: char| !(c.is_alphanumeric() || c == '_' || c == '.'))
        .map(|i| i + 1)
        .unwrap_or(0);
    let ident = &before_trimmed[ident_start..];
    matches!(ident, "_ltr" | "Locale.ltr" | "ltr")
}

/// True when the literal is an identifier-like wire discriminator that the
/// GDScript code uses for `==` matching against FFI strings (e.g.
/// `"memory_recalled"`, `"cascade_bias"`, `"agent_decision"`) or a
/// `Locale.ltr` *key* like `"UI_…"`. These are NOT player-facing text and
/// are explicitly allowlisted by the plan.
fn literal_is_wire_discriminator(literal: &str) -> bool {
    if literal.is_empty() {
        return false;
    }
    // UI_* keys are excluded by the plan ("strings that begin with UI_
    // followed by uppercase / underscore identifier characters").
    if literal.starts_with("UI_")
        && literal
            .chars()
            .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
    {
        return true;
    }
    // snake_case wire discriminator: all lowercase ASCII, digits, and `_`.
    // Examples: "memory_recalled", "cascade_bias", "agent_decision",
    // "memory_reason", "similarity_search", "periodic", "building_placed".
    // These are FFI contract strings, not player-facing text.
    if literal
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return true;
    }
    false
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 1 — locale_compiled_en_seven_keys_present
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_p8_delta_memory_locale_compiled_en_seven_keys_present() {
    // Type A — plan-locked: all 7 keys present under strings, value length
    // >= 3 AND >= 2 ASCII letters (Phase 7-δ A19 precedent).
    let json = read_locale_json("en");
    let mut present = 0usize;
    for key in LOCALE_KEYS {
        let value = strings_lookup(&json, key)
            .unwrap_or_else(|| panic!("A1: en.json missing strings.{key}"));
        let len = value.chars().count();
        let alpha = value.chars().filter(|c| c.is_ascii_alphabetic()).count();
        assert!(
            len >= 3,
            "A1: en[{key}] must have length >= 3, got {len} ({value:?})",
        );
        assert!(
            alpha >= 2,
            "A1: en[{key}] must contain >= 2 ASCII letters, got {alpha} ({value:?})",
        );
        present += 1;
    }
    assert_eq!(present, 7, "A1: must observe all 7 en locale keys");
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 2 — locale_compiled_ko_seven_keys_present_with_hangul
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_p8_delta_memory_locale_compiled_ko_seven_keys_present_with_hangul() {
    // Type A — plan-locked: all 7 keys present in ko.json, each value has
    // length >= 2, >= 1 Hangul syllable, and is byte-distinct from en.
    let en = read_locale_json("en");
    let ko = read_locale_json("ko");
    for key in LOCALE_KEYS {
        let en_val = strings_lookup(&en, key)
            .unwrap_or_else(|| panic!("A2: en.json missing strings.{key}"));
        let ko_val = strings_lookup(&ko, key)
            .unwrap_or_else(|| panic!("A2: ko.json missing strings.{key}"));
        let len = ko_val.chars().count();
        let hangul = ko_val
            .chars()
            .filter(|c| ('\u{AC00}'..='\u{D7A3}').contains(c))
            .count();
        assert!(
            len >= 2,
            "A2: ko[{key}] must have length >= 2, got {len} ({ko_val:?})",
        );
        assert!(
            hangul >= 1,
            "A2: ko[{key}] must contain >= 1 Hangul syllable, got {hangul} ({ko_val:?})",
        );
        assert_ne!(
            en_val.as_bytes(),
            ko_val.as_bytes(),
            "A2: ko[{key}] must differ from en[{key}] (no copy-paste)",
        );
    }
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 3 — locale_seven_keys_pairwise_distinct_en
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_p8_delta_memory_locale_seven_keys_pairwise_distinct_en() {
    // Type A — plan-locked: 21 pairs, 0 collisions expected.
    let en = read_locale_json("en");
    let values: Vec<&str> = LOCALE_KEYS
        .iter()
        .map(|k| {
            strings_lookup(&en, k)
                .unwrap_or_else(|| panic!("A3: en.json missing strings.{k}"))
        })
        .collect();
    let mut collisions = 0;
    for i in 0..values.len() {
        for j in (i + 1)..values.len() {
            if values[i].as_bytes() == values[j].as_bytes() {
                collisions += 1;
            }
        }
    }
    assert_eq!(
        collisions, 0,
        "A3: all 7 en values must be pairwise distinct, got {collisions} duplicate pairs in {values:?}",
    );
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 4 — locale_seven_keys_pairwise_distinct_ko
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_p8_delta_memory_locale_seven_keys_pairwise_distinct_ko() {
    // Type A — plan-locked: same as A3 but applied to ko (closes the gap
    // Phase 7-δ left open).
    let ko = read_locale_json("ko");
    let values: Vec<&str> = LOCALE_KEYS
        .iter()
        .map(|k| {
            strings_lookup(&ko, k)
                .unwrap_or_else(|| panic!("A4: ko.json missing strings.{k}"))
        })
        .collect();
    let mut collisions = 0;
    for i in 0..values.len() {
        for j in (i + 1)..values.len() {
            if values[i].as_bytes() == values[j].as_bytes() {
                collisions += 1;
            }
        }
    }
    assert_eq!(
        collisions, 0,
        "A4: all 7 ko values must be pairwise distinct, got {collisions} duplicate pairs in {values:?}",
    );
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
        other => panic!(
            "A6: production FFI dict[\"triggered_by\"] must be Str(_), got {other:?}",
        ),
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
// Assertion 7 — ffi_dict_schema_single_source_of_truth
// ────────────────────────────────────────────────────────────────────────
//
// Plan §A7: the production Godot-bound `event_view_to_dict` and the
// inspectable helper `event_view_to_owned_dict` MUST share one source of
// truth — symmetric difference of their key sets == ∅.
//
// Strategy: structural source-code inspection of
// `rust/crates/sim-bridge/src/ffi/world_node.rs`. Two invariants enforced:
//
//   (a) `fn event_view_to_dict(...)`'s body MUST call
//       `event_view_to_owned_dict(view)`. This is the explicit delegation
//       that makes the owned-dict the schema generator for both helpers.
//   (b) `event_view_to_dict`'s body MUST NOT insert any key via the
//       `dict.set("…", …)` form. Every dict key MUST flow from the
//       owned-dict iteration loop.
//
// If both invariants hold, the two helpers produce the same key set by
// construction: symmetric difference is provably ∅. This closes the prior
// RE-CODE drift risk where two parallel marshallers could silently diverge.
#[test]
fn harness_p8_delta_memory_ffi_dict_schema_single_source_of_truth() {
    let world_node_path = project_root()
        .join("rust")
        .join("crates")
        .join("sim-bridge")
        .join("src")
        .join("ffi")
        .join("world_node.rs");
    let src = fs::read_to_string(&world_node_path)
        .unwrap_or_else(|e| panic!("A7: read {world_node_path:?}: {e}"));

    // Slice out the `fn event_view_to_dict(...)` body. We locate the
    // function header, then walk forward counting braces until depth
    // returns to 0 — that delimits the body.
    let header_idx = src
        .find("fn event_view_to_dict(view: &CausalEventView) -> VarDictionary {")
        .expect("A7: world_node.rs must declare `fn event_view_to_dict(view: &CausalEventView) -> VarDictionary {`");
    let body_start = src[header_idx..]
        .find('{')
        .map(|off| header_idx + off + 1)
        .expect("A7: event_view_to_dict opening brace");
    let mut depth = 1i32;
    let mut cursor = body_start;
    for (i, ch) in src[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    cursor = body_start + i;
                    break;
                }
            }
            _ => {}
        }
    }
    assert!(
        cursor > body_start,
        "A7: event_view_to_dict body must be brace-balanced (matching close not found)",
    );
    let body = &src[body_start..cursor];

    // (a) the body MUST delegate to `event_view_to_owned_dict(view)`.
    assert!(
        body.contains("event_view_to_owned_dict(view)"),
        "A7: event_view_to_dict body must call `event_view_to_owned_dict(view)` so the two helpers share a single schema source. Body:\n{body}",
    );

    // (b) the body MUST NOT insert any new dict key independently. Any
    // `dict.set("…", …)` call inside this function would create a key the
    // owned-dict helper does not know about, breaking the single-source
    // invariant. We allow `dict.set(key, …)` (where `key` is a variable
    // binding from the owned-dict iteration) but forbid `dict.set("…"`.
    let new_key_inserts: Vec<&str> = body
        .lines()
        .filter(|line| line.contains("dict.set(\""))
        .collect();
    assert!(
        new_key_inserts.is_empty(),
        "A7: event_view_to_dict must not insert keys with `dict.set(\"…\", …)` outside the owned-dict iteration — every key MUST flow through `event_view_to_owned_dict`. Offending lines:\n{}",
        new_key_inserts.join("\n"),
    );

    // Liveness check: the owned-dict helper must still emit the four
    // canonical MemoryRecalled-relevant fields the Godot UI reads.
    let owned = ffi_dict_for_memory_recalled(MemoryRecallTrigger::CascadeBias);
    for required_key in ["kind", "triggered_by", "recalled_event", "agent_id"] {
        assert!(
            owned.contains_key(required_key),
            "A7: event_view_to_owned_dict must emit \"{required_key}\" for CausalEvent::MemoryRecalled (owned dict keys: {:?})",
            owned.keys().collect::<Vec<_>>(),
        );
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
// Assertion 10 — phase7_delta_state_tag_regression_guard
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_p8_delta_memory_phase7_delta_state_tag_regression_guard() {
    // Type D regression guard — Phase 7-δ A22 contract: state_tag ∈ {0,1,2,3}.
    let mut engine = make_stage1_engine(42, 20);
    let mut observed: BTreeSet<u8> = BTreeSet::new();
    for _ in 0..200 {
        engine.tick();
        let rows = collect_agent_snapshot(&engine.world);
        for row in &rows {
            assert!(
                matches!(row.state_tag, 0..=3),
                "A10: Phase 7-δ A22 violated — state_tag {} outside {{0,1,2,3}}",
                row.state_tag,
            );
            observed.insert(row.state_tag);
        }
    }
    assert!(
        !observed.is_empty(),
        "A10: at least one snapshot row must have been observed across 200 ticks",
    );
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 11 — causal_panel_no_hardcoded_english_for_recalled_event
// ────────────────────────────────────────────────────────────────────────
//
// Plan §A11 (broad scan): the memory-rendering region — the `_format_event()`
// body's `"memory_recalled"` branch and the `"agent_decision"` branch where
// it routes `"memory_reason"` — MUST NOT contain:
//   (i)   any of the named English fragments `"event #"`, `"event#"`,
//         `"Event #"`, `"(event id"`;
//   (ii)  the literal `"Memory recall"` outside `Locale.ltr(...)`;
//   (iii) ANY quoted alphabetic string of length >= 4 that is NOT either:
//           - wrapped in `Locale.ltr(...)` / `_ltr(...)`,
//           - a `UI_*` identifier key,
//           - a snake_case wire discriminator the GDScript `==`-matches
//             against (e.g. `"memory_recalled"`, `"cascade_bias"`,
//             `"agent_decision"`, `"memory_reason"`, etc.).
//
// The third clause is the plan's explicit "broad scan" — it catches
// future regressions like `extra += " (event id: " + str(rid) + ")"`
// that the named-literal scan in (i) would miss after a re-wording.
//
// Scope: we deliberately narrow the scan to the MEMORY-rendering region
// (per the plan text) and explicitly NOT the building_placed /
// stamp_dirty / influence_changed branches, which carry pre-existing
// Phase 3-γ formatting strings (e.g. " radius=") that predate Phase 8-δ
// and are out of scope for this feature.
#[test]
fn harness_p8_delta_memory_causal_panel_no_hardcoded_english_for_recalled_event() {
    let panel_path = project_root()
        .join("scripts")
        .join("ui")
        .join("panels")
        .join("causal_panel.gd");
    let src = fs::read_to_string(&panel_path)
        .unwrap_or_else(|e| panic!("A11: read {panel_path:?}: {e}"));

    // Find the `_format_event` function body.
    let fn_start = src
        .find("func _format_event(")
        .expect("A11: causal_panel.gd must declare _format_event(ev: Dictionary) -> String");
    let next_fn = src[fn_start + 1..]
        .find("\nfunc ")
        .map(|off| fn_start + 1 + off)
        .unwrap_or(src.len());
    let full_fn = &src[fn_start..next_fn];

    // Extract the "memory-rendering region" exactly per the plan:
    //  (R1) the `"memory_recalled":` match arm body until the next match
    //       arm or the function's end;
    //  (R2) the `if reason == "memory_reason":` branch inside the
    //       `"agent_decision":` match arm until its `else` (or `match` arm
    //       end / function end if no else).
    //
    // We aggregate both into a single `String` so the same scan rules apply.
    let region_owned = {
        let mut out = String::new();
        // R1: "memory_recalled" match arm.
        if let Some(idx) = full_fn.find("\"memory_recalled\":") {
            // Walk to the next match-arm at the same indent (8 spaces under
            // the match) — heuristically the next `\n\t\t\t"` after idx, OR
            // function end.
            let after = &full_fn[idx..];
            let end_off = after[1..]
                .find("\n\t\t\t\"")
                .map(|o| 1 + o)
                .unwrap_or(after.len());
            out.push_str(&after[..end_off]);
            out.push('\n');
        }
        // R2: the `memory_reason` inner branch of `"agent_decision":`.
        if let Some(idx) = full_fn.find("if reason == \"memory_reason\":") {
            let after = &full_fn[idx..];
            // End at the matching `else` (next line whose trim_start begins
            // with `else`) OR the next outer match arm.
            let mut end_off = after.len();
            for (i, _) in after.match_indices('\n') {
                let next_line = &after[(i + 1)..];
                let trimmed = next_line.trim_start();
                if trimmed.starts_with("else") || trimmed.starts_with('"') {
                    end_off = i;
                    break;
                }
            }
            out.push_str(&after[..end_off]);
            out.push('\n');
        }
        out
    };
    let region: &str = &region_owned;
    assert!(
        !region.is_empty(),
        "A11: must locate the memory-rendering region in _format_event() — got empty region",
    );

    // (i) Named-literal scan. The plan calls out these substrings
    // explicitly; they are the regressions we are guarding against. Scan
    // only *code lines* (line-comments stripped) inside quoted literals.
    let forbidden_named_fragments = ["event #", "event#", "(event id"];

    // (iii) Broad scan accumulator — collect every player-facing English
    // literal violation across the region so the assertion error reports
    // all offenders at once.
    let mut broad_violations: Vec<String> = Vec::new();

    for (lineno_off, raw_line) in region.lines().enumerate() {
        // Strip everything from the first `#` outside of a string literal
        // — a conservative GDScript line-comment stripper. We approximate
        // by treating any `#` outside a "…" range as a comment start.
        let code_part = strip_gdscript_line_comment(raw_line);
        // Extract every double-quoted string literal on this code-line.
        let literals = extract_double_quoted_literals(code_part);
        for (literal, quote_open_idx) in literals {
            let lc_literal = literal.to_ascii_lowercase();

            // (i) named-literal hard bans (always fatal regardless of
            // wrapping — these are textbook anti-patterns).
            for needle in &forbidden_named_fragments {
                let lc_needle = needle.to_ascii_lowercase();
                assert!(
                    !lc_literal.contains(&lc_needle),
                    "A11(i): causal_panel.gd _format_event() line {lineno_off} contains forbidden hardcoded English fragment {needle:?} inside quoted literal {literal:?} — must use Locale.ltr(\"UI_*\") or a bare numeric/hex literal instead.",
                );
            }

            // (iii) broad scan: any quoted alphabetic literal of length
            // >= 4 NOT wrapped in `Locale.ltr(...)` / `_ltr(...)` AND not
            // an allowlisted UI_* identifier or wire discriminator.
            let alpha_count =
                literal.chars().filter(|c| c.is_ascii_alphabetic()).count();
            if alpha_count >= 4
                && !literal_is_localized(code_part, quote_open_idx)
                && !literal_is_wire_discriminator(literal)
            {
                broad_violations.push(format!(
                    "line {lineno_off}: {literal:?} (alpha_count={alpha_count}, not Locale.ltr-wrapped, not an allowlisted UI_*/wire discriminator)",
                ));
            }
        }
    }

    // (ii) literal `"Memory recall"` outside `Locale.ltr(...)` — explicit
    // plan-named ban (it's an en.json value, not a key).
    //
    // Note: identifier substrings like `recalled_event` / `InputEventKey`
    // are dotted-property accesses, not quoted literals, so they cannot
    // match this check (which scans the raw file text — but the file does
    // not quote either identifier).
    assert!(
        !region.contains("\"Memory recall\""),
        "A11(ii): causal_panel.gd _format_event() contains the literal string \"Memory recall\" outside Locale.ltr — must be rendered via `Locale.ltr(\"UI_CAUSAL_REASON_MEMORY\")`.",
    );

    // Report all (iii) broad-scan violations together so the developer
    // sees the full picture.
    assert!(
        broad_violations.is_empty(),
        "A11(iii) broad scan: causal_panel.gd _format_event() contains {} player-facing English quoted literal(s) not wrapped in Locale.ltr(...). Each must use a UI_* locale key. Offenders:\n  {}",
        broad_violations.len(),
        broad_violations.join("\n  "),
    );

    // Positive check: the memory branch MUST actually use the new locale
    // keys via `_ltr(...)` / `Locale.ltr(...)`. Without these the test
    // would pass on an empty branch; the keys are the contract.
    for required_key in [
        "UI_CAUSAL_REASON_MEMORY",
        "UI_CAUSAL_EVENT_MEMORY_RECALLED",
        "UI_CAUSAL_EVENT_MEMORY_RECALLED_CASCADE",
    ] {
        assert!(
            region.contains(required_key),
            "A11 positive check: causal_panel.gd _format_event() must reference Locale key {required_key:?} so the memory branch is actually localized.",
        );
    }
}

// ────────────────────────────────────────────────────────────────────────
// Assertion 12 — locale_compiled_json_structural_integrity
// ────────────────────────────────────────────────────────────────────────
//
// Plan §A12: BOTH files (a) parse cleanly, (b) contain a top-level
// `strings` JSON object, (c) every value under `strings` is a JSON string
// (not number / null), and (d) the file's pre-existing keys (snapshotted
// at the Phase 7-δ baseline) are still present. Guards against a
// Python JSON merge bug that silently deletes keys outside the 7 new ones.
#[test]
fn harness_p8_delta_memory_locale_compiled_json_structural_integrity() {
    for locale in ["en", "ko"] {
        let path = project_root()
            .join("localization")
            .join("compiled")
            .join(format!("{locale}.json"));
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("A12: read {path:?}: {e}"));
        let json: serde_json::Value = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("A12: parse {path:?}: {e}"));

        // (b) `strings` is a JSON object.
        let strings = json
            .get("strings")
            .unwrap_or_else(|| panic!("A12: {locale}.json missing top-level `strings`"));
        let map = strings.as_object().unwrap_or_else(|| {
            panic!(
                "A12: {locale}.json `strings` must be a JSON object, got {strings:?}",
            )
        });

        // (c) every value under `strings` is a JSON string.
        for (key, value) in map {
            assert!(
                value.is_string(),
                "A12: {locale}.json strings[{key}] must be a JSON string, got {value:?}",
            );
        }

        // (d) baseline pre-existing keys MUST still be present after the
        // Phase 8-δ merge. If any of these go missing the merge has
        // silently deleted state outside the 7 new keys' window.
        for baseline_key in A12_PREEXISTING_KEY_BASELINE {
            assert!(
                map.contains_key(baseline_key),
                "A12(d): {locale}.json `strings` is missing pre-existing key {baseline_key:?} — Phase 8-δ JSON merge must not delete pre-existing keys.",
            );
        }
    }
}

// ────────────────────────────────────────────────────────────────────────
// Wiring guard — `mark_agent_recalling()` must have a real caller
// ────────────────────────────────────────────────────────────────────────
//
// Code-attempt 1 of this plan left `mark_agent_recalling()` as a public
// function with no production caller — the recall cue could never fire
// because nothing ever read `memory_recalled` events from the FFI. This
// test guards against that regression by grepping the AgentRenderer
// GDScript file for both the public sink (`mark_agent_recalling(`) AND
// the source loop that fans `"memory_recalled"` events into it. If the
// loop is removed, the test fails — the cue would be dead UI again.
//
// This is intentionally a static file-scan (not an FFI invocation) so the
// guard is non-circular: it does not depend on the Rust side observing
// the GDScript side via the FFI it is trying to test.
#[test]
fn harness_p8_delta_memory_recalling_cue_has_real_caller() {
    let renderer_path = project_root()
        .join("scripts")
        .join("ui")
        .join("agent_renderer.gd");
    let src = fs::read_to_string(&renderer_path)
        .unwrap_or_else(|e| panic!("WIRING: read {renderer_path:?}: {e}"));
    // (a) the sink must still exist
    assert!(
        src.contains("func mark_agent_recalling("),
        "WIRING: scripts/ui/agent_renderer.gd must still declare `mark_agent_recalling()` (the recall-cue sink)",
    );
    // (b) the sink must have at least one *call site* outside its own
    // declaration line. We strip the declaration line and look for the
    // call form. This catches "function exists but never called" — the
    // exact gap that code-attempt 1 left.
    let call_count = src
        .lines()
        .filter(|line| !line.trim_start().starts_with("func mark_agent_recalling("))
        .filter(|line| line.contains("mark_agent_recalling("))
        .count();
    assert!(
        call_count >= 1,
        "WIRING: scripts/ui/agent_renderer.gd must call `mark_agent_recalling(...)` from at least one real code path (event polling, signal handler, etc.); found 0. Without a caller the Phase 8-δ recall cue is dead UI.",
    );
    // (c) the file must contain the event-source loop — i.e. it must
    // string-match `"memory_recalled"` (the FFI discriminator) so the
    // caller is reading real causal events, not synthetic ones.
    assert!(
        src.contains("\"memory_recalled\""),
        "WIRING: scripts/ui/agent_renderer.gd must reference the `\"memory_recalled\"` FFI discriminator so its `mark_agent_recalling(...)` call site is driven by real causal events (not a synthetic stub).",
    );
    // (d) the file must call `get_tile_causal_history` (the existing FFI
    // surface that returns recall events). This anchors (c) to a real FFI
    // read instead of, say, a hard-coded constant that happens to spell
    // the discriminator.
    assert!(
        src.contains("get_tile_causal_history"),
        "WIRING: scripts/ui/agent_renderer.gd must call `world_sim.get_tile_causal_history(...)` so the recall cue is driven by the actual causal-log FFI, not a stub.",
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
        let entity = hecs::Entity::from_bits(row.entity_bits)
            .expect("entity_bits is a valid hecs::Entity");
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
fn harness_p8_delta_memory_renderer_recall_lookup_uses_agent_id_domain() {
    // Type A static-source contract — the agent_renderer.gd MUST read
    // `agent_ids` from the snapshot dictionary AND use it (not `ids`) as
    // the lookup key for `_recalling_agents`. Closes the prior code
    // attempt's bug where the cue could never fire because of a domain
    // mismatch.
    let renderer_path = project_root()
        .join("scripts")
        .join("ui")
        .join("agent_renderer.gd");
    let src = fs::read_to_string(&renderer_path)
        .unwrap_or_else(|e| panic!("read {renderer_path:?}: {e}"));

    // (a) the snapshot dict must be unpacked into a local `agent_ids` array.
    assert!(
        src.contains("\"agent_ids\""),
        "agent_renderer.gd must read `\"agent_ids\"` from the snapshot dict (the parallel array carrying Agent.id per row).",
    );
    // (b) `_recalling_agents.has(agent_ids[` — the recall lookup must
    // index into the agent_ids array, NOT into `ids` (entity bits). This
    // pins the domain match end-to-end: the event marks by AgentId, the
    // dict carries AgentId, the snapshot exposes AgentId, the renderer
    // looks up by AgentId.
    assert!(
        src.contains("_recalling_agents.has(agent_ids["),
        "agent_renderer.gd must check `_recalling_agents.has(agent_ids[i])` so the recall cue lookup uses the AgentId domain (matching `event.agent_id` from `CausalEvent::MemoryRecalled`). Found no such expression.",
    );
    // (c) `mark_agent_recalling(agent_id)` — the producer side must still
    // mark using the AgentId from the event dict (the field the FFI emits
    // for `CausalEvent::MemoryRecalled.agent`).
    assert!(
        src.contains("mark_agent_recalling(agent_id)"),
        "agent_renderer.gd must call `mark_agent_recalling(agent_id)` with the AgentId value from the causal event dict, so the marked key matches the lookup key.",
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
    let agent_id_value = dict
        .get("agent_id")
        .unwrap_or_else(|| panic!("FFI dict missing \"agent_id\" for MemoryRecalled; keys={:?}", dict.keys().collect::<Vec<_>>()));
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

// ────────────────────────────────────────────────────────────────────────
// Phase 7-δ silent-collaborator imports (avoid dead-imports clippy warning
// in case the assertions above are refactored).
// ────────────────────────────────────────────────────────────────────────
#[test]
fn harness_p8_delta_memory_imports_compile() {
    // Touch each imported symbol so a future refactor that removes one of
    // the imports trips this file before the workspace gate.
    let _idle = AgentState::Idle;
    let _tk = TargetKind::Food;
    let _trig = MemoryRecallTrigger::CascadeBias;
    let _reason = DecisionReason::MemoryReason;
}
