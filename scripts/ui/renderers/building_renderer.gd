extends Node2D

var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _sim_engine: RefCounted
var _current_lod: int = 1
var _runtime_minimap_cache: Dictionary = {}
var _runtime_minimap_cache_tick: int = -1
var _runtime_world_summary_cache: Dictionary = {}
var _runtime_world_summary_cache_tick: int = -1
var _last_redraw_cam_pos: Vector2 = Vector2.INF
var _last_redraw_zoom: float = -1.0
var _last_redraw_tick: int = -1
var _redraw_cooldown: float = 0.0


func init(building_manager: RefCounted, settlement_manager: RefCounted = null, sim_engine: RefCounted = null) -> void:
	_building_manager = building_manager
	_settlement_manager = settlement_manager
	_sim_engine = sim_engine


func _process(delta: float) -> void:
	_redraw_cooldown = maxf(0.0, _redraw_cooldown - delta)
	var cam: Camera2D = get_viewport().get_camera_2d()
	if cam == null:
		return
	var cam_pos: Vector2 = cam.global_position
	var cam_zoom: float = cam.zoom.x
	var runtime_tick: int = -1
	if _sim_engine != null:
		runtime_tick = int(_sim_engine.get("current_tick"))
	var camera_changed: bool = (
		_last_redraw_cam_pos == Vector2.INF
		or cam_pos.distance_to(_last_redraw_cam_pos) > 8.0
		or absf(cam_zoom - _last_redraw_zoom) > 0.01
	)
	var data_changed: bool = runtime_tick != _last_redraw_tick
	if _redraw_cooldown > 0.0 and not data_changed:
		return
	if camera_changed or data_changed:
		_last_redraw_cam_pos = cam_pos
		_last_redraw_zoom = cam_zoom
		_last_redraw_tick = runtime_tick
		_redraw_cooldown = 0.1
		queue_redraw()


func force_redraw() -> void:
	_redraw_cooldown = 0.0
	_last_redraw_cam_pos = Vector2.INF
	queue_redraw()


