extends "res://scripts/core/simulation_system.gd"

# NO class_name — headless compatibility

var _entity_manager
var _stages: Dictionary = {}


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


func get_current_stage(age_ticks: int) -> String:
	var years: float = float(age_ticks) / 8760.0
	for stage_name in ["infant", "toddler", "child", "teen"]:
		var stage_data = _stages.get(stage_name, {})
		if not (stage_data is Dictionary):
			continue
		var age_range = stage_data.get("age_range", [])
		if age_range is Array and age_range.size() >= 2:
			var min_age: float = float(age_range[0])
			var max_age: float = float(age_range[1])
			if years >= min_age and years < max_age:
				return stage_name

	if years < 2.0:
		return "infant"
	if years < 5.0:
		return "toddler"
	if years < 12.0:
		return "child"
	if years < 18.0:
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
			developmental_damage += intensity * float(stage_data.get("break_threshold_mult", 1.0)) * 0.02
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
	if not bool(stage_data.get("shrp_active", false)):
		return intensity
	var threshold: float = float(stage_data.get("shrp_override_threshold", 0.85))
	if intensity < threshold:
		return 0.0
	_handle_shrp_override(entity, stressor)
	return intensity * float(stage_data.get("vulnerability_mult", 1.0))


func _handle_shrp_override(entity, stressor: Dictionary) -> void:
	if entity == null:
		return
	if entity.emotion_data != null:
		entity.emotion_data.set_meta("shrp_breached", true)

	var chronicle = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
	if chronicle != null:
		var tick: int = int(stressor.get("tick", -1))
		var params: Dictionary = {"name": entity.entity_name}
		var desc: String = Locale.trf("SHRP_OVERRIDE", params)
		chronicle.log_event("child_stress", entity.id, desc, 4, [], tick, {
			"key": "SHRP_OVERRIDE",
			"params": params,
		})


func _accumulate_deprivation_damage(entity, stressor: Dictionary, _stage_data: Dictionary) -> void:
	if entity == null or entity.emotion_data == null:
		return
	var rate: float = float(stressor.get("developmental_damage_rate", 0.01))
	var current: float = float(entity.emotion_data.get_meta("developmental_damage", 0.0))
	entity.emotion_data.set_meta("developmental_damage", current + rate)


## [Shonkoff et al., 2012 - Toxic Stress 3-category model]
## Key: caregiver presence determines toxic vs tolerable, NOT just intensity.
func _classify_stress_type(intensity: float, attachment_present: bool, quality: float,
		entity = null, tick: int = -1) -> String:
	if intensity < 0.30:
		return "positive"
	if attachment_present and quality > 0.50:
		return "tolerable"

	if entity != null:
		var chronicle = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
		if chronicle != null:
			var desc: String = Locale.ltr("TOXIC_STRESS_ONSET")
			desc = desc.format({"name": entity.entity_name})
			chronicle.log_event("child_stress", entity.id, desc, 3, [], tick, {
				"key": "TOXIC_STRESS_ONSET",
				"params": {"name": entity.entity_name},
			})
	return "toxic"


## [Hostinar, Sullivan & Gunnar, 2014 - Social Buffering]
func _apply_social_buffer(intensity: float, stage: String,
		attachment_quality: float, caregiver_present: bool) -> float:
	if not caregiver_present:
		return intensity
	var stage_data = _stages.get(stage, {})
	var buffer_power: float = float(stage_data.get("buffer_power", 0.0))
	var social_buffer: float = attachment_quality * buffer_power
	return intensity * (1.0 - social_buffer)


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


func execute_tick(tick: int) -> void:
	if _entity_manager == null:
		return
	var alive = _entity_manager.get_alive_entities()
	for entity in alive:
		if entity.emotion_data == null:
			continue
		var childhood_data = entity.get_meta("childhood_data", null)
		if childhood_data == null:
			continue

		var stage = childhood_data.get("current_stage", "")
		if stage == "":
			stage = get_current_stage(int(entity.age))
			childhood_data["current_stage"] = stage
		if stage == "" or stage == "adult":
			entity.set_meta("childhood_data", childhood_data)
			continue

		# Process parent stress transfer this tick
		var parent_stress = childhood_data.get("parent_stress_level", 0.0)
		if parent_stress > 0.1:
			var contagion_in = entity.emotion_data.get_meta("contagion_stress_this_tick", 0.0)
			var transferred = _calculate_parent_stress_transfer(
				parent_stress,
				_stages.get(stage, {}).get("parent_dependency", 0.5),
				childhood_data.get("attachment_type", "secure"),
				childhood_data.get("caregiver_support_active", false),
				stage,
				contagion_in
			)
			if transferred > 0.05:
				entity.emotion_data.stress = clampf(
					entity.emotion_data.stress + transferred * 20.0, 0.0, 2000.0
				)

		var pending_stressors = childhood_data.get("pending_stressors", [])
		if pending_stressors is Array:
			for i in range(pending_stressors.size()):
				var stressor = pending_stressors[i]
				if stressor is Dictionary:
					process_stressor(entity, stressor, tick)
			childhood_data["pending_stressors"] = []

		var simultaneous_events = childhood_data.get("simultaneous_ace_events", [])
		if simultaneous_events is Array and simultaneous_events.size() > 0:
			var prev_residual: float = float(childhood_data.get("ace_residual_arousal", 0.0))
			var ace_result: Dictionary = _handle_simultaneous_ace_events(simultaneous_events, prev_residual)
			childhood_data["ace_residual_arousal"] = ace_result.get("effective_damage", 0.0)
			if entity.emotion_data != null:
				entity.emotion_data.set_meta("ace_kindling_bonus", ace_result.get("kindling_bonus", 0))
				entity.emotion_data.set_meta("ace_scar_candidate", ace_result.get("scar_candidate", ""))
			childhood_data["simultaneous_ace_events"] = []

		entity.set_meta("childhood_data", childhood_data)
