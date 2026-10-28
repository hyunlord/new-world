# Screen-only marker placement: retain true anchors, separate visible discs/glyphs and avoid visible UI surfaces.
# Bridge methods: none. Stable input ID order determines priority; off-screen anchors remain off screen.
extends RefCounted

static var profile_rows: Array[Dictionary] = []
static var profile_geometry: Array = []
static var _profile_enabled: bool = OS.get_environment("WORLDSIM_MUX2_RENDER_PROFILE") == "1"
static var _profile_search: Dictionary = {}


static func reset_profile() -> void:
	profile_rows.clear()
	profile_geometry.clear()
	_profile_search = {}


static func _finish_badge(rectangle: Rect2, exhausted: bool) -> Rect2:
	if _profile_enabled:
		_profile_search["elapsed_usec"] = Time.get_ticks_usec() - int(_profile_search["started_usec"])
		_profile_search["exhausted"] = exhausted
		_profile_search["result"] = [rectangle.position.x, rectangle.position.y, rectangle.size.x, rectangle.size.y]
		_profile_search.erase("started_usec")
		profile_rows.append(_profile_search)
	return rectangle


## Place a group label clear of every figure/UI rectangle, retaining its preferred position above the group when free.
static func badge(figure: Rect2, size: Vector2, viewport: Rect2, blocked: Array[Rect2], index: Dictionary = {}) -> Rect2:
	if index.is_empty():
		index = badge_index(blocked)
	if _profile_enabled:
		_profile_search = {"started_usec": Time.get_ticks_usec(), "candidates": 0, "viewport_rejects": 0, "obstacle_checks": 0, "bucket_visits": 0, "bucket_entries": 0, "size": [size.x, size.y], "viewport": [viewport.position.x, viewport.position.y, viewport.size.x, viewport.size.y], "obstacles": blocked.size(), "figure_on_screen": figure.intersects(viewport), "figure": [figure.position.x, figure.position.y, figure.size.x, figure.size.y]}
	var gap: float = 4.0
	var preferred: Vector2 = Vector2(figure.get_center().x - size.x * 0.5, figure.position.y - gap - size.y).round()
	if badge_start_blocked(figure, index):
		if _profile_enabled:
			_profile_search["blocked_start_proof"] = true
		return _finish_badge(Rect2(preferred, size), true)
	var nearby: Array[Vector2] = [preferred,
		Vector2(figure.position.x - gap - size.x, figure.get_center().y - size.y * 0.5),
		Vector2(figure.end.x + gap, figure.get_center().y - size.y * 0.5),
		Vector2(figure.get_center().x - size.x * 0.5, figure.end.y + gap)]
	for position: Vector2 in nearby:
		var candidate: Rect2 = Rect2(position.round(), size)
		if _badge_free(candidate, figure, viewport, index):
			return _finish_badge(candidate, false)
	var step: float = maxf(size.x, size.y) + gap
	var limit: int = ceili(maxf(viewport.size.x, viewport.size.y) / step)
	# Conservative lattice bounds skip only off-screen candidates; the exact viewport predicate remains authoritative.
	var low: Vector2i = Vector2i(((viewport.position - preferred - Vector2.ONE) / step).floor())
	var high: Vector2i = Vector2i(((viewport.end - size - preferred + Vector2.ONE) / step).ceil())
	for ring: int in range(1, limit + 1):
		for y: int in range(maxi(-ring, low.y), mini(ring, high.y) + 1):
			if absi(y) == ring:
				for x: int in range(maxi(-ring, low.x), mini(ring, high.x) + 1):
					var candidate: Rect2 = Rect2((preferred + Vector2(x, y) * step).round(), size)
					if _badge_free(candidate, figure, viewport, index):
						return _finish_badge(candidate, false)
			else:
				for x: int in [-ring, ring]:
					if x < low.x or x > high.x:
						continue
					var candidate: Rect2 = Rect2((preferred + Vector2(x, y) * step).round(), size)
					if _badge_free(candidate, figure, viewport, index):
						return _finish_badge(candidate, false)
	# A completely occupied viewport has no truthful free placement; preserve the cue for visual failure reporting.
	return _finish_badge(Rect2(preferred, size), true)


## Index grown obstacles once per screen-space pass; appending a badge reserves it for later groups.
static func badge_index(blocked: Array[Rect2]) -> Dictionary:
	var index: Dictionary = {}
	if _profile_enabled and profile_geometry.is_empty():
		for rectangle: Rect2 in blocked:
			profile_geometry.append([rectangle.position.x, rectangle.position.y, rectangle.size.x, rectangle.size.y])
	for rectangle: Rect2 in blocked:
		badge_index_add(index, rectangle)
	return index


