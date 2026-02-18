extends "res://scripts/core/simulation_system.gd"

## TraitSystem — 2-level hybrid personality trait system.
## Design: Costa & McCrae (1992), Asendorpf (2003), Smithson & Verkuilen (2006).
## Games: DF (continuous + filter), CK3 (Top-K), RimWorld (few traits).

const PersonalityData = preload("res://scripts/core/personality_data.gd")
const EntityData = preload("res://scripts/core/entity_data.gd")

const TOP_K: int = 5
const MIN_DISPLAY_SALIENCE: float = 0.10
const MAX_PER_AXIS: int = 2
const MAX_DARK_DISPLAY: int = 2
const VIOLATION_ALPHA: float = 1.2
const EPS: float = 0.0001

const TRAIT_DEFS_PATH: String = "res://data/personality/trait_defs_v2.json"
const BEHAVIOR_MAP_PATH: String = "res://data/personality/behavior_mappings.json"
const EMOTION_MAP_PATH: String = "res://data/personality/emotion_mappings.json"
const VIOLATION_MAP_PATH: String = "res://data/personality/violation_mappings.json"

const FACET_ALIASES: Dictionary = {
	"O_aesthetic_appreciation": "O_aesthetic",
}

static var _trait_defs: Array = []
static var _trait_index: Dictionary = {}
static var _mutex_pairs: Dictionary = {}
static var _behavior_map: Dictionary = {}
static var _emotion_map: Dictionary = {}
static var _violation_map: Dictionary = {}
static var _loaded: bool = false


static func _ensure_loaded() -> void:
	if _loaded:
		return

	_trait_defs = []
	_trait_index = {}
	_mutex_pairs = {}
	_behavior_map = {}
	_emotion_map = {}
	_violation_map = {}

	var trait_defs_data = _load_json(TRAIT_DEFS_PATH)
	if trait_defs_data is Array:
		var raw_defs: Array = trait_defs_data
		for i in range(raw_defs.size()):
			var def: Dictionary = raw_defs[i]
			var tid: String = str(def.get("id", ""))
			if tid == "":
				continue
			_trait_defs.append(def)
			_trait_index[tid] = def
	else:
		push_warning("[TraitSystem] trait_defs_v2.json root is not an Array")

	var behavior_data = _load_json(BEHAVIOR_MAP_PATH)
	if behavior_data is Dictionary:
		_behavior_map = behavior_data
	else:
		push_warning("[TraitSystem] behavior_mappings.json root is not a Dictionary")

	var emotion_data = _load_json(EMOTION_MAP_PATH)
	if emotion_data is Dictionary:
		_emotion_map = emotion_data
	else:
		push_warning("[TraitSystem] emotion_mappings.json root is not a Dictionary")

	var violation_data = _load_json(VIOLATION_MAP_PATH)
	if violation_data is Dictionary:
		_violation_map = violation_data
	else:
		push_warning("[TraitSystem] violation_mappings.json root is not a Dictionary")

	_build_mutex_pairs()

	_loaded = true
	print("[TraitSystem] Loaded defs=%d behavior=%d emotion=%d violation=%d" % [
		_trait_defs.size(),
		_behavior_map.size(),
		_emotion_map.size(),
		_violation_map.size(),
	])


static func _load_json(path: String):
	var file: FileAccess = FileAccess.open(path, FileAccess.READ)
	if file == null:
		push_warning("[TraitSystem] Cannot load " + path)
		return null
	var json: JSON = JSON.new()
	if json.parse(file.get_as_text()) != OK:
		push_warning("[TraitSystem] Invalid JSON " + path)
		return null
	return json.data


static func _build_mutex_pairs() -> void:
	_mutex_pairs = {}
	var groups: Dictionary = {}
	for i in range(_trait_defs.size()):
		var def: Dictionary = _trait_defs[i]
		if str(def.get("category", "")) != "facet":
			continue
		var mutex_group: String = str(def.get("mutex_group", ""))
		var direction: String = str(def.get("direction", ""))
		var tid: String = str(def.get("id", ""))
		if mutex_group == "" or tid == "":
			continue
		var pair: Dictionary = groups.get(mutex_group, {"high": "", "low": ""})
		if direction == "high":
			pair["high"] = tid
		elif direction == "low":
			pair["low"] = tid
		groups[mutex_group] = pair

	var keys: Array = groups.keys()
	for i in range(keys.size()):
		var key: String = str(keys[i])
		var item: Dictionary = groups[key]
		_mutex_pairs[key] = [str(item.get("high", "")), str(item.get("low", ""))]


