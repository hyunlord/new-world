extends RefCounted

const _RUST_BRIDGE_NODE_NAME: String = "SimBridge"
const _RUST_BRIDGE_METHOD_NAME: String = "pathfind_grid"
const _RUST_BRIDGE_METHOD_XY_NAME: String = "pathfind_grid_xy"
const _RUST_BRIDGE_BATCH_METHOD_NAME: String = "pathfind_grid_batch"
const _RUST_BRIDGE_BATCH_XY_METHOD_NAME: String = "pathfind_grid_batch_xy"

var _bridge_checked: bool = false
var _rust_bridge: Object = null

var _cached_world_data: RefCounted = null
var _cached_width: int = 0
var _cached_height: int = 0
var _cached_walkable: PackedByteArray = PackedByteArray()
var _cached_move_cost: PackedFloat32Array = PackedFloat32Array()
var _batch_from_xy: PackedInt32Array = PackedInt32Array()
var _batch_to_xy: PackedInt32Array = PackedInt32Array()
var _batch_from_points: PackedVector2Array = PackedVector2Array()
var _batch_to_points: PackedVector2Array = PackedVector2Array()


## A* pathfinding with 8-directional movement and Chebyshev heuristic
func find_path(world_data: RefCounted, from: Vector2i, to: Vector2i, max_steps: int = 200) -> Array:
	if from == to:
		return [from]
	if not world_data.is_walkable(to.x, to.y):
		return []

	var rust_result: Dictionary = _find_path_rust(world_data, from, to, max_steps)
	if bool(rust_result.get("used", false)):
		return rust_result.get("path", [])

	return _find_path_gd(world_data, from, to, max_steps)

## Batch pathfinding for multiple (from,to) requests.
## Each request item must be a Dictionary: {"from": Vector2i, "to": Vector2i}
func find_paths_batch(world_data: RefCounted, requests: Array, max_steps: int = 200) -> Array:
	if requests.is_empty():
		return []

	var rust_batch: Dictionary = _find_paths_rust_batch(world_data, requests, max_steps)
	if bool(rust_batch.get("used", false)):
		return rust_batch.get("paths", [])

	var out: Array = []
	for i in range(requests.size()):
		var req: Dictionary = requests[i]
		var from: Vector2i = req.get("from", Vector2i(-1, -1))
		var to: Vector2i = req.get("to", Vector2i(-1, -1))
		out.append(find_path(world_data, from, to, max_steps))
	return out


## Batch pathfinding for packed XY request pairs.
## Inputs are [x0, y0, x1, y1, ...] for from/to arrays.
func find_paths_batch_xy(
	world_data: RefCounted,
	from_xy: PackedInt32Array,
	to_xy: PackedInt32Array,
	max_steps: int = 200
) -> Array:
	if from_xy.is_empty() or to_xy.is_empty():
		return []
	if from_xy.size() != to_xy.size() or (from_xy.size() % 2) != 0:
		return []

	var rust_batch: Dictionary = _find_paths_rust_batch_xy(world_data, from_xy, to_xy, max_steps)
	if bool(rust_batch.get("used", false)):
		return rust_batch.get("paths", [])

	var out: Array = []
	var pair_count: int = mini(from_xy.size(), to_xy.size()) / 2
	for i in range(pair_count):
		var base_idx: int = i * 2
		var from: Vector2i = Vector2i(from_xy[base_idx], from_xy[base_idx + 1])
		var to: Vector2i = Vector2i(to_xy[base_idx], to_xy[base_idx + 1])
		out.append(find_path(world_data, from, to, max_steps))
	return out


func _find_path_rust(world_data: RefCounted, from: Vector2i, to: Vector2i, max_steps: int) -> Dictionary:
	var bridge: Object = _get_rust_bridge()
	if bridge == null:
		return {"used": false, "path": []}
	var has_xy: bool = bridge.has_method(_RUST_BRIDGE_METHOD_XY_NAME)
	var has_vec2: bool = bridge.has_method(_RUST_BRIDGE_METHOD_NAME)
	if not has_xy and not has_vec2:
		return {"used": false, "path": []}

	_ensure_world_cache(world_data)
	if _cached_width <= 0 or _cached_height <= 0:
		return {"used": false, "path": []}

	var result: Variant = null
	var used_xy: bool = false
	if has_xy:
		result = bridge.call(
			_RUST_BRIDGE_METHOD_XY_NAME,
			_cached_width,
			_cached_height,
			_cached_walkable,
			_cached_move_cost,
			from.x,
			from.y,
			to.x,
			to.y,
			max_steps
		)
		used_xy = (result != null)
	if result == null and has_vec2:
		result = bridge.call(
			_RUST_BRIDGE_METHOD_NAME,
			_cached_width,
			_cached_height,
			_cached_walkable,
			_cached_move_cost,
			from.x,
			from.y,
			to.x,
			to.y,
			max_steps
		)
	if result == null:
		return {"used": false, "path": []}
	if used_xy:
		return {"used": true, "path": _normalize_path_xy_result(result)}
	return {"used": true, "path": _normalize_path_result(result)}


