# Bridge Boundary Enforcement

## Required Architecture
Godot  
↓  
`scripts/core/simulation/sim_bridge.gd`  
↓  
`rust/crates/sim-bridge`  
↓  
Rust ECS runtime

## Allowed Godot Operations
- initialize runtime through `runtime_init()`
- request runtime registration through `runtime_register_default_systems()`
- advance frame simulation through `runtime_tick_frame()`
- read snapshots, summaries, entity detail/tab/list getters
- send approved command payloads through `runtime_apply_commands_v2()`
- mutate pre-runtime setup/editor state before bootstrap

## Forbidden Godot Operations
- direct ECS mutation
- direct scheduler registration by string key
- clearing/rebuilding runtime registry from GDScript
- owning the authoritative simulation tick in `_process()` / `_physics_process()` outside bridge tick forwarding
- post-bootstrap gameplay mutation through local manager systems

## Enforcement in This Pass
- `main.gd` now aborts before shadow-manager boot if Rust registry validation fails.
- `simulation_engine.gd` remains the only Godot-side runtime tick forwarder.
- camera and building renderer active boot path no longer receive legacy manager state when runtime-backed alternatives already exist.
- active boot path no longer initializes deprecated local spawn helpers.

## State Flow
- Player/setup input → Godot UI/setup scripts
- Runtime command → `SimulationBusV2` / `sim_bridge.gd`
- Mutation → Rust ECS only
- Snapshot/detail/list → `sim-bridge`
- Presentation → Godot UI/renderers
