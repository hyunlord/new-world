# Implementation Task — p7-delta-social-ui

## Test Plan (thresholds are LOCKED — do not modify)

---
feature: p7-delta-social-ui
plan_attempt: 2
seed: 42
agent_count: 20
---

## Assertions

### Assertion 1: state_tag_idle_default
- metric: For an agent inserted with `AgentState::Idle`, the `state_tag` field on the row returned by `collect_agent_snapshot(&world)` for that agent's entity.
- threshold: exactly == 0
- type: A
- rationale: "Locked mapping in §2-A-1: Idle=0. Any other value is a tag-table bug."
- ticks: 0 (no `engine.tick()` calls; assert directly after spawn + Idle insertion)
- components_read: [Agent, Position, AgentState]

### Assertion 2: state_tag_seeking_agent
- metric: For an agent whose `AgentState` is overwritten to `AgentState::Seeking { target: TargetKind::Agent(<other_id>) }`, the `state_tag` field on the row from `collect_agent_snapshot`.
- threshold: exactly == 1
- type: A
- rationale: "Locked mapping in §2-A-1: Seeking=1. Direct invariant — no dependence on system execution."
- ticks: 0
- components_read: [Agent, Position, AgentState]

### Assertion 3: state_tag_consuming_agent
- metric: For an agent whose `AgentState` is overwritten to `AgentState::Consuming { target: TargetKind::Agent(<other_id>) }`, the `state_tag` field on the row from `collect_agent_snapshot`.
- threshold: exactly == 2
- type: A
- rationale: "Locked mapping in §2-A-1: ConsumingAgent=2. This is the value the renderer keys off to tint socializing agents — wrong value = silent visual regression."
- ticks: 0
- components_read: [Agent, Position, AgentState]

### Assertion 4: state_tag_consuming_other
- metric: For an agent whose `AgentState` is overwritten to `AgentState::Consuming { target: TargetKind::Food }` (a non-Agent TargetKind), the `state_tag` field on the row from `collect_agent_snapshot`.
- threshold: exactly == 3
- type: A
- rationale: "Locked mapping in §2-A-1: ConsumingOther=3. Disambiguates from ConsumingAgent so the renderer does NOT tint non-social Consuming as socializing (false positive)."
- ticks: 0
- components_read: [Agent, Position, AgentState]

### Assertion 5: state_tag_row_count_matches_agent_count
- metric: In a single test world containing exactly 4 agents — one with `AgentState::Idle`, one with `AgentState::Seeking { target: TargetKind::Agent(_) }`, one with `AgentState::Consuming { target: TargetKind::Agent(_) }`, and one with `AgentState::Consuming { target: TargetKind::Food }` — each with `Agent` and `Position` components, the length of `collect_agent_snapshot(&world)`.
- threshold: exactly == 4 (literal)
- type: A
- rationale: "Snapshot must include every agent matching (Agent, Position, AgentState). A short row count means the query lost agents — the renderer would silently drop them. Literal count removes ambiguity about whether N is parameterized."
- ticks: 0
- components_read: [Agent, Position, AgentState]

### Assertion 6: state_tag_seeking_non_agent_target
- metric: For an agent whose `AgentState` is overwritten to `AgentState::Seeking { target: TargetKind::Food }` (a non-Agent target on the Seeking variant), the `state_tag` field on the row from `collect_agent_snapshot`.
- threshold: exactly == 1
- type: A
- rationale: "§2-A-1 specifies Seeking → 1 with no branching on TargetKind. Without this assertion, a Generator could ship a match arm that returns 0 for Seeking{Food} (e.g. by mistakenly mirroring the Consuming branch's TargetKind discrimination onto Seeking) and pass every other test. Promoted from Edge Cases."
- ticks: 0
- components_read: [Agent, Position, AgentState]

### Assertion 7: state_tag_matches_live_agentstate_same_query
- metric: For each entity returned by `collect_agent_snapshot(&world)` on a 20-agent stage1 engine after 50 ticks, read that entity's `AgentState` directly via `world.get::<AgentState>(entity)` and compute the expected tag from the §2-A-1 mapping. Compare row.state_tag to expected_tag for every row.
- threshold: every row: row.state_tag == expected_tag (computed from the same world snapshot at the same call site, no intervening ticks)
- type: A
- rationale: "Closes the hardcoded-match-table gaming vector. A Generator that computes state_tag from a stale cached value (e.g. previous tick's AgentState) would pass A1–A4 + A12 + A14 + A18 but fail this. Forces tag derivation from the same world the rest of the snapshot reads."
- ticks: 50
- components_read: [Agent, Position, AgentState]

### Assertion 8: relationship_snapshot_empty_when_no_relationships
- metric: Length of the vector returned by `collect_relationship_snapshot(&resources)` on a freshly built engine before any social interaction has occurred (`SimResources::relationships` empty).
- threshold: exactly == 0
- type: A
- rationale: "Filter contract §2-A-2: only pairs with familiarity > 0 OR hostility > 0. Empty map ⇒ empty result. Any non-zero count is a default-value leak (e.g. emitting a stub row)."
- ticks: 0
- components_read: [] (resource-only — `SimResources::relationships`)

