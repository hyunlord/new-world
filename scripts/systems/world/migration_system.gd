extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _world_data: RefCounted
var _resource_map: RefCounted
var _rng: RandomNumberGenerator
var _last_migration_tick: int = 0


func _init() -> void:
	system_name = "migration"
	priority = 60
	tick_interval = GameConfig.MIGRATION_TICK_INTERVAL


func init(entity_manager: RefCounted, building_manager: RefCounted, settlement_manager: RefCounted, world_data: RefCounted, resource_map: RefCounted, rng: RandomNumberGenerator) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_settlement_manager = settlement_manager
	_world_data = world_data
	_resource_map = resource_map
	_rng = rng


func execute_tick(tick: int) -> void:
	## Cleanup empty settlements periodically
	if tick % GameConfig.SETTLEMENT_CLEANUP_INTERVAL == 0:
		_settlement_manager.cleanup_empty_settlements()

	## Cooldown check
	if tick - _last_migration_tick < GameConfig.MIGRATION_COOLDOWN_TICKS:
		return

	## Settlement cap check
	var active_count: int = _settlement_manager.get_active_settlements().size()
	if active_count >= GameConfig.MAX_SETTLEMENTS:
		return

	var settlements: Array = _settlement_manager.get_all_settlements()
	for i in range(settlements.size()):
		var settlement: RefCounted = settlements[i]
		var population: int = _settlement_manager.get_settlement_population(settlement.id)

		## Must have minimum population for migration
		if population < GameConfig.MIGRATION_MIN_POP:
			continue

		## Re-check cap (might have created one already)
		if _settlement_manager.get_active_settlements().size() >= GameConfig.MAX_SETTLEMENTS:
			break

		var shelter_count: int = _count_settlement_shelters(settlement.id)
		var overcrowded: bool = population > shelter_count * 8

		var nearby_food: float = _get_food_in_radius(settlement.center_x, settlement.center_y, 20)
		var food_scarce: bool = nearby_food < float(population) * 0.3

		var explorer_chance: bool = _rng.randf() < GameConfig.MIGRATION_CHANCE

		if not overcrowded and not food_scarce and not explorer_chance:
			continue

		## Build candidate list from this settlement
		var candidates: Array = []
		var alive_entities: Array = _entity_manager.get_alive_entities()
		for j in range(alive_entities.size()):
			var entity: RefCounted = alive_entities[j]
			if entity.settlement_id == settlement.id:
				candidates.append(entity)

		if candidates.size() < GameConfig.MIGRATION_GROUP_SIZE_MIN:
			continue

		## Sort by social ascending (least social leave first)
		candidates.sort_custom(Callable(self, "_sort_social_ascending"))
		var group_size: int = _rng.randi_range(GameConfig.MIGRATION_GROUP_SIZE_MIN, GameConfig.MIGRATION_GROUP_SIZE_MAX)
		if group_size > candidates.size():
			group_size = candidates.size()

		var migrants: Array = []
		for j in range(group_size):
			migrants.append(candidates[j])

		## Ensure proper group composition (builder + gatherer + lumberjack)
		_ensure_group_composition(migrants, candidates)

		if migrants.size() < GameConfig.MIGRATION_GROUP_SIZE_MIN:
			continue

		## Check if source settlement can supply startup resources
		var startup: Dictionary = {
			"food": GameConfig.MIGRATION_STARTUP_FOOD,
			"wood": GameConfig.MIGRATION_STARTUP_WOOD,
			"stone": GameConfig.MIGRATION_STARTUP_STONE,
		}
		if not _can_withdraw_from_stockpiles(settlement.id, startup):
			continue

		## Find a suitable migration site
		var best_site: Vector2i = _find_migration_site(settlement, settlements)
		if best_site.x < 0 or best_site.y < 0:
			continue

		## Withdraw resources from source settlement stockpiles
		_withdraw_from_stockpiles(settlement.id, startup)

		## Distribute startup resources to migrants
		_distribute_startup_resources(migrants, startup)

		## Create new settlement
		var new_settlement: RefCounted = _settlement_manager.create_settlement(best_site.x, best_site.y, tick)

		## Transfer migrants to new settlement
		for j in range(migrants.size()):
			var migrant_entity: RefCounted = migrants[j]
			_settlement_manager.remove_member(settlement.id, migrant_entity.id)
			_settlement_manager.add_member(new_settlement.id, migrant_entity.id)
			migrant_entity.settlement_id = new_settlement.id
			migrant_entity.current_action = "migrate"
			migrant_entity.action_target = Vector2i(best_site.x, best_site.y)
			migrant_entity.action_timer = 100
			migrant_entity.cached_path = []
			migrant_entity.path_index = 0

		_last_migration_tick = tick

		emit_event("migration_started", {
			"from_settlement": settlement.id,
			"to_settlement": new_settlement.id,
			"migrant_count": migrants.size(),
			"site_x": best_site.x,
			"site_y": best_site.y,
			"tick": tick,
		})
		emit_event("settlement_founded", {
			"settlement_id": new_settlement.id,
			"center_x": best_site.x,
			"center_y": best_site.y,
			"tick": tick,
		})

		## Only one migration per execution tick
		break


## ─── Group Composition ─────────────────────────────────────

func _ensure_group_composition(migrants: Array, candidates: Array) -> void:
	var needed_jobs: Array = ["builder", "gatherer", "lumberjack"]
	for ni in range(needed_jobs.size()):
		var job: String = needed_jobs[ni]
		var found: bool = false
		for mi in range(migrants.size()):
			if migrants[mi].job == job:
				found = true
				break
		if found:
			continue
		## Find someone with this job from the broader candidate pool
		for ci in range(candidates.size()):
			var candidate: RefCounted = candidates[ci]
			if candidate.job != job:
				continue
			var already_in: bool = false
			for mi in range(migrants.size()):
				if migrants[mi].id == candidate.id:
					already_in = true
					break
			if not already_in and migrants.size() > 0:
				migrants[migrants.size() - 1] = candidate
				break


