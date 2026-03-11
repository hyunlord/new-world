# Verify Boot Authority

## Static Checks

### No boot-time legacy system registration remains
```bash
rg -n "register_system\\(" scenes/main/main.gd scripts/core/simulation/simulation_engine.gd
# expect: 0 hits
```

### No boot-time registry clearing remains
```bash
rg -n "runtime_clear_registry" scripts/core/simulation
# expect: 0 hits
```

### Rust default registration path is active
```bash
rg -n "runtime_register_default_systems" \
  scenes/main/main.gd \
  scripts/core/simulation/simulation_engine.gd \
  scripts/core/simulation/sim_bridge.gd \
  rust/crates/sim-bridge/src
# expect: hits in simulation_engine.gd, sim_bridge.gd, and sim-bridge Rust code
```

### `main.gd` no longer preloads simulation systems
```bash
rg -n 'const .*preload\\("res://scripts/(systems|ai)/' scenes/main/main.gd
# expect: 0 hits
```

## Boot Truth Checks

### Rust loads the registry during startup
```bash
rg -n "Authoritative RON registry loaded|Could not load authoritative RON registry" \
  rust/crates/sim-bridge/src/lib.rs
# expect: hits
```

### Rust owns frame stepping
```bash
rg -n "runtime_tick_frame" \
  scripts/core/simulation/simulation_engine.gd \
  scripts/core/simulation/sim_bridge.gd \
  rust/crates/sim-bridge/src/lib.rs
# expect: hits in all three layers
```

## Verification Commands
```bash
cd rust && cargo build -p sim-bridge
cd rust && cargo test -p sim-bridge
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace -- -D warnings
"/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot" \
  --headless \
  --path /Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a \
  --quit
```

## Acceptance Criteria
- Godot does not register or clear runtime systems during boot.
- Rust loads authoritative RON data during runtime init.
- Rust registers the default runtime manifest.
- Simulation frame stepping exists only through Rust `runtime_tick_frame`.
- Godot boot exits successfully and UI shell still loads.
- Remaining shadow/bootstrap helpers are documented, but not part of the active boot scheduler.
