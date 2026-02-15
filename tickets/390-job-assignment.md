# Ticket 390: JobAssignmentSystem [Batch 2]

## Objective
Automatically assign jobs to agents based on population needs and resource availability.

## Dependencies
- 320 (EntityData job field), 340 (BuildingManager for stockpile checks)

## Files to change
- NEW `scripts/systems/job_assignment_system.gd`

## Step-by-step
1. job_assignment_system.gd (extends simulation_system.gd):
   - priority=8, tick_interval=GameConfig.JOB_ASSIGNMENT_TICK_INTERVAL
   - init(entity_manager, building_manager)
   - execute_tick:
     - Count current job distribution
     - Calculate target counts from JOB_RATIOS × alive_count
     - If pop < 10: override to mostly gatherer (70% gatherer, 20% lumberjack, 10% builder, 0% miner)
     - For each "none" job entity: assign most-needed job
     - Emit "job_assigned" event

## Done Definition
- Agents get jobs assigned based on ratios
- Small populations prioritize food gathering
- Events emitted
- No SCRIPT ERROR in headless
