# Ticket 300: GameConfig Phase 1 Constants [Foundation]

## Objective
Add all Phase 1 constants to GameConfig: resource types, building types, job ratios, tick intervals for new systems.

## Dependencies
None (first ticket)

## Non-goals
- No new autoloads
- No system implementations

## Files to change
- `scripts/core/game_config.gd`

## Step-by-step
1. Add resource type enum: `enum Resource { FOOD, WOOD, STONE }`
2. Add biome-resource mapping dictionary (biome → {food_min, food_max, wood_min, wood_max, stone_min, stone_max})
3. Add resource regen rates: FOOD_REGEN=0.5, WOOD_REGEN=0.3, STONE_REGEN=0.0
4. Add resource regen tick interval: RESOURCE_REGEN_TICK_INTERVAL=100
5. Add building type definitions:
   - BUILDING_TYPES dict with cost, build_ticks, effect_radius
   - stockpile: {cost: {wood:3}, build_ticks:50, radius:8}
   - shelter: {cost: {wood:5, stone:2}, build_ticks:80, radius:0}
   - campfire: {cost: {wood:2}, build_ticks:30, radius:5}
6. Add job ratios: JOB_RATIOS = {gatherer:0.4, lumberjack:0.3, builder:0.2, miner:0.1}
7. Add new system tick intervals:
   - GATHERING_TICK_INTERVAL=3
   - CONSTRUCTION_TICK_INTERVAL=5
   - BUILDING_EFFECT_TICK_INTERVAL=10
   - JOB_ASSIGNMENT_TICK_INTERVAL=50
   - POPULATION_TICK_INTERVAL=100
8. Add entity constants: MAX_CARRY=10.0, GATHER_AMOUNT=1.0
9. Add population constants: BIRTH_FOOD_COST=5.0, OLD_AGE_TICKS=8640, MAX_AGE_TICKS=17280
10. Add A* constant: PATHFIND_MAX_STEPS=200

## Done Definition
- GameConfig has all Phase 1 constants
- No SCRIPT ERROR in headless
