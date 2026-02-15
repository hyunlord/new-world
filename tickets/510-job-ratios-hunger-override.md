# T-510: Job Ratios & Hunger Override [Critical]

## Problem
- JOB_RATIOS: 40% gatherer → 60% NOT gathering food → Wood:284 Food:0
- Lumberjacks ignore hunger until hunger < 0.45
- No dynamic rebalancing based on food levels

## Changes
### game_config.gd
- JOB_RATIOS: gatherer 0.5, lumberjack 0.25, builder 0.15, miner 0.1

### job_assignment_system.gd
- Small pop (<10): gatherer 0.8, lumberjack 0.1, builder 0.1, miner 0.0
- Add dynamic rebalancing: reassign jobs if food crisis detected
- Reassign existing entities (not just unassigned)

### behavior_system.gd
- Hunger override: when hunger < 0.3, ALL jobs force gather_food = 1.0
- Lower deliver_to_stockpile threshold: 7.0 → 3.0
- Gradual deliver score: carry > 3.0 → 0.6, carry > 6.0 → 0.9

## Done Definition
- Wood-only surplus doesn't occur
- All jobs eat when hungry

## Dependencies: T-500
