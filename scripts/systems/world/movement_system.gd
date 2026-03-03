extends "res://scripts/core/simulation/simulation_system.gd"

var _entity_manager: RefCounted
var _world_data: RefCounted
var _pathfinder: RefCounted
var _building_manager: RefCounted
var _recalc_from_xy: PackedInt32Array = PackedInt32Array()
var _recalc_to_xy: PackedInt32Array = PackedInt32Array()
var _path_entities_scratch: Array = []
var _recalc_entities_scratch: Array = []
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_MOVE_SKIP_METHOD: String = "body_movement_should_skip_tick"
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func _init() -> void:
	system_name = "movement"
	priority = 30
	tick_interval = GameConfig.MOVEMENT_TICK_INTERVAL


## Initialize (pathfinder and building_manager optional for backward compat)
func init(entity_manager: RefCounted, world_data: RefCounted, pathfinder: RefCounted = null, building_manager: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_world_data = world_data
	_pathfinder = pathfinder
	_building_manager = building_manager


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
	if node != null and node.has_method(_SIM_BRIDGE_MOVE_SKIP_METHOD):
		_sim_bridge = node
	return _sim_bridge


func execute_tick(tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	var path_entities: Array = _path_entities_scratch
	var recalc_entities: Array = _recalc_entities_scratch
	var periodic_recalc_tick: bool = (tick % 50) == 0
	path_entities.clear()
	recalc_entities.clear()
	_recalc_from_xy.resize(0)
	_recalc_to_xy.resize(0)
	var bridge: Object = _get_sim_bridge()

	for i in range(alive.size()):
		var entity = alive[i]
		# Countdown action timer
		if entity.action_timer > 0:
			entity.action_timer -= 1

		# Check if action completed (timer expired)
		if entity.action_timer <= 0 and entity.current_action != "idle":
			_apply_arrival_effect(entity, tick)
			entity.current_action = "idle"
			entity.action_target = Vector2i(-1, -1)
			_clear_cached_path(entity)
			continue

		# Age-based movement speed: skip ticks based on config
		var skip_mod: int = GameConfig.CHILD_MOVE_SKIP_MOD.get(entity.age_stage, 0)
		var should_skip: bool = skip_mod > 0 and (tick + entity.id) % skip_mod == 0
		if bridge != null:
			var rust_variant: Variant = bridge.call(
				_SIM_BRIDGE_MOVE_SKIP_METHOD,
				skip_mod,
				tick,
				entity.id,
			)
			if rust_variant != null:
				should_skip = bool(rust_variant)
		if should_skip:
			continue

		# Skip movement for rest/idle or if already at target
		if entity.current_action == "rest" or entity.current_action == "idle":
			continue
		if entity.action_target == Vector2i(-1, -1):
			continue
		if entity.action_target == entity.position:
			continue

		# Move: A* if pathfinder available, else greedy
		if _pathfinder != null:
			path_entities.append(entity)
			if _needs_path_recalc(entity, periodic_recalc_tick):
				recalc_entities.append(entity)
				_recalc_from_xy.append(entity.position.x)
				_recalc_from_xy.append(entity.position.y)
				_recalc_to_xy.append(entity.action_target.x)
				_recalc_to_xy.append(entity.action_target.y)
		else:
			_move_toward_target(entity, tick)

	if _pathfinder != null and not recalc_entities.is_empty():
		var paths: Array = _pathfinder.find_paths_batch_xy(
			_world_data,
			_recalc_from_xy,
			_recalc_to_xy
		)
		var recalc_count: int = mini(paths.size(), recalc_entities.size())
		for i in range(recalc_count):
			_apply_recalculated_path(recalc_entities[i], paths[i])
		for i in range(recalc_count, recalc_entities.size()):
			_clear_cached_path(recalc_entities[i])

	if _pathfinder != null:
		for i in range(path_entities.size()):
			_move_with_pathfinding(path_entities[i], tick, false)


## ─── A* Pathfinding Movement ─────────────────────────────

func _move_with_pathfinding(entity: RefCounted, tick: int, allow_recalc: bool = true) -> void:
	if allow_recalc and _needs_path_recalc(entity, (tick % 50) == 0):
		var recalculated_path: Array = _pathfinder.find_path(
			_world_data, entity.position, entity.action_target
		)
		_apply_recalculated_path(entity, recalculated_path)

	# Follow cached path
	if entity.path_index < entity.cached_path.size():
		var next_pos: Vector2i = entity.cached_path[entity.path_index]
		if _world_data.is_walkable(next_pos.x, next_pos.y):
			var old_pos: Vector2i = entity.position
			_entity_manager.move_entity(entity, next_pos)
			entity.path_index += 1
			SimulationBus.emit_event("entity_moved", {
				"entity_id": entity.id,
				"from_x": old_pos.x,
				"from_y": old_pos.y,
				"to_x": next_pos.x,
				"to_y": next_pos.y,
				"tick": tick,
			})
		else:
			# Path blocked, clear and fall back to greedy
			_clear_cached_path(entity)
			_move_toward_target(entity, tick)
	else:
		# Path exhausted, fall back to greedy
		_move_toward_target(entity, tick)


func _needs_path_recalc(entity: RefCounted, periodic_recalc_tick: bool) -> bool:
	var cached_size: int = entity.cached_path.size()
	if cached_size <= 0:
		return true
	if entity.path_index >= cached_size:
		return true
	if periodic_recalc_tick:
		return true
	return false


func _apply_recalculated_path(entity: RefCounted, path: Array) -> void:
	var path_size: int = path.size()
	if path_size <= 0:
		_clear_cached_path(entity)
		return
	entity.cached_path = path
	entity.path_index = 0
	# Skip starting position if it matches current
	if entity.cached_path[0] == entity.position:
		entity.path_index = 1


func _clear_cached_path(entity: RefCounted) -> void:
	if entity.cached_path is Array:
		entity.cached_path.clear()
	else:
		entity.cached_path = []
	entity.path_index = 0


## ─── Greedy Fallback Movement ────────────────────────────

func _move_toward_target(entity: RefCounted, tick: int) -> void:
	var pos: Vector2i = entity.position
	var target: Vector2i = entity.action_target
	var dx: int = signi(target.x - pos.x)
	var dy: int = signi(target.y - pos.y)

	# Try diagonal first, then axis-aligned (same priority, no temp array allocation).
	if dx != 0 and dy != 0:
		if _try_move_candidate(entity, tick, Vector2i(pos.x + dx, pos.y + dy)):
			return
	if dx != 0:
		if _try_move_candidate(entity, tick, Vector2i(pos.x + dx, pos.y)):
			return
	if dy != 0:
		_try_move_candidate(entity, tick, Vector2i(pos.x, pos.y + dy))


func _try_move_candidate(entity: RefCounted, tick: int, candidate: Vector2i) -> bool:
	if not _world_data.is_walkable(candidate.x, candidate.y):
		return false
	var old_pos: Vector2i = entity.position
	_entity_manager.move_entity(entity, candidate)
	SimulationBus.emit_event("entity_moved", {
		"entity_id": entity.id,
		"from_x": old_pos.x,
		"from_y": old_pos.y,
		"to_x": candidate.x,
		"to_y": candidate.y,
		"tick": tick,
	})
	return true


## ─── Arrival Effects ─────────────────────────────────────

func _apply_arrival_effect(entity: RefCounted, tick: int) -> void:
	match entity.current_action:
		"gather_food":
			# Eat gathered food from inventory
			var food_in_inv: float = entity.inventory.get("food", 0.0)
			var eat_amount: float = minf(food_in_inv, 3.0)
			if eat_amount > 0.0:
				entity.remove_item("food", eat_amount)
				entity.hunger = minf(entity.hunger + eat_amount * GameConfig.FOOD_HUNGER_RESTORE, 1.0)
			else:
				# Minor foraging (backward compat with seek_food)
				entity.hunger = minf(entity.hunger + 0.15, 1.0)
			emit_event("entity_ate", {
				"entity_id": entity.id,
				"entity_name": entity.entity_name,
				"hunger_after": entity.hunger,
				"tick": tick,
			})
		"seek_food":
			# Legacy: direct hunger recovery
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
		"deliver_to_stockpile":
			_deliver_to_stockpile(entity, tick)
		"take_from_stockpile":
			_take_from_stockpile(entity, tick)
		"drink_water":
			entity.thirst = minf(entity.thirst + GameConfig.THIRST_DRINK_RESTORE, 1.0)
			entity.action_timer = 0
			emit_event("entity_drank", {
				"entity_id": entity.id,
				"entity_name": entity.entity_name,
				"thirst_after": entity.thirst,
				"tick": tick,
			})
		"sit_by_fire":
			## [Cannon 1932] Fire restores warmth at WARMTH_FIRE_RESTORE per arrival
			if _building_manager != null:
				var fire_building = _building_manager.get_nearest_building(
					entity.position.x, entity.position.y, "campfire", true)
				if fire_building != null:
					var dist: int = absi(entity.position.x - fire_building.tile_x) + absi(entity.position.y - fire_building.tile_y)
					if dist <= 1:
						entity.warmth = minf(entity.warmth + GameConfig.WARMTH_FIRE_RESTORE, 1.0)
						entity.action_timer = 0
						emit_event("entity_warmed", {
							"entity_id": entity.id,
							"entity_name": entity.entity_name,
							"warmth_after": entity.warmth,
							"tick": tick,
						})
		"seek_shelter":
			## [Maslow 1943 L2] Shelter restores warmth (minor) + safety (minor) per tick inside
			if _building_manager != null:
				var shelter_building = _building_manager.get_nearest_building(
					entity.position.x, entity.position.y, "shelter", true)
				if shelter_building != null:
					var dist: int = absi(entity.position.x - shelter_building.tile_x) + absi(entity.position.y - shelter_building.tile_y)
					if dist <= 1:
						entity.warmth = minf(entity.warmth + GameConfig.WARMTH_SHELTER_RESTORE, 1.0)
						entity.safety = minf(entity.safety + GameConfig.SAFETY_SHELTER_RESTORE, 1.0)
						entity.action_timer = 0
						emit_event("entity_sheltered", {
							"entity_id": entity.id,
							"entity_name": entity.entity_name,
							"warmth_after": entity.warmth,
							"safety_after": entity.safety,
							"tick": tick,
						})
		"gather_wood", "gather_stone", "build", "wander":
			pass

	# Auto-eat on any action completion when hungry
	_try_auto_eat(entity, tick)


func _try_auto_eat(entity: RefCounted, tick: int) -> void:
	if entity.hunger >= GameConfig.HUNGER_EAT_THRESHOLD:
		return
	var food_in_inv: float = entity.inventory.get("food", 0.0)
	if food_in_inv <= 0.0:
		return
	var eat_amount: float = minf(food_in_inv, 2.0)
	entity.remove_item("food", eat_amount)
	entity.hunger = minf(entity.hunger + eat_amount * GameConfig.FOOD_HUNGER_RESTORE, 1.0)
	emit_event("auto_eat", {
		"entity_id": entity.id,
		"entity_name": entity.entity_name,
		"amount": eat_amount,
		"hunger_after": entity.hunger,
		"tick": tick,
	})


func _deliver_to_stockpile(entity: RefCounted, tick: int) -> void:
	if _building_manager == null:
		return
	var stockpile = _building_manager.get_nearest_building(
		entity.position.x, entity.position.y, "stockpile", true
	)
	if stockpile == null:
		return
	var dist: int = absi(entity.position.x - stockpile.tile_x) + absi(entity.position.y - stockpile.tile_y)
	if dist > 1:
		return
	var inv_keys: Array = entity.inventory.keys()
	var total_delivered: float = 0.0
	for j in range(inv_keys.size()):
		var res_type: String = inv_keys[j]
		var amount: float = entity.inventory[res_type]
		if amount > 0.0:
			stockpile.storage[res_type] = stockpile.storage.get(res_type, 0.0) + amount
			entity.inventory[res_type] = 0.0
			total_delivered += amount
	if total_delivered > 0.0:
		emit_event("resources_delivered", {
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"building_id": stockpile.id,
			"amount": total_delivered,
			"tick": tick,
		})


func _take_from_stockpile(entity: RefCounted, tick: int) -> void:
	if _building_manager == null:
		return
	var stockpile = _building_manager.get_nearest_building(
		entity.position.x, entity.position.y, "stockpile", true
	)
	if stockpile == null:
		return
	var dist: int = absi(entity.position.x - stockpile.tile_x) + absi(entity.position.y - stockpile.tile_y)
	if dist > 1:
		return
	var available_food: float = stockpile.storage.get("food", 0.0)
	var take_amount: float = minf(available_food, 3.0)
	if take_amount > 0.0:
		stockpile.storage["food"] = available_food - take_amount
		entity.hunger = minf(entity.hunger + take_amount * 0.25, 1.0)
		emit_event("food_taken", {
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"building_id": stockpile.id,
			"amount": take_amount,
			"hunger_after": entity.hunger,
			"tick": tick,
		})
