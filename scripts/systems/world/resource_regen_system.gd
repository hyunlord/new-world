extends "res://scripts/core/simulation/simulation_system.gd"

var _resource_map: RefCounted
var _world_data: RefCounted
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_RESOURCE_REGEN_METHOD: String = "body_resource_regen_next"
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func init(resource_map: RefCounted, world_data: RefCounted) -> void:
	system_name = "resource_regen"
	priority = 5
	tick_interval = GameConfig.RESOURCE_REGEN_TICK_INTERVAL
	_resource_map = resource_map
	_world_data = world_data


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
	if node != null and node.has_method(_SIM_BRIDGE_RESOURCE_REGEN_METHOD):
		_sim_bridge = node
	return _sim_bridge


func execute_tick(_tick: int) -> void:
	var w: int = _world_data.width
	var h: int = _world_data.height
	var bridge: Object = _get_sim_bridge()
	for y in range(h):
		for x in range(w):
			var biome: int = _world_data.get_biome(x, y)
			# Food regen (grassland, forest, beach)
			var food_max: float = _resource_map.get_max_for_biome(biome, GameConfig.ResourceType.FOOD)
			if food_max > 0.0:
				var current_food: float = _resource_map.get_food(x, y)
				if current_food < food_max:
					var next_food: float = minf(current_food + GameConfig.FOOD_REGEN_RATE, food_max)
					if bridge != null:
						var food_variant: Variant = bridge.call(
							_SIM_BRIDGE_RESOURCE_REGEN_METHOD,
							current_food,
							food_max,
							float(GameConfig.FOOD_REGEN_RATE),
						)
						if food_variant != null:
							next_food = float(food_variant)
					_resource_map.set_food(x, y, next_food)
			# Wood regen (forest, dense forest)
			var wood_max: float = _resource_map.get_max_for_biome(biome, GameConfig.ResourceType.WOOD)
			if wood_max > 0.0:
				var current_wood: float = _resource_map.get_wood(x, y)
				if current_wood < wood_max:
					var next_wood: float = minf(current_wood + GameConfig.WOOD_REGEN_RATE, wood_max)
					if bridge != null:
						var wood_variant: Variant = bridge.call(
							_SIM_BRIDGE_RESOURCE_REGEN_METHOD,
							current_wood,
							wood_max,
							float(GameConfig.WOOD_REGEN_RATE),
						)
						if wood_variant != null:
							next_wood = float(wood_variant)
					_resource_map.set_wood(x, y, next_wood)
			# Stone does NOT regen
