extends RefCounted
## Screen-space geometry shared by founding destinations and selected hearths.
const SIZE_PX := 9.0
const DARK := Color("352b23")
const FONT = preload("res://assets/fonts/galmuri/Galmuri9.ttf")

static func geometry(target: Vector2, viewport: Rect2, playable: Rect2) -> Dictionary:
	var center: Vector2 = viewport.get_center()
	var direction: Vector2 = center.direction_to(target)
	var on_screen: bool = viewport.has_point(target)
	var edge: Vector2 = center
	var bounds: Rect2 = playable.grow(-6.0)
	if not on_screen:
		var reach: float = INF
		if not is_zero_approx(direction.x):
			reach = minf(reach, ((bounds.end.x if direction.x > 0.0 else bounds.position.x) - center.x) / direction.x)
		if not is_zero_approx(direction.y):
			reach = minf(reach, ((bounds.end.y if direction.y > 0.0 else bounds.position.y) - center.y) / direction.y)
		edge = center + direction * maxf(0.0, reach)
	return {"site_on_screen": on_screen, "stake_visible": on_screen, "edge_visible": not on_screen, "edge_screen_position": [edge.x, edge.y], "target_screen_position": [target.x, target.y], "screen_center": [center.x, center.y], "viewport_size": [viewport.size.x, viewport.size.y], "edge_direction": [direction.x, direction.y], "edge_hit_rect": [edge.x - 10.0, edge.y - 10.0, 20.0, 20.0], "edge_size_px": SIZE_PX}

static func playable_rect(node: CanvasItem) -> Rect2:
	var viewport: Rect2 = node.get_viewport_rect()
	var result: Rect2 = viewport.grow(-8.0)
	var ui: Node = node.get_node_or_null("/root/Main/UI")
	if ui == null:
		return result
	for panel: Node in ui.get_children():
		if not panel is Control or not panel.is_visible_in_tree():
			continue
		var controls: Array[Node] = [panel]
		if not panel is PanelContainer:
			controls = panel.get_children()
		for item: Node in controls:
			if not item is Control or not item.is_visible_in_tree():
				continue
			var rect: Rect2 = item.get_global_rect()
			if rect.size.x <= 0.0 or rect.size.y <= 0.0:
				continue
			if rect.position.y < viewport.size.y * 0.25 and rect.end.y < viewport.get_center().y:
				var bottom: float = result.end.y
				result.position.y = maxf(result.position.y, rect.end.y + 8.0)
				result.size.y = bottom - result.position.y
			elif rect.end.y > viewport.size.y * 0.75 and rect.position.y > viewport.get_center().y:
				result.size.y = minf(result.end.y, rect.position.y - 8.0) - result.position.y
	return result

static func draw_marker(node: Node2D, row: Dictionary, color: Color, text: String = "") -> void:
	if not bool(row.get("edge_visible", false)):
		return
	var p: Array = row["edge_screen_position"]
	var d: Array = row["edge_direction"]
	var center := Vector2(p[0], p[1])
	var direction := Vector2(d[0], d[1])
	var side := direction.orthogonal()
	var points := PackedVector2Array([center + _square_tip(direction), center + _square_tip(-direction + side), center - direction, center + _square_tip(-direction - side)])
	node.draw_set_transform_matrix(node.get_global_transform_with_canvas().affine_inverse())
	node.draw_colored_polygon(points, color)
	points.append(points[0])
	node.draw_polyline(points, DARK, 1.0, false)
	if not text.is_empty():
		var width: float = FONT.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, 12).x
		var size: Array = row["viewport_size"]
		var anchor := Vector2(clampf(center.x + 10.0 if direction.x <= 0.0 else center.x - width - 10.0, 8.0, float(size[0]) - width - 8.0), center.y + 4.0)
		node.draw_string_outline(FONT, anchor, text, HORIZONTAL_ALIGNMENT_LEFT, -1, 12, 2, DARK)
		node.draw_string(FONT, anchor, text, HORIZONTAL_ALIGNMENT_LEFT, -1, 12, Color("EEEAD2"))
	node.draw_set_transform_matrix(Transform2D.IDENTITY)

static func _square_tip(direction: Vector2) -> Vector2:
	return direction * (4.0 / maxf(absf(direction.x), absf(direction.y)))

static func hit(row: Dictionary, position: Vector2) -> bool:
	if not bool(row.get("edge_visible", false)):
		return false
	var rect: Array = row["edge_hit_rect"]
	return Rect2(rect[0], rect[1], rect[2], rect[3]).has_point(position)

static func pan(camera: Camera2D, target: Vector2) -> void:
	if camera == null:
		return
	if camera.has_method("set_auto_track"):
		camera.call("set_auto_track", false)
	camera.position = target
	camera.force_update_scroll()
