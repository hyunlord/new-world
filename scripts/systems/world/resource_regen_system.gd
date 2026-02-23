extends "res://scripts/core/simulation_system.gd"

var _resource_map: RefCounted
var _world_data: RefCounted


func init(resource_map: RefCounted, world_data: RefCounted) -> void:
	system_name = "resource_regen"
	priority = 5
	tick_interval = GameConfig.RESOURCE_REGEN_TICK_INTERVAL
	_resource_map = resource_map
	_world_data = world_data


func execute_tick(tick: int) -> void:
	var w: int = _world_data.width
	var h: int = _world_data.height
	for y in range(h):
		for x in range(w):
			var biome: int = _world_data.get_biome(x, y)
			# Food regen (grassland, forest, beach)
			var food_max: float = _resource_map.get_max_for_biome(biome, GameConfig.ResourceType.FOOD)
			if food_max > 0.0:
				var current_food: float = _resource_map.get_food(x, y)
				if current_food < food_max:
					_resource_map.set_food(x, y, minf(current_food + GameConfig.FOOD_REGEN_RATE, food_max))
			# Wood regen (forest, dense forest)
			var wood_max: float = _resource_map.get_max_for_biome(biome, GameConfig.ResourceType.WOOD)
			if wood_max > 0.0:
				var current_wood: float = _resource_map.get_wood(x, y)
				if current_wood < wood_max:
					_resource_map.set_wood(x, y, minf(current_wood + GameConfig.WOOD_REGEN_RATE, wood_max))
			# Stone does NOT regen
