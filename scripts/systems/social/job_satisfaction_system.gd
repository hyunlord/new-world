extends "res://scripts/core/simulation/simulation_system.gd"

## [Holland 1959, Hackman & Oldham 1976, Deci & Ryan 1985, Judge 2001]
## Computes job satisfaction from job profile fit. Handles job drift for dissatisfied agents.

var _entity_manager: RefCounted
var _settlement_manager: RefCounted
var _rng: RandomNumberGenerator
var _job_profiles: Dictionary = {}  # job_id -> profile dict


func _init() -> void:
	system_name = "job_satisfaction"
	priority = 40
	tick_interval = GameConfig.JOB_SAT_TICK_INTERVAL


func init(entity_manager: RefCounted, settlement_manager: RefCounted,
		rng: RandomNumberGenerator) -> void:
	_entity_manager = entity_manager
	_settlement_manager = settlement_manager
	_rng = rng
	if _rng == null:
		_rng = RandomNumberGenerator.new()
		_rng.randomize()
	_load_job_profiles()


func _load_job_profiles() -> void:
	var job_ids: Array = ["gatherer", "lumberjack", "builder", "miner"]
	for job_id in job_ids:
		var path: String = "res://data/jobs/" + job_id + ".json"
		var file = FileAccess.open(path, FileAccess.READ)
		if file == null:
			continue
		var json = JSON.new()
		if json.parse(file.get_as_text()) == OK:
			_job_profiles[job_id] = json.data
		file.close()


func execute_tick(tick: int) -> void:
	if _entity_manager == null:
		return
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity = alive[i]
		if entity.age_stage == "child" or entity.age_stage == "infant":
			entity.set_meta("job_sat_work_speed_mult", 1.0)
			continue
		if entity.job == "none" or entity.job == "":
			entity.job_satisfaction = 0.50
			_apply_work_speed_modifier_flag(entity)
			continue
		var profile = _job_profiles.get(entity.job, {})
		if profile.is_empty():
			entity.job_satisfaction = 0.50
			_apply_work_speed_modifier_flag(entity)
			continue
		entity.job_satisfaction = _compute_satisfaction(entity, profile)
		_apply_work_speed_modifier_flag(entity)
		_check_job_drift(entity, tick)


func _compute_satisfaction(entity: RefCounted, job_profile: Dictionary) -> float:
	if entity.personality == null:
		return 0.50

	# 1. Personality fit
	var hex_ideal: Dictionary = job_profile.get("hexaco_ideal", {})
	var personality_fit: float = 0.0
	var total_weight: float = 0.0
	for axis in ["H", "E", "X", "A", "C", "O"]:
		var ideal: float = float(hex_ideal.get(axis, 0.0))
		if is_zero_approx(ideal):
			continue
		var actual: float = float(entity.personality.axes.get(axis, 0.5))
		if ideal > 0.0:
			personality_fit += absf(ideal) * actual
		else:
			personality_fit += absf(ideal) * (1.0 - actual)
		total_weight += absf(ideal)
	if total_weight > 0.0:
		personality_fit /= total_weight
	else:
		personality_fit = 0.5

	# 2. Value fit
	var val_weights: Dictionary = job_profile.get("value_weights", {})
	var value_fit: float = 0.0
	var val_total: float = 0.0
	for vkey in val_weights:
		var w: float = float(val_weights[vkey])
		var v01: float = (float(entity.values.get(vkey, 0.0)) + 1.0) / 2.0
		value_fit += w * v01
		val_total += w
	if val_total > 0.0:
		value_fit /= val_total
	else:
		value_fit = 0.5

	# 3. Skill fit
	var primary_skill: String = str(job_profile.get("primary_skill", ""))
	var skill_level: int = int(entity.skill_levels.get(StringName(primary_skill), 0))
	var skill_fit: float = clampf(float(skill_level) / 10.0, 0.0, 1.0)

	# 4. Need fit
	var autonomy_level: float = float(job_profile.get("autonomy_level", 0.5))
	var prestige: float = float(job_profile.get("prestige", 0.3))
	var need_fit: float = (
		entity.autonomy * autonomy_level * 0.35
		+ entity.competence * skill_fit * 0.35
		+ entity.meaning * prestige * 0.30
	)

	return clampf(
		skill_fit * GameConfig.JOB_SAT_W_SKILL_FIT
		+ value_fit * GameConfig.JOB_SAT_W_VALUE_FIT
		+ personality_fit * GameConfig.JOB_SAT_W_PERSONALITY_FIT
		+ need_fit * GameConfig.JOB_SAT_W_NEED_FIT,
		0.0, 1.0)


func _apply_work_speed_modifier_flag(entity: RefCounted) -> void:
	var sat: float = float(entity.job_satisfaction)
	var speed_mult: float = 1.0
	if sat >= GameConfig.JOB_SAT_HIGH_THRESHOLD:
		speed_mult = GameConfig.JOB_SAT_HIGH_SPEED_MULT
	elif sat < GameConfig.JOB_SAT_CRITICAL_THRESHOLD:
		speed_mult = GameConfig.JOB_SAT_CRITICAL_SPEED_MULT
	elif sat < GameConfig.JOB_SAT_LOW_THRESHOLD:
		speed_mult = GameConfig.JOB_SAT_LOW_SPEED_MULT
	entity.set_meta("job_sat_work_speed_mult", speed_mult)


func _check_job_drift(entity: RefCounted, tick: int) -> void:
	if entity.job_satisfaction >= GameConfig.JOB_SAT_LOW_THRESHOLD:
		return
	var drift_prob: float = GameConfig.JOB_SAT_DRIFT_BASE * (1.0 - entity.job_satisfaction)
	if _rng.randf() > drift_prob:
		return

	var best_job: String = entity.job
	var best_sat: float = entity.job_satisfaction
	for profile_id in _job_profiles:
		if profile_id == entity.job:
			continue
		var sat: float = _compute_satisfaction(entity, _job_profiles[profile_id])
		if sat > best_sat + 0.10:
			best_job = str(profile_id)
			best_sat = sat

	if best_job != entity.job:
		var old_job: String = entity.job
		var old_satisfaction: float = entity.job_satisfaction
		entity.job = best_job
		entity.job_satisfaction = best_sat
		_apply_work_speed_modifier_flag(entity)
		SimulationBus.emit_event("job_drift", {
			"entity_id": entity.id, "entity_name": entity.entity_name,
			"old_job": old_job, "new_job": best_job,
			"old_satisfaction": old_satisfaction,
			"new_satisfaction": best_sat, "tick": tick,
		})
