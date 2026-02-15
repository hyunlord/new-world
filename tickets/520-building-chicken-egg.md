# T-520: Building Cost & Chicken-and-Egg Fix [Critical]

## Problem
- Stockpile costs wood:3 but builder doesn't gather wood
- Construction progress fixed at 0.05, ignores build_ticks config
- Builder sits idle when can't afford building

## Changes
### game_config.gd
- stockpile cost: wood 3 → 2, build_ticks 50 → 30
- shelter cost: wood 5 + stone 2 → wood 4 + stone 1, build_ticks 80 → 50
- campfire cost: wood 2 → 1, build_ticks 30 → 20

### construction_system.gd
- Calculate progress_per_tick from build_ticks config instead of hardcoded 0.05

### behavior_system.gd
- Builder should gather_wood when can't afford any building (new fallback)
- Non-builder entities with wood in inventory: score build higher when no stockpile exists

## Done Definition
- 5min at 5x: at least 1 stockpile built
- Console shows building_completed

## Dependencies: T-510
