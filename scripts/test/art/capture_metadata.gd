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


static func life_stage_counts(snapshot: Dictionary) -> Dictionary:
	var counts: Dictionary = {"adult": 0, "child": 0, "elder": 0, "unknown": 0}
	var stages: Variant = snapshot.get("life_stages", PackedByteArray())
	for stage: int in stages:
		var key: String = ["adult", "child", "elder"][stage] if stage >= 0 and stage <= 2 else "unknown"
		counts[key] += 1
	return counts


static func deadwood_tile(snapshot: Dictionary, village: Vector2) -> Vector2:
	var kinds: PackedInt32Array = snapshot.get("kinds", PackedInt32Array())
	var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
	var best: Vector2 = Vector2(-1, -1)
	var distance: float = INF
	for index: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[index] != 3:
			continue
		var tile: Vector2 = Vector2(xs[index], ys[index])
		var candidate: float = maxf(absf(tile.x - village.x), absf(tile.y - village.y))
		if candidate < distance or (candidate == distance and (tile.y < best.y or (tile.y == best.y and tile.x < best.x))):
			best = tile
			distance = candidate
	return best


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
			"shown": item.is_visible_in_tree(),
			"node_path": str(main.get_path_to(node)),
			"modulate": [item.modulate.r, item.modulate.g, item.modulate.b, item.modulate.a],
			"self_modulate": [item.self_modulate.r, item.self_modulate.g, item.self_modulate.b, item.self_modulate.a],
		}
		for key: String in ["entity_bits", "building_kind", "sprite_id", "frame_id", "body", "variant", "tile_x", "tile_y", "activity", "pose", "facing", "calibration_kind", "missing_lineup_frame", "fallback_kind", "qa_status", "fire_lit", "glow_alpha", "wood_stock", "food_stock", "settlement_id", "woodpile_stage", "deadwood_stage", "resource_amount", "resource_max", "seated"]:
			if node.has_meta(key):
				row[key] = node.get_meta(key)
		for key: String in ["cold", "shiver_offset", "puff_alpha", "wood_target", "cold_pose", "cold_base_pose", "moving", "cold_sleepers", "frost_alpha", "doorway_puff_alpha", "cold_ash", "chill_tint_strength", "night_chill_strength", "cold_snap_tint", "chill_alpha", "chill_since_tick", "chill_shiver_offset", "chill_puff_alpha", "load_visible", "load_kind", "load_frame_id", "load_flip_h", "load_z_index", "rain_streak_count", "rain_ticks_left", "windfall_alpha", "windfall_offset", "stock_flash_alpha"]:
			if node.has_meta(key):
				row[key] = node.get_meta(key)
		for key: String in ["drawn_position", "tile", "declump_offset", "drawn_target", "spread_offset", "seat", "load_anchor"]:
			if node.has_meta(key):
				var point: Vector2 = Vector2(node.get_meta(key))
				row[key] = [point.x, point.y]
		if node is Label:
			row["text"] = (node as Label).text
		if node is Sprite2D:
			var sprite: Sprite2D = node as Sprite2D
			row["modulate_alpha"] = sprite.modulate.a
			row["self_modulate_alpha"] = sprite.self_modulate.a
			row["world_anchor"] = [sprite.global_position.x, sprite.global_position.y]
			row["z_index"] = sprite.z_index + (sprite.get_parent().z_index if sprite.get_parent() is CanvasItem else 0)
			row["sibling_index"] = sprite.get_index()
			var cue_rect: Rect2 = rect
			for child: Node in sprite.get_children():
				if child is CanvasItem and (child as CanvasItem).is_visible_in_tree():
					cue_rect = cue_rect.merge(bounds(child, scale_factor))
			row["cue_rect"] = rect_array(cue_rect)
			row["frame_size"] = [sprite.get_rect().size.x, sprite.get_rect().size.y]
			row["flip_h"] = sprite.flip_h
			row["texture_path"] = sprite.texture.resource_path if sprite.texture != null else ""
			row["region"] = rect_array(sprite.region_rect)
		rows.append(row)
	var ui: Node = main.get_node_or_null("UI")
	if ui != null:
		for node: Node in ui.find_children("*", "PanelContainer", true, false):
			var panel: Control = node as Control
			if panel.is_visible_in_tree():
				rows.append({"kind": "panel", "id": node.get_instance_id(), "name": str(node.name),
					"rect": rect_array(screen_rect(Rect2(Vector2.ZERO, panel.size), panel.get_global_transform_with_canvas(), scale_factor)), "visible": true})
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
	var renderer: Node = main.get_node_or_null("AgentRenderer")
	if renderer != null and renderer.has_method("set_capture_visible"):
		renderer.call("set_capture_visible", shown)
		return
	for node: Node in tagged_nodes(main.get_node_or_null("WorldObjects")):
		if str(node.get_meta("art_kind", "")) == "agent" and node is CanvasItem:
			(node as CanvasItem).visible = shown
	var shadows: Node = main.get_node_or_null("Shadows")
	if shadows != null:
		for node: Node in shadows.get_children():
			if bool(node.get_meta("agent_shadow", false)) and node is CanvasItem:
				(node as CanvasItem).visible = shown
