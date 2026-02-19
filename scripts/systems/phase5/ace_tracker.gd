extends RefCounted

const ACE_DEFINITIONS_PATH: String = "res://data/ace_definitions.json"
const ACE_ADULT_MODIFIERS_PATH: String = "res://data/ace_adult_modifiers.json"
const ACE_ITEM_IDS: Array = [
	"physical_abuse",
	"emotional_abuse",
	"sexual_abuse",
	"physical_neglect",
	"emotional_neglect",
	"domestic_violence",
	"substance_household",
	"mental_illness_household",
	"parental_separation",
	"incarceration",
]

var ace_score_total: float = 0.0
var is_backfilled: bool = false

## ace_items: { "physical_abuse": {"partial_score": 0.0~1.0, "event_count": int, "last_tick": int}, ... }
var ace_items: Dictionary = {}

var _ace_defs: Dictionary = {}
var _curve_table: Dictionary = {}
var _config: Dictionary = {}

var _protective_factor: float = 0.0


func _init() -> void:
	_load_data()
	# Initialize all 10 items to zero
	for item_id in ACE_ITEM_IDS:
		ace_items[item_id] = {"partial_score": 0.0, "event_count": 0, "last_tick": -1}


func _load_data() -> void:
	_ace_defs = _load_json_dictionary(ACE_DEFINITIONS_PATH)
	_config = _load_json_dictionary(ACE_ADULT_MODIFIERS_PATH)
	_curve_table = _config.get("curve_table", {})


func _load_json_dictionary(path: String) -> Dictionary:
	var file = FileAccess.open(path, FileAccess.READ)
	if file == null:
		push_warning("[AceTracker] Failed to open %s" % path)
		return {}

	var json = JSON.new()
	var parse_error = json.parse(file.get_as_text())
	file.close()
	if parse_error != OK:
		push_warning("[AceTracker] JSON parse error in %s" % path)
		return {}

	if typeof(json.data) != TYPE_DICTIONARY:
		push_warning("[AceTracker] Expected dictionary JSON in %s" % path)
		return {}

	return json.data


func record_ace_event(item_id: String, severity: float, tick: int, entity_name: String) -> void:
	## [Felitti et al., 1998 - ACE Study original methodology]
	## [McLaughlin et al., 2014 - Threat vs Deprivation neural pathway distinction]
	## Threat type → amygdala/PFC pathway (emotional reactivity).
	## Deprivation type → reward circuit/hippocampus (social/cognitive function).
	## Design Note: partial_score accumulates per exposure; ace_score_total = sum of partial scores capped at 10.
	## Reference: Felitti, V.J. et al. (1998). Am J Prev Med, 14(4).
	## Reference: McLaughlin, K.A. et al. (2014). Neuroscience & Biobehavioral Reviews, 47.
	if not _ace_defs.has(item_id):
		push_warning("[AceTracker] Unknown ACE item_id: %s" % item_id)
		return

	var item_def = _ace_defs.get(item_id, {})
	var item_state = ace_items.get(item_id, {"partial_score": 0.0, "event_count": 0, "last_tick": -1})

	var last_tick = int(item_state.get("last_tick", -1))
	var cooldown_ticks = int(item_def.get("cooldown_ticks", 0))
	if last_tick >= 0 and tick - last_tick < cooldown_ticks:
		return

	var ace_weight = float(item_def.get("ace_weight", 1.0))
	var partial_score = float(item_state.get("partial_score", 0.0))
	partial_score = minf(1.0, partial_score + maxf(0.0, severity) * ace_weight)

	item_state["partial_score"] = partial_score
	item_state["event_count"] = int(item_state.get("event_count", 0)) + 1
	item_state["last_tick"] = tick
	ace_items[item_id] = item_state

	_recalculate_total_score()

	if severity > 0.3:
		var name_key = str(item_def.get("name_key", item_id))
		var ace_type_name = Locale.ltr(name_key)
		var params = {"name": entity_name, "ace_type": ace_type_name}
		var desc = Locale.trf("ACE_EVENT_RECORDED", params)
		var chronicle = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
		if chronicle != null and chronicle.has_method("log_event"):
			chronicle.log_event("ace_event", -1, desc, 2, [], tick, {"key": "ACE_EVENT_RECORDED", "params": params})


