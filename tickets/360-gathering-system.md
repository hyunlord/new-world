# Ticket 360: GatheringSystem [Batch 2]

## Objective
Create system that lets agents harvest resources from tiles into their inventory.

## Dependencies
- 310 (ResourceMap), 320 (EntityData inventory)

## Files to change
- NEW `scripts/systems/gathering_system.gd`

## Step-by-step
1. gathering_system.gd (extends simulation_system.gd):
   - priority=25, tick_interval=GameConfig.GATHERING_TICK_INTERVAL
   - init(entity_manager, resource_map)
   - execute_tick:
     - For each alive entity with action in ["gather_food", "gather_wood", "gather_stone"]:
       - Determine resource type from action name
       - Check tile has resource >= 0.5
       - Check entity carry capacity
       - amount = min(GATHER_AMOUNT * entity.speed, available, remaining_capacity)
       - resource_map.harvest(x, y, type, amount)
       - entity.add_item(type, amount)
       - emit "resource_gathered" event

## Done Definition
- Agents with gather actions extract resources from tiles
- Resources decrease on tiles
- Entity inventory increases
- Events emitted
- No SCRIPT ERROR in headless
