# Frozen badge grid/packing reference before bitmap/pruning optimization; test-only.
# Bridge methods: none. Source SHA256: 6754898b06c07368211036a6acef8f9858b134fc42ba18d39cd034b5e3b89a2b
extends RefCounted

static var _profile_enabled: bool = false
static var _profile_search: Dictionary = {}
static var profile_rows: Array[Dictionary] = []

static func _finish_badge(rectangle: Rect2, exhausted: bool) -> Rect2:
	if _profile_enabled:
		_profile_search["elapsed_usec"] = Time.get_ticks_usec() - int(_profile_search["started_usec"])
		_profile_search["exhausted"] = exhausted
		_profile_search["result"] = [rectangle.position.x, rectangle.position.y, rectangle.size.x, rectangle.size.y]
		_profile_search.erase("started_usec")
		profile_rows.append(_profile_search)
	return rectangle


static func _profile_bucket(index: Dictionary, cell: Vector2i) -> void:
	if _profile_enabled and not _profile_search.is_empty():
		_profile_search["bucket_visits"] = int(_profile_search["bucket_visits"]) + 1
		_profile_search["bucket_entries"] = int(_profile_search["bucket_entries"]) + (index.get(cell, []) as Array).size()


static func _profile_obstacle() -> void:
	if _profile_enabled and not _profile_search.is_empty():
		_profile_search["obstacle_checks"] = int(_profile_search["obstacle_checks"]) + 1


static func badge_grid(maximum: Vector2, viewport: Rect2, blocked: Array[Rect2]) -> Dictionary:
	var size: Vector2 = maximum.ceil()
	var origin: Vector2 = viewport.position.ceil() + Vector2.ONE
	var pitch: Vector2 = size + Vector2(4, 4)
	var counts: Vector2i = Vector2i(((viewport.end - Vector2.ONE - origin - size) / pitch).floor()) + Vector2i.ONE
	counts = counts.max(Vector2i.ZERO)
	var rows: Array = []
	for y: int in counts.y:
		var columns: Array[int] = []
		for x: int in counts.x:
			columns.append(x)
		rows.append(columns)
	var grid: Dictionary = {"origin": origin, "pitch": pitch, "size": size, "counts": counts, "rows": rows, "exhausted": 0}
	for obstacle: Rect2 in blocked:
		badge_grid_reserve(grid, obstacle)
	return grid


static func badge_grid_reserve(grid: Dictionary, rectangle: Rect2) -> void:
	var origin: Vector2 = grid["origin"]
	var pitch: Vector2 = grid["pitch"]
	var size: Vector2 = grid["size"]
	var counts: Vector2i = grid["counts"]
	var obstacle: Rect2 = rectangle.grow(2.0)
	var first: Vector2i = Vector2i(((obstacle.position - origin - size) / pitch).floor()).max(Vector2i.ZERO)
	var last: Vector2i = Vector2i(((obstacle.end - origin) / pitch).ceil()).min(counts - Vector2i.ONE)
	for y: int in range(first.y, last.y + 1):
		var row: Array[int] = grid["rows"][y]
		for x: int in range(first.x, last.x + 1):
			if Rect2(origin + Vector2(x, y) * pitch, size).intersects(obstacle):
				var at: int = row.bsearch(x)
				if at < row.size() and row[at] == x:
					row.remove_at(at)


static func _label_free(label: Rect2, viewport: Rect2, index: Dictionary) -> bool:
	if _profile_enabled:
		_profile_search["candidates"] = int(_profile_search["candidates"]) + 1
	if not viewport.encloses(label.grow(1.0)):
		return false
	var first: Vector2i = Vector2i((label.position / 64.0).floor())
	var last: Vector2i = Vector2i((label.end / 64.0).floor())
	for y: int in range(first.y, last.y + 1):
		for x: int in range(first.x, last.x + 1):
			_profile_bucket(index, Vector2i(x, y))
			for obstacle: Rect2 in index.get(Vector2i(x, y), []):
				_profile_obstacle()
				if label.intersects(obstacle.grow(2.0)):
					return false
	return true


static func badge_packed(figure: Rect2, size: Vector2, viewport: Rect2, index: Dictionary, grid: Dictionary) -> Rect2:
	if _profile_enabled:
		_profile_search = {"started_usec": Time.get_ticks_usec(), "policy": "label_slots_clipped_leaders", "candidates": 0, "obstacle_checks": 0, "bucket_visits": 0, "bucket_entries": 0, "grid_rows_examined": 0, "figure": [figure.position.x, figure.position.y, figure.size.x, figure.size.y], "size": [size.x, size.y], "viewport": [viewport.position.x, viewport.position.y, viewport.size.x, viewport.size.y]}
	var preferred: Vector2 = Vector2(figure.get_center().x - size.x * 0.5, figure.position.y - 4.0 - size.y).round()
	var nearby: Array[Vector2] = [preferred, Vector2(figure.position.x - 4.0 - size.x, figure.get_center().y - size.y * 0.5), Vector2(figure.end.x + 4.0, figure.get_center().y - size.y * 0.5), Vector2(figure.get_center().x - size.x * 0.5, figure.end.y + 4.0)]
	for position: Vector2 in nearby:
		var candidate: Rect2 = Rect2(position.round(), size)
		if _label_free(candidate, viewport, index):
			badge_grid_reserve(grid, candidate)
			return _finish_badge(candidate, false)
	var origin: Vector2 = grid["origin"]
	var pitch: Vector2 = grid["pitch"]
	var slot_size: Vector2 = grid["size"]
	var counts: Vector2i = grid["counts"]
	var wanted: float = (figure.get_center().x - origin.x - slot_size.x * 0.5) / pitch.x
	var best_distance: float = INF
	var chosen: Rect2 = Rect2(preferred, size)
	for y: int in counts.y:
		if _profile_enabled:
			_profile_search["grid_rows_examined"] = int(_profile_search["grid_rows_examined"]) + 1
		var row: Array[int] = grid["rows"][y]
		var at: int = row.bsearch(ceili(wanted))
		for offset: int in [at - 1, at]:
			if offset < 0 or offset >= row.size():
				continue
			var position: Vector2 = (origin + Vector2(row[offset], y) * pitch + (slot_size - size) * 0.5).floor()
			var candidate: Rect2 = Rect2(position, size)
			var distance: float = candidate.get_center().distance_squared_to(figure.get_center())
			if distance < best_distance:
				chosen = candidate
				best_distance = distance
	if best_distance < INF:
		badge_grid_reserve(grid, chosen)
		return _finish_badge(chosen, false)
	grid["exhausted"] = int(grid["exhausted"]) + 1
	badge_grid_reserve(grid, chosen)
	return _finish_badge(chosen, true)
