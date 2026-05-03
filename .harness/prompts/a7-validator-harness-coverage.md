# Feature: a7-validator-harness-coverage

## Summary

A-7 Tag+Threshold Recipe System was 100% implemented (RecipeDef +
TagRequirement + RecipeRequires + 7 validator functions + crafting
integration + Material/Furniture tags). `DataRegistry::load_from_directory`
already calls `validate_registry` at line 124 and aborts with `Err` on any
`Severity::Error`. Inline unit tests existed for 2 of the validator paths.

What was missing: production-level harness coverage proving the validator
actually catches malformed data. This commit adds 7 harness tests that
exercise the validator on production data and on synthetic malformed
registries.

## Changes Made

### rust/crates/sim-test/src/main.rs (+~280 lines)

7 new harness tests, no production code changed:

- `harness_a7_validator_accepts_production_data` (A1) — production directory
  loads with zero `Severity::Error` and at least 1 recipe / material /
  structure (currently: 4 recipes / 20 materials / 7 furniture / 5 structures
  / 5 actions).
- `harness_a7_recipe_with_invalid_tag_rejected` (A2) — recipe with a tag no
  material defines → `Severity::Error` from `validate_recipe_tags`.
- `harness_a7_recipe_with_too_high_threshold_rejected` (A3) — recipe with
  `min_hardness: 999.0` → 0 matches → `Severity::Error`.
- `harness_a7_action_with_invalid_tool_tag_rejected` (A4) — action with a
  bogus `tool_tag` → `Severity::Error` from `validate_action_tool_tags`.
- `harness_a7_material_out_of_range_rejected` (A5) — material with
  `hardness: -5.0` → `Severity::Error` from `validate_material_ranges`.
- `harness_a7_unknown_field_rejected` (A6) — RecipeDef RON with a typo'd
  field name → `ron::de::Error` (proves `#[serde(deny_unknown_fields)]`).
- `harness_a7_recipe_count_baseline` (A7) — at least the 4 production
  recipes from `stone_tools.ron + basic_crafting.ron` remain loadable
  (regression guard).

Helper: `a7_load_production_registry()` loads `<workspace>/rust/crates/
sim-data/data` via `CARGO_MANIFEST_DIR` + `..` traversal — keeps tests
independent of `cwd`.

## Scope

- 0 production-code changes (validator + load path unchanged)
- 0 new RON content (Phase 1 grep showed `bone` material + `needle`
  template are absent → bonus recipe omitted to avoid validator rejection)
- 0 changes to `validate_registry` / 7 sub-validators
- DataRegistry shape unchanged
- Cycle-detection test omitted (would require constructing matched
  Material+Recipe pairs that produce a deterministic cycle without
  polluting production data; the existing inline `validate_recipe_cycles`
  test already covers it at the unit level)

## Verification

- `cargo test -p sim-test --bin sim-test harness_a7 -- --nocapture` →
  7/7 PASS
- `cargo clippy --workspace -- -D warnings` → clean
- Production registry confirmed valid: 4 recipes / 20 materials /
  7 furniture / 5 structures / 5 actions / 0 fatal validator errors

## Roadmap v4 Status

| Prereq | State |
|--------|-------|
| A-3 Effect Primitive | DONE |
| A-4 Causal Tracking | DONE |
| A-5 System Frequency Tiering | DONE |
| A-6 Room BFS | DONE |
| **A-7 Tag+Threshold Recipes** | **DONE (this feature — production-level proof)** |
| A-8 Temperament | DONE |

6/13 → 7/13 prerequisite items complete.
