extends RefCounted

const TechState = preload("res://scripts/core/tech/tech_state.gd")

## [Henrich 2004] Per-settlement, per-tech dynamic state.
## Stored in SettlementData.tech_states[tech_id] as a Dictionary.
## This class provides static helpers for creating/reading/validating state dicts.


## Create a new CivTechState dict with defaults (for initial discovery).
static func create_discovered(tech_id: String, tick: int, discoverer_id: int = -1) -> Dictionary:
	return {
		"tech_id": tech_id,
		"state": TechState.STATE_TO_STRING[TechState.State.KNOWN_LOW],
		"discovered_tick": tick,
		"discoverer_id": discoverer_id,
		"practitioner_count": 0,
		"effective_carriers": 0,
		"atrophy_years": 0,
		"cultural_memory": 1.0,
		"last_active_use_tick": tick,
		"rediscovered_count": 0,
		"acquisition_method": "invented",
		"source_settlement_id": -1,
		"propagation_rate": 0.0,
		"adoption_curve_phase": "innovator",
		"total_ever_learned": 1,
		"cross_settlement_sources": [],
	}


## Create state dict for legacy migration (V1 -> V2).
## Assumes all V1 discovered techs are stable.
static func create_migrated(tech_id: String) -> Dictionary:
	return {
		"tech_id": tech_id,
		"state": TechState.STATE_TO_STRING[TechState.State.KNOWN_STABLE],
		"discovered_tick": 0,
		"discoverer_id": -1,
		"practitioner_count": 99,
		"effective_carriers": 99,
		"atrophy_years": 0,
		"cultural_memory": 1.0,
		"last_active_use_tick": 0,
		"rediscovered_count": 0,
		"acquisition_method": "invented",
		"source_settlement_id": -1,
		"propagation_rate": 0.0,
		"adoption_curve_phase": "laggard",
		"total_ever_learned": 99,
		"cross_settlement_sources": [],
	}


## Get the TechState.State enum value from a CivTechState dict.
static func get_state_enum(cts: Dictionary) -> int:
	return TechState.STATE_FROM_STRING.get(cts.get("state", "unknown"), TechState.State.UNKNOWN)


## Is this tech currently usable (known_low or known_stable)?
static func is_active(cts: Dictionary) -> bool:
	return TechState.is_known(get_state_enum(cts))


## Serialize for save
static func to_save_dict(cts: Dictionary) -> Dictionary:
	return cts.duplicate(true)


## Deserialize from save
static func from_save_dict(data: Dictionary) -> Dictionary:
	var result: Dictionary = data.duplicate(true)
	if not result.has("state"):
		result["state"] = "unknown"
	if not result.has("cultural_memory"):
		result["cultural_memory"] = 1.0
	if not result.has("atrophy_years"):
		result["atrophy_years"] = 0
	if not result.has("rediscovered_count"):
		result["rediscovered_count"] = 0
	if not result.has("acquisition_method"):
		result["acquisition_method"] = "invented"
	if not result.has("source_settlement_id"):
		result["source_settlement_id"] = -1
	if not result.has("propagation_rate"):
		result["propagation_rate"] = 0.0
	if not result.has("adoption_curve_phase"):
		result["adoption_curve_phase"] = "innovator"
	if not result.has("total_ever_learned"):
		result["total_ever_learned"] = 1
	if not result.has("cross_settlement_sources"):
		result["cross_settlement_sources"] = []
	return result
