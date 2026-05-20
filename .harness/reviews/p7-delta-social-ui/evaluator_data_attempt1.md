# Evaluation Task — p7-delta-social-ui

## Test Plan (the spec — this defines what SHOULD be true)

---
feature: p7-delta-social-ui
plan_attempt: 2
seed: 42
agent_count: 20
---

## Assertions

### Assertion 1: state_tag_idle_default
- threshold: exactly == 0
- type: A

### Assertion 2: state_tag_seeking_agent
- threshold: exactly == 1
- type: A

### Assertion 3: state_tag_consuming_agent
- threshold: exactly == 2
- type: A

### Assertion 4: state_tag_consuming_other
- threshold: exactly == 3
- type: A

### Assertion 5: state_tag_row_count_matches_agent_count
- threshold: exactly == 4 (literal)
- type: A

### Assertion 6: state_tag_seeking_non_agent_target
- threshold: exactly == 1
- type: A

### Assertion 7: state_tag_matches_live_agentstate_same_query
- threshold: every row: row.state_tag == expected_tag
- type: A
- ticks: 50

### Assertion 8: relationship_snapshot_empty_when_no_relationships
- threshold: exactly == 0
- type: A

### Assertion 9: relationship_snapshot_filters_zero_pairs
- threshold: exactly == 0
- type: A

### Assertion 10: relationship_snapshot_includes_familiarity_only_pair
- threshold: length == 1; row.familiarity ≈ 0.1 (1e-9); row.hostility ≈ 0.0 (1e-9)
- type: A

### Assertion 11: relationship_snapshot_includes_hostility_only_pair
- threshold: length == 1; row.familiarity ≈ 0.0 (1e-9); row.hostility ≈ 0.2 (1e-9)
- type: A

### Assertion 12: relationship_snapshot_includes_mixed_pair_both_fields_preserved
- threshold: length == 1; familiarity ≈ 0.1 (1e-9); hostility ≈ 0.05 (1e-9)
- type: A

### Assertion 13: relationship_snapshot_excludes_negative_values
- threshold: exactly == 0
- type: A

### Assertion 14: relationship_snapshot_id_a_lt_id_b_canonical
- threshold: every row: row.id_a < row.id_b
- type: A

### Assertion 15: relationship_snapshot_one_row_per_pair_after_two_inserts_same_pair
- threshold: exactly == 1
- type: A

### Assertion 16: end_to_end_socializing_pair_produces_snapshot_state_tag_2
- threshold: both state_tag values exactly == 2 at qualifying tick t
- type: A
- ticks: up to 80

### Assertion 17: end_to_end_relationship_snapshot_after_completed_interaction
- threshold: 1 row for pair; familiarity ≈ 0.1 (hardcoded); both AgentState == Idle; interaction_progress absent or ≈ 0.0
- type: A
- ticks: 80

### Assertion 18: end_to_end_state_tag_idle_after_interaction
- threshold: both state_tag values exactly == 0
- type: A
- ticks: 80

### Assertion 19: locale_compiled_contains_all_seven_keys_en
- threshold: all 7 keys present; each value length ≥ 3; each value ≥ 2 ASCII alpha chars
- type: A

### Assertion 20: locale_compiled_contains_all_seven_keys_ko
- threshold: all 7 keys present; each value length ≥ 2; each has ≥ 1 Hangul char; each differs from en
- type: A

### Assertion 21: locale_seven_keys_pairwise_distinct_en
- threshold: exactly == 0 identical pairs (all 7 values pairwise distinct)
- type: A

### Assertion 22: agent_snapshot_state_tag_byte_range
- threshold: every row.state_tag ∈ {0, 1, 2, 3}
- type: A
- ticks: 200

### Assertion 23: state_tag_stream_deterministic_across_two_runs_same_seed
- threshold: both engines produce byte-identical (entity_id, state_tag) vectors at every tick
- type: A
- ticks: 100

---

## Test Results (what actually happened)

