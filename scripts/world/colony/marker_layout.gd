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
## PERF2-1 E: forbidden rectangles that miss the feasible box are skipped and rows that cannot beat the clamped-zero
## row are not scanned (same result as the full search). Above FRAME_PAIR_BUDGET body×obstacle pairs (a crowd, e.g.
## 1,000 people at the night camp, where the full search took 14.8 s; 512 pairs bound the exact search to about
## 0.5 M row tests) the crowd's bounding box is framed instead: O(n + m) and conservative (every body stays inside
## the box, so a found translation still clears every body).
const FRAME_PAIR_BUDGET: int = 512

static func frame_translation(bodies: Array[Rect2], obstacles: Array[Rect2], viewport: Rect2, margin: float = 4.0) -> Dictionary:
	var failure: Dictionary = {"found": false, "translation": Vector2.ZERO, "body_count": bodies.size()}
	if bodies.is_empty() or not viewport.has_area():
		return failure
	var low: Vector2 = Vector2(-INF, -INF)
	var high: Vector2 = Vector2(INF, INF)
	var bounds: Rect2 = bodies[0]
	for body: Rect2 in bodies:
		if not body.has_area() or not viewport.encloses(body):
			return failure
		low = low.max(viewport.position - body.position)
		high = high.min(viewport.end - body.end)
		bounds = bounds.merge(body)
	if low.x > high.x or low.y > high.y:
		return failure
	var grown: Array[Rect2] = []
	for obstacle: Rect2 in obstacles:
		if obstacle.has_area():
			grown.append(obstacle.grow(maxf(0.0, margin)))
	var framed: Array[Rect2] = bodies
	if bodies.size() * grown.size() > FRAME_PAIR_BUDGET:
		framed = []
		framed.append(bounds)
		low = viewport.position - bounds.position
		high = viewport.end - bounds.end
	var forbidden: Array[Rect2] = []
	for body: Rect2 in framed:
		for expanded: Rect2 in grown:
			var rectangle: Rect2 = Rect2(expanded.position - body.end, expanded.size + body.size)
			if rectangle.position.x >= high.x or rectangle.end.x <= low.x or rectangle.position.y >= high.y or rectangle.end.y <= low.y:
				continue
			forbidden.append(rectangle)
	var zero_row: float = clampf(0.0, low.y, high.y)
	var rows: PackedFloat64Array = PackedFloat64Array([zero_row, low.y, high.y])
	for rectangle: Rect2 in forbidden:
		if rectangle.position.y >= low.y:
			rows.append(rectangle.position.y)
		if rectangle.end.y <= high.y:
			rows.append(rectangle.end.y)
	rows.sort()
	# The best row is at least as good as the clamped-zero row: a row whose y² exceeds that cost cannot win.
	var bound: Array = _best_in_row(zero_row, forbidden, low, high, [INF, Vector2.ZERO])
	var limit: float = float(bound[0])
	var best: Array = [INF, Vector2.ZERO]
	var previous: float = NAN
	for y: float in rows:
		if y == previous:
			continue
		previous = y
		if y * y > limit:
			continue
		best = _best_in_row(y, forbidden, low, high, best)
	if float(best[0]) == INF:
		return failure
	return {"found": true, "translation": best[1], "body_count": bodies.size()}


## [cost, translation]: `best` improved (strictly) by the free point nearest x = 0 on row `y`.
static func _best_in_row(y: float, forbidden: Array[Rect2], low: Vector2, high: Vector2, best: Array) -> Array:
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
	var result: Array = best
	for gap: Vector2 in gaps:
		var candidate: Vector2 = Vector2(clampf(0.0, gap.x, gap.y), y)
		var cost: float = candidate.length_squared()
		if cost < float(result[0]):
			result = [cost, candidate]
	return result


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
