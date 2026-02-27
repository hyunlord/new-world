extends RefCounted

## [Boyd & Richerson 1985, Henrich 2004] Data-driven tech tree.
## Loads all TechNode JSON definitions from data/tech/.
## Provides: definition lookup, prerequisite checking, unlock queries.
## Stateless per-discovery — SettlementData.discovered_techs stores state.

const TECH_DATA_DIRS: Array = [
	"res://data/tech/stone_age/",
	"res://data/tech/tribal/",
]

var _definitions: Dictionary = {}  ## tech_id -> Dictionary (raw JSON)


## Load all tech node definitions from the configured JSON data directories.
func load_all() -> void:
	for dir_path in TECH_DATA_DIRS:
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
						_definitions[tech_id] = parsed
			fname = dir.get_next()


## Return the raw JSON definition dictionary for the given tech ID, or empty if not found.
func get_def(tech_id: String) -> Dictionary:
	return _definitions.get(tech_id, {})


## Return an array of all loaded tech IDs.
func get_all_ids() -> Array:
	return _definitions.keys()


## Check if settlement can potentially discover this tech (prerequisites met)
func can_discover(settlement: RefCounted, tech_id: String) -> bool:
	if tech_id in settlement.discovered_techs:
		return false  ## already discovered
	var def: Dictionary = _definitions.get(tech_id, {})
	if def.is_empty():
		return false
	var prereqs: Array = def.get("prerequisites", [])
	for prereq in prereqs:
		if prereq not in settlement.discovered_techs:
			return false
	return true


## Apply all unlocks from a discovered tech.
## Returns dict with skills/buildings/jobs arrays.
func apply_unlocks(tech_id: String) -> Dictionary:
	var def: Dictionary = _definitions.get(tech_id, {})
	return {
		"skills":    def.get("unlocks_skills", []),
		"buildings": def.get("unlocks_buildings", []),
		"jobs":      def.get("unlocks_jobs", []),
	}


## Update settlement.tech_era based on all discovered techs.
func update_era(settlement: RefCounted) -> void:
	var era_reqs: Array = GameConfig.TECH_ERA_TRIBAL_REQUIRES
	var all_met: bool = true
	for req in era_reqs:
		if req not in settlement.discovered_techs:
			all_met = false
			break
	if all_met and settlement.tech_era == "stone_age":
		settlement.tech_era = "tribal"
		SimulationBus.emit_event("era_advanced", {
			"settlement_id": settlement.id,
			"new_era": "tribal",
		})
