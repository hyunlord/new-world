# rust/crates/sim-core/AGENTS.md

## Purpose

- Foundation schema crate for shared component data, world data, IDs, enums, config, temperament state, and structural tile data.

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
- Keep structural building state in tile/grid data; walls, floors, and roofs are not ECS entities.
- Keep temperament data as shared state only: genes, TCI axes, latent/expressed values, and display labels live here, not decision logic.
- Shared types that cross crate boundaries should remain serde-friendly and deterministic.

## Do Not

- Do not import from higher crates such as `sim-systems`, `sim-engine`, or `sim-bridge`.
- Do not change field types casually.
- Do not add Godot-facing conversion concerns here.
- Do not hide magic numbers in downstream crates when they belong in shared config.
- Do not move rule composition, recipe resolution, or oracle interpretation logic into schema types.

## Verification

- `cd rust && cargo test -p sim-core`
- `cd rust && cargo clippy -p sim-core -- -D warnings`
