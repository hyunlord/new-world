extends RefCounted


static func screen_rect(local: Rect2, transform: Transform2D, scale_factor: Vector2) -> Rect2:
	var rect: Rect2 = transform * local
	return Rect2(rect.position * scale_factor, rect.size * scale_factor)


static func bounds(node: Node, scale_factor: Vector2) -> Rect2:
	if node is Sprite2D:
		var sprite: Sprite2D = node as Sprite2D
		return screen_rect(sprite.get_rect(), sprite.get_global_transform_with_canvas(), scale_factor)
	var result: Rect2 = Rect2()
	if node is Control:
		var control: Control = node as Control
		result = screen_rect(Rect2(Vector2.ZERO, control.size), control.get_global_transform_with_canvas(), scale_factor)
	for child: Node in node.get_children():
		if child is CanvasItem and not (child as CanvasItem).is_visible_in_tree():
			continue
		var rect: Rect2 = bounds(child, scale_factor)
		if rect.has_area():
			result = result.merge(rect) if result.has_area() else rect
	return result


static func rect_array(rect: Rect2) -> Array[float]:
	return [rect.position.x, rect.position.y, rect.size.x, rect.size.y]


static func tagged_nodes(parent: Node) -> Array[Node]:
	var result: Array[Node] = []
	if parent == null:
		return result
	for child: Node in parent.get_children():
		if child.has_meta("art_kind") or child.has_meta("calibration_kind"):
			result.append(child)
		result.append_array(tagged_nodes(child))
	return result


static func entities(main: Node, viewport: Vector2, scale_factor: Vector2) -> Array[Dictionary]:
	var rows: Array[Dictionary] = []
	for node: Node in tagged_nodes(main):
		if not node is CanvasItem:
			continue
		var item: CanvasItem = node as CanvasItem
		var rect: Rect2 = bounds(node, scale_factor)
		var row: Dictionary = {
			"kind": str(node.get_meta("art_kind", node.get_meta("calibration_kind", "object"))),
			"id": int(node.get_meta("art_id", 0)), "rect": rect_array(rect),
			"visible": item.is_visible_in_tree() and rect.intersects(Rect2(Vector2.ZERO, viewport)),
			"node_path": str(main.get_path_to(node)),
		}
		for key: String in ["entity_bits", "building_kind", "sprite_id", "frame_id", "body", "variant", "tile_x", "tile_y", "activity", "calibration_kind"]:
			if node.has_meta(key):
				row[key] = node.get_meta(key)
		if node is Sprite2D:
			var sprite: Sprite2D = node as Sprite2D
			row["frame_size"] = [sprite.get_rect().size.x, sprite.get_rect().size.y]
			row["flip_h"] = sprite.flip_h
			row["texture_path"] = sprite.texture.resource_path if sprite.texture != null else ""
			row["region"] = rect_array(sprite.region_rect)
		rows.append(row)
	var ui: Node = main.get_node_or_null("UI")
	if ui != null:
		for node: Node in ui.get_children():
			if node is Control and (node as Control).is_visible_in_tree():
				rows.append({"kind": "panel", "id": node.get_instance_id(), "name": str(node.name),
					"rect": rect_array(bounds(node, scale_factor)), "visible": true})
	return rows


static func labels(parent: Node, scale_factor: Vector2) -> Array[Dictionary]:
	var rows: Array[Dictionary] = []
	if parent == null:
		return rows
	for child: Node in parent.get_children():
		if child is Label and (child as Label).is_visible_in_tree():
			var label: Label = child as Label
			var font: Font = label.get_theme_font("font")
			rows.append({"name": str(child.get_path()), "text": label.text,
				"rect": rect_array(bounds(label, scale_factor)),
				"font": font.get_font_name() if font != null else "",
				"font_size": label.get_theme_font_size("font_size")})
		rows.append_array(labels(child, scale_factor))
	return rows


static func show_agents(main: Node, shown: bool) -> void:
	for node: Node in tagged_nodes(main.get_node_or_null("WorldObjects")):
		if str(node.get_meta("art_kind", "")) == "agent" and node is CanvasItem:
			(node as CanvasItem).visible = shown
	var shadows: Node = main.get_node_or_null("Shadows")
	if shadows != null:
		for node: Node in shadows.get_children():
			if bool(node.get_meta("agent_shadow", false)) and node is CanvasItem:
				(node as CanvasItem).visible = shown
