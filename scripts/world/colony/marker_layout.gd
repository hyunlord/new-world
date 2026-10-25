# Screen-only marker placement: retain true anchors, separate visible discs/glyphs and avoid visible UI surfaces.
# Bridge methods: none. Stable input ID order determines priority; off-screen anchors remain off screen.
extends RefCounted


## Place a group label clear of every figure/UI rectangle, retaining its preferred position above the group when free.
static func badge(figure: Rect2, size: Vector2, viewport: Rect2, blocked: Array[Rect2]) -> Rect2:
	var gap: float = 4.0
	var preferred: Vector2 = Vector2(figure.get_center().x - size.x * 0.5, figure.position.y - gap - size.y).round()
	var nearby: Array[Vector2] = [preferred,
		Vector2(figure.position.x - gap - size.x, figure.get_center().y - size.y * 0.5),
		Vector2(figure.end.x + gap, figure.get_center().y - size.y * 0.5),
		Vector2(figure.get_center().x - size.x * 0.5, figure.end.y + gap)]
	for position: Vector2 in nearby:
		var candidate: Rect2 = Rect2(position.round(), size)
		if _badge_free(candidate, figure, viewport, blocked):
			return candidate
	var step: float = maxf(size.x, size.y) + gap
	var limit: int = ceili(maxf(viewport.size.x, viewport.size.y) / step)
	for ring: int in range(1, limit + 1):
		for y: int in range(-ring, ring + 1):
			for x: int in range(-ring, ring + 1):
				if maxi(absi(x), absi(y)) != ring:
					continue
				var candidate: Rect2 = Rect2((preferred + Vector2(x, y) * step).round(), size)
				if _badge_free(candidate, figure, viewport, blocked):
					return candidate
	# A completely occupied viewport has no truthful free placement; preserve the cue for visual failure reporting.
	return Rect2(preferred, size)


static func badge_leader(figure: Rect2, label: Rect2) -> PackedVector2Array:
	return PackedVector2Array([_edge_toward(figure.grow(3.0), label.get_center()), _edge_toward(label, figure.get_center())])


static func _edge_toward(rectangle: Rect2, target: Vector2) -> Vector2:
	var direction: Vector2 = target - rectangle.get_center()
	var distance: float = minf(rectangle.size.x * 0.5 / maxf(absf(direction.x), 0.001), rectangle.size.y * 0.5 / maxf(absf(direction.y), 0.001))
	return rectangle.get_center() + direction * distance


static func _badge_free(label: Rect2, figure: Rect2, viewport: Rect2, blocked: Array[Rect2]) -> bool:
	if not viewport.encloses(label.grow(1.0)):
		return false
	var leader: PackedVector2Array = badge_leader(figure, label)
	for obstacle: Rect2 in blocked:
		if label.intersects(obstacle.grow(2.0)) or segment_hits_rect(leader[0], leader[1], obstacle.grow(2.5)):
			return false
	return true


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
	var occupied: Dictionary = {}
	var radius: float = extent * 0.5 + gap
	var glyph: float = 14.0 if glyphs else 0.0
	var size: Vector2 = Vector2(radius * 2.0 + glyph, radius * 2.0 + glyph)
	var cell_size: float = maxf(size.x, size.y)
	for index: int in order:
		var anchor: Vector2 = points[index].round()
		if not Rect2(anchor - Vector2.ONE * extent * 0.5, Vector2.ONE * extent).intersects(viewport):
			continue
		var chosen: Vector2 = anchor
		var found: bool = false
		var limit: int = ceili(maxf(viewport.size.x, viewport.size.y) / cell_size)
		for ring: int in range(limit + 1):
			for y: int in range(-ring, ring + 1):
				for x: int in range(-ring, ring + 1):
					if maxi(absi(x), absi(y)) != ring:
						continue
					var candidate: Vector2 = anchor + Vector2(x, y) * cell_size
					var rectangle: Rect2 = Rect2(candidate - Vector2(radius, radius + glyph), size)
					if viewport.encloses(rectangle) and _free(rectangle, blocked, occupied, cell_size):
						chosen = candidate
						_reserve(rectangle, occupied, cell_size)
						found = true
						break
				if found:
					break
			if found:
				break
		# Exhaustion keeps the original visible marker; the visibility oracle still records any overlap.
		result[index] = chosen
	return result


static func _free(rectangle: Rect2, blocked: Array[Rect2], occupied: Dictionary, cell_size: float) -> bool:
	for obstacle: Rect2 in blocked:
		if rectangle.intersects(obstacle):
			return false
	var first: Vector2i = Vector2i((rectangle.position / cell_size).floor())
	var last: Vector2i = Vector2i((rectangle.end / cell_size).floor())
	for y: int in range(first.y, last.y + 1):
		for x: int in range(first.x, last.x + 1):
			for other: Rect2 in occupied.get(Vector2i(x, y), []):
				if rectangle.intersects(other):
					return false
	return true


static func _reserve(rectangle: Rect2, occupied: Dictionary, cell_size: float) -> void:
	var first: Vector2i = Vector2i((rectangle.position / cell_size).floor())
	var last: Vector2i = Vector2i((rectangle.end / cell_size).floor())
	for y: int in range(first.y, last.y + 1):
		for x: int in range(first.x, last.x + 1):
			var cell: Vector2i = Vector2i(x, y)
			if not occupied.has(cell):
				occupied[cell] = []
			occupied[cell].append(rectangle)
