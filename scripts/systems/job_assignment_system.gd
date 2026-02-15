extends "res://scripts/core/simulation_system.gd"

var _entity_manager: RefCounted
var _building_manager: RefCounted


func init(entity_manager: RefCounted, building_manager: RefCounted) -> void:
	system_name = "job_assignment"
	priority = 8
	tick_interval = GameConfig.JOB_ASSIGNMENT_TICK_INTERVAL
	_entity_manager = entity_manager
	_building_manager = building_manager


func execute_tick(tick: int) -> void:
	var entities: Array = _entity_manager.get_alive_entities()
	var alive_count: int = entities.size()
	if alive_count == 0:
		return

	# Count current job distribution
	var job_counts: Dictionary = {"gatherer": 0, "lumberjack": 0, "builder": 0, "miner": 0}
	var unassigned: Array = []
	for i in range(entities.size()):
		var entity = entities[i]
		if entity.job == "none":
			unassigned.append(entity)
		elif job_counts.has(entity.job):
			job_counts[entity.job] += 1

	# Determine target ratios based on population and economy
	var ratios: Dictionary = _get_dynamic_ratios(alive_count)

	# Assign most-needed job to each unassigned entity
	for i in range(unassigned.size()):
		var entity = unassigned[i]
		var best_job: String = _find_most_needed_job(ratios, job_counts, alive_count)
		entity.job = best_job
		job_counts[best_job] = job_counts.get(best_job, 0) + 1
		emit_event("job_assigned", {
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"job": best_job,
			"tick": tick,
		})

	# Dynamic rebalancing: reassign if ratios are very off
	if unassigned.is_empty() and alive_count >= 5:
		_rebalance_jobs(entities, ratios, job_counts, alive_count, tick)


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
	var worst_surplus: float = 0.0
	var worst_deficit_job: String = ""
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
