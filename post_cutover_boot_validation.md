# Post-Cutover Boot Validation

## Boot Order
1. `main.gd` creates `SimulationEngine`
2. `SimulationEngine.init_with_seed()` calls Rust `runtime_init()`
3. `SimulationEngine` requests Rust default registration through `runtime_register_default_systems()`
4. `main.gd` immediately validates registry authority
5. only after successful validation does Godot instantiate shadow/bootstrap managers and render/UI shells
6. setup world is edited in Godot
7. `bootstrap_world()` transfers setup payload into Rust ECS
8. frame updates call Rust `runtime_tick_frame()`

## Validation Checks
- Godot boot does not register runtime systems by string key
- Godot boot does not clear/rebuild the runtime registry
- Rust registry validation occurs before shadow manager composition
- Rust remains the owner of the first authoritative runtime tick
- Godot render/UI starts only after bridge/runtime setup is valid

## Residual Boot Debt
- `main.gd` still instantiates shadow managers after Rust validation passes
- setup/editor world generation still happens in Godot before bootstrap
- autoload `ChronicleSystem` and `NameGenerator` remain present, though the active boot path no longer initializes naming helpers for gameplay spawn
