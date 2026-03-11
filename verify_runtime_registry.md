# Verify Runtime Registry

## Static Checks
- `rg -n "system_key|runtime_system_key_from_name|runtime_supports_rust_system|register_system|clear_registry|registry_name\\(" rust/crates/sim-bridge/src scripts/core/simulation tests -g '*.rs' -g '*.gd'`
  - Expect: 0 hits
- `rg -n "RuntimeSystemId|DEFAULT_RUNTIME_SYSTEMS|register_runtime_system|display_label|perf_label" rust/crates/sim-bridge/src -g '*.rs'`
  - Expect: typed registry paths present
- `rg -n "runtime_register_default_systems" tests/test_stage1.gd scripts/core/simulation/simulation_engine.gd scripts/core/simulation/sim_bridge.gd`
  - Expect: typed default registration path is used by both boot and headless harnesses

## Rust Verification
- `cd rust && cargo build -p sim-bridge`
  - Expect: PASS
- `cd rust && cargo test -p sim-bridge`
  - Expect: PASS
- `cd rust && cargo test --workspace`
  - Expect: PASS
- `cd rust && cargo clippy --workspace -- -D warnings`
  - Expect: PASS

## Godot Boundary Verification
- `"/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot" --headless --path /Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004b --quit`
  - Expect: exit code 0

## Runtime Snapshot Expectations
- `runtime_get_registry_snapshot()` rows include:
  - `name`
  - `system_id`
  - `priority`
  - `tick_interval`
  - `active`
  - `registration_index`
  - `rust_implemented = true`
  - `rust_registered = true`
  - `exec_backend = "rust"`
- No row should require or expose a legacy string registration key.
- `name` is a display label only; `system_id` is the authoritative identity.

## Determinism Checks
- `DEFAULT_RUNTIME_SYSTEMS` order is unique and deterministic.
- `DEFAULT_RUNTIME_SYSTEMS.len() == RuntimeSystemId::all().len()`
- `registered_systems` are sorted by:
  1. `priority`
  2. `registration_index`
  3. `RuntimeSystemId`

## Acceptance Criteria
- Legacy string registration keys: 0
- Runtime boot path registers systems by type only
- The typed default manifest covers every `RuntimeSystemId`
- Godot cannot re-register systems by string command
- Registry snapshot remains readable for debug/validation
- Remaining string `perf_label()` values are compatibility labels only, not registry identity
