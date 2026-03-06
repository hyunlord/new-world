# rust/crates/sim-systems/AGENTS.md

## Purpose

- Runtime simulation systems and shared system-side helpers.

## Current Boundary

- This crate owns gameplay behavior and per-tick state transitions.
- Systems communicate through the existing event/deferred-mutation patterns, not direct system calls.
- Follow the actual crate pattern: runtime system structs implementing `SimSystem`.

## Must Follow

- Follow the existing `RuntimeSystem` + `SimSystem` pattern already used in `src/runtime/`.
- Use shared constants from `sim-core::config`.
- Keep system behavior deterministic.
- Choose priorities and intervals intentionally and explain changes when touched.
- Add or update focused tests in the same file/module you change.
- Use existing event/deferred-mutation patterns instead of inventing direct cross-system wiring.

## Do Not

- Do not call one system from another directly.
- Do not add Godot types.
- Do not introduce new string-matching hot paths when enums or typed fields exist.
- Do not rely on the stale free-function examples in old docs over current code.

## Verification

- `cd rust && cargo test -p sim-systems`
- `cd rust && cargo clippy -p sim-systems -- -D warnings`
