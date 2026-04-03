# Harness Generator Task

You are the GENERATOR. You write harness tests AND implementation code.

## Test Plan (from Planner — DO NOT modify thresholds)

{{PLAN}}

## Feature to Implement

{{FEATURE_PROMPT}}

## Your Task

1. Write harness test(s) in `rust/crates/sim-test/src/main.rs`
   - Follow patterns from `.claude/skills/worldsim-harness/test_templates.md`
   - Use `make_stage1_engine(42, 20)` unless plan specifies otherwise
   - Include Type annotation comment above each `assert!()`
   - Test name: `harness_<category>_<assertion>`

2. Implement the feature
   - Follow the feature prompt exactly
   - All simulation logic in Rust (sim-core, sim-systems, sim-engine)
   - GDScript only for UI rendering (never writes simulation state)
   - SimBridge wrappers for any new Rust→GDScript data

3. Run gate:
   ```bash
   cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings
   ```

4. Run harness:
   ```bash
   cargo test -p sim-test harness_ -- --nocapture
   ```

5. Write result summary to the designated output file:
   ```
   ---
   feature: {{FEATURE}}
   code_attempt: {{CODE_ATTEMPT}}
   ---
   ## Files Changed
   - <file1>: <what changed>
   
   ## Observed Values (seed 42, 20 agents)
   - <metric1>: <value>
   - <metric2>: <value>
   
   ## Gate Result
   cargo test: PASS|FAIL
   clippy: PASS|FAIL
   
   ## Harness Result
   <paste harness test output>
   ```

## Rules
- Do NOT change thresholds from the plan. If a threshold seems wrong, note it in the result but keep the plan's value.
- Do NOT skip writing the harness test and go straight to implementation.
- If the test fails after implementation, debug the implementation — not the test.
{{FEEDBACK}}
