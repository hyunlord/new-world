# Plan Quality Review — p7-delta-social-ui

Review the following three documents and issue your verdict.

## 1. Original Plan Draft

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

## 2. Challenger's Attack

## Challenge Report: p7-delta-social-ui

### Threshold Issues

- [OK] Assertion 1: Locked Idle=0 invariant, ticks=0, no sim dependency — clean Type A.
- [OK] Assertion 2: Seeking=1 invariant verifiable directly via component insertion.
- [OK] Assertion 3: ConsumingAgent=2, clearly distinguishable from non-Agent Consuming.
- [OK] Assertion 4: ConsumingOther=3, disambiguation test is sound.
- [ISSUE] Assertion 5: "exactly == N where N is the number of agents inserted" — N is parameterized in the rationale but the threshold reads "(e.g. 4 for assertions 1–4 combined)". This is ambiguous. Is the test forming a single combined world of 4 agents, or one world per assertion? If "combined", you depend on side-effect-free re-insertion. State the exact N as a literal in the threshold.
- [OK] Assertion 6: Empty-map → empty-vec is a clean default check.
- [OK] Assertion 7: Filter contract on zero pair is essential and well-specified.
- [OK] Assertion 8: f64 round-trip via 1e-9 catches f32 truncation paths.
- [OK] Assertion 9: Hostility-only OR-branch check — catches the most likely implementation regression.
- [OK] Assertion 10: Canonical ordering invariant covered.
- [OK] Assertion 11: Deduplication contract well-tested.
- [ISSUE] Assertion 12: "the first tick t where AgentState is Consuming { Agent(_) } AND interaction_progress < REQUIRED_INTERACTION_PROGRESS". The plan does not assert that such a tick exists. If the scenario fails to ever enter Consuming within 80 ticks (e.g. seeking phase misbehaves), the test has no observation point. Threshold needs a precondition assertion: "such a tick t MUST exist in [0..80); otherwise FAIL". Otherwise an implementation that never reaches Consuming silently passes (vacuously true / scan loop falls off the end).
- [ISSUE] Assertion 13: Threshold says "within 1e-9 of FAMILIARITY_BUMP (0.1)". If the harness imports `FAMILIARITY_BUMP` from sim-systems instead of using the literal `0.1`, this assertion becomes circular — changing the constant in production code would change the test's expected value in lockstep, never failing. Pin the literal `0.1`. Additionally, A13 does not assert that the full Idle→Seeking→Consuming→Idle cycle actually completed; it only asserts the post-state. Add an assertion that interaction_progress reset to 0 and both agents are back in Idle at tick 80 (currently only A14 covers Idle).
- [OK] Assertion 14: Latch-on tint regression catcher — useful.
- [ISSUE] Assertion 15: "each value is a non-empty string" is a very weak guard. A Generator could ship `{"UI_CAUSAL_REASON_SOCIAL": "x"}` and pass. Threshold should require minimum length (e.g. ≥3 chars) or that the value contains at least one alphabetic character — or better, that values match expected reference text snippets.
- [ISSUE] Assertion 16: "differs from the en value" is defeatable by appending a single character (e.g. `"Social need."` vs `"Social need"`). Strengthen to: value contains at least one CJK character (Hangul range U+AC00–U+D7A3) — that's the actual constraint a Korean translation must satisfy.
- [ISSUE] Assertion 17: This test inserts a pair, then removes it, then asserts empty result. This is identical in effect to Assertion 6 (empty map → empty vec). The "dead-defender purge" framing is rhetorical — the test does not exercise any purge code, only manual map mutation. Either invoke the actual purge codepath, or admit this is a duplicate of A6 and drop it.
- [OK] Assertion 18: Byte-range enumeration over 200 ticks is exactly the right invariant to catch unmapped-variant leaks.

### Gaming Vectors

