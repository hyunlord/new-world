class_name EntityRenderer
extends Node2D

const EntityDataClass = preload("res://scripts/core/entity_data.gd")
const EntityManagerClass = preload("res://scripts/core/entity_manager.gd")

var _entity_manager: RefCounted
var selected_entity_id: int = -1

const SELECTION_RADIUS: float = 7.0
const HUNGER_WARNING_RADIUS: float = 2.0
const HUNGER_WARNING_THRESHOLD: float = 0.2

## Job visual definitions: shape, size, color
const JOB_VISUALS: Dictionary = {
	"none": {"size": 3.0, "color": Color(0.6, 0.6, 0.6)},
	"gatherer": {"size": 4.0, "color": Color(0.3, 0.8, 0.2)},
	"lumberjack": {"size": 5.0, "color": Color(0.6, 0.35, 0.1)},
	"builder": {"size": 5.0, "color": Color(0.9, 0.6, 0.1)},
	"miner": {"size": 4.0, "color": Color(0.5, 0.6, 0.75)},
}

## Resource indicator colors
const RES_COLORS: Dictionary = {
	"food": Color(0.8, 0.9, 0.2),
	"wood": Color(0.2, 0.5, 0.1),
	"stone": Color(0.7, 0.7, 0.72),
}


## Initialize with entity manager reference
func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager


func _process(_delta: float) -> void:
	queue_redraw()


func _draw() -> void:
	if _entity_manager == null:
		return
	var alive: Array = _entity_manager.get_alive_entities()
	var half_tile := Vector2(GameConfig.TILE_SIZE * 0.5, GameConfig.TILE_SIZE * 0.5)

	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		var pos := Vector2(entity.position) * GameConfig.TILE_SIZE + half_tile
		var vis: Dictionary = JOB_VISUALS.get(entity.job, JOB_VISUALS["none"])
		var size: float = vis["size"]
		var color: Color = vis["color"]

		# Draw job-based shape
		match entity.job:
			"lumberjack":
				_draw_triangle(pos, size, color)
			"builder":
				_draw_square(pos, size, color)
			"miner":
				_draw_diamond(pos, size, color)
			_:
				draw_circle(pos, size, color)

		# Carrying indicator (small dot above entity)
		if entity.get_total_carry() > 0.0:
			var best_res: String = _get_dominant_resource(entity)
			var dot_color: Color = RES_COLORS.get(best_res, Color.WHITE)
			draw_circle(pos + Vector2(0, -(size + 3.0)), 1.5, dot_color)

		# Hunger warning
		if entity.hunger < HUNGER_WARNING_THRESHOLD:
			draw_circle(pos + Vector2(0, -(size + 5.0)), HUNGER_WARNING_RADIUS, Color.RED)

		# Selection highlight
		if entity.id == selected_entity_id:
			draw_arc(pos, SELECTION_RADIUS, 0, TAU, 24, Color.WHITE, 1.5)
			# Draw line to action target
			if entity.action_target != Vector2i(-1, -1):
				var target_pos := Vector2(entity.action_target) * GameConfig.TILE_SIZE + half_tile
				draw_dashed_line(pos, target_pos, Color(1, 1, 1, 0.3), 1.0, 4.0)


func _draw_triangle(center: Vector2, size: float, color: Color) -> void:
	var points := PackedVector2Array([
		center + Vector2(0, -size),
		center + Vector2(-size * 0.87, size * 0.5),
		center + Vector2(size * 0.87, size * 0.5),
	])
	draw_colored_polygon(points, color)


func _draw_square(center: Vector2, size: float, color: Color) -> void:
	var half: float = size * 0.5
	draw_rect(Rect2(center.x - half, center.y - half, size, size), color)


func _draw_diamond(center: Vector2, size: float, color: Color) -> void:
	var points := PackedVector2Array([
		center + Vector2(0, -size),
		center + Vector2(size, 0),
		center + Vector2(0, size),
		center + Vector2(-size, 0),
	])
	draw_colored_polygon(points, color)


func _get_dominant_resource(entity: RefCounted) -> String:
	var best: String = "food"
	var best_amount: float = 0.0
	var keys: Array = entity.inventory.keys()
	for j in range(keys.size()):
		var res: String = keys[j]
		var amount: float = entity.inventory[res]
		if amount > best_amount:
			best_amount = amount
			best = res
	return best


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		_handle_click(event.global_position)


func _handle_click(screen_pos: Vector2) -> void:
	if _entity_manager == null:
		return
	# Convert screen position to world position
	var canvas_transform := get_canvas_transform()
	var world_pos: Vector2 = canvas_transform.affine_inverse() * screen_pos
	var tile := Vector2i(int(world_pos.x) / GameConfig.TILE_SIZE, int(world_pos.y) / GameConfig.TILE_SIZE)

	# Find entity at or near this tile
	var alive: Array = _entity_manager.get_alive_entities()
	var best_entity: RefCounted = null
	var best_dist: float = 3.0  # max click distance in tiles
	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		var dist: float = Vector2(entity.position - tile).length()
		if dist < best_dist:
			best_dist = dist
			best_entity = entity

	if best_entity:
		selected_entity_id = best_entity.id
		SimulationBus.entity_selected.emit(best_entity.id)
	else:
		selected_entity_id = -1
		SimulationBus.entity_deselected.emit()
