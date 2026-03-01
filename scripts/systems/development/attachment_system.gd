extends RefCounted

const ATTACHMENT_CONFIG_PATH: String = "res://data/attachment_config.json"
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_TYPE_CODE_METHOD: String = "body_attachment_type_code"
const _SIM_BRIDGE_RAW_QUALITY_METHOD: String = "body_attachment_raw_parenting_quality"
const _SIM_BRIDGE_COPING_QUALITY_STEP_METHOD: String = "body_attachment_coping_quality_step"
const _SIM_BRIDGE_PROTECTIVE_FACTOR_METHOD: String = "body_attachment_protective_factor"

var _attach_config: Dictionary = {}
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func _init() -> void:
	_load_config()


func _load_config() -> void:
	_attach_config.clear()
	if not FileAccess.file_exists(ATTACHMENT_CONFIG_PATH):
		return
	var file = FileAccess.open(ATTACHMENT_CONFIG_PATH, FileAccess.READ)
	if file == null:
		return
	var text: String = file.get_as_text()
	file.close()

	var json = JSON.new()
	var err: int = json.parse(text)
	if err != OK:
		return

	var data = json.get_data()
	if typeof(data) == TYPE_DICTIONARY:
		_attach_config = data.duplicate(true)


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
	and node.has_method(_SIM_BRIDGE_TYPE_CODE_METHOD) \
	and node.has_method(_SIM_BRIDGE_RAW_QUALITY_METHOD) \
	and node.has_method(_SIM_BRIDGE_COPING_QUALITY_STEP_METHOD) \
	and node.has_method(_SIM_BRIDGE_PROTECTIVE_FACTOR_METHOD):
		_sim_bridge = node
	return _sim_bridge


## [Ainsworth, Blehar, Waters & Wall, 1978 - Strange Situation Procedure]
## Attachment type formed during first 12-18 months based on caregiver responsiveness patterns.
## sensitivity = average of caregiver_sensitivity_samples; consistency = caregiver_consistency value.
## Disorganized requires BOTH ACE 4+ AND abuser=caregiver (Main & Solomon 1990 criteria).
func determine_attachment_type(child_data: Dictionary) -> String:
	var samples = child_data.get("caregiver_sensitivity_samples", [0.5])
	if typeof(samples) != TYPE_ARRAY:
		samples = [0.5]

	var sensitivity: float = 0.0
	for s in samples:
		sensitivity += float(s)
	sensitivity /= float(max(1, samples.size()))

	var consistency = float(child_data.get("caregiver_consistency", 0.5))
	var cfg = _attach_config
	var sensitivity_threshold_secure: float = float(cfg.get("sensitivity_threshold_secure", 0.65))
	var consistency_threshold_secure: float = float(cfg.get("consistency_threshold_secure", 0.60))
	var sensitivity_threshold_anxious: float = float(cfg.get("sensitivity_threshold_anxious", 0.40))
	var consistency_threshold_disorganized: float = float(cfg.get("consistency_threshold_disorganized", 0.35))
	var abuser_is_caregiver_ace_min: float = float(cfg.get("abuser_is_caregiver_ace_min", 4))
	var avoidant_sensitivity_max: float = float(cfg.get("avoidant_sensitivity_max", 0.35))
	var avoidant_consistency_min: float = float(cfg.get("avoidant_consistency_min", 0.50))
	var ace_score: float = float(child_data.get("ace_score", 0.0))
	var abuser_is_caregiver: bool = bool(child_data.get("abuser_is_caregiver", false))

	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_code_variant: Variant = bridge.call(
			_SIM_BRIDGE_TYPE_CODE_METHOD,
			sensitivity,
			consistency,
			ace_score,
			abuser_is_caregiver,
			sensitivity_threshold_secure,
			consistency_threshold_secure,
			sensitivity_threshold_anxious,
			consistency_threshold_disorganized,
			abuser_is_caregiver_ace_min,
			avoidant_sensitivity_max,
			avoidant_consistency_min
		)
		if rust_code_variant is int:
			var code: int = int(rust_code_variant)
			if code == 0:
				return "secure"
			if code == 1:
				return "anxious"
			if code == 2:
				return "avoidant"
			if code == 3:
				return "disorganized"

	if sensitivity >= sensitivity_threshold_secure and consistency >= consistency_threshold_secure:
		return "secure"
	elif sensitivity >= sensitivity_threshold_anxious and consistency < consistency_threshold_disorganized:
		return "anxious"
	elif sensitivity < avoidant_sensitivity_max and consistency >= avoidant_consistency_min:
		return "avoidant"
	else:
		if ace_score >= abuser_is_caregiver_ace_min and abuser_is_caregiver:
			return "disorganized"
		return "anxious"


