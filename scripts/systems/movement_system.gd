class_name MovementSystem
extends SimulationSystem

var _entity_manager: EntityManager
var _world_data: WorldData


func _init() -> void:
	system_name = "movement"
	priority = 30
	tick_interval = 1


## Initialize with references
func init(entity_manager: EntityManager, world_data: WorldData) -> void:
	_entity_manager = entity_manager
	_world_data = world_data


func execute_tick(tick: int) -> void:
	var alive := _entity_manager.get_alive_entities()
	for entity in alive:
		# Countdown action timer
		if entity.action_timer > 0:
			entity.action_timer -= 1

		# Check if action completed (timer expired)
		if entity.action_timer <= 0 and entity.current_action != "idle":
			_apply_arrival_effect(entity, tick)
			entity.current_action = "idle"
			entity.action_target = Vector2i(-1, -1)
			continue

		# Skip movement for rest/idle or if already at target
		if entity.current_action == "rest" or entity.current_action == "idle":
			continue
		if entity.action_target == Vector2i(-1, -1):
			continue
		if entity.action_target == entity.position:
			continue

		# Move toward target (greedy 8-direction)
		_move_toward_target(entity, tick)


func _move_toward_target(entity: EntityData, tick: int) -> void:
	var pos := entity.position
	var target := entity.action_target
	var dx: int = signi(target.x - pos.x)
	var dy: int = signi(target.y - pos.y)

	# Try diagonal first, then axis-aligned
	var candidates: Array[Vector2i] = []
	if dx != 0 and dy != 0:
		candidates.append(Vector2i(pos.x + dx, pos.y + dy))
	if dx != 0:
		candidates.append(Vector2i(pos.x + dx, pos.y))
	if dy != 0:
		candidates.append(Vector2i(pos.x, pos.y + dy))

	for candidate in candidates:
		if _world_data.is_walkable(candidate.x, candidate.y):
			var old_pos := entity.position
			_entity_manager.move_entity(entity, candidate)
			SimulationBus.emit_event("entity_moved", {
				"entity_id": entity.id,
				"from_x": old_pos.x,
				"from_y": old_pos.y,
				"to_x": candidate.x,
				"to_y": candidate.y,
				"tick": tick,
			})
			return


func _apply_arrival_effect(entity: EntityData, tick: int) -> void:
	match entity.current_action:
		"seek_food":
			entity.hunger = minf(entity.hunger + 0.4, 1.0)
			emit_event("entity_ate", {
				"entity_id": entity.id,
				"entity_name": entity.entity_name,
				"hunger_after": entity.hunger,
				"tick": tick,
			})
		"rest":
			entity.energy = minf(entity.energy + 0.5, 1.0)
			emit_event("entity_rested", {
				"entity_id": entity.id,
				"entity_name": entity.entity_name,
				"energy_after": entity.energy,
				"tick": tick,
			})
		"socialize":
			entity.social = minf(entity.social + 0.3, 1.0)
			emit_event("entity_socialized", {
				"entity_id": entity.id,
				"entity_name": entity.entity_name,
				"social_after": entity.social,
				"tick": tick,
			})
		"wander":
			pass
