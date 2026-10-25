# M-UX2 frame evidence helpers: strict stall scoring and read-only script/engine telemetry.
# Bridge methods: mux2_native_timings (optional diagnostic only). SimReads counters are sampled after the rendered frame, without a new getter.
extends RefCounted

const WARMUP_SECONDS: float = 10.0
const MEASURE_SECONDS: float = 300.0
const TOTAL_SECONDS: float = WARMUP_SECONDS + MEASURE_SECONDS
const MINUTES: int = 5
const REQUIRED_TIMINGS: Dictionary = {"buildings": ["process", "draw", "rebuild", "huts", "floor_prepare", "door_prepare"], "ground": ["process", "chunk_bakes", "compose", "upload", "strips", "space_read", "apply_space", "wall_prepare", "prepare_grounds", "trees", "visible", "doors", "wall_warmup"], "alerts": ["process", "poll", "rebuild"], "steward_hud": ["process", "steward", "facts", "learning", "layout"]}
var _minimap: Node
var _reads: Node
var _world: Node
var _native_previous: Dictionary = {}
var _timed_nodes: Dictionary = {}
var _previous_minimap_usec: int = 0
var _invalid: Dictionary = {}
var _tree: SceneTree
var _process_usec: int = -1
var _pre_draw_usec: int = -1
var _last_post_usec: int = -1
var _wall_warmup_observed_usec: int = 0


func setup(main: Node) -> void:
	_tree = main.get_tree()
	_tree.process_frame.connect(_on_process_frame)
	RenderingServer.frame_pre_draw.connect(_on_pre_draw)
	_minimap = main.get_node("UI/MuxHud/MapDock/Minimap")
	_reads = main.get_node("SimReads")
	_world = main.get_node("WorldSim")
	if _world.has_method("mux2_native_timings"):
		_native_previous = _world.call("mux2_native_timings")
	_previous_minimap_usec = int(_minimap.get("_total_usec"))
	_timed_nodes = {"buildings": main.get_node("ColonyView/BuildingLayer"), "ground": main.get_node("ColonyView/GroundLayer"), "alerts": main.get_node("UI/MuxHud/Alerts"), "steward_hud": main.get_node("UI/MuxHud")}

func begin_sampling(post_draw_usec: int) -> void:
	_last_post_usec = post_draw_usec

func finish() -> void:
	if _tree.process_frame.is_connected(_on_process_frame): _tree.process_frame.disconnect(_on_process_frame)
	if RenderingServer.frame_pre_draw.is_connected(_on_pre_draw): RenderingServer.frame_pre_draw.disconnect(_on_pre_draw)

func _on_process_frame() -> void:
	_process_usec = Time.get_ticks_usec()

func _on_pre_draw() -> void:
	_pre_draw_usec = Time.get_ticks_usec()

func _phase_sample(post_draw_usec: int) -> Dictionary:
	var previous: int = _last_post_usec
	_last_post_usec = post_draw_usec
	if previous < 0 or _process_usec < previous or _pre_draw_usec < _process_usec or post_draw_usec < _pre_draw_usec:
		_invalid["frame_phases"] = "Missing or unordered frame signals"
		return {"status": "INVALID", "previous_post_usec": previous, "process_usec": _process_usec, "pre_draw_usec": _pre_draw_usec, "post_draw_usec": post_draw_usec}
	return {"status": "MEASURED", "post_to_process_usec": _process_usec - previous,
		"process_to_pre_draw_usec": _pre_draw_usec - _process_usec, "pre_to_post_draw_usec": post_draw_usec - _pre_draw_usec}

func sample(post_draw_usec: int) -> Dictionary:
	var frame: int = Engine.get_process_frames()
	var minimap_total: int = int(_minimap.get("_total_usec"))
	var result: Dictionary = {
		"frame": frame,
		"minimap_process_usec": minimap_total - _previous_minimap_usec,
		"sim_reads_usec": _reads.call("frame_call_usec"),
		"sim_reads_calls": _reads.call("frame_call_counts"),
		"performance": {
			"process_seconds": Performance.get_monitor(Performance.TIME_PROCESS),
			"physics_seconds": Performance.get_monitor(Performance.TIME_PHYSICS_PROCESS),
			"memory_static_bytes": Performance.get_monitor(Performance.MEMORY_STATIC),
			"objects": Performance.get_monitor(Performance.OBJECT_COUNT),
			"nodes": Performance.get_monitor(Performance.OBJECT_NODE_COUNT),
			"orphans": Performance.get_monitor(Performance.OBJECT_ORPHAN_NODE_COUNT),
			"draw_calls": Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME),
		},
	}
	_previous_minimap_usec = minimap_total
	for label: String in _timed_nodes:
		var node: Node = _timed_nodes[label]
		if node.has_method("mux2_frame_timings"):
			var timing: Dictionary = node.call("mux2_frame_timings")
			result[label] = timing
			if not _valid_timing(label, timing, frame): _invalid[label] = "Missing, disabled, stale-frame or invalid timing fields"
		else:
			result[label] = {"status": "NOT_MEASURED", "reason": "mux2_frame_timings instrumentation absent"}
			_invalid[label] = "Instrumentation absent"
	var ground_values: Dictionary = (result["ground"] as Dictionary).get("usec", {})
	_wall_warmup_observed_usec += int(ground_values.get("wall_warmup", 0))
	result["frame_phases"] = _phase_sample(post_draw_usec)
	result["freeze"] = _freeze_sample(frame)
	result["native"] = _native_sample(frame)
	return result

