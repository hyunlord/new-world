# Boot Pipeline Map

## Scope

- [project.godot](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/project.godot)
- [scenes/main/main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd)
- [scripts/core/simulation/simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd)
- [scripts/core/simulation/sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/sim_bridge.gd)
- [rust/crates/sim-bridge/src/lib.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/rust/crates/sim-bridge/src/lib.rs)
- [rust/crates/sim-bridge/src/runtime_registry.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/rust/crates/sim-bridge/src/runtime_registry.rs)

## Boot Order

1. Godot loads autoloads from [project.godot](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/project.godot).
2. Main scene enters [main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd).
3. [main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd) creates [simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd).
4. [simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd) calls `runtime_init(seed, config_json)`.
5. Rust runtime loads authoritative RON data during `runtime_init`.
6. [simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd) calls `runtime_register_default_systems()`.
7. [main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd) validates the Rust-owned registry immediately and aborts boot if authority is not Rust-backed.
8. Only after that validation passes does Godot build shell/bootstrap helpers such as `WorldData`, `ResourceMap`, `EntityManager`, `BuildingManager`, `SettlementManager`, and UI managers.
9. Setup UI submits bootstrap payload.
10. [simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd) calls `runtime_bootstrap_world(...)`.
11. After bootstrap, `main.gd` still wires `ChronicleSystem.init(entity_manager)` and `SimulationBus` chronicle hooks for observer/compatibility behavior.
12. Frame stepping runs through `update(delta)` -> `runtime_tick_frame(...)` -> Rust ECS tick.

## Autoload Script List

- `GameConfig`
- `SpeciesManager`
- `SimulationBus`
- `SimulationBusV2`
- `EventLogger`
- `DeceasedRegistry`
- `ChronicleSystem`
- `NameGenerator`
- `Locale`
- `StatQuery`
- `SimBridge`
- `ComputeBackend`
- `HarnessServer`

## Scene Preload Chain

### Active startup preloads in [main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd)

- runtime wrapper: `simulation_engine.gd`
- setup shell: `world_setup.gd`
- world/bootstrap helpers: `world_data.gd`, `world_generator.gd`, `resource_map.gd`
- legacy/shadow helper managers: `entity_manager.gd`, `building_manager.gd`, `settlement_manager.gd`, `relationship_manager.gd`, `reputation_manager.gd`
- UI shell helpers: `pause_menu.gd`, `ambience_manager.gd`

### Not part of active boot authority

- no preload from `scripts/systems/**`
- no preload from `scripts/ai/**`

## sim-bridge Initialization

### Godot side

[sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/sim_bridge.gd) provides:

- `runtime_init`
- `runtime_register_default_systems`
- `runtime_bootstrap_world`
- `runtime_tick_frame`
- runtime getters for snapshots/details/debug

[simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd) provides:

- startup config packaging
- runtime init
- default system registration
- frame tick relay
- command/event relay via `SimulationBusV2`

### Rust side

[lib.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/rust/crates/sim-bridge/src/lib.rs) exposes the runtime entrypoints.

[runtime_registry.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/rust/crates/sim-bridge/src/runtime_registry.rs) owns runtime state and typed registry entries.

## Rust Runtime Initialization

- first runtime state creation: `RuntimeState::from_seed(...)`
- first ECS world creation: inside Rust runtime state/engine setup during `runtime_init`
- first authoritative registry load: `DataRegistry::load_from_directory(...)`
- first simulation scheduler registration: `runtime_register_default_systems()`
- first simulation tick location: Rust `runtime_tick_frame(...)`

## Boot Truth

- active boot registry authority: Rust
- active simulation tick authority: Rust
- active bridge boundary: Godot -> `sim-bridge` -> Rust ECS
- residual shadow/bootstrap managers and chronicle/bus observer hookups still exist, but boot now refuses to continue if the Rust registry authority check fails
- boot is Rust-authoritative, not pure Rust-only