func _draw() -> void:
	if _building_manager == null and _sim_engine == null:
		return
	var cam := get_viewport().get_camera_2d()
	var zl: float = cam.zoom.x if cam else 1.0
	_update_lod(zl)

	# Viewport culling
	var viewport_size := get_viewport_rect().size
	var cam_pos := cam.global_position if cam else Vector2.ZERO
	var half_view := viewport_size / cam.zoom * 0.5 if cam else viewport_size * 0.5
	var min_tile_x: int = int((cam_pos.x - half_view.x) / GameConfig.TILE_SIZE) - 2
	var max_tile_x: int = int((cam_pos.x + half_view.x) / GameConfig.TILE_SIZE) + 2
	var min_tile_y: int = int((cam_pos.y - half_view.y) / GameConfig.TILE_SIZE) - 2
	var max_tile_y: int = int((cam_pos.y + half_view.y) / GameConfig.TILE_SIZE) + 2

	var buildings: Array = []
	if _building_manager != null:
		buildings = _building_manager.get_all_buildings()
	if buildings.is_empty():
		buildings = _get_runtime_buildings()
	var tile_size: int = GameConfig.TILE_SIZE
	var half: float = tile_size * 0.5
	var font: Font = ThemeDB.fallback_font
	var font_size: int = 10

	for i in range(buildings.size()):
		var b = buildings[i]

		# Viewport culling
		var tile_x: int = int(_building_value(b, "tile_x", 0))
		var tile_y: int = int(_building_value(b, "tile_y", 0))
		var building_type: String = str(_building_value(b, "building_type", ""))
		var is_built: bool = bool(_building_value(b, "is_built", _building_value(b, "is_constructed", false)))
		var build_progress: float = float(_building_value(b, "build_progress", _building_value(b, "construction_progress", 0.0)))

		if tile_x < min_tile_x or tile_x > max_tile_x:
			continue
		if tile_y < min_tile_y or tile_y > max_tile_y:
			continue

		var cx: float = float(tile_x) * tile_size + half
		var cy: float = float(tile_y) * tile_size + half
		var alpha: float = 1.0 if is_built else 0.4

		if _current_lod == 0:
			if zl < 0.4:
				continue
			var strategic_color: Color = Color(0.6, 0.35, 0.15, alpha)
			match building_type:
				"stockpile":
					strategic_color = Color(0.6, 0.35, 0.15, alpha)
				"shelter":
					strategic_color = Color(0.9, 0.55, 0.2, alpha)
				"campfire":
					strategic_color = Color(0.9, 0.2, 0.15, alpha)
			draw_rect(Rect2(cx - 1.5, cy - 1.5, 3.0, 3.0), strategic_color, true)
			continue

			match building_type:
				"stockpile":
					_draw_stockpile(cx, cy, alpha, tile_size, zl)
				"shelter":
					_draw_shelter(cx, cy, alpha, tile_size, zl)
				"campfire":
					_draw_campfire(cx, cy, alpha, tile_size, zl)
				_:
					pass
		_draw_building_interior(b, tile_x, tile_y, tile_size, zl)

		# Construction progress bar
		if not is_built:
			var building_size: float = tile_size * 0.8
			var bar_w: float = building_size
			var bar_h: float = 3.0
			var bar_x: float = cx - bar_w * 0.5
			var bar_y: float = cy + building_size * 0.5 + 2.0
			draw_rect(Rect2(bar_x, bar_y, bar_w, bar_h), Color(0.2, 0.2, 0.2, 0.6))
			draw_rect(Rect2(bar_x, bar_y, bar_w * build_progress, bar_h), Color(0.2, 0.8, 0.2, 0.8))

		if _current_lod == 2 and building_type == "stockpile" and is_built:
			var storage: Dictionary = {}
			var storage_raw: Variant = _building_value(b, "storage", {})
			if storage_raw is Dictionary:
				storage = storage_raw
			var food: int = int(round(storage.get("food", 0.0)))
			var wood: int = int(round(storage.get("wood", 0.0)))
			var stone: int = int(round(storage.get("stone", 0.0)))
			var text: String = Locale.trf3("UI_STATS_RESOURCES_FMT", "food", food, "wood", wood, "stone", stone)
			draw_string(font, Vector2(cx - 20, cy + half + 14), text, HORIZONTAL_ALIGNMENT_CENTER, -1, font_size, Color.WHITE)
		elif _current_lod >= 1:
			var building_label: String = Locale.tr_id("BUILDING", building_type)
			if building_label.is_empty() or building_label == building_type:
				building_label = Locale.tr_id("BUILDING_TYPE", building_type)
			if building_label.is_empty() or building_label == building_type:
				building_label = building_type.capitalize()
			draw_string(
				font,
				Vector2(cx, cy - (tile_size * 0.8) - 4.0),
				building_label,
				HORIZONTAL_ALIGNMENT_CENTER,
				64.0,
				9,
				Color(0.95, 0.84, 0.58, 0.92)
			)

	# Settlement labels at settlement-scale zoom
	if _current_lod == 0 and zl >= 0.4 and zl < 0.8:
		var settlements: Array = _get_runtime_settlements()
		if settlements.is_empty() and _settlement_manager != null:
			settlements = _settlement_manager.get_active_settlements()
		for i in range(settlements.size()):
			var s: Variant = settlements[i]
			var sx: float = float(_settlement_value(s, "center_x", 0)) * tile_size + half
			var sy: float = float(_settlement_value(s, "center_y", 0)) * tile_size + half
			var sid: int = int(_settlement_value(s, "id", 0))
			var pop: int = int(_settlement_value(s, "population", _settlement_member_count(s)))
			var label: String = Locale.trf2("UI_SETTLEMENT_LABEL_FMT", "id", sid, "pop", pop)
			draw_string(font, Vector2(sx - 15, sy - 8), label, HORIZONTAL_ALIGNMENT_CENTER, -1, 10, Color(1, 1, 0.6, 0.9))


func _update_lod(zl: float) -> void:
	match _current_lod:
		0:
			if zl >= 0.9:
				_current_lod = 1
		1:
			if zl < 0.75:
				_current_lod = 0
			elif zl >= 2.2:
				_current_lod = 2
		2:
			if zl < 2.0:
				_current_lod = 1


