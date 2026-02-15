# Ticket 220: Agent Movement Speed Tuning [Medium]

## Objective
Slow down 1x agent movement from ~10 tiles/sec to ~3 tiles/sec for a relaxed god-sim feel.

## Non-goals
- No pathfinding changes
- No new movement mechanics

## Files to change
- `scripts/core/game_config.gd` — add tick interval constants, adjust decay rates
- `scripts/systems/movement_system.gd` — tick_interval = 3
- `scripts/ai/behavior_system.gd` — tick_interval = 10
- `scripts/systems/needs_system.gd` — tick_interval = 2, use new decay rates

## Steps
1. Add to GameConfig:
   - MOVEMENT_TICK_INTERVAL = 3
   - BEHAVIOR_TICK_INTERVAL = 10
   - NEEDS_TICK_INTERVAL = 2
2. Update tick_interval in each system's _init()
3. Adjust decay rates for 2-tick needs interval:
   - HUNGER_DECAY_RATE: 0.003 → 0.005 (compensate for 2x interval)
   - ENERGY_DECAY_RATE: 0.002 → 0.003
   - ENERGY_ACTION_COST: 0.004 → 0.006
   - SOCIAL_DECAY_RATE: 0.001 → 0.002

## Done Definition
- 1x: agents move ~3 tiles/sec (relaxed observation pace)
- Agents survive several in-game days before starvation risk
- GameConfig has tick interval constants
