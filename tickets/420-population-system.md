# Ticket 420: PopulationSystem [Batch 4]

## Objective
Implement population growth (births when food sufficient) and natural death (old age).

## Dependencies
- 340 (BuildingManager for shelter/stockpile checks), 320 (EntityData age)

## Files to change
- NEW `scripts/systems/population_system.gd`

## Step-by-step
1. population_system.gd (extends simulation_system.gd):
   - priority=50, tick_interval=GameConfig.POPULATION_TICK_INTERVAL
   - init(entity_manager, building_manager, rng)
   - execute_tick:
     - BIRTHS: Check conditions (stockpile food >= pop*2, pop < MAX_ENTITIES, shelters*4 > pop)
       - If met: consume food 5 from stockpile, spawn new entity near stockpile
       - Emit "entity_born"
     - NATURAL DEATH: For each entity with age > OLD_AGE_TICKS:
       - If age > MAX_AGE_TICKS: 10% chance of death per check
       - Elif age > OLD_AGE_TICKS: 2% chance
       - Emit "entity_died" with cause "old_age"

## Done Definition
- Population grows when conditions met
- Old agents die naturally
- Food consumed for births
- Events emitted
- No SCRIPT ERROR in headless
