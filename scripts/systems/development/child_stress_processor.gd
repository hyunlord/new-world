extends "res://scripts/core/simulation/simulation_system.gd"

# NO class_name — headless compatibility

var _entity_manager
var _stages: Dictionary = {}
var _stage_age_cutoffs: PackedFloat32Array = PackedFloat32Array([2.0, 5.0, 12.0, 18.0])


func _init() -> void:
	system_name = "child_stress"
	priority = 32  # before mental_break(35)
	tick_interval = 2


func init(entity_manager) -> void:
	_entity_manager = entity_manager
	_load_stages()


func _load_stages() -> void:
	var path: String = "res://data/developmental_stages.json"
	if not FileAccess.file_exists(path):
		push_warning("[ChildStressProcessor] developmental_stages.json not found")
		return
	var file = FileAccess.open(path, FileAccess.READ)
	if file == null:
		push_warning("[ChildStressProcessor] Cannot open developmental_stages.json")
		return
	var text: String = file.get_as_text()
	file.close()
	var json = JSON.new()
	var err: int = json.parse(text)
	if err != OK:
		push_error("[ChildStressProcessor] developmental_stages.json parse error: " + json.get_error_message())
		return
	var data = json.get_data()
	if data is Dictionary:
		_stages = data
	_refresh_stage_age_cutoffs()


func _refresh_stage_age_cutoffs() -> void:
	if _stage_age_cutoffs.size() != 4:
		_stage_age_cutoffs.resize(4)
	_stage_age_cutoffs[0] = 2.0
	_stage_age_cutoffs[1] = 5.0
	_stage_age_cutoffs[2] = 12.0
	_stage_age_cutoffs[3] = 18.0
	var stage_names: Array[String] = ["infant", "toddler", "child", "teen"]
	for i in range(stage_names.size()):
		var stage_name: String = stage_names[i]
		var stage_data_variant: Variant = _stages.get(stage_name, {})
		if not (stage_data_variant is Dictionary):
			continue
		var stage_data: Dictionary = stage_data_variant
		var age_range_variant: Variant = stage_data.get("age_range", [])
		if age_range_variant is Array:
			var age_range: Array = age_range_variant
			if age_range.size() >= 2:
				_stage_age_cutoffs[i] = float(age_range[1])


func get_current_stage(age_ticks: int) -> String:
	var infant_max: float = float(_stage_age_cutoffs[0]) if _stage_age_cutoffs.size() > 0 else 2.0
	var toddler_max: float = float(_stage_age_cutoffs[1]) if _stage_age_cutoffs.size() > 1 else 5.0
	var child_max: float = float(_stage_age_cutoffs[2]) if _stage_age_cutoffs.size() > 2 else 12.0
	var teen_max: float = float(_stage_age_cutoffs[3]) if _stage_age_cutoffs.size() > 3 else 18.0
	var stage_code_variant: Variant = SimBridge.body_child_stage_code_from_age_ticks(
		age_ticks,
		infant_max,
		toddler_max,
		child_max,
		teen_max
	)
	if stage_code_variant != null:
		return _stage_name_from_code(int(stage_code_variant))

	var years: float = float(age_ticks) / 8760.0
	if years < infant_max:
		return "infant"
	if years < toddler_max:
		return "toddler"
	if years < child_max:
		return "child"
	if years < teen_max:
		return "teen"
	return "adult"


func _stage_name_from_code(stage_code: int) -> String:
	match stage_code:
		0:
			return "infant"
		1:
			return "toddler"
		2:
			return "child"
		3:
			return "teen"
	return "adult"


