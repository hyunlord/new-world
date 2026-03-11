# Simulation Ownership Map

## Target ownership

Godot  
-> `sim-bridge`  
-> Rust ECS

## Ownership by responsibility

### Simulation tick

- owner: Rust
- path:
  - [simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd) calls `runtime_tick_frame(...)`
  - [sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/sim_bridge.gd) forwards it
  - [lib.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/rust/crates/sim-bridge/src/lib.rs) executes the Rust runtime tick

### Agent state mutation

- owner: Rust
- active path: runtime systems in `rust/crates/sim-systems/src/runtime/**`
- Godot status: may still hold shadow objects, but active state mutation is not driven from Godot boot/tick

### World state mutation

- owner: Rust for authoritative runtime state
- residual Godot shadow/bootstrap state:
  - `WorldData`
  - `ResourceMap`
  - legacy managers created in `main.gd`

### Registry initialization

- owner: Rust
- path:
  - `runtime_init(...)`
  - `DataRegistry::load_from_directory(...)`
  - `runtime_register_default_systems()`

## First-owner checkpoints

- first authoritative registry load: Rust
- first ECS world creation: Rust
- first scheduler registration: Rust
- first active simulation tick: Rust

## Godot-only allowed responsibilities

- boot shell
- setup input
- snapshot/detail reads
- rendering
- UI
- player command relay

## Godot-forbidden responsibilities

- owning scheduler registration
- clearing/rebuilding authoritative runtime registry during boot
- owning simulation tick
- directly mutating ECS state

## Current verdict

- `simulation_tick_owner = rust_only`
- `registry_owner = rust_only`
- `boot_authority_secure = true for active runtime path`
- remaining risk: hybrid shadow/bootstrap helpers still exist and are instantiated after Rust authority validation
- remaining risk: `ChronicleSystem.init(entity_manager)` and `SimulationBus` observer hookups stay on the boot path as simulation-adjacent compatibility debt
