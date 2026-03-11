# Legacy Cleanup Map

## `valid_runtime_system`

- `rust/crates/sim-systems/src/runtime/**`
- `rust/crates/sim-engine/src/engine.rs`
- `rust/crates/sim-bridge/src/runtime_*`

## `shadow_system`

- `scripts/core/entity/entity_manager.gd`
- `scripts/core/settlement/building_manager.gd`
- `scripts/core/world/resource_map.gd`
- `scripts/core/social/relationship_manager.gd`
- `scripts/core/social/reputation_manager.gd`
- `scripts/core/settlement/settlement_manager.gd`
- `scripts/systems/**`
- `scripts/ai/**`

These are not the active Rust scheduler path, but still contain simulation-era state mutation logic.

## `obsolete_system`

- boot-time GDScript runtime system registration paths removed by prior refactors
- string-key runtime registry adapters removed from active boot path
- legacy-manager wiring reduced where runtime fallbacks already existed:
  - `building_renderer`
  - `camera_controller`
  - `minimap_panel`

## `dead_code`

- `Pathfinder` instantiation in `scenes/main/main.gd`
  - removed in this ticket because it was created but not consumed by the active runtime shell
- legacy JSON bundle path for full simulation authority
  - still present, but no longer the primary runtime content path

## Cleanup priority

1. Remove obviously unused boot-time residue.
2. Stop wiring live UI/runtime paths through legacy managers when runtime fallbacks already exist.
3. Keep legacy panels/managers only behind explicit compatibility paths until full migration tickets remove them.
