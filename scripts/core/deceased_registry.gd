extends Node

## Registry of deceased entities for historical viewing.
## Preserves essential data after entity death for family trees, chronicles, and UI.

var _records: Dictionary = {}  # entity_id -> Dictionary (lightweight record)

## Memory management: max records before pruning old unconnected entries
const MAX_RECORDS: int = 500
const PRUNE_THRESHOLD_YEARS: int = 100  # In-game years before eligible for pruning


## Register a death — call BEFORE removing entity from EntityManager
func register_death(entity: RefCounted, cause: String, current_tick: int) -> void:
	var age_years: float = float(entity.age) / float(GameConfig.TICKS_PER_YEAR)
	var record: Dictionary = {
		"id": entity.id,
		"name": entity.entity_name,
		"gender": entity.gender,
		"birth_tick": entity.birth_tick,
		"death_tick": current_tick,
		"death_cause": cause,
		"death_age_years": age_years,
		"parent_ids": entity.parent_ids.duplicate(),
		"partner_id": entity.partner_id,
		"children_ids": entity.children_ids.duplicate(),
		"settlement_id": entity.settlement_id,
		"personality": entity.personality.to_dict() if entity.personality != null else {},
		"job": entity.job,
		"total_gathered": entity.total_gathered,
		"buildings_built": entity.buildings_built,
		"age_stage": entity.age_stage,
		"frailty": entity.frailty,
		"speed": entity.speed,
		"strength": entity.strength,
		"trauma_scars": entity.trauma_scars.duplicate(true) if "trauma_scars" in entity and entity.trauma_scars != null else [],
		"violation_history": entity.violation_history.duplicate() if "violation_history" in entity and entity.violation_history != null else {},
		"display_traits": _snapshot_display_traits(entity),
		"hunger": entity.hunger,
		"energy": entity.energy,
		"social": entity.social,
		"current_action": entity.current_action,
		"inventory": entity.inventory.duplicate() if entity.inventory != null else {},
		"emotion_data": entity.emotion_data.to_dict() if entity.emotion_data != null else {},
		"birth_date": entity.birth_date.duplicate() if not entity.birth_date.is_empty() else {},
		"death_date": _current_date_from_tick(current_tick),
		"death_age_days": (current_tick - entity.birth_tick) / 12,
	}
	_records[entity.id] = record

	# Prune if too many records
	if _records.size() > MAX_RECORDS:
		_prune_old_records(current_tick)


func _current_date_from_tick(tick: int) -> Dictionary:
	var GameCalendar = load("res://scripts/core/game_calendar.gd")
	var d: Dictionary = GameCalendar.tick_to_date(tick)
	return {"year": d.year, "month": d.month, "day": d.day}


## Get a deceased record by entity ID
func get_record(id: int) -> Dictionary:
	return _records.get(id, {})


## Check if an entity is in the deceased registry
func is_deceased(id: int) -> bool:
	return id in _records


## Get all deceased records
func get_all() -> Array:
	return _records.values()


## Get deceased count
func get_count() -> int:
	return _records.size()


## Prune old records with no living connections
func _prune_old_records(current_tick: int) -> void:
	var threshold_ticks: int = PRUNE_THRESHOLD_YEARS * GameConfig.TICKS_PER_YEAR
	var to_remove: Array = []
	var values: Array = _records.values()
	for i in range(values.size()):
		var r: Dictionary = values[i]
		if (current_tick - r.death_tick) < threshold_ticks:
			continue
		# Keep if has living children (checked externally would be better,
		# but for simplicity we keep all records with children_ids)
		if r.children_ids.size() > 0:
			continue
		to_remove.append(r.id)

	# Remove oldest first, keep at least half of MAX_RECORDS
	var remove_count: int = mini(to_remove.size(), _records.size() - MAX_RECORDS / 2)
	for i in range(remove_count):
		_records.erase(to_remove[i])


## Serialize for save/load
func to_save_data() -> Array:
	return _records.values()


## Load from saved data
func load_save_data(data: Array) -> void:
	_records.clear()
	for i in range(data.size()):
		var d: Dictionary = data[i]
		_records[d.get("id", -1)] = d


func _snapshot_display_traits(entity: RefCounted) -> Array:
	var result: Array = []
	if "display_traits" in entity:
		for dt in entity.display_traits:
			result.append({"id": dt.get("id", ""), "salience": dt.get("salience", 0.0)})
	return result