func get_threat_deprivation_scores() -> Dictionary:
	## [McLaughlin et al., 2014 - Threat vs Deprivation Distinction]
	## ACE type determines which brain regions are damaged and which adult outcomes result.
	## Threat → emotional reactivity (rage/panic probability).
	## Deprivation → reward/social function (social support efficiency, cognitive flexibility).
	## Reference: McLaughlin, K.A. et al. (2014). Neuroscience & Biobehavioral Reviews, 47.
	var threat_score: float = 0.0
	var deprivation_score: float = 0.0

	for item_id in ace_items:
		var item_state = ace_items.get(item_id, {})
		var partial_score = float(item_state.get("partial_score", 0.0))
		var item_type = classify_ace_type(str(item_id))
		if item_type == "threat":
			threat_score += partial_score
		elif item_type == "deprivation":
			deprivation_score += partial_score

	return {
		"threat": threat_score,
		"deprivation": deprivation_score,
	}


func calculate_adult_modifiers() -> Dictionary:
	## [Felitti et al., 1998 - ACE Study dose-response curve]
	## [Werner & Smith, 1992 - Protective factors in resilience]
	## ACE-outcome relationship is convex (3-segment acceleration): gradual 0~3, accelerating 4~6, steep 7~10.
	## Design Note (GPT bug fix): break_threshold_mult floor = 0.50.
	##   Raw ACE10 → 0.24 causes daily break explosions. Instead, ace_definitions break_probability_mods
	##   handles type-specific OR increases separately.
	## Protective factor partially mitigates effects; allostatic_base only 50% mitigated (biological embedding).
	## Reference: Felitti, V.J. et al. (1998). Am J Prev Med, 14(4).
	## Reference: Werner, E.E. & Smith, R.S. (1992). Overcoming the odds. Cornell University Press.
	var ace_int: int = int(clampf(ace_score_total, 0.0, 10.0))
	var curve_key: String = str(ace_int)
	var curve = _curve_table.get(curve_key, {})
	var base: Dictionary = curve.duplicate() if typeof(curve) == TYPE_DICTIONARY else {}

	if base.is_empty():
		base = {
			"stress_gain_mult": 1.0,
			"break_threshold_mult": 1.0,
			"allostatic_base": 0.0,
		}

	var floor = _config.get("break_threshold_floor", 0.50)
	base["break_threshold_mult"] = maxf(float(base.get("break_threshold_mult", 1.0)), float(floor))

	var pf: float = _calculate_protective_factor()
	base["stress_gain_mult"] = 1.0 + (float(base.get("stress_gain_mult", 1.0)) - 1.0) * (1.0 - pf)
	base["break_threshold_mult"] = 1.0 - (1.0 - float(base.get("break_threshold_mult", 1.0))) * (1.0 - pf)
	base["allostatic_base"] = float(base.get("allostatic_base", 0.0)) * (1.0 - 0.5 * pf)

	var merged_break_probability_mods: Dictionary = {}
	for item_id in ace_items:
		var item_state = ace_items.get(item_id, {})
		var partial_score = float(item_state.get("partial_score", 0.0))
		if partial_score < 0.5:
			continue

		var item_def = _ace_defs.get(item_id, {})
		var break_probability_mods = item_def.get("break_probability_mods", {})
		if typeof(break_probability_mods) != TYPE_DICTIONARY:
			continue
		for break_type in break_probability_mods:
			var current_mult = float(merged_break_probability_mods.get(break_type, 1.0))
			merged_break_probability_mods[break_type] = current_mult * float(break_probability_mods.get(break_type, 1.0))

	if merged_break_probability_mods.size() > 0:
		base["break_probability_mods"] = merged_break_probability_mods

	return base


func _calculate_protective_factor() -> float:
	# Simplified: returns 0.0 by default
	# Will be set by parenting_system once attachment is determined
	return _protective_factor


func set_protective_factor(pf: float) -> void:
	_protective_factor = clampf(pf, 0.0, 0.45)


