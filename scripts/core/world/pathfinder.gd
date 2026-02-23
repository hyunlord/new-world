extends RefCounted


## A* pathfinding with 8-directional movement and Chebyshev heuristic
func find_path(world_data: RefCounted, from: Vector2i, to: Vector2i, max_steps: int = 200) -> Array:
	if from == to:
		return [from]
	if not world_data.is_walkable(to.x, to.y):
		return []

	var open_set: Array = [from]
	var in_open: Dictionary = {from: true}
	var came_from: Dictionary = {}
	var g_score: Dictionary = {}
	var f_score: Dictionary = {}
	var closed_set: Dictionary = {}

	g_score[from] = 0.0
	f_score[from] = _chebyshev(from, to)

	var steps: int = 0

	while open_set.size() > 0 and steps < max_steps:
		steps += 1

		# Find node with lowest f_score in open set
		var best_idx: int = 0
		var best_f: float = f_score.get(open_set[0], 999999.0)
		for i in range(1, open_set.size()):
			var f: float = f_score.get(open_set[i], 999999.0)
			if f < best_f:
				best_f = f
				best_idx = i

		var current: Vector2i = open_set[best_idx]

		if current == to:
			return _reconstruct_path(came_from, current)

		open_set.remove_at(best_idx)
		in_open.erase(current)
		closed_set[current] = true

		# Expand all 8 neighbors
		for dy in range(-1, 2):
			for dx in range(-1, 2):
				if dx == 0 and dy == 0:
					continue
				var nx: int = current.x + dx
				var ny: int = current.y + dy
				var neighbor: Vector2i = Vector2i(nx, ny)

				if closed_set.has(neighbor):
					continue
				if not world_data.is_walkable(nx, ny):
					continue

				var move_cost: float = 1.0 if (absi(dx) + absi(dy) == 1) else 1.414
				var terrain_cost: float = world_data.get_move_cost(nx, ny)
				var tentative_g: float = g_score.get(current, 999999.0) + move_cost * terrain_cost

				if tentative_g < g_score.get(neighbor, 999999.0):
					came_from[neighbor] = current
					g_score[neighbor] = tentative_g
					f_score[neighbor] = tentative_g + _chebyshev(neighbor, to)

					if not in_open.has(neighbor):
						open_set.append(neighbor)
						in_open[neighbor] = true

	return []


## Chebyshev distance (supports diagonal movement)
func _chebyshev(a: Vector2i, b: Vector2i) -> float:
	return float(maxi(absi(a.x - b.x), absi(a.y - b.y)))


## Reconstruct path from came_from map
func _reconstruct_path(came_from: Dictionary, current: Vector2i) -> Array:
	var path: Array = [current]
	while came_from.has(current):
		current = came_from[current]
		path.append(current)
	path.reverse()
	return path
