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
