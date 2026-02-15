# Ticket 370: ConstructionSystem [Batch 2]

## Objective
Create system that progresses building construction when builders are present.

## Dependencies
- 340 (BuildingManager), 320 (EntityData inventory)

## Files to change
- NEW `scripts/systems/construction_system.gd`

## Step-by-step
1. construction_system.gd (extends simulation_system.gd):
   - priority=28, tick_interval=GameConfig.CONSTRUCTION_TICK_INTERVAL
   - init(entity_manager, building_manager)
   - execute_tick:
     - For each alive entity with action == "build":
       - Get building at action_target
       - If no building or already built, skip
       - Check entity is at or adjacent to building tile
       - build_progress += 0.05
       - If build_progress >= 1.0: is_built = true, emit "building_completed"

## Done Definition
- Builders progress construction
- Buildings complete when progress reaches 1.0
- Events emitted
- No SCRIPT ERROR in headless
