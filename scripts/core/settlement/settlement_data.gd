extends RefCounted

const CivTechState = preload("res://scripts/core/tech/civ_tech_state.gd")

var id: int = 0
var center_x: int = 0
var center_y: int = 0
var founding_tick: int = 0
var culture_id: String = "proto_syllabic"
var member_ids: Array = []
var building_ids: Array = []
## 정착지 공유 가치관 캐시 [Axelrod 1997] - value_system이 200 tick마다 재계산
var shared_values: Dictionary = {}
## [Weber 1922] Charismatic leader entity ID. -1 means no leader elected yet.
var leader_id: int = -1
## Tick of the last completed election. -1 = never elected.
## Used for per-settlement re-election cycle. [Boehm 1999]
var last_election_tick: int = -1

## === Stratification Monitor [Kohler 2017, Boehm 1999] ===
## Gini coefficient of wealth distribution [0.0, 1.0]
var gini_coefficient: float = 0.0
## Effectiveness of egalitarian leveling mechanisms [0.0, 1.0]
var leveling_effectiveness: float = 1.0
## Phase: "egalitarian", "transitional", "stratified"
var stratification_phase: String = "egalitarian"
## 90th percentile wealth score (for normalization)
var wealth_p90: float = 1.0

## [Weber 1922] Authority type: "charismatic" | "traditional" | "rational_legal"
## Computed by NetworkSystem from shared_values. Default: charismatic (primitive era).
var authority_type: String = "charismatic"

## [Tilly 1978] Tick of last revolution event (cooldown guard). -1 = never.
var revolution_cooldown_tick: int = -1

## ── Tech Tree State [Boyd & Richerson 1985, Henrich 2004] ──────────────────
## [Henrich 2004] Per-tech dynamic state. Key = tech_id (String), Value = CivTechState dict.
## See civ_tech_state.gd for dict structure.
var tech_states: Dictionary = {}
## Current era label (highest era achieved). "stone_age" -> "tribal" -> "bronze_age" ...
var tech_era: String = "stone_age"


## Is this tech currently known (active/usable)?
func has_tech(tech_id: String) -> bool:
	if not tech_states.has(tech_id):
		return false
	return CivTechState.is_active(tech_states[tech_id])


## Get all currently known (active) tech IDs.
func get_known_techs() -> Array:
	var result: Array = []
	for tech_id in tech_states:
		if CivTechState.is_active(tech_states[tech_id]):
			result.append(tech_id)
	return result


## Get all tech IDs in any non-unknown state (including forgotten).
func get_all_encountered_techs() -> Array:
	return tech_states.keys()


func to_dict() -> Dictionary:
	return {
		"id": id,
		"center_x": center_x,
		"center_y": center_y,
		"founding_tick": founding_tick,
		"culture_id": culture_id,
		"member_ids": member_ids.duplicate(),
		"building_ids": building_ids.duplicate(),
		"leader_id": leader_id,
		"last_election_tick": last_election_tick,
		"gini_coefficient": gini_coefficient,
		"leveling_effectiveness": leveling_effectiveness,
		"stratification_phase": stratification_phase,
		"wealth_p90": wealth_p90,
		"authority_type": authority_type,
		"revolution_cooldown_tick": revolution_cooldown_tick,
		"tech_states": _serialize_tech_states(),
		"tech_era": tech_era,
	}


func _serialize_tech_states() -> Dictionary:
	var result: Dictionary = {}
	for tech_id in tech_states:
		result[tech_id] = CivTechState.to_save_dict(tech_states[tech_id])
	return result


static func from_dict(data: Dictionary) -> RefCounted:
	var script = load("res://scripts/core/settlement/settlement_data.gd")
	var settlement = script.new()
	settlement.id = data.get("id", 0)
	settlement.center_x = data.get("center_x", 0)
	settlement.center_y = data.get("center_y", 0)
	settlement.founding_tick = data.get("founding_tick", 0)
	settlement.culture_id = data.get("culture_id", "proto_nature")

	settlement.member_ids.clear()
	var raw_member_ids = data.get("member_ids", [])
	if raw_member_ids is Array:
		for i in range(raw_member_ids.size()):
			settlement.member_ids.append(int(raw_member_ids[i]))

	settlement.building_ids.clear()
	var raw_building_ids = data.get("building_ids", [])
	if raw_building_ids is Array:
		for i in range(raw_building_ids.size()):
			settlement.building_ids.append(int(raw_building_ids[i]))

	settlement.leader_id = data.get("leader_id", -1)
	settlement.last_election_tick = data.get("last_election_tick", -1)
	settlement.gini_coefficient = data.get("gini_coefficient", 0.0)
	settlement.leveling_effectiveness = data.get("leveling_effectiveness", 1.0)
	settlement.stratification_phase = data.get("stratification_phase", "egalitarian")
	settlement.wealth_p90 = data.get("wealth_p90", 1.0)
	settlement.authority_type = data.get("authority_type", "charismatic")
	settlement.revolution_cooldown_tick = data.get("revolution_cooldown_tick", -1)
	## V2 tech_states deserialization with V1 migration
	var raw_ts = data.get("tech_states", {})
	if raw_ts is Dictionary and raw_ts.size() > 0:
		for tech_id in raw_ts:
			settlement.tech_states[tech_id] = CivTechState.from_save_dict(raw_ts[tech_id])
	else:
		## V1 migration: convert discovered_techs array to tech_states dict
		var old_techs: Array = data.get("discovered_techs", [])
		for tech_id in old_techs:
			settlement.tech_states[tech_id] = CivTechState.create_migrated(tech_id)
	settlement.tech_era = data.get("tech_era", "stone_age")

	return settlement