## [Lazarus & Folkman, 1984 - Transactional Model of Stress]
## Children use immature appraisal — cognitive_appraisal_enabled gates secondary appraisal.
## [Gunnar & Quevedo, 2007 - SHRP] Infant HPA axis suppressed by default.
## Design Note: Stage-specific pipeline: SHRP check → social buffer → classify → apply.
func process_stressor(entity, stressor: Dictionary, tick: int) -> void:
	if entity == null or entity.emotion_data == null:
		return

	var childhood_data = entity.get_meta("childhood_data", {})
	var stage = childhood_data.get("current_stage", "")
	if stage == "":
		stage = get_current_stage(int(entity.age))
	var stage_data = _stages.get(stage, {})

	var stressor_type: String = str(stressor.get("type", "threat"))
	var intensity: float = float(stressor.get("intensity", 0.0))

	if bool(stage_data.get("shrp_active", false)) and stressor_type == "threat":
		var shrp_stressor = stressor.duplicate()
		shrp_stressor["tick"] = tick
		intensity = _apply_shrp(shrp_stressor, stage_data, entity)

	if stressor_type == "deprivation":
		_accumulate_deprivation_damage(entity, stressor, stage_data)
		emit_event("child_stress_processed", {
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"stressor_type": stressor_type,
			"stage": stage,
			"stress_type": "deprivation",
			"intensity": intensity,
			"tick": tick,
		})
		return

	var attachment_quality: float = float(stressor.get("attachment_quality", childhood_data.get("attachment_quality", 0.5)))
	var caregiver_present: bool = bool(stressor.get("caregiver_present", childhood_data.get("caregiver_present", childhood_data.get("caregiver_support_active", false))))

	intensity = _apply_social_buffer(intensity, stage, attachment_quality, caregiver_present)

	var stress_type: String = _classify_stress_type(intensity, caregiver_present, attachment_quality, entity, tick)
	var spike_mult: float = float(stage_data.get("cortisol_spike_mult", 1.0))
	var vulnerability_mult: float = float(stage_data.get("vulnerability_mult", 1.0))
	var applied_rust: bool = false
	var break_threshold_mult: float = float(stage_data.get("break_threshold_mult", 1.0))
	var apply_variant: Variant = SimBridge.body_child_stress_apply_step(
		float(entity.emotion_data.resilience),
		float(entity.emotion_data.reserve),
		float(entity.emotion_data.stress),
		float(entity.emotion_data.allostatic),
		intensity,
		spike_mult,
		vulnerability_mult,
		break_threshold_mult,
		_stress_type_to_code(stress_type)
	)
	if apply_variant is PackedFloat32Array:
		var packed_apply: PackedFloat32Array = apply_variant
		if packed_apply.size() >= 5:
			entity.emotion_data.resilience = float(packed_apply[0])
			entity.emotion_data.reserve = float(packed_apply[1])
			entity.emotion_data.stress = float(packed_apply[2])
			entity.emotion_data.allostatic = float(packed_apply[3])
			var dd_delta: float = float(packed_apply[4])
			if dd_delta > 0.0:
				var developmental_damage: float = float(entity.emotion_data.get_meta("developmental_damage", 0.0))
				entity.emotion_data.set_meta("developmental_damage", developmental_damage + dd_delta)
			applied_rust = true
	if not applied_rust:
		match stress_type:
			"positive":
				entity.emotion_data.resilience = clampf(entity.emotion_data.resilience + 0.01 * intensity, 0.0, 1.0)
				entity.emotion_data.reserve = clampf(entity.emotion_data.reserve + 0.5 * intensity, 0.0, 100.0)
			"tolerable":
				var gas_cost: float = intensity * spike_mult * 6.0
				entity.emotion_data.reserve = clampf(entity.emotion_data.reserve - gas_cost, 0.0, 100.0)
				entity.emotion_data.stress = clampf(entity.emotion_data.stress + intensity * spike_mult * 8.0, 0.0, 2000.0)
			"toxic":
				entity.emotion_data.stress = clampf(
					entity.emotion_data.stress + intensity * spike_mult * vulnerability_mult * 16.0,
					0.0,
					2000.0
				)
				entity.emotion_data.allostatic = clampf(
					entity.emotion_data.allostatic + intensity * vulnerability_mult * 1.5,
					0.0,
					100.0
				)
				var developmental_damage = float(entity.emotion_data.get_meta("developmental_damage", 0.0))
				developmental_damage += intensity * break_threshold_mult * 0.02
				entity.emotion_data.set_meta("developmental_damage", developmental_damage)

	var stress_type_label: String = Locale.ltr("STRESS_TYPE_" + stress_type.to_upper())
	emit_event("child_stress_processed", {
		"entity_id": entity.id,
		"entity_name": entity.entity_name,
		"stressor_type": stressor_type,
		"stage": stage,
		"stress_type": stress_type,
		"stress_type_label": stress_type_label,
		"intensity": intensity,
		"tick": tick,
	})


