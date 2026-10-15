extends SceneTree

const MAX_DAYS: int = 10
const ACTIVITY_KEYS: Array[String] = ["ids", "kinds", "place_x", "place_y", "until_tick"]
const DETAIL_KEYS: Array[String] = [
	"found", "agent_id", "x", "y", "state_tag", "hunger", "thirst", "sleep", "target_kind",
]

var _world: Node = null
var _require_routine: bool = false
var _passed: int = 0
var _failures: Array[String] = []
var _tick: int = 0
var _samples: int = 0
var _active_rows: int = 0
var _stale_rows: int = 0
var _stale_rechecks: int = 0
var _first_active_tick: int = -1


func _init() -> void:
	for argument: String in OS.get_cmdline_user_args():
		if argument == "--routine":
			_require_routine = true
		else:
			_check(false, "unsupported argument: " + argument)
			_finish(2)
			return
	call_deferred("_run")


func _run() -> void:
	if not _check(ClassDB.class_exists("WorldSimNode"), "WorldSimNode registered"):
		_finish(1)
		return
	_world = ClassDB.instantiate("WorldSimNode") as Node
	if not _check(_world != null, "WorldSimNode instantiates as Node"):
		_finish(1)
		return
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(_world)
	for method: String in [
		"get_activity_snapshot", "get_agent_snapshot", "get_agent_detail",
		"get_world_info", "get_time_snapshot", "advance_ticks",
	]:
		if not _check(_world.has_method(method), "method exists: " + method):
			_finish(1)
			return
	var info_value: Variant = _world.call("get_world_info")
	var time_value: Variant = _world.call("get_time_snapshot")
	if not _check(info_value is Dictionary and time_value is Dictionary, "world and time dictionaries"):
		_finish(1)
		return
	var info: Dictionary = info_value
	var time: Dictionary = time_value
	var ticks_per_day: int = int(info.get("ticks_per_day", 0))
	_tick = int(time.get("tick", -1))
	if not _check(ticks_per_day > 0 and ticks_per_day % 24 == 0 and _tick == 0, "hourly clock at initial tick zero"):
		_finish(1)
		return
	if not _check_detail(0, false, false) or not _sample_and_recheck_stale():
		_finish(1)
		return
	if _require_routine and _active_rows == 0:
		var ticks_per_hour: int = int(float(ticks_per_day) / 24.0)
		for hour: int in range(1, MAX_DAYS * 24 + 1):
			if not _advance(hour * ticks_per_hour - _tick) or not _sample_and_recheck_stale():
				_finish(1)
				return
			if _active_rows > 0:
				break
	if _require_routine:
		_check(_active_rows > 0, "nonempty activity joins state 4 and target 8 within ten days")
	_finish(0 if _failures.is_empty() else 1)


func _sample_and_recheck_stale() -> bool:
	var stale: Array[int] = _sample()
	if not _failures.is_empty():
		return false
	if not stale.is_empty():
		# Combat may leave Activity until the next decision tick removes it.
		_stale_rechecks += 1
		if not _advance(1):
			return false
		var after: Array[int] = _sample()
		for entity_bits: int in stale:
			_check(not after.has(entity_bits), "stale activity resolved next tick: %d" % entity_bits)
	return _failures.is_empty()


