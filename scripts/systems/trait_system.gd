extends RefCounted

## Discrete trait emergence and effects system.
## Use preload("res://scripts/systems/trait_system.gd") for access.

static var _trait_definitions: Array = []
static var _trait_index: Dictionary = {}  # id -> trait definition Dictionary
static var _loaded: bool = false


static func _ensure_loaded() -> void:
	if _loaded:
		return
	var file = FileAccess.open("res://data/species/human/personality/trait_definitions.json", FileAccess.READ)
	if file == null:
		push_warning("[TraitSystem] Cannot load trait_definitions.json")
		_loaded = true
		return

	var json = JSON.new()
	if json.parse(file.get_as_text()) != OK:
		push_warning("[TraitSystem] Invalid trait_definitions.json")
		_loaded = true
		return

	var raw_traits: Array = []
	if json.data is Array:
		raw_traits = json.data
	else:
		push_warning("[TraitSystem] trait_definitions.json root is not an Array")
		_loaded = true
		return

	_trait_definitions = []
	_trait_index = {}
	for i in range(raw_traits.size()):
		var entry: Dictionary = raw_traits[i]
		if entry.has("comment") and not entry.has("id"):
			continue
		_trait_definitions.append(entry)
		var tid: String = entry.get("id", "")
		if tid != "":
			_trait_index[tid] = entry

	_loaded = true
	print("[TraitSystem] Loaded %d trait definitions" % _trait_definitions.size())


## Check which traits are active for a given PersonalityData.
## Returns Array of trait ID strings (all matching, before display filtering).
static func check_traits(pd: RefCounted) -> Array:
	_ensure_loaded()
	var traits: Array = []
	for i in range(_trait_definitions.size()):
		var tdef: Dictionary = _trait_definitions[i]
		var cond: Dictionary = tdef.get("condition", {})
		if _evaluate_condition(cond, pd):
			traits.append(tdef.get("id", ""))
	return traits


## Evaluate a trait condition against PersonalityData.
## Supports single conditions (facet/axis) and composite conditions ("all" array).
static func _evaluate_condition(condition: Dictionary, pd: RefCounted) -> bool:
	if condition.has("all"):
		var subs: Array = condition.get("all", [])
		for i in range(subs.size()):
			var sub: Dictionary = subs[i]
			if not _evaluate_single(sub, pd):
				return false
		return true
	return _evaluate_single(condition, pd)


## Evaluate a single facet/axis condition.
## "facet" key is used for both axes ("H") and facets ("H_sincerity").
static func _evaluate_single(cond: Dictionary, pd: RefCounted) -> bool:
	var facet_key: String = cond.get("facet", "")
	var value: float = 0.5
	if "_" in facet_key:
		value = float(pd.facets.get(facet_key, 0.5))
	else:
		value = float(pd.axes.get(facet_key, 0.5))

	var threshold: float = float(cond.get("threshold", 0.5))
	var direction: String = cond.get("direction", "")
	if direction == "high":
		return value >= threshold
	if direction == "low":
		return value <= threshold
	return false


## Evaluate entity traits and cache full/display trait dictionaries on entity_data.
static func evaluate_traits(entity: RefCounted) -> void:
	_ensure_loaded()
	if entity == null or entity.personality == null:
		return

	var pd: RefCounted = entity.personality
	var matched: Array = []
	for i in range(_trait_definitions.size()):
		var tdef: Dictionary = _trait_definitions[i]
		var cond: Dictionary = tdef.get("condition", {})
		if _evaluate_condition(cond, pd):
			matched.append(tdef)

	var all_traits: Array = _apply_axis_cap(matched, pd)
	var display_traits: Array = _sort_and_cap_display(all_traits)

	entity.active_traits = all_traits
	entity.display_traits = display_traits
	entity.traits_dirty = false

	var ids: Array = []
	for i in range(all_traits.size()):
		var t: Dictionary = all_traits[i]
		ids.append(t.get("id", ""))
	entity.personality.active_traits = ids


## Apply per-axis cap: max 2 facet traits per HEXACO axis.
## Composite traits bypass this cap.
static func _apply_axis_cap(traits: Array, pd: RefCounted) -> Array:
	var axis_traits: Dictionary = {}  # axis_letter -> Array of {trait, extremeness}
	var non_facet: Array = []

	for i in range(traits.size()):
		var t: Dictionary = traits[i]
		var cond: Dictionary = t.get("condition", {})
		if cond.has("all"):
			non_facet.append(t)
			continue

		var facet_key: String = cond.get("facet", "")
		if "_" not in facet_key:
			non_facet.append(t)
			continue

		var axis: String = facet_key.substr(0, 1) if facet_key.length() > 0 else ""
		if not axis_traits.has(axis):
			axis_traits[axis] = []

		var value: float = float(pd.facets.get(facet_key, pd.axes.get(facet_key, 0.5)))
		var thr: float = float(cond.get("threshold", 0.85))
		var extremeness: float = absf(value - thr)
		axis_traits[axis].append({"trait": t, "extremeness": extremeness})

	var result: Array = non_facet.duplicate()
	var axis_keys: Array = axis_traits.keys()
	for i in range(axis_keys.size()):
		var axis_key: String = axis_keys[i]
		var arr: Array = axis_traits[axis_key]
		arr.sort_custom(func(a: Dictionary, b: Dictionary): return float(a.get("extremeness", 0.0)) > float(b.get("extremeness", 0.0)))
		for j in range(mini(arr.size(), 2)):
			var item: Dictionary = arr[j]
			result.append(item.get("trait", {}))

	return result


