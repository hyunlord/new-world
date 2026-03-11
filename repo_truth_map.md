# WorldSim Repository Truth Map

## Rust crate structure

- `rust/crates/sim-core`
  - Shared components, enums, config constants, influence/effect/room/temperament foundations.
- `rust/crates/sim-data`
  - RON `DataRegistry` plus legacy JSON compatibility loaders.
- `rust/crates/sim-systems`
  - Runtime `SimSystem` implementations and entity spawning.
- `rust/crates/sim-engine`
  - ECS world/resources, scheduler, tick orchestration, perf tracking.
- `rust/crates/sim-bridge`
  - Godot boundary, runtime boot/tick/init, debug/query surface, runtime registry.
- `rust/crates/sim-test`
  - Headless runtime bootstrap executable for smoke/debug runs.

## Rust boundary summary

- `sim-core`
  - Owns shared data types and simulation-facing component definitions.
- `sim-systems`
  - Owns gameplay state transitions and per-tick mutation.
- `sim-engine`
  - Owns scheduler order, event flow, resources, and perf tracker.
- `sim-data`
  - Owns authoritative RON registry load plus compatibility JSON loaders.
- `sim-bridge`
  - Owns Godot type translation and runtime boot/tick commands only.

## Godot script hierarchy

- `scenes/main/main.gd`
  - Main scene entry point.
- `scripts/core/**`
  - Locale, simulation wrapper, legacy/shadow managers, setup/bootstrap helpers.
- `scripts/ui/**`
  - HUD, panels, camera, renderers.
- `scripts/debug/**`
  - Debug/inspection helpers.
- `scripts/rendering/**`
  - Snapshot decoding and presentation helpers.
- `scripts/systems/**`, `scripts/ai/**`
  - Mostly residual/legacy shadow simulation-era code.

Note:
- `scripts/agents/` and `scripts/world/` do not exist in the current repository.

## Data loading pipeline

### Authoritative runtime path

1. Godot boots `SimulationEngine`.
2. `sim-bridge::runtime_init()` loads RON via `DataRegistry::load_from_directory`.
3. The `DataRegistry` is stored in `RuntimeState.data_registry`.
4. `bootstrap_world()` injects world/bootstrap payload into ECS resources/world.
5. Rust runtime systems consume registry/world/config through ECS resources.

Relevant paths:
- `rust/crates/sim-bridge/src/lib.rs`
- `rust/crates/sim-data/src/registry.rs`
- `rust/crates/sim-test/src/main.rs`

### Compatibility path

- Legacy JSON is still loaded for:
  - personality distribution
  - name cultures
  - legacy JSON bundle tests

Relevant paths:
- `rust/crates/sim-bridge/src/runtime_registry.rs`
- `rust/crates/sim-data/src/lib.rs`
- `rust/tests/data_loading_test.rs`

## Boot pipeline

### Active boot order

1. `project.godot` boots autoloads.
2. `scenes/main/main.gd::_ready()` creates `SimulationEngine`.
3. `SimulationEngine.init_with_seed()` calls Rust `runtime_init`.
4. Rust loads RON registry first.
5. Rust registers default runtime systems.
6. Godot setup UI creates the bootstrap payload.
7. Rust `bootstrap_world()` builds ECS state.
8. Godot renderers/HUD subscribe to runtime snapshots and detail getters.

### Bridge initialization

- GDScript wrapper: `scripts/core/simulation/sim_bridge.gd`
- GDScript runtime wrapper: `scripts/core/simulation/simulation_engine.gd`
- Rust entrypoint: `rust/crates/sim-bridge/src/lib.rs`

## ECS runtime initialization

- `RuntimeState::from_seed()` creates `SimEngine` and `SimResources`.
- `runtime_register_default_systems()` applies the typed default manifest.
- `runtime_tick_frame()` owns the simulation tick loop in Rust.

Relevant paths:
- `rust/crates/sim-bridge/src/runtime_registry.rs`
- `rust/crates/sim-bridge/src/runtime_system.rs`
- `rust/crates/sim-bridge/src/lib.rs`
- `rust/crates/sim-engine/src/engine.rs`

## Runtime system registry

- Current identity is typed `RuntimeSystemId`.
- Default scheduler manifest is `DEFAULT_RUNTIME_SYSTEMS`.
- Registry boot still exports human-readable names for debug/UI.

Relevant paths:
- `rust/crates/sim-bridge/src/runtime_system.rs`
- `rust/crates/sim-bridge/src/runtime_registry.rs`
- `rust/crates/sim-bridge/src/runtime_commands.rs`

## Legacy compatibility paths

- `sim_data::load_all()` still aggregates legacy JSON definitions.
- `load_personality_distribution()` and `load_name_cultures()` still support boot compatibility.
- `main.gd` still instantiates legacy/shadow managers for UI, save/load, and fallback-era panels:
  - `EntityManager`
  - `BuildingManager`
  - `SettlementManager`
  - `RelationshipManager`
  - `ReputationManager`
  - `TechTreeManager`
  - `ResourceMap`
  - `SaveManager`

At the same time, some active presentation paths no longer consume those
legacy managers when runtime-backed data is already available:

- `building_renderer`
- `camera_controller`
- `minimap_panel`

## Current repository truth

- Simulation tick authority is Rust-owned.
- Data authority is mostly RON-owned, but not purely so because JSON compatibility still boots.
- Godot still carries residual shadow/bootstrap managers and some legacy panels.
- The repository is closer to the target architecture than before, but not yet a pure Rust-only runtime shell.
