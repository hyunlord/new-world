# M-UX2 frame evidence helpers: strict stall scoring and read-only script/engine telemetry.
# Bridge methods: mux2_native_timings (optional diagnostic only). SimReads counters are sampled after the rendered frame, without a new getter.
# M3D-SWITCH E2: the 2D GroundLayer/BuildingLayer timings (and the wall warm-up) are replaced by the 3D view's per-layer
#   process time: `setup` puts a Probe node before World3D, before each of its children and after the last; every probe
#   stamps Time.get_ticks_usec() in its _process, and the tree's process order (World3D, then each layer and its
#   subtree) makes the differences World3D's own and each layer's process time this frame (their rebuilds included).
#   `rebuild` is the arm's summary: the terrain built at the end and the summed layer time the samples observed.
extends RefCounted


## A process-order stamp (tools only; added beside the 3D layers by `setup`).
class Probe extends Node:
	var usec: int = -1
	var frame: int = -1

	func _process(_delta: float) -> void:
		usec = Time.get_ticks_usec()
		frame = Engine.get_process_frames()

const WARMUP_SECONDS: float = 10.0
const MEASURE_SECONDS: float = 300.0
const TOTAL_SECONDS: float = WARMUP_SECONDS + MEASURE_SECONDS
const MINUTES: int = 5
const REQUIRED_TIMINGS: Dictionary = {"world3d": ["world3d", "terrain", "flora", "sky", "structures", "props", "plants", "people", "animals", "visitors", "overlay", "labels", "camera_rig"], "alerts": ["process", "poll", "rebuild"], "steward_hud": ["process", "steward", "facts", "learning", "layout"]}
## World3D child name → its timing key (the order is the tree's, so probes bracket each child).
const LAYER_KEYS: Dictionary = {"Terrain": "terrain", "Flora": "flora", "Sky": "sky", "Structures": "structures", "Props": "props", "Plants": "plants", "People": "people", "Animals": "animals", "Visitors": "visitors", "Overlay": "overlay", "LabelLayer": "labels", "CameraRig": "camera_rig"}
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
var _world3d: Node
## [[timing key, probe before it]] in process order, then the closing probe.
var _brackets: Array = []
var _end_probe: Probe
var _rebuild_observed_usec: int = 0


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
	_timed_nodes = {"alerts": main.get_node("UI/MuxHud/Alerts"), "steward_hud": main.get_node("UI/MuxHud")}
	_world3d = main.get_node("World3D")
	var start: Probe = Probe.new()
	start.name = "Mux2ProbeWorld3D"
	main.add_child(start)
	main.move_child(start, _world3d.get_index())
	_brackets = [["world3d", start]]
	for child: Node in _world3d.get_children():
		if child is Probe or not LAYER_KEYS.has(str(child.name)):
			continue
		var probe: Probe = Probe.new()
		probe.name = "Mux2Probe" + str(child.name)
		_world3d.add_child(probe)
		_world3d.move_child(probe, child.get_index())
		_brackets.append([LAYER_KEYS[str(child.name)], probe])
	_end_probe = Probe.new()
	_end_probe.name = "Mux2ProbeEnd"
	_world3d.add_child(_end_probe)

func begin_sampling(post_draw_usec: int) -> void:
	_last_post_usec = post_draw_usec

func finish() -> void:
	if _tree.process_frame.is_connected(_on_process_frame): _tree.process_frame.disconnect(_on_process_frame)
	if RenderingServer.frame_pre_draw.is_connected(_on_pre_draw): RenderingServer.frame_pre_draw.disconnect(_on_pre_draw)


## World3D's own and each layer's process time this frame from the probe stamps; INVALID unless every probe ran
## this frame in order and every key of REQUIRED_TIMINGS.world3d is bracketed.
func world3d_timings(frame: int) -> Dictionary:
	var values: Dictionary = {}
	var probes: Array = []
	for bracket: Array in _brackets:
		probes.append(bracket[1])
	probes.append(_end_probe)
	for k: int in _brackets.size():
		var a: Probe = probes[k]
		var b: Probe = probes[k + 1]
		if not is_instance_valid(a) or not is_instance_valid(b) or a.frame != frame or b.frame != frame or b.usec < a.usec:
			return {"status": "INVALID", "frame": frame}
		values[_brackets[k][0]] = b.usec - a.usec
	for key: String in REQUIRED_TIMINGS["world3d"]:
		if not values.has(key):
			return {"status": "INVALID", "frame": frame, "missing": key}
	return {"status": "MEASURED", "frame": frame, "usec": values}


