class_name CampSeats
extends RefCounted

## Render-only, persistent camp positions. Caller supplies all obstructed tiles.
var _hearths: Array[Vector2i] = []
var _blocked: Dictionary = {}
var _candidates: Array[Dictionary] = []
var _assignments: Dictionary = {}
var _input_cache_enabled: bool = true
var _input_cache_valid: bool = false
var _input_cache_rows: Array = []
var _input_cache_result: Dictionary = {}

func set_input_cache_enabled(enabled: bool) -> void:
	if _input_cache_enabled != enabled:
		_input_cache_enabled = enabled
		_invalidate_input_cache()

func input_cache_enabled() -> bool:
	return _input_cache_enabled

func _invalidate_input_cache() -> void:
	_input_cache_valid = false
	_input_cache_rows.clear()
	_input_cache_result.clear()

func clear() -> void:
	_invalidate_input_cache()
	_hearths.clear()
	_blocked.clear()
	_candidates.clear()
	_assignments.clear()

func update(hearths: Array[Vector2i], blocked: Dictionary, agents: Array[Dictionary]) -> Dictionary:
	var ordered_hearths: Array[Vector2i] = hearths.duplicate()
	ordered_hearths.sort_custom(_tile_before)
	if ordered_hearths != _hearths or blocked != _blocked:
		_rebuild(ordered_hearths, blocked)
	var rows: Dictionary = {}
	for row: Dictionary in agents:
		rows[int(row["id"])] = row
	var ids: Array = rows.keys()
	ids.sort()
	var input_rows: Array = []
	if _input_cache_enabled:
		for agent_id: int in ids:
			var row: Dictionary = rows[agent_id]
			input_rows.append([agent_id, bool(row.get("visible", false)), _walking(row),
				bool(row.get("activity_place_known", false)),
				not (float(row.get("stationary_seconds", 0.0)) < 0.3),
				bool(row.get("meal", false)), str(row.get("pose", "")), row["tile"], row["ipos"]])
		if _input_cache_valid and input_rows == _input_cache_rows:
			return _input_cache_result.duplicate()
	for agent_id: int in _assignments.keys():
		var seat: Dictionary = _assignments[agent_id]
		if not rows.has(agent_id) or not _eligible(rows[agent_id], seat["hearth"]):
			_assignments.erase(agent_id)
	for agent_id: int in ids:
		if _assignments.has(agent_id):
			continue
		var row: Dictionary = rows[agent_id]
		var nearest: Dictionary = {}
		var best_distance: float = 64.0 * 64.0
		for candidate: Dictionary in _candidates:
			if not _eligible(row, candidate["hearth"]) or not _free(candidate["position"]):
				continue
			var distance: float = (row["ipos"] as Vector2).distance_squared_to(candidate["position"])
			if distance > best_distance:
				continue
			if nearest.is_empty() or distance < best_distance:
				nearest = candidate
				best_distance = distance
		if not nearest.is_empty():
			_assignments[agent_id] = nearest
	var result: Dictionary = {}
	for agent_id: int in _assignments:
		result[agent_id] = _assignments[agent_id]["position"]
	if _input_cache_enabled:
		_input_cache_rows = input_rows
		_input_cache_result = result.duplicate()
		_input_cache_valid = true
	return result

func _rebuild(hearths: Array[Vector2i], blocked: Dictionary) -> void:
	_invalidate_input_cache()
	_hearths = hearths.duplicate()
	_blocked = blocked.duplicate()
	_candidates.clear()
	for hearth: Vector2i in _hearths:
		var feet := Vector2(456.0 + 16.0 * hearth.x, 42.0 + 16.0 * hearth.y)
		for j: int in range(-4, 5):
			for i: int in range(-7, 8):
				var offset := Vector2(16 * i + 8 * posmod(j, 2), 18 * j)
				if pow(offset.x / 104.0, 2) + pow(offset.y / 72.0, 2) > 1.0 or offset.length() < 16.0:
					continue
				var point: Vector2 = feet + offset
				var tile := Vector2i(((point - Vector2(456, 42)) / 16.0).round())
				if _blocked.has(tile) or _hearths.has(tile):
					continue
				_candidates.append({"position": point, "hearth": hearth, "i": i, "j": j})
	_candidates.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		if int(a["j"]) != int(b["j"]):
			return int(a["j"]) < int(b["j"])
		if int(a["i"]) != int(b["i"]):
			return int(a["i"]) < int(b["i"])
		return _tile_before(a["hearth"], b["hearth"])
	)
	for agent_id: int in _assignments.keys():
		if not _candidates.has(_assignments[agent_id]):
			_assignments.erase(agent_id)