static func badge_index_add(index: Dictionary, rectangle: Rect2) -> void:
	var grown: Rect2 = rectangle.grow(2.5)
	var first: Vector2i = Vector2i((grown.position / 64.0).floor())
	var last: Vector2i = Vector2i((grown.end / 64.0).floor())
	for y: int in range(first.y, last.y + 1):
		for x: int in range(first.x, last.x + 1):
			var cell: Vector2i = Vector2i(x, y)
			if not index.has(cell):
				index[cell] = []
			index[cell].append(rectangle)


static func _badge_obstacles(index: Dictionary, query: Rect2) -> Array[Rect2]:
	var unique: Dictionary = {}
	var first: Vector2i = Vector2i((query.position / 64.0).floor())
	var last: Vector2i = Vector2i((query.end / 64.0).floor())
	for y: int in range(first.y, last.y + 1):
		for x: int in range(first.x, last.x + 1):
			for rectangle: Rect2 in index.get(Vector2i(x, y), []):
				if rectangle.grow(2.5).intersects(query, true):
					unique[rectangle] = true
	var result: Array[Rect2] = []
	for rectangle: Rect2 in unique:
		result.append(rectangle)
	return result


## Every leader begins on this perimeter. Complete closed-interval coverage proves all candidates blocked.
static func badge_start_blocked(figure: Rect2, index: Dictionary) -> bool:
	var perimeter: Rect2 = figure.grow(3.0)
	var obstacles: Array[Rect2] = _badge_obstacles(index, perimeter.grow(0.001))
	var body_enclosed: bool = false
	for obstacle: Rect2 in obstacles:
		body_enclosed = body_enclosed or obstacle.encloses(figure)
	var covered_perimeter: bool = _perimeter_covered(perimeter, obstacles)
	# Union coverage includes every clamped/zero-direction leader start, not only points on the perimeter.
	var covered_area: bool = covered_perimeter and _area_covered(perimeter, obstacles)
	if _profile_enabled and not _profile_search.is_empty():
		_profile_search["perimeter_covered"] = covered_perimeter
		_profile_search["body_enclosed"] = body_enclosed
		_profile_search["expanded_area_covered"] = covered_area
	return covered_area


static func _perimeter_covered(perimeter: Rect2, obstacles: Array[Rect2]) -> bool:
	for axis: int in 2:
		var other: int = 1 - axis
		for edge: float in [perimeter.position[other], perimeter.end[other]]:
			var intervals: Array[Vector2] = []
			for obstacle: Rect2 in obstacles:
				var grown: Rect2 = obstacle.grow(2.5)
				if grown.position[other] <= edge and grown.end[other] >= edge:
					intervals.append(Vector2(grown.position[axis], grown.end[axis]))
			if not _interval_covered(intervals, perimeter.position[axis], perimeter.end[axis]):
				return false
	return true


static func _interval_covered(intervals: Array[Vector2], start: float, finish: float) -> bool:
	intervals.sort_custom(func(a: Vector2, b: Vector2) -> bool: return a.x < b.x)
	var covered: float = start
	for interval: Vector2 in intervals:
		if interval.x > covered:
			break
		covered = maxf(covered, interval.y)
	return covered >= finish


static func _area_covered(rectangle: Rect2, obstacles: Array[Rect2]) -> bool:
	var edges: Array[float] = [rectangle.position.x, rectangle.end.x]
	for obstacle: Rect2 in obstacles:
		var grown: Rect2 = obstacle.grow(2.5)
		edges.append(clampf(grown.position.x, rectangle.position.x, rectangle.end.x))
		edges.append(clampf(grown.end.x, rectangle.position.x, rectangle.end.x))
	edges.sort()
	for i: int in range(1, edges.size()):
		if edges[i] == edges[i - 1]:
			continue
		var middle: float = (edges[i] + edges[i - 1]) * 0.5
		var intervals: Array[Vector2] = []
		for obstacle: Rect2 in obstacles:
			var grown: Rect2 = obstacle.grow(2.5)
			if grown.position.x <= middle and grown.end.x >= middle:
				intervals.append(Vector2(grown.position.y, grown.end.y))
		if not _interval_covered(intervals, rectangle.position.y, rectangle.end.y):
			return false
	return true


static func badge_leader(figure: Rect2, label: Rect2) -> PackedVector2Array:
	return PackedVector2Array([_edge_toward(figure.grow(3.0), label.get_center()), _edge_toward(label, figure.get_center())])


static func _edge_toward(rectangle: Rect2, target: Vector2) -> Vector2:
	var direction: Vector2 = target - rectangle.get_center()
	var distance: float = minf(rectangle.size.x * 0.5 / maxf(absf(direction.x), 0.001), rectangle.size.y * 0.5 / maxf(absf(direction.y), 0.001))
	return rectangle.get_center() + direction * distance


