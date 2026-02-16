extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted
var _world_data: RefCounted
var _pathfinder: RefCounted
var _building_manager: RefCounted


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


func execute_tick(tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
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
			entity.cached_path = []
			entity.path_index = 0
			continue

		# Age-based movement speed reduction (skip some movement ticks)
		# child=50%, teen=80%, elder=~67%
		var skip_move: bool = false
		match entity.age_stage:
			"infant", "toddler":
				if (tick + entity.id) % 2 == 0:
					skip_move = true
			"child":
				if (tick + entity.id) % 2 == 0:
					skip_move = true
			"teen":
				if (tick + entity.id) % 5 == 0:
					skip_move = true
			"elder":
				if (tick + entity.id) % 3 == 0:
					skip_move = true
			"ancient":
				if (tick + entity.id) % 2 == 0:
					skip_move = true
		if skip_move:
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
			_move_with_pathfinding(entity, tick)
		else:
			_move_toward_target(entity, tick)


## ─── A* Pathfinding Movement ─────────────────────────────

func _move_with_pathfinding(entity: RefCounted, tick: int) -> void:
	var needs_recalc: bool = false
	if entity.cached_path.is_empty():
		needs_recalc = true
	elif entity.path_index >= entity.cached_path.size():
		needs_recalc = true
	elif tick % 50 == 0:
		needs_recalc = true

	if needs_recalc:
		entity.cached_path = _pathfinder.find_path(
			_world_data, entity.position, entity.action_target
		)
		entity.path_index = 0
		# Skip starting position if it matches current
		if entity.cached_path.size() > 0 and entity.cached_path[0] == entity.position:
			entity.path_index = 1

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
			entity.cached_path = []
			entity.path_index = 0
			_move_toward_target(entity, tick)
	else:
		# Path exhausted, fall back to greedy
		_move_toward_target(entity, tick)


## ─── Greedy Fallback Movement ────────────────────────────

func _move_toward_target(entity: RefCounted, tick: int) -> void:
	var pos: Vector2i = entity.position
	var target: Vector2i = entity.action_target
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

	for j in range(candidates.size()):
		var candidate: Vector2i = candidates[j]
		if _world_data.is_walkable(candidate.x, candidate.y):
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
			return


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
