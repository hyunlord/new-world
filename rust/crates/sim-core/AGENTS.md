# rust/crates/sim-core/AGENTS.md

## Purpose

- Foundation schema crate for shared component data, world data, IDs, enums, and config.

## Current Boundary

- Treat this crate as schema: changes here affect every other Rust crate and parts of the Godot bridge.
- Components are shared data containers, not behavior owners.

## Must Follow

- Treat public/shared types as compatibility-sensitive schema.
- Keep components plain data with no side-effectful methods.
- Keep Godot-specific types out of this crate.
- Prefer enums over string categories in shared types.
- Put new shared constants in `config.rs`.
- For new simulation math, prefer `f64`; do not change existing public field types without an explicit schema-migration need.

## Do Not

- Do not import from higher crates such as `sim-systems`, `sim-engine`, or `sim-bridge`.
- Do not change field types casually.
- Do not add Godot-facing conversion concerns here.
- Do not hide magic numbers in downstream crates when they belong in shared config.

## Verification

- `cd rust && cargo test -p sim-core`
- `cd rust && cargo clippy -p sim-core -- -D warnings`
