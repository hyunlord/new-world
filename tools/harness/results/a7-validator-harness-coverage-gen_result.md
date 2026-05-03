---
feature: a7-validator-harness-coverage
code_attempt: 1
---

## Files Changed
- `rust/crates/sim-test/src/main.rs`: Added 7 new A-7 harness tests (~290 lines) inside the existing `tests` module; added private helper `a7_load_production_registry()` that resolves `<workspace>/rust/crates/sim-data/data` via `CARGO_MANIFEST_DIR + ../..`. No production code modified.

## Observed Values (seed 42, 20 agents — registry load is seed-independent)
- Production registry contents: recipes=4, materials=20, furniture=7, structures=5, actions=5
- A1 fatal validator errors on production data: 0
- A2 phantom-tag (`nonexistent_tag_no_material_has_this`) → matched_errors: 1
- A3 impossible-threshold (`min_hardness=999.0` on tag `stone`) → matched_errors: 1
- A4 phantom-tool-tag (`nonexistent_tool_tag_no_material_has`) → matched_errors: 1
- A5 out-of-range material (`hardness=-5.0`) → matched_errors: 1
- A6 unknown-field RecipeDef RON → `ron::de::Error` (deny_unknown_fields enforced)
- A7 baseline recipe count: 4 (≥4 required)

## Threshold Compliance
- Assertion A1 (validator_accepts_production_data): plan=0 fatal & ≥1 recipe/material/structure, observed=0 fatal, 4/20/5, PASS
- Assertion A2 (recipe_with_invalid_tag_rejected): plan=≥1 Severity::Error matching def_id, observed=1, PASS
- Assertion A3 (recipe_with_too_high_threshold_rejected): plan=≥1 Severity::Error matching def_id, observed=1, PASS
- Assertion A4 (action_with_invalid_tool_tag_rejected): plan=≥1 Severity::Error on ActionDef, observed=1, PASS
- Assertion A5 (material_out_of_range_rejected): plan=≥1 Severity::Error on MaterialDef, observed=1, PASS
- Assertion A6 (unknown_field_rejected): plan=ron::de::Error mentioning the unknown field, observed=err contains "unknown" or "typo", PASS
- Assertion A7 (recipe_count_baseline): plan=≥4 recipes loadable, observed=4, PASS

## Gate Result
- cargo test (workspace): PASS (sim-test 326/326 passed, 1 ignored, 0 failed; finished in 236.88s)
- clippy (workspace, -D warnings): PASS (0 warnings)
- harness (`harness_a7` filter): PASS (7/7 passed in 0.02s after build)

## Notes
- Pure test-coverage commit. Per the feature prompt's explicit scope, no production code (`validator.rs`, `registry.rs`, RON files, `RecipeDef` shape) was modified.
- The traditional RED → GREEN cycle does not literally apply here: the validator + load-path already exist and are correct, so the new tests pass on first run. They function as **regression guards** proving the validator catches the documented failure modes on real production data.
- The cycle-detection assertion was intentionally omitted by the plan to avoid polluting production data; the existing inline `validate_recipe_cycles` unit test in `sim-data/src/validator.rs` still covers cycle detection at the unit level.
- Helper `a7_load_production_registry()` keeps the tests `cwd`-independent by traversing from `CARGO_MANIFEST_DIR`.
- No threshold discrepancies: every assertion's observed value sits exactly at or above the plan's lower bound.
- Hook noise observed: several `PostToolUse:Bash` reminders flagged "Command failed" despite exit code 0 from the underlying command — these are false positives, not test or build failures. All gates verified by inspecting the actual output and process exit status.
