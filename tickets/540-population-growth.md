# T-540: Population Growth Relaxation [Medium]

## Problem
- Birth needs stockpile food >= pop*2 (impossible without stockpiles)
- Shelter capacity only 4 per shelter
- BIRTH_FOOD_COST = 5.0 too high
- POPULATION_TICK_INTERVAL = 100 too slow

## Changes
### game_config.gd
- BIRTH_FOOD_COST: 5.0 → 3.0
- POPULATION_TICK_INTERVAL: 100 → 60

### population_system.gd
- Food threshold: pop*2.0 → pop*1.0
- Shelter capacity: 4 → 6 per shelter
- Allow births up to 25 pop without shelters
- Min alive count for births: 5 (don't grow from 1-2 entities)

## Done Definition
- 10min at 10x: pop 20 → 30+
- Console shows entity_born

## Dependencies: T-520
