extends RefCounted

const EntityDataScript = preload("res://scripts/core/entity_data.gd")
const ChunkIndex = preload("res://scripts/core/chunk_index.gd")
const GameCalendarScript = preload("res://scripts/core/game_calendar.gd")
const PersonalityDataScript = preload("res://scripts/core/personality_data.gd")
const PersonalityGeneratorScript = preload("res://scripts/systems/personality_generator.gd")

var _entities: Dictionary = {}  # id -> entity
var _next_id: int = 1
var _world_data: RefCounted
var _rng: RandomNumberGenerator
var _settlement_manager: RefCounted
var _personality_generator: RefCounted
var chunk_index: RefCounted  # ChunkIndex for O(1) spatial lookups
var total_deaths: int = 0
var total_births: int = 0


## Initialize with world data and RNG reference
func init(world_data: RefCounted, rng: RandomNumberGenerator) -> void:
	_world_data = world_data
	_rng = rng
	chunk_index = ChunkIndex.new()
	_personality_generator = PersonalityGeneratorScript.new()
	_personality_generator.init(rng)


## Spawn a new entity at the given position
## initial_age: age in ticks (0 = newborn child, use AGE_TEEN_END+ for adults)
func spawn_entity(pos: Vector2i, gender_override: String = "", initial_age: int = 0, parent_a: RefCounted = null, parent_b: RefCounted = null) -> RefCounted:
	var entity = EntityDataScript.new()
	entity.id = _next_id
	_next_id += 1
	entity.position = pos
	entity.speed = 0.8 + _rng.randf() * 0.4
	entity.strength = 0.8 + _rng.randf() * 0.4
	entity.hunger = 0.7 + _rng.randf() * 0.3
	entity.energy = 0.7 + _rng.randf() * 0.3
	entity.social = 0.5 + _rng.randf() * 0.5
	# Gender (50:50 or override) — must be set before name generation
	if gender_override != "":
		entity.gender = gender_override
	else:
		entity.gender = "female" if _rng.randf() < 0.5 else "male"
	entity.entity_name = NameGenerator.generate_name(entity.gender)
	# Personality (HEXACO 24-facet, Cholesky-correlated with parental inheritance)
	var pa_pd = parent_a.personality if parent_a != null else null
	var pb_pd = parent_b.personality if parent_b != null else null
	entity.personality = _personality_generator.generate_personality(entity.gender, "", pa_pd, pb_pd)
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
	# Set birth_tick: negative for pre-existing entities (born before game start)
	if initial_age > 0:
		entity.birth_tick = -initial_age
	entity.birth_date = GameCalendarScript.birth_date_from_tick(entity.birth_tick, _rng)
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
	# Register in DeceasedRegistry BEFORE removing
	if Engine.has_singleton("DeceasedRegistry") or entity.get_meta("_skip_deceased", false) == false:
		var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
		if registry != null:
			registry.register_death(entity, cause, tick)
	# Settlement cleanup
	if _settlement_manager != null and entity.settlement_id > 0:
		_settlement_manager.remove_member(entity.settlement_id, entity.id)
	var age_years: float = float(entity.age) / float(GameConfig.TICKS_PER_YEAR)
	entity.is_alive = false
	total_deaths += 1
	_world_data.unregister_entity(entity.position, entity.id)
	chunk_index.remove_entity(entity.id, entity.position)
	SimulationBus.emit_event("entity_died", {
		"entity_id": entity.id,
		"entity_name": entity.entity_name,
		"cause": cause,
		"position": entity.position,
		"tick": tick,
	})
	# Lifecycle signal for ChronicleSystem
	SimulationBus.entity_died.emit(entity.id, entity.entity_name, cause, age_years, tick)


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


## Register a birth (called by FamilySystem)
func register_birth() -> void:
	total_births += 1


## Load entities from saved data
func load_save_data(data: Array, world_data: RefCounted) -> void:
	_entities.clear()
	_next_id = 1
	total_deaths = 0
	total_births = 0
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
