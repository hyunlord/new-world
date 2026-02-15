class_name BuildingRenderer
extends Node2D

var _building_manager: RefCounted
var _current_lod: int = 1


func init(building_manager: RefCounted) -> void:
	_building_manager = building_manager


func _process(_delta: float) -> void:
	queue_redraw()


func _draw() -> void:
	if _building_manager == null:
		return
	var cam := get_viewport().get_camera_2d()
	var zl: float = cam.zoom.x if cam else 1.0
	_update_lod(zl)

	var buildings: Array = _building_manager.get_all_buildings()
	var tile_size: int = GameConfig.TILE_SIZE
	var half: float = tile_size * 0.5
	var font: Font = ThemeDB.fallback_font
	var font_size: int = 10

	for i in range(buildings.size()):
		var b = buildings[i]
		var cx: float = float(b.tile_x) * tile_size + half
		var cy: float = float(b.tile_y) * tile_size + half
		var alpha: float = 1.0 if b.is_built else 0.4

		if _current_lod == 0:
			var strategic_color: Color = Color(0.6, 0.35, 0.15, alpha)
			match b.building_type:
				"stockpile":
					strategic_color = Color(0.6, 0.35, 0.15, alpha)
				"shelter":
					strategic_color = Color(0.9, 0.55, 0.2, alpha)
				"campfire":
					strategic_color = Color(0.9, 0.2, 0.15, alpha)
			draw_rect(Rect2(cx - 1.5, cy - 1.5, 3.0, 3.0), strategic_color, true)
			continue

		match b.building_type:
			"stockpile":
				_draw_stockpile(cx, cy, alpha, tile_size)
			"shelter":
				_draw_shelter(cx, cy, alpha, tile_size)
			"campfire":
				_draw_campfire(cx, cy, alpha, tile_size)
			_:
				pass

		# Construction progress bar
		if not b.is_built:
			var building_size: float = tile_size * 0.8
			var bar_w: float = building_size
			var bar_h: float = 3.0
			var bar_x: float = cx - bar_w * 0.5
			var bar_y: float = cy + building_size * 0.5 + 2.0
			draw_rect(Rect2(bar_x, bar_y, bar_w, bar_h), Color(0.2, 0.2, 0.2, 0.6))
			draw_rect(Rect2(bar_x, bar_y, bar_w * b.build_progress, bar_h), Color(0.2, 0.8, 0.2, 0.8))

		if _current_lod == 2 and b.building_type == "stockpile" and b.is_built:
			var food: int = int(round(b.storage.get("food", 0.0)))
			var wood: int = int(round(b.storage.get("wood", 0.0)))
			var stone: int = int(round(b.storage.get("stone", 0.0)))
			var text: String = "F:%d W:%d S:%d" % [food, wood, stone]
			draw_string(font, Vector2(cx - 20, cy + half + 14), text, HORIZONTAL_ALIGNMENT_CENTER, -1, font_size, Color.WHITE)


func _update_lod(zl: float) -> void:
	match _current_lod:
		0:
			if zl >= 1.7:
				_current_lod = 1
		1:
			if zl < 1.3:
				_current_lod = 0
			elif zl >= 4.2:
				_current_lod = 2
		2:
			if zl < 3.8:
				_current_lod = 1


func _draw_stockpile(cx: float, cy: float, alpha: float, tile_size: int) -> void:
	var size: float = tile_size * 0.8
	var half_size: float = size * 0.5
	var fill_color := Color(0.55, 0.35, 0.15, alpha)
	var outline_color := Color(0.9, 0.7, 0.3, alpha)

	# Filled rectangle
	draw_rect(Rect2(cx - half_size, cy - half_size, size, size), fill_color, true)
	# Bright yellow outline border
	draw_rect(Rect2(cx - half_size, cy - half_size, size, size), outline_color, false, 2.0)


func _draw_shelter(cx: float, cy: float, alpha: float, tile_size: int) -> void:
	var size: float = tile_size * 0.8
	var half_size: float = size * 0.5
	var fill_color := Color(0.7, 0.4, 0.2, alpha)
	var outline_color := Color(1.0, 0.8, 0.4, alpha)

	# Triangle points: top vertex, bottom-left, bottom-right
	var points := PackedVector2Array([
		Vector2(cx, cy - half_size),           # Top vertex
		Vector2(cx - half_size, cy + half_size),  # Bottom-left
		Vector2(cx + half_size, cy + half_size),  # Bottom-right
	])

	# Filled triangle
	draw_colored_polygon(points, fill_color)
	# Light outline
	draw_polyline(PackedVector2Array([points[0], points[1], points[2], points[0]]), outline_color, 2.0)


func _draw_campfire(cx: float, cy: float, alpha: float, tile_size: int) -> void:
	var size: float = tile_size * 0.8
	var radius: float = size * 0.35
	var fill_color := Color(1.0, 0.4, 0.1, alpha)
	var glow_color := Color(1.0, 0.4, 0.1, alpha * 0.15)

	# Filled circle
	draw_circle(Vector2(cx, cy), radius, fill_color)
	# Glow ring at 3x tile_size radius
	draw_arc(Vector2(cx, cy), tile_size * 3.0, 0, TAU, 32, glow_color, 1.5)
