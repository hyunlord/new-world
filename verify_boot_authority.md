# Verify Boot Authority

## Static checks

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
  scripts/core/simulation/simulation_engine.gd \
  scripts/core/simulation/sim_bridge.gd \
  rust/crates/sim-bridge/src
# expect: hits in GDScript bridge/wrapper and Rust bridge
```

### `main.gd` now fails fast if Rust registry authority validation fails

```bash
rg -n "validate_runtime_registry|aborting boot" scenes/main/main.gd
# expect: hits before shadow/bootstrap manager initialization
```

### No direct `scripts/systems` or `scripts/ai` preload remains in `main.gd`

```bash
rg -n 'const .*preload\\("res://scripts/(systems|ai)/' scenes/main/main.gd
# expect: 0 hits
```

## Boot truth checks

### RON registry loads during runtime init

```bash
rg -n "DataRegistry::load_from_directory" \
  rust/crates/sim-bridge/src/lib.rs \
  rust/crates/sim-test/src/main.rs
# expect: hits
```

### Simulation tick exists only through Rust runtime

```bash
rg -n "runtime_tick_frame" \
  scripts/core/simulation/simulation_engine.gd \
  scripts/core/simulation/sim_bridge.gd \
  rust/crates/sim-bridge/src/lib.rs
# expect: hits in all three layers
```

### Godot boot still instantiates residual shadow/bootstrap managers

```bash
rg -n "EntityManager\\.new|BuildingManager\\.new|SettlementManager\\.new|ResourceMap\\.new" scenes/main/main.gd
# expect: hits
```

This is allowed as documented technical debt, but these objects must not become the active tick owner.

## Verification commands

```bash
cd rust && cargo check --workspace
cd rust && cargo build -p sim-bridge
cd rust && cargo test --workspace
cd rust && cargo clippy --workspace -- -D warnings
"/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot" \
  --headless \
  --path /Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a \
  --quit
```

## Acceptance criteria

- Godot does not register or clear runtime systems during boot.
- Rust loads authoritative RON data before ECS bootstrap.
- Rust owns default runtime system registration.
- Rust owns the active frame tick.
- `main.gd` aborts boot if runtime registry authority validation fails.
- Godot shell/UI still boots successfully when validation passes.
