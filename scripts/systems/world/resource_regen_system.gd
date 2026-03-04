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