## The arm's 3D rebuild summary: the terrain complete at the end, and the summed layer time the samples observed.
func rebuild_report() -> Dictionary:
	var terrain: Node = _world3d.get_node_or_null("Terrain") if is_instance_valid(_world3d) else null
	return {"terrain_complete": terrain != null and bool(terrain.call("is_complete")), "observed_usec": _rebuild_observed_usec}

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
	var layers: Dictionary = world3d_timings(frame)
	result["world3d"] = layers
	if not _valid_timing("world3d", layers, frame): _invalid["world3d"] = "Missing, stale-frame or unordered layer probes"
	for key: String in (layers.get("usec", {}) as Dictionary):
		_rebuild_observed_usec += int(layers["usec"][key])
	for label: String in _timed_nodes:
		var node: Node = _timed_nodes[label]
		if node.has_method("mux2_frame_timings"):
			var timing: Dictionary = node.call("mux2_frame_timings")
			result[label] = timing
			if not _valid_timing(label, timing, frame): _invalid[label] = "Missing, disabled, stale-frame or invalid timing fields"
		else:
			result[label] = {"status": "NOT_MEASURED", "reason": "mux2_frame_timings instrumentation absent"}
			_invalid[label] = "Instrumentation absent"
	result["frame_phases"] = _phase_sample(post_draw_usec)
	result["freeze"] = _freeze_sample(frame)
	result["native"] = _native_sample(frame)
	return result

func _valid_timing(label: String, timing: Dictionary, frame: int) -> bool:
	if str(timing.get("status", "")) != "MEASURED" or int(timing.get("frame", -1)) != frame: return false
	var values: Dictionary = timing.get("usec", {})
	for metric: String in REQUIRED_TIMINGS[label]:
		if typeof(values.get(metric)) != TYPE_INT or int(values[metric]) < 0: return false
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
	var native: Dictionary = _world.call("mux2_native_timings") if _world.has_method("mux2_native_timings") else {}
	if not _native_valid(native, Engine.get_process_frames()): _invalid["native"] = "Native coverage unavailable"
	_freeze_sample(Engine.get_process_frames())
	var result: Dictionary = {"rebuild_observed_usec": _rebuild_observed_usec, "frame_phases": "MEASURED" if not _invalid.has("frame_phases") else "NOT_MEASURED", "native": "MEASURED" if not _invalid.has("native") else "NOT_MEASURED", "freeze": "MEASURED" if not _invalid.has("freeze") else "NOT_MEASURED", "minimap_process": "MEASURED", "sim_reads_raw_getters": "MEASURED", "complete": _invalid.is_empty(), "invalid": _invalid.duplicate()}
	for label: String in _timed_nodes:
		var node: Node = _timed_nodes[label]
		var available: bool = node.has_method("mux2_frame_timings")
		if available: available = _valid_timing(label, node.call("mux2_frame_timings"), Engine.get_process_frames())
		result[label] = "INSTRUMENTED" if available else "NOT_MEASURED"
		result["complete"] = bool(result["complete"]) and available
	# Every sample validated the probes; any stale or unordered frame recorded `world3d` in _invalid.
	result["world3d"] = "INSTRUMENTED" if not _brackets.is_empty() and not _invalid.has("world3d") else "NOT_MEASURED"
	result["complete"] = bool(result["complete"]) and result["world3d"] == "INSTRUMENTED"
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
	failures.append_array(_probe_self_test())
	return failures


## The probe bracket arithmetic on hand-set stamps: exact differences, and INVALID for a stale or unordered probe or a
## missing layer key.
static func _probe_self_test() -> Array[String]:
	var failures: Array[String] = []
	var monitor: RefCounted = load("res://tools/v8/mux2_perf_monitor.gd").new()
	var probes: Array[Probe] = []
	var keys: Array = REQUIRED_TIMINGS["world3d"]
	for k: int in keys.size() + 1:
		var probe: Probe = Probe.new()
		probe.frame = 7
		probe.usec = 1000 + k * 10 + k * k
		probes.append(probe)
	var brackets: Array = []
	for k: int in keys.size():
		brackets.append([keys[k], probes[k]])
	monitor.set("_brackets", brackets)
	monitor.set("_end_probe", probes[keys.size()])
	var good: Dictionary = monitor.call("world3d_timings", 7)
	var exact: bool = str(good.get("status", "")) == "MEASURED"
	for k: int in keys.size():
		exact = exact and int((good.get("usec", {}) as Dictionary).get(keys[k], -1)) == probes[k + 1].usec - probes[k].usec
	if not exact: failures.append("probe differences")
	if str((monitor.call("world3d_timings", 8) as Dictionary).get("status", "")) != "INVALID": failures.append("stale probes accepted")
	probes[3].usec = probes[4].usec + 1
	if str((monitor.call("world3d_timings", 7) as Dictionary).get("status", "")) != "INVALID": failures.append("unordered probes accepted")
	probes[3].usec = 1000 + 3 * 10 + 9
	monitor.set("_brackets", brackets.slice(0, brackets.size() - 1))
	monitor.set("_end_probe", probes[keys.size() - 1])
	if str((monitor.call("world3d_timings", 7) as Dictionary).get("status", "")) != "INVALID": failures.append("missing layer accepted")
	for probe: Probe in probes:
		probe.free()
	return failures
