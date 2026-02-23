extends Node

var cultures: Dictionary = {}
var used_names_per_settlement: Dictionary = {}
var _rng: RandomNumberGenerator
var _entity_manager: RefCounted


func _ready() -> void:
	_load_all_cultures()


func init(rng: RandomNumberGenerator, entity_manager: RefCounted) -> void:
	_rng = rng
	_entity_manager = entity_manager
	if not SimulationBus.entity_died.is_connected(_on_entity_died):
		SimulationBus.entity_died.connect(_on_entity_died)


func _load_all_cultures() -> void:
	cultures.clear()
	var dir_path: String = "res://data/naming_cultures/"
	var dir: DirAccess = DirAccess.open(dir_path)
	if dir == null:
		push_warning("[NameGenerator] Cannot open %s" % dir_path)
		return

	var loaded_count: int = 0
	dir.list_dir_begin()
	var file_name: String = dir.get_next()
	while file_name != "":
		if not dir.current_is_dir() and file_name.ends_with(".json"):
			var full_path: String = dir_path + file_name
			var fa: FileAccess = FileAccess.open(full_path, FileAccess.READ)
			if fa != null:
				var json: JSON = JSON.new()
				if json.parse(fa.get_as_text()) == OK and json.data is Dictionary:
					var data: Dictionary = json.data
					var cid = data.get("culture_id", "")
					if cid != "":
						cultures[cid] = data
						loaded_count += 1
				fa = null
		file_name = dir.get_next()
	dir.list_dir_end()

	if loaded_count == 0:
		push_warning("[NameGenerator] No naming culture JSON files found in %s" % dir_path)
	print("[NameGenerator] Loaded %d cultures: %s" % [cultures.size(), str(cultures.keys())])


func generate_name(gender: String, culture_id: String = "proto_syllabic", settlement_id: int = -1, parent_a_name: String = "", parent_b_name: String = "") -> String:
	var culture = cultures.get(culture_id, {})
	if culture.is_empty():
		culture = cultures.get("proto_nature", {})

	var patronymic_rule: String = str(culture.get("patronymic_rule", "none"))
	var parent_name_to_use: String = parent_a_name
	if parent_name_to_use == "":
		parent_name_to_use = parent_b_name

	var chosen_given: String = ""
	var chosen_full: String = ""
	for _i in range(20):
		var candidate: String = ""
		var allow_syllabic: bool = bool(culture.get("allow_markov_generation", true))
		if allow_syllabic and _rng != null and _rng.randf() > 0.3:
			candidate = generate_syllabic_name(culture, gender)
		else:
			var given_names = culture.get("given_names", {})
			var pool: Array = []
			if given_names is Dictionary:
				var gdict: Dictionary = given_names
				if gender == "male":
					pool = gdict.get("male", [])
				elif gender == "female":
					pool = gdict.get("female", [])
				else:
					pool = gdict.get("neutral", [])
				if pool.is_empty():
					pool = gdict.get("neutral", [])
			if not pool.is_empty() and _rng != null:
				var idx: int = _rng.randi_range(0, pool.size() - 1)
				candidate = str(pool[idx])
			else:
				candidate = generate_syllabic_name(culture, gender)
		if candidate == "":
				candidate = "Nameless"
		var candidate_full: String = candidate
		if patronymic_rule != "none" and parent_name_to_use != "":
			candidate_full = apply_patronymic(candidate, parent_name_to_use, gender, culture)
		if not is_name_duplicate(candidate_full, settlement_id):
			chosen_given = candidate
			chosen_full = candidate_full
			break

	if chosen_given == "":
		var fallback_given: String = generate_syllabic_name(culture, gender)
		if fallback_given == "":
			fallback_given = "Nameless"
		chosen_given = fallback_given + " II"
		chosen_full = chosen_given
		if patronymic_rule != "none" and parent_name_to_use != "":
			chosen_full = apply_patronymic(chosen_given, parent_name_to_use, gender, culture)

	register_name(chosen_full, settlement_id)
	return chosen_full


