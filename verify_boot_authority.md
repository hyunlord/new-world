# Verify Boot Authority

## Goal

Confirm that Godot boot no longer owns simulation-system registration and that Rust runtime remains the only authoritative simulation tick path.

## Static Checks

### 1. No startup `register_system(...)` calls remain in main boot

```bash
rg -n "register_system\\(" scenes/main/main.gd
```

Expected:

- no hits

### 2. `simulation_engine.gd` no longer clears the runtime registry after init

```bash
rg -n "runtime_clear_registry" scripts/core/simulation/simulation_engine.gd
```

Expected:

- no hits

### 3. Startup uses the Rust default manifest

```bash
rg -n "runtime_register_default_systems" scripts/core/simulation/simulation_engine.gd scripts/core/simulation/sim_bridge.gd rust/crates/sim-bridge/src/lib.rs
```

Expected:

- one startup call in `simulation_engine.gd`
- one wrapper in `sim_bridge.gd`
- one native `#[func]` in `sim-bridge`

### 4. Legacy registry bookkeeping is removed from `simulation_engine.gd`

```bash
rg -n "^[[:space:]]*var[[:space:]]+_systems\\b|_registered_system_payloads|_system_key_by_instance_id|_build_runtime_system_payload|_runtime_system_key_from_name" scripts/core/simulation/simulation_engine.gd
```

Expected:

- no hits

### 5. Boot registry is Rust-backed

```bash
cd rust && cargo test -p sim-bridge default_runtime_manifest_registers_rust_backed_entries -- --nocapture
```

Expected:

- PASS
- manifest entries are marked `exec_backend = "rust"`

## Runtime Verification

### 6. Workspace tests

```bash
cd rust && cargo test --workspace
```

Expected:

- PASS

### 7. Workspace lint

```bash
cd rust && cargo clippy --workspace -- -D warnings
```

Expected:

- PASS

### 8. Godot headless startup smoke

```bash
/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot --headless --path /Users/rexxa/github/new-world-wt/codex-refactor-boot-authority --quit
```

Expected:

- exit code `0`
- existing project warnings may remain
- no new parse/runtime crash from boot refactor

## Runtime Truth Checks

### 9. Rust owns frame stepping

```bash
rg -n "runtime_tick_frame" scripts/core/simulation/simulation_engine.gd scripts/core/simulation/sim_bridge.gd rust/crates/sim-bridge/src/lib.rs
```

Expected:

- GDScript forwards to Rust
- native `runtime_tick_frame` exists in `sim-bridge`

### 10. Boot validates Rust registry rather than rebuilding it

```bash
rg -n "validate_runtime_registry|get_registered_system_count" scenes/main/main.gd scripts/core/simulation/simulation_engine.gd
```

Expected:

- startup performs registry validation
- banner/debug reads registered system count from Rust-backed startup state

## Acceptance Criteria

Boot authority is secure when all are true:

- startup no longer registers runtime systems from GDScript
- startup no longer clears native registry after init
- Rust default manifest populates the runtime registry
- simulation ticks through Rust only
- tests and clippy pass
- Godot headless startup exits successfully

## Residual Legacy Allowed By This Ticket

The following may still exist on disk after this ticket without violating the boot boundary:

- `scripts/systems/**`
- `scripts/ai/behavior_system.gd`
- GDScript shell/bootstrap managers used by UI/save/setup
- `runtime_clear_registry()` wrapper still exposed but unused in boot
