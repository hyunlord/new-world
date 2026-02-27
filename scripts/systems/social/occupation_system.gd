extends "res://scripts/core/simulation/simulation_system.gd"
## OccupationSystem: evaluates each agent's highest skill → assigns occupation.
## [Holland 1959 RIASEC, Super 1957 Career Development]
## priority=36 — runs before TitleSystem(37) and ReputationSystem(38).

var _entity_manager: RefCounted

## Maps occupation name → legacy job category for behavior_system compatibility
var _category_map: Dictionary = {}
## Reverse: occupation name → legacy job string
var _occupation_to_job: Dictionary = {}


func _init() -> void:
	system_name = "occupation"
	priority = 36
	tick_interval = GameConfig.OCCUPATION_EVAL_INTERVAL


func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_load_category_map()


func _load_category_map() -> void:
	var path: String = "res://data/occupations/occupation_categories.json"
	if not FileAccess.file_exists(path):
		push_warning("[OccupationSystem] Missing occupation_categories.json")
		return
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		return
	var json := JSON.new()
	if json.parse(file.get_as_text()) != OK:
		push_warning("[OccupationSystem] Failed to parse occupation_categories.json")
		return
	var data: Dictionary = json.data
	var categories: Dictionary = data.get("categories", {})
	for job_cat in categories:
		var occupations: Array = categories[job_cat]
		for occ in occupations:
			_occupation_to_job[occ] = job_cat


func execute_tick(tick: int) -> void:
	if _entity_manager == null:
		return
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity = alive[i]
		## Skip infants and toddlers — no occupation
		if entity.age_stage == "infant" or entity.age_stage == "toddler":
			continue
		_evaluate_occupation(entity, tick)


func _evaluate_occupation(entity: RefCounted, tick: int) -> void:
	## Step 1: Find highest skill
	var best_skill_id: StringName = &""
	var best_skill_level: int = 0

	var skill_keys: Array = entity.skill_levels.keys()
	for j in range(skill_keys.size()):
		var sid = skill_keys[j]
		var lvl: int = int(entity.skill_levels[sid])
		if lvl > best_skill_level:
			best_skill_level = lvl
			best_skill_id = sid

	## Step 2: Check minimum threshold
	if best_skill_level < GameConfig.OCCUPATION_MIN_SKILL_LEVEL:
		if entity.age_stage == "child" or entity.age_stage == "teen":
			_set_occupation(entity, "none", tick)
		else:
			## Adults with no skill above threshold → laborer
			_set_occupation(entity, "laborer", tick)
		return

	## Step 3: Convert skill_id to occupation name
	var new_occupation: String = _skill_id_to_occupation(best_skill_id)

	## Step 4: Hysteresis check — prevent flip-flopping
	if new_occupation != entity.occupation and entity.occupation != "none" and entity.occupation != "laborer":
		var current_occ_skill: StringName = _occupation_to_skill_id(entity.occupation)
		var current_level: int = int(entity.skill_levels.get(current_occ_skill, 0))
		var normalized_margin: float = (float(best_skill_level) - float(current_level)) / 100.0
		if normalized_margin < GameConfig.OCCUPATION_CHANGE_HYSTERESIS:
			return  ## Not enough margin, keep current

	## Step 5: Apply
	_set_occupation(entity, new_occupation, tick)


func _set_occupation(entity: RefCounted, new_occupation: String, tick: int) -> void:
	if entity.occupation == new_occupation:
		return
	var old: String = entity.occupation
	entity.previous_occupation = old
	entity.occupation = new_occupation

	## Map to legacy job for behavior_system compatibility
	entity.job = _occupation_to_legacy_job(new_occupation)

	SimulationBus.occupation_changed.emit(
		entity.id, entity.entity_name, old, new_occupation, tick
	)


## SKILL_FORAGING → "foraging"
func _skill_id_to_occupation(skill_id: StringName) -> String:
	var s: String = str(skill_id)
	if s.begins_with("SKILL_"):
		return s.substr(6).to_lower()
	return s.to_lower()


## "foraging" → &"SKILL_FORAGING"
func _occupation_to_skill_id(occupation: String) -> StringName:
	return StringName("SKILL_" + occupation.to_upper())


## Maps occupation to legacy 4-job category via loaded JSON
func _occupation_to_legacy_job(occupation: String) -> String:
	if occupation == "none" or occupation == "laborer":
		return "gatherer"
	return _occupation_to_job.get(occupation, "gatherer")
