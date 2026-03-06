# rust/AGENTS.md

## Purpose

- Rust workspace for authoritative simulation, shared data, orchestration, and the Godot bridge.

## Current Boundary

- New simulation logic belongs in Rust.
- Internal dependencies flow downward from `sim-bridge` to lower crates.
- `godot::` types are allowed only in `sim-bridge`.

## Must Follow

- Choose the narrowest crate that matches the responsibility.
- Keep cross-crate changes minimal and intentional.
- When shared interfaces change, update all consumers in the same task.
- Prefer crate-local verification for local changes and workspace verification for shared-interface changes.
- Follow deeper crate `AGENTS.md` files when present.

## Do Not

- Do not put Godot types outside `sim-bridge`.
- Do not create upward/internal dependency cycles.
- Do not move gameplay logic back into GDScript.
- Do not rely on stale CLAUDE examples instead of current code.

## Verification

- `cd rust && cargo test --workspace`
- `cd rust && cargo clippy --workspace -- -D warnings`
