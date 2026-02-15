# T-470: Save/Load Settlement Support

## Action: DISPATCH (Codex)
## Files: scripts/core/save_manager.gd
## Depends: T-410

### Changes:
- save_game: add settlement_manager parameter, save settlement_manager.to_save_data()
- load_game: add settlement_manager parameter, restore from data
- Bump version to 2

### Updated signatures:
```gdscript
func save_game(path, sim_engine, entity_manager, building_manager, resource_map, settlement_manager) -> bool
func load_game(path, sim_engine, entity_manager, building_manager, resource_map, world_data, settlement_manager) -> bool
```

### Data schema addition:
```json
{
    "version": 2,
    "sim_engine": {...},
    "entities": [...],
    "buildings": [...],
    "resource_map": {...},
    "settlements": [...]
}
```

### Backward compat: if version=1, skip settlements load
