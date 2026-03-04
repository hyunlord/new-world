extends "res://scripts/core/simulation/simulation_system.gd"

## [Holland 1959, Hackman & Oldham 1976, Deci & Ryan 1985, Judge 2001]
## Computes job satisfaction from job profile fit. Handles job drift for dissatisfied agents.

var _entity_manager: RefCounted
var _settlement_manager: RefCounted
var _rng: RandomNumberGenerator
var _job_profiles: Dictionary = {}  # job_id -> profile dict
var _profile_runtime: Dictionary = {}  # job_id -> packed runtime profile
var _value_key_list: PackedStringArray = PackedStringArray()
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_JOB_SAT_METHOD: String = "body_job_satisfaction_score"
const _SIM_BRIDGE_JOB_SAT_BATCH_METHOD: String = "body_job_satisfaction_score_batch"
const _HEXACO_AXES: PackedStringArray = ["H", "E", "X", "A", "C", "O"]
var _bridge_checked: bool = false
var _sim_bridge: Object = null


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
	## [Layer 4.5] Load ALL job profiles from data/jobs/ — not just 4 hardcoded ones
	_job_profiles.clear()
	var dir: DirAccess = DirAccess.open("res://data/jobs")
	if dir == null:
		push_warning("[JobSatisfactionSystem] Cannot open data/jobs/")
		return
	dir.list_dir_begin()
	var fname: String = dir.get_next()
	while fname != "":
		if not dir.current_is_dir() and fname.ends_with(".json"):
			var job_id: String = fname.get_basename()
			var path: String = "res://data/jobs/" + fname
			var file = FileAccess.open(path, FileAccess.READ)
			if file != null:
				var json = JSON.new()
				if json.parse(file.get_as_text()) == OK:
					_job_profiles[job_id] = json.data
				file.close()
		fname = dir.get_next()
	dir.list_dir_end()
	_rebuild_profile_runtime_cache()


func _rebuild_profile_runtime_cache() -> void:
	_profile_runtime.clear()
	_value_key_list = PackedStringArray()

	var value_key_set: Dictionary = {}
	for profile_id_variant in _job_profiles:
		var profile_data: Dictionary = _job_profiles[profile_id_variant]
		var value_weights: Dictionary = profile_data.get("value_weights", {})
		for vkey in value_weights:
			value_key_set[str(vkey)] = true

	var value_keys: Array[String] = []
	for vkey_variant in value_key_set:
		value_keys.append(str(vkey_variant))
	value_keys.sort()
	for vkey in value_keys:
		_value_key_list.append(vkey)

	for profile_id_variant in _job_profiles:
		var profile_id: String = str(profile_id_variant)
		var profile_data: Dictionary = _job_profiles[profile_id_variant]
		var hex_ideal: Dictionary = profile_data.get("hexaco_ideal", {})
		var value_weights: Dictionary = profile_data.get("value_weights", {})
		var packed_hex_ideal: PackedFloat32Array = PackedFloat32Array()
		for axis in _HEXACO_AXES:
			packed_hex_ideal.append(float(hex_ideal.get(axis, 0.0)))
		var packed_value_weights: PackedFloat32Array = PackedFloat32Array()
		for vkey in _value_key_list:
			packed_value_weights.append(float(value_weights.get(vkey, 0.0)))

		_profile_runtime[profile_id] = {
			"personality_ideal": packed_hex_ideal,
			"value_weights": packed_value_weights,
			"primary_skill_name": StringName(str(profile_data.get("primary_skill", ""))),
			"autonomy_level": float(profile_data.get("autonomy_level", 0.5)),
			"prestige": float(profile_data.get("prestige", 0.3))
		}


func _build_entity_personality_actual(entity: RefCounted) -> PackedFloat32Array:
	var out: PackedFloat32Array = PackedFloat32Array()
	for axis in _HEXACO_AXES:
		out.append(float(entity.personality.axes.get(axis, 0.5)))
	return out


func _build_entity_value_actual(entity: RefCounted) -> PackedFloat32Array:
	var out: PackedFloat32Array = PackedFloat32Array()
	for vkey in _value_key_list:
		var v01: float = (float(entity.values.get(vkey, 0.0)) + 1.0) / 2.0
		out.append(v01)
	return out


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
	if node != null and node.has_method(_SIM_BRIDGE_JOB_SAT_METHOD):
		_sim_bridge = node
	return _sim_bridge

