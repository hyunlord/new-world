class_name BehaviorSystem
extends SimulationSystem

var _entity_manager: EntityManager
var _world_data: WorldData
var _rng: RandomNumberGenerator


func _init() -> void:
	system_name = "behavior"
	priority = 20
	tick_interval = 5


## Initialize with references
func init(entity_manager: EntityManager, world_data: WorldData, rng: RandomNumberGenerator) -> void:
	_entity_manager = entity_manager
	_world_data = world_data
	_rng = rng


func execute_tick(tick: int) -> void:
	var alive: Array[EntityData] = _entity_manager.get_alive_entities()
	for entity: EntityData in alive:
		if entity.action_timer > 0:
			continue
		var scores := _evaluate_actions(entity)
		var best_action: String = "wander"
		var best_score: float = -1.0
		for action: String in scores:
			if scores[action] > best_score:
				best_score = scores[action]
				best_action = action
		_assign_action(entity, best_action, tick)


func _evaluate_actions(entity: EntityData) -> Dictionary:
	var hunger_deficit: float = 1.0 - entity.hunger
	var energy_deficit: float = 1.0 - entity.energy
	var social_deficit: float = 1.0 - entity.social
	return {
		"wander": 0.2 + _rng.randf() * 0.1,
		"seek_food": _urgency_curve(hunger_deficit) * 1.5,
		"rest": _urgency_curve(energy_deficit) * 1.2,
		"socialize": _urgency_curve(social_deficit) * 0.8,
	}


## Exponential urgency: higher deficit = much higher urgency
func _urgency_curve(deficit: float) -> float:
	return pow(deficit, 2.0)


func _assign_action(entity: EntityData, action: String, tick: int) -> void:
	entity.current_action = action
	match action:
		"wander":
			entity.action_target = _find_random_walkable_nearby(entity.position, 5)
			entity.action_timer = 5
		"seek_food":
			var food_tile := _find_food_tile(entity.position, 10)
			entity.action_target = food_tile
			entity.action_timer = 15
		"rest":
			entity.action_target = entity.position
			entity.action_timer = 10
		"socialize":
			var neighbor := _find_nearest_entity(entity)
			entity.action_target = neighbor
			entity.action_timer = 8
	emit_event("action_chosen", {
		"entity_id": entity.id,
		"entity_name": entity.entity_name,
		"action": action,
		"tick": tick,
	})


func _find_random_walkable_nearby(pos: Vector2i, radius: int) -> Vector2i:
	var candidates: Array[Vector2i] = []
	for dy in range(-radius, radius + 1):
		for dx in range(-radius, radius + 1):
			if dx == 0 and dy == 0:
				continue
			var nx: int = pos.x + dx
			var ny: int = pos.y + dy
			if _world_data.is_walkable(nx, ny):
				candidates.append(Vector2i(nx, ny))
	if candidates.is_empty():
		return pos
	return candidates[_rng.randi() % candidates.size()]


func _find_food_tile(pos: Vector2i, radius: int) -> Vector2i:
	var candidates: Array[Vector2i] = []
	for dy in range(-radius, radius + 1):
		for dx in range(-radius, radius + 1):
			var nx: int = pos.x + dx
			var ny: int = pos.y + dy
			if not _world_data.is_valid(nx, ny):
				continue
			var biome: int = _world_data.get_biome(nx, ny)
			if biome == GameConfig.Biome.GRASSLAND or biome == GameConfig.Biome.FOREST:
				candidates.append(Vector2i(nx, ny))
	if candidates.is_empty():
		return _find_random_walkable_nearby(pos, radius)
	return candidates[_rng.randi() % candidates.size()]


func _find_nearest_entity(entity: EntityData) -> Vector2i:
	var nearby := _entity_manager.get_entities_near(entity.position, 10)
	var best_dist: int = 999999
	var best_pos: Vector2i = entity.position
	for other: EntityData in nearby:
		if other.id == entity.id:
			continue
		var dist: int = absi(other.position.x - entity.position.x) + absi(other.position.y - entity.position.y)
		if dist < best_dist:
			best_dist = dist
			best_pos = other.position
	if best_dist == 999999:
		return _find_random_walkable_nearby(entity.position, 5)
	return best_pos
