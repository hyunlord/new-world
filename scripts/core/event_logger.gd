extends Node

var _events: Array[Dictionary] = []
var _type_counts: Dictionary = {}
const MAX_EVENTS: int = 100000
const PRUNE_AMOUNT: int = 10000

## High-frequency events suppressed from console output
const QUIET_EVENTS: PackedStringArray = ["entity_moved"]


func _ready() -> void:
	SimulationBus.simulation_event.connect(_on_simulation_event)


func _on_simulation_event(event: Dictionary) -> void:
	_events.append(event)
	var etype: String = event.get("type", "unknown")
	_type_counts[etype] = _type_counts.get(etype, 0) + 1
	if _events.size() > MAX_EVENTS:
		_events = _events.slice(PRUNE_AMOUNT)
	_debug_log_event(etype, event)


func _debug_log_event(event_type: String, data: Dictionary) -> void:
	if event_type in QUIET_EVENTS:
		return
	var tick: int = data.get("tick", -1)
	var ename: String = data.get("entity_name", "")
	match event_type:
		"entity_spawned":
			var pos = data.get("position", Vector2i.ZERO)
			print("[Tick %d] + %s spawned at (%d,%d)" % [tick, ename, pos.x, pos.y])
		"entity_starved":
			print("[Tick %d] x %s starved" % [tick, ename])
		"entity_died":
			var cause: String = data.get("cause", "unknown")
			print("[Tick %d] x %s died (%s)" % [tick, ename, cause])
		"action_changed":
			var from_action: String = data.get("from", "?")
			var to_action: String = data.get("to", "?")
			print("[Tick %d] ~ %s: %s -> %s" % [tick, ename, from_action, to_action])
		"entity_ate":
			print("[Tick %d] * %s ate (hunger: %.0f%%)" % [tick, ename, data.get("hunger_after", 0.0) * 100])
		"entity_rested":
			print("[Tick %d] * %s rested (energy: %.0f%%)" % [tick, ename, data.get("energy_after", 0.0) * 100])
		"entity_socialized":
			print("[Tick %d] * %s socialized (social: %.0f%%)" % [tick, ename, data.get("social_after", 0.0) * 100])
		"action_chosen":
			pass  # Suppress — action_changed covers this
		_:
			print("[Tick %d] %s" % [tick, event_type])


## Query: history for a specific entity
func get_entity_history(entity_id: int, limit: int = 50) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for i in range(_events.size() - 1, -1, -1):
		if _events[i].get("entity_id", -1) == entity_id:
			result.append(_events[i])
			if result.size() >= limit:
				break
	result.reverse()
	return result


## Query: events of a specific type
func get_by_type(event_type: String, limit: int = 50) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for i in range(_events.size() - 1, -1, -1):
		if _events[i].get("type", "") == event_type:
			result.append(_events[i])
			if result.size() >= limit:
				break
	result.reverse()
	return result


## Query: events within a tick range
func get_tick_range(from_tick: int, to_tick: int) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for ev: Dictionary in _events:
		var t: int = ev.get("tick", -1)
		if t >= from_tick and t <= to_tick:
			result.append(ev)
	return result


## Get event type counts
func get_stats() -> Dictionary:
	return _type_counts.duplicate()


## Total event count
func get_total_count() -> int:
	return _events.size()


## Most recent events
func get_recent(count: int = 20) -> Array[Dictionary]:
	var start: int = maxi(0, _events.size() - count)
	return _events.slice(start)


## Serialization for save/load
func to_save_data() -> Array[Dictionary]:
	return _events.duplicate()


func load_save_data(data: Array[Dictionary]) -> void:
	_events.clear()
	_type_counts.clear()
	for item: Dictionary in data:
		_events.append(item)
		var etype: String = item.get("type", "unknown")
		_type_counts[etype] = _type_counts.get(etype, 0) + 1


func clear() -> void:
	_events.clear()
	_type_counts.clear()
