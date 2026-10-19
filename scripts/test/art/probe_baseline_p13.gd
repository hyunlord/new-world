extends SceneTree

const ReadRoute = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
## Live bridge census. No scene/renderers, interventions, or simulated fixtures.
const KINDS: Array[String] = ["TendFire", "Craft", "Repair", "HearthCircle", "Explore", "Play", "SitByFire"]
const FRAMINGS: Dictionary = {
	"P1": [20261010, 40560, 0], "P2": [20261010, 31320, 0],
	"P3": [20261013, 46189, 0], "P4": [20261013, 126994, 0],
	"W0": [20261010, 40560, 0], "W1": [20261013, 126994, 0], "W200": [15100, 1440, 200], "W500": [15100, 1440, 500], "W-multi": [19603, 90840, 0],
	"P5": [15100, 1440, 500], "P6": [20261010, 40560, 0],
}
var _world: Node
var _options: Dictionary = {}
var _rows: Array[Dictionary] = []
var _failures: Array[String] = []
var _out: String
var _started: int

func _initialize() -> void:
	call_deferred("_run")

func _run() -> void:
	_started = Time.get_ticks_msec()
	for argument: String in OS.get_cmdline_user_args():
		if argument.begins_with("--") and argument.contains("="):
			var pair: PackedStringArray = argument.trim_prefix("--").split("=", true, 1)
			_options[pair[0]] = pair[1]
	var mode: String = str(_options.get("mode", "live"))
	_out = OS.get_environment("P13_BASELINE_OUT")
	if _out.is_empty():
		var evidence: String = OS.get_environment("CAPTURE_EVIDENCE")
		if evidence.is_empty():
			evidence = "/Users/rexxa/work/art-production13-evidence"
		_out = evidence.path_join("baseline-probe-" + mode + ".json")
	_world = ClassDB.instantiate("WorldSimNode") as Node
	if _world == null:
		_failures.append("WorldSimNode unavailable")
		_finish()
		return
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(_world)
	_world.call("set_sim_speed", 0.0)
	for getter: String in ["get_activity_snapshot", "get_agent_family", "get_agent_temperament"]:
		if not _world.has_method(getter):
			_failures.append("missing getter: " + getter)
	if not _failures.is_empty():
		_finish()
		return
	match mode:
		"live":
			_reset(int(_options.get("seed", 20261010)))
			var info: Dictionary = ReadRoute.read(_world, "get_world_info")
			var ticks_per_day: int = int(info["ticks_per_day"])
			var target: int = 29 * ticks_per_day + int(600.0 * ticks_per_day / 1440.0) - int(info["clock_offset_ticks"])
			if _advance(target):
				_rows.append(_census("day30_1000", true))
		"framings", "worlds":
			var selected: String = str(_options.get("case", "all"))
			if selected != "all" and not FRAMINGS.has(selected):
				_failures.append("unknown framing: " + selected)
			for name: String in FRAMINGS:
				if mode == "worlds" and not name.begins_with("W"):
					continue
				if selected != "all" and selected != name:
					continue
				var spec: Array = FRAMINGS[name]
				_reset(int(spec[0]), int(spec[2]))
				if _advance(int(spec[1])):
					_rows.append(_census(name, false))
				_write(false)
				await process_frame
		"search":
			await _search()
		_:
			_failures.append("unknown mode: " + mode)
	_finish()

func _reset(seed: int, population: int = 0) -> void:
	if population > 0:
		ReadRoute.mutate(_world, "reset_world_stress", [seed, population])
	else:
		ReadRoute.mutate(_world, "reset_world", [seed])
	_world.call("set_sim_speed", 0.0)

func _advance(target: int) -> bool:
	var clock: Dictionary = ReadRoute.read(_world, "get_time_snapshot")
	var tick: int = int(clock["tick"])
	if target < tick:
		_failures.append("advance target precedes current tick")
		return false
	while tick < target:
		var next: int = int(ReadRoute.mutate(_world, "advance_ticks", [mini(240, target - tick)]))
		if next <= tick or next > target:
			_failures.append("advance_ticks failed exact progress at %d toward %d" % [tick, target])
			return false
		tick = next
	return true