func _valid_timing(label: String, timing: Dictionary, frame: int) -> bool:
	if str(timing.get("status", "")) != "MEASURED" or int(timing.get("frame", -1)) != frame: return false
	var values: Dictionary = timing.get("usec", {})
	for metric: String in REQUIRED_TIMINGS[label]:
		if typeof(values.get(metric)) != TYPE_INT or int(values[metric]) < 0: return false
	if label == "buildings":
		var counts: Dictionary = timing.get("counts", {})
		for metric: String in ["sprite_creates", "sprite_frees", "texture_requests"]:
			if typeof(counts.get(metric)) != TYPE_INT or int(counts[metric]) < 0: return false
		if not timing.get("texture_ids") is Array: return false
	return true

func _freeze_sample(frame: int) -> Dictionary:
	if not _reads.has_method("mux2_freeze_timings"):
		_invalid["freeze"] = "Missing freeze instrumentation"
		return {"status": "NOT_MEASURED"}
	var timing: Dictionary = _reads.call("mux2_freeze_timings")
	if str(timing.get("status", "")) != "MEASURED" or int(timing.get("frame", -1)) != frame or not _numeric_values(timing.get("usec", null)):
		_invalid["freeze"] = "Disabled, stale or invalid freeze timing"
	return timing

func _numeric_values(value: Variant) -> bool:
	if not value is Dictionary: return false
	for key: Variant in value:
		if typeof(value[key]) != TYPE_INT or int(value[key]) < 0: return false
	return true

func _native_valid(timing: Dictionary, frame: int) -> bool:
	if str(timing.get("status", "")) != "MEASURED" or int(timing.get("frame", -1)) != frame: return false
	for metric: String in ["epoch", "frame_ticks", "executed_ticks"]:
		if typeof(timing.get(metric)) != TYPE_INT or int(timing[metric]) < 0: return false
	var totals: Variant = timing.get("totals_ns", null)
	var systems: Variant = timing.get("systems_ns", null)
	if not _numeric_values(totals) or not _numeric_values(systems) or (systems as Dictionary).is_empty(): return false
	for metric: String in ["process", "tick", "prune", "process_residual", "tick_residual"]:
		if not (totals as Dictionary).has(metric): return false
	return true

func _native_sample(frame: int) -> Dictionary:
	if not _world.has_method("mux2_native_timings"):
		_invalid["native"] = "Native instrumentation absent"
		return {"status": "NOT_MEASURED"}
	var timing: Dictionary = _world.call("mux2_native_timings")
	if not _native_valid(timing, frame):
		_invalid["native"] = "Disabled, stale or invalid native timing"
		return timing
	var previous: Dictionary = _native_previous
	_native_previous = timing
	if int(previous.get("epoch", -1)) != int(timing["epoch"]):
		_invalid["native"] = "Counter epoch changed during arm"
		return {"status": "RESET", "frame": frame, "epoch": timing["epoch"]}
	var result: Dictionary = {"status": "MEASURED", "frame": frame, "epoch": timing["epoch"], "frame_ticks": timing["frame_ticks"], "executed_ticks": timing["executed_ticks"]}
	for category: String in ["totals_ns", "systems_ns"]:
		var delta: Dictionary = {}
		var before: Dictionary = previous.get(category, {})
		var now: Dictionary = timing[category]
		for metric: String in now:
			if not before.has(metric) or int(now[metric]) < int(before[metric]):
				_invalid["native"] = "Missing or regressed cumulative counter"
				return {"status": "INVALID", "frame": frame}
			delta[metric] = float(int(now[metric]) - int(before[metric])) / 1000.0
		result[category.replace("_ns", "_usec")] = delta
	return result

