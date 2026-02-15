# Ticket 430: Visual Upgrade - Entity + Building Renderers [Batch 4]

## Objective
Draw agents as job-specific shapes and buildings as distinct shapes.

## Dependencies
- 320 (job field), 340 (BuildingManager)

## Files to change
- `scripts/ui/entity_renderer.gd` — job-based shapes
- NEW `scripts/ui/building_renderer.gd` — building shapes
- `scenes/main/main.tscn` — add BuildingRenderer node

## Step-by-step
1. entity_renderer.gd: Replace uniform circles with job-based shapes:
   - none: circle 3px gray
   - gatherer: circle 4px green
   - lumberjack: triangle 5px brown
   - builder: square 5px orange
   - miner: diamond 4px blue-gray
   - Carrying resources: small dot above (resource color)
   - Selection: white arc (keep existing)
2. building_renderer.gd (extends Node2D):
   - init(building_manager)
   - _draw(): For each building:
     - stockpile: brown outline rect 12px
     - shelter: brown filled triangle 14px
     - campfire: orange/red circle 6px
     - Under construction: same but alpha 0.4
3. Add BuildingRenderer to main.tscn between EntityRenderer and Camera

## Done Definition
- Agents visually distinct by job
- Buildings rendered as shapes
- Construction shown as translucent
- No SCRIPT ERROR in headless