func _census(label: String, include_families: bool) -> Dictionary:
	var agents: Dictionary = ReadRoute.read(_world, "get_agent_snapshot")
	var settlements: Dictionary = ReadRoute.read(_world, "get_settlement_snapshot")
	var activity: Dictionary = ReadRoute.read(_world, "get_activity_snapshot")
	var clock: Dictionary = ReadRoute.read(_world, "get_time_snapshot")
	var by_entity: Dictionary = {}
	var counts: Dictionary = {}
	for kind: String in KINDS:
		counts[kind] = 0
	for i: int in activity["ids"].size():
		by_entity[int(activity["ids"][i])] = int(activity["kinds"][i])
	var unknown: int = 0
	var active_rows: Array[Dictionary] = []
	var families: Array[Dictionary] = []
	for i: int in agents["ids"].size():
		var entity: int = int(agents["ids"][i])
		if int(agents["states"][i]) == 4:
			var kind: int = int(by_entity.get(entity, -1))
			if kind >= 0 and kind < KINDS.size():
				counts[KINDS[kind]] += 1
			else:
				unknown += 1
			active_rows.append({"entity_bits": entity, "agent_id": agents["agent_ids"][i], "kind": kind})
		if include_families and families.size() < 5:
			var family: Dictionary = _world.call("get_agent_family", entity)
			if bool(family["found"]):
				families.append({"entity_bits": entity, "agent_id": agents["agent_ids"][i], "family": family,
					"temperament": _world.call("get_agent_temperament", entity), "detail": _world.call("get_agent_detail", entity)})
	var lit: int = 0
	for fire: int in settlements["fire_lit"]:
		lit += int(fire != 0)
	var hour: float = float(clock["time_of_day"])
	var info: Dictionary = ReadRoute.read(_world, "get_world_info")
	if label == "W-multi" and (int(clock["tick"]) + int(info["clock_offset_ticks"])) % int(info["ticks_per_day"]) != int(info["ticks_per_day"]) * 10 / 24:
		_failures.append("W-multi clock must be 10:00")
	var alive: int = 0
	for members: int in settlements.get("member_counts", []):
		alive += int(members > 0)
	return {"alive_settlements": alive, "porters": ReadRoute.read(_world, "get_porter_snapshot") if _world.has_method("get_porter_snapshot") else {}, "label": label, "world_info": ReadRoute.read(_world, "get_world_info"), "clock": clock,
		"population": agents["ids"].size(), "settlement_count": settlements["settlement_ids"].size(),
		"settlements": settlements, "buildings": ReadRoute.read(_world, "get_building_snapshot"), "lit_fire_count": lit,
		"lit_night": lit > 0 and (hour >= 20.0 or hour < 6.0),
		"activity_snapshot": activity, "state_tag4_counts": counts, "state_tag4_unknown": unknown,
		"state_tag4_agents": active_rows, "five_families": families}

func _search() -> void:
	var first_seed: int = int(_options.get("seed", 20261011))
	var last_seed: int = int(_options.get("last-seed", first_seed))
	var start: int = int(_options.get("start-tick", 0))
	var limit: int = int(_options.get("max-tick", 172800))
	var stride: int = int(_options.get("stride", 1))
	var purpose: String = str(_options.get("purpose", "villages"))
	if first_seed < 20261010 or last_seed > 20261042 or last_seed < first_seed or start < 0 or limit < start or stride < 1 or purpose not in ["villages", "night", "occupied"]:
		_failures.append("invalid bounded search arguments")
		return
	for seed: int in range(first_seed, last_seed + 1):
		_reset(seed)
		if not _advance(start):
			return
		var tick: int = start
		while tick <= limit:
			var settlements: Dictionary = ReadRoute.read(_world, "get_settlement_snapshot")
			var inhabited: int = 0
			for count: int in settlements.get("member_counts", PackedInt32Array()):
				inhabited += int(count > 0)
			var matched: bool = inhabited >= int(_options.get("min-villages", 2))
			if purpose == "occupied":
				matched = inhabited > 0
			if purpose == "night":
				var clock: Dictionary = ReadRoute.read(_world, "get_time_snapshot")
				var hour: float = float(clock["time_of_day"])
				matched = (hour >= 20.0 or hour < 6.0) and not settlements["fire_lit"].is_empty() and int(settlements["fire_lit"][0]) == 1
			if matched:
				var row: Dictionary = _census("search_" + purpose, false)
				row["search"] = {"seed": seed, "start_tick": start, "max_tick": limit, "stride": stride,
					"first_tick_proven": stride == 1 and start == 0, "first_sample_in_range": true}
				_rows.append(row)
				return
			if tick == limit:
				break
			tick = mini(limit, tick + stride)
			if not _advance(tick):
				return
			if tick % 1440 == 0:
				print("P12 SEARCH seed=%d tick=%d purpose=%s" % [seed, tick, purpose])
				await process_frame
		_rows.append({"seed": seed, "search_exhausted": true, "start_tick": start, "max_tick": limit, "stride": stride, "purpose": purpose})
		_write(false)

func _write(complete: bool) -> void:
	var directory: String = ProjectSettings.globalize_path(_out.get_base_dir())
	if DirAccess.make_dir_recursive_absolute(directory) != OK:
		_failures.append("cannot create output directory")
		return
	var file: FileAccess = FileAccess.open(_out, FileAccess.WRITE)
	if file == null:
		_failures.append("cannot open output: " + _out)
		return
	file.store_string(JSON.stringify({"schema_version": 1, "source": "WorldSimNode reset + deterministic advance_ticks; no interventions",
		"godot_double": OS.has_feature("double"), "options": _options, "complete": complete, "rows": _rows, "failures": _failures,
		"elapsed_seconds": (Time.get_ticks_msec() - _started) / 1000.0}, "\t") + "\n")
	file.close()

func _finish() -> void:
	_write(true)
	if is_instance_valid(_world):
		_world.free()
	print("P12 BASELINE rows=%d errors=%d output=%s" % [_rows.size(), _failures.size(), _out])
	quit(0 if _failures.is_empty() else 1)