## [Gunnar & Quevedo, 2007 - Stress Hypo-Responsive Period]
## Infant HPA axis intentionally suppressed — protects developing brain from glucocorticoid overexposure.
## Design Note (GPT bug fix): SHRP applies to threat type ONLY.
##   deprivation bypasses SHRP entirely (handled via developmental_damage channel).
func _apply_shrp(stressor: Dictionary, stage_data: Dictionary, entity = null) -> float:
	var intensity: float = float(stressor.get("intensity", 0.0))
	var shrp_active: bool = bool(stage_data.get("shrp_active", false))
	if not shrp_active:
		return intensity
	var threshold: float = float(stage_data.get("shrp_override_threshold", 0.85))
	var vulnerability_mult: float = float(stage_data.get("vulnerability_mult", 1.0))
	var shrp_variant: Variant = SimBridge.body_child_shrp_step(
		intensity,
		shrp_active,
		threshold,
		vulnerability_mult
	)
	if shrp_variant is PackedFloat32Array:
		var packed_shrp: PackedFloat32Array = shrp_variant
		if packed_shrp.size() >= 2:
			if int(round(float(packed_shrp[1]))) != 0:
				_handle_shrp_override(entity, stressor)
			return float(packed_shrp[0])
	if intensity < threshold:
		return 0.0
	_handle_shrp_override(entity, stressor)
	return intensity * vulnerability_mult


func _handle_shrp_override(entity, stressor: Dictionary) -> void:
	if entity == null:
		return
	if entity.emotion_data != null:
		entity.emotion_data.set_meta("shrp_breached", true)

	var chronicle = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
	if chronicle != null:
		var tick: int = int(stressor.get("tick", -1))
		var params: Dictionary = {"name": entity.entity_name}
		var desc: String = Locale.trf1("SHRP_OVERRIDE", "name", params.get("name", ""))
		chronicle.log_event("child_stress", entity.id, desc, 4, [], tick, {
			"key": "SHRP_OVERRIDE",
			"params": params,
		})


func _accumulate_deprivation_damage(entity, stressor: Dictionary, _stage_data: Dictionary) -> void:
	if entity == null or entity.emotion_data == null:
		return
	var rate: float = float(stressor.get("developmental_damage_rate", 0.01))
	var current: float = float(entity.emotion_data.get_meta("developmental_damage", 0.0))
	var next_variant: Variant = SimBridge.body_child_deprivation_damage_step(current, rate)
	if next_variant != null:
		entity.emotion_data.set_meta("developmental_damage", float(next_variant))
		return
	entity.emotion_data.set_meta("developmental_damage", current + rate)


