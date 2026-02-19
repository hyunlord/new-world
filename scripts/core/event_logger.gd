extends Node

var _events: Array[Dictionary] = []
var _type_counts: Dictionary = {}
const MAX_EVENTS: int = 100000
const PRUNE_AMOUNT: int = 10000

var _gather_count: int = 0
var _gather_totals: Dictionary = {"food": 0.0, "wood": 0.0, "stone": 0.0}
var _last_summary_tick: int = 0

## High-frequency events suppressed from console output
const QUIET_EVENTS: PackedStringArray = [
	"entity_moved", "resource_gathered", "needs_updated", "auto_eat", "action_chosen",
	"entity_ate", "entity_rested", "entity_socialized",
	"action_changed", "food_taken", "resources_delivered", "entity_starving",
]


func _ready() -> void:
	SimulationBus.simulation_event.connect(_on_simulation_event)


func _on_simulation_event(event: Dictionary) -> void:
	_events.append(event)
	var etype: String = event.get("type", "unknown")
	_type_counts[etype] = _type_counts.get(etype, 0) + 1
	if _events.size() > MAX_EVENTS:
		_events = _events.slice(PRUNE_AMOUNT)

	if etype == "resource_gathered":
		_gather_count += 1
		var res_type: String = event.get("resource_type", "")
		var amount: float = event.get("amount", 0.0)
		if _gather_totals.has(res_type):
			_gather_totals[res_type] += amount
		var tick: int = event.get("tick", 0)
		if tick - _last_summary_tick >= 50 and _gather_count > 0:
			print("[Tick %d] Gathered %dx: Food+%.0f Wood+%.0f Stone+%.0f" % [
				tick, _gather_count,
				_gather_totals.get("food", 0.0),
				_gather_totals.get("wood", 0.0),
				_gather_totals.get("stone", 0.0),
			])
			_gather_count = 0
			_gather_totals = {"food": 0.0, "wood": 0.0, "stone": 0.0}
			_last_summary_tick = tick

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
		"entity_born":
			var pos_x: int = data.get("position_x", 0)
			var pos_y: int = data.get("position_y", 0)
			print("[Tick %d] + BORN: %s at (%d,%d)" % [tick, ename, pos_x, pos_y])
		"entity_starved":
			print("[Tick %d] x %s starved" % [tick, ename])
		"entity_died":
			var cause: String = data.get("cause", "unknown")
			print("[Tick %d] x %s died (%s)" % [tick, ename, cause])
		"entity_died_natural":
			var age: int = data.get("age", 0)
			var age_days: int = age / 24
			print("[Tick %d] x DIED: %s age %dd (old age)" % [tick, ename, age_days])
		"building_completed":
			var btype: String = data.get("building_type", "?")
			var bx: int = data.get("tile_x", 0)
			var by: int = data.get("tile_y", 0)
			print("[Tick %d] # BUILT: %s at (%d,%d)" % [tick, btype, bx, by])
		"entity_starving":
			var timer: int = data.get("starving_timer", 0)
			print("[Tick %d] ! STARVING: %s (timer: %d/50)" % [tick, ename, timer])
		"job_assigned":
			var job: String = data.get("job", "?")
			print("[Tick %d] > %s assigned: %s" % [tick, ename, job])
		"job_reassigned":
			var from_job: String = data.get("from_job", "?")
			var to_job: String = data.get("to_job", "?")
			print("[Tick %d] > %s: %s -> %s" % [tick, ename, from_job, to_job])
		"trait_violation":
			var action: String = data.get("action_id", "?")
			var sev: String = data.get("severity", "?")
			var sv: float = data.get("stress", 0.0)
			print("[Tick %d] ! %s violated trait via '%s': stress=%.1f (%s)" % [tick, ename, action, sv, sev])
		_:
			pass  # Unknown/future events — suppress to avoid output overflow


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