func _sample() -> Array[int]:
	_samples += 1
	var stale: Array[int] = []
	var value: Variant = _world.call("get_activity_snapshot")
	var agents_value: Variant = _world.call("get_agent_snapshot")
	if not _check(value is Dictionary and agents_value is Dictionary, "snapshot dictionaries"):
		return stale
	var snapshot: Dictionary = value
	var agents: Dictionary = agents_value
	if not _exact_keys(snapshot, ACTIVITY_KEYS, "activity"):
		return stale
	if not _check(
		snapshot["ids"] is PackedInt64Array and snapshot["kinds"] is PackedByteArray
		and snapshot["place_x"] is PackedInt32Array and snapshot["place_y"] is PackedInt32Array
		and snapshot["until_tick"] is PackedInt64Array, "activity packed array types"
	):
		return stale
	var ids: PackedInt64Array = snapshot["ids"]
	var kinds: PackedByteArray = snapshot["kinds"]
	var place_x: PackedInt32Array = snapshot["place_x"]
	var place_y: PackedInt32Array = snapshot["place_y"]
	var until_tick: PackedInt64Array = snapshot["until_tick"]
	if not _check(
		ids.size() == kinds.size() and ids.size() == place_x.size()
		and ids.size() == place_y.size() and ids.size() == until_tick.size(), "activity parallel lengths"
	):
		return stale
	if not _check(
		agents.get("ids") is PackedInt64Array and agents.get("states") is PackedByteArray
		and agents.get("target_kinds") is PackedByteArray, "agent snapshot join array types"
	):
		return stale
	var agent_ids: PackedInt64Array = agents["ids"]
	var states: PackedByteArray = agents["states"]
	var targets: PackedByteArray = agents["target_kinds"]
	if not _check(agent_ids.size() == states.size() and agent_ids.size() == targets.size(), "agent join lengths"):
		return stale
	if not _check(not agent_ids.is_empty(), "live agents available for detail contract"):
		return stale
	if not _check_detail(agent_ids[0], true, false):
		return stale
	var agent_index: Dictionary[int, int] = {}
	for index: int in range(agent_ids.size()):
		agent_index[agent_ids[index]] = index
	var seen: Dictionary[int, bool] = {}
	for index: int in range(ids.size()):
		var entity_bits: int = ids[index]
		if not _check(agent_index.has(entity_bits) and not seen.has(entity_bits), "unique activity id joins live snapshot"):
			return stale
		seen[entity_bits] = true
		if not _check(kinds[index] <= 6 and place_x[index] >= 0 and place_y[index] >= 0 and until_tick[index] >= -1, "activity value domains"):
			return stale
		var agent_row: int = agent_index[entity_bits]
		if states[agent_row] == 4:
			if not _check(targets[agent_row] == 8, "activity state 4 joins target 8"):
				return stale
			if not _check_detail(entity_bits, true, true):
				return stale
			_active_rows += 1
			if _first_active_tick < 0:
				_first_active_tick = _tick
		else:
			stale.append(entity_bits)
			_stale_rows += 1
	return stale


func _check_detail(entity_bits: int, expected_found: bool, activity: bool) -> bool:
	var value: Variant = _world.call("get_agent_detail", entity_bits)
	if not _check(value is Dictionary, "detail dictionary"):
		return false
	var detail: Dictionary = value
	if not _exact_keys(detail, DETAIL_KEYS, "detail"):
		return false
	if not _check(detail["found"] is bool and bool(detail["found"]) == expected_found, "detail found sentinel"):
		return false
	if activity:
		return _check(int(detail["state_tag"]) == 4 and int(detail["target_kind"]) == 8, "activity detail state 4 target 8")
	return true


func _advance(count: int) -> bool:
	var value: Variant = _world.call("advance_ticks", count)
	if not _check(value is int and int(value) == _tick + count, "advance_ticks exact clock"):
		return false
	_tick = int(value)
	return true


func _exact_keys(value: Dictionary, expected: Array[String], label: String) -> bool:
	var exact: bool = value.size() == expected.size()
	for key: String in expected:
		exact = exact and value.has(key)
	return _check(exact, "%s exact %d keys" % [label, expected.size()])


func _check(condition: bool, label: String) -> bool:
	if condition:
		_passed += 1
	else:
		_failures.append("tick %d: %s" % [_tick, label])
	return condition


func _finish(exit_code: int) -> void:
	print(JSON.stringify({
		"suite": "pass17_bridge", "ok": exit_code == 0, "routine_required": _require_routine,
		"passed": _passed, "failures": _failures, "tick": _tick, "samples": _samples,
		"active_joined_rows": _active_rows, "first_active_tick": _first_active_tick,
		"stale_rows": _stale_rows, "stale_rechecks": _stale_rechecks,
		"stale_policy": "removed or active again after one tick",
	}))
	if is_instance_valid(_world):
		_world.free()
	quit(exit_code)