## [Shonkoff et al., 2012 - Toxic Stress 3-category model]
## Key: caregiver presence determines toxic vs tolerable, NOT just intensity.
func _classify_stress_type(intensity: float, attachment_present: bool, quality: float,
		entity = null, tick: int = -1) -> String:
	var stress_type: String = ""
	var type_variant: Variant = SimBridge.body_child_stress_type_code(
		intensity,
		attachment_present,
		quality
	)
	if type_variant != null:
		stress_type = _stress_type_from_code(int(type_variant))
	else:
		if intensity < 0.30:
			stress_type = "positive"
		elif attachment_present and quality > 0.50:
			stress_type = "tolerable"
		else:
			stress_type = "toxic"

	if stress_type == "toxic" and entity != null:
		var chronicle = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
		if chronicle != null:
			var desc: String = Locale.ltr("TOXIC_STRESS_ONSET")
			chronicle.log_event("child_stress", entity.id, desc, 3, [], tick, {
				"key": "TOXIC_STRESS_ONSET",
				"params": {"name": entity.entity_name},
			})
	return stress_type


func _stress_type_from_code(stress_type_code: int) -> String:
	match stress_type_code:
		0:
			return "positive"
		1:
			return "tolerable"
	return "toxic"


func _stress_type_to_code(stress_type: String) -> int:
	match stress_type:
		"positive":
			return 0
		"tolerable":
			return 1
	return 2


## [Hostinar, Sullivan & Gunnar, 2014 - Social Buffering]
func _apply_social_buffer(intensity: float, stage: String,
		attachment_quality: float, caregiver_present: bool) -> float:
	if not caregiver_present:
		return intensity
	var stage_data = _stages.get(stage, {})
	var buffer_power: float = float(stage_data.get("buffer_power", 0.0))
	var buffered_variant: Variant = SimBridge.body_child_social_buffered_intensity(
		intensity,
		attachment_quality,
		caregiver_present,
		buffer_power
	)
	if buffered_variant != null:
		return float(buffered_variant)
	var social_buffer: float = attachment_quality * buffer_power
	return intensity * (1.0 - social_buffer)


func _attachment_type_to_code(attachment_type: String) -> int:
	match attachment_type:
		"secure":
			return 0
		"anxious":
			return 1
		"avoidant":
			return 2
		"disorganized":
			return 3
	return -1


## [Conradt et al., 2013 - Co-regulation / Parent→Child stress transfer]
func _calculate_parent_stress_transfer(parent_stress: float, parent_dependency: float,
		attachment_type: String, caregiver_support_active: bool,
		stage: String, contagion_input: float) -> float:
	var coefficient: float = 0.25
	match attachment_type:
		"secure":
			coefficient = 0.15
		"anxious":
			coefficient = 0.35
		"avoidant":
			coefficient = 0.20
		"disorganized":
			coefficient = 0.45

	var base_transfer: float = parent_stress * parent_dependency * coefficient
	if caregiver_support_active:
		var stage_data = _stages.get(stage, {})
		base_transfer *= (1.0 - float(stage_data.get("buffer_power", 0.0)))

	var combined: float = 1.0 - (1.0 - base_transfer) * (1.0 - contagion_input)
	return clampf(combined, 0.0, 1.0)


## [Cicchetti & Toth, 2005 - Cumulative Risk Model]
func _handle_simultaneous_ace_events(events: Array, prev_residual: float) -> Dictionary:
	if events.is_empty():
		return {
			"effective_damage": 0.0,
			"scar_candidate": "",
			"kindling_bonus": 0,
		}

	var burst: float = 1.0
	var max_severity: float = -1.0
	var scar_candidate: String = ""
	for i in range(events.size()):
		var event = events[i]
		if not (event is Dictionary):
			continue
		var severity: float = clampf(float(event.get("severity", 0.0)), 0.0, 1.0)
		burst *= (1.0 - severity)
		if severity > max_severity:
			max_severity = severity
			scar_candidate = str(event.get("scar_type", ""))

	burst = 1.0 - burst
	var effective_damage: float = clampf(burst * (1.0 + prev_residual), 0.0, 1.25)
	var kindling_bonus: int = events.size() - 1
	if kindling_bonus < 0:
		kindling_bonus = 0
	return {
		"effective_damage": effective_damage,
		"scar_candidate": scar_candidate,
		"kindling_bonus": kindling_bonus,
	}
