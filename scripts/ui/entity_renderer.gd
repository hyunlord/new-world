class_name EntityRenderer
extends Node2D

var _entity_manager: EntityManager
var selected_entity_id: int = -1

const DOT_RADIUS: float = 4.0
const SELECTION_RADIUS: float = 6.0
const WARNING_RADIUS: float = 2.0
const HUNGER_WARNING_THRESHOLD: float = 0.2

const ACTION_COLORS: Dictionary = {
	"idle": Color.WHITE,
	"wander": Color(0.8, 0.8, 0.9),
	"seek_food": Color(1.0, 0.9, 0.2),
	"rest": Color(0.4, 0.4, 0.7),
	"socialize": Color(0.2, 0.9, 0.9),
}


## Initialize with entity manager reference
func init(entity_manager: EntityManager) -> void:
	_entity_manager = entity_manager


func _process(_delta: float) -> void:
	queue_redraw()


func _draw() -> void:
	if _entity_manager == null:
		return
	var alive: Array[EntityData] = _entity_manager.get_alive_entities()
	var half_tile := Vector2(GameConfig.TILE_SIZE * 0.5, GameConfig.TILE_SIZE * 0.5)

	for entity: EntityData in alive:
		var pos := Vector2(entity.position) * GameConfig.TILE_SIZE + half_tile
		var color: Color = ACTION_COLORS.get(entity.current_action, Color.WHITE)
		draw_circle(pos, DOT_RADIUS, color)

		# Hunger warning
		if entity.hunger < HUNGER_WARNING_THRESHOLD:
			draw_circle(pos + Vector2(0, -6), WARNING_RADIUS, Color.RED)

		# Selection highlight
		if entity.id == selected_entity_id:
			draw_arc(pos, SELECTION_RADIUS, 0, TAU, 24, Color.WHITE, 1.5)
			# Draw line to action target
			if entity.action_target != Vector2i(-1, -1):
				var target_pos := Vector2(entity.action_target) * GameConfig.TILE_SIZE + half_tile
				draw_dashed_line(pos, target_pos, Color(1, 1, 1, 0.3), 1.0, 4.0)


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
	var alive: Array[EntityData] = _entity_manager.get_alive_entities()
	var best_entity: EntityData = null
	var best_dist: float = 3.0  # max click distance in tiles
	for entity in alive:
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
