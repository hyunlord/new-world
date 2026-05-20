## Quality Review: p7-delta-social-ui (Round 2)

### Challenge Coverage

- Challenge: "A5 'exactly == N' is ambiguous (combined vs separate worlds)"
  Status: ADDRESSED
  Evidence: A5 now reads "In a single test world containing exactly 4 agents..." and "threshold: exactly == 4 (literal)"

- Challenge: "A12 may pass vacuously if Consuming is never reached"
  Status: ADDRESSED
  Evidence: New A16 — "(precondition) such observation tick `t` MUST exist in [0, 80); the test FAILS if no qualifying tick is found within the scan window"

- Challenge: "A13 FAMILIARITY_BUMP import creates circular tautology; cycle completion not asserted"
  Status: ADDRESSED
  Evidence: New A17 — "within 1e-9 of the literal `0.1` (NOT imported from `FAMILIARITY_BUMP` — the test MUST hardcode 0.1...)" and adds Idle + interaction_progress reset checks

- Challenge: "A15 'non-empty string' too weak — `.` passes"
  Status: ADDRESSED
  Evidence: New A19 — "each value has character length ≥ 3; each value contains at least 2 ASCII alphabetic characters [A-Za-z]"

- Challenge: "A16 'differs from en' defeatable by one byte; needs CJK constraint"
  Status: ADDRESSED
  Evidence: New A20 — "each value contains at least 1 Hangul syllable character (U+AC00–U+D7A3)"

- Challenge: "A17 is duplicate of A6 (manual map mutation, not purge codepath)"
  Status: ADDRESSED
  Evidence: "the original Assertion 17 was dropped (per Challenger, manual map mutation duplicated Assertion 8)" in NOT in Scope

- Challenge: "Hardcoded match-table bypass — no tie to live AgentState"
  Status: ADDRESSED
  Evidence: New A7 `state_tag_matches_live_agentstate_same_query` — "Closes the hardcoded-match-table gaming vector"

- Challenge: "Locale stub commit via punctuation"
  Status: ADDRESSED
  Evidence: A19 alphabetic-character count + A20 Hangul syllable check + new A21 pairwise distinctness

- Challenge: "Filter `!=` vs `>` allows negative values"
  Status: ADDRESSED
  Evidence: New A13 `relationship_snapshot_excludes_negative_values` with strict `> 0` rationale

- Challenge: "Missing mixed familiarity AND hostility pair"
  Status: ADDRESSED
  Evidence: New A12 — distinct values 0.1 and 0.05 detect struct-field swap

- Challenge: "Missing Seeking{Food} → 1 assertion"
  Status: ADDRESSED
  Evidence: New A6 `state_tag_seeking_non_agent_target` with threshold == 1

- Challenge: "Missing locale key uniqueness"
  Status: ADDRESSED
  Evidence: New A21 `locale_seven_keys_pairwise_distinct_en`

- Challenge: "Missing determinism of state_tag stream across same-seed runs"
  Status: ADDRESSED
  Evidence: New A23 `state_tag_stream_deterministic_across_two_runs_same_seed` — byte-identical comparison over 100 ticks

- Challenge: "Hostility round-trip precision (A8 mirror missing in A9)"
  Status: ADDRESSED
  Evidence: New A11 — "Pin BOTH fields with 1e-9 tolerance to match A10's precision"

- Challenge: "NaN edge case"
  Status: REBUTTED("NaN excluded by `> 0` semantics; not asserted because NaN-producing math is a separate upstream concern")
  Evidence: Documented in Edge Cases

- Challenge: "Self-pair `RelationshipKey::new(a, a)`"
  Status: REBUTTED("Behavior is RelationshipKey-internal, not δ collector's concern")
  Evidence: Documented in Edge Cases

- Challenge: "Stage1 may not exercise tags 2/3 in A18 200-tick run"
  Status: ADDRESSED
  Evidence: A22 NOTE — "does NOT require all four values to appear at runtime... Runtime coverage of tags 1, 2, 3 is provided by Assertions 2/3/4 (direct insertion) and Assertion 16"

- Challenge: "Concurrent mutation during snapshot"
  Status: ADDRESSED
  Evidence: Edge Cases — "the collector takes `&resources` (shared borrow); single-threaded harness execution makes this non-issue"

### Revision Issues
- No new issues introduced. A7's "expected_tag from §2-A-1 mapping" risks the test itself being a duplicated match table, but this is unavoidable for a mapping invariant and the value comes from the live AgentState read at the same call site — which is the correct anti-circularity guarantee.

### Completeness Check
- All assertions have required fields: YES (metric, threshold, type, rationale, ticks, components_read all present on A1–A23)
- Edge cases listed: YES
- Scope boundary clear: YES (NOT in Scope expanded with negative-value setter contract + dropped A17 rationale)

### Generator-Readiness
- Unambiguous enough for blind implementation: YES. Thresholds are literal (4, 0.1, 0.05, 1e-9, ≥3 chars, ≥2 letters, U+AC00–U+D7A3, byte-identical, exactly == 0). Two different Generators would produce essentially the same test.

### Overall Assessment
The revision systematically addresses every Challenger issue with concrete threshold tightening (literal 0.1, character-class constraints, Hangul range), promotes deferred edge cases into numbered assertions (A6, A12, A13, A21, A23), and closes both identified gaming vectors (live-AgentState tie in A7, vacuous-pass precondition in A16). Two challenges are reasonably rebutted rather than addressed. The plan is implementation-ready.

verdict: PLAN_APPROVED
