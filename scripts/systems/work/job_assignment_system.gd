extends "res://scripts/core/simulation/simulation_system.gd"

const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_BEST_JOB_METHOD: String = "body_job_assignment_best_job_code"
const _SIM_BRIDGE_REBALANCE_METHOD: String = "body_job_assignment_rebalance_codes"
const _JOB_ORDER: Array = ["gatherer", "lumberjack", "builder", "miner"]

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func init(entity_manager: RefCounted, building_manager: RefCounted) -> void:
	system_name = "job_assignment"
	priority = 8
	tick_interval = GameConfig.JOB_ASSIGNMENT_TICK_INTERVAL
	_entity_manager = entity_manager
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
	if node != null \
	and node.has_method(_SIM_BRIDGE_BEST_JOB_METHOD) \
	and node.has_method(_SIM_BRIDGE_REBALANCE_METHOD):
		_sim_bridge = node
	return _sim_bridge

func _get_dynamic_ratios(alive_count: int) -> Dictionary:
	# Very small pop: survival mode
	if alive_count < 10:
		return {"gatherer": 0.8, "lumberjack": 0.1, "builder": 0.1, "miner": 0.0}

	# Check food situation for dynamic adjustment
	var food_crisis: bool = false
	if _building_manager != null:
		var total_food: float = _get_total_stockpile_food()
		if total_food < float(alive_count) * 1.5:
			food_crisis = true

	if food_crisis:
		return {"gatherer": 0.6, "lumberjack": 0.2, "builder": 0.1, "miner": 0.1}

	return GameConfig.JOB_RATIOS.duplicate()


func _find_most_needed_job(ratios: Dictionary, job_counts: Dictionary, alive_count: int) -> String:
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var ratio_values: PackedFloat32Array = PackedFloat32Array()
		var count_values: PackedInt32Array = PackedInt32Array()
		for i in range(_JOB_ORDER.size()):
			var job_name: String = String(_JOB_ORDER[i])
			ratio_values.append(float(ratios.get(job_name, 0.0)))
			count_values.append(int(job_counts.get(job_name, 0)))
		var idx_variant: Variant = bridge.call(
			_SIM_BRIDGE_BEST_JOB_METHOD,
			ratio_values,
			count_values,
			alive_count,
		)
		if idx_variant != null:
			var idx: int = int(idx_variant)
			if idx >= 0 and idx < _JOB_ORDER.size():
				return String(_JOB_ORDER[idx])

	var best_job: String = "gatherer"
	var best_deficit: float = -999.0
	var ratio_keys: Array = ratios.keys()
	for j in range(ratio_keys.size()):
		var job_name: String = ratio_keys[j]
		var target: float = ratios[job_name] * float(alive_count)
		var current: float = float(job_counts.get(job_name, 0))
		var deficit: float = target - current
		if deficit > best_deficit:
			best_deficit = deficit
			best_job = job_name
	return best_job


func _rebalance_jobs(entities: Array, ratios: Dictionary, job_counts: Dictionary, alive_count: int, tick: int) -> void:
	# Only rebalance one entity per tick to avoid chaos
	var worst_surplus_job: String = ""
	var worst_deficit_job: String = ""
	var used_rust: bool = false
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var ratio_values: PackedFloat32Array = PackedFloat32Array()
		var count_values: PackedInt32Array = PackedInt32Array()
		for i in range(_JOB_ORDER.size()):
			var job_name: String = String(_JOB_ORDER[i])
			ratio_values.append(float(ratios.get(job_name, 0.0)))
			count_values.append(int(job_counts.get(job_name, 0)))
		var pair_variant: Variant = bridge.call(
			_SIM_BRIDGE_REBALANCE_METHOD,
			ratio_values,
			count_values,
			alive_count,
			1.5,
		)
		if pair_variant is PackedInt32Array:
			var pair: PackedInt32Array = pair_variant
			if pair.size() >= 2:
				var surplus_idx: int = int(pair[0])
				var deficit_idx: int = int(pair[1])
				if surplus_idx >= 0 and surplus_idx < _JOB_ORDER.size() \
				and deficit_idx >= 0 and deficit_idx < _JOB_ORDER.size():
					worst_surplus_job = String(_JOB_ORDER[surplus_idx])
					worst_deficit_job = String(_JOB_ORDER[deficit_idx])
					used_rust = true

	if not used_rust:
		var worst_surplus: float = 0.0
		var worst_deficit: float = 0.0
		var ratio_keys: Array = ratios.keys()
		for i in range(ratio_keys.size()):
			var job_name: String = ratio_keys[i]
			var target: float = ratios[job_name] * float(alive_count)
			var current: float = float(job_counts.get(job_name, 0))
			var surplus: float = current - target
			if surplus > worst_surplus:
				worst_surplus = surplus
				worst_surplus_job = job_name
			var deficit: float = target - current
			if deficit > worst_deficit:
				worst_deficit = deficit
				worst_deficit_job = job_name

		# Only rebalance if surplus and deficit are both significant (> 1.5)
		if worst_surplus < 1.5 or worst_deficit < 1.5:
			return
		if worst_surplus_job == "" or worst_deficit_job == "":
			return

	# Find an entity to reassign (prefer idle ones)
	for i in range(entities.size()):
		var entity = entities[i]
		if entity.job == worst_surplus_job and entity.current_action == "idle":
			entity.job = worst_deficit_job
			emit_event("job_reassigned", {
				"entity_id": entity.id,
				"entity_name": entity.entity_name,
				"from_job": worst_surplus_job,
				"to_job": worst_deficit_job,
				"tick": tick,
			})
			return


func _get_total_stockpile_food() -> float:
	var total: float = 0.0
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	for i in range(stockpiles.size()):
		var sp = stockpiles[i]
		if sp.is_built:
			total += sp.storage.get("food", 0.0)
	return total
