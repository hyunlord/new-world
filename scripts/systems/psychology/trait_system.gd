extends "res://scripts/core/simulation/simulation_system.gd"

## TraitSystem v3 — Binary threshold personality trait evaluation.
## v3 traits: HEXACO extreme-only (top/bottom 1-3%) → binary ON/OFF.
## v2 facet saliences retained for backward-compatible effect queries.

const PersonalityData = preload("res://scripts/core/entity/personality_data.gd")
const EntityData = preload("res://scripts/core/entity/entity_data.gd")

# ── v2 backward compat constants ──
const TOP_K: int = 5
const MIN_DISPLAY_SALIENCE: float = 0.10
const MAX_PER_AXIS: int = 2
const MAX_DARK_DISPLAY: int = 2
const VIOLATION_ALPHA: float = 1.2
const EPS: float = 0.0001

# ── Paths ──
const TRAIT_DEFS_V3_PATH: String = "res://data/personality/trait_defs_v3.json"
const TRAIT_DEFS_V2_PATH: String = "res://data/personality/trait_defs_v2.json"
const BEHAVIOR_MAP_PATH: String = "res://data/personality/behavior_mappings.json"
const EMOTION_MAP_PATH: String = "res://data/personality/emotion_mappings.json"
const VIOLATION_MAP_PATH: String = "res://data/personality/violation_mappings.json"

const FACET_ALIASES: Dictionary = {
	"O_aesthetic_appreciation": "O_aesthetic",
}

# ── v3 data ──
static var _v3_defs: Array = []
static var _v3_index: Dictionary = {}

# ── v2 data (backward compat for effect system) ──
static var _v2_defs: Array = []
static var _v2_index: Dictionary = {}
static var _mutex_pairs: Dictionary = {}

# ── Combined index (v3 overrides v2 on collision) ──
static var _trait_index: Dictionary = {}

# ── Effect mapping data ──
static var _behavior_map: Dictionary = {}
static var _emotion_map: Dictionary = {}
static var _violation_map: Dictionary = {}

static var _loaded: bool = false
static var _effects_cache: Dictionary = {}


