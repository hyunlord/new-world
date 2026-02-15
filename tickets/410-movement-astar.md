# Ticket 410: MovementSystem A* Upgrade [Batch 3]

## Objective
Replace greedy movement with A* pathfinding, using cached paths.

## Dependencies
- 330 (Pathfinder), 320 (EntityData cached_path)

## Files to change
- `scripts/systems/movement_system.gd`

## Step-by-step
1. Add pathfinder reference: init(entity_manager, world_data, pathfinder)
2. In execute_tick, for each moving entity:
   - If cached_path is empty or action changed: recalculate path
   - If path_index < cached_path.size(): move to cached_path[path_index], increment
   - If path exhausted or invalid: fall back to greedy movement
3. Path recalculation triggers:
   - action_target changed (new action)
   - Every 50 ticks (recompute stale paths)
   - Path blocked (next tile became impassable)
4. Keep greedy fallback for when A* returns empty path
5. Keep existing arrival effects

## Done Definition
- Agents follow A* paths around obstacles
- Greedy fallback when no path found
- Path caching prevents per-tick recalculation
- No SCRIPT ERROR in headless
