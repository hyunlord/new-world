# Ticket 350: ResourceRegenSystem + WorldRenderer Resource Tinting [Batch 2]

## Objective
Create resource regeneration system and add resource-based color tinting to world rendering.

## Dependencies
- 310 (ResourceMap)

## Files to change
- NEW `scripts/systems/resource_regen_system.gd`
- `scripts/ui/world_renderer.gd` — add resource tinting to render

## Step-by-step
1. resource_regen_system.gd (extends simulation_system.gd):
   - priority=5, tick_interval=GameConfig.RESOURCE_REGEN_TICK_INTERVAL
   - init(resource_map, world_data)
   - execute_tick: iterate all tiles, regen food/wood based on biome, cap at max
   - Stone does NOT regen
2. world_renderer.gd modification:
   - Accept resource_map reference via init()
   - In render_world(): blend resource amounts into tile colors
   - Food-rich tiles: slightly brighter green
   - Wood-rich tiles: slightly darker green
   - Stone-rich tiles: slightly lighter gray
   - Use lerp between base biome color and tinted color based on resource ratio

## Done Definition
- Resources regenerate over time (food, wood only)
- World colors subtly reflect resource density
- No SCRIPT ERROR in headless