func coverage() -> Dictionary:
	var warmup: Dictionary = _timed_nodes["ground"].call("mux2_wall_warmup_report")
	if int(warmup.get("elapsed_usec", 0)) > _wall_warmup_observed_usec:
		_invalid["wall_warmup"] = "Warmup ran outside sampled frames"
	var native: Dictionary = _world.call("mux2_native_timings") if _world.has_method("mux2_native_timings") else {}
	if not _native_valid(native, Engine.get_process_frames()): _invalid["native"] = "Native coverage unavailable"
	_freeze_sample(Engine.get_process_frames())
	var result: Dictionary = {"wall_warmup_observed_usec": _wall_warmup_observed_usec, "frame_phases": "MEASURED" if not _invalid.has("frame_phases") else "NOT_MEASURED", "native": "MEASURED" if not _invalid.has("native") else "NOT_MEASURED", "freeze": "MEASURED" if not _invalid.has("freeze") else "NOT_MEASURED", "minimap_process": "MEASURED", "sim_reads_raw_getters": "MEASURED", "complete": _invalid.is_empty(), "invalid": _invalid.duplicate()}
	for label: String in _timed_nodes:
		var node: Node = _timed_nodes[label]
		var available: bool = node.has_method("mux2_frame_timings")
		if available: available = _valid_timing(label, node.call("mux2_frame_timings"), Engine.get_process_frames())
		result[label] = "INSTRUMENTED" if available else "NOT_MEASURED"
		result["complete"] = bool(result["complete"]) and available
	return result

static func score(samples: Array[Dictionary], elapsed: float) -> Dictionary:
	var counts: Array[int] = [0, 0, 0, 0, 0]
	var startup_over33: int = 0
	var over50: int = 0
	var intervals: Array[float] = []
	var startup_max: float = 0.0
	var slow: Array[Dictionary] = []
	for row: Dictionary in samples:
		var end: float = float(row["elapsed_seconds"])
		var interval: float = float(row["interval_ms"])
		if interval > 33.0:
			slow.append(row)
		if end <= WARMUP_SECONDS:
			startup_max = maxf(startup_max, interval)
			if interval > 33.0: startup_over33 += 1
			continue
		intervals.append(interval)
		# A frame crossing the 10-second boundary is conservatively included in full.
		if interval > 50.0: over50 += 1
		if interval > 33.0:
			var minute: int = clampi(ceili((end - WARMUP_SECONDS) / 60.0) - 1, 0, MINUTES - 1)
			counts[minute] += 1
	var each_minute_pass: bool = true
	var total_over33: int = 0
	for count: int in counts:
		each_minute_pass = each_minute_pass and count <= 3
		total_over33 += count
	var complete: bool = elapsed >= TOTAL_SECONDS and not intervals.is_empty()
	return {
		"complete": complete, "pass": complete and over50 == 0 and each_minute_pass,
		"warmup_seconds": WARMUP_SECONDS, "required_measured_seconds": MEASURE_SECONDS,
		"actual_play_seconds": elapsed, "actual_post_warmup_seconds": maxf(0.0, elapsed - WARMUP_SECONDS),
		"post_warmup_over50": over50, "post_warmup_over33": total_over33,
		"over33_per_minute": counts, "minute_windows_seconds": [[10, 70], [70, 130], [130, 190], [190, 250], [250, 310]],
		"startup_over33": startup_over33, "startup_max_ms": startup_max,
		"all_over33_frames": slow, "post_warmup_intervals_ms": summarize(intervals),
		"scoring_note": "Strict > thresholds; boundary-crossing frames included. Final render overshoot is charged to minute five, never discarded. Full 300 post-warmup seconds required.",
	}

static func summarize(values: Array[float]) -> Dictionary:
	if values.is_empty(): return {"count": 0}
	var ordered: Array[float] = values.duplicate()
	ordered.sort()
	var total: float = 0.0
	for value: float in ordered: total += value
	return {"count": ordered.size(), "mean": total / ordered.size(), "max": ordered[-1], "p99": ordered[ceili(ordered.size() * 0.99) - 1]}

static func self_test() -> Array[String]:
	var failures: Array[String] = []
	var passing: Array[Dictionary] = [{"elapsed_seconds": 1.0, "interval_ms": 650.0}, {"elapsed_seconds": 10.0, "interval_ms": 99.0}, {"elapsed_seconds": 11.0, "interval_ms": 33.0}, {"elapsed_seconds": 12.0, "interval_ms": 50.0}, {"elapsed_seconds": 310.0, "interval_ms": 16.0}]
	var result: Dictionary = score(passing, 310.0)
	if not bool(result["pass"]) or int(result["post_warmup_over33"]) != 1 or int(result["startup_over33"]) != 2:
		failures.append("exact thresholds or warmup handling")
	if bool(score(passing, 309.99)["pass"]): failures.append("short run accepted")
	var too_slow: Array[Dictionary] = passing.duplicate(true)
	too_slow.append({"elapsed_seconds": 10.01, "interval_ms": 50.001})
	if bool(score(too_slow, 310.0)["pass"]): failures.append("post-warmup >50 accepted")
	var burst: Array[Dictionary] = passing.duplicate(true)
	for second: float in [70.01, 71.0, 72.0, 73.0]:
		burst.append({"elapsed_seconds": second, "interval_ms": 34.0})
	if bool(score(burst, 310.0)["pass"]): failures.append("four slow frames in one minute accepted")
	if bool(score([], 310.0)["pass"]): failures.append("empty samples accepted")
	return failures
