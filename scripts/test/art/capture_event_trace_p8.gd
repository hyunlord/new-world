extends RefCounted

var _events: Array[Dictionary] = []
var _seen: Dictionary = {}
var _settlements: Dictionary = {}
var _sites: Dictionary = {}
var _births: int = -1
var _tick: int = -1
var _start: int = -1
var _max_interval: int = 0
var _complete: bool = true
var _baseline: bool = true

func reset() -> void:
	_events.clear()
	_seen.clear()
	_settlements.clear()
	_sites.clear()
	_births = -1
	_tick = -1
	_start = -1
	_max_interval = 0
	_complete = true
	_baseline = true

func ingest(time: Dictionary, settlements: Dictionary, society: Dictionary, construction: Dictionary, tallies: Dictionary, intervention: Dictionary, agents: Dictionary = {}) -> void:
	var now: int = int(time.get("tick", -1))
	if _tick >= 0 and now < _tick:
		reset()
	if _start < 0:
		_start = now
	else:
		_max_interval = maxi(_max_interval, now - _tick)
	_complete = _complete and now >= 0 and _max_interval <= 1
	_tick = now
	_society(settlements, society)
	_construction(construction)
	var births: int = int(tallies.get("births", 0))
	if _births >= 0 and births > _births:
		var has_agents: bool = agents.has("agent_ids") or agents.has("ids")
		_complete = _complete and has_agents
		var args: Array = []
		if has_agents:
			args.append(agents.get("agent_ids", agents.get("ids", [])).size())
		for ordinal: int in range(_births + 1, births + 1):
			_add({"event_id": "birth:%d" % ordinal, "key": "HUD_FEED_AGENT_BORN", "tick": now, "args_raw": args.duplicate(), "class": "ordinary", "source": "birth_tally_delta", "count": 1, "births_before": _births, "births_after": births, "arguments_complete": has_agents})
	_births = births
	_casts(intervention)
	_baseline = false

func _add(event: Dictionary) -> void:
	var identity: String = str(event["event_id"])
	if _seen.has(identity):
		return
	_seen[identity] = true
	event["observed_tick"] = _tick
	if not event.has("count"):
		event["count"] = 1
	if not event.has("raw_settlement_id"):
		event["raw_settlement_id"] = -1
	if not _baseline:
		_events.append(event)

func _society(settlements: Dictionary, society: Dictionary) -> void:
	var current: Dictionary = {}
	for sid: int in settlements.get("settlement_ids", settlements.get("ids", [])):
		current[sid] = true
	var ids: Variant = society.get("fission_ids", [])
	for index: int in ids.size():
		var fid: int = int(ids[index])
		var parent: int = int(society["fission_parents"][index])
		_add({"event_id": "depart:%d" % fid, "key": "HUD_FEED_FISSION_DEPARTED", "tick": int(society["fission_depart_ticks"][index]), "args_raw": [int(society["fission_sizes"][index]), parent], "raw_settlement_id": parent, "fission_id": fid, "class": "protected", "source": "fission_log"})
		var founded: int = int(society["fission_founded_ticks"][index])
		var daughter: int = int(society["fission_daughters"][index])
		if founded >= 0 and daughter >= 0:
			_add({"event_id": "founded:%d" % daughter, "key": "HUD_FEED_SETTLEMENT_FOUNDED", "tick": founded, "args_raw": [daughter], "raw_settlement_id": daughter, "fission_id": fid, "class": "protected", "source": "fission_log"})
		var cancelled: int = int(society["fission_cancelled_ticks"][index])
		if cancelled >= 0:
			_add({"event_id": "cancel:%d" % fid, "key": "HUD_FEED_FISSION_CANCELLED", "tick": cancelled, "args_raw": [], "raw_settlement_id": parent, "fission_id": fid, "class": "protected", "source": "fission_log"})
	var settlement_ids: Variant = society.get("settlement_ids", [])
	for sid: int in current:
		if _settlements.has(sid):
			continue
		var index: int = settlement_ids.find(sid)
		var parent: int = int(society["parent_ids"][index]) if index >= 0 else -1
		var founded: int = int(society["founded_ticks"][index]) if index >= 0 else _tick
		_add({"event_id": "founded:%d" % sid, "key": "HUD_FEED_SETTLEMENT_FOUNDED" if parent >= 0 else "HUD_FEED_SETTLEMENT_FORMED", "tick": founded, "args_raw": [sid], "raw_settlement_id": sid, "class": "protected", "source": "settlement_snapshot"})
	for sid: int in _settlements:
		if not current.has(sid):
			_add({"event_id": "abandoned:%d" % sid, "key": "HUD_FEED_SETTLEMENT_ABANDONED", "tick": _tick, "args_raw": [sid], "raw_settlement_id": sid, "class": "protected", "source": "first_observed_absent"})
	_settlements = current

func _construction(construction: Dictionary) -> void:
	var current: Dictionary = {}
	for id: int in construction.get("ids", []):
		current[id] = true
		if not _sites.has(id):
			_add({"event_id": "site:start:%d" % id, "key": "HUD_FEED_SITE_STARTED", "tick": _tick, "args_raw": [id], "class": "ordinary", "source": "construction_id_appeared"})
	for id: int in _sites:
		if not current.has(id):
			_add({"event_id": "site:complete:%d" % id, "key": "HUD_FEED_SITE_COMPLETED", "tick": _tick, "args_raw": [id], "class": "ordinary", "source": "construction_id_disappeared"})
	_sites = current

func _casts(state: Dictionary) -> void:
	var ids: Variant = state.get("log_ids", [])
	for index: int in ids.size():
		var sid: int = int(state["log_settlement_ids"][index])
		var kind: String = str(state["log_kinds"][index])
		_add({"event_id": "cast:%d" % int(ids[index]), "key": "HUD_FEED_GOD_CAST_FMT", "tick": int(state["log_ticks"][index]), "args_raw": [sid, kind], "raw_settlement_id": sid, "kind": kind, "applied": int(state["log_applied"][index]), "class": "protected", "source": "intervention_log"})

func rows() -> Array[Dictionary]:
	return _events.duplicate(true)

func metadata() -> Dictionary:
	return {"events": rows(), "coverage_complete": _complete, "max_sample_interval_ticks": _max_interval, "window_start_tick": _start, "window_end_tick": _tick}
