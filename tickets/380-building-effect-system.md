# Ticket 380: BuildingEffectSystem [Batch 2]

## Objective
Apply passive effects from completed buildings to nearby agents.

## Dependencies
- 340 (BuildingManager), 320 (EntityData)

## Files to change
- NEW `scripts/systems/building_effect_system.gd`

## Step-by-step
1. building_effect_system.gd (extends simulation_system.gd):
   - priority=15, tick_interval=GameConfig.BUILDING_EFFECT_TICK_INTERVAL
   - init(entity_manager, building_manager, sim_engine)
   - execute_tick:
     - For each completed building:
       - campfire: agents within radius 5 get social += 0.01
         - If night (hour 20-6): social += 0.02 instead
       - shelter: agents ON the tile get energy *= 2.0 recovery bonus
         (applied by marking entity, NeedsSystem checks this)
       - stockpile: handled by behavior/delivery logic, not here

## Done Definition
- Campfire boosts social for nearby agents
- Shelter boosts energy recovery
- Night bonus for campfire
- No SCRIPT ERROR in headless