func apply_hexaco_caps(entity) -> void:
	## [Teicher & Samson, 2016 - Neurobiological Effects of Childhood Maltreatment]
	## Chronic childhood maltreatment permanently alters amygdala, PFC, hippocampus structure.
	## WorldSim: brain structural changes → HEXACO facet min/max limits permanently modified.
	## Applied ONCE at adulthood transition; changes are irreversible.
	## Reference: Teicher, M.H. & Samson, J.A. (2016). J Child Psychol Psychiatry, 57(3).
	if entity == null:
		return

	var chronicle = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")

	for item_id in ace_items:
		var item_state = ace_items.get(item_id, {})
		var partial_score = float(item_state.get("partial_score", 0.0))
		if partial_score < 0.5:
			continue

		var item_def = _ace_defs.get(item_id, {})
		var cap_mods = item_def.get("hexaco_cap_mods", {})
		if typeof(cap_mods) != TYPE_DICTIONARY:
			continue

		for facet_id in cap_mods:
			var mod = cap_mods.get(facet_id, {})
			if typeof(mod) != TYPE_DICTIONARY:
				continue

			var base_axis_value: float = 0.5
			if entity.personality != null:
				var axes = entity.personality.axes
				if typeof(axes) == TYPE_DICTIONARY:
					var axis_id = str(facet_id).split("_")[0]
					base_axis_value = float(axes.get(axis_id, 0.5))

			var meta_key: String = "hexaco_cap_" + str(facet_id)
			var existing_caps = entity.get_meta(meta_key, {})
			var cap_entry: Dictionary = existing_caps.duplicate() if typeof(existing_caps) == TYPE_DICTIONARY else {}

			if mod.has("min_increase"):
				var min_increase = float(mod.get("min_increase", 0.0)) / 100.0
				var new_min = clampf(base_axis_value + min_increase, 0.0, 1.0)
				if cap_entry.has("min"):
					new_min = maxf(float(cap_entry.get("min", 0.0)), new_min)
				cap_entry["min"] = new_min

			if mod.has("max_decrease"):
				var max_decrease = float(mod.get("max_decrease", 0.0)) / 100.0
				var new_max = clampf(base_axis_value - max_decrease, 0.0, 1.0)
				if cap_entry.has("max"):
					new_max = minf(float(cap_entry.get("max", 1.0)), new_max)
				cap_entry["max"] = new_max

			if cap_entry.has("min") and cap_entry.has("max"):
				var min_cap = float(cap_entry.get("min", 0.0))
				var max_cap = float(cap_entry.get("max", 1.0))
				if min_cap > max_cap:
					cap_entry["max"] = min_cap

			entity.set_meta(meta_key, cap_entry)

			if chronicle != null and chronicle.has_method("log_event"):
				var params = {"facet": str(facet_id)}
				var desc = Locale.trf("HEXACO_CAP_MODIFIED", params)
				var entity_id: int = int(entity.get("id")) if entity.get("id") != null else -1
				var tick: int = int(entity.get_meta("current_tick", -1))
				if tick >= 0:
					chronicle.log_event("hexaco_cap_modified", entity_id, desc, 3, [], tick, {
						"key": "HEXACO_CAP_MODIFIED",
						"params": params,
					})


func backfill_ace_for_adult(entity) -> void:
	## [Design: Adult entity backfill — Phase 2~4 data as ACE proxy]
	## Existing adult entities have no childhood history.
	## Two paths: demographic sampling OR individual trait-based estimation.
	## Using trait estimation: allostatic load, trauma scars, attachment style, coping patterns
	## as proxies for probable ACE exposure.
	if ace_score_total > 0.0:
		return
	if entity == null:
		return

	var ss = entity.emotion_data
	var trauma_scars = entity.get_meta("trauma_scars", [])
	var trauma_count: int = trauma_scars.size() if typeof(trauma_scars) == TYPE_ARRAY else 0
	var attachment = str(entity.get_meta("attachment_type", "secure"))

	var disorg_bonus: float = 1.5 if attachment == "disorganized" else 0.0
	var insecure_bonus: float = 0.7 if attachment in ["anxious", "avoidant"] else 0.0
	var allostatic: float = 0.0
	if ss != null:
		allostatic = float(ss.get("allostatic")) if ss.get("allostatic") != null else 0.0
	var stress_component: float = 0.08 * clampf(allostatic, 0.0, 100.0)
	var scar_component: float = 0.8 * float(trauma_count)
	var ace_est: float = stress_component + scar_component + disorg_bonus + insecure_bonus

	ace_score_total = clampf(ace_est, 0.0, 10.0)
	is_backfilled = true


func classify_ace_type(item_id: String) -> String:
	var item_def = _ace_defs.get(item_id, {})
	if typeof(item_def) != TYPE_DICTIONARY:
		return "unknown"
	return str(item_def.get("type", "unknown"))


func _recalculate_total_score() -> void:
	var total: float = 0.0
	for item_id in ace_items:
		var item_state = ace_items.get(item_id, {})
		total += float(item_state.get("partial_score", 0.0))
	ace_score_total = clampf(total, 0.0, 10.0)