### Assertion 9: relationship_snapshot_filters_zero_pairs
- metric: After directly inserting a pair into `resources.relationships` with `familiarity = 0.0` and `hostility = 0.0` (using `RelationshipState::default()` or explicit zero), the length of `collect_relationship_snapshot(&resources)`.
- threshold: exactly == 0
- type: A
- rationale: "Filter contract §2-A-2 explicitly excludes the (0.0, 0.0) case. A pair that was created but never bumped must not be returned — otherwise the debug overlay would surface every default-initialized pair."
- ticks: 0
- components_read: [] (resource-only)

### Assertion 10: relationship_snapshot_includes_familiarity_only_pair
- metric: After inserting a single pair into `resources.relationships` with `familiarity = 0.1`, `hostility = 0.0`, the (length of returned vector, the single row's `familiarity`, the single row's `hostility`).
- threshold: length exactly == 1; row.familiarity within 1e-9 of 0.1; row.hostility within 1e-9 of 0.0
- type: A
- rationale: "Inclusion contract §2-A-2: familiarity > 0 is sufficient. The exact f64 round-trip proves the collector does not lossy-convert (e.g. truncate to f32 mid-pipeline)."
- ticks: 0
- components_read: [] (resource-only)

### Assertion 11: relationship_snapshot_includes_hostility_only_pair
- metric: After inserting a single pair with `familiarity = 0.0`, `hostility = 0.2`, the (length of returned vector, the single row's `familiarity`, the single row's `hostility`).
- threshold: length exactly == 1; row.familiarity within 1e-9 of 0.0; row.hostility within 1e-9 of 0.2
- type: A
- rationale: "Inclusion contract §2-A-2 uses OR: hostility > 0 is also sufficient. A collector that only checks familiarity would drop this row. Pin BOTH fields with 1e-9 tolerance to match A10's precision (closes the looser-threshold gap noted by Challenger)."
- ticks: 0
- components_read: [] (resource-only)

### Assertion 12: relationship_snapshot_includes_mixed_pair_both_fields_preserved
- metric: After inserting a single pair with `familiarity = 0.1` AND `hostility = 0.05`, the (length of returned vector, the single row's `familiarity`, the single row's `hostility`).
- threshold: length exactly == 1; row.familiarity within 1e-9 of 0.1; row.hostility within 1e-9 of 0.05
- type: A
- rationale: "Most likely real-world configuration. Catches struct-field swap (familiarity↔hostility) — a swap would still pass A10 and A11 individually because each tests only one field, but here the two fields have distinct values so a swap produces row.familiarity ≈ 0.05 and row.hostility ≈ 0.1, failing both tolerance checks. Promoted from Edge Cases per Challenger."
- ticks: 0
- components_read: [] (resource-only)

### Assertion 13: relationship_snapshot_excludes_negative_values
- metric: After inserting a single pair with `familiarity = -0.1` and `hostility = -0.1` (assuming the f64 type permits this), the length of `collect_relationship_snapshot(&resources)`.
- threshold: exactly == 0
- type: A
- rationale: "Filter contract §2-A-2 uses strict `> 0`, not `!= 0`. A Generator using `!= 0.0` instead of `> 0.0` would pass A8–A12 but include negative-value rows here. If RelationshipState rejects negative writes at the insertion site, document that and the test still passes (length=0 because nothing was inserted)."
- ticks: 0
- components_read: [] (resource-only)

### Assertion 14: relationship_snapshot_id_a_lt_id_b_canonical
- metric: For every row returned by `collect_relationship_snapshot(&resources)` after seeding pairs via `RelationshipKey::new(...)` in both `(a, b)` and `(b, a)` orders, the pair `(row.id_a, row.id_b)`.
- threshold: For every row, `row.id_a < row.id_b` (strict ordering)
- type: A
- rationale: "`RelationshipKey::new` is canonical (smaller id first — verified by Phase 7-γ A17). Exposing rows in non-canonical order would make the debug overlay print duplicate `0↔1` and `1↔0` rows for the same pair."
- ticks: 0
- components_read: [] (resource-only)

### Assertion 15: relationship_snapshot_one_row_per_pair_after_two_inserts_same_pair
- metric: Insert the same logical pair twice via `RelationshipKey::new(a, b)` and `RelationshipKey::new(b, a)` (both with familiarity = 0.1). Count of rows returned by `collect_relationship_snapshot(&resources)`.
- threshold: exactly == 1
- type: A
- rationale: "Canonical key deduplication is a `RelationshipKey` invariant — both inserts must collapse into one map entry, therefore one row. A count of 2 means the collector or store is not using the canonical key."
- ticks: 0
- components_read: [] (resource-only)

