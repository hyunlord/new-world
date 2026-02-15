class_name BuildingRenderer
extends Node2D

var _building_manager: RefCounted


func init(building_manager: RefCounted) -> void:
	_building_manager = building_manager


func _process(_delta: float) -> void:
	queue_redraw()


func _draw() -> void:
	if _building_manager == null:
		return
	var buildings: Array = _building_manager.get_all_buildings()
	var tile_size: int = GameConfig.TILE_SIZE
	var half: float = tile_size * 0.5

	for i in range(buildings.size()):
		var b = buildings[i]
		var cx: float = float(b.tile_x) * tile_size + half
		var cy: float = float(b.tile_y) * tile_size + half
		var alpha: float = 1.0 if b.is_built else 0.4

		match b.building_type:
			"stockpile":
				_draw_stockpile(cx, cy, alpha)
			"shelter":
				_draw_shelter(cx, cy, alpha)
			"campfire":
				_draw_campfire(cx, cy, alpha)

		# Construction progress bar
		if not b.is_built:
			var bar_w: float = 10.0
			var bar_h: float = 2.0
			var bar_x: float = cx - bar_w * 0.5
			var bar_y: float = cy + 10.0
			draw_rect(Rect2(bar_x, bar_y, bar_w, bar_h), Color(0.2, 0.2, 0.2, 0.6))
			draw_rect(Rect2(bar_x, bar_y, bar_w * b.build_progress, bar_h), Color(0.2, 0.8, 0.2, 0.8))


func _draw_stockpile(cx: float, cy: float, alpha: float) -> void:
	var size: float = 6.0
	var color := Color(0.55, 0.35, 0.1, alpha)
	draw_rect(Rect2(cx - size, cy - size, size * 2, size * 2), color, false, 2.0)


func _draw_shelter(cx: float, cy: float, alpha: float) -> void:
	var size: float = 7.0
	var color := Color(0.55, 0.35, 0.1, alpha)
	var points := PackedVector2Array([
		Vector2(cx, cy - size),
		Vector2(cx - size, cy + size * 0.6),
		Vector2(cx + size, cy + size * 0.6),
	])
	draw_colored_polygon(points, color)


func _draw_campfire(cx: float, cy: float, alpha: float) -> void:
	var color := Color(0.95, 0.5, 0.1, alpha)
	draw_circle(Vector2(cx, cy), 3.0, color)
	# Glow ring
	draw_arc(Vector2(cx, cy), 5.0, 0, TAU, 16, Color(0.9, 0.3, 0.05, alpha * 0.3), 1.0)
