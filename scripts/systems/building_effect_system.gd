extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _sim_engine: RefCounted


func init(entity_manager: RefCounted, building_manager: RefCounted, sim_engine: RefCounted) -> void:
	system_name = "building_effect"
	priority = 15
	tick_interval = GameConfig.BUILDING_EFFECT_TICK_INTERVAL
	_entity_manager = entity_manager
	_building_manager = building_manager
	_sim_engine = sim_engine


func execute_tick(tick: int) -> void:
	var buildings: Array = _building_manager.get_all_buildings()
	for i in range(buildings.size()):
		var building = buildings[i]
		if not building.is_built:
			continue
		match building.building_type:
			"campfire":
				_apply_campfire(building)
			"shelter":
				_apply_shelter(building)


func _apply_campfire(building: RefCounted) -> void:
	var time_data: Dictionary = _sim_engine.get_game_time()
	var hour: int = time_data.get("hour", 12)
	var is_night: bool = hour >= 20 or hour < 6
	var social_boost: float = 0.02 if is_night else 0.01
	var radius: int = GameConfig.BUILDING_TYPES["campfire"]["radius"]
	var nearby: Array = _entity_manager.get_entities_near(
		Vector2i(building.tile_x, building.tile_y), radius
	)
	for j in range(nearby.size()):
		var entity = nearby[j]
		entity.social = minf(entity.social + social_boost, 1.0)


func _apply_shelter(building: RefCounted) -> void:
	var nearby: Array = _entity_manager.get_entities_near(
		Vector2i(building.tile_x, building.tile_y), 0
	)
	for j in range(nearby.size()):
		var entity = nearby[j]
		entity.energy = minf(entity.energy + 0.01, 1.0)
