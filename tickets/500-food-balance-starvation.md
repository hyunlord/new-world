# T-500: Food Balance & Starvation Grace Period [Critical]

## Problem
- HUNGER_DECAY_RATE=0.005 per 2 ticks → full hunger drains in 40s at 1x
- Instant death at hunger=0 with no recovery chance
- Eating restores only 0.4 hunger (eat_amount*0.2) per gather_food cycle
- GRASSLAND food 3-5 too low, FOOD_REGEN_RATE 0.5 too slow
- GATHER_AMOUNT 1.0 too low

## Changes
### game_config.gd
- HUNGER_DECAY_RATE: 0.005 → 0.002
- ENERGY_DECAY_RATE: 0.003 → 0.002
- GRASSLAND food: 3-5 → 5-10
- FOREST food: 1-2 → 2-5
- FOOD_REGEN_RATE: 0.5 → 1.0
- RESOURCE_REGEN_TICK_INTERVAL: 100 → 50
- GATHER_AMOUNT: 1.0 → 2.0
- Add STARVATION_GRACE_TICKS: 50
- Add FOOD_HUNGER_RESTORE: 0.3

### entity_data.gd
- Add starving_timer: int = 0
- Serialize/deserialize

### needs_system.gd
- Starvation grace: increment starving_timer when hunger=0, kill at 50
- Auto-eat from inventory when hunger < 0.5

### movement_system.gd
- Increase hunger restore per food: 0.2 → 0.3
- Auto-eat on any idle transition when hungry

## Done Definition
- 1x speed 3min: 20 → 15+ survive
- HUD Food > 0 regularly

## Dependencies: none
