extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _world_data: RefCounted
var _resource_map: RefCounted
var _rng: RandomNumberGenerator


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
	var settlements: Array = _settlement_manager.get_all_settlements()
	for i in range(settlements.size()):
		var settlement: RefCounted = settlements[i]
		var population: int = _settlement_manager.get_settlement_population(settlement.id)
		if population < GameConfig.MIGRATION_GROUP_SIZE_MIN:
			continue

		var shelter_count: int = _count_settlement_shelters(settlement.id)
		var overcrowded: bool = population > shelter_count * 8

		var nearby_food: float = _get_food_in_radius(settlement.center_x, settlement.center_y, 20)
		var food_scarce: bool = nearby_food < float(population) * 0.5

		var explorer_chance: bool = population > GameConfig.MIGRATION_MIN_POP and _rng.randf() < GameConfig.MIGRATION_CHANCE

		if not overcrowded and not food_scarce and not explorer_chance:
			continue

		var candidates: Array = []
		var alive_entities: Array = _entity_manager.get_alive_entities()
		for j in range(alive_entities.size()):
			var entity: RefCounted = alive_entities[j]
			if entity.settlement_id == settlement.id:
				candidates.append(entity)

		if candidates.size() < GameConfig.MIGRATION_GROUP_SIZE_MIN:
			continue

		candidates.sort_custom(Callable(self, "_sort_social_ascending"))
		var group_size: int = _rng.randi_range(GameConfig.MIGRATION_GROUP_SIZE_MIN, GameConfig.MIGRATION_GROUP_SIZE_MAX)
		if group_size > candidates.size():
			group_size = candidates.size()

		var migrants: Array = []
		for j in range(group_size):
			migrants.append(candidates[j])

		var has_builder: bool = false
		for j in range(migrants.size()):
			var migrant: RefCounted = migrants[j]
			if migrant.job == "builder":
				has_builder = true
				break

		if not has_builder:
			var builder_entity: RefCounted = null
			for j in range(candidates.size()):
				var candidate_entity: RefCounted = candidates[j]
				if candidate_entity.job == "builder":
					builder_entity = candidate_entity
					break
			if builder_entity != null and migrants.size() > 0:
				migrants[migrants.size() - 1] = builder_entity

		if migrants.size() < GameConfig.MIGRATION_GROUP_SIZE_MIN:
			continue

		var best_site: Vector2i = Vector2i(-1, -1)
		var best_score: float = -1.0
		for j in range(20):
			var dx: int = _rng.randi_range(-GameConfig.MIGRATION_SEARCH_RADIUS_MAX, GameConfig.MIGRATION_SEARCH_RADIUS_MAX)
			var dy: int = _rng.randi_range(-GameConfig.MIGRATION_SEARCH_RADIUS_MAX, GameConfig.MIGRATION_SEARCH_RADIUS_MAX)
			var distance: int = absi(dx) + absi(dy)
			if distance < GameConfig.MIGRATION_SEARCH_RADIUS_MIN or distance > GameConfig.MIGRATION_SEARCH_RADIUS_MAX:
				continue

			var x: int = settlement.center_x + dx
			var y: int = settlement.center_y + dy

			if not _world_data.is_valid(x, y):
				continue
			if not _world_data.is_walkable(x, y):
				continue

			var far_enough: bool = true
			for k in range(settlements.size()):
				var other_settlement: RefCounted = settlements[k]
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

		if best_site.x < 0 or best_site.y < 0:
			continue

		var new_settlement: RefCounted = _settlement_manager.create_settlement(best_site.x, best_site.y, tick)
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