## Sort traits for display priority and cap to 5.
## Priority: Dark > Composite > Facet.
static func _sort_and_cap_display(traits: Array) -> Array:
	var scored: Array = []
	for i in range(traits.size()):
		var t: Dictionary = traits[i]
		var priority: int = 0
		var tid: String = t.get("id", "")
		var cond: Dictionary = t.get("condition", {})
		if tid.begins_with("d_"):
			priority = 100
		elif cond.has("all"):
			var all_cond: Array = cond.get("all", [])
			priority = 50 + all_cond.size() * 10
		else:
			priority = 10
		scored.append({"trait": t, "priority": priority})

	scored.sort_custom(func(a: Dictionary, b: Dictionary): return int(a.get("priority", 0)) > int(b.get("priority", 0)))

	var composite_facets: Dictionary = {}
	var display: Array = []
	for i in range(scored.size()):
		var s: Dictionary = scored[i]
		var t: Dictionary = s.get("trait", {})
		var cond: Dictionary = t.get("condition", {})
		if cond.has("all"):
			var subs: Array = cond.get("all", [])
			for j in range(subs.size()):
				var sub: Dictionary = subs[j]
				composite_facets[sub.get("facet", "")] = true
			display.append(t)
		else:
			var facet_key: String = cond.get("facet", "")
			if not composite_facets.has(facet_key):
				display.append(t)
		if display.size() >= 5:
			break

	return display


## Backward-compatible display filtering for ID arrays.
## Returns sorted/capped trait IDs.
static func filter_display_traits(all_trait_ids: Array) -> Array:
	_ensure_loaded()
	var defs: Array = []
	for i in range(all_trait_ids.size()):
		var tid: String = str(all_trait_ids[i])
		var tdef: Dictionary = _trait_index.get(tid, {})
		if not tdef.is_empty():
			defs.append(tdef)

	var sorted_defs: Array = _sort_and_cap_display(defs)

	var ids: Array = []
	for i in range(sorted_defs.size()):
		var t: Dictionary = sorted_defs[i]
		ids.append(t.get("id", ""))
	return ids


## Get combined behavior_weight multiplier for an action (multiplicative stacking).
static func get_behavior_weight(entity: RefCounted, action: String) -> float:
	var mult: float = 1.0
	for i in range(entity.active_traits.size()):
		var t: Dictionary = entity.active_traits[i]
		var effects: Dictionary = t.get("effects", {})
		var bw: Dictionary = effects.get("behavior_weights", {})
		if bw.has(action):
			mult *= float(bw[action])
	return mult


## Get combined emotion_modifier (additive stacking around 1.0 base).
static func get_emotion_modifier(entity: RefCounted, modifier_key: String) -> float:
	var total: float = 0.0
	for i in range(entity.active_traits.size()):
		var t: Dictionary = entity.active_traits[i]
		var effects: Dictionary = t.get("effects", {})
		var em: Dictionary = effects.get("emotion_modifiers", {})
		if em.has(modifier_key):
			total += float(em[modifier_key]) - 1.0
	return 1.0 + total


## Get total violation_stress for an action.
static func get_violation_stress(entity: RefCounted, action: String) -> float:
	var total: float = 0.0
	for i in range(entity.active_traits.size()):
		var t: Dictionary = entity.active_traits[i]
		var effects: Dictionary = t.get("effects", {})
		var sm: Dictionary = effects.get("stress_modifiers", {})
		var vs: Dictionary = sm.get("violation_stress", {})
		if vs.has(action):
			total += float(vs[action])
	return total


## Get multiplier from any effects category (multiplicative stacking).
static func get_effect_mult(entity: RefCounted, category: String, key: String) -> float:
	var mult: float = 1.0
	for i in range(entity.active_traits.size()):
		var t: Dictionary = entity.active_traits[i]
		var effects: Dictionary = t.get("effects", {})
		var cat: Dictionary = effects.get(category, {})
		if cat.has(key):
			mult *= float(cat[key])
	return mult


## Called when entity performs an action — applies violation stress.
## Phase C1 will call this from behavior_system.
static func on_action_performed(entity: RefCounted, action: String) -> void:
	var stress_amount: float = get_violation_stress(entity, action)
	if stress_amount > 0.0:
		entity.emotions["stress"] = float(entity.emotions.get("stress", 0.0)) + stress_amount
		var guilt_sens: float = get_emotion_modifier(entity, "guilt_sensitivity")
		if entity.emotion_data != null:
			entity.emotion_data.add_impulse("disgust", stress_amount * 0.3 * guilt_sens)


## Backward compatibility helper.
## Returns flattened combined behavior_weights across trait IDs.
static func get_trait_effects(trait_ids: Array) -> Dictionary:
	_ensure_loaded()
	var combined: Dictionary = {}
	for i in range(trait_ids.size()):
		var tid: String = str(trait_ids[i])
		var tdef: Dictionary = _trait_index.get(tid, {})
		if tdef.is_empty():
			continue
		var effects: Dictionary = tdef.get("effects", {})
		var behavior_weights: Dictionary = effects.get("behavior_weights", {})
		var keys: Array = behavior_weights.keys()
		for j in range(keys.size()):
			var key: String = keys[j]
			var value: float = float(behavior_weights.get(key, 1.0))
			if combined.has(key):
				combined[key] = float(combined[key]) * value
			else:
				combined[key] = value
	return combined


## Get trait definition by ID (for UI display).
static func get_trait_definition(trait_id: String) -> Dictionary:
	_ensure_loaded()
	return _trait_index.get(trait_id, {})


## Get valence for a trait ("positive", "negative", "neutral").
static func get_trait_sentiment(trait_id: String) -> String:
	var tdef: Dictionary = get_trait_definition(trait_id)
	return tdef.get("valence", "neutral")
