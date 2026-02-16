extends RefCounted

const EntityDataScript = preload("res://scripts/core/entity_data.gd")
const ChunkIndex = preload("res://scripts/core/chunk_index.gd")

var _entities: Dictionary = {}  # id -> entity
var _next_id: int = 1
var _world_data: RefCounted
var _rng: RandomNumberGenerator
var _settlement_manager: RefCounted
var chunk_index: RefCounted  # ChunkIndex for O(1) spatial lookups

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
	chunk_index = ChunkIndex.new()


func _generate_name() -> String:
	return FIRST_NAMES[_rng.randi() % FIRST_NAMES.size()]


## Spawn a new entity at the given position
## initial_age: age in ticks (0 = newborn child, use AGE_TEEN_END+ for adults)
func spawn_entity(pos: Vector2i, gender_override: String = "", initial_age: int = 0) -> RefCounted:
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
	# Gender (50:50 or override)
	if gender_override != "":
		entity.gender = gender_override
	else:
		entity.gender = "female" if _rng.randf() < 0.5 else "male"
	# Personality (immutable, randomized)
	entity.personality = {
		"openness": _rng.randf_range(0.1, 0.9),
		"agreeableness": _rng.randf_range(0.1, 0.9),
		"extraversion": _rng.randf_range(0.1, 0.9),
		"diligence": _rng.randf_range(0.1, 0.9),
		"emotional_stability": _rng.randf_range(0.1, 0.9),
	}
	# Emotions (defaults)
	entity.emotions = {
		"happiness": 0.5,
		"loneliness": 0.0,
		"stress": 0.0,
		"grief": 0.0,
		"love": 0.0,
	}
	# Age and age stage
	entity.age = initial_age
	entity.age_stage = GameConfig.get_age_stage(entity.age)
	# Frailty: N(1.0, 0.15), clamped [0.5, 2.0] (Vaupel frailty model)
	entity.frailty = clampf(_rng.randfn(1.0, 0.15), 0.5, 2.0)
	_entities[entity.id] = entity
	_world_data.register_entity(pos, entity.id)
	chunk_index.add_entity(entity.id, pos)
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
	chunk_index.update_entity(entity.id, old_pos, new_pos)
	entity.position = new_pos


## Kill an entity
func kill_entity(entity_id: int, cause: String, tick: int = -1) -> void:
	if not _entities.has(entity_id):
		return
	var entity = _entities[entity_id]
	# Settlement cleanup
	if _settlement_manager != null and entity.settlement_id > 0:
		_settlement_manager.remove_member(entity.settlement_id, entity.id)
	entity.is_alive = false
	_world_data.unregister_entity(entity.position, entity.id)
	chunk_index.remove_entity(entity.id, entity.position)
	SimulationBus.emit_event("entity_died", {
		"entity_id": entity.id,
		"entity_name": entity.entity_name,
		"cause": cause,
		"position": entity.position,
		"tick": tick,
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


## Get entities within radius of position (chunk-based, O(chunks * chunk_size))
func get_entities_near(pos: Vector2i, radius: int) -> Array:
	var result: Array = []
	var ids: Array = chunk_index.get_nearby_entity_ids(pos, radius)
	for i in range(ids.size()):
		var entity = _entities.get(ids[i], null)
		if entity != null and entity.is_alive:
			var dx: int = absi(entity.position.x - pos.x)
			var dy: int = absi(entity.position.y - pos.y)
			if dx <= radius and dy <= radius:
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
	chunk_index.clear()
	for i in range(data.size()):
		var item = data[i]
		if item is Dictionary:
			var entity = EntityDataScript.from_dict(item)
			_entities[entity.id] = entity
			if entity.is_alive:
				world_data.register_entity(entity.position, entity.id)
				chunk_index.add_entity(entity.id, entity.position)
			if entity.id >= _next_id:
				_next_id = entity.id + 1
