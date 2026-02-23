extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _world_data: RefCounted
var _rng: RandomNumberGenerator
var _settlement_manager: RefCounted


func _init() -> void:
	system_name = "population"
	priority = 50
	tick_interval = GameConfig.POPULATION_TICK_INTERVAL


func init(entity_manager: RefCounted, building_manager: RefCounted, world_data: RefCounted, rng: RandomNumberGenerator, settlement_manager: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_world_data = world_data
	_rng = rng
	_settlement_manager = settlement_manager


func execute_tick(tick: int) -> void:
	# Births disabled: all reproduction handled by FamilySystem (T-1090)
	# Natural deaths disabled: handled by MortalitySystem (T-2000, Siler model)
	pass


func _check_births(tick: int) -> void:
	if _building_manager == null:
		return
	var alive_count: int = _entity_manager.get_alive_count()

	# Diagnostic logging every 200 ticks (more frequent for debugging)
	if tick % 200 == 0 and alive_count >= 5:
		_log_population_status(tick, alive_count)

	if alive_count >= GameConfig.MAX_ENTITIES:
		return

	# Minimum population for births
	if alive_count < 5:
		return

	# Count ALL shelters (built + under construction count toward housing)
	var shelters: Array = _building_manager.get_buildings_by_type("shelter")
	var total_shelters: int = shelters.size()

	# Housing check: allow up to 25 pop without shelters, then need shelters
	# Use < instead of <= so growth is allowed at exact boundary
	if alive_count >= 25 and total_shelters * 6 < alive_count:
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

	# Food threshold: need food >= alive_count * 0.5 (lowered from 1.0 — was blocking growth at ~49)
	if total_food < float(alive_count) * 0.5:
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
	# Assign to nearest settlement
	if _settlement_manager != null:
		var nearest_settlement: RefCounted = _settlement_manager.get_nearest_settlement(spawn_pos.x, spawn_pos.y)
		if nearest_settlement != null:
			new_entity.settlement_id = nearest_settlement.id
			_settlement_manager.add_member(nearest_settlement.id, new_entity.id)
	emit_event("entity_born", {
		"entity_id": new_entity.id,
		"entity_name": new_entity.entity_name,
		"reason": "population_growth",
		"position_x": spawn_pos.x,
		"position_y": spawn_pos.y,
		"tick": tick,
	})


## Old natural death logic removed — replaced by MortalitySystem (Siler model, T-2000)


func _log_population_status(tick: int, alive_count: int) -> void:
	var shelters: Array = _building_manager.get_buildings_by_type("shelter")
	var total_shelters: int = shelters.size()
	var built_shelters: int = 0
	for i in range(shelters.size()):
		if shelters[i].is_built:
			built_shelters += 1
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	var total_food: float = 0.0
	for i in range(stockpiles.size()):
		var sp = stockpiles[i]
		if sp.is_built:
			total_food += sp.storage.get("food", 0.0)
	var housing_cap: int = 25 if total_shelters == 0 else total_shelters * 6
	var food_threshold: float = float(alive_count) * 0.5
	var food_ok: bool = total_food >= food_threshold
	var housing_ok: bool = alive_count < 25 or total_shelters * 6 >= alive_count
	var at_max: bool = alive_count >= GameConfig.MAX_ENTITIES
	var block_reason: String = ""
	if at_max:
		block_reason = "MAX_ENTITIES(%d)" % GameConfig.MAX_ENTITIES
	elif not food_ok:
		block_reason = "food(%.0f < %.0f)" % [total_food, food_threshold]
	elif not housing_ok:
		block_reason = "housing(%d < %d)" % [total_shelters * 6, alive_count]
	else:
		block_reason = "NONE (birth allowed)"
	print("[Tick %d] [Pop] pop=%d food=%.0f/%.0f shelters=%d(%d built) cap=%d max=%d | block=%s" % [
		tick, alive_count, total_food, food_threshold, total_shelters, built_shelters, housing_cap, GameConfig.MAX_ENTITIES, block_reason,
	])


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
