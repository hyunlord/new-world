extends SceneTree
const Seats = preload("res://scripts/ui/camp_seats.gd")

# Frozen pre-cache assignment algorithm, independent of the candidate update path.
class ReferenceSeats:
	extends RefCounted
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
		return result

	func _rebuild(hearths: Array[Vector2i], blocked: Dictionary) -> void:
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
		if not bool(row.get("visible", false)) or _walking(row):
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


class ObservedSeats:
	extends "res://scripts/ui/camp_seats.gd"
	var free_calls: int = 0
	func _free(point: Vector2) -> bool:
		free_calls += 1
		return super._free(point)

var failures: int = 0

func check(ok: bool, message: String) -> void:
	if not ok:
		push_error(message)
		failures += 1

func agent(agent_id: int) -> Dictionary:
	return {"id": agent_id, "tile": Vector2i(10, 10), "ipos": Vector2(616, 202),
		"visible": true, "moving": false, "activity": "idle", "stationary_seconds": 0.3,
		"pose": "idle", "meal": false}

func _initialize() -> void:
	_test_reuse_and_invalidation()
	_test_histories()
	print("CAMP SEAT CACHE P10 %s" % ("PASS" if failures == 0 else "FAIL"))
	quit(1 if failures else 0)

func _test_reuse_and_invalidation() -> void:
	var seats := ObservedSeats.new()
	var reference := ReferenceSeats.new()
	var hearths: Array[Vector2i] = [Vector2i(10, 10)]
	var rows: Array[Dictionary] = [agent(1), agent(2)]
	# An unassigned but eligible claimant makes recomputation observable via _free.
	rows[1]["ipos"] = Vector2(-999, -999)
	check(seats.input_cache_enabled(), "cache defaults enabled")
	var result: Dictionary = seats.update(hearths, {}, rows)
	check(result == reference.update(hearths, {}, rows), "initial result matches original")
	var calls: int = seats.free_calls
	result[1] = Vector2(-500, -500)
	result[999] = Vector2.ZERO
	check(seats.update(hearths, {}, rows) == reference.update(hearths, {}, rows), "caller cannot mutate cached result")
	check(seats.free_calls == calls, "identical inputs skip reassignment")
	rows[0]["stationary_seconds"] = 8.0
	rows[1]["stationary_seconds"] = 9.0
	seats.update(hearths, {}, rows)
	check(seats.free_calls == calls, "elapsed stationary time above threshold reuses cache")
	rows.reverse()
	seats.update(hearths, {}, rows)
	check(seats.free_calls == calls, "row order is canonical")
	var ignored: Dictionary = agent(1)
	ignored["visible"] = false
	rows.push_front(ignored)
	seats.update(hearths, {}, rows)
	check(seats.free_calls == calls, "duplicate IDs use the last row only")
	rows.remove_at(0)
	rows.reverse()
	var mutations: Array[Dictionary] = [
		{"visible": false}, {"visible": true}, {"moving": true}, {"moving": false},
		{"activity": "carry_wood"}, {"activity": "idle"}, {"pose": "eat"},
		{"meal": true}, {"meal": false}, {"pose": "idle"},
		{"stationary_seconds": 0.299}, {"stationary_seconds": 0.3},
		{"tile": Vector2i(17, 10)}, {"tile": Vector2i(10, 10)},
		{"ipos": Vector2(620, 205)}]
	for mutation: Dictionary in mutations:
		for key: String in mutation:
			rows[0][key] = mutation[key]
		calls = seats.free_calls
		check(seats.update(hearths, {}, rows) == reference.update(hearths, {}, rows), "changed input preserves original: %s" % str(mutation))
		check(seats.free_calls > calls, "changed input recomputes: %s" % str(mutation))
	rows.append(agent(3))
	check(seats.update(hearths, {}, rows) == reference.update(hearths, {}, rows), "new claimant")
	rows.remove_at(0)
	check(seats.update(hearths, {}, rows) == reference.update(hearths, {}, rows), "removed claimant")
	var blocked: Dictionary = {Vector2i(9, 10): true}
	check(seats.update(hearths, blocked, rows) == reference.update(hearths, blocked, rows), "blocked set changes")
	blocked[Vector2i(11, 10)] = true
	check(seats.update(hearths, blocked, rows) == reference.update(hearths, blocked, rows), "caller mutates blocked dictionary")
	hearths.append(Vector2i(12, 10))
	check(seats.update(hearths, blocked, rows) == reference.update(hearths, blocked, rows), "hearth rebuild")
	calls = seats.free_calls
	hearths.reverse()
	seats.update(hearths, blocked, rows)
	check(seats.free_calls == calls, "hearth order is canonical")
	seats.set_input_cache_enabled(false)
	check(not seats.input_cache_enabled(), "cache can be disabled")
	for index: int in range(2):
		calls = seats.free_calls
		check(seats.update(hearths, blocked, rows) == reference.update(hearths, blocked, rows), "disabled cache preserves assignment history")
		check(seats.free_calls > calls, "disabled cache recomputes")
	seats.set_input_cache_enabled(true)
	calls = seats.free_calls
	check(seats.update(hearths, blocked, rows) == reference.update(hearths, blocked, rows), "reenabling preserves history")
	check(seats.free_calls > calls, "reenabling invalidates cache")
	seats.clear()
	reference.clear()
	check(seats.update(hearths, blocked, rows) == reference.update(hearths, blocked, rows), "clear resets assignments and cache")
	check(seats.update([], {}, rows).is_empty(), "empty hearths release all seats")

func _test_histories() -> void:
	var seats := Seats.new()
	var uncached := Seats.new()
	uncached.set_input_cache_enabled(false)
	var reference := ReferenceSeats.new()
	var hearths: Array[Vector2i] = [Vector2i(10, 10), Vector2i(13, 12)]
	var rows: Array[Dictionary] = []
	for index: int in range(30):
		rows.append(agent(index + 1))
	var blocked: Dictionary = {}
	for step: int in range(180):
		var row: Dictionary = rows[(step * 7) % rows.size()]
		match step % 12:
			0: row["stationary_seconds"] = 0.299
			1: row["stationary_seconds"] = 0.3
			2: row["visible"] = not bool(row["visible"])
			3: row["activity"] = "carry" if step % 24 == 3 else "idle"
			4: row["tile"] = Vector2i(8 + step % 9, 10)
			5: row["ipos"] = Vector2(584 + step % 80, 202)
			6: row["meal"] = not bool(row["meal"])
			7: row["pose"] = "eat" if step % 24 == 7 else "sit"
			8: blocked[Vector2i(8 + step % 7, 9)] = true
			9:
				rows.remove_at(0)
				rows.append(agent(100 + step))
			10: rows.reverse()
			11: row["stationary_seconds"] = 6.0
		if step == 60:
			hearths.append(Vector2i(9, 9))
		if step == 120:
			seats.clear()
			uncached.clear()
			reference.clear()
		var expected: Dictionary = reference.update(hearths, blocked, rows)
		check(seats.update(hearths, blocked, rows) == expected, "history step %d cached matches frozen algorithm" % step)
		check(uncached.update(hearths, blocked, rows) == expected, "history step %d uncached matches frozen algorithm" % step)
		check(seats.update(hearths, blocked, rows) == reference.update(hearths, blocked, rows), "history step %d cache hit matches incremental algorithm" % step)