static func _has_property(target: Object, property_name: String) -> bool:
	if target == null:
		return false
	var props: Array = target.get_property_list()
	for i in range(props.size()):
		var p: Dictionary = props[i]
		if str(p.get("name", "")) == property_name:
			return true
	return false


static func _get_facet_value(pd: RefCounted, facet_key: String) -> float:
	if pd == null:
		return 0.5
	if "_" in facet_key:
		if pd.facets.has(facet_key):
			return float(pd.facets.get(facet_key, 0.5))
		var alias_key: String = str(FACET_ALIASES.get(facet_key, ""))
		if alias_key != "" and pd.facets.has(alias_key):
			return float(pd.facets.get(alias_key, 0.5))
		return 0.5
	return float(pd.axes.get(facet_key, 0.5))


static func _sigmoid(x: float, center: float, width: float) -> float:
	var safe_width: float = maxf(width, EPS)
	var k: float = 10.0 / safe_width
	return 1.0 / (1.0 + exp(-k * (x - center)))


static func _calc_facet_salience(facet_val: float, direction: String, center: float, width: float) -> float:
	if direction == "high":
		return _sigmoid(facet_val, center, width)
	return _sigmoid(-facet_val, -center, width)


static func _calc_composite_salience(pd: RefCounted, def: Dictionary) -> float:
	var conditions: Array = def.get("conditions", [])
	if conditions.is_empty():
		return 0.0
	var product: float = 1.0
	for i in range(conditions.size()):
		var cond: Dictionary = conditions[i]
		var fval: float = _get_facet_value(pd, str(cond.get("facet", "")))
		var center: float = float(cond.get("cond_center", 0.5))
		var width: float = float(cond.get("cond_width", 0.2))
		var direction: String = str(cond.get("direction", "high"))
		var sat: float = _calc_facet_salience(fval, direction, center, width)
		product *= sat
	var n: int = conditions.size()
	var geo_mean: float = pow(product, 1.0 / float(n))
	var rarity_bonus: float = float(def.get("rarity_bonus", 1.0))
	return clamp(geo_mean * rarity_bonus, 0.0, 1.0)


static func _inverse_lerp(a: float, b: float, v: float) -> float:
	var denom: float = b - a
	if absf(denom) <= EPS:
		return 0.0
	return (v - a) / denom


static func _get_trait_strength(entity: RefCounted, trait_id: String) -> float:
	if entity == null:
		return 0.0
	if not _has_property(entity, "trait_strengths"):
		return 0.0
	var raw = entity.get("trait_strengths")
	if raw is Dictionary:
		var strengths: Dictionary = raw
		return float(strengths.get(trait_id, 0.0))
	return 0.0


static func _update_hysteresis(entity: RefCounted, trait_id: String, facet_val: float, def: Dictionary) -> bool:
	var prev: bool = false
	var state: Dictionary = {}
	var has_state: bool = false
	if entity != null and _has_property(entity, "_trait_display_active"):
		var raw_state = entity.get("_trait_display_active")
		if raw_state is Dictionary:
			state = raw_state
		has_state = true
		prev = bool(state.get(trait_id, false))

	var t_on: float = float(def.get("t_on", 0.9))
	var t_off: float = float(def.get("t_off", 0.84))
	var direction: String = str(def.get("direction", "high"))
	var new_on: bool
	if direction == "high":
		new_on = facet_val > t_off if prev else facet_val >= t_on
	else:
		new_on = facet_val < t_off if prev else facet_val <= t_on

	if has_state:
		state[trait_id] = new_on
		entity.set("_trait_display_active", state)
	return new_on


