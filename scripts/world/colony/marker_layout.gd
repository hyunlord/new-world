# marker_layout.gd — screen-space layout helpers for the camera and the panels that pin to the map (M-UX2; trimmed to
#   what survives the 2D view in M3D-SWITCH E2).
# Purpose: `ui_rects` lists the visible HUD surfaces (panels and buttons) a map point or a framed body must avoid;
#   `frame_translation` finds the one common screen translation that puts every on-screen person body clear of them
#   (the camera's camp framing), and `frame_translation_clear` validates the translation the camera achieved.
# Bridge methods: none.
extends RefCounted


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


## One common screen translation for every original on-screen body; no filtering or zoom.
## Forbidden translation rectangles reuse the popup edge-row/merged-interval approach.
static func frame_translation(bodies: Array[Rect2], obstacles: Array[Rect2], viewport: Rect2, margin: float = 4.0) -> Dictionary:
	var failure: Dictionary = {"found": false, "translation": Vector2.ZERO, "body_count": bodies.size()}
	if bodies.is_empty() or not viewport.has_area():
		return failure
	var low: Vector2 = Vector2(-INF, -INF)
	var high: Vector2 = Vector2(INF, INF)
	var forbidden: Array[Rect2] = []
	for body: Rect2 in bodies:
		if not body.has_area() or not viewport.encloses(body):
			return failure
		low = low.max(viewport.position - body.position)
		high = high.min(viewport.end - body.end)
		for obstacle: Rect2 in obstacles:
			if not obstacle.has_area():
				continue
			var expanded: Rect2 = obstacle.grow(maxf(0.0, margin))
			forbidden.append(Rect2(expanded.position - body.end, expanded.size + body.size))
	if low.x > high.x or low.y > high.y:
		return failure
	var rows: Array[float] = [clampf(0.0, low.y, high.y), low.y, high.y]
	for rectangle: Rect2 in forbidden:
		for y: float in [rectangle.position.y, rectangle.end.y]:
			if y >= low.y and y <= high.y and not rows.has(y):
				rows.append(y)
	rows.sort()
	var best: Vector2 = Vector2.ZERO
	var distance: float = INF
	for y: float in rows:
		var intervals: Array[Vector2] = []
		for rectangle: Rect2 in forbidden:
			if y > rectangle.position.y and y < rectangle.end.y:
				intervals.append(Vector2(rectangle.position.x, rectangle.end.x))
		intervals.sort_custom(func(a: Vector2, b: Vector2) -> bool: return a.x < b.x)
		var gaps: Array[Vector2] = []
		var cursor: float = low.x
		for interval: Vector2 in intervals:
			if interval.y <= cursor or interval.x >= high.x:
				continue
			if interval.x >= cursor:
				gaps.append(Vector2(cursor, minf(interval.x, high.x)))
			cursor = maxf(cursor, interval.y)
		if cursor <= high.x:
			gaps.append(Vector2(cursor, high.x))
		for gap: Vector2 in gaps:
			var candidate: Vector2 = Vector2(clampf(0.0, gap.x, gap.y), y)
			var cost: float = candidate.length_squared()
			if cost < distance:
				distance = cost
				best = candidate
	if distance == INF:
		return failure
	return {"found": true, "translation": best, "body_count": bodies.size()}


## Validate the achieved (pixel-rounded/clamped) camera translation without dropping bodies.
static func frame_translation_clear(bodies: Array[Rect2], obstacles: Array[Rect2], viewport: Rect2, shift: Vector2, margin: float = 2.0) -> bool:
	if bodies.is_empty():
		return false
	for body: Rect2 in bodies:
		var moved: Rect2 = Rect2(body.position + shift, body.size)
		if not viewport.encloses(moved):
			return false
		for obstacle: Rect2 in obstacles:
			if obstacle.has_area() and moved.intersects(obstacle.grow(maxf(0.0, margin))):
				return false
	return true
