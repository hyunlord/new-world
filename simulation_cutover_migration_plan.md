# Simulation Cutover Migration Plan

## Completed in This Pass

| Godot Path | Rust / Bridge Replacement | Action |
|---|---|---|
| `main.gd` late registry validation | `SimulationEngine.validate_runtime_registry()` immediately after runtime init | Moved validation earlier so boot aborts before shadow manager creation on Rust authority mismatch |
| `main.gd` `NameGenerator.init()` on active boot path | none needed for active runtime | Removed boot-time legacy naming init |
| `main.gd` save/load name registry calls | none needed while save/load remains disabled | Removed dead legacy calls |
| `EntityManager.init()` eager spawn-generator setup | deprecated local spawn path only | Converted to lazy init inside `spawn_entity()` |
| `camera.set_entity_manager(entity_manager)` | `camera.set_sim_engine(sim_engine)` + snapshots/detail fallback | Removed active boot-time shadow entity dependency for camera |
| `building_renderer.init(building_manager, settlement_manager, sim_engine)` | runtime minimap/world summary getters | Switched active boot to runtime-backed rendering path |

## Remaining Follow-Up Migration

| Godot Script / Function | Rust / Bridge Owner | Next Step |
|---|---|---|
| `scripts/core/entity/entity_manager.gd::spawn_entity` | `runtime_spawn_agents` / `runtime_bootstrap_world` | Remove deprecated local spawn path once save/editor fallbacks stop referencing `EntityManager` |
| `scripts/core/world/resource_map.gd` live overlay fallback | runtime minimap/resource snapshot | Replace setup/editor-only data holder with bridge-backed read model after setup cutover |
| `scripts/core/social/relationship_manager.gd` | runtime relationship getters/tabs | Migrate legacy panels/debug paths fully to bridge queries |
| `scripts/core/social/reputation_manager.gd` | runtime reputation getters | Replace legacy entity-detail fallbacks |
| `scripts/systems/record/chronicle_system.gd` | Rust-side causal/event archive or read-only bridge feed | Keep as observer for now; migrate archive ownership later |
| `scripts/systems/biology/personality_generator.gd` | Rust temperament/personality initialization | Delete after deprecated local spawn path is removed |
| `scripts/systems/cognition/intelligence_generator.gd` | Rust intelligence/bootstrap assignment | Delete after deprecated local spawn path is removed |
| `scripts/systems/social/value_system.gd` | Rust value/settlement culture runtime | Delete after deprecated local spawn path is removed |