## Update trait strengths from entity.personality.
static func update_trait_strengths(entity: RefCounted) -> void:
	_ensure_loaded()
	if entity == null:
		return
	if not _has_property(entity, "personality"):
		return
	var pd: RefCounted = entity.get("personality")
	if pd == null:
		return

	var strengths: Dictionary = {}

	for i in range(_trait_defs.size()):
		var def: Dictionary = _trait_defs[i]
		var category: String = str(def.get("category", ""))
		if category != "facet":
			continue
		var tid: String = str(def.get("id", ""))
		var facet: String = str(def.get("facet", ""))
		if tid == "" or facet == "":
			continue
		var fval: float = _get_facet_value(pd, facet)
		var direction: String = str(def.get("direction", "high"))
		var center: float = float(def.get("salience_center", def.get("threshold", 0.5)))
		var width: float = float(def.get("salience_width", 0.12))
		strengths[tid] = _calc_facet_salience(fval, direction, center, width)

	var mutex_keys: Array = _mutex_pairs.keys()
	for i in range(mutex_keys.size()):
		var key: String = str(mutex_keys[i])
		var pair_data = _mutex_pairs.get(key, [])
		if not (pair_data is Array):
			continue
		var pair: Array = pair_data
		if pair.size() < 2:
			continue
		var hi_id: String = str(pair[0])
		var lo_id: String = str(pair[1])
		if hi_id == "" or lo_id == "":
			continue
		var raw_hi: float = float(strengths.get(hi_id, 0.0))
		var raw_lo: float = float(strengths.get(lo_id, 0.0))
		if raw_hi >= raw_lo:
			strengths[lo_id] = 0.0
		else:
			strengths[hi_id] = 0.0

	for i in range(_trait_defs.size()):
		var def: Dictionary = _trait_defs[i]
		var category: String = str(def.get("category", ""))
		if category == "facet":
			continue
		var tid: String = str(def.get("id", ""))
		if tid == "":
			continue
		strengths[tid] = _calc_composite_salience(pd, def)

	if _has_property(entity, "trait_strengths"):
		entity.set("trait_strengths", strengths)
	if _has_property(entity, "display_traits"):
		entity.set("display_traits", get_display_traits(entity, TOP_K))
	if _has_property(entity, "traits_dirty"):
		entity.set("traits_dirty", false)


## Select Top-K display traits with diversity constraints.
static func get_display_traits(entity: RefCounted, k: int = TOP_K) -> Array:
	_ensure_loaded()
	if entity == null:
		return []

	var strengths: Dictionary = {}
	if _has_property(entity, "trait_strengths"):
		var raw_strengths = entity.get("trait_strengths")
		if raw_strengths is Dictionary:
			strengths = raw_strengths
	if strengths.is_empty():
		return []

	var pd: RefCounted = entity.get("personality") if _has_property(entity, "personality") else null
	var candidates: Array = []
	var trait_ids: Array = strengths.keys()
	for i in range(trait_ids.size()):
		var tid: String = str(trait_ids[i])
		var sal: float = float(strengths.get(tid, 0.0))
		if sal < MIN_DISPLAY_SALIENCE:
			continue
		var def: Dictionary = _trait_index.get(tid, {})
		if def.is_empty():
			continue
		var category: String = str(def.get("category", ""))
		if category == "facet" and pd != null:
			var facet_key: String = str(def.get("facet", ""))
			if facet_key != "":
				var fval: float = _get_facet_value(pd, facet_key)
				if not _update_hysteresis(entity, tid, fval, def):
					continue
		candidates.append({"id": tid, "salience": sal})

	candidates.sort_custom(func(a: Dictionary, b: Dictionary): return float(a.get("salience", 0.0)) > float(b.get("salience", 0.0)))

	var selected: Array = []
	var axis_count: Dictionary = {}
	var dark_count: int = 0
	var used_mutex: Dictionary = {}

	for i in range(candidates.size()):
		if selected.size() >= k:
			break
		var c: Dictionary = candidates[i]
		var tid: String = str(c.get("id", ""))
		var def: Dictionary = _trait_index.get(tid, {})
		if def.is_empty():
			continue
		var category: String = str(def.get("category", ""))
		var axis: String = str(def.get("axis", ""))
		var mutex_group: String = str(def.get("mutex_group", ""))
		if category == "" and tid.begins_with("d_"):
			category = "dark"

		if mutex_group != "" and used_mutex.has(mutex_group):
			continue
		if axis != "" and int(axis_count.get(axis, 0)) >= MAX_PER_AXIS:
			continue
		if category == "dark" and dark_count >= MAX_DARK_DISPLAY:
			continue

		var entry: Dictionary = {
			"id": tid,
			"name_key": str(def.get("name_key", "TRAIT_" + tid + "_NAME")),
			"salience": float(c.get("salience", 0.0)),
			"valence": str(def.get("valence", "neutral")),
			"category": category,
		}
		selected.append(entry)

		if axis != "":
			axis_count[axis] = int(axis_count.get(axis, 0)) + 1
		if category == "dark":
			dark_count += 1
		if mutex_group != "":
			used_mutex[mutex_group] = true

	return selected


