# Simulation Authority Audit

## Target boundary

Godot  
-> `sim-bridge`  
-> Rust ECS simulation

## Current authority verdict

- Active scheduler authority: Rust
- Active simulation tick authority: Rust
- Active render/UI authority: Godot
- Residual shadow/bootstrap logic on disk: present
- Residual boot-time legacy manager instantiation: present

## Classification

### `bridge_only`

- `scripts/core/simulation/sim_bridge.gd`
- `scripts/core/simulation/simulation_engine.gd`
- `scripts/ui/**` files that only query runtime state or render it

### `render_only`

- `scripts/ui/renderers/**`
- `scripts/rendering/**`

### `ui_only`

- `scripts/ui/panels/**`
- `scripts/ui/hud.gd`
- `scripts/ui/camera_controller.gd`

### `simulation_authority_leak`

These files still contain simulation-state mutation logic even if they are no longer the active tick owner:

- `scripts/core/entity/entity_manager.gd`
  - entity creation, stat initialization, death removal, legacy state mutation
- `scripts/core/settlement/building_manager.gd`
  - building placement/removal/progress state mutation
- `scripts/core/world/resource_map.gd`
  - resource availability and harvest mutation
- `scripts/core/social/relationship_manager.gd`
  - relationship state mutation
- `scripts/core/social/reputation_manager.gd`
  - reputation state mutation
- `scripts/core/settlement/settlement_manager.gd`
  - settlement membership/state mutation
- `scripts/systems/**`
  - residual simulation-system era logic
- `scripts/ai/**`
  - residual behavior/decision-era logic

## Active path vs residual path

### Active path

- `main.gd` -> `SimulationEngine` -> `sim-bridge` -> Rust ECS
- `runtime_tick_frame()` in Rust owns stepping
- default runtime systems are registered from Rust
- several live presentation paths now prefer runtime-first wiring:
  - `building_renderer`
  - `camera_controller`
  - `minimap_panel`

### Residual path

- `main.gd` still instantiates legacy managers for save/load, bootstrap helpers, and older panel paths
- many renderers and panels have runtime fallbacks now, but the legacy objects still exist in the live scene

## Practical consequence

- Godot is no longer the active scheduler authority.
- Godot is still not a pure UI/render shell because live legacy manager graphs are created and consumed by some runtime-facing UI paths.

## Audit conclusion

WorldSim currently has:

- `Rust-owned active runtime authority`
- `Godot-owned residual shadow/bootstrap state`

The active authority boundary is mostly correct, but the repository still carries shadow state that can confuse maintenance and future feature work.
