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

	if unassigned.is_empty():
		return

	# Adjusted ratios for small populations
	var ratios: Dictionary = {}
	if alive_count < 10:
		ratios = {"gatherer": 0.7, "lumberjack": 0.2, "builder": 0.1, "miner": 0.0}
	else:
		ratios = GameConfig.JOB_RATIOS.duplicate()

	# Assign most-needed job to each unassigned entity
	for i in range(unassigned.size()):
		var entity = unassigned[i]
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

		entity.job = best_job
		job_counts[best_job] = job_counts.get(best_job, 0) + 1
		emit_event("job_assigned", {
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"job": best_job,
			"tick": tick,
		})