```
test harness_state_tag_idle_default ... ok
test harness_state_tag_consuming_agent ... ok
test harness_state_tag_consuming_other ... ok
test harness_state_tag_row_count_matches_agent_count ... ok
test harness_state_tag_seeking_non_agent_target ... ok
test harness_state_tag_seeking_agent ... ok
test harness_end_to_end_socializing_pair_produces_snapshot_state_tag_2 ... ok
test harness_end_to_end_relationship_snapshot_after_completed_interaction ... ok
test harness_end_to_end_state_tag_idle_after_interaction ... ok
test harness_state_tag_matches_live_agentstate_same_query ... ok
test harness_locale_compiled_contains_all_seven_keys_en ... ok
test harness_locale_seven_keys_pairwise_distinct_en ... ok
test harness_locale_compiled_contains_all_seven_keys_ko ... ok
test harness_agent_snapshot_state_tag_byte_range ... ok
test harness_state_tag_stream_deterministic_across_two_runs_same_seed ... ok
test harness_relationship_snapshot_empty_when_no_relationships ... ok
test harness_relationship_snapshot_filters_zero_pairs ... ok
test harness_relationship_snapshot_includes_familiarity_only_pair ... ok
test harness_relationship_snapshot_includes_hostility_only_pair ... ok
test harness_relationship_snapshot_includes_mixed_pair_both_fields_preserved ... ok
test harness_relationship_snapshot_excludes_negative_values ... ok
test harness_relationship_snapshot_id_a_lt_id_b_canonical ... ok
test harness_relationship_snapshot_one_row_per_pair_after_two_inserts_same_pair ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

Workspace regression (cargo test --workspace):
- All test suites: ok. 0 failures across 921+ tests in all workspace crates.
- Clippy: clean (0 warnings, 0 errors).

Visual verification: SKIP — Godot runtime absent (VLM env cost applied per hook policy).
FFI chain: SKIP — sim_bridge.gd absent (V7 reset early phase).
Regression guard: CLEAN — no pre-existing tests newly broken.

---

## Key Implementation Changes

### `rust/crates/sim-bridge/src/ffi/world_node.rs`

1. **`AgentSnapshotRow`** — added `state_tag: u8` field (Phase 7-δ extension).

2. **`collect_agent_snapshot`** — query changed from `(&Agent, &Position)` to
   `(&Agent, &Position, Option<&AgentState>)`. Tag computed at query time
   from the same `AgentState` reference (no caching): 
   - `None | Some(Idle) => 0`
   - `Some(Seeking { .. }) => 1`
   - `Some(Consuming { target: Agent(_) }) => 2`
   - `Some(Consuming { .. }) => 3`

3. **`agent_rows_split`** — now returns 4-tuple `(Vec<i64>, Vec<i32>, Vec<i32>, Vec<u8>)`
   (added `states` Vec). `agent_rows_to_dict` updated to include `states: PackedByteArray`.

4. **`RelationshipSnapshotRow`** — new struct `{id_a: i64, id_b: i64, familiarity: f64, hostility: f64}`.

5. **`collect_relationship_snapshot`** — new function iterating `resources.relationships`,
   filtering `familiarity > 0.0 || hostility > 0.0`. Returns rows in iteration order.

6. **`WorldSimNode::get_relationship_snapshot`** — new `#[func]` forwarding to
   `collect_relationship_snapshot` + `relationship_rows_to_variant_array`.

### GDScript

- **`scripts/ui/panels/causal_panel.gd`** — added `"agent_decision"`, 
  `"social_interaction_started"`, `"social_interaction_completed"` match arms
  in `_format_event()` using new locale keys.
  
- **`scripts/ui/agent_renderer.gd`** — reads `states` PackedByteArray from snapshot dict;
  applies green tint modulate when `state_tag == 2`.

### Localization

- **`localization/fluent/en/messages.ftl`** + **`ko/messages.ftl`** — 7 new keys appended.
- **`localization/compiled/en.json`** + **`ko.json`** — 7 keys added under `strings` object
  (5116 → 5123 strings each).

Keys:
  `UI_CAUSAL_REASON_SOCIAL`, `UI_CAUSAL_EVENT_AGENT_DECISION`,
  `UI_CAUSAL_EVENT_SOCIAL_INTERACTION_STARTED`, `UI_CAUSAL_EVENT_SOCIAL_INTERACTION_COMPLETED`,
  `UI_AGENT_STATE_SOCIALIZING`, `UI_RELATIONSHIP_PANEL_TITLE`, `UI_RELATIONSHIP_PAIR_ROW`

### Regression fix: `harness_p4_gamma_rendering.rs`

Line 226: `let (ids, xs, ys) = agent_rows_split(&rows)` →
`let (ids, xs, ys, states) = agent_rows_split(&rows)` (4-tuple destructure).
Added `states.len()` assertion. All 11 p4-γ tests continue to pass.