## Persist attachment outcome and log chronicle event.
func finalize_attachment(entity, attachment_type: String, tick: int) -> void:
	if entity == null:
		return
	entity.set_meta("attachment_type", attachment_type)

	var entity_name: String = "?"
	if "entity_name" in entity:
		entity_name = str(entity.entity_name)

	var type_key: String = "ATTACHMENT_" + attachment_type.to_upper()
	var desc: String = Locale.trf2("ATTACHMENT_FORMED", "name", entity_name, "type", Locale.ltr(type_key))

	var chronicle = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
	if chronicle != null:
		var entity_id: int = -1
		if "id" in entity:
			entity_id = int(entity.id)
		chronicle.log_event("attachment_formed", entity_id, desc, 3, [], tick)


## [Bowlby, 1969 - Attachment as lifelong internal working model]
## Store adult attachment effect multipliers for other systems.
func apply_adult_effects(entity, attachment_type: String) -> void:
	if entity == null:
		return
	var effects = _attach_config.get("adult_effects", {}).get(attachment_type, {})
	if typeof(effects) != TYPE_DICTIONARY:
		effects = {}

	entity.set_meta("attachment_type", attachment_type)
	entity.set_meta("attachment_transfer_coefficient", float(effects.get("transfer_coefficient", 0.25)))
	entity.set_meta("attachment_coping_mult", float(effects.get("coping_modeling_rate_mult", 1.0)))

	if attachment_type == "avoidant":
		entity.set_meta("stress_expression_suppressed", true)
	if attachment_type == "disorganized":
		entity.set_meta("coping_random_fire_chance", float(effects.get("coping_random_fire_chance", 0.0)))


## [GPT edge case analysis - Disorganized parent quality variability]
## Apply variability to parenting quality while preserving mean base quality.
func calculate_parenting_quality_with_noise(entity) -> float:
	var raw_quality: float = _compute_raw_quality(entity)

	var disorganized_level: float = 0.0
	if entity != null and str(entity.get_meta("attachment_type", "")) == "disorganized":
		disorganized_level = 1.0
	if entity != null:
		disorganized_level = clampf(float(entity.get_meta("disorganized_intensity", disorganized_level)), 0.0, 1.0)

	var noise_std: float = 0.08 + 0.12 * disorganized_level
	var noise: float = randf_range(-noise_std * 2.0, noise_std * 2.0)
	return clampf(raw_quality + noise, 0.0, 1.0)