static func _badge_free(label: Rect2, figure: Rect2, viewport: Rect2, index: Dictionary) -> bool:
	if _profile_enabled:
		_profile_search["candidates"] = int(_profile_search["candidates"]) + 1
	if not viewport.encloses(label.grow(1.0)):
		if _profile_enabled:
			_profile_search["viewport_rejects"] = int(_profile_search["viewport_rejects"]) + 1
		return false
	var leader: PackedVector2Array = badge_leader(figure, label)
	# Most dense-scene failures are local. Do not materialize a whole leader AABB before rejecting them.
	if _segment_cell_blocked(Vector2i((leader[0] / 64.0).floor()), leader[0], leader[0], index):
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
	return not badge_segment_blocked(leader[0], leader[1], index)


static func _profile_bucket(index: Dictionary, cell: Vector2i) -> void:
	if _profile_enabled and not _profile_search.is_empty():
		_profile_search["bucket_visits"] = int(_profile_search["bucket_visits"]) + 1
		_profile_search["bucket_entries"] = int(_profile_search["bucket_entries"]) + (index.get(cell, []) as Array).size()


static func _profile_obstacle() -> void:
	if _profile_enabled and not _profile_search.is_empty():
		_profile_search["obstacle_checks"] = int(_profile_search["obstacle_checks"]) + 1


static func _segment_cell_blocked(cell: Vector2i, start: Vector2, finish: Vector2, index: Dictionary) -> bool:
	_profile_bucket(index, cell)
	for obstacle: Rect2 in index.get(cell, []):
		_profile_obstacle()
		if segment_hits_rect(start, finish, obstacle.grow(2.5)):
			return true
	return false


## Traverse segment cells lazily. Closed grown-obstacle indexing also covers exact grid-corner contacts.
static func badge_segment_blocked(start: Vector2, finish: Vector2, index: Dictionary) -> bool:
	var cell: Vector2i = Vector2i((start / 64.0).floor())
	var last: Vector2i = Vector2i((finish / 64.0).floor())
	if _segment_cell_blocked(last, start, finish, index):
		return true
	var direction: Vector2 = finish - start
	var step: Vector2i = Vector2i(signf(direction.x), signf(direction.y))
	var limit: int = absi(last.x - cell.x) + absi(last.y - cell.y) + 1
	for unused: int in limit:
		if _segment_cell_blocked(cell, start, finish, index):
			return true
		if cell == last:
			break
		# Recompute from integer cell boundaries: repeated float32 additions can drift past a corner.
		var crossing: Array[float] = [1.0e30, 1.0e30]
		for axis: int in 2:
			if step[axis] != 0:
				var boundary: float = float(cell[axis] + (1 if step[axis] > 0 else 0)) * 64.0
				crossing[axis] = (boundary - start[axis]) / direction[axis]
		# Both adjacent cells are conservative candidates even when rounding changes crossing order.
		if step.x != 0 and _segment_cell_blocked(cell + Vector2i(step.x, 0), start, finish, index):
			return true
		if step.y != 0 and _segment_cell_blocked(cell + Vector2i(0, step.y), start, finish, index):
			return true
		var tolerance: float = 0.000001 * maxf(1.0, minf(absf(crossing[0]), absf(crossing[1])))
		if absf(crossing[0] - crossing[1]) <= tolerance:
			cell += step
		elif crossing[0] < crossing[1]:
			cell.x += step.x
		else:
			cell.y += step.y
	return false


## Clip the actual segment against both rectangle slabs; its bounding box can cover a corner the line avoids.
static func segment_hits_rect(start: Vector2, finish: Vector2, rectangle: Rect2) -> bool:
	var direction: Vector2 = finish - start
	var first: float = 0.0
	var last: float = 1.0
	for axis: int in 2:
		if absf(direction[axis]) < 0.000001:
			if start[axis] < rectangle.position[axis] or start[axis] > rectangle.end[axis]:
				return false
			continue
		var a: float = (rectangle.position[axis] - start[axis]) / direction[axis]
		var b: float = (rectangle.end[axis] - start[axis]) / direction[axis]
		first = maxf(first, minf(a, b))
		last = minf(last, maxf(a, b))
		if first > last:
			return false
	return true


static func ui_rects(node: Node, result: Array[Rect2]) -> void:
	if node is CanvasItem and not (node as CanvasItem).is_visible_in_tree():
		return
	if node is Panel or node is PanelContainer or node is BaseButton:
		var control: Control = node as Control
		var rectangle: Rect2 = control.get_global_rect()
		if rectangle.has_area():
			result.append(rectangle)
		return
	for child: Node in node.get_children():
		ui_rects(child, result)