## Unified entry point for effect queries.
static func get_effect_value(entity: RefCounted, effect_type: String, key: String = "") -> float:
	_ensure_loaded()
	if entity == null:
		return 1.0
	match effect_type:
		"behavior_weight":
			return _calc_behavior_weight(entity, key)
		"emotion_baseline":
			return _calc_emotion_baseline(entity, key)
		"emotion_sensitivity":
			return _calc_emotion_sensitivity(entity, key)
		"violation_stress":
			return _calc_violation_stress(entity, key)
		_:
			return _calc_generic_mult(entity, effect_type, key)
	return 1.0


static func _calc_behavior_weight(entity: RefCounted, action: String) -> float:
	var mappings: Array = _behavior_map.get(action, [])
	if mappings.is_empty():
		return 1.0
	if not _has_property(entity, "personality"):
		return 1.0
	var pd: RefCounted = entity.get("personality")
	if pd == null:
		return 1.0

	var final_weight: float = 1.0
	for i in range(mappings.size()):
		var m: Dictionary = mappings[i]
		var tid: String = str(m.get("trait_id", ""))
		var extreme_val: float = float(m.get("extreme_val", 1.0))
		var source: String = str(m.get("source", "facet"))
		var influence: float = 1.0
		if source == "facet":
			var facet: String = str(m.get("facet", ""))
			var threshold: float = float(m.get("threshold", 0.5))
			var fval: float = _get_facet_value(pd, facet)
			var t: float = clamp(_inverse_lerp(0.5, threshold, fval), 0.0, 1.0)
			influence = lerp(1.0, extreme_val, t)
		else:
			var sal: float = _get_trait_strength(entity, tid)
			influence = lerp(1.0, extreme_val, sal)
		final_weight *= influence
	return clamp(final_weight, 0.1, 3.0)


static func _calc_violation_stress(entity: RefCounted, action: String) -> float:
	var mappings: Array = _violation_map.get(action, [])
	if mappings.is_empty():
		return 0.0
	var total: float = 0.0
	for i in range(mappings.size()):
		var m: Dictionary = mappings[i]
		var tid: String = str(m.get("trait_id", ""))
		var base_stress: float = float(m.get("base_stress", 0.0))
		var alpha: float = float(m.get("alpha", VIOLATION_ALPHA))
		var sal: float = _get_trait_strength(entity, tid)
		total += base_stress * pow(sal, alpha)
	return total


static func _calc_emotion_baseline(entity: RefCounted, emotion: String) -> float:
	var baseline_map = _emotion_map.get("baseline", {})
	if not (baseline_map is Dictionary):
		return 0.0
	var mappings: Array = baseline_map.get(emotion, [])
	if mappings.is_empty():
		return 0.0
	if not _has_property(entity, "personality"):
		return 0.0
	var pd: RefCounted = entity.get("personality")
	if pd == null:
		return 0.0

	var total: float = 0.0
	for i in range(mappings.size()):
		var m: Dictionary = mappings[i]
		var max_offset: float = float(m.get("max_offset", m.get("extreme_mult", 0.0)))
		var facet: String = str(m.get("facet", ""))
		if facet != "":
			var direction: String = str(m.get("direction", "high"))
			var extreme_dir: float = 1.0 if direction == "high" else 0.0
			var fval: float = _get_facet_value(pd, facet)
			var extremeness: float = clamp(_inverse_lerp(0.5, extreme_dir, fval), 0.0, 1.0)
			total += max_offset * extremeness
		else:
			var tid: String = str(m.get("trait_id", ""))
			var sal: float = _get_trait_strength(entity, tid)
			total += max_offset * sal
	return total


