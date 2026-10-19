extends RefCounted
## Independent capture census of FFI deaths; never reads the status panel's log.

var _seen: Dictionary = {}
var _deaths: Array[Dictionary] = []
var _last_tick: int = -1
var max_interval_ticks: int = 0


func ingest(snapshot: Dictionary) -> void:
	var now: int = int(snapshot.get("current_tick", 0))
	if _last_tick > now:
		reset()
	if _last_tick >= 0:
		max_interval_ticks = maxi(max_interval_ticks, now - _last_tick)
	_last_tick = now
	var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
	var ticks: PackedInt64Array = snapshot.get("ticks", PackedInt64Array())
	var reasons: PackedInt32Array = snapshot.get("reasons", PackedInt32Array())
	if xs.size() != ys.size() or xs.size() != ticks.size() or xs.size() != reasons.size():
		push_error("capture death census received unaligned FFI arrays")
		return
	var occurrences: Dictionary = {}
	for index: int in xs.size():
		var tuple: String = "%d:%d:%d:%d" % [ticks[index], xs[index], ys[index], reasons[index]]
		var ordinal: int = int(occurrences.get(tuple, 0))
		occurrences[tuple] = ordinal + 1
		var key: String = "%s:%d" % [tuple, ordinal]
		if _seen.has(key):
			continue
		_seen[key] = true
		_deaths.append({"death_id": key, "tick": ticks[index], "x": xs[index], "y": ys[index], "reason": reasons[index], "ordinal": ordinal, "observed_tick": now})


func rows() -> Array[Dictionary]:
	return _deaths.duplicate(true)


func reset() -> void:
	_seen.clear()
	_deaths.clear()
	_last_tick = -1
	max_interval_ticks = 0
