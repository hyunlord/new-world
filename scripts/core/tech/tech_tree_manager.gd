extends RefCounted

## [Boyd & Richerson 1985, Henrich 2004] Data-driven tech tree (V2).
## Loads all TechNode V2 JSON definitions from data/tech/.
## Provides: definition lookup, V2 prerequisite checking (all_of/any_of/soft),
## environment gating, branching exclusion, unlock queries, era progression.
## Stateless per-discovery — SettlementData.tech_states stores state.

const TechSchemaValidator = preload("res://scripts/core/tech/tech_schema_validator.gd")

var _definitions: Dictionary = {}  ## tech_id -> Dictionary (raw JSON)
var _world_data: RefCounted  ## WorldData reference for biome scanning


## Load all tech node definitions from the configured V2 JSON data directories.
func load_all() -> void:
	_definitions.clear()
	for dir_path in GameConfig.TECH_DATA_DIRS_V2:
		var dir: DirAccess = DirAccess.open(dir_path)
		if dir == null:
			continue
		dir.list_dir_begin()
		var fname: String = dir.get_next()
		while fname != "":
			if fname.ends_with(".json"):
				var full_path: String = dir_path + fname
				var text: String = FileAccess.get_file_as_string(full_path)
				var parsed = JSON.parse_string(text)
				if parsed is Dictionary:
					var tech_id: String = parsed.get("id", "")
					if tech_id != "":
						## Validate V2 schema
						var errors: Array = TechSchemaValidator.validate(parsed)
						if errors.size() > 0:
							for err in errors:
								push_warning("[TechTreeManager] %s: %s" % [tech_id, err])
						_definitions[tech_id] = parsed
			fname = dir.get_next()
	if OS.is_debug_build():
		print("[TechTreeManager] Loaded %d tech definitions" % _definitions.size())


## Set world data reference for biome scanning.
func set_world_data(wd: RefCounted) -> void:
	_world_data = wd


## Return the raw JSON definition dictionary for the given tech ID, or empty if not found.
func get_def(tech_id: String) -> Dictionary:
	return _definitions.get(tech_id, {})


## Return an array of all loaded tech IDs.
func get_all_ids() -> Array:
	return _definitions.keys()


## Check if settlement can potentially discover this tech (V2 logic).
## Returns false if already known, no definition, prereqs unmet, environment blocked, or branch excluded.
func can_discover(settlement: RefCounted, tech_id: String) -> bool:
	## Already known?
	if settlement.has_tech(tech_id):
		return false
	var def: Dictionary = _definitions.get(tech_id, {})
	if def.is_empty():
		return false
	## V2 prereq_logic check
	if not _check_prereqs(settlement, def):
		return false
	## V2 environment requirements
	if not _check_environment(settlement, def):
		return false
	## V2 branching exclusion
	if not _check_branching(settlement, def):
		return false
	return true


## Apply all unlocks from a discovered tech (V2 format with V1 fallback).
## Returns dict with skills/buildings/jobs/actions/techs_enabled arrays.
func apply_unlocks(tech_id: String) -> Dictionary:
	var def: Dictionary = _definitions.get(tech_id, {})
	var unlocks: Dictionary = def.get("unlocks", {})
	## V2 unlocks block
	if not unlocks.is_empty():
		return {
			"skills":    unlocks.get("skills", []),
			"buildings": unlocks.get("buildings", []),
			"jobs":      unlocks.get("jobs", []),
			"actions":   unlocks.get("actions", []),
			"techs_enabled": unlocks.get("techs_enabled", []),
		}
	## V1 fallback
	return {
		"skills":    def.get("unlocks_skills", []),
		"buildings": def.get("unlocks_buildings", []),
		"jobs":      def.get("unlocks_jobs", []),
		"actions":   [],
		"techs_enabled": [],
	}