static func place(points: PackedVector2Array, ids: PackedInt64Array, extent: float, glyphs: bool, viewport: Rect2, blocked: Array[Rect2], gap: float) -> PackedVector2Array:
	var result: PackedVector2Array = points.duplicate()
	var order: Array[int] = []
	for i: int in points.size():
		order.append(i)
	order.sort_custom(func(a: int, b: int) -> bool: return ids[a] < ids[b])
	var radius: float = extent * 0.5 + gap
	var glyph: float = 14.0 if glyphs else 0.0
	var offset: Vector2 = Vector2(radius, radius + glyph)
	var size: Vector2 = Vector2(radius * 2.0 + glyph, radius * 2.0 + glyph)
	var cell_size: float = maxf(size.x, size.y)
	var limit: int = ceili(maxf(viewport.size.x, viewport.size.y) / cell_size)
	var integral_lattice: bool = cell_size == floorf(cell_size) and offset == offset.round()
	var regions: Array[Rect2] = []
	if viewport.size.x >= size.x and viewport.size.y >= size.y:
		regions.append(viewport)
	for obstacle: Rect2 in blocked:
		regions = _marker_subtract(regions, obstacle, size)
	for index: int in order:
		var anchor: Vector2 = points[index].round()
		if not Rect2(anchor - Vector2.ONE * extent * 0.5, Vector2.ONE * extent).intersects(viewport):
			continue
		var best_ring: int = limit + 1
		var best: Vector2i = Vector2i.ZERO
		var fast_lattice: bool = integral_lattice and anchor.abs().x < 1048576 and anchor.abs().y < 1048576 and viewport.size.x < 1048576 and viewport.size.y < 1048576
		for region: Rect2 in regions:
			var first: Vector2i
			var last: Vector2i
			if fast_lattice:
				# Integer marker coordinates are exact in float32 at ordinary viewport sizes.
				first = Vector2i(ceili((region.position.x + offset.x - anchor.x) / cell_size), ceili((region.position.y + offset.y - anchor.y) / cell_size))
				last = Vector2i(floori((region.end.x - size.x + offset.x - anchor.x) / cell_size), floori((region.end.y - size.y + offset.y - anchor.y) / cell_size))
			else:
				# Expand by one cell, then trim with the same Vector2 arithmetic used to draw the rectangle.
				first = Vector2i(((region.position + offset - anchor) / cell_size).ceil()) - Vector2i.ONE
				last = Vector2i(((region.end - size + offset - anchor) / cell_size).floor()) + Vector2i.ONE
				while first.x <= last.x and (anchor + Vector2(first) * cell_size - offset).x < region.position.x:
					first.x += 1
				while first.y <= last.y and (anchor + Vector2(first) * cell_size - offset).y < region.position.y:
					first.y += 1
				while last.x >= first.x and (anchor + Vector2(last) * cell_size - offset + size).x > region.end.x:
					last.x -= 1
				while last.y >= first.y and (anchor + Vector2(last) * cell_size - offset + size).y > region.end.y:
					last.y -= 1
			if first.x > last.x or first.y > last.y:
				continue
			var ring: int = maxi(absi(clampi(0, first.x, last.x)), absi(clampi(0, first.y, last.y)))
			if ring > limit or ring > best_ring:
				continue
			# The original search orders candidates by Chebyshev ring, then row, then column.
			var candidate: Vector2i = Vector2i(maxi(first.x, -ring), maxi(first.y, -ring))
			if ring == best_ring and (candidate.y > best.y or (candidate.y == best.y and candidate.x >= best.x)):
				continue
			var rectangle: Rect2 = Rect2(anchor + Vector2(candidate) * cell_size - offset, size)
			if viewport.encloses(rectangle) and region.encloses(rectangle):
				best_ring = ring
				best = candidate
		if best_ring <= limit:
			result[index] = anchor + Vector2(best) * cell_size
			regions = _marker_subtract(regions, Rect2(result[index] - offset, size), size)
		else:
			# Exhausted visible markers retain their anchor and do not reserve space.
			result[index] = anchor
	return result


## Overlapping maximal free rectangles retain every feasible marker placement, including edge contact.
static func _marker_subtract(regions: Array[Rect2], obstacle: Rect2, size: Vector2) -> Array[Rect2]:
	var kept: Array[Rect2] = []
	var split: Array[Rect2] = []
	for region: Rect2 in regions:
		if not region.intersects(obstacle):
			kept.append(region)
			continue
		if obstacle.position.x > region.position.x:
			split.append(Rect2(region.position, Vector2(obstacle.position.x - region.position.x, region.size.y)))
		if obstacle.end.x < region.end.x:
			split.append(Rect2(Vector2(obstacle.end.x, region.position.y), Vector2(region.end.x - obstacle.end.x, region.size.y)))
		if obstacle.position.y > region.position.y:
			split.append(Rect2(region.position, Vector2(region.size.x, obstacle.position.y - region.position.y)))
		if obstacle.end.y < region.end.y:
			split.append(Rect2(Vector2(region.position.x, obstacle.end.y), Vector2(region.size.x, region.end.y - obstacle.end.y)))
	for rectangle: Rect2 in split:
		if rectangle.size.x < size.x or rectangle.size.y < size.y:
			continue
		var contained: bool = false
		for other: Rect2 in kept:
			if other.encloses(rectangle):
				contained = true
				break
		if contained:
			continue
		for i: int in range(kept.size() - 1, -1, -1):
			if rectangle.encloses(kept[i]):
				kept.remove_at(i)
		kept.append(rectangle)
	return kept


