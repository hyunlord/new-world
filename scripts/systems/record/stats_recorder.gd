extends "res://scripts/core/simulation/simulation_system.gd"

const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_RESOURCE_DELTAS_METHOD: String = "body_stats_resource_deltas_per_100"

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var history: Array = []
const MAX_HISTORY: int = 200
var peak_pop: int = 0
var total_births: int = 0
var total_deaths: int = 0
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func _init() -> void:
	system_name = "stats_recorder"
	priority = 90
	tick_interval = 200


## Initializes the recorder with entity, building, and optional settlement managers.
func init(entity_manager: RefCounted, building_manager: RefCounted, settlement_manager: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
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
	if node != null and node.has_method(_SIM_BRIDGE_RESOURCE_DELTAS_METHOD):
		_sim_bridge = node
	return _sim_bridge

## Get resource delta per 100 ticks (rate of change)
func get_resource_deltas() -> Dictionary:
	if history.size() < 3:
		return {"food": 0.0, "wood": 0.0, "stone": 0.0}
	var latest: Dictionary = history[history.size() - 1]
	var older: Dictionary = history[maxi(0, history.size() - 3)]
	var tick_diff: float = float(latest.tick - older.tick)
	if tick_diff <= 0.0:
		return {"food": 0.0, "wood": 0.0, "stone": 0.0}

	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var out_variant: Variant = bridge.call(
			_SIM_BRIDGE_RESOURCE_DELTAS_METHOD,
			float(latest.food),
			float(latest.wood),
			float(latest.stone),
			float(older.food),
			float(older.wood),
			float(older.stone),
			tick_diff,
		)
		if out_variant is PackedFloat32Array:
			var out: PackedFloat32Array = out_variant
			if out.size() >= 3:
				return {
					"food": float(out[0]),
					"wood": float(out[1]),
					"stone": float(out[2]),
				}

	var scale: float = 100.0 / tick_diff
	return {
		"food": (latest.food - older.food) * scale,
		"wood": (latest.wood - older.wood) * scale,
		"stone": (latest.stone - older.stone) * scale,
	}


## Get per-settlement stats
func get_settlement_stats() -> Array:
	if _settlement_manager == null:
		return []
	var result: Array = []
	var active: Array = _settlement_manager.get_active_settlements()
	for i in range(active.size()):
		var s: RefCounted = active[i]
		var pop: int = _settlement_manager.get_settlement_population(s.id)
		var bld_count: int = s.building_ids.size()
		var food: float = 0.0
		var wood: float = 0.0
		var stone: float = 0.0
		if _building_manager != null:
			var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
			for j in range(stockpiles.size()):
				var sp = stockpiles[j]
				if sp.settlement_id == s.id and sp.is_built:
					food += sp.storage.get("food", 0.0)
					wood += sp.storage.get("wood", 0.0)
					stone += sp.storage.get("stone", 0.0)
		result.append({
			"id": s.id, "pop": pop, "buildings": bld_count,
			"food": food, "wood": wood, "stone": stone,
		})
	return result