## ─── Startup Resource Management ───────────────────────────

func _can_withdraw_from_stockpiles(settlement_id: int, resources: Dictionary) -> bool:
	var available: Dictionary = _get_settlement_stockpile_totals(settlement_id)
	var keys: Array = resources.keys()
	for i in range(keys.size()):
		var res: String = keys[i]
		if available.get(res, 0.0) < resources[res]:
			return false
	return true


func _get_settlement_stockpile_totals(settlement_id: int) -> Dictionary:
	var totals: Dictionary = {"food": 0.0, "wood": 0.0, "stone": 0.0}
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	for i in range(stockpiles.size()):
		var sp: RefCounted = stockpiles[i]
		if sp.settlement_id != settlement_id or not sp.is_built:
			continue
		var keys: Array = sp.storage.keys()
		for j in range(keys.size()):
			var res: String = keys[j]
			totals[res] = totals.get(res, 0.0) + sp.storage[res]
	return totals


func _withdraw_from_stockpiles(settlement_id: int, resources: Dictionary) -> void:
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	var res_keys: Array = resources.keys()
	for i in range(res_keys.size()):
		var res: String = res_keys[i]
		var needed: float = resources[res]
		for j in range(stockpiles.size()):
			if needed <= 0.0:
				break
			var sp: RefCounted = stockpiles[j]
			if sp.settlement_id != settlement_id or not sp.is_built:
				continue
			var available: float = sp.storage.get(res, 0.0)
			var take: float = minf(available, needed)
			sp.storage[res] = available - take
			needed -= take


func _distribute_startup_resources(migrants: Array, resources: Dictionary) -> void:
	if migrants.is_empty():
		return
	## Food distributed evenly
	var per_person_food: float = resources.get("food", 0.0) / float(migrants.size())
	for i in range(migrants.size()):
		migrants[i].add_item("food", per_person_food)
	## Wood and stone go to builder
	var wood: float = resources.get("wood", 0.0)
	var stone: float = resources.get("stone", 0.0)
	var builder_found: bool = false
	for i in range(migrants.size()):
		if migrants[i].job == "builder":
			migrants[i].add_item("wood", wood)
			migrants[i].add_item("stone", stone)
			builder_found = true
			break
	if not builder_found and migrants.size() > 0:
		migrants[0].add_item("wood", wood)
		migrants[0].add_item("stone", stone)


## ─── Site Selection ────────────────────────────────────────

func _find_migration_site(source: RefCounted, all_settlements: Array) -> Vector2i:
	var best_site: Vector2i = Vector2i(-1, -1)
	var best_score: float = -1.0
	for j in range(20):
		var dx: int = _rng.randi_range(-GameConfig.MIGRATION_SEARCH_RADIUS_MAX, GameConfig.MIGRATION_SEARCH_RADIUS_MAX)
		var dy: int = _rng.randi_range(-GameConfig.MIGRATION_SEARCH_RADIUS_MAX, GameConfig.MIGRATION_SEARCH_RADIUS_MAX)
		var distance: int = absi(dx) + absi(dy)
		if distance < GameConfig.MIGRATION_SEARCH_RADIUS_MIN or distance > GameConfig.MIGRATION_SEARCH_RADIUS_MAX:
			continue

		var x: int = source.center_x + dx
		var y: int = source.center_y + dy

		if not _world_data.is_valid(x, y):
			continue
		if not _world_data.is_walkable(x, y):
			continue

		var far_enough: bool = true
		for k in range(all_settlements.size()):
			var other_settlement: RefCounted = all_settlements[k]
			var dist_to_settlement: int = absi(other_settlement.center_x - x) + absi(other_settlement.center_y - y)
			if dist_to_settlement < GameConfig.SETTLEMENT_MIN_DISTANCE:
				far_enough = false
				break
		if not far_enough:
			continue

		var food_score: float = _get_food_score(x, y, 5)
		if food_score <= 3.0:
			continue

		if food_score > best_score:
			best_score = food_score
			best_site = Vector2i(x, y)

	return best_site


## ─── Utility Functions ─────────────────────────────────────

func _count_settlement_shelters(settlement_id: int) -> int:
	var shelters: int = 0
	var buildings: Array = _building_manager.get_all_buildings()
	for i in range(buildings.size()):
		var building: RefCounted = buildings[i]
		if building.settlement_id == settlement_id and building.building_type == "shelter" and building.is_built:
			shelters += 1
	return shelters


func _get_food_in_radius(cx: int, cy: int, radius: int) -> float:
	var total_food: float = 0.0
	for dx in range(-radius, radius + 1):
		for dy in range(-radius, radius + 1):
			if absi(dx) + absi(dy) > radius:
				continue
			var x: int = cx + dx
			var y: int = cy + dy
			if not _world_data.is_valid(x, y):
				continue
			total_food += _resource_map.get_food(x, y)
	return total_food


func _get_food_score(x: int, y: int, radius: int) -> float:
	var score: float = 0.0
	for dx in range(-radius, radius + 1):
		for dy in range(-radius, radius + 1):
			if absi(dx) + absi(dy) > radius:
				continue
			var tx: int = x + dx
			var ty: int = y + dy
			if not _world_data.is_valid(tx, ty):
				continue
			score += _resource_map.get_food(tx, ty)
	return score


func _sort_social_ascending(a: RefCounted, b: RefCounted) -> bool:
	return a.social < b.social
