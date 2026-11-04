# M-UX2 normal debug gameplay stall measurement: seed 0, steward ON, zoom 1 and 0.5.
# Bridge methods: reset_world, cmd_set_steward, get_time_snapshot, get_steward, get_agent_snapshot. Monitor optionally calls mux2_native_timings.
# Automatic accumulator stays active. Windowed only; each arm has 10 startup + 300 measured seconds.
extends SceneTree

const Monitor = preload("res://tools/v8/mux2_perf_monitor.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
const MAX_SECONDS: float = 900.0
const P7_WAIT_SECONDS: float = 900.0
const P7_MAX_WAITS: int = 8
const SOURCE_PATHS: Array[String] = [
	"tools/v8/mux2_stall_test.gd", "tools/v8/mux2_perf_monitor.gd", "project.godot", "sim_bridge.gdextension", "rust/Cargo.toml", "rust/Cargo.lock",
	"scripts/core/sim_reads.gd", "scripts/world/camera_controller.gd", "scripts/world/colony/ground_layer.gd", "scripts/world/art/ground_baker.gd", "scripts/world/art/wall_baker.gd", "scripts/world/colony/building_layer.gd",
	"scripts/ui/mux/minimap.gd", "scripts/ui/mux/alerts_panel.gd", "scripts/ui/mux/alert_evaluator.gd", "scripts/ui/mux/mux_hud.gd", "scripts/ui/mux/learning_facts.gd",
	"rust/crates/sim-bridge/src/ffi/node.rs", "rust/crates/sim-systems/src/runtime/steward/mod.rs", "rust/crates/sim-core/data/colony/steward.ron",
]
var _out: String
var _settings_dir: String
var _main: Node
var _world: Node
var _camera: Camera2D
var _errors: Array[String] = []
var _report: Dictionary = {}
var _finished: bool = false
var _diagnostic: bool = false
var _p7: bool = OS.has_environment("CORE_P7_RUN")
var _p7_deadline: float = 0.0

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	if args.size() == 1 and args[0] == "--self-test":
		_errors = Monitor.self_test()
		_errors.append_array(_admission_self_test())
		print("MUX2_STALL_SELF_TEST failures=%s" % [_errors])
		quit(0 if _errors.is_empty() else 1)
		return
	if args.size() == 2 and args[1] == "--diagnostic-120":
		_diagnostic = true
		args.resize(1)
	if args.size() != 1:
		push_error("usage: mux2_stall_test.gd -- /absolute/external/new-output-directory [--diagnostic-120]")
		quit(1)
		return
	_out = args[0]
	_settings_dir = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	if not _external(_out) or not _external(_settings_dir) or DirAccess.dir_exists_absolute(_out):
		push_error("MUX2_STALL requires new external output directory and external WORLDSIM_TEST_SETTINGS_DIR")
		quit(1)
		return
	if DisplayServer.get_name() == "headless" or not OS.has_feature("debug"):
		push_error("MUX2_STALL requires windowed debug Godot, not dummy rendering")
		quit(1)
		return
	if DirAccess.make_dir_recursive_absolute(_out) != OK or DirAccess.make_dir_recursive_absolute(_settings_dir) != OK:
		push_error("MUX2_STALL cannot create evidence directories")
		quit(1)
		return
	_report = {"started_utc": Time.get_datetime_string_from_system(true), "seed": 0, "speed": 1, "steward_enabled": true, "arms": [], "errors": _errors, "complete": false, "pass": false,
		"purpose": "diagnostic_120_seconds_zoom1" if _diagnostic else "acceptance_5_minutes_each_zoom",
		"environment": _environment(), "source_sha256": _hashes(), "bridge": _bridge(),
		"measurement": "RenderingServer.frame_post_draw wall-clock intervals. Fresh seed 0 per zoom. No manual ticks; all ordinary UI and camera processing stays enabled. No frame cap or vsync override.",
		"limits": "Selected source paths and configured library on disk are hashed, not process memory. Parent must freeze entire checkout. Performance monitors can lag and are corroboration, not exclusive CPU attribution. Nested timings must not be summed. Sampling overhead is included in subsequent frame intervals; file output occurs after pausing each arm.",
		"gc": {"status": "NOT_MEASURED" if OS.has_feature("mono") else "NOT_APPLICABLE", "reason": "No tracing-GC timing API used. Native GDScript resources use reference counting; allocator/free/deferred node destruction costs remain part of frame time and are not assumed zero."},
	}
	if not _errors.is_empty():
		_finish()
		return
	auto_accept_quit = false
	root.close_requested.connect(_closed)
	var timeout: float = MAX_SECONDS
	if _p7:
		var deadline_text: String = OS.get_environment("CORE_P7_DEADLINE_EPOCH")
		if not deadline_text.is_valid_float() or not is_finite(float(deadline_text)):
			_errors.append("NOT_RUN: P7 requires finite CORE_P7_DEADLINE_EPOCH")
			_finish()
			return
		_p7_deadline = float(deadline_text)
		timeout = minf(MAX_SECONDS + 2.0 * P7_MAX_WAITS * P7_WAIT_SECONDS, _p7_deadline - Time.get_unix_time_from_system())
		if timeout <= 0.0:
			_errors.append("NOT_RUN: P7 deadline expired")
			_finish()
			return
		_report["p7_admission"] = {"deadline_epoch": _p7_deadline, "load1_limit_exclusive": 8.0, "wait_seconds": P7_WAIT_SECONDS, "max_waits": P7_MAX_WAITS}
	create_timer(timeout).timeout.connect(_timed_out)
	var zoom_levels: Array[float] = [1.0, 0.5]
	if _diagnostic:
		zoom_levels = [1.0]
	for zoom_level: float in zoom_levels:
		await _phase(zoom_level)
		if not _errors.is_empty(): break
	if _hashes() != _report["source_sha256"] or _bridge() != _report["bridge"]:
		_errors.append("source or configured bridge changed during measurement")
	_finish()

func _phase(zoom_level: float) -> void:
	if is_instance_valid(_main):
		_main.queue_free()
		await process_frame
		await process_frame
	var settings_path: String = _settings_dir.path_join("stall-%d-%s.cfg" % [OS.get_process_id(), str(zoom_level)])
	if FileAccess.file_exists(settings_path):
		_errors.append("refusing existing arm settings")
		return
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	_main.get_node("UI/MuxHud").set("store", Store.new(settings_path))
	root.add_child(_main)
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D") as Camera2D
	_camera.call("set_paused", true)
	if not bool(_main.get("boot_ok")) or not bool(SimReads.mutate(_world, "reset_world", [0])):
		_errors.append("bootstrap or reset failed")
		return
	var command: Dictionary = SimReads.mutate(_world, "cmd_set_steward", [true, 0])
	if not bool(command.get("accepted", false)):
		_errors.append("SetSteward command rejected")
		return
	_camera.call("set_speed", 1.0)
	_camera.call("set_edge_pan", false)
	# A normal ladder request settles before centering; no process callback is disabled.
	if zoom_level < 1.0: _camera.call("step_zoom", -1, root.get_visible_rect().size * 0.5)
	for _frame: int in 120:
		await RenderingServer.frame_post_draw
		if is_equal_approx(_camera.zoom.x, zoom_level): break
	_camera.call("center_on_camp")
	await RenderingServer.frame_post_draw
	if not is_equal_approx(_camera.zoom.x, zoom_level):
		_errors.append("requested zoom did not settle")
		return
	if _tick() != 0:
		_errors.append("paused preparation advanced the fresh world")
		return
	var monitor: Monitor = Monitor.new()
	if not _p7: monitor.setup(_main)
	var samples: Array[Dictionary] = []
	# Paused preparation disables WorldSim processing; native frame coverage starts with the first measured frame.
	var arm: Dictionary = {"zoom": zoom_level, "samples": samples, "instrumentation": {"complete": false, "status": "NOT_SAMPLED"}, "start_tick": _tick(), "population_start": _population(), "view": _view(), "complete": false}
	(_report["arms"] as Array).append(arm)
	if _p7:
		if not await _admit_arm(arm):
			return
		if _tick() != 0 or not bool(_camera.call("is_paused")) or _view() != arm["view"]:
			_errors.append("NOT_RUN: P7 paused preparation changed during admission")
			return
		monitor.setup(_main)
		arm["sampling_started_epoch"] = Time.get_unix_time_from_system()
	var previous: int = Time.get_ticks_usec()
	monitor.begin_sampling(previous)
	var started: int = previous
	var previous_tick: int = _tick()
	_camera.call("set_paused", false)
	var required_seconds: float = 120.0 if _diagnostic else Monitor.TOTAL_SECONDS
	while float(previous - started) / 1000000.0 < required_seconds:
		await RenderingServer.frame_post_draw
		var drawn: int = Time.get_ticks_usec()
		var sample_start: int = drawn
		var tick: int = _tick()
		var row: Dictionary = {"elapsed_seconds": float(drawn - started) / 1000000.0, "interval_ms": float(drawn - previous) / 1000.0, "post_draw_usec": drawn, "tick": tick, "ticks_advanced": tick - previous_tick, "telemetry": monitor.sample(drawn)}
		samples.append(row)
		# The normal first-tick camp centering is part of startup, not a camera fault.
		if float(row["elapsed_seconds"]) <= Monitor.WARMUP_SECONDS: arm["view"] = _view()
		if tick < previous_tick or tick - previous_tick > 8 or _view() != arm["view"] or paused or bool(_camera.call("is_paused")) or not is_equal_approx(float(_camera.call("get_speed")), 1.0) or not _world.is_processing() or _world.process_mode != Node.PROCESS_MODE_INHERIT or not is_equal_approx(Engine.time_scale, 1.0):
			_errors.append("tick, view, speed or automatic processing changed")
			break
		previous = drawn
		previous_tick = tick
		row["sampling_overhead_usec"] = Time.get_ticks_usec() - sample_start
	_camera.call("set_paused", true)
	var elapsed: float = float(previous - started) / 1000000.0
	arm["score"] = Monitor.score(samples, elapsed)
	arm["instrumentation"] = monitor.coverage()
	monitor.finish()
	arm["wall_warmup"] = _main.get_node("ColonyView/GroundLayer").call("mux2_wall_warmup_report")
	arm["end_tick"] = _tick()
	arm["population_end"] = _population()
	arm["observed_ticks_per_second"] = float(int(arm["end_tick"]) - int(arm["start_tick"])) / maxf(elapsed, 0.001)
	var state: Dictionary = _world.get_steward()
	arm["final_steward"] = state
	arm["capture_complete"] = elapsed >= required_seconds and bool(state.get("enabled", false)) and _errors.is_empty() and int(arm["end_tick"]) > int(arm["start_tick"])
	arm["complete"] = bool(arm["score"]["complete"]) and bool(arm["capture_complete"])
	_write_report()
	print("MUX2_STALL_ARM zoom=%s complete=%s pass=%s >50=%s >33/min=%s" % [zoom_level, arm["complete"], arm["score"]["pass"], arm["score"]["post_warmup_over50"], arm["score"]["over33_per_minute"]])

# Pure decision helper: fake samples exercise boundaries without timing or load probes.
static func _admission_action(loads: Array, waits: int, remaining: float, required: float) -> String:
	if loads.size() != 3 or not is_finite(remaining) or remaining < required:
		return "NOT_RUN"
	for value: Variant in loads:
		if not (value is float or value is int) or not is_finite(float(value)) or float(value) < 0.0:
			return "NOT_RUN"
	if float(loads[0]) < 8.0:
		return "ADMITTED"
	if waits >= P7_MAX_WAITS or remaining < P7_WAIT_SECONDS + required:
		return "NOT_RUN"
	return "WAIT"

static func _admission_self_test() -> Array[String]:
	var errors: Array[String] = []
	var cases: Array = [
		[[7.99, 12.0, 15.0], 0, 2000.0, "ADMITTED"],
		[[8.0, 1.0, 1.0], 0, 2000.0, "WAIT"],
		[[9.0, 1.0, 1.0], 8, 2000.0, "NOT_RUN"],
		[[7.0, 1.0, 1.0], 8, 2000.0, "ADMITTED"],
		[[7.0, 1.0, 1.0], 0, 309.0, "NOT_RUN"],
		[[8.0, 1.0, 1.0], 0, 1209.0, "NOT_RUN"],
		[[NAN, 1.0, 1.0], 0, 2000.0, "NOT_RUN"],
		[[1.0, INF, 1.0], 0, 2000.0, "NOT_RUN"],
		[[1.0, -1.0, 1.0], 0, 2000.0, "NOT_RUN"],
		[[1.0], 0, 2000.0, "NOT_RUN"],
	]
	for row: Array in cases:
		if _admission_action(row[0], row[1], row[2], 310.0) != row[3]:
			errors.append("P7 admission boundary: " + str(row))
	return errors

func _admit_arm(arm: Dictionary) -> bool:
	var history: Array[Dictionary] = []
	arm["admission"] = {"status": "NOT_RUN", "samples": history}
	# Reserve both complete arms on the first zoom, plus report/shutdown time.
	var arms_left: int = 1 if _diagnostic or float(arm["zoom"]) < 1.0 else 2
	var required: float = (120.0 if _diagnostic else Monitor.TOTAL_SECONDS) * arms_left + 30.0
	for waits: int in P7_MAX_WAITS + 1:
		if _finished: return false
		# Timer resumption can be mid-frame; admit only at a fresh post-draw boundary.
		await RenderingServer.frame_post_draw
		if _finished: return false
		var output: Array = []
		var code: int = OS.execute("/usr/sbin/sysctl", PackedStringArray(["-n", "vm.loadavg"]), output)
		var loads: Array = []
		if code == 0 and output.size() == 1:
			var parts: PackedStringArray = str(output[0]).replace("{", "").replace("}", "").strip_edges().split(" ", false)
			for part: String in parts:
				if not part.is_valid_float():
					loads.clear()
					break
				loads.append(float(part))
		var epoch: float = Time.get_unix_time_from_system()
		var action: String = _admission_action(loads, waits, _p7_deadline - epoch, required)
		history.append({"epoch": epoch, "utc": Time.get_datetime_string_from_system(true), "loads_1_5_15": loads, "waits": waits, "action": action, "sysctl_exit": code, "raw": output})
		arm["admission"]["status"] = action
		if not _write_report(): return false
		if action == "ADMITTED": return true
		if action == "NOT_RUN":
			_errors.append("NOT_RUN: P7 zoom %s admission unavailable, contended or deadline exhausted" % arm["zoom"])
			return false
		# World remains paused; admission waiting precedes monitor.begin_sampling.
		await create_timer(P7_WAIT_SECONDS, true, false, true).timeout
	return false

func _tick() -> int:
	var value: Dictionary = _world.get_time_snapshot()
	return int(value.get("tick", -1))

func _population() -> int:
	var value: Dictionary = _world.get_agent_snapshot()
	return (value.get("agent_ids", PackedInt64Array()) as PackedInt64Array).size()

func _view() -> Dictionary:
	return {"position": str(_camera.position), "zoom": str(_camera.zoom), "viewport": str(root.get_visible_rect()), "window_size": str(root.size), "window_mode": DisplayServer.window_get_mode()}

func _external(path: String) -> bool:
	var project: String = ProjectSettings.globalize_path("res://").simplify_path().trim_suffix("/")
	var normalized: String = path.simplify_path()
	return path.is_absolute_path() and normalized != project and not normalized.begins_with(project + "/")

func _environment() -> Dictionary:
	return {"engine": Engine.get_version_info(), "os": OS.get_name(), "cpu": OS.get_processor_name(), "gpu": RenderingServer.get_video_adapter_name(), "render_driver": RenderingServer.get_current_rendering_driver_name(), "render_method": RenderingServer.get_current_rendering_method(), "debug": OS.has_feature("debug"), "vsync": DisplayServer.window_get_vsync_mode(), "max_fps": Engine.max_fps, "settings_dir": _settings_dir}

func _hashes() -> Dictionary:
	var result: Dictionary = {}
	for path: String in SOURCE_PATHS:
		var digest: String = FileAccess.get_sha256("res://" + path)
		if digest.length() != 64: _errors.append("source SHA256 missing: " + path)
		result[path] = digest
	return result

func _bridge() -> Dictionary:
	var config: ConfigFile = ConfigFile.new()
	var error: Error = config.load("res://sim_bridge.gdextension")
	var path: String = str(config.get_value("libraries", "macos.debug", "")) if error == OK else ""
	var loaded: bool = false
	for descriptor: String in GDExtensionManager.get_loaded_extensions():
		if ProjectSettings.globalize_path(descriptor) == ProjectSettings.globalize_path("res://sim_bridge.gdextension"): loaded = true
	var digest: String = FileAccess.get_sha256(path) if not path.is_empty() else ""
	if not loaded or digest.length() != 64: _errors.append("configured debug bridge not loaded or hash unavailable")
	return {"path": path, "sha256": digest, "descriptor_loaded": loaded}

func _finish() -> void:
	if _finished: return
	_finished = true
	var arms: Array = _report.get("arms", [])
	var complete: bool = arms.size() == 2 and _errors.is_empty()
	var passed: bool = complete
	var profiled: bool = complete
	for arm: Dictionary in arms:
		complete = complete and bool(arm.get("complete", false))
		passed = passed and bool((arm.get("score", {}) as Dictionary).get("pass", false))
		profiled = profiled and bool((arm.get("instrumentation", {}) as Dictionary).get("complete", false))
	_report["complete"] = complete
	_report["bar_pass"] = complete and passed
	_report["profile_complete"] = complete and profiled
	_report["pass"] = complete and passed and profiled
	var diagnostic_complete: bool = _diagnostic and arms.size() == 1 and _errors.is_empty() and bool(arms[0].get("capture_complete", false)) and bool((arms[0].get("instrumentation", {}) as Dictionary).get("complete", false))
	_report["diagnostic_complete"] = diagnostic_complete
	_report["finished_utc"] = Time.get_datetime_string_from_system(true)
	if not _write_report():
		_report["pass"] = false
		diagnostic_complete = false
	print("MUX2_STALL_DONE complete=%s pass=%s errors=%s evidence=%s" % [complete, _report["pass"], _errors, _out.path_join("stall-report.json")])
	quit(0 if diagnostic_complete or bool(_report["pass"]) else 1)

func _write_report() -> bool:
	var file: FileAccess = FileAccess.open(_out.path_join("stall-report.json"), FileAccess.WRITE)
	if file == null:
		_errors.append("cannot write report")
		return false
	file.store_string(JSON.stringify(_report, "\t"))
	file.close()
	return true

func _closed() -> void:
	_errors.append("window closed before completion")
	_finish()

func _timed_out() -> void:
	_errors.append("NOT_RUN: P7 bounded capture watchdog expired" if _p7 else "900 second self-watchdog expired")
	_finish()