func _find_paths_rust_batch(world_data: RefCounted, requests: Array, max_steps: int) -> Dictionary:
	var bridge: Object = _get_rust_bridge()
	if bridge == null:
		return {"used": false, "paths": []}
	var has_batch_xy: bool = bridge.has_method(_RUST_BRIDGE_BATCH_XY_METHOD_NAME)
	var has_batch_vec2: bool = bridge.has_method(_RUST_BRIDGE_BATCH_METHOD_NAME)
	if not has_batch_xy and not has_batch_vec2:
		return {"used": false, "paths": []}

	_ensure_world_cache(world_data)
	if _cached_width <= 0 or _cached_height <= 0:
		return {"used": false, "paths": []}

	var result: Variant = null
	var used_batch_xy: bool = false
	if has_batch_xy:
		var pair_len: int = requests.size() * 2
		if _batch_from_xy.size() != pair_len:
			_batch_from_xy.resize(pair_len)
		if _batch_to_xy.size() != pair_len:
			_batch_to_xy.resize(pair_len)
		for i in range(requests.size()):
			var req_xy: Dictionary = requests[i]
			var from_xy_point: Vector2i = req_xy.get("from", Vector2i(-1, -1))
			var to_xy_point: Vector2i = req_xy.get("to", Vector2i(-1, -1))
			var base_idx: int = i * 2
			_batch_from_xy[base_idx] = from_xy_point.x
			_batch_from_xy[base_idx + 1] = from_xy_point.y
			_batch_to_xy[base_idx] = to_xy_point.x
			_batch_to_xy[base_idx + 1] = to_xy_point.y
		result = bridge.call(
			_RUST_BRIDGE_BATCH_XY_METHOD_NAME,
			_cached_width,
			_cached_height,
			_cached_walkable,
			_cached_move_cost,
			_batch_from_xy,
			_batch_to_xy,
			max_steps
		)
		used_batch_xy = (result != null)
	if result == null and has_batch_vec2:
		if _batch_from_points.size() != requests.size():
			_batch_from_points.resize(requests.size())
		if _batch_to_points.size() != requests.size():
			_batch_to_points.resize(requests.size())
		for i in range(requests.size()):
			var req: Dictionary = requests[i]
			var from: Vector2i = req.get("from", Vector2i(-1, -1))
			var to: Vector2i = req.get("to", Vector2i(-1, -1))
			_batch_from_points[i] = Vector2(from.x, from.y)
			_batch_to_points[i] = Vector2(to.x, to.y)

		result = bridge.call(
			_RUST_BRIDGE_BATCH_METHOD_NAME,
			_cached_width,
			_cached_height,
			_cached_walkable,
			_cached_move_cost,
			_batch_from_points,
			_batch_to_points,
			max_steps
		)

	if result == null:
		return {"used": false, "paths": []}

	if used_batch_xy:
		if not (result is Array):
			return {"used": false, "paths": []}
		var xy_groups: Array = result
		var xy_normalized: Array = []
		for i in range(xy_groups.size()):
			xy_normalized.append(_normalize_path_xy_result(xy_groups[i]))
		return {"used": true, "paths": xy_normalized}

	if result == null or not (result is Array):
		return {"used": false, "paths": []}

	var normalized: Array = []
	var groups: Array = result
	for i in range(groups.size()):
		normalized.append(_normalize_path_result(groups[i]))
	return {"used": true, "paths": normalized}


