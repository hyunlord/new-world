# Harness count-guard doctest speedup (Generator-stall root fix)

## Assertions

### Assertion 1: count-guard excludes doctests from enumeration
- metric: Read `rust/crates/sim-test/tests/harness_p8_beta_memory_system.rs`. The
  `harness_p8_beta_a26_test_count_regression_guard` test must invoke
  `cargo test --workspace --lib --bins --tests -- --list` (the `--lib --bins
  --tests` selector excludes doctests, so `rustdoc --test` is NOT invoked and no
  doctest recompilation occurs). The old `cargo test --workspace -- --list`
  (which forced rustdoc doctest enumeration) must be gone.
- threshold: source contains the literal arg sequence
  `"--workspace", "--lib", "--bins", "--tests", "--", "--list"`; the bare
  `"--workspace", "--", "--list"` form is absent. (Type A — exact source token.)
- type: A
- rationale: "ROOT CAUSE of the Generator 900s-timeout stall. The count-guard ran
  a nested `cargo test --workspace -- --list`; `--list` for doctests invokes
  `rustdoc --test` which RECOMPILES every doctest example even on a warm build
  (~91s warm; far worse during the Generator's self-gate when changed crates'
  doctests are rebuilt, ×N TDD cargo invocations → >900s). Excluding doctests
  (`--lib --bins --tests`) cuts the guard from ~91s to ~0.35s and removes the
  doctest-rebuild cost from every `cargo test --workspace`."

### Assertion 2: count-guard still passes (floor preserved, no re-baseline)
- metric: Run `cargo test -p sim-test --test harness_p8_beta_memory_system
  harness_p8_beta_a26_test_count_regression_guard`. It must PASS — the
  `#[test]`-function count (lib+bin+integration, doctests excluded) still meets
  the floor `BASELINE_TEST_COUNT (787) + MIN_NEW_TESTS (17) = 804`.
- threshold: test result ok (1 passed, 0 failed). Observed non-doctest count
  (1630) ≫ floor (804), so excluding doctests neither weakens the silent-removal
  guard nor needs a re-baseline.
- type: A
- rationale: "The floor is conservative; the doctest-excluded count clears it with
  large headroom. Doctest breakage is still caught by the gate EXECUTING them
  (`cargo test --workspace`), just not COUNTED by this guard."

### Assertion 3: full workspace gate is green and fast
- metric: `cargo test --workspace` completes with 0 new failures vs baseline.
- threshold: 0 failures. (Observed ~141s, down from the prior 13–50min count-guard
  pathology — measured, Type C.)
- type: C

## Visual Verification Hints
No visual surface — this is a test-tooling/pipeline-infra change (no `.gd`, no
renderer, no scene). VLM analysis is not meaningful here.

## NOT in Scope
- Game/simulation behaviour (no sim-core/systems/engine logic touched).
- The companion `harness_pipeline.sh` GENERATOR_TIMEOUT_SECONDS 900→1800 raise
  (committed separately as harness infra).
- Re-baselining BASELINE_TEST_COUNT (unnecessary — floor still cleared).
