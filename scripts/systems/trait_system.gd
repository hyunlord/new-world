extends RefCounted

## Discrete trait emergence system with composite trait support.
## Checks personality extremes and returns active traits + combined effects.
## Supports single conditions (facet/axis threshold) and composite conditions
## ("all" array = AND logic across multiple facet/axis checks).
## Use preload("res://scripts/systems/trait_system.gd") for access.

static var _trait_definitions: Array = []
static var _trait_index: Dictionary = {}  # id -> trait definition Dictionary
static var _loaded: bool = false


static func _ensure_loaded() -> void:
	if _loaded:
		return
	var file = FileAccess.open("res://data/personality/trait_definitions.json", FileAccess.READ)
	if file == null:
		push_warning("[TraitSystem] Cannot load trait_definitions.json")
		_loaded = true
		return
	var json = JSON.new()
	if json.parse(file.get_as_text()) != OK:
		push_warning("[TraitSystem] Invalid trait_definitions.json")
		_loaded = true
		return
	var raw_traits = json.data.get("traits", [])
	# Filter out comment entries and build index
	_trait_definitions = []
	_trait_index = {}
	for i in range(raw_traits.size()):
		var entry = raw_traits[i]
		if entry.has("comment") and not entry.has("id"):
			continue  # Skip comment-only entries
		_trait_definitions.append(entry)
		var tid = entry.get("id", "")
		if tid != "":
			_trait_index[tid] = entry
	_loaded = true


## Check which traits are active for a given PersonalityData.
## Returns Array of trait ID strings (all matching, before display filtering).
static func check_traits(pd: RefCounted) -> Array:
	_ensure_loaded()
	var traits: Array = []
	for i in range(_trait_definitions.size()):
		var tdef = _trait_definitions[i]
		var cond = tdef.get("condition", {})
		if _evaluate_condition(cond, pd):
			traits.append(tdef.get("id", ""))
	return traits


## Evaluate a trait condition against PersonalityData.
## Supports single conditions (facet/axis) and composite conditions ("all" array).
static func _evaluate_condition(condition: Dictionary, pd: RefCounted) -> bool:
	if condition.has("all"):
		# Composite: ALL sub-conditions must pass (AND logic)
		var subs = condition.get("all", [])
		for i in range(subs.size()):
			if not _evaluate_single(subs[i], pd):
				return false
		return true
	else:
		# Single condition
		return _evaluate_single(condition, pd)


## Evaluate a single facet/axis condition.
static func _evaluate_single(cond: Dictionary, pd: RefCounted) -> bool:
	var value: float = 0.5
	if cond.has("facet"):
		value = pd.facets.get(cond.get("facet", ""), 0.5)
	elif cond.has("axis"):
		value = pd.axes.get(cond.get("axis", ""), 0.5)

	var threshold = float(cond.get("threshold", 0.5))
	var direction = cond.get("direction", "")
	if direction == "high":
		return value >= threshold
	elif direction == "low":
		return value <= threshold
	return false


## Filter traits for UI display.
## Composite traits suppress their component single traits.
## Returns at most max_display traits (composites prioritized).
static func filter_display_traits(all_trait_ids: Array, max_display: int = 5) -> Array:
	_ensure_loaded()
	var composites: Array = []
	var singles: Array = []
	for i in range(all_trait_ids.size()):
		var tid = all_trait_ids[i]
		var tdef = _trait_index.get(tid, {})
		var cond = tdef.get("condition", {})
		if cond.has("all"):
			composites.append(tid)
		else:
			singles.append(tid)

	# Build suppression set: single traits overlapping with composite sub-conditions
	var suppressed: Dictionary = {}
	for i in range(composites.size()):
		var cid = composites[i]
		var cdef = _trait_index.get(cid, {})
		var subs = cdef.get("condition", {}).get("all", [])
		for j in range(subs.size()):
			var sub = subs[j]
			# Check each single trait for overlap
			for k in range(singles.size()):
				var sid = singles[k]
				var sdef = _trait_index.get(sid, {})
				var scond = sdef.get("condition", {})
				if _conditions_overlap(sub, scond):
					suppressed[sid] = true

	# Build result: composites first, then non-suppressed singles
	var filtered: Array = composites.duplicate()
	for i in range(singles.size()):
		if not suppressed.has(singles[i]):
			filtered.append(singles[i])

	# Cap at max_display
	if filtered.size() > max_display:
		filtered.resize(max_display)
	return filtered


## Check if a composite sub-condition overlaps with a single trait condition.
## Overlap = same facet or same axis with same direction.
static func _conditions_overlap(sub: Dictionary, single_cond: Dictionary) -> bool:
	if sub.has("facet") and single_cond.has("facet"):
		return sub.get("facet", "") == single_cond.get("facet", "") and sub.get("direction", "") == single_cond.get("direction", "")
	if sub.has("axis") and single_cond.has("axis"):
		return sub.get("axis", "") == single_cond.get("axis", "") and sub.get("direction", "") == single_cond.get("direction", "")
	# Axis sub vs facet single: check if facet belongs to axis
	if sub.has("axis") and single_cond.has("facet"):
		var facet_key = single_cond.get("facet", "")
		var axis_key = sub.get("axis", "")
		if facet_key.begins_with(axis_key + "_") and sub.get("direction", "") == single_cond.get("direction", ""):
			return true
	return false


## Get combined effect multipliers from a list of active trait IDs.
## Effects are combined multiplicatively (multiple traits stack).
## Returns Dictionary of effect_key -> combined_multiplier.
static func get_trait_effects(trait_ids: Array) -> Dictionary:
	_ensure_loaded()
	var combined: Dictionary = {}
	for i in range(trait_ids.size()):
		var tid = trait_ids[i]
		var tdef = _trait_index.get(tid, {})
		if tdef.is_empty():
			continue
		var effects = tdef.get("effects", {})
		var effect_keys = effects.keys()
		for k in range(effect_keys.size()):
			var ek = effect_keys[k]
			var ev = float(effects[ek])
			if combined.has(ek):
				combined[ek] = combined[ek] * ev
			else:
				combined[ek] = ev
	return combined


## Get trait definition by ID (for UI display).
## Returns Dictionary with id, name_kr, name_en, sentiment, etc.
static func get_trait_definition(trait_id: String) -> Dictionary:
	_ensure_loaded()
	return _trait_index.get(trait_id, {})


## Get sentiment for a trait ("positive", "negative", "neutral").
static func get_trait_sentiment(trait_id: String) -> String:
	var tdef = get_trait_definition(trait_id)
	return tdef.get("sentiment", "neutral")
