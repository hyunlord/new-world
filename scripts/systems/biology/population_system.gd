extends "res://scripts/core/simulation/simulation_system.gd"

const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_POP_BLOCK_METHOD: String = "body_population_birth_block_code"
const _SIM_BRIDGE_POP_HOUSING_CAP_METHOD: String = "body_population_housing_cap"
const _POP_MIN_FOR_BIRTH: int = 5
const _POP_FREE_HOUSING_CAP: int = 25
const _POP_SHELTER_CAPACITY: int = 6
const _POP_FOOD_PER_ALIVE: float = 0.5

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _world_data: RefCounted
var _rng: RandomNumberGenerator
var _settlement_manager: RefCounted
var _bridge_checked: bool = false
var _sim_bridge: Object = null


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


func _get_sim_bridge() -> Object:
	if _bridge_checked:
		return _sim_bridge
	_bridge_checked = true
	var tree: SceneTree = Engine.get_main_loop() as SceneTree
	if tree == null:
		return null
	var root: Node = tree.get_root()
	if root == null:
		return null
	var node: Node = root.get_node_or_null(_SIM_BRIDGE_NODE_NAME)
	if node != null \
	and node.has_method(_SIM_BRIDGE_POP_BLOCK_METHOD) \
	and node.has_method(_SIM_BRIDGE_POP_HOUSING_CAP_METHOD):
		_sim_bridge = node
	return _sim_bridge


func _population_birth_block_code(alive_count: int, total_shelters: int, total_food: float) -> int:
	var max_entities: int = int(GameConfig.MAX_ENTITIES)
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_POP_BLOCK_METHOD,
			alive_count,
			max_entities,
			total_shelters,
			total_food,
			_POP_MIN_FOR_BIRTH,
			_POP_FREE_HOUSING_CAP,
			_POP_SHELTER_CAPACITY,
			_POP_FOOD_PER_ALIVE
		)
		if rust_variant is int:
			return int(rust_variant)
	if alive_count >= max_entities:
		return 1
	if alive_count < _POP_MIN_FOR_BIRTH:
		return 2
	if alive_count >= _POP_FREE_HOUSING_CAP and total_shelters * _POP_SHELTER_CAPACITY < alive_count:
		return 3
	if total_food < float(alive_count) * _POP_FOOD_PER_ALIVE:
		return 4
	return 0


func _population_housing_cap(total_shelters: int) -> int:
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_POP_HOUSING_CAP_METHOD,
			total_shelters,
			_POP_FREE_HOUSING_CAP,
			_POP_SHELTER_CAPACITY
		)
		if rust_variant is int:
			return int(rust_variant)
	if total_shelters <= 0:
		return _POP_FREE_HOUSING_CAP
	return total_shelters * _POP_SHELTER_CAPACITY


func execute_tick(_tick: int) -> void:
	# Births disabled: all reproduction handled by FamilySystem (T-1090)
	# Natural deaths disabled: handled by MortalitySystem (T-2000, Siler model)
	pass


func _check_births(tick: int) -> void:
	if _building_manager == null:
		return
	var alive_count: int = _entity_manager.get_alive_count()

	# Diagnostic logging every 200 ticks (more frequent for debugging)
	if tick % 200 == 0 and alive_count >= _POP_MIN_FOR_BIRTH:
		_log_population_status(tick, alive_count)

	# Count ALL shelters (built + under construction count toward housing)
	var shelters: Array = _building_manager.get_buildings_by_type("shelter")
	var total_shelters: int = shelters.size()

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

	var block_code: int = _population_birth_block_code(alive_count, total_shelters, total_food)
	if block_code != 0:
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
	var housing_cap: int = _population_housing_cap(total_shelters)
	var food_threshold: float = float(alive_count) * _POP_FOOD_PER_ALIVE
	var block_code: int = _population_birth_block_code(alive_count, total_shelters, total_food)
	var block_reason: String = ""
	if block_code == 1:
		block_reason = "MAX_ENTITIES(%d)" % GameConfig.MAX_ENTITIES
	elif block_code == 2:
		block_reason = "MIN_POPULATION(%d)" % _POP_MIN_FOR_BIRTH
	elif block_code == 3:
		block_reason = "housing(%d < %d)" % [total_shelters * _POP_SHELTER_CAPACITY, alive_count]
	elif block_code == 4:
		block_reason = "food(%.0f < %.0f)" % [total_food, food_threshold]
	else:
		block_reason = "NONE (birth allowed)"
	if GameConfig.DEBUG_DEMOGRAPHY_LOG:
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
