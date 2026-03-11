# Godot Boot Pipeline Map

## Scope

This map covers the active Godot boot path for:

- [project.godot](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/project.godot)
- [scenes/main/main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/scenes/main/main.gd)
- [scripts/core/simulation/simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/scripts/core/simulation/simulation_engine.gd)
- [scripts/core/simulation/sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/scripts/core/simulation/sim_bridge.gd)
- active autoload scripts in `project.godot`

## Startup Order

1. Godot loads autoload singletons from [project.godot](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/project.godot).
2. Main scene [main.tscn](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/scenes/main/main.tscn) enters [main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/scenes/main/main.gd).
3. `main.gd` creates shell/bootstrap managers:
   - `SimulationEngine`
   - `WorldData`
   - `ResourceMap`
   - `EntityManager`
   - `BuildingManager`
   - `SettlementManager`
   - `RelationshipManager`
   - `ReputationManager`
   - `TechTreeManager`
4. `SimulationEngine.init_with_seed()` initializes Rust runtime.
5. `SimulationEngine._init_rust_runtime()` calls:
   - `SimBridge.runtime_init(seed, config_json)`
   - `SimBridge.runtime_register_default_systems()`
6. `main.gd` validates the Rust registry snapshot.
7. Setup scene runs; on confirm, `SimulationEngine.bootstrap_world(...)` bootstraps the Rust ECS world.
8. Frame ticking runs through `SimulationEngine.update(delta)` → `SimBridge.runtime_tick_frame(...)`.

## Autoload Scripts

Autoloads declared in [project.godot](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/project.godot):

- `GameConfig` — config mirror / startup constants
- `SpeciesManager` — legacy/shared species helper
- `SimulationBus` — UI event relay
- `SimulationBusV2` — runtime command/event relay
- `EventLogger` — UI/event logging
- `DeceasedRegistry` — archive/lookups
- `ChronicleSystem` — legacy chronicle store, not startup tick authority
- `NameGenerator` — naming helper
- `Locale` — localization
- `StatQuery` — shared stat lookup helper
- `SimBridge` — native bridge wrapper
- `ComputeBackend` — compute mode shell
- `HarnessServer` — test harness

## Preload Chains

### Main scene preload chain

[main.gd](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/scenes/main/main.gd) preloads only shell/bootstrap dependencies in the active boot path:

- `simulation_engine.gd`
- world/map/bootstrap helpers
- save manager
- settlement/relationship/reputation/tech managers
- pause menu / ambience
- setup scene

### Removed simulation-authority preload chain

The previous boot path also preloaded dozens of legacy simulation systems from:

- `scripts/systems/**`
- `scripts/ai/behavior_system.gd`

Those preloads and startup registrations were removed from active boot.

## Bridge Initialization

### Godot side

[simulation_engine.gd](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/scripts/core/simulation/simulation_engine.gd):

- owns startup seed/config assembly
- calls native runtime init
- calls native default-system registration
- ticks via `runtime_tick_frame`
- reads snapshots/details through bridge getters

[sim_bridge.gd](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/scripts/core/simulation/sim_bridge.gd):

- wrapper only
- forwards `runtime_init`
- forwards `runtime_register_default_systems`
- forwards `runtime_tick_frame`
- forwards detail/snapshot/debug getters

### Rust side

[lib.rs](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/rust/crates/sim-bridge/src/lib.rs):

- exposes `runtime_init`
- exposes `runtime_register_default_systems`
- exposes `runtime_tick_frame`

[runtime_registry.rs](/Users/rexxa/github/new-world-wt/codex-refactor-boot-authority/rust/crates/sim-bridge/src/runtime_registry.rs):

- defines the authoritative default runtime manifest
- registers Rust runtime systems
- reports registry snapshot entries as Rust-backed

## Boot-Time Simulation Authority Leak Findings

### Previous leaks found

1. `main.gd` instantiated legacy GDScript simulation systems.
2. `main.gd` called `sim_engine.register_system(...)` for those systems.
3. `simulation_engine.gd` maintained GDScript-side registry metadata and pushed `register_system` commands into Rust.
4. `simulation_engine.gd` called `runtime_clear_registry()` after init, forcing Godot to rebuild startup authority.

### Current status after refactor

- `main.gd` no longer instantiates or registers legacy simulation systems at boot.
- `simulation_engine.gd` no longer clears the registry after init.
- `simulation_engine.gd` no longer builds the authoritative startup registry from GDScript metadata.
- Rust default manifest now owns startup simulation registration.

## Correct Authority Boundary

```text
Godot shell / UI / rendering
    ↓
sim_bridge (GDExtension boundary)
    ↓
Rust runtime registry
    ↓
Rust ECS simulation tick
```

## Current Authority Statement

Boot authority is secure for this ticket when all of the following are true:

- Godot starts the runtime but does not build the simulation registry itself.
- Rust registers the default runtime systems.
- Rust owns simulation ticking.
- Godot reads snapshots, emits UI signals, and submits commands only through bridge APIs.
