# Runtime Registry Report

## Current state

- Runtime system identity is typed: `RuntimeSystemId`
- Scheduler registration is Rust-owned
- Default manifest is `DEFAULT_RUNTIME_SYSTEMS`
- Godot no longer registers runtime systems itself during boot

## Files

- `rust/crates/sim-bridge/src/runtime_system.rs`
- `rust/crates/sim-bridge/src/runtime_registry.rs`
- `rust/crates/sim-bridge/src/runtime_commands.rs`
- `rust/crates/sim-bridge/src/debug_api.rs`

## Modern pieces

- typed enum-backed registration
- typed default manifest
- eager Rust registration in `register_runtime_system(...)`
- registry snapshot exported to Godot only for debug/UI

## Remaining legacy residue

- human-readable string names are still exported in snapshots/debug/perf surfaces
- perf maps still use string labels because `PerfTracker` records system names by string

## Determinism concerns

- scheduler order is primarily determined by:
  - `priority`
  - `registration_index`
  - then typed `RuntimeSystemId`

This ticket removed the remaining string-name tie-break from active registry sorting.

## Refactor direction

- keep typed `RuntimeSystemId` as the only scheduler identity
- keep strings only as presentation/debug labels
- keep deterministic order based on typed ids, not string key sorting
