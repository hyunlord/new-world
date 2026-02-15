extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _world_data: RefCounted
var _rng: RandomNumberGenerator


func _init() -> void:
	system_name = "population"
	priority = 50
	tick_interval = GameConfig.POPULATION_TICK_INTERVAL


func init(entity_manager: RefCounted, building_manager: RefCounted, world_data: RefCounted, rng: RandomNumberGenerator) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_world_data = world_data
	_rng = rng


func execute_tick(tick: int) -> void:
	_check_births(tick)
	_check_natural_deaths(tick)


func _check_births(tick: int) -> void:
	if _building_manager == null:
		return
	var alive_count: int = _entity_manager.get_alive_count()
	if alive_count >= GameConfig.MAX_ENTITIES:
		return

	# Minimum population for births
	if alive_count < 5:
		return

	# Count shelters
	var shelters: Array = _building_manager.get_buildings_by_type("shelter")
	var built_shelters: int = 0
	for i in range(shelters.size()):
		var s = shelters[i]
		if s.is_built:
			built_shelters += 1

	# Housing check: allow up to 25 pop without shelters, then need shelters
	if alive_count >= 25 and built_shelters * 6 <= alive_count:
		return

	# Sum food across all built stockpiles
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	var total_food: float = 0.0
	var best_stockpile: RefCounted = null
	var best_food: float = 0.0
	for i in range(stockpiles.size()):
		var sp = stockpiles[i]
		if not sp.is_built:
			continue
		var food: float = sp.storage.get("food", 0.0)
		total_food += food
		if food > best_food:
			best_food = food
			best_stockpile = sp

	# Food threshold: need food >= alive_count * 1.0 (relaxed from *2.0)
	if total_food < float(alive_count) * 1.0:
		return
	if best_stockpile == null:
		return

	# Consume food from best stockpile
	var food_available: float = best_stockpile.storage.get("food", 0.0)
	if food_available < GameConfig.BIRTH_FOOD_COST:
		return
	best_stockpile.storage["food"] = food_available - GameConfig.BIRTH_FOOD_COST

	# Spawn near stockpile
	var spawn_pos: Vector2i = _find_walkable_near(best_stockpile.tile_x, best_stockpile.tile_y)
	if spawn_pos == Vector2i(-1, -1):
		# Refund food if no spawn location
		best_stockpile.storage["food"] = best_stockpile.storage.get("food", 0.0) + GameConfig.BIRTH_FOOD_COST
		return

	var new_entity: RefCounted = _entity_manager.spawn_entity(spawn_pos)
	emit_event("entity_born", {
		"entity_id": new_entity.id,
		"entity_name": new_entity.entity_name,
		"reason": "population_growth",
		"position_x": spawn_pos.x,
		"position_y": spawn_pos.y,
		"tick": tick,
	})


func _check_natural_deaths(tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity = alive[i]
		if entity.age <= GameConfig.OLD_AGE_TICKS:
			continue
		var death_chance: float = 0.02
		if entity.age > GameConfig.MAX_AGE_TICKS:
			death_chance = 0.10
		if _rng.randf() < death_chance:
			_entity_manager.kill_entity(entity.id, "old_age")
			emit_event("entity_died_natural", {
				"entity_id": entity.id,
				"entity_name": entity.entity_name,
				"age": entity.age,
				"tick": tick,
			})


func _find_walkable_near(cx: int, cy: int) -> Vector2i:
	for radius in range(1, 6):
		for dy in range(-radius, radius + 1):
			for dx in range(-radius, radius + 1):
				if absi(dx) != radius and absi(dy) != radius:
					continue
				var x: int = cx + dx
				var y: int = cy + dy
				if _world_data.is_walkable(x, y):
					return Vector2i(x, y)
	return Vector2i(-1, -1)
