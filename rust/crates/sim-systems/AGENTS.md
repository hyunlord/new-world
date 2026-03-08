# rust/crates/sim-systems/AGENTS.md

## Purpose

- Runtime simulation systems and shared system-side helpers.

## Current Boundary

- This crate owns gameplay behavior and per-tick state transitions.
- Systems communicate through the existing event/deferred-mutation patterns, not direct system calls.
- Follow the actual crate pattern: runtime system structs implementing `SimSystem`.
- v3.1 additions here include material auto-derivation, building GOAP, temperament derivation/shifts, and World Rules application.

## Must Follow

- Follow the existing `RuntimeSystem` + `SimSystem` pattern already used in `src/runtime/`.
- Use shared constants from `sim-core::config`.
- Keep system behavior deterministic.
- Choose priorities and intervals intentionally and explain changes when touched.
- Add or update focused tests in the same file/module you change.
- Use existing event/deferred-mutation patterns instead of inventing direct cross-system wiring.
- Use Influence Grid, tile-grid structure data, and compiled rule sets as the primary interaction surfaces.
- Resolve recipes by tag+threshold selectors, not direct material IDs.
- Apply World Rules on settings compile/init or explicit action events; do not add polling-based refresh.
- Treat temperament changes as causal, event-driven state transitions with clear provenance.

## Do Not

- Do not call one system from another directly.
- Do not add Godot types.
- Do not introduce new string-matching hot paths when enums or typed fields exist.
- Do not rely on the stale free-function examples in old docs over current code.
- Do not treat walls as ECS entities or update structural building state every tick.
- Do not hardcode temperament shifts, oracle outcomes, or rule overrides in hot-path code when data/rules already model them.

## Verification

- `cd rust && cargo test -p sim-systems`
- `cd rust && cargo clippy -p sim-systems -- -D warnings`
