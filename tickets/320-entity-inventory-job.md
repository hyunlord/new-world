# Ticket 320: EntityData Inventory + Job Fields [Foundation]

## Objective
Add inventory dict, job field, and cached_path to EntityData. Add helper methods.

## Dependencies
- 300 (GameConfig constants for MAX_CARRY)

## Non-goals
- No job assignment logic
- No gathering logic

## Files to change
- `scripts/core/entity_data.gd`

## Step-by-step
1. Add new fields:
   - var inventory: Dictionary = {"food": 0.0, "wood": 0.0, "stone": 0.0}
   - var job: String = "none"
   - var cached_path: Array = []  # Array[Vector2i] for A*
   - var path_index: int = 0
2. Add helper methods:
   - add_item(type: String, amount: float) → float (actual added, respects MAX_CARRY)
   - remove_item(type: String, amount: float) → float (actual removed)
   - get_total_carry() → float
   - has_item(type: String, min_amount: float) → bool
3. Update to_dict() to include inventory, job, (not cached_path — runtime only)
4. Update from_dict() to restore inventory and job

## Done Definition
- EntityData has inventory, job, cached_path fields
- Helper methods work correctly
- Serialization includes inventory and job
- No SCRIPT ERROR in headless
