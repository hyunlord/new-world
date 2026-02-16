extends Node

## Chronicles significant events in the simulation for historical viewing.
## Events are categorized by type and importance for memory management.

const GameCalendar = preload("res://scripts/core/game_calendar.gd")

## Event type constants
const EVENT_BIRTH: String = "birth"
const EVENT_DEATH: String = "death"
const EVENT_MARRIAGE: String = "marriage"
const EVENT_ORPHANED: String = "orphaned"
const EVENT_PARTNER_DIED: String = "partner_died"
const EVENT_SETTLEMENT_FOUNDED: String = "settlement_founded"
const EVENT_POPULATION_MILESTONE: String = "population_milestone"
const EVENT_FAMINE: String = "famine"
const EVENT_ADULT: String = "became_adult"

## Memory limits
const MAX_WORLD_EVENTS: int = 1000
const PRUNE_INTERVAL_YEARS: int = 10
const LOW_IMPORTANCE_MAX_AGE_YEARS: int = 20
const MED_IMPORTANCE_MAX_AGE_YEARS: int = 50

## Storage
var _world_events: Array = []
var _personal_events: Dictionary = {}  # entity_id -> Array of entries
var _last_prune_year: int = 0

## Entity manager reference (for name lookups)
var _entity_manager: RefCounted


func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager


## Log an event
func log_event(type: String, entity_id: int, description: String,
		importance: int = 3, related_ids: Array = [], tick: int = -1) -> void:
	if tick < 0:
		return
	var date: Dictionary = GameCalendar.tick_to_date(tick)
	var entry: Dictionary = {
		"tick": tick,
		"year": date.year,
		"month": date.month,
		"day": date.day,
		"event_type": type,
		"entity_id": entity_id,
		"entity_name": _get_entity_name(entity_id),
		"description": description,
		"related_ids": related_ids,
		"importance": importance,
	}

	_world_events.append(entry)

	# Add to personal events for entity and related entities
	if entity_id >= 0:
		if entity_id not in _personal_events:
			_personal_events[entity_id] = []
		_personal_events[entity_id].append(entry)
	for rid in related_ids:
		if rid not in _personal_events:
			_personal_events[rid] = []
		_personal_events[rid].append(entry)


## Get world events (newest first), optionally filtered by type
func get_world_events(filter_type: String = "", limit: int = 100) -> Array:
	var result: Array = []
	# Iterate from end (newest) to start
	var idx: int = _world_events.size() - 1
	while idx >= 0 and result.size() < limit:
		var e: Dictionary = _world_events[idx]
		if filter_type == "" or e.event_type == filter_type:
			result.append(e)
		idx -= 1
	return result


## Get personal events for an entity
func get_personal_events(entity_id: int) -> Array:
	return _personal_events.get(entity_id, [])


## Get total event count
func get_event_count() -> int:
	return _world_events.size()


## Periodic memory management
func prune_old_events(current_tick: int) -> void:
	var current_year: int = current_tick / GameConfig.TICKS_PER_YEAR
	if current_year - _last_prune_year < PRUNE_INTERVAL_YEARS:
		return
	_last_prune_year = current_year

	var low_cutoff: int = (current_year - LOW_IMPORTANCE_MAX_AGE_YEARS) * GameConfig.TICKS_PER_YEAR
	var med_cutoff: int = (current_year - MED_IMPORTANCE_MAX_AGE_YEARS) * GameConfig.TICKS_PER_YEAR

	var kept: Array = []
	for i in range(_world_events.size()):
		var e: Dictionary = _world_events[i]
		var imp: int = e.importance
		if imp <= 2 and e.tick < low_cutoff:
			continue  # Drop old low-importance
		if imp == 3 and e.tick < med_cutoff:
			continue  # Drop old medium-importance
		kept.append(e)

	# Enforce hard limit
	if kept.size() > MAX_WORLD_EVENTS:
		kept = kept.slice(kept.size() - MAX_WORLD_EVENTS)

	_world_events = kept

	# Prune personal events: remove entries for events no longer in world
	var valid_ticks: Dictionary = {}
	for i in range(_world_events.size()):
		valid_ticks[_world_events[i].tick] = true

	var entity_ids: Array = _personal_events.keys()
	for i in range(entity_ids.size()):
		var eid: int = entity_ids[i]
		var events: Array = _personal_events[eid]
		var filtered: Array = []
		for j in range(events.size()):
			if events[j].tick in valid_ticks or events[j].importance >= 4:
				filtered.append(events[j])
		if filtered.size() > 0:
			_personal_events[eid] = filtered
		else:
			_personal_events.erase(eid)


## Helper: get entity name by ID
func _get_entity_name(entity_id: int) -> String:
	if _entity_manager == null or entity_id < 0:
		return "?"
	var entity: RefCounted = _entity_manager.get_entity(entity_id)
	if entity != null:
		return entity.entity_name
	# Check DeceasedRegistry
	if has_node("/root/DeceasedRegistry"):
		var registry: Node = get_node("/root/DeceasedRegistry")
		var record: Dictionary = registry.get_record(entity_id)
		if record.size() > 0:
			return record.get("name", "?")
	return "?"


## Serialize for save/load
func to_save_data() -> Dictionary:
	return {
		"world_events": _world_events.duplicate(true),
		"personal_events": _personal_events.duplicate(true),
	}


## Load from saved data
func load_save_data(data: Dictionary) -> void:
	_world_events = data.get("world_events", [])
	_personal_events = data.get("personal_events", {})
