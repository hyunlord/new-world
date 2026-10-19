extends SceneTree

const ReadRoute = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
## One launch = one world. The caller owns quiet/window locks and validity snapshots.
## --world W0|W1|W200|W500|W1000 --phase baseline|after --out <session.json>

const Session = preload("res://scripts/test/art/frame_measure_session_p12.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const WORLDS: Dictionary = {"W0": [20261010, 40560, 0], "W1": [20261013, 126994, 0], "W200": [15100, 1440, 200], "W500": [15100, 1440, 500], "W1000": [15100, 1440, 1000], "W-multi": [19400, 121080, 0]}
const METRICS: Array[String] = ["primary_cpu_ms", "process_ms", "render_cpu_ms", "setup_cpu_ms", "gpu_ms", "pacing_ms", "ticks_run"]

class MeasurementRuntime extends Session.Runtime:
	var reader: Node
	func read(getter: String) -> Dictionary:
		return ReadRoute.read(world, getter, true)
	func snapshot() -> Dictionary:
		var settled: Dictionary = read("get_settlement_snapshot")
		var alive_settlements: int = 0
		for count: int in settled.get("member_counts", []):
			alive_settlements += int(count > 0)
		return {"tick": int(read("get_time_snapshot")["tick"]), "camera": [camera.position.x, camera.position.y], "zoom": [camera.zoom.x, camera.zoom.y], "population": read("get_agent_snapshot")["agent_ids"].size(), "alive_settlements": alive_settlements, "selected_id": Selection.selected_id}

var _main: Node
var _world: Node
var _camera: Camera2D
var _runtime: MeasurementRuntime
var _blocks: Array[Dictionary] = []
var _errors: Array[String] = []
var _ablation: Array[Dictionary] = []
var _last_tick: int = 0
var _prior_camera_mode: int = Node.PROCESS_MODE_INHERIT
var _options: Dictionary = {"world": "W0", "phase": "baseline", "out": "", "skip-ablation": true, "ablation-only": false}
var _result: Dictionary = {}

func _initialize() -> void:
	_run.call_deferred()

func _run() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var index: int = 0
	while index < args.size():
		var key: String = args[index].trim_prefix("--")
		if key in ["skip-ablation", "ablation-only"]:
			_options[key] = true
		elif _options.has(key) and index + 1 < args.size():
			index += 1
			_options[key] = args[index]
		else:
			push_error("Unknown or incomplete argument: " + args[index])
			quit(2)
			return
		index += 1
	if not WORLDS.has(_options["world"]) or not ["baseline", "after"].has(_options["phase"]) or str(_options["out"]).is_empty():
		push_error("Requires valid --world, --phase and --out")
		quit(2)
		return
	if OS.get_cmdline_args().has("--fixed-fps") or DisplayServer.get_name() == "headless":
		push_error("Measurements require windowed Godot without --fixed-fps")
		quit(2)
		return
	_result = {"schema_version": 1, "world": _options["world"], "phase": _options["phase"], "started": Time.get_datetime_string_from_system(true), "build_profile": OS.get_environment("P12_BUILD_PROFILE"), "library": "res://rust/target/debug/libsim_bridge.dylib", "godot": Engine.get_version_info(), "machine": _machine(), "load_before": _command("/usr/sbin/sysctl", ["-n", "vm.loadavg"]), "contended": OS.get_environment("P12_CONTENDED") == "1", "metric": "process_ms + render_cpu_ms", "warmup_frames": 30, "counted_frames": 300, "ablation_counted_frames": 150, "forced_draw": true, "process_priorities": [-1000, 1000], "harness_reads": "peek_uncounted(raw getter) when Main/SimReads exists; direct fallback otherwise"}
	_result["checkout"] = ProjectSettings.globalize_path("res://").trim_suffix("/")
	_result["build_sha256"] = FileAccess.get_sha256("res://rust/target/debug/libsim_bridge.dylib")
	_result["release_sha256"] = FileAccess.get_sha256("res://rust/target/release/libsim_bridge.dylib")
	if _result["build_profile"] != "release" or str(_result["release_sha256"]).is_empty() or _result["build_sha256"] != _result["release_sha256"]:
		push_error("P12 requires P12_BUILD_PROFILE=release and identical debug/release dylib SHA256")
		quit(2)
		return
	_result["split_selection"] = {"running_block": OS.get_environment("P12_RUNNING_BLOCK"), "frozen_only": OS.get_environment("P12_FROZEN_ONLY") == "1"}
	_result["baseline_commit"] = OS.get_environment("P12_BASELINE_SHA")
	_result["collector_sha256"] = FileAccess.get_sha256("res://scripts/test/art/perf_frames_p12.gd")
	# Git's cwd is the runner repository, so bind HEAD explicitly to the selected checkout.
	_result["build_commit"] = _command("/usr/bin/git", ["-C", _result["checkout"], "rev-parse", "HEAD"])
	for pair: Array in [["checkout", "P12_MEASURE_CHECKOUT"], ["build_commit", "P12_MEASURE_HEAD"], ["build_sha256", "P12_MEASURE_DYLIB_SHA256"], ["collector_sha256", "P12_MEASURE_SCRIPT_SHA256"]]:
		if str(_result[pair[0]]) != OS.get_environment(pair[1]) or str(_result[pair[0]]).is_empty():
			push_error("Missing or mismatched measurement provenance: " + str(pair[0]))
			quit(2)
			return
	if _options["world"] == "W1000" and _options["phase"] != "after":
		push_error("W1000 is an after-only frozen Z4 measurement")
		quit(2)
		return

	var packed: PackedScene = load("res://scenes/main.tscn") as PackedScene
	_main = packed.instantiate()
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D") as Camera2D
	_world.call("set_sim_speed", 0.0)
	root.size = Vector2i(1920, 1080)
	root.add_child(_main)
	current_scene = _main
	_camera.call("set_auto_track", false)
	_camera.position_smoothing_enabled = false
	_prior_camera_mode = _camera.process_mode
	_camera.process_mode = Node.PROCESS_MODE_DISABLED
	var reader: Node = ReadRoute.resolve(_world)
	var spec: Array = WORLDS[_options["world"]]
	ReadRoute.mutate(_world, "reset_world_stress" if int(spec[2]) > 0 else "reset_world", [spec[0], spec[2]] if int(spec[2]) > 0 else [spec[0]])
	_world.call("set_sim_speed", 0.0)
	ReadRoute.mutate(_world, "advance_ticks", [spec[1]])
	_runtime = MeasurementRuntime.new()
	_runtime.tree = self
	_runtime.world = _world
	_runtime.camera = _camera
	_runtime.reader = reader
	var settled: Dictionary = _runtime.read("get_settlement_snapshot")
	var ids: PackedInt32Array = settled.get("settlement_ids", PackedInt32Array())
	if ids.is_empty():
		_errors.append("No settlement available for hearth framing")
		_finish()
		return
	var first: int = 0
	for i: int in ids.size():
		if ids[i] < ids[first]:
			first = i
	_camera.position = Vector2(448, 28) + Vector2(settled["formation_xs"][first], settled["formation_ys"][first]) * 16.0 + Vector2(8, 8)
	_result["framing_settlement_id"] = ids[first]
	_result["seed"] = spec[0]
	_result["world_info"] = _runtime.read("get_world_info")
	if _options["world"] == "W-multi":
		var info: Dictionary = _result["world_info"]
		if ((int(spec[1]) + int(info["clock_offset_ticks"])) % int(info["ticks_per_day"])) * 24 != int(info["ticks_per_day"]) * 10:
			_errors.append("W-multi is not at 10:00 under delivered world clock")
	_result["requested_population"] = spec[2]
	_result["requested_tick"] = spec[1]
	var prior_max_fps: int = Engine.max_fps
	Engine.max_fps = 0
	_result["initial"] = _runtime.begin()
	# begin() sees our already frozen camera; restore the mode from before setup.
	_runtime.prior_camera_mode = _prior_camera_mode
	_last_tick = int(_result["initial"]["tick"])
	if _last_tick != int(spec[1]):
		_errors.append("World did not reach requested tick")
	if bool(_options["ablation-only"]):
		_camera.zoom = Vector2(4, 4)
		await _run_ablation()
		_result["final"] = _runtime.snapshot()
		_result["restoration"] = _runtime.restore()
		Engine.max_fps = prior_max_fps
		_finish()
		return
	for zoom: int in ([] if not OS.get_environment("P12_RUNNING_BLOCK").is_empty() else [4] if _options["world"] == "W1000" else [4]):
		_camera.zoom = Vector2.ONE * zoom
		for repeat: int in range(3):
			_reset_running_world(spec)
			await _block("frozen_Z%d_%d" % [zoom, repeat], 300, "frozen", zoom, 0.0)
	_camera.zoom = Vector2(4, 4)
	if _options["world"] in ["W0", "W500"] and not bool(_options["skip-ablation"]) and OS.get_environment("P12_RUNNING_BLOCK").is_empty():
		await _run_ablation()
	Engine.max_fps = 60
	for speed: float in ([] if _options["world"] == "W1000" or OS.get_environment("P12_FROZEN_ONLY") == "1" else [2.0]):
		for repeat: int in range(3):
			var requested_block: String = OS.get_environment("P12_RUNNING_BLOCK")
			if not requested_block.is_empty() and requested_block != "running_%dx_%d" % [int(speed), repeat]:
				continue
			_reset_running_world(spec)
			_world.call("set_sim_speed", speed)
			await _block("running_%dx_%d" % [int(speed), repeat], 300, "running", 4, speed)
	_world.call("set_sim_speed", 0.0)
	_result["final"] = _runtime.snapshot()
	_result["restoration"] = _runtime.restore()
	Engine.max_fps = prior_max_fps
	_finish()

func _reset_running_world(spec: Array) -> void:
	_world.call("set_sim_speed", 0.0)
	ReadRoute.mutate(_world, "reset_world_stress" if int(spec[2]) > 0 else "reset_world", [spec[0], spec[2]] if int(spec[2]) > 0 else [spec[0]])
	_world.call("set_sim_speed", 0.0)
	ReadRoute.mutate(_world, "advance_ticks", [spec[1]])
	var settled: Dictionary = _runtime.read("get_settlement_snapshot")
	var ids: PackedInt32Array = settled.get("settlement_ids", PackedInt32Array())
	var first: int = -1
	for index: int in ids.size():
		if first < 0 or ids[index] < ids[first]:
			first = index
	if first < 0:
		_errors.append("Fresh running setup has no settlement")
	else:
		_camera.position = Vector2(448, 28) + Vector2(settled["formation_xs"][first], settled["formation_ys"][first]) * 16.0 + Vector2(8, 8)
	_camera.zoom = Vector2(4, 4)
	_last_tick = int(_runtime.read("get_time_snapshot")["tick"])
	if _last_tick != int(spec[1]):
		_errors.append("Fresh running setup missed requested tick")

func _sample() -> Dictionary:
	var sample: Dictionary = await _runtime.next_frame()
	var inspector: CanvasItem = _main.get_node_or_null("UI/AgentInspectorPanel") as CanvasItem
	var renderer: Node = _main.get_node_or_null("WorldRenderer")
	sample["direct_read_context"] = {"inspector_visible": inspector.visible if inspector != null else null, "influence_channel": renderer.get("current_channel") if renderer != null else null}
	var reader: Node = _runtime.reader
	if is_instance_valid(reader) and reader.has_method("frame_call_counts"):
		var counts: Dictionary = reader.call("frame_call_counts")
		sample["ffi_calls"] = counts.duplicate(true)
		sample["ffi_counter_frame"] = int(reader.call("frame_id")) if reader.has_method("frame_id") else -1
		if int(sample["ffi_counter_frame"]) != int(sample["process_frame"]):
			_errors.append("FFI counters do not match process frame: %s != %s" % [sample["ffi_counter_frame"], sample["process_frame"]])
		if reader.has_method("frame_call_usec"):
			var timings: Dictionary = reader.call("frame_call_usec")
			sample["ffi_call_usec"] = timings.duplicate(true)
	var tick: int = int(sample["state"]["tick"])
	sample["ticks_run"] = tick - _last_tick
	_last_tick = tick
	if not bool(sample.get("paired", false)):
		_errors.append("Unpaired process stamps at frame %s" % sample.get("process_frame"))
	for metric: String in METRICS:
		if not is_finite(float(sample[metric])) or float(sample[metric]) < 0.0:
			_errors.append("Invalid %s at frame %s" % [metric, sample.get("process_frame")])
	if _options["phase"] == "after" and _options["world"] == "W500" and not sample.has("ffi_calls"):
		_errors.append("After W500 requires per-frame SimReads FFI counters")
	return sample

func _block(label: String, count: int, condition: String, zoom: int, speed: float) -> Dictionary:
	var start_state: Dictionary = _runtime.snapshot()
	var discarded: Array[Dictionary] = []
	var samples: Array[Dictionary] = []
	for i: int in range(30):
		discarded.append(await _sample())
	for i: int in range(count):
		var sample: Dictionary = await _sample()
		sample["sample_index"] = i
		if speed == 0.0 and int(sample["ticks_run"]) != 0:
			_errors.append("Frozen block advanced ticks: " + label)
		samples.append(sample)
	if condition == "running" and _tick_samples(samples).is_empty():
		_errors.append("Running block has no tick-carrying frames: " + label)
	var block: Dictionary = {"label": label, "condition": condition, "zoom": zoom, "speed": speed, "max_fps": Engine.max_fps, "vsync_disabled": DisplayServer.window_get_vsync_mode() == DisplayServer.VSYNC_DISABLED, "camera_process_frozen": _camera.process_mode == Node.PROCESS_MODE_DISABLED, "window_size": [root.size.x, root.size.y], "samples": samples, "discarded": discarded, "statistics": _statistics(samples), "tick_carrying": _statistics(_tick_samples(samples))}
	block["start_state"] = start_state
	block["end_state"] = _runtime.snapshot()
	block["population_start"] = int(start_state["population"])
	block["population_end"] = int(block["end_state"]["population"])
	block["population_flagged"] = speed > 0.0 and float(block["population_end"]) < 0.9 * float(block["population_start"])
	block["fresh_setup"] = condition in ["frozen", "running"]
	_blocks.append(block)
	print("P12_BLOCK ", label, " median=", block["statistics"]["primary_cpu_ms"]["median"])
	return block

func _walk(node: Node) -> Array[Node]:
	var nodes: Array[Node] = []
	for child: Node in node.get_children():
		nodes.append(child)
		nodes.append_array(_walk(child))
	return nodes

func _state(targets: Array[Dictionary]) -> Array[Dictionary]:
	var state: Array[Dictionary] = []
	for target: Dictionary in targets:
		var node: Node = target["node"]
		var row: Dictionary = {"node": node, "process_mode": node.process_mode}
		if node is CanvasItem or node is CanvasLayer:
			row["visible"] = node.get("visible")
		state.append(row)
	return state

func _restore_state(state: Array[Dictionary]) -> void:
	for row: Dictionary in state:
		var node: Node = row["node"]
		if is_instance_valid(node):
			node.process_mode = int(row["process_mode"]) as Node.ProcessMode
			if row.has("visible"):
				node.set("visible", row["visible"])

func _state_matches(state: Array[Dictionary]) -> bool:
	for row: Dictionary in state:
		var node: Node = row["node"]
		if not is_instance_valid(node) or node.process_mode != int(row["process_mode"]):
			return false
		if row.has("visible") and node.get("visible") != row["visible"]:
			return false
	return true

func _disable(node: Node, layer_only: bool) -> void:
	if not layer_only:
		node.process_mode = Node.PROCESS_MODE_DISABLED
	if node is CanvasItem or node is CanvasLayer:
		node.set("visible", false)

func _ablation_block(label: String, condition: String) -> Dictionary:
	_reset_running_world(WORLDS[_options["world"]])
	var speed: float = 2.0 if _options["world"] == "W200" else 0.0
	Engine.max_fps = 60 if speed > 0.0 else 0
	_world.call("set_sim_speed", speed)
	var block: Dictionary = await _block(label, 150, condition, 4, speed)
	_world.call("set_sim_speed", 0.0)
	block["fresh_setup"] = true
	return block

func _run_ablation() -> void:
	var before: Dictionary = await _ablation_block("ablation_all_on_0", "ablation_reference")
	var targets: Array[Dictionary] = []
	for node: Node in _main.get_children() + (_main.get_node("UI").get_children() if _main.has_node("UI") else []):
		if node == _world or node == _camera or node.name in ["SimReads", "UI"]:
			continue
		if node.has_method("_process") or node.name not in ["WorldObjects", "Shadows", "NightChillLayer"]:
			var members: Array[String] = []
			for child: Node in _walk(node):
				members.append(str(_main.get_path_to(child)))
			targets.append({"node": node, "layer_only": false, "descendants": members})
		else:
			targets.append({"node": node, "layer_only": true, "descendants": []})
	# Keep stable targets, including both LOD-owned renderers; effect sprites expire normally.
	var saved: Array[Dictionary] = _state(targets)
	for target: Dictionary in targets:
		var node: Node = target["node"]
		var label: String = str(_main.get_path_to(node))
		_disable(node, bool(target["layer_only"]))
		var off: Dictionary = await _ablation_block("off_" + label, "ablation_off")
		_restore_state(saved)
		var after: Dictionary = await _ablation_block("on_after_" + label, "ablation_reference")
		var neighbours: Array = before["samples"].duplicate()
		neighbours.append_array(after["samples"])
		var cost: float = float(_statistics(neighbours)["primary_cpu_ms"]["median"]) - float(off["statistics"]["primary_cpu_ms"]["median"])
		_ablation.append({"node": label, "descendants_disabled_as_group": target["descendants"], "layer_only": target["layer_only"], "off_block": off["label"], "neighbour_blocks": [before["label"], after["label"]], "cost_ms": cost})
		before = after
	for target: Dictionary in targets:
		_disable(target["node"], bool(target["layer_only"]))
	await _ablation_block("ablation_floor", "ablation_floor")
	_restore_state(saved)
	await _ablation_block("ablation_restored", "ablation_reference")
	_result["ablation_visibility_restored"] = _state_matches(saved)
	if not bool(_result["ablation_visibility_restored"]):
		_errors.append("Ablation failed exact visibility/process mode restoration")

func _tick_samples(samples: Array) -> Array:
	return samples.filter(func(sample: Dictionary) -> bool: return int(sample["ticks_run"]) > 0)

func _statistics(samples: Array) -> Dictionary:
	var result: Dictionary = {"frames": samples.size(), "frames_over_16_67_ms": 0}
	for sample: Dictionary in samples:
		result["frames_over_16_67_ms"] += int(float(sample["primary_cpu_ms"]) > 16.67)
	for metric: String in METRICS:
		var values: Array[float] = []
		for sample: Dictionary in samples:
			values.append(float(sample[metric]))
		values.sort()
		result[metric] = {"median": Session.median(values) if not values.is_empty() else null, "p90": _percentile(values, 0.90), "p99": _percentile(values, 0.99), "max": values[-1] if not values.is_empty() else null}
	return result

func _percentile(values: Array[float], fraction: float) -> Variant:
	return values[clampi(int(ceil(values.size() * fraction)) - 1, 0, values.size() - 1)] if not values.is_empty() else null

func _command(executable: String, args: Array) -> String:
	var output: Array = []
	var code: int = OS.execute(executable, PackedStringArray(args), output, true)
	return "".join(output).strip_edges() if code == 0 else "unavailable (exit %d): %s" % [code, "".join(output)]

func _machine() -> Dictionary:
	return {"cpu": _command("/usr/sbin/sysctl", ["-n", "machdep.cpu.brand_string"]), "cpus": _command("/usr/sbin/sysctl", ["-n", "hw.ncpu"]), "memory_bytes": _command("/usr/sbin/sysctl", ["-n", "hw.memsize"]), "macos": _command("/usr/bin/sw_vers", []), "os": OS.get_version()}

func _finish() -> void:
	if _result.has("restoration"):
		for key: String in ["vsync_restored", "layers_restored", "camera_process_restored", "probes_removed"]:
			if not bool(_result["restoration"].get(key, false)):
				_errors.append("Runtime restoration failed: " + key)
	else:
		_camera.process_mode = _prior_camera_mode as Node.ProcessMode
		_errors.append("Measurement runtime did not reach restoration")
	var pooled: Dictionary = {}
	var excluded: Array[String] = []
	var render_positive: int = 0
	for block: Dictionary in _blocks:
		for sample: Dictionary in block["samples"]:
			render_positive += int(float(sample["render_cpu_ms"]) > 0.0)
		if block["condition"] not in ["frozen", "running"]:
			continue
		if bool(block["population_flagged"]):
			excluded.append(str(block["label"]))
			continue
		var key: String = "%s_Z%d_%sx" % [block["condition"], block["zoom"], block["speed"]]
		if not pooled.has(key):
			pooled[key] = []
		pooled[key].append_array(block["samples"])
	var summaries: Dictionary = {}
	_result["pooled_excluded_blocks"] = excluded
	for key: String in pooled:
		summaries[key] = {"all_frames": _statistics(pooled[key]), "tick_carrying": _statistics(_tick_samples(pooled[key]))}
	if render_positive == 0:
		_errors.append("Render CPU unavailable: every counted sample is zero")
	_result.merge({"blocks": _blocks, "pooled": summaries, "ablation": _ablation, "ablation_dropped": _options["skip-ablation"], "ablation_only": _options["ablation-only"], "finished": Time.get_datetime_string_from_system(true), "load_after": _command("/usr/sbin/sysctl", ["-n", "vm.loadavg"]), "errors": _errors, "valid": _errors.is_empty()})
	var output_path: String = ProjectSettings.globalize_path(str(_options["out"]))
	DirAccess.make_dir_recursive_absolute(output_path.get_base_dir())
	var file: FileAccess = FileAccess.open(output_path, FileAccess.WRITE)
	if file == null:
		push_error("Cannot write " + output_path)
		quit(2)
		return
	file.store_string(JSON.stringify(_result, "\t"))
	file.close()
	quit(0 if _errors.is_empty() else 1)