func _free(point: Vector2) -> bool:
	for seat: Dictionary in _assignments.values():
		if point.distance_squared_to(seat["position"]) < 16.0 * 16.0:
			return false
	return true

static func _eligible(row: Dictionary, hearth: Vector2i) -> bool:
	if bool(row.get("activity_place_known", false)) or not bool(row.get("visible", false)) or _walking(row):
		return false
	if float(row.get("stationary_seconds", 0.0)) < 0.3:
		return false
	if not bool(row.get("meal", false)) and str(row.get("pose", "")) not in ["idle", "sit", "hop", "crossed"]:
		return false
	var delta: Vector2i = (row["tile"] as Vector2i) - hearth
	return maxi(absi(delta.x), absi(delta.y)) <= 6

static func _walking(row: Dictionary) -> bool:
	return bool(row.get("moving", true)) or str(row.get("activity", row.get("pose", ""))) in ["walk", "run", "carry", "carry_wood"]

static func _tile_before(a: Vector2i, b: Vector2i) -> bool:
	return a.y < b.y if a.y != b.y else a.x < b.x

static func spread_offsets(agents: Array[Dictionary], seated: Dictionary) -> Dictionary:
	var groups: Dictionary = {}
	for row: Dictionary in agents:
		var agent_id: int = int(row["id"])
		if (_walking(row) and not bool(row.get("activity_place_known", false))) or seated.has(agent_id):
			continue
		var tile: Vector2i = row["tile"]
		if not groups.has(tile):
			groups[tile] = []
		groups[tile].append(agent_id)
	var result: Dictionary = {}
	for ids: Array in groups.values():
		ids.sort()
		for rank: int in range(ids.size()):
			var offset := Vector2.ZERO
			match ids.size():
				2: offset = [Vector2(-5, 0), Vector2(5, 0)][rank]
				3: offset = [Vector2(-6, -2), Vector2(6, -2), Vector2(0, 3)][rank]
				4: offset = [Vector2(-6, -3), Vector2(6, -3), Vector2(-6, 3), Vector2(6, 3)][rank]
				_:
					if ids.size() >= 5:
						offset = Vector2.from_angle(TAU * float(rank) / float(ids.size())) * 7.0
			result[int(ids[rank])] = offset
	return result

## Rects are manifest canvas rectangles relative to each agent's feet.
## Only drawing targets change; simulation positions remain untouched.
static func declump(rows: Array[Dictionary], _seated: Dictionary, targets: Dictionary, terrain: Dictionary, blocked: Dictionary, rects: Dictionary, use_spatial_hash: bool = true) -> Dictionary:
	var ordered: Array[Dictionary] = []
	for row: Dictionary in rows:
		var agent_id: int = int(row["id"])
		if bool(row.get("visible", false)) and targets.has(agent_id) and rects.has(agent_id):
			ordered.append(row)
	ordered.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		var ay: float = (targets[int(a["id"])] as Vector2).y
		var by: float = (targets[int(b["id"])] as Vector2).y
		return ay > by if ay != by else int(a["id"]) > int(b["id"])
	)
	var fronts: Array[Dictionary] = []
	var spatial: SpatialIndex = SpatialIndex.new(_spatial_cell_size(rects)) if use_spatial_hash else null
	var result: Dictionary = {}
	for row: Dictionary in ordered:
		var agent_id: int = int(row["id"])
		var target: Vector2 = targets[agent_id]
		var local_rect: Rect2 = rects[agent_id]
		var offset := Vector2.ZERO
		if not bool(row.get("activity_place_known", false)) and not _walking(row):
			if _covered(Rect2(target + local_rect.position, local_rect.size), fronts, spatial) >= 0.30:
				var nearest: Vector2 = target
				var distance: float = INF
				for front: Dictionary in fronts:
					var front_target: Vector2 = front["target"]
					var candidate_distance: float = target.distance_squared_to(front_target)
					if candidate_distance < distance:
						distance = candidate_distance
						nearest = front_target
				var side: float = 1.0 if target.x > nearest.x else -1.0
				if target.x == nearest.x:
					side = 1.0 if posmod(agent_id, 2) == 0 else -1.0
				var best_coverage: float = _covered(Rect2(target + local_rect.position, local_rect.size), fronts, spatial)
				var found: bool = false
				for magnitude: int in range(1, 9):
					for direction: float in [side, -side]:
						var shift := Vector2(direction * magnitude, 0.0)
						if not _declump_allowed(target + shift, terrain, blocked):
							continue
						var coverage: float = _covered(Rect2(target + shift + local_rect.position, local_rect.size), fronts, spatial)
						if coverage < best_coverage:
							best_coverage = coverage
							offset = shift
						if coverage < 0.30:
							found = true
							break
					if found:
						break
			result[agent_id] = offset
		if spatial != null:
			spatial.insert(Rect2(target + offset + local_rect.position, local_rect.size), agent_id)
		fronts.append({"target": target + offset, "rect": Rect2(target + offset + local_rect.position, local_rect.size)})
	return result

