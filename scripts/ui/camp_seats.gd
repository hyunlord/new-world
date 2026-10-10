class_name CampSeats
extends RefCounted

## Render-only, persistent camp positions. Caller supplies all obstructed tiles.
var _hearths: Array[Vector2i] = []
var _blocked: Dictionary = {}
var _candidates: Array[Dictionary] = []
var _assignments: Dictionary = {}

func clear() -> void:
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
	for agent_id: int in _assignments.keys():
		var seat: Dictionary = _assignments[agent_id]
		if not rows.has(agent_id) or not _eligible(rows[agent_id], seat["hearth"]):
			_assignments.erase(agent_id)
	var ids: Array = rows.keys()
	ids.sort()
	for agent_id: int in ids:
		if _assignments.has(agent_id):
			continue
		var row: Dictionary = rows[agent_id]
		var nearest: Dictionary = {}
		var best_distance: float = 48.0 * 48.0
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
	return result

func _rebuild(hearths: Array[Vector2i], blocked: Dictionary) -> void:
	_hearths = hearths.duplicate()
	_blocked = blocked.duplicate()
	_candidates.clear()
	for hearth: Vector2i in _hearths:
		var feet := Vector2(456.0 + 16.0 * hearth.x, 42.0 + 16.0 * hearth.y)
		for j: int in range(-4, 5):
			for i: int in range(-6, 7):
				var offset := Vector2(16 * i + 8 * posmod(j, 2), 14 * j)
				if pow(offset.x / 88.0, 2) + pow(offset.y / 60.0, 2) > 1.0 or offset.length() < 14.0:
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
		if point.distance_squared_to(seat["position"]) < 14.0 * 14.0:
			return false
	return true

static func _eligible(row: Dictionary, hearth: Vector2i) -> bool:
	if not bool(row.get("visible", false)) or bool(row.get("moving", true)):
		return false
	if float(row.get("stationary_seconds", 0.0)) < 0.3:
		return false
	if not bool(row.get("meal", false)) and str(row.get("pose", "")) not in ["idle", "sit", "hop"]:
		return false
	var delta: Vector2i = (row["tile"] as Vector2i) - hearth
	return maxi(absi(delta.x), absi(delta.y)) <= 4

static func _tile_before(a: Vector2i, b: Vector2i) -> bool:
	return a.y < b.y if a.y != b.y else a.x < b.x

static func spread_offsets(agents: Array[Dictionary], seated: Dictionary) -> Dictionary:
	var groups: Dictionary = {}
	for row: Dictionary in agents:
		var agent_id: int = int(row["id"])
		if bool(row.get("moving", true)) or seated.has(agent_id):
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
