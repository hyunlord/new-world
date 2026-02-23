extends Node

## Species data loader singleton (Autoload).
## Loads species-specific JSON data at startup, exposes as Dictionaries.
## Engine code reads from these instead of hardcoded constants.
## No class_name — Autoload accessed by name (SpeciesManager).

## Current species ID
var species_id: String = "human"

## Loaded data dictionaries
var species_config: Dictionary = {}
var personality_distribution: Dictionary = {}
var emotion_definition: Dictionary = {}
var dyad_definition: Dictionary = {}
var decay_parameters: Dictionary = {}
var siler_parameters: Dictionary = {}
var culture_configs: Dictionary = {}  # culture_id -> Dictionary


func _ready() -> void:
	load_species(species_id)


## Load all data files for a species.
## Called once at startup; cached in memory.
func load_species(sid: String) -> void:
	species_id = sid
	var base: String = "res://data/species/" + sid + "/"
	species_config = _load_json(base + "species_definition.json")
	personality_distribution = _load_json(base + "personality/distribution.json")
	emotion_definition = _load_json(base + "emotions/emotion_definition.json")
	dyad_definition = _load_json(base + "emotions/dyad_definition.json")
	decay_parameters = _load_json(base + "emotions/decay_parameters.json")
	siler_parameters = _load_json(base + "mortality/siler_parameters.json")
	_load_cultures(base + "cultures/")


func _load_cultures(dir_path: String) -> void:
	culture_configs.clear()
	var cultures = species_config.get("available_cultures", [])
	for i in range(cultures.size()):
		var culture_id: String = str(cultures[i])
		var path: String = dir_path + culture_id + ".json"
		var data: Dictionary = _load_json(path)
		if not data.is_empty():
			culture_configs[culture_id] = data


## Get personality z-score shift for a culture + axis.
## Returns 0.0 if culture or axis not found.
func get_culture_shift(culture_id: String, axis_id: String) -> float:
	var culture = culture_configs.get(culture_id, {})
	var shifts = culture.get("personality_shift", {})
	return float(shifts.get(axis_id, 0.0))


## Get emotion sensitivity multiplier for a culture + emotion.
## Returns 1.0 (no modification) if culture or emotion not found.
func get_culture_emotion_modifier(culture_id: String, emotion_id: String) -> float:
	var culture = culture_configs.get(culture_id, {})
	var mods = culture.get("emotion_modifiers", {})
	return float(mods.get(emotion_id, 1.0))


func _load_json(path: String) -> Dictionary:
	if not FileAccess.file_exists(path):
		push_warning("[SpeciesManager] JSON not found: %s" % path)
		return {}
	var file: FileAccess = FileAccess.open(path, FileAccess.READ)
	if file == null:
		push_warning("[SpeciesManager] Cannot open: %s" % path)
		return {}
	var json: JSON = JSON.new()
	var err: int = json.parse(file.get_as_text())
	if err != OK:
		push_warning("[SpeciesManager] Parse error in %s: %s" % [path, json.get_error_message()])
		return {}
	if json.data is Dictionary:
		return json.data
	return {}
