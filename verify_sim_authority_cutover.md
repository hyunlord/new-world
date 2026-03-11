# Verify Simulation Authority Cutover

## Static Checks

### Boot does not register GDScript simulation systems
```bash
rg -n "register_system\\(" scenes/main/main.gd scripts/core/simulation/simulation_engine.gd
# expect: 0 hits
```

### Boot does not clear and rebuild a legacy GDScript runtime registry
```bash
rg -n "runtime_clear_registry" scripts/core/simulation
# expect: 0 hits
```

### Rust default registry path is active
```bash
rg -n "runtime_register_default_systems" \
  scripts/core/simulation/sim_bridge.gd \
  scripts/core/simulation/simulation_engine.gd \
  rust/crates/sim-bridge/src
# expect: hits in both GDScript bridge and Rust sim-bridge
```

### `main.gd` no longer preloads runtime system files from `scripts/systems` or `scripts/ai`
```bash
rg -n 'const .*preload\\("res://scripts/(systems|ai)/' scenes/main/main.gd
# expect: 0 hits
```

### Dead legacy runtime systems were removed from disk
```bash
find scripts/systems -name '*.gd' | sort
# expect: only observer/bootstrap residue remains, no old runtime tick systems
```

## Rust Verification
```bash
cd rust && cargo build -p sim-bridge
cd rust && cargo test -p sim-bridge
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace -- -D warnings
```

## Godot Verification
```bash
"/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot" \
  --headless \
  --path /Users/rexxa/github/new-world-wt/codex-refactor-sim-authority-cutover \
  --quit
```

## Acceptance Criteria
- All simulation ticks occur through Rust runtime `runtime_tick_frame()`.
- Boot-time runtime registry population occurs via Rust `runtime_register_default_systems()`.
- `main.gd` does not instantiate or register legacy GDScript simulation systems.
- Legacy dead runtime system scripts have been removed from `scripts/systems/**` and `scripts/ai/behavior_system.gd`.
- Godot remains a shell/UI/render/bridge layer.
- Remaining GDScript shadow/bootstrap helpers are documented and are not part of the active runtime scheduler.