func _draw_stockpile(cx: float, cy: float, alpha: float, tile_size: int, zoom_level: float) -> void:
	var size: float = tile_size * 0.8 * _zoom_shape_scale(zoom_level)
	var half_size: float = size * 0.5
	var fill_color := Color(0.55, 0.35, 0.15, alpha)
	var outline_color := Color(0.9, 0.7, 0.3, alpha)

	draw_rect(Rect2(cx - half_size, cy - half_size, size, size), fill_color, true)
	draw_rect(Rect2(cx - half_size, cy - half_size, size, size), outline_color, false, 2.0)


func _draw_shelter(cx: float, cy: float, alpha: float, tile_size: int, zoom_level: float) -> void:
	var size: float = tile_size * 0.8 * _zoom_shape_scale(zoom_level)
	var half_size: float = size * 0.5
	var fill_color := Color(0.7, 0.4, 0.2, alpha)
	var outline_color := Color(1.0, 0.8, 0.4, alpha)

	var points := PackedVector2Array([
		Vector2(cx, cy - half_size),
		Vector2(cx - half_size, cy + half_size),
		Vector2(cx + half_size, cy + half_size),
	])

	draw_colored_polygon(points, fill_color)
	draw_polyline(PackedVector2Array([points[0], points[1], points[2], points[0]]), outline_color, 2.0)


func _draw_campfire(cx: float, cy: float, alpha: float, tile_size: int, zoom_level: float) -> void:
	var size: float = tile_size * 0.8 * _zoom_shape_scale(zoom_level)
	var radius: float = size * 0.35
	var fill_color := Color(1.0, 0.4, 0.1, alpha)
	var glow_color := Color(1.0, 0.4, 0.1, alpha * 0.15)

	draw_circle(Vector2(cx, cy), radius, fill_color)
	draw_arc(Vector2(cx, cy), tile_size * 3.0, 0, TAU, 32, glow_color, 1.5)


func _zoom_shape_scale(zoom_level: float) -> float:
	return clampf(2.5 / maxf(zoom_level, 0.5), 0.9, 2.2)


func _draw_building_interior(building: Variant, tile_x: int, tile_y: int, tile_size: int, zoom_level: float) -> void:
	if zoom_level < 2.0:
		return
	var building_type: String = str(_building_value(building, "building_type", ""))
	var dimensions: Vector2i = _building_dimensions(building_type, building)
	var width_tiles: int = maxi(1, dimensions.x)
	var height_tiles: int = maxi(1, dimensions.y)
	var px: float = float(tile_x) * tile_size
	var py: float = float(tile_y) * tile_size
	var width_px: float = float(width_tiles * tile_size)
	var height_px: float = float(height_tiles * tile_size)

	if building_type != "campfire":
		draw_rect(Rect2(px, py, width_px, height_px), Color(0.10, 0.08, 0.03, 0.32), true)

	if building_type in ["shelter", "stockpile", "workshop"]:
		var wall_color := Color(0.35, 0.29, 0.16, 0.8)
		var wall_width: float = maxf(2.0, tile_size * 0.18)
		var door_left: float = px + width_px * 0.5 - tile_size * 0.4
		var door_right: float = door_left + tile_size * 0.8
		draw_line(Vector2(px, py), Vector2(px + width_px, py), wall_color, wall_width)
		draw_line(Vector2(px, py), Vector2(px, py + height_px), wall_color, wall_width)
		draw_line(Vector2(px + width_px, py), Vector2(px + width_px, py + height_px), wall_color, wall_width)
		draw_line(Vector2(px, py + height_px), Vector2(door_left, py + height_px), wall_color, wall_width)
		draw_line(Vector2(door_right, py + height_px), Vector2(px + width_px, py + height_px), wall_color, wall_width)
		draw_line(
			Vector2(door_left + tile_size * 0.1, py + height_px),
			Vector2(door_right - tile_size * 0.1, py + height_px),
			Color(0.41, 0.28, 0.09, 0.85),
			maxf(1.0, wall_width * 0.5)
		)

	var font: Font = ThemeDB.fallback_font
	var icon_size: int = maxi(7, int(tile_size * 0.55))
	match building_type:
		"stockpile":
			_draw_furniture_icon(font, px + tile_size * 1.0, py + tile_size * 1.1, "📦", icon_size)
			_draw_furniture_icon(font, px + tile_size * 2.0, py + tile_size * 1.1, "📦", icon_size)
		"shelter":
			for fy: int in range(mini(height_tiles, 2)):
				for fx: int in range(mini(width_tiles, 3)):
					_draw_furniture_icon(
						font,
						px + (float(fx) + 0.5) * tile_size,
						py + (float(fy) + 1.2) * tile_size,
						"🛏️",
						icon_size
					)
		"campfire":
			_draw_furniture_icon(font, px + width_px * 0.5, py + height_px * 0.5 + tile_size * 0.15, "🔥", maxi(9, int(tile_size * 0.7)))
		"workshop":
			_draw_furniture_icon(font, px + tile_size * 1.0, py + tile_size * 1.1, "🪓", icon_size)
			_draw_furniture_icon(font, px + tile_size * 2.0, py + tile_size * 1.1, "⚒️", icon_size)
		_:
			pass


