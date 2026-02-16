extends RefCounted

## Discrete trait emergence system.
## Checks personality extremes and returns active traits + combined effects.
## Use preload("res://scripts/systems/trait_system.gd") for access.

static var _trait_definitions: Array = []
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
	_trait_definitions = json.data.get("traits", [])
	_loaded = true


## Check which traits are active for a given PersonalityData.
## Returns Array of trait ID strings.
static func check_traits(pd: RefCounted) -> Array:
	_ensure_loaded()
	var traits: Array = []
	for i in range(_trait_definitions.size()):
		var tdef = _trait_definitions[i]
		var cond = tdef.get("condition", {})
		var value: float = 0.5
		if cond.has("facet"):
			value = pd.facets.get(cond.get("facet", ""), 0.5)
		elif cond.has("axis"):
			value = pd.axes.get(cond.get("axis", ""), 0.5)

		var threshold = float(cond.get("threshold", 0.5))
		var direction = cond.get("direction", "")
		if direction == "high" and value >= threshold:
			traits.append(tdef.get("id", ""))
		elif direction == "low" and value <= threshold:
			traits.append(tdef.get("id", ""))
	return traits


## Get combined effect multipliers from a list of active trait IDs.
## Effects are combined multiplicatively (multiple traits stack).
## Returns Dictionary of effect_key -> combined_multiplier.
static func get_trait_effects(trait_ids: Array) -> Dictionary:
	_ensure_loaded()
	var combined: Dictionary = {}
	for i in range(trait_ids.size()):
		var tid = trait_ids[i]
		for j in range(_trait_definitions.size()):
			var tdef = _trait_definitions[j]
			if tdef.get("id", "") == tid:
				var effects = tdef.get("effects", {})
				var effect_keys = effects.keys()
				for k in range(effect_keys.size()):
					var ek = effect_keys[k]
					var ev = float(effects[ek])
					if combined.has(ek):
						combined[ek] = combined[ek] * ev
					else:
						combined[ek] = ev
				break
	return combined


## Get trait definition by ID (for UI display).
## Returns Dictionary with id, name_kr, name_en, sentiment, etc.
static func get_trait_definition(trait_id: String) -> Dictionary:
	_ensure_loaded()
	for i in range(_trait_definitions.size()):
		if _trait_definitions[i].get("id", "") == trait_id:
			return _trait_definitions[i]
	return {}


## Get sentiment for a trait ("positive", "negative", "neutral").
static func get_trait_sentiment(trait_id: String) -> String:
	var tdef = get_trait_definition(trait_id)
	return tdef.get("sentiment", "neutral")