## Conservative free label slots; a fallback examines at most two columns per viewport row.
static func badge_grid(maximum: Vector2, viewport: Rect2, blocked: Array[Rect2]) -> Dictionary:
	var size: Vector2 = maximum.ceil()
	var origin: Vector2 = viewport.position.ceil() + Vector2.ONE
	var pitch: Vector2 = size + Vector2(4, 4)
	var counts: Vector2i = Vector2i(((viewport.end - Vector2.ONE - origin - size) / pitch).floor()) + Vector2i.ONE
	counts = counts.max(Vector2i.ZERO)
	# Initial reservations are monotone: mark once, then emit each sorted free row once.
	var occupied: PackedByteArray = PackedByteArray()
	occupied.resize(counts.x * counts.y)
	occupied.fill(0)
	for rectangle: Rect2 in blocked:
		var obstacle: Rect2 = rectangle.grow(2.0)
		var first: Vector2i = Vector2i(((obstacle.position - origin - size) / pitch).floor()).max(Vector2i.ZERO)
		var last: Vector2i = Vector2i(((obstacle.end - origin) / pitch).ceil()).min(counts - Vector2i.ONE)
		for y: int in range(first.y, last.y + 1):
			for x: int in range(first.x, last.x + 1):
				var cell: int = y * counts.x + x
				if occupied[cell] == 0 and Rect2(origin + Vector2(x, y) * pitch, size).intersects(obstacle):
					occupied[cell] = 1
	var rows: Array = []
	for y: int in counts.y:
		var columns: Array[int] = []
		for x: int in counts.x:
			if occupied[y * counts.x + x] == 0:
				columns.append(x)
		rows.append(columns)
	return {"origin": origin, "pitch": pitch, "size": size, "counts": counts, "rows": rows, "exhausted": 0}


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


## Label-only clearance: leaders are clipped separately, rather than making dense groups unplaceable.
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
		if row.is_empty():
			continue
		# Keep original Vector2 rounding and distance arithmetic; dx=0 is a lower bound.
		var row_position: Vector2 = (origin + Vector2(0, y) * pitch + (slot_size - size) * 0.5).floor()
		var row_center: Vector2 = Rect2(row_position, size).get_center()
		var center: Vector2 = figure.get_center()
		var lower_distance: float = Vector2(0, row_center.y).distance_squared_to(Vector2(0, center.y))
		if lower_distance >= best_distance:
			continue
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


static func _segment_interval(start: Vector2, finish: Vector2, rectangle: Rect2) -> Vector2:
	var direction: Vector2 = finish - start
	var first: float = 0.0
	var last: float = 1.0
	for axis: int in 2:
		if absf(direction[axis]) < 0.000001:
			if start[axis] < rectangle.position[axis] or start[axis] > rectangle.end[axis]:
				return Vector2(-1, -1)
			continue
		var a: float = (rectangle.position[axis] - start[axis]) / direction[axis]
		var b: float = (rectangle.end[axis] - start[axis]) / direction[axis]
		first = maxf(first, minf(a, b))
		last = minf(last, maxf(a, b))
		if first > last:
			return Vector2(-1, -1)
	return Vector2(first, last)


## Collect only segment-adjacent cells; side-cell visits conservatively cover float rounding at corners.
static func _segment_obstacles(start: Vector2, finish: Vector2, index: Dictionary) -> Array[Rect2]:
	var cell: Vector2i = Vector2i((start / 64.0).floor())
	var last: Vector2i = Vector2i((finish / 64.0).floor())
	var direction: Vector2 = finish - start
	var step: Vector2i = Vector2i(signf(direction.x), signf(direction.y))
	var cells: Dictionary = {cell: true, last: true}
	var limit: int = absi(last.x - cell.x) + absi(last.y - cell.y) + 1
	for unused: int in limit:
		cells[cell] = true
		if cell == last:
			break
		var crossing: Array[float] = [1.0e30, 1.0e30]
		for axis: int in 2:
			if step[axis] != 0:
				var boundary: float = float(cell[axis] + (1 if step[axis] > 0 else 0)) * 64.0
				crossing[axis] = (boundary - start[axis]) / direction[axis]
		if step.x != 0:
			cells[cell + Vector2i(step.x, 0)] = true
		if step.y != 0:
			cells[cell + Vector2i(0, step.y)] = true
		if absf(crossing[0] - crossing[1]) <= 0.000001:
			cell += step
		elif crossing[0] < crossing[1]:
			cell.x += step.x
		else:
			cell.y += step.y
	var unique: Dictionary = {}
	for key: Vector2i in cells:
		for rectangle: Rect2 in index.get(key, []):
			unique[rectangle] = true
	var result: Array[Rect2] = []
	for rectangle: Rect2 in unique:
		result.append(rectangle)
	return result