static func _covered(rect: Rect2, fronts: Array[Dictionary], spatial: SpatialIndex = null) -> float:
	var area: float = rect.get_area()
	if area <= 0.0:
		return 0.0
	if spatial != null:
		return minf(spatial.overlap(rect) / area, 1.0)
	var overlap: float = 0.0
	for front: Dictionary in fronts:
		overlap += rect.intersection(front["rect"]).get_area()
	return minf(overlap / area, 1.0)

static func _declump_allowed(point: Vector2, terrain: Dictionary, blocked: Dictionary) -> bool:
	var tile := Vector2i(((point - Vector2(456, 42)) / 16.0).round())
	if blocked.has(tile):
		return false
	var width: int = int(terrain.get("width", 0))
	var height: int = int(terrain.get("height", 0))
	var cells: PackedByteArray = terrain.get("terrain", PackedByteArray())
	var index: int = tile.y * width + tile.x
	return tile.x >= 0 and tile.x < width and tile.y >= 0 and tile.y < height and index < cells.size() and cells[index] != 4

## Screen-pixel lateral relief uses the already rounded drawing positions.
static func walker_offsets(rows: Array[Dictionary], positions: Dictionary, rects: Dictionary, terrain: Dictionary, blocked: Dictionary, canvas: Transform2D, use_spatial_hash: bool = true) -> Dictionary:
	var ordered: Array[Dictionary] = []
	for row: Dictionary in rows:
		var agent_id: int = int(row["id"])
		if bool(row.get("visible", false)) and positions.has(agent_id) and rects.has(agent_id):
			ordered.append(row)
	ordered.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a["id"]) < int(b["id"]))
	var spatial: SpatialIndex = SpatialIndex.new(_spatial_cell_size(rects)) if use_spatial_hash else null
	if spatial != null:
		for row: Dictionary in ordered:
			var agent_id: int = int(row["id"])
			var local_rect: Rect2 = rects[agent_id]
			spatial.insert(Rect2(Vector2(positions[agent_id]) + local_rect.position, local_rect.size), agent_id)
	var result: Dictionary = {}
	var inverse: Transform2D = canvas.affine_inverse()
	for row: Dictionary in ordered:
		if bool(row.get("activity_place_known", false)) or not _walking(row):
			continue
		var agent_id: int = int(row["id"])
		var position: Vector2 = positions[agent_id]
		var direction: Vector2 = canvas.basis_xform(Vector2(row.get("walking_direction", Vector2.ZERO))).normalized()
		if direction.is_zero_approx():
			continue
		var local_rect: Rect2 = rects[agent_id]
		var others: Array[Rect2] = []
		if spatial == null:
			for other: Dictionary in ordered:
				var other_id: int = int(other["id"])
				if other_id != agent_id:
					var other_rect: Rect2 = rects[other_id]
					others.append(Rect2(Vector2(positions[other_id]) + Vector2(result.get(other_id, Vector2.ZERO)) + other_rect.position, other_rect.size))
		var best: float = _walker_overlap(Rect2(position + local_rect.position, local_rect.size), others, spatial, agent_id)
		var offset := Vector2.ZERO
		if best <= 0.0:
			continue
		var side: float = 1.0 if posmod(agent_id, 2) == 0 else -1.0
		var perpendicular := Vector2(-direction.y, direction.x)
		for magnitude: int in range(1, 4):
			for sign_value: float in [side, -side]:
				var screen_shift: Vector2 = (perpendicular * float(magnitude) * sign_value).round()
				if screen_shift.length() > 3.0 or absf(screen_shift.dot(direction)) > 0.001:
					continue
				var shift: Vector2 = inverse.basis_xform(screen_shift)
				if not _declump_allowed(position + shift, terrain, blocked):
					continue
				var overlap: float = _walker_overlap(Rect2(position + shift + local_rect.position, local_rect.size), others, spatial, agent_id)
				if overlap < best - 0.00001:
					best = overlap
					offset = shift
		result[agent_id] = offset
		if spatial != null:
			spatial.move_agent(agent_id, Rect2(position + offset + local_rect.position, local_rect.size))
	return result

