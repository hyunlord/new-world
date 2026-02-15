# Ticket 330: A* Pathfinder [Foundation]

## Objective
Create Pathfinder (RefCounted) with A* algorithm for 8-directional movement.

## Dependencies
None (uses WorldData.is_walkable which already exists)

## Non-goals
- No MovementSystem integration (separate ticket)
- No building avoidance

## Files to change
- NEW `scripts/core/pathfinder.gd`

## Step-by-step
1. Create pathfinder.gd extending RefCounted
2. Implement find_path(world_data: RefCounted, from: Vector2i, to: Vector2i, max_steps: int = 200) → Array:
   - A* with open set (Array sorted by f-score, or manual min extraction)
   - g_score: Dictionary (Vector2i → float)
   - came_from: Dictionary (Vector2i → Vector2i)
   - 8-directional neighbors
   - Heuristic: Chebyshev distance (max of dx, dy)
   - Cost: 1.0 for cardinal, 1.414 for diagonal, multiplied by biome move cost
   - Skip impassable tiles (is_walkable == false)
   - Return path as Array[Vector2i] from start to end (inclusive)
   - Return empty array if no path found or max_steps exceeded
3. Helper: _chebyshev(a: Vector2i, b: Vector2i) → float
4. Helper: _get_neighbors(pos: Vector2i, world_data: RefCounted) → Array[Vector2i]
5. Use explicit types (no :=) for headless compat

## Done Definition
- Pathfinder.find_path returns valid A* path
- Handles impassable tiles
- Returns empty array when no path
- No SCRIPT ERROR in headless
