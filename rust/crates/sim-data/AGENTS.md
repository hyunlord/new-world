# rust/crates/sim-data/AGENTS.md

## Purpose

- Read-only JSON loading and schema structs for project data.

## Current Boundary

- This crate owns parsing, validation, and typed access to `data/`.
- Data is immutable after loading.

## Must Follow

- Use `serde` as the default parsing path.
- Keep schema parity with `data/`.
- Use locale keys for display-facing fields, not user-visible text.
- Keep IDs lowercase `snake_case`.
- Use `#[serde(default)]` for optional fields where omission is valid.
- Add or update parsing tests whenever schema changes.

## Do Not

- Do not mutate loaded data at runtime.
- Do not add Godot types.
- Do not add ad-hoc parsing paths when `serde` is sufficient.
- Do not change JSON schema without updating `data/` and tests together.

## Verification

- `cd rust && cargo test -p sim-data`
- `cd rust && cargo test --test data_loading_test`
