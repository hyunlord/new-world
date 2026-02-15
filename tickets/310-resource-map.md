# Ticket 310: ResourceMap Class [Foundation]

## Objective
Create ResourceMap (RefCounted) that stores per-tile resource amounts using PackedFloat32Array.

## Dependencies
- 300 (GameConfig Phase 1 constants)

## Non-goals
- No rendering
- No regen system (separate ticket)

## Files to change
- NEW `scripts/core/resource_map.gd`
- `scripts/core/world_generator.gd` — init resources after biome generation

## Step-by-step
1. Create resource_map.gd extending RefCounted
2. Three PackedFloat32Array: _food, _wood, _stone (width × height)
3. Methods:
   - init_resources(width, height) — allocate arrays
   - populate_from_biomes(world_data, rng) — fill based on biome-resource table
   - get_food/wood/stone(x, y) → float
   - set_food/wood/stone(x, y, val)
   - harvest(x, y, resource_type, amount) → float (actual harvested)
   - get_max_for_biome(biome, resource_type) → float (for regen cap)
   - to_save_data() / load_save_data()
4. In world_generator.gd generate():
   - After biome generation, call resource_map.populate_from_biomes()
5. Use RNG for randomized initial amounts within biome ranges

## Done Definition
- ResourceMap stores per-tile food/wood/stone
- WorldGenerator populates resources based on biomes
- No SCRIPT ERROR in headless