func generate_syllabic_name(culture: Dictionary, gender: String) -> String:
	var syllable_pools = culture.get("syllable_pools", {})
	var pools: Dictionary = {}
	if syllable_pools is Dictionary:
		pools = syllable_pools

	var onset_male: Array = pools.get("onset_male", [])
	var onset_female: Array = pools.get("onset_female", [])
	var nucleus: Array = pools.get("nucleus", [])
	var coda: Array = pools.get("coda", [])
	var coda_final: Array = pools.get("coda_final", [])

	if nucleus.is_empty():
		nucleus = ["a", "e", "i", "o", "u"]

	var syllable_count_cfg = culture.get("syllable_count", {})
	var min_count: int = 2
	var max_count: int = 3
	if syllable_count_cfg is Dictionary:
		min_count = int(syllable_count_cfg.get("min", 2))
		max_count = int(syllable_count_cfg.get("max", 3))
	if min_count < 1:
		min_count = 1
	if max_count < min_count:
		max_count = min_count

	var count: int = min_count
	if _rng != null:
		count = _rng.randi_range(min_count, max_count)

	var built: String = ""
	for i in range(count):
		var is_first: bool = i == 0
		var is_last: bool = i == count - 1

		var onset_pool: Array = onset_male
		if gender == "female":
			onset_pool = onset_female
		if onset_pool.is_empty():
			onset_pool = onset_male
		if onset_pool.is_empty():
			onset_pool = onset_female

		if not onset_pool.is_empty():
			var add_onset: bool = is_first
			if not is_first and _rng != null:
				add_onset = _rng.randf() < 0.7
			if add_onset:
				var onset_idx: int = 0
				if _rng != null:
					onset_idx = _rng.randi_range(0, onset_pool.size() - 1)
				built += str(onset_pool[onset_idx])

		var nuc_idx: int = 0
		if _rng != null:
			nuc_idx = _rng.randi_range(0, nucleus.size() - 1)
		built += str(nucleus[nuc_idx])

		if is_last:
			if not coda_final.is_empty():
				var coda_final_idx: int = 0
				if _rng != null:
					coda_final_idx = _rng.randi_range(0, coda_final.size() - 1)
				built += str(coda_final[coda_final_idx])
		elif not coda.is_empty():
			var add_coda: bool = false
			if _rng != null:
				add_coda = _rng.randf() < 0.6
			if add_coda:
				var coda_idx: int = 0
				if _rng != null:
					coda_idx = _rng.randi_range(0, coda.size() - 1)
				built += str(coda[coda_idx])

	if built.length() == 0:
		return "Nameless"
	return built.left(1).to_upper() + built.substr(1)


func apply_patronymic(given: String, parent_name: String, gender: String, culture: Dictionary) -> String:
	var patronymic_rule: String = str(culture.get("patronymic_rule", "none"))
	var config = culture.get("patronymic_config", {})
	if not (config is Dictionary):
		config = {}
	match patronymic_rule:
		"prefix":
			var suffix_key: String = "female_suffix" if gender == "female" else "male_suffix"
			var child_suffix: String = str(config.get(suffix_key, "'s child"))
			return "%s, %s%s" % [given, parent_name, child_suffix]
		"suffix":
			var suffix_key: String = "female_suffix" if gender == "female" else "male_suffix"
			var suffix: String = str(config.get(suffix_key, "son"))
			return "%s %s%s" % [given, parent_name, suffix]
		_:
			return given


func is_name_duplicate(name_str: String, settlement_id: int) -> bool:
	if settlement_id < 0:
		return false
	var names = used_names_per_settlement.get(settlement_id, {})
	if names is Dictionary:
		var names_dict: Dictionary = names
		return names_dict.has(name_str)
	return false


func register_name(name_str: String, settlement_id: int) -> void:
	if settlement_id < 0:
		return
	if not used_names_per_settlement.has(settlement_id):
		used_names_per_settlement[settlement_id] = {}
	var names = used_names_per_settlement.get(settlement_id, {})
	if names is Dictionary:
		var names_dict: Dictionary = names
		names_dict[name_str] = true


func unregister_name(name_str: String, settlement_id: int) -> void:
	if settlement_id < 0:
		return
	if not used_names_per_settlement.has(settlement_id):
		return
	var names = used_names_per_settlement.get(settlement_id, {})
	if names is Dictionary:
		var names_dict: Dictionary = names
		names_dict.erase(name_str)


func _on_entity_died(entity_id: int, _entity_name: String, _cause: String, _age_years: float, _tick: int) -> void:
	if _entity_manager == null:
		return
	if not _entity_manager.has_method("get_entity"):
		return
	var entity = _entity_manager.get_entity(entity_id)
	if entity == null:
		return
	var settlement_id: int = int(entity.settlement_id)
	unregister_name(_entity_name, settlement_id)


func save_registry(path: String) -> void:
	var save_data: Dictionary = {}
	var keys: Array = used_names_per_settlement.keys()
	for i in range(keys.size()):
		var sid: int = int(keys[i])
		var names = used_names_per_settlement.get(sid, {})
		var names_dict: Dictionary = {}
		if names is Dictionary:
			names_dict = names
		save_data[str(sid)] = names_dict.keys()
	var fa: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if fa != null:
		fa.store_string(JSON.stringify(save_data))


func load_registry(path: String) -> void:
	used_names_per_settlement.clear()
	if not FileAccess.file_exists(path):
		return
	var fa: FileAccess = FileAccess.open(path, FileAccess.READ)
	if fa == null:
		return
	var json: JSON = JSON.new()
	if json.parse(fa.get_as_text()) != OK:
		return
	if not (json.data is Dictionary):
		return
	var data: Dictionary = json.data
	var keys: Array = data.keys()
	for i in range(keys.size()):
		var sid: int = int(str(keys[i]))
		var names = data.get(keys[i], [])
		var names_array: Array = []
		if names is Array:
			names_array = names
		used_names_per_settlement[sid] = {}
		for j in range(names_array.size()):
			used_names_per_settlement[sid][str(names_array[j])] = true


func clear_registry() -> void:
	used_names_per_settlement.clear()
