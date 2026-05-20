```
---
feature: p7-delta-social-ui
plan_attempt: 1
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
- metric: `collect_agent_snapshot(&world).len()` for a world with exactly N agents that all carry (Agent, Position, AgentState).
- threshold: exactly == N where N is the number of agents inserted in the test (e.g. 4 for assertions 1–4 combined)
- type: A
- rationale: "Snapshot must include every agent (Agent + Position + AgentState query). A short row count means the query lost agents — the renderer would silently drop them."
- ticks: 0
- components_read: [Agent, Position, AgentState]

### Assertion 6: relationship_snapshot_empty_when_no_relationships
- metric: Length of the vector returned by `collect_relationship_snapshot(&resources)` on a freshly built engine before any social interaction has occurred (`SimResources::relationships` empty).
- threshold: exactly == 0
- type: A
- rationale: "Filter contract §2-A-2: only pairs with familiarity > 0 OR hostility > 0. Empty map ⇒ empty result. Any non-zero count is a default-value leak (e.g. emitting a stub row)."
- ticks: 0
- components_read: [] (resource-only — `SimResources::relationships`)

### Assertion 7: relationship_snapshot_filters_zero_pairs
- metric: After directly inserting a pair into `resources.relationships` with `familiarity = 0.0` and `hostility = 0.0` (using `RelationshipState::default()` or explicit zero), the length of `collect_relationship_snapshot(&resources)`.
- threshold: exactly == 0
- type: A
- rationale: "Filter contract §2-A-2 explicitly excludes the (0.0, 0.0) case. A pair that was created but never bumped must not be returned — otherwise the debug overlay would surface every default-initialized pair."
- ticks: 0
- components_read: [] (resource-only)

### Assertion 8: relationship_snapshot_includes_familiarity_only_pair
- metric: After inserting a single pair into `resources.relationships` with `familiarity = 0.1`, `hostility = 0.0`, the (length of returned vector, the single row's `familiarity`, the single row's `hostility`).
- threshold: length exactly == 1; row.familiarity within 1e-9 of 0.1; row.hostility within 1e-9 of 0.0
- type: A
- rationale: "Inclusion contract §2-A-2: familiarity > 0 is sufficient. The exact f64 round-trip proves the collector does not lossy-convert (e.g. truncate to f32 mid-pipeline)."
- ticks: 0
- components_read: [] (resource-only)

### Assertion 9: relationship_snapshot_includes_hostility_only_pair
- metric: After inserting a single pair with `familiarity = 0.0`, `hostility = 0.2`, the length of `collect_relationship_snapshot(&resources)` and the single row's `hostility`.
- threshold: length exactly == 1; row.hostility within 1e-9 of 0.2
- type: A
- rationale: "Inclusion contract §2-A-2 uses OR: hostility > 0 is also sufficient. A collector that only checks familiarity (an easy mistake when adapting from a previous familiarity-only design) would drop this row."
- ticks: 0
- components_read: [] (resource-only)

### Assertion 10: relationship_snapshot_id_a_lt_id_b_canonical
- metric: For every row returned by `collect_relationship_snapshot(&resources)` after seeding pairs via `RelationshipKey::new(...)` in both `(a, b)` and `(b, a)` orders, the pair `(row.id_a, row.id_b)`.
- threshold: For every row, `row.id_a < row.id_b` (strict ordering)
- type: A
- rationale: "`RelationshipKey::new` is canonical (smaller id first — verified by Phase 7-γ A17). Exposing rows in non-canonical order would make the debug overlay print duplicate `0↔1` and `1↔0` rows for the same pair."
- ticks: 0
- components_read: [] (resource-only)

### Assertion 11: relationship_snapshot_one_row_per_pair_after_two_inserts_same_pair
- metric: Insert the same logical pair twice via `RelationshipKey::new(a, b)` and `RelationshipKey::new(b, a)` (both with familiarity = 0.1). Count of rows returned by `collect_relationship_snapshot(&resources)`.
- threshold: exactly == 1
- type: A
- rationale: "Canonical key deduplication is a `RelationshipKey` invariant — both inserts must collapse into one map entry, therefore one row. A count of 2 means the collector or store is not using the canonical key."
- ticks: 0
- components_read: [] (resource-only)

### Assertion 12: end_to_end_socializing_pair_produces_snapshot_state_tag_2
- metric: Run the Phase 7-γ social interaction scenario (2 co-located agents at (6,5), `Social::new(0.0, 1.0)`, all other needs growth_rate=0.0). At the first tick `t` where the `AgentState` of agent_1 (read via direct world query) is `Consuming { target: TargetKind::Agent(_) }` AND interaction_progress is strictly less than `REQUIRED_INTERACTION_PROGRESS`, the value of `state_tag` for both agents in `collect_agent_snapshot(&world)`.
- threshold: both state_tag values exactly == 2 at that observation tick
- type: A
- rationale: "End-to-end: the renderer-facing tag must agree with the live `AgentState` for the exact frames the player would see the tint. If state_tag drifts from AgentState at the snapshot boundary, the renderer tints the wrong agents."
- ticks: Up to 80 (Phase 7-γ N_TICKS); stop scanning at the first qualifying observation tick.
- components_read: [Agent, Position, AgentState, Social]; resource read: interaction_progress

### Assertion 13: end_to_end_relationship_snapshot_after_completed_interaction
- metric: After running the same Phase 7-γ scenario for the full 80 ticks (mutual Idle→Seeking→Consuming→Idle cycle completes once), `collect_relationship_snapshot(&resources)` filtered to the (id_1, id_2) pair: (length, the row's familiarity).
- threshold: there exists exactly 1 row matching `{id_a, id_b} == {id_1, id_2}` (as a set); that row's `familiarity` is within 1e-9 of `FAMILIARITY_BUMP` (0.1)
- type: A
- rationale: "Phase 7-γ proved the underlying SimResources transition None → 0.1 (A0 + A7). δ surfaces this through the new collector. If the collector reports a different value, it has injected/transformed data instead of mirroring SimResources verbatim — Bridge Identity Contract violation."
- ticks: 80
- components_read: [Agent]; resource read: relationships

### Assertion 14: end_to_end_state_tag_idle_after_interaction
- metric: After the same Phase 7-γ 80-tick run completes (both agents back to Idle per Phase 7-γ A6), the `state_tag` for both agents in `collect_agent_snapshot(&world)`.
- threshold: both state_tag values exactly == 0
- type: A
- rationale: "The tint must turn off when agents return to Idle. A renderer that latches on `state_tag == 2` once and never clears would also pass Assertion 12 — this assertion catches that."
- ticks: 80
- components_read: [Agent, Position, AgentState]

### Assertion 15: locale_compiled_contains_all_seven_keys_en
- metric: After running the localization compile step, parse `localization/compiled/en.json` and check for presence of each key string: `UI_CAUSAL_REASON_SOCIAL`, `UI_CAUSAL_EVENT_AGENT_DECISION`, `UI_CAUSAL_EVENT_SOCIAL_INTERACTION_STARTED`, `UI_CAUSAL_EVENT_SOCIAL_INTERACTION_COMPLETED`, `UI_AGENT_STATE_SOCIALIZING`, `UI_RELATIONSHIP_PANEL_TITLE`, `UI_RELATIONSHIP_PAIR_ROW`.
- threshold: all 7 keys present (each lookup returns Some / not-None) AND each value is a non-empty string
- type: A
- rationale: "§4 locks the locale key set. A missing key means the GDScript layer falls back to the raw key string at runtime — the player sees `UI_CAUSAL_REASON_SOCIAL` instead of `Social need`. Non-empty check defeats a stub commit that registers the key with an empty value."
- ticks: 0 (filesystem read only)
- components_read: [] (filesystem only)

### Assertion 16: locale_compiled_contains_all_seven_keys_ko
- metric: Same as Assertion 15 but for `localization/compiled/ko.json`.
- threshold: all 7 keys present AND each value is a non-empty string that differs from the en value for that key (rules out a copy-paste that ships English text under the Korean locale)
- type: A
- rationale: "WorldSim non-negotiable locale rule: every key has en AND ko translations. The differ-from-en check is the cheapest available guard against an untranslated key landing in ko.json."
- ticks: 0
- components_read: [] (filesystem only)

### Assertion 17: relationship_snapshot_no_dead_agent_leak
- metric: Insert one pair (id_a=1, id_b=2) with familiarity=0.1. Then remove that pair from `resources.relationships` (simulating the Phase 9-β A13 dead-defender purge). Length of `collect_relationship_snapshot(&resources)`.
- threshold: exactly == 0
- type: A
- rationale: "The collector must reflect SimResources state at call time — no caching, no stale rows. Phase 9-β established that dead agents are purged from `relationships`; the δ collector must respect that purge so the debug overlay does not display ghost relationships."
- ticks: 0
- components_read: [] (resource-only)

### Assertion 18: agent_snapshot_state_tag_byte_range
- metric: For every row returned by `collect_agent_snapshot(&world)` across a fresh 20-agent stage1 engine after 200 ticks, `row.state_tag`.
- threshold: every row.state_tag ∈ {0, 1, 2, 3}
- type: A
- rationale: "Tag domain is fully enumerated in §2-A-1 (Idle/Seeking/ConsumingAgent/ConsumingOther). Any other byte value = an unmapped `AgentState` variant leaking through — the renderer would silently treat it as Idle and the tint logic would diverge from sim reality."
- ticks: 200
- components_read: [Agent, Position, AgentState]

## Edge Cases
- Mixed familiarity / hostility pair (familiarity=0.1 AND hostility=0.05): must appear exactly once in `collect_relationship_snapshot` with both fields preserved within 1e-9.
- `AgentState::Seeking { target: TargetKind::Food }` (Seeking, but a non-Agent target): expected `state_tag == 1` (the §2-A-1 mapping does not branch on Seeking's TargetKind — Seeking maps to 1 regardless). Document the chosen behavior; do NOT assert a value the planner cannot derive from §2-A-1.
- Agent without `AgentState` component: expected to not be included in `collect_agent_snapshot` (the query is `(Agent, Position, AgentState)` — missing component drops the row). Assertion 5 already covers row count integrity.
- Locale file missing at the expected path: Assertions 15/16 must fail with a clear "file not found" message rather than silently passing with an empty key set.
- `RelationshipState::default().hostility` baseline (per Phase 9-β A23) is 0.0 — Assertion 7's filter contract relies on this; if the baseline ever changes, this plan's threshold needs revisiting.

## NOT in Scope
- Visual verification of the green tint on screen (handled by the pipeline VLM step, not the Rust harness).
- CausalPanel GDScript rendering of the three new event labels (VLM step).
- RelationshipState debug panel keybind toggle behavior (VLM step).
- AgentRenderer shader / per-instance custom data choice between `use_colors = true` vs `INSTANCE_CUSTOM.a` (an implementation choice §2-C explicitly leaves to the Generator).
- Phase 8-δ MemoryReason rendering and Phase 9-δ CombatReason rendering (separate dispatches per the feature prompt's "Out of Scope" list).
- Any new `AgentState` variant (e.g. `Socializing`) — the prompt explicitly forbids introducing one.
- Determinism of the state_tag byte sequence across seeds (the existing Phase 9-β A27 determinism guard already protects the underlying `AgentState` stream; the byte projection adds no new nondeterministic input).
- FFI marshalling correctness of `PackedByteArray` into Godot (the harness exercises the pure-Rust `collect_*` helpers per the Bridge Identity Contract; the gdext wrapper layer is verified by the pipeline VLM step).
```
