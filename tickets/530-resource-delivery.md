# T-530: Resource Delivery & Auto-Eat [Medium]

## Problem
- Deliver threshold 7.0/10.0 = 70% full before delivering
- Entities hoard food in inventory instead of depositing
- No auto-eat: entities only eat after gather_food action completes

## Changes
### behavior_system.gd (already modified in T-510)
- Deliver threshold changes already in T-510

### movement_system.gd
- Auto-eat on ANY action completion: if hungry and has food, eat
- Increase take_from_stockpile hunger restore: 0.15 → 0.25 per food

## Done Definition
- Entities deliver resources to stockpile regularly
- HUD Food shows increasing values

## Dependencies: T-500, T-520
