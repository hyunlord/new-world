# Verify Simulation Authority

## Static Checks

### Boot does not register GDScript simulation systems
```bash
rg -n "register_system\\(" scenes/main/main.gd scripts/core/simulation/simulation_engine.gd
# expect: 0 hits
```

### Boot does not clear and rebuild a GDScript runtime registry
```bash
rg -n "runtime_clear_registry" scenes/main/main.gd scripts/core/simulation
# expect: 0 hits
```

### Active frame tick exists only in Rust runtime
```bash
rg -n "runtime_tick_frame" scripts/core/simulation/simulation_engine.gd scripts/core/simulation/sim_bridge.gd rust/crates/sim-bridge/src
# expect: hits
```

### Rust default runtime registration is active
```bash
rg -n "runtime_register_default_systems" \
  scripts/core/simulation/sim_bridge.gd \
  scripts/core/simulation/simulation_engine.gd \
  rust/crates/sim-bridge/src
# expect: hits in both GDScript bridge and Rust sim-bridge
```

### `main.gd` no longer preloads runtime systems from `scripts/systems` or `scripts/ai`
```bash
rg -n 'const .*preload\\("res://scripts/(systems|ai)/' scenes/main/main.gd
# expect: 0 hits
```

### Active bridge/runtime path contains no string-key registry rebuild
```bash
rg -n "register_system|clear_registry|runtime_system_key_from_name|runtime_supports_rust_system|system_key" rust/crates/sim-bridge/src scripts/core/simulation -g '*.rs' -g '*.gd'
# expect: 0 hits
```

### Dead legacy authority files are gone
```bash
test ! -f scripts/core/combat/combat_resolver.gd
test ! -f scripts/core/simulation/runtime_shadow_reporter.gd
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
- All active simulation ticks occur through Rust runtime `runtime_tick_frame()`.
- Boot-time runtime registry population occurs through Rust `runtime_register_default_systems()`.
- `main.gd` does not instantiate or register legacy GDScript runtime systems from `scripts/systems/**` or `scripts/ai/**`.
- Godot GDScript does not rebuild scheduler authority locally.
- Godot remains a shell/UI/render/bridge layer.
- Remaining GDScript shadow/bootstrap helpers are documented as non-authoritative residue.