func _find_paths_rust_batch_xy(
	world_data: RefCounted,
	from_xy: PackedInt32Array,
	to_xy: PackedInt32Array,
	max_steps: int
) -> Dictionary:
	var bridge: Object = _get_rust_bridge()
	if bridge == null:
		return {"used": false, "paths": []}
	var has_batch_xy: bool = bridge.has_method(_RUST_BRIDGE_BATCH_XY_METHOD_NAME)
	var has_batch_vec2: bool = bridge.has_method(_RUST_BRIDGE_BATCH_METHOD_NAME)
	if not has_batch_xy and not has_batch_vec2:
		return {"used": false, "paths": []}

	_ensure_world_cache(world_data)
	if _cached_width <= 0 or _cached_height <= 0:
		return {"used": false, "paths": []}

	var result: Variant = null
	var used_batch_xy: bool = false
	if has_batch_xy:
		result = bridge.call(
			_RUST_BRIDGE_BATCH_XY_METHOD_NAME,
			_cached_width,
			_cached_height,
			_cached_walkable,
			_cached_move_cost,
			from_xy,
			to_xy,
			max_steps
		)
		used_batch_xy = (result != null)

	if result == null and has_batch_vec2:
		var pair_count: int = mini(from_xy.size(), to_xy.size()) / 2
		if _batch_from_points.size() != pair_count:
			_batch_from_points.resize(pair_count)
		if _batch_to_points.size() != pair_count:
			_batch_to_points.resize(pair_count)
		for i in range(pair_count):
			var base_idx: int = i * 2
			_batch_from_points[i] = Vector2(from_xy[base_idx], from_xy[base_idx + 1])
			_batch_to_points[i] = Vector2(to_xy[base_idx], to_xy[base_idx + 1])
		result = bridge.call(
			_RUST_BRIDGE_BATCH_METHOD_NAME,
			_cached_width,
			_cached_height,
			_cached_walkable,
			_cached_move_cost,
			_batch_from_points,
			_batch_to_points,
			max_steps
		)

	if result == null:
		return {"used": false, "paths": []}

	if used_batch_xy:
		if not (result is Array):
			return {"used": false, "paths": []}
		var xy_groups: Array = result
		var xy_normalized: Array = []
		for i in range(xy_groups.size()):
			xy_normalized.append(_normalize_path_xy_result(xy_groups[i]))
		return {"used": true, "paths": xy_normalized}

	if not (result is Array):
		return {"used": false, "paths": []}
	var normalized: Array = []
	var groups: Array = result
	for i in range(groups.size()):
		normalized.append(_normalize_path_result(groups[i]))
	return {"used": true, "paths": normalized}


func _normalize_path_xy_result(result: Variant) -> Array:
	var path: Array = []
	if result is PackedInt32Array:
		var packed: PackedInt32Array = result
		var pair_count: int = packed.size() / 2
		for i in range(pair_count):
			var base_idx: int = i * 2
			path.append(Vector2i(packed[base_idx], packed[base_idx + 1]))
		return path
	if result is Array:
		var arr: Array = result
		for i in range(arr.size()):
			var item: Variant = arr[i]
			if item is Vector2i:
				path.append(item)
	return path


func _get_rust_bridge() -> Object:
	if _bridge_checked:
		return _rust_bridge
	_bridge_checked = true

	var tree: SceneTree = Engine.get_main_loop() as SceneTree
	if tree != null and tree.root != null:
		var node_from_root: Node = tree.root.get_node_or_null(_RUST_BRIDGE_NODE_NAME)
		if node_from_root != null:
			_rust_bridge = node_from_root
			return _rust_bridge

	if Engine.has_singleton(_RUST_BRIDGE_NODE_NAME):
		_rust_bridge = Engine.get_singleton(_RUST_BRIDGE_NODE_NAME)

	return _rust_bridge


func _ensure_world_cache(world_data: RefCounted) -> void:
	var width: int = int(world_data.width)
	var height: int = int(world_data.height)
	var expected_size: int = width * height
	var needs_rebuild: bool = (
		world_data != _cached_world_data
		or width != _cached_width
		or height != _cached_height
		or _cached_walkable.size() != expected_size
		or _cached_move_cost.size() != expected_size
	)
	if not needs_rebuild:
		return

	_cached_world_data = world_data
	_cached_width = width
	_cached_height = height
	_cached_walkable.resize(expected_size)
	_cached_move_cost.resize(expected_size)

	var idx: int = 0
	for y in range(height):
		for x in range(width):
			var walkable: bool = world_data.is_walkable(x, y)
			_cached_walkable[idx] = 1 if walkable else 0
			_cached_move_cost[idx] = world_data.get_move_cost(x, y)
			idx += 1


func _normalize_path_result(result: Variant) -> Array:
	var path: Array = []
	if result is PackedVector2Array:
		var packed: PackedVector2Array = result
		for i in range(packed.size()):
			var p: Vector2 = packed[i]
			path.append(Vector2i(int(round(p.x)), int(round(p.y))))
		return path

	if result is Array:
		var arr: Array = result
		for i in range(arr.size()):
			var item: Variant = arr[i]
			if item is Vector2i:
				path.append(item)
			elif item is Vector2:
				var p2: Vector2 = item
				path.append(Vector2i(int(round(p2.x)), int(round(p2.y))))
			elif item is Dictionary:
				var d: Dictionary = item
				if d.has("x") and d.has("y"):
					path.append(Vector2i(int(d["x"]), int(d["y"])))
	return path


func _find_path_gd(world_data: RefCounted, from: Vector2i, to: Vector2i, max_steps: int = 200) -> Array:
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