static func _ensure_loaded() -> void:
	if _loaded:
		return

	_v3_defs = []
	_v3_index = {}
	_v2_defs = []
	_v2_index = {}
	_trait_index = {}
	_mutex_pairs = {}
	_behavior_map = {}
	_emotion_map = {}
	_violation_map = {}
	_effects_cache = {}

	# ── Load v3 trait definitions ──
	var v3_data = _load_json(TRAIT_DEFS_V3_PATH)
	if v3_data is Array:
		var raw: Array = v3_data
		for i in range(raw.size()):
			var def: Dictionary = raw[i]
			var tid: String = str(def.get("id", ""))
			if tid == "":
				continue
			_v3_defs.append(def)
			_v3_index[tid] = def
			_trait_index[tid] = def

	# ── Load v2 trait definitions (backward compat for effect system) ──
	var v2_data = _load_json(TRAIT_DEFS_V2_PATH)
	if v2_data is Array:
		var raw: Array = v2_data
		for i in range(raw.size()):
			var def: Dictionary = raw[i]
			var tid: String = str(def.get("id", ""))
			if tid == "":
				continue
			_v2_defs.append(def)
			_v2_index[tid] = def
			if not _trait_index.has(tid):
				_trait_index[tid] = def

	# ── Load effect mapping files ──
	var behavior_data = _load_json(BEHAVIOR_MAP_PATH)
	if behavior_data is Dictionary:
		_behavior_map = behavior_data

	var emotion_data = _load_json(EMOTION_MAP_PATH)
	if emotion_data is Dictionary:
		_emotion_map = emotion_data

	var violation_data = _load_json(VIOLATION_MAP_PATH)
	if violation_data is Dictionary:
		_violation_map = violation_data

	_build_mutex_pairs()

	_loaded = true
	print("[TraitSystem] v3=%d v2=%d behavior=%d emotion=%d violation=%d" % [
		_v3_defs.size(),
		_v2_defs.size(),
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
	for i in range(_v2_defs.size()):
		var def: Dictionary = _v2_defs[i]
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


# ══════════════════════════════════════════════════════
# Utility helpers
# ══════════════════════════════════════════════════════

static func _has_property(target: Object, property_name: String) -> bool:
	if target == null:
		return false
	var props: Array = target.get_property_list()
	for i in range(props.size()):
		var p: Dictionary = props[i]
		if str(p.get("name", "")) == property_name:
			return true
	return false


static func _get_facet_value(pd: RefCounted, facet_key: String, entity: RefCounted = null) -> float:
	if pd == null:
		return 0.5
	if "_" in facet_key:
		if pd.facets.has(facet_key):
			return float(pd.facets.get(facet_key, 0.5))
		var alias_key: String = str(FACET_ALIASES.get(facet_key, ""))
		if alias_key != "" and pd.facets.has(alias_key):
			return float(pd.facets.get(alias_key, 0.5))
		return 0.5
	if pd.axes.has(facet_key):
		return float(pd.axes[facet_key])
	if entity != null:
		return StatQuery.get_normalized(entity, StringName("HEXACO_" + facet_key))
	return 0.5


# ══════════════════════════════════════════════════════
# v3 Binary Evaluation
# ══════════════════════════════════════════════════════

## Evaluate a single v3 trait. Returns true if ALL conditions pass.
static func _evaluate_v3_conditions(entity: RefCounted, pd: RefCounted, def: Dictionary) -> bool:
	var acq: Dictionary = def.get("acquisition", {})
	var conditions: Array = acq.get("conditions", [])
	if conditions.is_empty():
		return false

	for i in range(conditions.size()):
		var cond: Dictionary = conditions[i]
		var source: String = str(cond.get("source", ""))
		var direction: String = str(cond.get("direction", "high"))
		var threshold: float = float(cond.get("threshold", 0.5))

		if source == "hexaco":
			var axis: String = str(cond.get("axis", ""))
			var val: float = _get_facet_value(pd, axis, entity)
			if direction == "high":
				if val < threshold:
					return false
			else:
				if val > threshold:
					return false

		elif source == "value":
			if entity == null:
				return false
			if not _has_property(entity, "values"):
				return false
			var raw = entity.get("values")
			if not (raw is Dictionary):
				return false
			var key: String = str(cond.get("key", ""))
			var val: float = float(raw.get(key, 0.5))
			if direction == "high":
				if val < threshold:
					return false
			else:
				if val > threshold:
					return false

		elif source == "body":
			if entity == null:
				return false
			if not _has_property(entity, "body"):
				return false
			var body_obj = entity.get("body")
			if body_obj == null:
				return false
			if not _has_property(body_obj, "realized"):
				return false
			var realized = body_obj.get("realized")
			if not (realized is Dictionary):
				return false
			var axis: String = str(cond.get("axis", ""))
			var raw_val: float = float(realized.get(axis, 5000))
			var val: float = raw_val / 10000.0
			if direction == "high":
				if val < threshold:
					return false
			else:
				if val > threshold:
					return false

		elif source == "intelligence":
			if entity == null:
				return false
			if not _has_property(entity, "intelligences"):
				return false
			var intels = entity.get("intelligences")
			if not (intels is Dictionary):
				return false
			var key: String = str(cond.get("key", ""))
			var val: float = float(intels.get(key, 0.5))
			if direction == "high":
				if val < threshold:
					return false
			else:
				if val > threshold:
					return false
		else:
			return false

	return true


## Remove weaker trait when incompatibles both qualify.
static func _resolve_incompatibles(strengths: Dictionary) -> void:
	var to_remove: Dictionary = {}
	for i in range(_v3_defs.size()):
		var def: Dictionary = _v3_defs[i]
		var tid: String = str(def.get("id", ""))
		if float(strengths.get(tid, 0.0)) < 0.5:
			continue
		if to_remove.has(tid):
			continue
		var incompat: Array = def.get("incompatible_with", [])
		var my_conds: int = def.get("acquisition", {}).get("conditions", []).size()
		for j in range(incompat.size()):
			var other_id: String = str(incompat[j])
			if float(strengths.get(other_id, 0.0)) < 0.5:
				continue
			if to_remove.has(other_id):
				continue
			var other_def: Dictionary = _v3_index.get(other_id, {})
			var other_conds: int = other_def.get("acquisition", {}).get("conditions", []).size()
			if my_conds >= other_conds:
				to_remove[other_id] = true
			else:
				to_remove[tid] = true
				break

	var keys: Array = to_remove.keys()
	for i in range(keys.size()):
		strengths.erase(str(keys[i]))


## Build display traits from qualified v3 traits. No Top-K limit.
## Also evaluates synergy traits: if all required_traits are present, synergy is added.
static func _build_v3_display(strengths: Dictionary) -> Array:
	var result: Array = []
	var active_ids: Dictionary = {}  # track active trait IDs for synergy check
	for i in range(_v3_defs.size()):
		var def: Dictionary = _v3_defs[i]
		var tid: String = str(def.get("id", ""))
		if tid == "":
			continue
		var category: String = str(def.get("category", "archetype"))
		# Skip synergy traits in first pass — evaluated below
		if category == "synergy":
			continue
		if float(strengths.get(tid, 0.0)) < 0.5:
			continue
		active_ids[tid] = true
		result.append({
			"id": tid,
			"name_key": str(def.get("name_key", "TRAIT_" + tid.to_upper() + "_NAME")),
			"salience": 1.0,
			"category": category,
		})
	# Second pass: evaluate synergy traits
	for i in range(_v3_defs.size()):
		var def: Dictionary = _v3_defs[i]
		if str(def.get("category", "")) != "synergy":
			continue
		var tid: String = str(def.get("id", ""))
		if tid == "":
			continue
		var acq: Dictionary = def.get("acquisition", {})
		var required: Array = acq.get("required_traits", [])
		if required.is_empty():
			continue
		var all_met: bool = true
		for j in range(required.size()):
			if not active_ids.has(str(required[j])):
				all_met = false
				break
		if all_met:
			result.append({
				"id": tid,
				"name_key": str(def.get("name_key", "TRAIT_" + tid.to_upper() + "_NAME")),
				"salience": 1.0,
				"category": "synergy",
			})
	return result


# ══════════════════════════════════════════════════════
# v2 Sigmoid (backward compat for effect system)
# ══════════════════════════════════════════════════════

static func _sigmoid(x: float, center: float, width: float) -> float:
	var safe_width: float = maxf(width, EPS)
	var k: float = 10.0 / safe_width
	return 1.0 / (1.0 + exp(-k * (x - center)))


static func _calc_facet_salience(facet_val: float, direction: String, center: float, width: float) -> float:
	if direction == "high":
		return _sigmoid(facet_val, center, width)
	return _sigmoid(-facet_val, -center, width)


static func _calc_composite_salience(pd: RefCounted, def: Dictionary, entity: RefCounted = null) -> float:
	var conditions: Array = def.get("conditions", [])
	if conditions.is_empty():
		return 0.0
	var product: float = 1.0
	for i in range(conditions.size()):
		var cond: Dictionary = conditions[i]
		var fval: float = _get_facet_value(pd, str(cond.get("facet", "")), entity)
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


# ══════════════════════════════════════════════════════
# Core API
# ══════════════════════════════════════════════════════

## Update trait strengths: v2 saliences (effect compat) + v3 binary evaluation (display).
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

	# ── v2 facet saliences (backward compat for effect mappings) ──
	for i in range(_v2_defs.size()):
		var def: Dictionary = _v2_defs[i]
		var category: String = str(def.get("category", ""))
		var tid: String = str(def.get("id", ""))
		if tid == "":
			continue
		if category == "facet":
			var facet: String = str(def.get("facet", ""))
			if facet == "":
				continue
			var fval: float = _get_facet_value(pd, facet, entity)
			var direction: String = str(def.get("direction", "high"))
			var center: float = float(def.get("salience_center", def.get("threshold", 0.5)))
			var width: float = float(def.get("salience_width", 0.12))
			strengths[tid] = _calc_facet_salience(fval, direction, center, width)
		else:
			strengths[tid] = _calc_composite_salience(pd, def, entity)

	# v2 mutex suppression
	var mutex_keys: Array = _mutex_pairs.keys()
	for i in range(mutex_keys.size()):
		var key: String = str(mutex_keys[i])
		var pair_data = _mutex_pairs.get(key, [])
		if not (pair_data is Array) or pair_data.size() < 2:
			continue
		var hi_id: String = str(pair_data[0])
		var lo_id: String = str(pair_data[1])
		if hi_id == "" or lo_id == "":
			continue
		var raw_hi: float = float(strengths.get(hi_id, 0.0))
		var raw_lo: float = float(strengths.get(lo_id, 0.0))
		if raw_hi >= raw_lo:
			strengths[lo_id] = 0.0
		else:
			strengths[hi_id] = 0.0

	# ── v3 binary evaluation (threshold-based: archetype, shadow, radiance, corpus, nous) ──
	for i in range(_v3_defs.size()):
		var def: Dictionary = _v3_defs[i]
		var tid: String = str(def.get("id", ""))
		if tid == "":
			continue
		var acq: Dictionary = def.get("acquisition", {})
		var conditions: Array = acq.get("conditions", [])
		if conditions.is_empty():
			continue
		if _evaluate_v3_conditions(entity, pd, def):
			strengths[tid] = 1.0

	# ── Merge granted traits (event-based: awakened, bond, mastery, fate) ──
	if _has_property(entity, "granted_traits"):
		var granted = entity.get("granted_traits")
		if granted is Dictionary:
			var gkeys: Array = granted.keys()
			for i in range(gkeys.size()):
				var tid: String = str(gkeys[i])
				if _v3_index.has(tid):
					strengths[tid] = 1.0

	# Resolve v3 incompatibles
	_resolve_incompatibles(strengths)

	if _has_property(entity, "trait_strengths"):
		entity.set("trait_strengths", strengths)
	if _has_property(entity, "display_traits"):
		entity.set("display_traits", _build_v3_display(strengths))
	if _has_property(entity, "traits_dirty"):
		entity.set("traits_dirty", false)


## Select display traits. v3: returns all qualified traits (variable count).
static func get_display_traits(entity: RefCounted, _k: int = TOP_K) -> Array:
	_ensure_loaded()
	if entity == null:
		return []
	if _has_property(entity, "display_traits"):
		var raw = entity.get("display_traits")
		if raw is Array:
			return raw
	var strengths: Dictionary = {}
	if _has_property(entity, "trait_strengths"):
		var raw_strengths = entity.get("trait_strengths")
		if raw_strengths is Dictionary:
			strengths = raw_strengths
	if strengths.is_empty():
		return []
	return _build_v3_display(strengths)


# ══════════════════════════════════════════════════════
# Effect System (v2 backward compat — reads from mapping files)
# ══════════════════════════════════════════════════════

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

	var log_sum: float = 0.0
	var active_count: int = 0
	for i in range(mappings.size()):
		var m: Dictionary = mappings[i]
		var tid: String = str(m.get("trait_id", ""))
		var extreme_val: float = float(m.get("extreme_val", 1.0))
		var source: String = str(m.get("source", "facet"))
		var influence: float = 1.0
		if source == "facet":
			var facet: String = str(m.get("facet", ""))
			var threshold: float = float(m.get("threshold", 0.5))
			var fval: float = _get_facet_value(pd, facet, entity)
			var t: float = clamp(_inverse_lerp(0.5, threshold, fval), 0.0, 1.0)
			influence = lerp(1.0, extreme_val, t)
		else:
			var sal: float = _get_trait_strength(entity, tid)
			influence = lerp(1.0, extreme_val, sal)
		if abs(influence - 1.0) > 0.01:
			log_sum += log(maxf(influence, 0.001))
			active_count += 1
	if active_count == 0:
		return 1.0
	return clamp(exp(log_sum / float(active_count)), 0.1, 3.0)


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
			var fval: float = _get_facet_value(pd, facet, entity)
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
	var log_sum: float = 0.0
	var active_count: int = 0
	for i in range(mappings.size()):
		var m: Dictionary = mappings[i]
		var tid: String = str(m.get("trait_id", ""))
		var extreme_mult: float = float(m.get("extreme_mult", 1.0))
		var sal: float = _get_trait_strength(entity, tid)
		var mult: float = lerp(1.0, extreme_mult, sal)
		if abs(mult - 1.0) > 0.01:
			log_sum += log(maxf(mult, 0.001))
			active_count += 1
	if active_count == 0:
		return 1.0
	return clamp(exp(log_sum / float(active_count)), 0.2, 3.0)


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


# ══════════════════════════════════════════════════════
# Public API & Backward Compat
# ══════════════════════════════════════════════════════

## Summarize v3 effects array from a trait definition into display-ready buckets.
static func get_v3_trait_effects_summary(trait_def: Dictionary) -> Dictionary:
	var result: Dictionary = {
		"skill_mults": {},
		"blocked": [],
		"immune": [],
		"derived_mults": {},
		"emotion_caps": {},
		"has_aura": false,
		"has_on_event": false,
		"raw_count": 0
	}
	var effects: Array = trait_def.get("effects", [])
	result["raw_count"] = effects.size()
	for i in range(effects.size()):
		var e: Dictionary = effects[i]
		if e.has("on_event"):
			result["has_on_event"] = true
			continue
		var system: String = str(e.get("system", ""))
		var op: String = str(e.get("op", ""))
		var target = e.get("target", "")
		var value = e.get("value", 1.0)
		match system:
			"skill":
				if op == "mult":
					var targets: Array = target if target is Array else [target]
					for t in targets:
						result["skill_mults"][str(t)] = float(value)
			"behavior":
				if op == "block":
					var targets: Array = target if target is Array else [target]
					for t in targets:
						result["blocked"].append(str(t))
				elif op == "inject":
					result["blocked"].append("+" + str(target))
			"stress":
				if op == "immune":
					result["immune"].append(str(target))
			"derived":
				if op == "mult":
					result["derived_mults"][str(target)] = float(value)
			"emotion":
				if op in ["max", "min", "set"]:
					result["emotion_caps"][str(target)] = float(value)
			"aura":
				result["has_aura"] = true
	return result


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


## Get trait definition by ID (searches v3 first, then v2).
static func get_trait_definition(trait_id: String) -> Dictionary:
	_ensure_loaded()
	return _trait_index.get(trait_id, {})


## Return all known behavior action keys.
static func get_known_behavior_actions() -> Array:
	_ensure_loaded()
	return _behavior_map.keys()


## Return all known emotion baseline keys.
static func get_known_emotion_baselines() -> Array:
	_ensure_loaded()
	var baseline_map = _emotion_map.get("baseline", {})
	if baseline_map is Dictionary:
		return baseline_map.keys()
	return []


## Get valence for a trait. v3 derives from category; v2 uses explicit field.
static func get_trait_sentiment(trait_id: String) -> String:
	var def: Dictionary = get_trait_definition(trait_id)
	if def.is_empty():
		return "neutral"
	var cat: String = str(def.get("category", ""))
	if cat == "shadow":
		return "negative"
	if cat == "radiance":
		return "positive"
	return str(def.get("valence", "neutral"))


## Build per-trait display effects by inverting mapping files.
static func get_trait_display_effects(trait_id: String) -> Dictionary:
	_ensure_loaded()
	if _effects_cache.has(trait_id):
		return _effects_cache[trait_id]

	var effects: Dictionary = {
		"behavior_weights": {},
		"emotion_modifiers": {},
		"violation_stress": {}
	}

	var bw_keys: Array = _behavior_map.keys()
	for i in range(bw_keys.size()):
		var action: String = str(bw_keys[i])
		var mappings: Array = _behavior_map[action]
		for j in range(mappings.size()):
			var m: Dictionary = mappings[j]
			if str(m.get("trait_id", "")) == trait_id:
				effects["behavior_weights"][action] = float(m.get("extreme_val", 1.0))
				break

	var sens_map = _emotion_map.get("sensitivity", {})
	if sens_map is Dictionary:
		var sens_keys: Array = sens_map.keys()
		for i in range(sens_keys.size()):
			var emotion: String = str(sens_keys[i])
			var mappings: Array = sens_map[emotion]
			for j in range(mappings.size()):
				var m: Dictionary = mappings[j]
				if str(m.get("trait_id", "")) == trait_id:
					effects["emotion_modifiers"][emotion + "_sensitivity"] = float(m.get("extreme_mult", 1.0))
					break

	var base_map = _emotion_map.get("baseline", {})
	if base_map is Dictionary:
		var base_keys: Array = base_map.keys()
		for i in range(base_keys.size()):
			var emotion: String = str(base_keys[i])
			var mappings: Array = base_map[emotion]
			for j in range(mappings.size()):
				var m: Dictionary = mappings[j]
				if str(m.get("trait_id", "")) == trait_id:
					effects["emotion_modifiers"][emotion + "_baseline"] = float(m.get("extreme_mult", 0.0))
					break

	var mult_map = _emotion_map.get("mult", {})
	if mult_map is Dictionary:
		var mult_keys: Array = mult_map.keys()
		for i in range(mult_keys.size()):
			var mk: String = str(mult_keys[i])
			var mappings: Array = mult_map[mk]
			for j in range(mappings.size()):
				var m: Dictionary = mappings[j]
				if str(m.get("trait_id", "")) == trait_id:
					effects["emotion_modifiers"][mk + "_mult"] = float(m.get("extreme_mult", 1.0))
					break

	var vio_keys: Array = _violation_map.keys()
	for i in range(vio_keys.size()):
		var action: String = str(vio_keys[i])
		var mappings: Array = _violation_map[action]
		for j in range(mappings.size()):
			var m: Dictionary = mappings[j]
			if str(m.get("trait_id", "")) == trait_id:
				effects["violation_stress"][action] = float(m.get("base_stress", 0.0))
				break

	_effects_cache[trait_id] = effects
	return effects


## Backward-compatible display filtering for ID arrays.
static func filter_display_traits(all_trait_ids: Array) -> Array:
	_ensure_loaded()
	var result: Array = []
	for i in range(all_trait_ids.size()):
		result.append(str(all_trait_ids[i]))
	return result


## Check which v3 traits a personality qualifies for (used at birth/maturation).
static func check_traits(pd: RefCounted) -> Array:
	_ensure_loaded()
	var ids: Array = []
	if pd == null:
		return ids
	for i in range(_v3_defs.size()):
		var def: Dictionary = _v3_defs[i]
		var tid: String = str(def.get("id", ""))
		if tid == "":
			continue
		var acq: Dictionary = def.get("acquisition", {})
		var conditions: Array = acq.get("conditions", [])
		if conditions.is_empty():
			continue
		var all_pass: bool = true
		for j in range(conditions.size()):
			var cond: Dictionary = conditions[j]
			var source: String = str(cond.get("source", ""))
			if source != "hexaco":
				all_pass = false
				break
			var axis: String = str(cond.get("axis", ""))
			var direction: String = str(cond.get("direction", "high"))
			var threshold: float = float(cond.get("threshold", 0.5))
			var val: float = _get_facet_value(pd, axis)
			if direction == "high":
				if val < threshold:
					all_pass = false
					break
			else:
				if val > threshold:
					all_pass = false
					break
		if all_pass:
			ids.append(tid)
	return ids


# ══════════════════════════════════════════════════════
# Grant / Revoke API (for event-based traits: awakened, bond, mastery, fate)
# ══════════════════════════════════════════════════════

## Grant an event-based trait to an entity. Returns true if newly granted.
static func grant_trait(entity: RefCounted, trait_id: String) -> bool:
	_ensure_loaded()
	if entity == null:
		return false
	if not _v3_index.has(trait_id):
		push_warning("[TraitSystem] grant_trait: unknown trait_id '%s'" % trait_id)
		return false
	if not _has_property(entity, "granted_traits"):
		return false
	var granted = entity.get("granted_traits")
	if not (granted is Dictionary):
		granted = {}
	if granted.has(trait_id):
		return false
	granted[trait_id] = true
	entity.set("granted_traits", granted)
	if _has_property(entity, "traits_dirty"):
		entity.set("traits_dirty", true)
	return true


## Revoke an event-based trait from an entity. Returns true if removed.
static func revoke_trait(entity: RefCounted, trait_id: String) -> bool:
	_ensure_loaded()
	if entity == null:
		return false
	if not _has_property(entity, "granted_traits"):
		return false
	var granted = entity.get("granted_traits")
	if not (granted is Dictionary):
		return false
	if not granted.has(trait_id):
		return false
	granted.erase(trait_id)
	entity.set("granted_traits", granted)
	if _has_property(entity, "traits_dirty"):
		entity.set("traits_dirty", true)
	return true


## Check if an entity has a specific granted trait.
static func has_trait(entity: RefCounted, trait_id: String) -> bool:
	if entity == null:
		return false
	if _has_property(entity, "granted_traits"):
		var granted = entity.get("granted_traits")
		if granted is Dictionary and granted.has(trait_id):
			return true
	if _has_property(entity, "trait_strengths"):
		var strengths = entity.get("trait_strengths")
		if strengths is Dictionary:
			return float(strengths.get(trait_id, 0.0)) >= 0.5
	return false