## [Meaney, 2001 + Morris, 2007 - Parenting quality determinants]
## Compute parenting quality from personality, stress burden, and ACE history.
func _compute_raw_quality(entity) -> float:
	if entity == null:
		return 0.5
	var has_personality: bool = entity.personality != null
	var a_axis: float = 0.0
	var e_axis: float = 0.0
	if has_personality:
		a_axis = StatQuery.get_normalized(entity, &"HEXACO_A")
		e_axis = StatQuery.get_normalized(entity, &"HEXACO_E")
	var has_emotion_data: bool = entity.emotion_data != null
	var stress: float = 0.0
	var allostatic: float = 0.0
	var has_active_break: bool = false
	if has_emotion_data:
		stress = entity.emotion_data.stress
		allostatic = entity.emotion_data.allostatic
		has_active_break = entity.emotion_data.mental_break_type != ""
	var ace_score: float = float(entity.get_meta("ace_score_total", 0.0))

	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_quality_variant: Variant = bridge.call(
			_SIM_BRIDGE_RAW_QUALITY_METHOD,
			has_personality,
			a_axis,
			e_axis,
			has_emotion_data,
			stress,
			allostatic,
			has_active_break,
			ace_score
		)
		if rust_quality_variant is float:
			return float(rust_quality_variant)

	var base: float = 0.50
	if has_personality:
		base += 0.15 * a_axis
		base += 0.10 * e_axis

	if has_emotion_data:
		base -= 0.20 * clampf(stress / 2000.0, 0.0, 1.0)
		base -= 0.15 * clampf(allostatic / 100.0, 0.0, 1.0)

	if has_active_break:
		base -= 0.30

	base -= 0.10 * clampf(ace_score / 10.0, 0.0, 1.0)
	return clampf(base, 0.0, 1.0)


## [GPT bug fix - substance_use parent paradox]
## Apply direct quality penalty for substance coping regardless of stress level.
func _apply_coping_modifiers_to_quality(entity, base_quality: float) -> float:
	var quality: float = base_quality
	if entity == null:
		return clampf(quality, 0.0, 1.0)

	var has_substance = bool(entity.get_meta("coping_substance_use", false))
	if has_substance:
		var dependency: float = clampf(float(entity.get_meta("substance_dependency", 0.0)), 0.0, 1.0)
		var neglect_chance: float = float(entity.get_meta("neglect_event_chance", 0.0))
		var consistency_pen: float = float(entity.get_meta("consistency_penalty", 0.0))
		var bridge: Object = _get_sim_bridge()
		if bridge != null:
			var rust_step_variant: Variant = bridge.call(
				_SIM_BRIDGE_COPING_QUALITY_STEP_METHOD,
				quality,
				dependency,
				neglect_chance,
				consistency_pen
			)
			if rust_step_variant is PackedFloat32Array:
				var out: PackedFloat32Array = rust_step_variant
				if out.size() >= 3:
					quality = float(out[0])
					entity.set_meta("neglect_event_chance", float(out[1]))
					entity.set_meta("consistency_penalty", float(out[2]))
					return clampf(quality, 0.0, 1.0)

		quality -= 0.10 + 0.15 * dependency
		entity.set_meta("neglect_event_chance", neglect_chance + 0.02 * (1.0 + dependency))
		entity.set_meta("consistency_penalty", consistency_pen + 0.15)

	return clampf(quality, 0.0, 1.0)


## Compute full parenting quality (raw + disorganized variance + coping penalties).
func get_full_parenting_quality(entity) -> float:
	var base: float = calculate_parenting_quality_with_noise(entity)
	return _apply_coping_modifiers_to_quality(entity, base)


## [attachment_config protective_factor section]
## Compute partial protective factor against ACE burden.
func calculate_protective_factor(attachment_type: String, entity) -> float:
	var pf_cfg = _attach_config.get("protective_factor", {})
	if typeof(pf_cfg) != TYPE_DICTIONARY:
		pf_cfg = {}

	var pf: float = 0.0
	var is_secure: bool = attachment_type == "secure"
	var secure_weight: float = float(pf_cfg.get("secure_weight", 0.30))
	var eh_weight: float = float(pf_cfg.get("eh_weight", 0.15))
	var max_pf: float = float(pf_cfg.get("max_pf", 0.45))
	var eh: float = 0.0

	if entity != null and entity.emotion_data != null:
		eh = clampf(1.0 - entity.emotion_data.allostatic / 100.0, 0.0, 1.0)

	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_pf_variant: Variant = bridge.call(
			_SIM_BRIDGE_PROTECTIVE_FACTOR_METHOD,
			is_secure,
			eh,
			secure_weight,
			eh_weight,
			max_pf
		)
		if rust_pf_variant is float:
			return float(rust_pf_variant)

	if is_secure:
		pf += secure_weight
	pf += eh_weight * eh
	return clampf(pf, 0.0, max_pf)