## Return visible endpoint pairs; expanded clipping prevents a stroke from tinting translucent bodies/UI.
static func clipped_leader(start: Vector2, finish: Vector2, index: Dictionary) -> PackedVector2Array:
	var result: PackedVector2Array = PackedVector2Array()
	if start.distance_squared_to(finish) < 0.000001:
		return result
	var intervals: Array[Vector2] = []
	for obstacle: Rect2 in _segment_obstacles(start, finish, index):
		var interval: Vector2 = _segment_interval(start, finish, obstacle.grow(2.5))
		if interval.x >= 0.0:
			intervals.append(interval)
	intervals.sort_custom(func(a: Vector2, b: Vector2) -> bool: return a.x < b.x)
	var cursor: float = 0.0
	for interval: Vector2 in intervals:
		if interval.x > cursor:
			result.append(start.lerp(finish, cursor))
			result.append(start.lerp(finish, interval.x))
		cursor = maxf(cursor, interval.y)
	if cursor < 1.0:
		result.append(start.lerp(finish, cursor))
		result.append(finish)
	return result


## Exact overlap membership: each person independently stops at its first intersecting neighbor.
static func overlapping_people(rectangles: Dictionary, cell_size: float) -> Dictionary:
	var buckets: Dictionary = {}
	for bits: int in rectangles:
		var rectangle: Rect2 = rectangles[bits]
		var first: Vector2i = Vector2i((rectangle.position / cell_size).floor())
		var last: Vector2i = Vector2i((rectangle.end / cell_size).floor())
		for y: int in range(first.y, last.y + 1):
			for x: int in range(first.x, last.x + 1):
				var cell: Vector2i = Vector2i(x, y)
				if not buckets.has(cell):
					buckets[cell] = []
				buckets[cell].append(bits)
	var result: Dictionary = {}
	for bits: int in rectangles:
		var rectangle: Rect2 = rectangles[bits]
		var first: Vector2i = Vector2i((rectangle.position / cell_size).floor())
		var last: Vector2i = Vector2i((rectangle.end / cell_size).floor())
		var found: bool = false
		for y: int in range(first.y, last.y + 1):
			for x: int in range(first.x, last.x + 1):
				for other: int in buckets.get(Vector2i(x, y), []):
					if other != bits and rectangle.intersects(rectangles[other]):
						result[bits] = true
						found = true
						break
				if found:
					break
			if found:
				break
	return result


## Collision-connected annotation sources: label breathing room4px, actual half-body contour1px.
## Only exact coincident groups enter this graph; unrelated single people remain placement obstacles.
static func aggregate_badges(sources: Array[Dictionary], agent_ids: Dictionary) -> Array[Dictionary]:
	var ordered: Array[Dictionary] = sources.duplicate()
	ordered.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a["key"]) < int(b["key"]))
	var shapes: Array = []
	var parents: Array[int] = []
	var buckets: Dictionary = {}
	for i: int in ordered.size():
		var source: Dictionary = ordered[i]
		var figure: Rect2 = source["figure"]
		var size: Vector2 = source["size"]
		var preferred: Rect2 = Rect2(Vector2(figure.get_center().x - size.x * 0.5, figure.position.y - 4.0 - size.y).round(), size)
		var rectangles: Array[Rect2] = [preferred.grow(4.0), figure.grow(1.0)]
		shapes.append(rectangles)
		parents.append(i)
		var cells: Dictionary = {}
		for rectangle: Rect2 in rectangles:
			var first: Vector2i = Vector2i((rectangle.position / 64.0).floor())
			var last: Vector2i = Vector2i((rectangle.end / 64.0).floor())
			for y: int in range(first.y, last.y + 1):
				for x: int in range(first.x, last.x + 1):
					cells[Vector2i(x, y)] = true
		var seen: Dictionary = {}
		for cell: Vector2i in cells:
			for other: int in buckets.get(cell, []):
				if seen.has(other):
					continue
				seen[other] = true
				var collision: bool = false
				for rectangle: Rect2 in rectangles:
					for previous: Rect2 in shapes[other]:
						collision = collision or rectangle.intersects(previous)
				if collision:
					var a: int = _annotation_root(parents, i)
					var b: int = _annotation_root(parents, other)
					parents[maxi(a, b)] = mini(a, b)
			if not buckets.has(cell):
				buckets[cell] = []
			buckets[cell].append(i)
	var components: Dictionary = {}
	for i: int in ordered.size():
		var key: int = _annotation_root(parents, i)
		if not components.has(key):
			components[key] = []
		components[key].append(i)
	var result: Array[Dictionary] = []
	for key: int in components:
		var descriptor: Dictionary = ordered[key].duplicate()
		var members: Array[int] = []
		var source_groups: Array = []
		var source_figures: Array[Rect2] = []
		var bounds: Rect2 = descriptor["figure"]
		for i: int in components[key]:
			var source: Dictionary = ordered[i]
			members.append_array(source["members"])
			source_groups.append((source["members"] as Array[int]).duplicate())
			source_figures.append(source["figure"])
			bounds = bounds.merge(source["figure"])
		members.sort_custom(func(a: int, b: int) -> bool: return int(agent_ids[a]) < int(agent_ids[b]))
		descriptor["members"] = members
		descriptor["source_groups"] = source_groups
		descriptor["source_figures"] = source_figures
		descriptor["figure"] = bounds
		descriptor["text"] = str(members.size())
		result.append(descriptor)
	result.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return int(a["key"]) < int(b["key"]))
	return result


