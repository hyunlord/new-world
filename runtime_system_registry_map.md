# Runtime System Registry Map

## Scope
- [rust/crates/sim-bridge/src/runtime_system.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004b/rust/crates/sim-bridge/src/runtime_system.rs)
- [rust/crates/sim-bridge/src/runtime_registry.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004b/rust/crates/sim-bridge/src/runtime_registry.rs)
- [rust/crates/sim-bridge/src/runtime_commands.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004b/rust/crates/sim-bridge/src/runtime_commands.rs)
- [rust/crates/sim-bridge/src/debug_api.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004b/rust/crates/sim-bridge/src/debug_api.rs)
- [scripts/core/simulation/simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004b/scripts/core/simulation/simulation_engine.gd)

## Authority Path
```text
SimulationEngine.init_with_seed()
  -> runtime_init(seed, config_json)
  -> runtime_register_default_systems()
  -> Rust RuntimeSystemId manifest
  -> typed register_runtime_system(...)
  -> SimEngine scheduler
```

## Typed Registration Structures
- `RuntimeSystemId`
  - single authoritative system identity enum
  - stable numeric identity via `#[repr(i32)]`
- `DefaultRuntimeSystemSpec`
  - typed manifest row: `system_id + priority + tick_interval`
- `RuntimeSystemEntry`
  - typed runtime registry snapshot row inside `RuntimeState`
- `register_runtime_system(engine, system_id, priority, tick_interval)`
  - typed scheduler registration entrypoint
- `register_default_runtime_systems(state)`
  - applies the default typed manifest into the runtime

## Registration Order
- Primary sort key: `priority`
- Secondary sort key: `registration_index`
- Final deterministic tie-break: `RuntimeSystemId`

This means registry ordering no longer depends on string key ordering.

## Bridge/Debug Surfaces
- `runtime_get_registry_snapshot()`
  - exposes:
    - `system_id`
    - `name` (display label only)
    - `priority`
    - `tick_interval`
    - `registration_index`
    - `rust_registered`
    - `exec_backend`
- `get_system_perf()`
  - now iterates typed registered systems and looks up perf data by internal engine perf labels
  - exports display labels plus `system_id`

## Legacy/Compatibility Boundary
- Removed as authority:
  - string registration commands
  - `register_system(...)`
  - `runtime_clear_registry`
  - string key normalizers / alias dispatch
- Still present as non-authoritative compatibility:
  - `RuntimeSystemId::perf_label()`
  - these strings exist only to read existing engine perf labels
  - they are not used for scheduler registration, registry sorting, or boot authority

## Current Truth
- Runtime registration authority is typed.
- Boot registration is Rust-owned.
- Registry snapshot identity is numeric `system_id`, with display names layered on top.
- Remaining string labels are diagnostic compatibility, not runtime authority.
