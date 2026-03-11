# Verify Runtime Registry

## Static Checks
- `rg -n "system_key|runtime_system_key_from_name|runtime_supports_rust_system|register_system|clear_registry" rust/crates/sim-bridge/src scripts/core/simulation -g '*.rs' -g '*.gd'`
  - Expect: 0 hits
- `rg -n "RuntimeSystemId|DEFAULT_RUNTIME_SYSTEMS|register_runtime_system" rust/crates/sim-bridge/src -g '*.rs'`
  - Expect: typed registry paths present

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
- `"/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot" --headless --path /Users/rexxa/github/new-world-wt/codex-refactor-runtime-registry --quit`
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
- No row should require or expose a `system_key`.

## Determinism Checks
- `DEFAULT_RUNTIME_SYSTEMS` order is unique and deterministic.
- `registered_systems` are sorted by:
  1. `priority`
  2. `registration_index`
  3. `registry_name`

## Acceptance Criteria
- Legacy string system keys: 0
- Runtime boot path registers systems by type only
- Godot cannot re-register systems by string command
- Registry snapshot remains readable for debug/validation