static func _annotation_root(parents: Array[int], index: int) -> int:
	while parents[index] != index:
		parents[index] = parents[parents[index]]
		index = parents[index]
	return index


## Exact unfilled union boundary, including holes/disconnected components. No hull or pixel mask.
static func aggregate_contours(figures: Array, padding: float, counters: Dictionary = {}) -> PackedVector2Array:
	var rectangles: Array[Rect2] = []
	var ordinals: Dictionary = {}
	for figure: Rect2 in figures:
		var rectangle: Rect2 = figure.grow(padding)
		if rectangle.has_area() and not ordinals.has(rectangle):
			ordinals[rectangle] = rectangles.size()
			rectangles.append(rectangle)
	var index: Dictionary = badge_index(rectangles)
	var lines: Dictionary = {}
	for ordinal: int in rectangles.size():
		var rectangle: Rect2 = rectangles[ordinal]
		for axis: int in 2:
			var normal: int = 1 - axis
			for side: int in 2:
				var coordinate: float = rectangle.position[normal] if side == 0 else rectangle.end[normal]
				var start: Vector2 = rectangle.position
				var finish: Vector2 = rectangle.end
				start[normal] = coordinate
				finish[normal] = coordinate
				var covered: Array[Vector2] = []
				for other: Rect2 in _segment_obstacles(start, finish, index):
					if other == rectangle:
						continue
					if not counters.is_empty():
						counters["union_candidates"] = int(counters.get("union_candidates", 0)) + 1
					var outside: bool = (other.position[normal] < coordinate and other.end[normal] >= coordinate) if side == 0 else (other.position[normal] <= coordinate and other.end[normal] > coordinate)
					var same_edge: bool = (other.position[normal] == coordinate if side == 0 else other.end[normal] == coordinate) and int(ordinals[other]) < ordinal
					if outside or same_edge:
						var low: float = maxf(start[axis], other.position[axis])
						var high: float = minf(finish[axis], other.end[axis])
						if high > low:
							covered.append(Vector2(low, high))
				covered.sort_custom(func(a: Vector2, b: Vector2) -> bool: return a.x < b.x)
				var cursor: float = start[axis]
				var key: Vector2 = Vector2(axis, coordinate)
				if not lines.has(key):
					lines[key] = []
				for interval: Vector2 in covered:
					if interval.x > cursor:
						lines[key].append(Vector2(cursor, interval.x))
					cursor = maxf(cursor, interval.y)
				if cursor < finish[axis]:
					lines[key].append(Vector2(cursor, finish[axis]))
	var keys: Array = lines.keys()
	keys.sort_custom(func(a: Vector2, b: Vector2) -> bool: return a.x < b.x or (a.x == b.x and a.y < b.y))
	var result: PackedVector2Array = PackedVector2Array()
	for key: Vector2 in keys:
		var intervals: Array = lines[key]
		intervals.sort_custom(func(a: Vector2, b: Vector2) -> bool: return a.x < b.x or (a.x == b.x and a.y < b.y))
		var merged: Array[Vector2] = []
		for interval: Vector2 in intervals:
			if merged.is_empty() or interval.x > merged[-1].y:
				merged.append(interval)
			else:
				merged[-1] = Vector2(merged[-1].x, maxf(merged[-1].y, interval.y))
		for interval: Vector2 in merged:
			result.append(Vector2(interval.x, key.y) if key.x == 0.0 else Vector2(key.y, interval.x))
			result.append(Vector2(interval.y, key.y) if key.x == 0.0 else Vector2(key.y, interval.y))
	if not counters.is_empty():
		counters["source_rectangles"] = int(counters.get("source_rectangles", 0)) + figures.size()
		counters["input_edges"] = int(counters.get("input_edges", 0)) + rectangles.size() * 4
		counters["union_segments"] = int(counters.get("union_segments", 0)) + (result.size() >> 1)
	return result


