# Ticket 400: BehaviorSystem Expansion [Batch 3]

## Objective
Add new actions (gather, build, deliver, take) and job-based score bonuses to Utility AI.

## Dependencies
- 310 (ResourceMap), 320 (inventory/job), 340 (BuildingManager), 360 (GatheringSystem), 390 (jobs)

## Files to change
- `scripts/ai/behavior_system.gd`

## Step-by-step
1. Add new references: init(entity_manager, world_data, rng, resource_map, building_manager)
2. Add new actions to _evaluate_actions:
   - gather_food: score based on hunger_deficit + nearby food resources
   - gather_wood: base score 0.3 if wood tiles nearby
   - gather_stone: base score 0.2 if stone tiles nearby
   - deliver_to_stockpile: high score when inventory > 7.0 and stockpile exists
   - build: score when building_manager has unbuilt buildings
   - take_from_stockpile: score when hungry and stockpile has food
3. Apply job bonuses:
   - gatherer: gather_food *= 1.5
   - lumberjack: gather_wood *= 1.5
   - builder: build *= 1.5
   - miner: gather_stone *= 1.5
4. Update _assign_action for new actions:
   - gather_*: find nearest resource tile, set target, timer=20
   - deliver_to_stockpile: find nearest stockpile, set target, timer=30
   - build: find nearest unbuilt building, set target, timer=25
   - take_from_stockpile: find nearest stockpile with food, set target, timer=15
5. Keep existing actions (wander, seek_food, rest, socialize)
6. Replace seek_food with gather_food (seek_food as alias)

## Done Definition
- Agents choose from expanded action set
- Job bonuses influence decisions
- New actions have proper targets and timers
- No SCRIPT ERROR in headless