func _draw_furniture_icon(font: Font, x: float, y: float, icon: String, size: int) -> void:
	draw_string(font, Vector2(x, y), icon, HORIZONTAL_ALIGNMENT_CENTER, -1, size, Color(1.0, 1.0, 1.0, 0.92))


func _building_dimensions(building_type: String, building: Variant) -> Vector2i:
	var width_tiles: int = int(_building_value(building, "width", _building_value(building, "tile_w", 0)))
	var height_tiles: int = int(_building_value(building, "height", _building_value(building, "tile_h", 0)))
	if width_tiles > 0 and height_tiles > 0:
		return Vector2i(width_tiles, height_tiles)
	match building_type:
		"campfire":
			return Vector2i(1, 1)
		"shelter":
			return Vector2i(3, 2)
		"stockpile", "workshop":
			return Vector2i(3, 3)
		_:
			return Vector2i(2, 2)


func _building_value(building: Variant, key: String, default_value: Variant) -> Variant:
	if building is Dictionary:
		return building.get(key, default_value)
	if building == null:
		return default_value
	return building.get(key)


func _settlement_value(settlement: Variant, key: String, default_value: Variant) -> Variant:
	if settlement is Dictionary:
		return settlement.get(key, default_value)
	if settlement == null:
		return default_value
	return settlement.get(key)


func _settlement_member_count(settlement: Variant) -> int:
	if settlement is Dictionary:
		var member_ids: Variant = settlement.get("member_ids", [])
		if member_ids is Array:
			return member_ids.size()
		return int(settlement.get("population", 0))
	if settlement == null:
		return 0
	return settlement.member_ids.size()


func _get_runtime_minimap_snapshot() -> Dictionary:
	if _sim_engine == null or not _sim_engine.has_method("get_minimap_snapshot"):
		return {}
	var tick: int = int(_sim_engine.current_tick)
	if tick != _runtime_minimap_cache_tick:
		_runtime_minimap_cache_tick = tick
		_runtime_minimap_cache = _sim_engine.get_minimap_snapshot()
	return _runtime_minimap_cache


func _get_runtime_buildings() -> Array:
	var snapshot: Dictionary = _get_runtime_minimap_snapshot()
	var buildings: Variant = snapshot.get("buildings", [])
	return buildings if buildings is Array else []


func _get_runtime_settlements() -> Array:
	var summary: Dictionary = _get_runtime_world_summary()
	var settlement_summaries: Variant = summary.get("settlement_summaries", [])
	if settlement_summaries is Array:
		var settlements: Array = []
		for settlement_summary_raw: Variant in settlement_summaries:
			if not (settlement_summary_raw is Dictionary):
				continue
			var settlement_summary: Dictionary = settlement_summary_raw
			var settlement_raw: Variant = settlement_summary.get("settlement", {})
			if settlement_raw is Dictionary:
				settlements.append(settlement_raw)
		if not settlements.is_empty():
			return settlements
	var snapshot: Dictionary = _get_runtime_minimap_snapshot()
	var fallback_settlements: Variant = snapshot.get("settlements", [])
	return fallback_settlements if fallback_settlements is Array else []


func _get_runtime_world_summary() -> Dictionary:
	if _sim_engine == null or not _sim_engine.has_method("get_world_summary"):
		return {}
	var tick: int = int(_sim_engine.current_tick)
	if tick != _runtime_world_summary_cache_tick:
		_runtime_world_summary_cache_tick = tick
		_runtime_world_summary_cache = _sim_engine.get_world_summary()
	return _runtime_world_summary_cache