### Assertion 16: end_to_end_socializing_pair_produces_snapshot_state_tag_2
- metric: Run the Phase 7-γ social interaction scenario (2 co-located agents at (6,5), `Social::new(0.0, 1.0)`, all other needs growth_rate=0.0). Scan ticks 0..80; record the first tick `t` at which agent_1's `AgentState` (read via direct world query) is `Consuming { target: TargetKind::Agent(_) }` AND interaction_progress is strictly less than `REQUIRED_INTERACTION_PROGRESS`. At that tick, the `state_tag` for both agents in `collect_agent_snapshot(&world)`.
- threshold: (precondition) such observation tick `t` MUST exist in [0, 80); the test FAILS if no qualifying tick is found within the scan window. (main) both state_tag values exactly == 2 at tick `t`.
- type: A
- rationale: "End-to-end: the renderer-facing tag must agree with the live `AgentState` for the exact frames the player would see the tint. The explicit precondition closes the vacuous-pass gaming vector: if the scenario never reaches Consuming, the test must fail rather than silently terminate the scan loop without assertion."
- ticks: Up to 80 (Phase 7-γ N_TICKS); stop scanning at the first qualifying observation tick.
- components_read: [Agent, Position, AgentState, Social]; resource read: interaction_progress

### Assertion 17: end_to_end_relationship_snapshot_after_completed_interaction
- metric: After running the same Phase 7-γ scenario for the full 80 ticks, `collect_relationship_snapshot(&resources)` filtered to the (id_1, id_2) pair: (length, the row's familiarity). Additionally, read `resources.interaction_progress` for the pair AND each agent's `AgentState` via `world.get`.
- threshold: there exists exactly 1 row matching `{id_a, id_b} == {id_1, id_2}` (as a set); that row's `familiarity` is within 1e-9 of the literal `0.1` (NOT imported from `FAMILIARITY_BUMP` — the test MUST hardcode 0.1 so that a production-side constant change is detected, not silently mirrored); both agents' `AgentState` == `AgentState::Idle` at tick 80; interaction_progress for the pair is either absent from the map OR within 1e-9 of 0.0 (cycle reset confirmed).
- type: A
- rationale: "Phase 7-γ proved the underlying SimResources transition None → 0.1. δ surfaces this through the new collector. Hardcoding 0.1 closes the constant-import circularity vector. The Idle + interaction_progress reset checks confirm the full Idle→Seeking→Consuming→Idle cycle actually completed (not just that the post-state happens to have familiarity=0.1 via some other path)."
- ticks: 80
- components_read: [Agent, AgentState]; resource read: relationships, interaction_progress

### Assertion 18: end_to_end_state_tag_idle_after_interaction
- metric: After the same Phase 7-γ 80-tick run completes (both agents back to Idle per Phase 7-γ A6), the `state_tag` for both agents in `collect_agent_snapshot(&world)`.
- threshold: both state_tag values exactly == 0
- type: A
- rationale: "The tint must turn off when agents return to Idle. A renderer that latches on `state_tag == 2` once and never clears would also pass Assertion 16 — this assertion catches that."
- ticks: 80
- components_read: [Agent, Position, AgentState]

### Assertion 19: locale_compiled_contains_all_seven_keys_en
- metric: After running the localization compile step, parse `localization/compiled/en.json` and for each of the 7 keys (`UI_CAUSAL_REASON_SOCIAL`, `UI_CAUSAL_EVENT_AGENT_DECISION`, `UI_CAUSAL_EVENT_SOCIAL_INTERACTION_STARTED`, `UI_CAUSAL_EVENT_SOCIAL_INTERACTION_COMPLETED`, `UI_AGENT_STATE_SOCIALIZING`, `UI_RELATIONSHIP_PANEL_TITLE`, `UI_RELATIONSHIP_PAIR_ROW`): (presence, value length in characters, count of ASCII alphabetic characters [A-Za-z] in value).
- threshold: all 7 keys present; each value has character length ≥ 3; each value contains at least 2 ASCII alphabetic characters [A-Za-z]
- type: A
- rationale: "§4 locks the locale key set. A missing key means the player sees the raw key string at runtime. The ≥3-char + ≥2-letter threshold defeats a stub commit shipping single-character or punctuation-only values (the Challenger's `\".\"` stub vector)."
- ticks: 0 (filesystem read only)
- components_read: [] (filesystem only)

### Assertion 20: locale_compiled_contains_all_seven_keys_ko
- metric: Same as Assertion 19 but for `localization/compiled/ko.json`. For each of the 7 keys, additionally count Hangul syllable characters in the value (Unicode range U+AC00–U+D7A3).
- threshold: all 7 keys present; each value has character length ≥ 2; each value contains at least 1 Hangul syllable character (U+AC00–U+D7A3); each value differs (byte-wise) from the corresponding en.json value.
- type: A
- rationale: "WorldSim non-negotiable locale rule: every key has en AND ko translations. The Hangul-character requirement is the actual structural constraint a Korean translation must satisfy and defeats the Challenger's `\"。\"`-style single-CJK-punctuation stub (CJK ideographic punctuation lives outside U+AC00–U+D7A3). The byte-differs-from-en check remains as a cheap copy-paste guard."
- ticks: 0
- components_read: [] (filesystem only)

### Assertion 21: locale_seven_keys_pairwise_distinct_en
- metric: For the 7 key values in `localization/compiled/en.json` (same key list as Assertion 19), count how many of the C(7,2)=21 unordered pairs have byte-identical values.
- threshold: exactly == 0 (all 7 values pairwise distinct)
- type: A
- rationale: "Defeats a copy-paste error where the Generator reuses the same English string for two different keys (e.g. `UI_CAUSAL_REASON_SOCIAL` and `UI_AGENT_STATE_SOCIALIZING` both set to `\"Social\"`). Each key has a distinct semantic role per §4 — sharing strings is a translation-quality bug even if technically non-empty."
- ticks: 0
- components_read: [] (filesystem only)

### Assertion 22: agent_snapshot_state_tag_byte_range
- metric: For every row returned by `collect_agent_snapshot(&world)` across a fresh 20-agent stage1 engine after 200 ticks, `row.state_tag`. Additionally, record the set of distinct state_tag values observed across all rows in all 200 ticks (sampled every tick).
- threshold: every row.state_tag ∈ {0, 1, 2, 3}. NOTE: this assertion does NOT require all four values to appear at runtime — stage1 may not exercise ConsumingAgent. Runtime coverage of tags 1, 2, 3 is provided by Assertions 2/3/4 (direct insertion) and Assertion 16 (Consuming{Agent} runtime).
- type: A
- rationale: "Tag domain is fully enumerated in §2-A-1. Any byte outside {0,1,2,3} = an unmapped `AgentState` variant leaking through. Scoping the runtime-coverage expectation explicitly (per Challenger's edge case) prevents an over-broad threshold that fails when stage1 simply doesn't produce socializing agents."
- ticks: 200
- components_read: [Agent, Position, AgentState]

### Assertion 23: state_tag_stream_deterministic_across_two_runs_same_seed
- metric: Build two engines with `make_stage1_engine(42, 20)`. Run each for 100 ticks. At each tick t ∈ [0,100), call `collect_agent_snapshot(&world)` on both engines, sort each row list by entity id, and form a vector of (entity_id, state_tag) pairs. Compare the two engines' vectors tick-by-tick.
- threshold: For every tick t ∈ [0,100), the two engines' (entity_id, state_tag) vectors are byte-identical.
- type: A
- rationale: "The byte projection is NEW code (Phase 9-β A27 covered AgentState determinism, not the projection step). A non-deterministic mapping (e.g. iterating an unordered hashmap inside the collector) would pass A1–A22 in a single-run setting but produce different state_tag orderings across runs. Cheap two-run identity check costs nothing and locks in determinism per Challenger."
- ticks: 100
- components_read: [Agent, Position, AgentState]

## Edge Cases
- `RelationshipKey::new(a, a)` self-pair: if the constructor allows it, `id_a < id_b` (A14) would fail — the test setup must avoid constructing self-pairs. If a Generator's `RelationshipKey::new` rejects self-pairs (recommended), it will surface as a panic/None at insertion in the test, which is a passable signal. Document but do not pin a threshold (behavior is `RelationshipKey`-internal, not the δ collector's concern).
- `familiarity = f64::NAN` or `hostility = f64::NAN`: under `> 0` comparison NaN returns false, so a NaN pair is excluded by §2-A-2's filter. Not asserted (NaN should not enter the map in practice; if a Generator introduces NaN-producing math, that's a separate bug to be caught by upstream Phase 7-γ tests).
- Agent without `AgentState` component: expected to be excluded from `collect_agent_snapshot` (the query is `(Agent, Position, AgentState)`; missing component drops the row). Assertion 5 already pins the row-count integrity.
- Locale file missing at the expected path: Assertions 19/20 must fail with a clear "file not found" message rather than silently passing with an empty key set.
- `RelationshipState::default().hostility` baseline (per Phase 9-β A23) is 0.0 — Assertion 9's filter contract relies on this; if the baseline ever changes, this plan's threshold needs revisiting.
- Concurrent map mutation during snapshot: the collector takes `&resources` (shared borrow); single-threaded harness execution makes this non-issue. The signature alone enforces no `&mut` aliasing during the snapshot call.