- **Hardcoded match-table bypass**: A Generator can satisfy A1–A4 + A12 + A14 + A18 trivially with `match state { Idle => 0, Seeking{..} => 1, Consuming{target: TargetKind::Agent(_)} => 2, Consuming{..} => 3 }`. There is no test that verifies the tag derives from *the same* `AgentState` the rest of the sim observes. A generator could compute state_tag from a *stale cached* AgentState (e.g. previous tick's value) and still pass every assertion because none of them sample mid-tick. Add: state_tag must agree with `world.get::<AgentState>(entity)` read in the same query iteration.
- **Locale stub commit**: A15/A16 accept any non-empty value. A generator can write `{"UI_AGENT_STATE_SOCIALIZING": "."}` in en and `"。"` in ko (different by one byte). Both pass.
- **Filter-OR-short-circuit cheat**: A8 and A9 are tested separately. A Generator implementing `if familiarity > 0.0 { include }` would fail A9 — good. But a Generator implementing `if hostility > 0.0 || familiarity > 0.0` AND also `if familiarity > 0.0 || hostility > 0.0` (i.e. correct) passes — fine. However, a Generator implementing `if familiarity != 0.0 || hostility != 0.0` (using `!=` instead of `>`) would pass all tests but include rows with *negative* familiarity. Add a negative-value edge case.
- **FAMILIARITY_BUMP constant reuse (A13)**: If the harness imports the constant, the test becomes a tautology. Lock the literal.
- **Vacuous A12 pass**: If the scenario never reaches `Consuming`, the scan loop completes without observing any qualifying tick. Depending on test phrasing, this may pass vacuously.

### Missing Assertions

- **Mixed familiarity AND hostility pair** (familiarity=0.1, hostility=0.05) is listed under Edge Cases but **never asserted**. This is the most likely real-world configuration and the most likely place a struct-field swap (familiarity↔hostility) would manifest. Promote to a numbered assertion.
- **`Seeking { target: TargetKind::Food }` → state_tag == 1**: listed in Edge Cases as "document the chosen behavior; do NOT assert" — but this IS a derivable invariant from §2-A-1 ("Seeking maps to 1 regardless"). Not asserting it leaves a Generator free to ship `Seeking { Food } → 0` and pass.
- **Negative familiarity / hostility filter behavior**: if the underlying type permits negative values (it might, given f64), what does the filter do? `> 0` excludes them, but no test pins this.
- **state_tag derives from current-tick AgentState**: no assertion verifies the snapshot reads AgentState live vs from a cached copy. A renderer that lags by one tick on social tinting would visibly flicker but pass all current tests.
- **Locale key uniqueness**: nothing prevents two of the 7 keys from sharing the same value (a copy-paste error). Worth adding: all 7 values in en.json are pairwise distinct.
- **Determinism of state_tag stream across two same-seed runs**: deferred as "out of scope" citing Phase 9-β A27 — but A27 covered AgentState, not the new byte projection. The byte projection is new code; the cheap two-run identity check costs nothing.
- **Hostility round-trip precision** (A8 mirror for hostility): A8 pins familiarity to 1e-9; the equivalent hostility-precision assertion is absent (A9 checks presence and value but the threshold language is looser).

### Edge Cases Not Covered

- Pair with `familiarity = f64::NAN` or hostility = NAN: filter behavior undefined under `> 0` comparison (NaN comparisons return false). Worth pinning.
- Self-relationship pair `RelationshipKey::new(a, a)`: does the canonical key allow it? If yes, `id_a < id_b` (A10) fails. If forbidden, no test verifies the rejection.
- 200-tick run in A18 uses a stage1 engine — does stage1 actually produce any Consuming states? If stage1 agents never socialize, A18 only ever observes 0 and 1, leaving 2 and 3 untested at the runtime-stream level. The plan should verify the 200-tick run actually exercises all 4 tag values, or use a different scenario.
- Pair removed mid-snapshot (concurrency): not relevant if single-threaded, but worth confirming the collector takes `&resources` and not `&mut`.

### Overall Assessment

The plan has good coverage on the unit-level mapping and filter contracts, but several assertions are weakly-thresholded (A15, A16, A17), one is potentially vacuous (A12), one is potentially circular (A13), and two of the most interesting edge cases (mixed pair, Seeking-Food) are deliberately left unasserted despite being derivable invariants. The byte-projection logic is also gameable with a hardcoded match table because no assertion ties the tag to live AgentState.

challenge_verdict: MAJOR_ISSUES

## 3. Drafter's Revised Plan

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

## Instructions

Follow the checks defined in your system prompt:
A. Challenge Coverage — check every Challenger issue
B. Revision Quality — look for new problems
C. Completeness — verify all assertion fields
D. Generator-Readiness — check for ambiguity

End with your verdict: PLAN_APPROVED, PLAN_REVISE, or PLAN_FAIL.