## Update settlement.tech_era based on all known techs.
func update_era(settlement: RefCounted) -> void:
	## Stone Age -> Tribal
	if settlement.tech_era == "stone_age":
		var all_met: bool = true
		for req in GameConfig.TECH_ERA_TRIBAL_REQUIRES:
			if not settlement.has_tech(req):
				all_met = false
				break
		if all_met:
			settlement.tech_era = "tribal"
			SimulationBus.emit_event("era_advanced", {
				"settlement_id": settlement.id,
				"new_era": "tribal",
			})
	## Tribal -> Bronze Age
	if settlement.tech_era == "tribal":
		var all_met: bool = true
		for req in GameConfig.TECH_ERA_BRONZE_AGE_REQUIRES:
			if not settlement.has_tech(req):
				all_met = false
				break
		if all_met and GameConfig.TECH_ERA_BRONZE_AGE_REQUIRES.size() > 0:
			settlement.tech_era = "bronze_age"
			SimulationBus.emit_event("era_advanced", {
				"settlement_id": settlement.id,
				"new_era": "bronze_age",
			})


## ── V2 Prerequisite Logic ──────────────────────────────────────────────────

func _check_prereqs(settlement: RefCounted, def: Dictionary) -> bool:
	var prereq: Dictionary = def.get("prereq_logic", {})
	## Backward compatibility: V1 flat prerequisites array
	if prereq.is_empty() and def.has("prerequisites"):
		prereq = {"all_of": def["prerequisites"]}
	## all_of: every tech must be known
	for tech_id in prereq.get("all_of", []):
		if not settlement.has_tech(tech_id):
			return false
	## any_of: at least one from each group
	for group in prereq.get("any_of", []):
		if not group is Array:
			continue
		var any_met: bool = false
		for tech_id in group:
			if settlement.has_tech(tech_id):
				any_met = true
				break
		if not any_met:
			return false
	## soft prereqs don't block — they add bonus in discovery calc
	return true


## ── V2 Environment Gating ──────────────────────────────────────────────────

func _check_environment(settlement: RefCounted, def: Dictionary) -> bool:
	var reqs: Dictionary = def.get("requirements", {})
	## biomes_nearby check
	var needed_biomes: Array = reqs.get("biomes_nearby", [])
	if needed_biomes.size() > 0:
		if not _settlement_has_biome_nearby(settlement, needed_biomes):
			return false
	## resources_nearby check — deferred to C-1d (always pass for now)
	## settlement_tags, institution_tags, infrastructure_tags — deferred to C-1d+
	return true


## Scan tiles within radius around settlement center for matching biomes.
func _settlement_has_biome_nearby(settlement: RefCounted, biome_tags: Array) -> bool:
	if _world_data == null:
		return true  ## no world data = skip check (editor/test mode)
	var cx: int = settlement.center_x
	var cy: int = settlement.center_y
	var radius: int = GameConfig.TECH_BIOME_SCAN_RADIUS
	for dx in range(-radius, radius + 1):
		for dy in range(-radius, radius + 1):
			var bx: int = cx + dx
			var by: int = cy + dy
			if bx < 0 or by < 0 or bx >= _world_data.width or by >= _world_data.height:
				continue
			var biome_int: int = _world_data.get_biome(bx, by)
			var biome_name: String = _biome_int_to_tag(biome_int)
			if biome_name in biome_tags:
				return true
	return false


## Map GameConfig.Biome enum int -> lowercase string tag used in JSON.
static func _biome_int_to_tag(biome_int: int) -> String:
	match biome_int:
		GameConfig.Biome.DEEP_WATER: return "deep_water"
		GameConfig.Biome.SHALLOW_WATER: return "shallow_water"
		GameConfig.Biome.BEACH: return "beach"
		GameConfig.Biome.GRASSLAND: return "grassland"
		GameConfig.Biome.FOREST: return "forest"
		GameConfig.Biome.DENSE_FOREST: return "dense_forest"
		GameConfig.Biome.HILL: return "hill"
		GameConfig.Biome.MOUNTAIN: return "mountain"
		GameConfig.Biome.SNOW: return "snow"
		_: return "unknown"


## ── V2 Branching Exclusion ─────────────────────────────────────────────────

func _check_branching(settlement: RefCounted, def: Dictionary) -> bool:
	var branch: Dictionary = def.get("branching", {})
	var exclusions: Array = branch.get("mutually_exclusive_with", [])
	for excluded_id in exclusions:
		if settlement.has_tech(excluded_id):
			return false
	return true