static func _walker_overlap(rect: Rect2, others: Array[Rect2], spatial: SpatialIndex = null, agent_id: int = 0) -> float:
	if spatial != null:
		return spatial.overlap(rect, agent_id, true)
	var area: float = 0.0
	for other: Rect2 in others:
		area += rect.intersection(other).get_area()
	return area

static func _spatial_cell_size(rects: Dictionary) -> float:
	var extent: float = 0.0
	for rect: Rect2 in rects.values():
		extent = maxf(extent, maxf(absf(rect.size.x), absf(rect.size.y)))
	return maxf(8.0, ceilf(extent + 8.0))

class SpatialIndex:
	extends RefCounted
	var cell_size: float
	var rectangles: Array[Rect2] = []
	var agent_ids: Array[int] = []
	var buckets: Dictionary = {}
	var ordinals_by_agent: Dictionary = {}

	func _init(size: float) -> void:
		cell_size = size

	func cells(rect: Rect2) -> Array[Vector2i]:
		var bounds: Rect2 = rect.abs()
		var first := Vector2i(floori(bounds.position.x / cell_size), floori(bounds.position.y / cell_size))
		var last := Vector2i(floori(bounds.end.x / cell_size), floori(bounds.end.y / cell_size))
		var result: Array[Vector2i] = []
		for y: int in range(first.y, last.y + 1):
			for x: int in range(first.x, last.x + 1):
				result.append(Vector2i(x, y))
		return result

	func insert(rect: Rect2, agent_id: int) -> void:
		var ordinal: int = rectangles.size()
		rectangles.append(rect)
		agent_ids.append(agent_id)
		if not ordinals_by_agent.has(agent_id):
			ordinals_by_agent[agent_id] = []
		ordinals_by_agent[agent_id].append(ordinal)
		_add_cells(ordinal, rect)

	func _add_cells(ordinal: int, rect: Rect2) -> void:
		for cell: Vector2i in cells(rect):
			if not buckets.has(cell):
				buckets[cell] = []
			buckets[cell].append(ordinal)

	func move_agent(agent_id: int, rect: Rect2) -> void:
		for ordinal: int in ordinals_by_agent[agent_id]:
			for cell: Vector2i in cells(rectangles[ordinal]):
				buckets[cell].erase(ordinal)
				if buckets[cell].is_empty():
					buckets.erase(cell)
			rectangles[ordinal] = rect
			_add_cells(ordinal, rect)

	func query(rect: Rect2) -> Array:
		var found: Dictionary = {}
		for cell: Vector2i in cells(rect):
			for ordinal: int in buckets.get(cell, []):
				found[ordinal] = true
		var ordered: Array = found.keys()
		ordered.sort()
		return ordered

	func overlap(rect: Rect2, excluded_id: int = 0, exclude_agent: bool = false) -> float:
		var area: float = 0.0
		for ordinal: int in query(rect):
			if not exclude_agent or agent_ids[ordinal] != excluded_id:
				area += rect.intersection(rectangles[ordinal]).get_area()
		return area
