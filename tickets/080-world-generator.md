# 080 - WorldGenerator

## Objective
Create procedural world generation using FastNoiseLite with island mask, biome classification from elevation/moisture/temperature.

## Prerequisites
- 020-game-config
- 070-world-data

## Non-goals
- No river/road generation
- No resource placement
- No multi-biome blending

## Files to create
- `scripts/core/world_generator.gd`

## Implementation Steps
1. Create `scripts/core/world_generator.gd` extending RefCounted
2. ```gdscript
   class_name WorldGenerator
   extends RefCounted
   ```
3. `generate(world_data: WorldData, seed_value: int) -> void`:
   - Create FastNoiseLite instances for elevation, moisture, temperature
   - Configure elevation noise:
     - noise_type = FastNoiseLite.TYPE_SIMPLEX_SMOOTH
     - fractal_octaves = 5
     - frequency = 0.008
     - seed = seed_value
   - Configure moisture noise:
     - noise_type = FastNoiseLite.TYPE_SIMPLEX_SMOOTH
     - fractal_octaves = 4
     - frequency = 0.006
     - seed = seed_value + 1
   - Configure temperature noise:
     - noise_type = FastNoiseLite.TYPE_SIMPLEX_SMOOTH
     - fractal_octaves = 3
     - frequency = 0.004
     - seed = seed_value + 2
4. For each tile (x, y):
   - Raw elevation = noise.get_noise_2d(x, y) remapped from [-1,1] to [0,1]
   - Apply island mask: distance from center, multiply by falloff
   - Temperature affected by: noise + latitude (y distance from center) + elevation
   - Classify biome from (elevation, moisture, temperature):
     - elevation < 0.3 → DEEP_WATER
     - elevation < 0.4 → SHALLOW_WATER
     - elevation < 0.45 → BEACH
     - elevation < 0.65:
       - moisture < 0.3 → GRASSLAND
       - moisture < 0.6 → FOREST
       - else → DENSE_FOREST
     - elevation < 0.8 → HILL
     - elevation < 0.9 → MOUNTAIN
     - else → SNOW
   - Set tile data in world_data
5. Island mask function:
   ```gdscript
   func _island_mask(x: int, y: int, w: int, h: int) -> float:
       var nx = 2.0 * x / w - 1.0
       var ny = 2.0 * y / h - 1.0
       var d = max(abs(nx), abs(ny))
       return 1.0 - pow(d, GameConfig.ISLAND_FALLOFF * 3.0)
   ```

## Verification
- Gate PASS
- Deterministic: same seed → same world

## Acceptance Criteria
- [ ] Uses FastNoiseLite (Godot built-in)
- [ ] Island-shaped world (water at edges)
- [ ] All 9 biomes appear
- [ ] Deterministic with seed
- [ ] Populates WorldData arrays
- [ ] Gate PASS

## Risk Notes
- Noise parameters need tuning for good-looking world
- Island mask prevents entities spawning at edges

## Roll-back Plan
- Delete file