## Visual Verification Hints
- During socialization (when two co-located agents are in `Consuming { target: Agent(_) }`), both agents should display a distinct visual marker (green tint or equivalent) in the AgentRenderer that disappears once both return to Idle.
- The CausalPanel should show three new event labels in the chronicle stream for at least one full social interaction cycle: agent decision → social_interaction_started → social_interaction_completed.
- When the relationship debug overlay is toggled on, at least one row should appear after a completed social interaction, displaying both agent ids in canonical order (smaller id first) with non-zero familiarity (~0.1).
- A broken implementation could look like: tint never appearing, tint latching on permanently, tint appearing on non-social Consuming (e.g. eating), or duplicate `(a↔b)` and `(b↔a)` rows in the overlay.

## NOT in Scope
- Visual verification of the green tint on screen (handled by the pipeline VLM step, not the Rust harness).
- CausalPanel GDScript rendering of the three new event labels (VLM step).
- RelationshipState debug panel keybind toggle behavior (VLM step).
- AgentRenderer shader / per-instance custom data choice between `use_colors = true` vs `INSTANCE_CUSTOM.a` (an implementation choice §2-C explicitly leaves to the Generator).
- Phase 8-δ MemoryReason rendering and Phase 9-δ CombatReason rendering (separate dispatches per the feature prompt's "Out of Scope" list).
- Any new `AgentState` variant (e.g. `Socializing`) — the prompt explicitly forbids introducing one.
- FFI marshalling correctness of `PackedByteArray` into Godot (the harness exercises the pure-Rust `collect_*` helpers per the Bridge Identity Contract; the gdext wrapper layer is verified by the pipeline VLM step).
- `RelationshipState`-internal write-time validation (e.g. whether negative familiarity is rejected at the setter): Assertion 13 only pins the collector's read-side behavior. The setter contract is owned by Phase 7-γ / 9-β.
- Phase 9-β dead-agent purge codepath: the original Assertion 17 was dropped (per Challenger, manual map mutation duplicated Assertion 8). The purge codepath itself is verified by Phase 9-β A13; the δ collector's freshness-from-resources contract is implicitly covered by A8/A9 (no caching layer is introduced in §2).

## Feature to Implement

---
feature: p7-delta-social-ui
phase: 7-δ
lane: --full
seed: 42
---

# P7δ — Social UI Integration

## Section 1: Implementation Intent

V7 Phase 7-δ is the Social UI milestone: the first visual evidence that the
Social System (Phase 7-α/β/γ) is alive to the player. Three surfaces get
updated — CausalPanel, AgentRenderer, and a new RelationshipState debug
overlay — plus the supporting SimBridge extensions and locale keys.

**Planning §δ scope** (locked):
- `CausalPanel` SocialReason / SocialInteraction event rendering with
  localized labels.
- Locale keys: `UI_CAUSAL_REASON_SOCIAL`, `UI_CAUSAL_EVENT_SOCIAL_INTERACTION_STARTED`,
  `UI_CAUSAL_EVENT_SOCIAL_INTERACTION_COMPLETED`, `UI_AGENT_STATE_SOCIALIZING`.
- `AgentRenderer`: tint agents whose `AgentState` is `Consuming { Agent(_) }`
  (i.e. currently socializing) with a distinct colour.
- `RelationshipState` UI overlay: a debug panel listing all pairs with
  `familiarity > 0`.

**User mandate**: "current UI described as 'not game-like'". Phase 7-δ is
Stage 1 of the All chain (7-δ → 8-δ → 9-δ) visualisation path. This
dispatch is scoped to Social only; Phase 8-δ (Memory) and Phase 9-δ (Combat)
are separate dispatches.

**Phase 8-δ + 9-δ scope intact**: do NOT add MemoryReason or CombatReason
rendering here.

## Section 2: What to Build

### 2-A: `rust/crates/sim-bridge/src/ffi/world_node.rs`

#### 2-A-1: Extend `AgentSnapshotRow` with `state_tag: u8`

Add field to `AgentSnapshotRow` struct:
```rust
/// 0 = Idle, 1 = Seeking, 2 = ConsumingAgent (socializing), 3 = ConsumingOther
pub state_tag: u8,
```

Update `collect_agent_snapshot` to query `(Agent, Position, AgentState)`:
```rust
pub fn collect_agent_snapshot(world: &hecs::World) -> Vec<AgentSnapshotRow> {
    let mut rows = Vec::new();
    for (entity, (_, pos, state)) in world.query::<(&Agent, &Position, &AgentState)>().iter() {
        let state_tag: u8 = match state {
            AgentState::Idle => 0,
            AgentState::Seeking { .. } => 1,
            AgentState::Consuming { target: TargetKind::Agent(_) } => 2,
            AgentState::Consuming { .. } => 3,
        };
        rows.push(AgentSnapshotRow {
            entity_bits: entity.to_bits().get(),
            x: pos.x,
            y: pos.y,
            state_tag,
        });
    }
    rows
}
```

Update `agent_rows_split` to also return `Vec<u8>` for states.

Update `WorldSimNode::get_agent_snapshot()` to add `states: PackedByteArray`
to the returned `VarDictionary`.

#### 2-A-2: Add `get_relationship_snapshot()` `#[func]`

New `#[func]` method:
```rust
/// P7-δ — return all known relationship pairs as a flat array of Dicts.
/// Each dict: { "id_a": i64, "id_b": i64, "familiarity": f64,
///              "hostility": f64 }
/// Only pairs where familiarity > 0 OR hostility > 0 are returned.
#[func]
fn get_relationship_snapshot(&self) -> VarArray {
    collect_relationship_snapshot(&self.engine.resources)
}
```

Pure-Rust collector (Bridge Identity Contract — same pattern as
`collect_tile_causal_history`):
```rust
pub fn collect_relationship_snapshot(resources: &SimResources) -> Vec<RelationshipSnapshotRow> {
    resources.relationships
        .iter()
        .filter(|(_, v)| v.familiarity > 0.0 || v.hostility > 0.0)
        .map(|(k, v)| RelationshipSnapshotRow {
            id_a: k.0.0 as i64,
            id_b: k.1.0 as i64,
            familiarity: v.familiarity,
            hostility: v.hostility,
        })
        .collect()
}
```

`RelationshipSnapshotRow` struct (add near `AgentSnapshotRow`):
```rust
pub struct RelationshipSnapshotRow {
    pub id_a: i64,
    pub id_b: i64,
    pub familiarity: f64,
    pub hostility: f64,
}
```

Check `SimResources` for the actual field name:
```bash
grep -n "relationships\|HashMap.*Relationship" rust/crates/sim-core/src/sim_resources.rs | head
```
and verify `RelationshipState.hostility` exists:
```bash
grep -n "hostility\|familiarity" rust/crates/sim-core/src/components/relationship.rs | head
```

### 2-B: `scripts/ui/panels/causal_panel.gd`

Add three match arms to `_format_event()` (alongside existing
`building_placed`, `stamp_dirty`, `influence_changed`):

```gdscript
"agent_decision":
    var reason: String = ev.get("reason", "")
    if reason == "social_reason":
        kind_label = _ltr("UI_CAUSAL_REASON_SOCIAL")
    else:
        kind_label = _ltr("UI_CAUSAL_EVENT_AGENT_DECISION")
    var agent_id: int = int(ev.get("agent_id", -1))
    extra = " agent=" + str(agent_id)
"social_interaction_started":
    kind_label = _ltr("UI_CAUSAL_EVENT_SOCIAL_INTERACTION_STARTED")
"social_interaction_completed":
    kind_label = _ltr("UI_CAUSAL_EVENT_SOCIAL_INTERACTION_COMPLETED")
    var fam: float = float(ev.get("familiarity_after", 0.0))
    extra = " fam=" + ("%.2f" % fam)
```

Add `UI_CAUSAL_EVENT_AGENT_DECISION` to locale (generic fallback).

### 2-C: `scripts/ui/agent_renderer.gd`

Extend `_process()` to read `states: PackedByteArray` from snapshot and
apply a distinct tint when `state_tag == 2` (ConsumingAgent = socializing).

Since current `MultiMesh` uses `use_colors = false`, switch to
`use_colors = true` and set per-instance color:
- Default: `Color(1, 1, 1, 0)` (white — no tint change; palette_swap shader
  uses `INSTANCE_CUSTOM.rgb`, not `COLOR`)
- Socializing (state_tag == 2): `Color(0.4, 1.0, 0.6, 0)` (green tint
  applied via `set_instance_color`)

Alternatively, encode state in `custom_data.a` component (currently 0.0)
and update shader to tint on `a > 0.5`. Choose whichever approach compiles
cleanly — the key requirement is that a co-located socializing agent is
visually distinguishable from Idle agents.

Simplest approach (no shader change required):
- Keep `use_colors = false`
- Use `set_instance_custom_data` alpha channel: encode `state_tag` in
  `INSTANCE_CUSTOM.a` (currently unused, always 0.0)
- Update `palette_swap.gdshader` to apply green tint when
  `INSTANCE_CUSTOM.a > 0.5`

OR — if shader change is complex, use `use_colors = true` and
`set_instance_color(i, Color(0.4,1.0,0.6,1))` for state_tag==2, white
otherwise. This requires no shader change.

**Choose the approach that works without breaking existing palette swap
rendering.** Verify `assets/sprites/agent_base.png` and
`shaders/palette_swap.gdshader` are intact.

Snapshot read pattern:
```gdscript
var snap: Dictionary = world_sim.get_agent_snapshot()
var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
var states: PackedByteArray = snap.get("states", PackedByteArray())
```

### 2-D: RelationshipState debug overlay

Add a simple `_relationship_panel` element to `causal_panel.gd` (or as a
sibling Control in the UI scene) that calls
`world_sim.get_relationship_snapshot()` every N ticks and lists pairs:

```
[REL] agent_0 ↔ agent_1  fam=0.30  hos=0.00
[REL] agent_0 ↔ agent_2  fam=0.10  hos=0.10
```

Toggle with `R` key. Title: `_ltr("UI_RELATIONSHIP_PANEL_TITLE")`.

The panel can be a `VBoxContainer` inside a `ColorRect` background, same
style as the existing `CausalPanel`. Limit to top 10 pairs by familiarity.

**New locale keys for this panel**:
- `UI_RELATIONSHIP_PANEL_TITLE` = "Relationships" / "관계"
- `UI_RELATIONSHIP_PAIR_ROW` = "[REL]" / "[관계]"

Since this is a debug panel, position it at bottom-left of screen
(PANEL_MARGIN, screen_height - PANEL_HEIGHT - PANEL_MARGIN).

## Section 3: How to Implement

### 3-A: SimBridge changes (Rust)

**Order of changes**:
1. Add `state_tag: u8` to `AgentSnapshotRow`.
2. Update `collect_agent_snapshot` signature: add `AgentState` to query.
3. Update `agent_rows_split` → `agent_rows_to_parts` (or add `states` Vec).
4. Update `WorldSimNode::get_agent_snapshot` to insert `states` into dict.
5. Add `RelationshipSnapshotRow` struct.
6. Add `collect_relationship_snapshot(resources: &SimResources)` pure-Rust fn.
7. Add `WorldSimNode::get_relationship_snapshot` `#[func]`.

**Required imports** (verify in world_node.rs):
```rust
use sim_core::components::{Agent, AgentState, Position, TargetKind, ...};
use sim_core::sim_resources::SimResources;
use sim_core::components::relationship::{RelationshipKey, RelationshipState};
```

Check the exact import path before writing — use:
```bash
grep -n "^use\|^pub use" rust/crates/sim-bridge/src/ffi/world_node.rs | head -20
grep -rn "pub struct SimResources\|pub relationships" rust/crates/sim-core/src/ | head -5
```

### 3-B: GDScript changes

**`causal_panel.gd`**:
- Add match arms as shown in 2-B. No structural changes needed.

**`agent_renderer.gd`**:
- Add `states` read from snapshot.
- Use `use_colors = true` (change in `_ready`).
- Set `multi_mesh.set_instance_color(i, color)` in the per-agent loop.

**RelationshipState panel** (add to `causal_panel.gd` or separate file):
- If adding to `causal_panel.gd`, add `_rel_panel_visible: bool = false`
  and `_rel_container: VBoxContainer` in `_build_layout`.
- If separate file `scripts/ui/panels/relationship_panel.gd`, follow same
  pattern as `causal_panel.gd`.

### 3-C: Locale

Edit `localization/fluent/en/messages.ftl` and
`localization/fluent/ko/messages.ftl`.

Append at end of each file (following existing `UI_CAUSAL_*` block):

**en**:
```
UI_CAUSAL_REASON_SOCIAL = Social need
UI_CAUSAL_EVENT_AGENT_DECISION = Agent decision
UI_CAUSAL_EVENT_SOCIAL_INTERACTION_STARTED = Interaction started
UI_CAUSAL_EVENT_SOCIAL_INTERACTION_COMPLETED = Interaction completed
UI_AGENT_STATE_SOCIALIZING = Socializing
UI_RELATIONSHIP_PANEL_TITLE = Relationships
UI_RELATIONSHIP_PAIR_ROW = [REL]
```

**ko**:
```
UI_CAUSAL_REASON_SOCIAL = 사회적 필요
UI_CAUSAL_EVENT_AGENT_DECISION = 에이전트 결정
UI_CAUSAL_EVENT_SOCIAL_INTERACTION_STARTED = 교류 시작
UI_CAUSAL_EVENT_SOCIAL_INTERACTION_COMPLETED = 교류 완료
UI_AGENT_STATE_SOCIALIZING = 교류 중
UI_RELATIONSHIP_PANEL_TITLE = 관계
UI_RELATIONSHIP_PAIR_ROW = [관계]
```

After editing FTL files, compile:
```bash
python3 tools/localization_compile.py
```

Verify compiled output:
```bash
grep "UI_CAUSAL_REASON_SOCIAL\|UI_AGENT_STATE_SOCIALIZING" localization/compiled/en.json
```

## Section 4: Locale Summary

| Key | en | ko |
|-----|----|----|
| `UI_CAUSAL_REASON_SOCIAL` | "Social need" | "사회적 필요" |
| `UI_CAUSAL_EVENT_AGENT_DECISION` | "Agent decision" | "에이전트 결정" |
| `UI_CAUSAL_EVENT_SOCIAL_INTERACTION_STARTED` | "Interaction started" | "교류 시작" |
| `UI_CAUSAL_EVENT_SOCIAL_INTERACTION_COMPLETED` | "Interaction completed" | "교류 완료" |
| `UI_AGENT_STATE_SOCIALIZING` | "Socializing" | "교류 중" |
| `UI_RELATIONSHIP_PANEL_TITLE` | "Relationships" | "관계" |
| `UI_RELATIONSHIP_PAIR_ROW` | "[REL]" | "[관계]" |

Keys follow the existing `UI_CAUSAL_*` SCREAMING_SNAKE_CASE convention.
The locale system reads from `localization/compiled/{lang}.json` — the FTL
compile step is **required**.

## Section 5: Verification

### 5-A: Rust gate
```bash
cd rust && cargo build --workspace 2>&1 | tail -5
cd rust && cargo test --workspace 2>&1 | grep "test result"
cd rust && cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -5
```

Expected: all workspace tests pass (Phase 7-α/β/γ + Phase 8-α/β/γ + Phase
9-α/β/γ harness tests remain green). No new clippy warnings.

### 5-B: Harness tests (sim-test)

Write harness assertions in
`rust/crates/sim-test/tests/harness_p7_delta_social_ui.rs`:

**A1** [Type A]: `get_agent_snapshot()` dict contains `"states"` key.
```rust
let snap = collect_agent_snapshot(&engine.world());
// All rows have state_tag field — verify default Idle agents = 0
for row in &snap { assert_eq!(row.state_tag, 0); }
```

**A2** [Type A]: Idle agent → `state_tag == 0`.

**A3** [Type A]: Agent in `Consuming { Agent(_) }` state → `state_tag == 2`.
(Set up a pair running the social interaction loop to completion, then check
state mid-interaction at `progress < REQUIRED_INTERACTION_PROGRESS`.)

**A4** [Type A]: `collect_relationship_snapshot` returns empty when no
relationships exist.

**A5** [Type A]: After `REQUIRED_INTERACTION_PROGRESS` ticks of social
interaction, `collect_relationship_snapshot` returns ≥1 entry with
`familiarity > 0`.

**A6** [Type A]: Locale compiled JSON contains `UI_CAUSAL_REASON_SOCIAL` key.
(Read `localization/compiled/en.json` from test — or skip if filesystem
access is unreliable in test context.)

Total: ≥5 assertions (A1–A5 minimum; A6 optional).

### 5-C: Locale compile
```bash
python3 tools/localization_compile.py
grep "UI_CAUSAL_REASON_SOCIAL" localization/compiled/en.json localization/compiled/ko.json
```

Both files must contain the new keys.

### 5-D: In-game verification (VLM)

The pipeline VLM will verify:
1. Agents in `Consuming { Agent(_) }` state are visually distinct (tinted)
   from `Idle` agents in the same scene.
2. CausalPanel (Q key) shows social event entries when a social interaction
   has occurred on the selected tile.
3. RelationshipState panel (R key) shows `[REL]` entries after agents have
   completed a social interaction.

The Godot harness test script `scripts/test/harness_visual_verify.gd`
drives the visual verification step.

## Section 6: Pipeline Lane

**Lane**: `--full`

Rationale: GDScript UI changes (`.gd`) trigger hot-tier per hook auto-
detection. SimBridge `.rs` changes also trigger review. `--full` is
appropriate for a UI panel/renderer change with FFI extension.

Expected score: ≥85/100 (UI work, VLM hot-tier visual verification).
Adjusted score ≥90 after VLM env cost if needed.

Hook tier: cold-tier signal A (sim-test `.rs`) + GDScript (hot-tier `.gd`)
→ hook takes the most restrictive tier → `--full`.

## Section 7: In-Game Verification Checklist

For the VLM visual analysis step:

1. **AgentRenderer socializing tint**: Spawn 2 agents with `loneliness > 50`
   at the same tile. After ~4-5 ticks, during the 3-tick interaction window,
   both agents should appear with a distinct tint (green or similar) vs idle
   agents.

2. **CausalPanel social events**: Press Q to open CausalPanel. Click on the
   tile where the social interaction occurred. The panel should show:
   - `[T] Social need  agent=0` (AgentDecision with social_reason)
   - `[T] Interaction started`
   - `[T] Interaction completed  fam=0.10`

3. **RelationshipState panel**: Press R to open relationship debug panel.
   After social interaction completes, panel should list:
   - `[REL] 0 ↔ 1  fam=0.10  hos=0.00`

4. **No regressions**: Idle agents (not socializing) render normally with
   palette-swap colours. Non-social causal events (building_placed, etc.)
   still render correctly in CausalPanel.

## Precedent Files

1. `rust/crates/sim-bridge/src/ffi/world_node.rs` — existing SimBridge pattern
   (Bridge Identity Contract, `collect_*` pure-Rust helpers)
2. `scripts/ui/panels/causal_panel.gd` — existing panel pattern to extend
3. `scripts/ui/agent_renderer.gd` — existing MultiMeshInstance2D driver to extend
4. `rust/crates/sim-test/tests/harness_p7_gamma_social_chronicle.rs` — social
   system setup patterns (co-located agents, loneliness threshold)
5. `rust/crates/sim-test/tests/harness_p9_beta_combat_system.rs` — SimBridge
   collect_* test pattern precedent

## Out of Scope

- Phase 8-δ MemoryReason / MemoryRecalled UI (separate dispatch)
- Phase 9-δ CombatReason / CombatStarted UI (separate dispatch)
- Any changes to sim-core components or sim-systems behavior
- AgentState new variants (AgentState::Socializing NOT added — use existing
  `Consuming { Agent(_) }` check)
- New ECS components

## Acceptance

1. `cargo test --workspace` passes (all 898+ prior tests remain green).
2. `cargo clippy --workspace --all-targets -- -D warnings` clean.
3. `python3 tools/localization_compile.py` succeeds; all 7 new locale keys
   present in both `localization/compiled/en.json` and `ko.json`.
4. Harness tests A1–A5 pass.
5. VLM visual analysis shows socializing tint + CausalPanel social events
   + RelationshipState panel entries (or VISUAL_WARNING with env cost
   applied per §7.1 policy).
6. `cargo test -p sim-test harness_p7_delta -- --nocapture`: all assertions
   pass.

## Attempt

This is code attempt 1.
Follow the implementation order and result format defined in your system prompt.