static func _calc_emotion_sensitivity(entity: RefCounted, emotion: String) -> float:
	var sens_map = _emotion_map.get("sensitivity", {})
	if not (sens_map is Dictionary):
		return 1.0
	var mappings: Array = sens_map.get(emotion, [])
	if mappings.is_empty():
		return 1.0
	var total: float = 1.0
	for i in range(mappings.size()):
		var m: Dictionary = mappings[i]
		var tid: String = str(m.get("trait_id", ""))
		var extreme_mult: float = float(m.get("extreme_mult", 1.0))
		var sal: float = _get_trait_strength(entity, tid)
		var mult: float = lerp(1.0, extreme_mult, sal)
		total *= mult
	return clamp(total, 0.2, 3.0)


static func _calc_generic_mult(entity: RefCounted, modifier_key: String, _key: String) -> float:
	var mult_map = _emotion_map.get("mult", {})
	if not (mult_map is Dictionary):
		return 1.0
	var mappings: Array = mult_map.get(modifier_key, [])
	if mappings.is_empty():
		return 1.0
	var total: float = 1.0
	for i in range(mappings.size()):
		var m: Dictionary = mappings[i]
		var tid: String = str(m.get("trait_id", ""))
		var extreme_mult: float = float(m.get("extreme_mult", 1.0))
		var sal: float = _get_trait_strength(entity, tid)
		total *= lerp(1.0, extreme_mult, sal)
	return clamp(total, 0.2, 3.0)


## Backward-compatible evaluate_traits wrapper.
static func evaluate_traits(entity: RefCounted) -> void:
	update_trait_strengths(entity)


## Backward-compatible behavior weight.
static func get_behavior_weight(entity: RefCounted, action: String) -> float:
	return get_effect_value(entity, "behavior_weight", action)


## Backward-compatible violation stress.
static func get_violation_stress(entity: RefCounted, action: String) -> float:
	return get_effect_value(entity, "violation_stress", action)


## Backward-compatible emotion modifier.
static func get_emotion_modifier(entity: RefCounted, modifier_key: String) -> float:
	return get_effect_value(entity, "emotion_sensitivity", modifier_key)


## Get trait definition by ID.
static func get_trait_definition(trait_id: String) -> Dictionary:
	_ensure_loaded()
	return _trait_index.get(trait_id, {})


## Return all known behavior action keys from behavior_mappings.json.
static func get_known_behavior_actions() -> Array:
	_ensure_loaded()
	return _behavior_map.keys()


## Return all known emotion baseline keys from emotion_mappings.json.
static func get_known_emotion_baselines() -> Array:
	_ensure_loaded()
	var baseline_map = _emotion_map.get("baseline", {})
	if baseline_map is Dictionary:
		return baseline_map.keys()
	return []


## Get valence for a trait (positive/negative/neutral).
static func get_trait_sentiment(trait_id: String) -> String:
	return get_trait_definition(trait_id).get("valence", "neutral")


## Backward-compatible display filtering for ID arrays.
static func filter_display_traits(all_trait_ids: Array) -> Array:
	_ensure_loaded()
	var result: Array = []
	for i in range(mini(all_trait_ids.size(), TOP_K)):
		result.append(str(all_trait_ids[i]))
	return result


## Backward-compatible trait check for personality generators.
static func check_traits(pd: RefCounted) -> Array:
	_ensure_loaded()
	var ids: Array = []
	if pd == null:
		return ids
	for i in range(_trait_defs.size()):
		var def: Dictionary = _trait_defs[i]
		var category: String = str(def.get("category", ""))
		var tid: String = str(def.get("id", ""))
		if tid == "":
			continue
		var sal: float = 0.0
		if category == "facet":
			var facet: String = str(def.get("facet", ""))
			var fval: float = _get_facet_value(pd, facet)
			var direction: String = str(def.get("direction", "high"))
			var center: float = float(def.get("salience_center", def.get("threshold", 0.5)))
			var width: float = float(def.get("salience_width", 0.12))
			sal = _calc_facet_salience(fval, direction, center, width)
		else:
			sal = _calc_composite_salience(pd, def)
		if sal >= MIN_DISPLAY_SALIENCE:
			ids.append(tid)
	return ids
