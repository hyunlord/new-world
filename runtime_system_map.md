# Runtime System Map

## Scope
- Rust runtime registry: [rust/crates/sim-bridge/src/runtime_registry.rs](/Users/rexxa/github/new-world-wt/codex-refactor-runtime-registry/rust/crates/sim-bridge/src/runtime_registry.rs)
- Typed system manifest: [rust/crates/sim-bridge/src/runtime_system.rs](/Users/rexxa/github/new-world-wt/codex-refactor-runtime-registry/rust/crates/sim-bridge/src/runtime_system.rs)
- Runtime command boundary: [rust/crates/sim-bridge/src/runtime_commands.rs](/Users/rexxa/github/new-world-wt/codex-refactor-runtime-registry/rust/crates/sim-bridge/src/runtime_commands.rs)
- Debug snapshot/perf readers: [rust/crates/sim-bridge/src/debug_api.rs](/Users/rexxa/github/new-world-wt/codex-refactor-runtime-registry/rust/crates/sim-bridge/src/debug_api.rs)
- Godot wrapper entry: [scripts/core/simulation/sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-refactor-runtime-registry/scripts/core/simulation/sim_bridge.gd)

## Current Registry Authority
- Authoritative manifest: `DEFAULT_RUNTIME_SYSTEMS` in `runtime_system.rs`
- Authoritative identity: `RuntimeSystemId`
- Scheduler registration path:
  - `runtime_register_default_systems()`
  - `register_default_runtime_systems()`
  - `upsert_runtime_system_entry()`
  - `register_runtime_system()`
  - `SimEngine::register(...)`

## Typed Runtime Systems
- Registry identity is no longer derived from script paths or string keys.
- `RuntimeSystemId` holds the stable typed identity for each Rust-backed runtime system.
- `registry_name()` remains as a debug/display label only.

## Legacy vs Modern Classification

### Modern
- Typed manifest bootstrap via `RuntimeSystemId`
- Deterministic scheduler ordering by `priority`, `registration_index`, `registry_name`
- Rust-only default registration from `runtime_register_default_systems`

### Legacy Removed
- String key normalization from script names
- String dispatcher for `register_system`
- `system_key` field in runtime registry entries
- GDScript boot-time registry ownership
- `runtime_clear_registry` compatibility hook

### Bridge/Display Retained
- Registry snapshot still exports:
  - `name`
  - `system_id`
  - `priority`
  - `tick_interval`
  - `active`
  - `registration_index`
  - `rust_implemented`
  - `rust_registered`
  - `exec_backend`
- These are display/debug fields, not scheduler authority.

## Deterministic Registration Order
- Source of truth: `DEFAULT_RUNTIME_SYSTEMS`
- Pre-registered async systems in `RuntimeState::from_seed()`:
  - `LlmResponse`
  - `LlmTimeout`
  - `StorySifter`
  - `LlmRequest`
- Full runtime manifest is then upserted in deterministic order.

## Boot Flow
1. Godot calls `runtime_init`
2. Rust builds `RuntimeState`
3. Godot calls `runtime_register_default_systems`
4. Rust registers typed systems from `DEFAULT_RUNTIME_SYSTEMS`
5. Godot only validates snapshot shape and consumes state

## Remaining Boundary
- Godot may read registry/debug metadata
- Godot may adjust speed/compute-domain modes
- Godot no longer registers runtime systems by string name
- Scheduler authority lives in Rust only