## Attach to an actual exterior edge; stable ties use the canonical union-edge order.
static func aggregate_leader(contours: PackedVector2Array, label: Rect2) -> PackedVector2Array:
	var chosen: Vector2 = Vector2.ZERO
	var best: float = INF
	var label_end: Vector2 = Vector2.ZERO
	var center: Vector2 = label.get_center()
	for i: int in range(0, contours.size(), 2):
		var start: Vector2 = contours[i]
		var delta: Vector2 = contours[i + 1] - start
		var fraction: float = clampf((center - start).dot(delta) / delta.length_squared(), 0.0, 1.0)
		var point: Vector2 = start + delta * fraction
		var target: Vector2 = point.clamp(label.position, label.end)
		var distance: float = point.distance_squared_to(target)
		if distance < best:
			best = distance
			chosen = point
			label_end = target
	return PackedVector2Array([chosen, label_end]) if best < INF else PackedVector2Array()


## Shared indexed clipping input. Equal rectangles retain ALL owners so a nonmember is never hidden.
static func aggregate_body_obstacles(rectangles: Dictionary, canvas: Transform2D, clearance: float) -> Dictionary:
	var bodies: Array[Rect2] = []
	var owners: Dictionary = {}
	for bits: int in rectangles:
		var rectangle: Rect2 = (canvas * (rectangles[bits] as Rect2)).grow(clearance)
		if not owners.has(rectangle):
			owners[rectangle] = []
			bodies.append(rectangle)
		owners[rectangle].append(bits)
	return {"index": badge_index(bodies), "owners": owners}


## Clip every exterior fragment to the viewport and clear UI, labels and unrelated silhouettes.
static func clipped_aggregate(contours: PackedVector2Array, viewport: Rect2, common: Dictionary, bodies: Dictionary, members: Array, counters: Dictionary = {}) -> PackedVector2Array:
	var member_set: Dictionary = {}
	for bits: int in members:
		member_set[bits] = true
	var nonmember_by_rectangle: Dictionary = {}
	var result: PackedVector2Array = PackedVector2Array()
	for i: int in range(0, contours.size(), 2):
		var start: Vector2 = contours[i]
		var finish: Vector2 = contours[i + 1]
		var visible: Vector2 = _segment_interval(start, finish, viewport)
		if visible.x < 0.0 or visible.y <= visible.x:
			continue
		var clipped_start: Vector2 = start.lerp(finish, visible.x)
		finish = start.lerp(finish, visible.y)
		start = clipped_start
		var obstacles: Array[Rect2] = _segment_obstacles(start, finish, common)
		for rectangle: Rect2 in _segment_obstacles(start, finish, bodies["index"]):
			if not nonmember_by_rectangle.has(rectangle):
				var nonmember: bool = false
				for bits: int in bodies["owners"][rectangle]:
					if not counters.is_empty():
						counters["owner_checks"] = int(counters.get("owner_checks", 0)) + 1
					if not member_set.has(bits):
						nonmember = true
						break
				nonmember_by_rectangle[rectangle] = nonmember
			elif not counters.is_empty():
				counters["owner_cache_hits"] = int(counters.get("owner_cache_hits", 0)) + 1
			if bool(nonmember_by_rectangle[rectangle]):
				obstacles.append(rectangle)
		var intervals: Array[Vector2] = []
		for rectangle: Rect2 in obstacles:
			if not counters.is_empty():
				counters["clip_candidates"] = int(counters.get("clip_candidates", 0)) + 1
			var interval: Vector2 = _segment_interval(start, finish, rectangle)
			if interval.x >= 0.0:
				intervals.append(interval)
		intervals.sort_custom(func(a: Vector2, b: Vector2) -> bool: return a.x < b.x)
		var cursor: float = 0.0
		for interval: Vector2 in intervals:
			if interval.x > cursor:
				result.append(start.lerp(finish, cursor))
				result.append(start.lerp(finish, interval.x))
			cursor = maxf(cursor, interval.y)
		if cursor < 1.0:
			result.append(start.lerp(finish, cursor))
			result.append(finish)
	if not counters.is_empty():
		counters["emitted_segments"] = int(counters.get("emitted_segments", 0)) + (result.size() >> 1)
	return result
