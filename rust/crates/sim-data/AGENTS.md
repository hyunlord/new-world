# rust/crates/sim-data/AGENTS.md

## Purpose

- RON loading, validation, and schema structs for simulation content.

## Current Boundary

- This crate owns parsing, validation, and typed access to data-driven simulation content.
- Phase A-1/A-9 target content includes `MaterialDef`, `FurnitureDef`, `ActionDef`, `RecipeDef`, `StructureDef`, `WorldRuleset`, and `TemperamentRules`.
- Data is immutable after loading.

## Must Follow

- Use `serde` + RON as the default parsing path.
- Keep schema parity with `data/`.
- Use locale keys for display-facing fields, not user-visible text.
- Keep IDs lowercase `snake_case`.
- Use `#[serde(default)]` for optional fields where omission is valid.
- Add or update parsing tests whenever schema changes.
- Prefer tag+threshold selectors over direct material-ID coupling where the content model supports it.
- Preserve zero-code content expansion when the existing schema can express the new material, recipe, or structure.
- Keep World Rules composition, merge priority, and slot data declarative in schema form.
- Keep temperament PRS matrices, bias matrices, and shift rules data-driven.

## Do Not

- Do not mutate loaded data at runtime.
- Do not add Godot types.
- Do not add ad-hoc parsing paths when `serde` is sufficient.
- Do not add new JSON-only assumptions to v3 loaders or schemas.
- Do not change content schema without updating `data/` and tests together.
- Do not bake runtime merge policy or personality outcomes into loader code when the content schema can express them.

## Verification

- `cd rust && cargo test -p sim-data`
- `cd rust && cargo test --test data_loading_test`
