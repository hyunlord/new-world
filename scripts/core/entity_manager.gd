extends RefCounted

const EntityDataScript = preload("res://scripts/core/entity_data.gd")

var _entities: Dictionary = {}  # id -> entity
var _next_id: int = 1
var _world_data: RefCounted
var _rng: RandomNumberGenerator

const FIRST_NAMES: PackedStringArray = [
	"Alder", "Bryn", "Cedar", "Dawn", "Elm", "Fern", "Glen", "Heath",
	"Ivy", "Jade", "Kael", "Luna", "Moss", "Nix", "Oak", "Pine",
	"Quinn", "Reed", "Sage", "Thorn", "Uma", "Vale", "Wren", "Xara",
	"Yew", "Zara", "Ash", "Brook", "Clay", "Dusk",
]


## Initialize with world data and RNG reference
func init(world_data: RefCounted, rng: RandomNumberGenerator) -> void:
	_world_data = world_data
	_rng = rng


func _generate_name() -> String:
	return FIRST_NAMES[_rng.randi() % FIRST_NAMES.size()]


## Spawn a new entity at the given position
func spawn_entity(pos: Vector2i) -> RefCounted:
	var entity = EntityDataScript.new()
	entity.id = _next_id
	_next_id += 1
	entity.entity_name = _generate_name()
	entity.position = pos
	entity.speed = 0.8 + _rng.randf() * 0.4
	entity.strength = 0.8 + _rng.randf() * 0.4
	entity.hunger = 0.7 + _rng.randf() * 0.3
	entity.energy = 0.7 + _rng.randf() * 0.3
	entity.social = 0.5 + _rng.randf() * 0.5
	_entities[entity.id] = entity
	_world_data.register_entity(pos, entity.id)
	SimulationBus.emit_event("entity_spawned", {
		"entity_id": entity.id,
		"entity_name": entity.entity_name,
		"position": pos,
		"tick": 0,
	})
	return entity


## Move an entity to a new position
func move_entity(entity: RefCounted, new_pos: Vector2i) -> void:
	var old_pos: Vector2i = entity.position
	_world_data.move_entity(old_pos, new_pos, entity.id)
	entity.position = new_pos


## Kill an entity
func kill_entity(entity_id: int, cause: String) -> void:
	if not _entities.has(entity_id):
		return
	var entity = _entities[entity_id]
	entity.is_alive = false
	_world_data.unregister_entity(entity.position, entity.id)
	SimulationBus.emit_event("entity_died", {
		"entity_id": entity.id,
		"entity_name": entity.entity_name,
		"cause": cause,
		"position": entity.position,
	})


## Get entity by ID
func get_entity(id: int) -> RefCounted:
	return _entities.get(id, null)


## Get all alive entities
func get_alive_entities() -> Array:
	var result: Array = []
	var all_entities: Array = _entities.values()
	for i in range(all_entities.size()):
		var entity = all_entities[i]
		if entity.is_alive:
			result.append(entity)
	return result


## Get alive entity count
func get_alive_count() -> int:
	var count: int = 0
	var all_entities: Array = _entities.values()
	for i in range(all_entities.size()):
		var entity = all_entities[i]
		if entity.is_alive:
			count += 1
	return count


## Get entities within radius of position
func get_entities_near(pos: Vector2i, radius: int) -> Array:
	var result: Array = []
	var all_entities: Array = _entities.values()
	for i in range(all_entities.size()):
		var entity = all_entities[i]
		if entity.is_alive:
			var dist: int = absi(entity.position.x - pos.x) + absi(entity.position.y - pos.y)
			if dist <= radius:
				result.append(entity)
	return result


## Serialize all entities
func to_save_data() -> Array:
	var result: Array = []
	var all_entities: Array = _entities.values()
	for i in range(all_entities.size()):
		var entity = all_entities[i]
		result.append(entity.to_dict())
	return result


## Load entities from saved data
func load_save_data(data: Array, world_data: RefCounted) -> void:
	_entities.clear()
	_next_id = 1
	for i in range(data.size()):
		var item = data[i]
		if item is Dictionary:
			var entity = EntityDataScript.from_dict(item)
			_entities[entity.id] = entity
			if entity.is_alive:
				world_data.register_entity(entity.position, entity.id)
			if entity.id >= _next_id:
				_next_id = entity.id + 1
