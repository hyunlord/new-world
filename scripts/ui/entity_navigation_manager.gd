class_name EntityNavigationManager
extends Node

signal entity_focus_requested(entity_id: int, target_position: Vector2)

## Explicit preload avoids relying on class_name auto-resolution in headless/preload contexts.
const _NavHistory = preload("res://scripts/ui/navigation_history.gd")

var history = _NavHistory.new()
var _entity_manager: RefCounted = null
## Cached SimulationBus reference — resolved at runtime so preloading this script
## in headless test mode (before autoloads exist) does not cause a compile error.
var _sim_bus: Node = null


func setup(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	# Runtime lookup: autoloads are children of /root and available after _initialize().
	_sim_bus = Engine.get_main_loop().get_root().get_node_or_null("SimulationBus") as Node
	if _sim_bus != null:
		_sim_bus.entity_navigation_requested.connect(_on_entity_navigation_requested)


func _on_entity_navigation_requested(entity_id: int) -> void:
	focus(entity_id, true)


func focus(entity_id: int, push_history: bool = true) -> void:
	if entity_id < 0:
		return
	var pos: Vector2 = _get_entity_tile_pos(entity_id)
	if pos.x < 0.0:
		return
	if push_history:
		history.push(entity_id)
	entity_focus_requested.emit(entity_id, pos)


func go_back() -> void:
	var prev_id: int = history.go_back()
	if prev_id >= 0:
		if _sim_bus != null:
			_sim_bus.entity_selected.emit(prev_id)
		var pos: Vector2 = _get_entity_tile_pos(prev_id)
		if pos.x >= 0.0:
			entity_focus_requested.emit(prev_id, pos)


func go_forward() -> void:
	var next_id: int = history.go_forward()
	if next_id >= 0:
		if _sim_bus != null:
			_sim_bus.entity_selected.emit(next_id)
		var pos: Vector2 = _get_entity_tile_pos(next_id)
		if pos.x >= 0.0:
			entity_focus_requested.emit(next_id, pos)


func _get_entity_tile_pos(entity_id: int) -> Vector2:
	if _entity_manager == null:
		return Vector2(-1.0, -1.0)
	var entity = _entity_manager.get_entity(entity_id)
	if entity == null:
		return Vector2(-1.0, -1.0)
	if "position" in entity:
		var p: Vector2i = entity.position
		return Vector2(float(p.x), float(p.y))
	return Vector2(-1.0, -1.0)
