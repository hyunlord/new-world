# 020 - GameConfig Autoload

## Objective
Create the GameConfig singleton with all game constants, biome definitions, simulation parameters, and camera settings.

## Prerequisites
- 010-project-structure

## Non-goals
- No runtime modification of config (read-only constants)
- No file-based config loading (Phase 1)

## Files to create
- `scripts/core/game_config.gd`

## Implementation Steps
1. Create `scripts/core/game_config.gd` extending Node (autoload must be Node)
2. Define constants:
   - WORLD_SIZE = Vector2i(256, 256)
   - TILE_SIZE = 16
   - CHUNK_SIZE = 32
3. Simulation parameters:
   - TICKS_PER_SECOND = 10
   - MAX_ENTITIES = 500
   - INITIAL_SPAWN_COUNT = 20
   - MAX_TICKS_PER_FRAME = 5
   - TICK_HOURS = 1 (1 tick = 1 game hour)
   - DAYS_PER_YEAR = 360
   - HOURS_PER_DAY = 24
4. Speed multipliers: SPEED_OPTIONS = [1, 2, 3, 5, 10]
5. Biome enum and data:
   ```
   enum Biome { DEEP_WATER, SHALLOW_WATER, BEACH, GRASSLAND, FOREST, DENSE_FOREST, HILL, MOUNTAIN, SNOW }
   ```
   - BIOME_COLORS: Dictionary mapping Biome → Color
   - BIOME_MOVE_COST: Dictionary mapping Biome → float (0.0 = impassable)
   - BIOME_WALKABLE: derived from move_cost > 0
6. Camera settings:
   - CAMERA_ZOOM_MIN = 0.25
   - CAMERA_ZOOM_MAX = 4.0
   - CAMERA_ZOOM_STEP = 0.1
   - CAMERA_PAN_SPEED = 500.0
   - CAMERA_ZOOM_SPEED = 0.15
7. Entity defaults:
   - HUNGER_DECAY_RATE = 0.003
   - ENERGY_DECAY_RATE = 0.002
   - ENERGY_ACTION_COST = 0.004
   - SOCIAL_DECAY_RATE = 0.001
8. World gen params:
   - WORLD_SEED = 42
   - NOISE_OCTAVES = 5
   - ISLAND_FALLOFF = 0.7

## Verification
- File parses without error (Gate PASS)
- All constants accessible via GameConfig.CONSTANT_NAME

## Acceptance Criteria
- [ ] GameConfig.gd is a valid autoload script
- [ ] All biome enums defined
- [ ] All biome colors and costs defined
- [ ] All simulation parameters defined
- [ ] Gate PASS

## Risk Notes
- Biome colors should be visually distinct
- Move costs must be balanced (not all 1.0)
- DEEP_WATER and MOUNTAIN should be impassable (cost=0)

## Roll-back Plan
- Delete file, remove autoload from project.godot
