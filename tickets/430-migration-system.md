# T-430: Migration System

## Action: DISPATCH (Codex)
## Files: scripts/systems/migration_system.gd (NEW)
## Depends: T-410, T-420

### Spec
```
extends "res://scripts/core/simulation_system.gd"
priority = 60
tick_interval = GameConfig.MIGRATION_TICK_INTERVAL (200)
```

### init(entity_manager, building_manager, settlement_manager, world_data, resource_map, rng)

### execute_tick(tick):
1. For each settlement: check migration triggers
2. Triggers (any one):
   a. settlement pop > shelters * 8
   b. food within radius 20 < pop * 0.5
   c. pop > MIGRATION_MIN_POP and rng.randf() < MIGRATION_CHANCE
3. Select 3-5 migrants: prefer low social, must include 1 builder
4. Find valid site: radius 30-80, resource-rich, 25+ tiles from any settlement
5. Set migrants' current_action = "migrate", action_target = site
6. Track pending migrations
7. Check arrivals: when all migrants within 3 tiles of target, create new settlement
8. Emit events: "migration_started", "settlement_founded"

### CRITICAL: NO class_name
### Use preload for settlement constants from GameConfig
