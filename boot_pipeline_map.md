# Godot Boot Pipeline Map

## Scope
- [project.godot](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/project.godot)
- [scenes/main/main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd)
- [scripts/core/simulation/simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd)
- [scripts/core/simulation/sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/sim_bridge.gd)
- active autoloads declared in [project.godot](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/project.godot)

## Boot Order
1. Godot starts and loads autoloads from [project.godot](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/project.godot).
2. Main scene [scenes/main/main.tscn](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.tscn) enters [scenes/main/main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd).
3. [main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd) builds shell/bootstrap objects:
   - `SimulationEngine`
   - `WorldData`
   - `ResourceMap`
   - `EntityManager`
   - `BuildingManager`
   - `SettlementManager`
   - `RelationshipManager`
   - `ReputationManager`
   - `TechTreeManager`
4. [simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd) calls `runtime_init(seed, config_json)`.
5. Rust `sim-bridge` initializes and loads the authoritative RON registry before any world bootstrap.
6. [simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd) calls `runtime_register_default_systems()`.
7. [main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd) validates the runtime registry snapshot and only warns if it is not fully Rust-backed.
8. Setup scene confirms spawn/bootstrap parameters.
9. [simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd) calls `runtime_bootstrap_world(...)`.
10. Frame stepping runs through `update(delta)` -> `runtime_tick_frame(...)` -> Rust ECS tick.

## Autoload Scripts
Autoloads in [project.godot](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/project.godot):
- `GameConfig` — config mirror / startup constants
- `SpeciesManager` — species helper
- `SimulationBus` — legacy UI event relay
- `SimulationBusV2` — runtime command/event relay
- `EventLogger` — event observer/log sink
- `DeceasedRegistry` — deceased archive / lookup helper
- `ChronicleSystem` — chronicle observer store
- `NameGenerator` — naming helper
- `Locale` — localization
- `StatQuery` — stat lookup helper
- `SimBridge` — GDExtension wrapper
- `ComputeBackend` — compute/pathfinding mode shell
- `HarnessServer` — test harness

## Scene Preload Chain
Active preloads in [main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scenes/main/main.gd):
- `simulation_engine.gd`
- world/map/bootstrap helpers
- save/load helpers
- settlement/social bootstrap managers
- UI/pause/setup classes

Removed from active boot authority:
- legacy preloads from `scripts/systems/**`
- `scripts/ai/behavior_system.gd`

## Bridge Initialization
### Godot side
[scripts/core/simulation/sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/sim_bridge.gd):
- resolves the native runtime singleton
- forwards `runtime_init`
- forwards `runtime_register_default_systems`
- forwards `runtime_bootstrap_world`
- forwards `runtime_tick_frame`
- exposes read/query methods for UI/rendering

[scripts/core/simulation/simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/scripts/core/simulation/simulation_engine.gd):
- assembles startup config
- initializes the native runtime
- validates the registry snapshot
- ticks Rust every frame
- relays commands/events via `SimulationBusV2`

### Rust side
[rust/crates/sim-bridge/src/lib.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/rust/crates/sim-bridge/src/lib.rs):
- exposes `runtime_init`
- exposes `runtime_register_default_systems`
- exposes `runtime_bootstrap_world`
- exposes `runtime_tick_frame`

[rust/crates/sim-bridge/src/runtime_system.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/rust/crates/sim-bridge/src/runtime_system.rs):
- defines typed `RuntimeSystemId`
- defines the authoritative default runtime manifest

[rust/crates/sim-bridge/src/runtime_registry.rs](/Users/rexxa/github/new-world-wt/codex-refactor-ws-ref-004a/rust/crates/sim-bridge/src/runtime_registry.rs):
- stores typed runtime registry entries
- registers only Rust-backed systems in the boot manifest

## Current Boot Authority Statement
The active boot pipeline is authoritative when all are true:
- Godot starts the runtime but does not register simulation systems itself.
- Rust loads the RON registry during runtime init.
- Rust registers the default scheduler manifest.
- Rust owns frame stepping through `runtime_tick_frame`.
- Godot only boots shell/UI/render/bootstrap layers and reads runtime state through the bridge.