func _compute_satisfaction(entity: RefCounted, runtime_profile: Dictionary) -> float:
	if entity.personality == null:
		return 0.50

	var personality_actual: PackedFloat32Array = _build_entity_personality_actual(entity)
	var personality_ideal: PackedFloat32Array = runtime_profile.get(
		"personality_ideal", PackedFloat32Array()
	)
	var value_actual: PackedFloat32Array = _build_entity_value_actual(entity)
	var value_weight_packed: PackedFloat32Array = runtime_profile.get(
		"value_weights", PackedFloat32Array()
	)
	var primary_skill_name: StringName = runtime_profile.get("primary_skill_name", &"")
	var skill_level: int = int(entity.skill_levels.get(primary_skill_name, 0))
	var skill_fit: float = clampf(float(skill_level) / 10.0, 0.0, 1.0)

	var autonomy_level: float = float(runtime_profile.get("autonomy_level", 0.5))
	var prestige: float = float(runtime_profile.get("prestige", 0.3))
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_JOB_SAT_METHOD,
			personality_actual,
			personality_ideal,
			value_actual,
			value_weight_packed,
			skill_fit,
			float(entity.autonomy),
			float(entity.competence),
			float(entity.meaning),
			autonomy_level,
			prestige,
			float(GameConfig.JOB_SAT_W_SKILL_FIT),
			float(GameConfig.JOB_SAT_W_VALUE_FIT),
			float(GameConfig.JOB_SAT_W_PERSONALITY_FIT),
			float(GameConfig.JOB_SAT_W_NEED_FIT)
		)
		if rust_variant != null:
			return clampf(float(rust_variant), 0.0, 1.0)

	var personality_fit: float = 0.0
	var total_weight: float = 0.0
	for i in range(personality_ideal.size()):
		var ideal: float = personality_ideal[i]
		if is_zero_approx(ideal):
			continue
		var actual: float = personality_actual[i]
		if ideal > 0.0:
			personality_fit += absf(ideal) * actual
		else:
			personality_fit += absf(ideal) * (1.0 - actual)
		total_weight += absf(ideal)
	if total_weight > 0.0:
		personality_fit /= total_weight
	else:
		personality_fit = 0.5

	var value_fit: float = 0.0
	var val_total: float = 0.0
	var value_count: int = mini(value_weight_packed.size(), value_actual.size())
	for i in range(value_count):
		var w: float = value_weight_packed[i]
		var v01: float = value_actual[i]
		value_fit += w * v01
		val_total += w
	if val_total > 0.0:
		value_fit /= val_total
	else:
		value_fit = 0.5

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
	if entity.personality == null:
		return
	var drift_prob: float = GameConfig.JOB_SAT_DRIFT_BASE * (1.0 - entity.job_satisfaction)
	var drift_roll: float = _rng.randf()
	if drift_roll > drift_prob:
		return

	var best_job: String = entity.job
	var best_sat: float = entity.job_satisfaction
	if _profile_runtime.is_empty():
		return

	var candidate_ids: Array[String] = []
	var personality_ideals_flat: PackedFloat32Array = PackedFloat32Array()
	var value_weights_flat: PackedFloat32Array = PackedFloat32Array()
	var skill_fits: PackedFloat32Array = PackedFloat32Array()
	var autonomy_levels: PackedFloat32Array = PackedFloat32Array()
	var prestiges: PackedFloat32Array = PackedFloat32Array()

	var personality_actual: PackedFloat32Array = _build_entity_personality_actual(entity)
	var value_actual: PackedFloat32Array = _build_entity_value_actual(entity)

	for profile_id_variant in _profile_runtime:
		var profile_id: String = str(profile_id_variant)
		if profile_id == entity.job:
			continue
		var runtime_profile: Dictionary = _profile_runtime[profile_id]
		candidate_ids.append(profile_id)
		personality_ideals_flat.append_array(
			runtime_profile.get("personality_ideal", PackedFloat32Array())
		)
		value_weights_flat.append_array(
			runtime_profile.get("value_weights", PackedFloat32Array())
		)
		var primary_skill_name: StringName = runtime_profile.get("primary_skill_name", &"")
		var skill_level: int = int(entity.skill_levels.get(primary_skill_name, 0))
		skill_fits.append(clampf(float(skill_level) / 10.0, 0.0, 1.0))
		autonomy_levels.append(float(runtime_profile.get("autonomy_level", 0.5)))
		prestiges.append(float(runtime_profile.get("prestige", 0.3)))

	if candidate_ids.is_empty():
		return

	var scores: PackedFloat32Array = PackedFloat32Array()
	var bridge: Object = _get_sim_bridge()
	if bridge != null and bridge.has_method(_SIM_BRIDGE_JOB_SAT_BATCH_METHOD):
		var rust_scores_variant: Variant = bridge.call(
			_SIM_BRIDGE_JOB_SAT_BATCH_METHOD,
			personality_actual,
			personality_ideals_flat,
			value_actual,
			value_weights_flat,
			skill_fits,
			float(entity.autonomy),
			float(entity.competence),
			float(entity.meaning),
			autonomy_levels,
			prestiges,
			float(GameConfig.JOB_SAT_W_SKILL_FIT),
			float(GameConfig.JOB_SAT_W_VALUE_FIT),
			float(GameConfig.JOB_SAT_W_PERSONALITY_FIT),
			float(GameConfig.JOB_SAT_W_NEED_FIT)
		)
		if rust_scores_variant is PackedFloat32Array:
			scores = rust_scores_variant

	if scores.size() != candidate_ids.size():
		scores.resize(candidate_ids.size())
		for i in range(candidate_ids.size()):
			var fallback_profile: Dictionary = _profile_runtime.get(candidate_ids[i], {})
			scores[i] = _compute_satisfaction(entity, fallback_profile)

	for i in range(candidate_ids.size()):
		var sat: float = float(scores[i])
		if sat > best_sat + 0.10:
			best_sat = sat
			best_job = candidate_ids[i]

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
