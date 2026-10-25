# M-UX gameplay timing: actual main scene and normal WorldSim wall-time accumulator.
# Bridge methods: reset_world, cmd_set_steward, get_time_snapshot, get_world_info,
# get_steward, get_agent_snapshot; camera uses set_sim_speed. No manual tick stepping.
# Windowed debug only; parent builds/runs: --script tools/v8/mux_gameplay_perf.gd -- /external/dir
extends SceneTree

const Hud = preload("res://scripts/ui/mux/mux_hud.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
const WARMUP_FRAMES: int = 120
const SAMPLE_TICKS: int = 1441
const MAX_SECONDS: float = 720.0
# Matches the hashed bridge node.rs accumulator guard; reject unexpected multiple stepping.
const MAX_TICKS_PER_FRAME: int = 8
const SOURCE_PATHS: Array[String] = [
	"res://tools/v8/mux_gameplay_perf.gd", "res://sim_bridge.gdextension", "res://rust/Cargo.toml",
	"res://rust/crates/sim-systems/src/runtime/steward/planning.rs",
	"res://rust/crates/sim-systems/src/runtime/steward/production.rs",
	"res://rust/crates/sim-systems/src/runtime/steward/lifecycle.rs",
	"res://rust/crates/sim-systems/src/runtime/steward/mod.rs",
	"res://rust/crates/sim-systems/src/commands.rs", "res://rust/crates/sim-bridge/src/ffi/node.rs",
	"res://rust/crates/sim-core/data/colony/steward.ron",
	"res://scripts/world/camera_controller.gd", "res://scripts/core/sim_reads.gd",
]
var _out: String
var _main: Node
var _world: Node
var _camera: Camera2D
var _report: Dictionary = {}
var _errors: Array[String] = []

func _init() -> void:
	call_deferred("_run")

func _external(path: String) -> bool:
	var project: String = ProjectSettings.globalize_path("res://").simplify_path().trim_suffix("/")
	var normalized: String = path.simplify_path()
	return path.is_absolute_path() and normalized != project and not normalized.begins_with(project + "/")

func _run() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	_out = args[0] if not args.is_empty() else ""
	var settings_dir: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	if not _external(_out) or not _external(settings_dir):
		push_error("MUX_GAMEPLAY_PERF requires external absolute output and WORLDSIM_TEST_SETTINGS_DIR")
		quit(1)
		return
	if DirAccess.make_dir_recursive_absolute(_out) != OK or DirAccess.make_dir_recursive_absolute(settings_dir) != OK:
		push_error("MUX_GAMEPLAY_PERF cannot create external directories")
		quit(1)
		return
	if FileAccess.file_exists(_out.path_join("mux-gameplay-perf.json")):
		push_error("MUX_GAMEPLAY_PERF refuses to overwrite existing evidence")
		quit(1)
		return
	if DisplayServer.get_name() == "headless" or not OS.has_feature("debug"):
		push_error("MUX_GAMEPLAY_PERF requires windowed debug Godot")
		quit(1)
		return
	_report = {
		"started_utc": Time.get_datetime_string_from_system(true), "seed": 0,
		"scope": "Real automatic-accumulator gameplay, fresh default seed0 population, >=1441 ticks per arm at 1x/4x. Not N1000 or full-capacity acceptance.",
		"limitations": "Sequential OFF/ON arms; shared renderer caches, background load, physical display/vsync limits and engine delta clamping can affect results. Rendered frame intervals include UI, pacing and harness overhead; no exclusive system CPU attribution. No process-memory binary hash claim.",
		"warmup_rendered_frames": WARMUP_FRAMES, "minimum_ticks_per_arm": SAMPLE_TICKS,
		"watchdog_seconds": MAX_SECONDS, "environment": _environment(),
		"source_files_on_disk": _source_hashes(), "bridge": _bridge_metadata(),
		"settings_directory": settings_dir, "phases": [], "errors": _errors, "complete": false,
	}
	if not _errors.is_empty():
		_write_report()
		quit(1)
		return
	auto_accept_quit = false
	root.close_requested.connect(_closed)
	create_timer(MAX_SECONDS).timeout.connect(_timed_out)
	for speed: int in [1, 4]:
		for enabled: bool in [false, true]:
			await _phase(enabled, speed, settings_dir)
			if not _errors.is_empty(): break
		if not _errors.is_empty(): break
	if _source_hashes() != _report["source_files_on_disk"] or _bridge_metadata() != _report["bridge"]:
		_errors.append("source or bridge provenance changed during measurement")
	_report["complete"] = _errors.is_empty() and (_report["phases"] as Array).size() == 4
	_report["finished_utc"] = Time.get_datetime_string_from_system(true)
	_write_report()
	print("MUX_GAMEPLAY_PERF_DONE complete=%s errors=%s evidence=%s" % [_report["complete"], _errors, _out.path_join("mux-gameplay-perf.json")])
	quit(0 if bool(_report["complete"]) else 1)

func _tick() -> int:
	# Direct read after draw avoids a UI-populated per-frame SimReads snapshot predating WorldSim.process.
	var snapshot: Dictionary = _world.call("get_time_snapshot")
	return int(snapshot.get("tick", -1))

func _population() -> int:
	var snapshot: Dictionary = _world.call("get_agent_snapshot")
	return (snapshot.get("agent_ids", PackedInt64Array()) as PackedInt64Array).size()

func _phase(enabled: bool, speed: int, settings_dir: String) -> void:
	if is_instance_valid(_main):
		_main.queue_free()
		await process_frame
		await process_frame
	var label: String = "%dx-%s" % [speed, "on" if enabled else "off"]
	var settings_path: String = settings_dir.path_join("gameplay-%d-%s.cfg" % [OS.get_process_id(), label])
	if FileAccess.file_exists(settings_path):
		_errors.append("refusing existing arm settings: " + settings_path)
		return
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var hud: Hud = _main.get_node("UI/MuxHud") as Hud
	hud.store = Store.new(settings_path)
	_camera = _main.get_node("Camera2D") as Camera2D
	root.add_child(_main)
	if not bool(_main.get("boot_ok")):
		_errors.append("content bootstrap failed")
		return
	_world = _main.get_node("WorldSim")
	_camera.call("set_paused", true)
	if not bool(SimReads.mutate(_world, "reset_world", [0])):
		_errors.append("reset_world failed")
		return
	var command: Dictionary = SimReads.mutate(_world, "cmd_set_steward", [enabled, 0])
	if not bool(command.get("accepted", false)):
		_errors.append("ordinary SetSteward command rejected")
		return
	_camera.call("set_speed", float(speed))
	_camera.set_process(false)
	_camera.set_process_input(false)
	_camera.set_process_unhandled_input(false)
	_camera.zoom = Vector2.ONE
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	_camera.call("center_on_tile", Vector2i(floori(float(info.get("width", 0)) / 2.0), floori(float(info.get("height", 0)) / 2.0)))
	for _frame: int in WARMUP_FRAMES: await RenderingServer.frame_post_draw
	var stable_view: Dictionary = _view()
	var steward: Dictionary = _world.call("get_steward")
	if _tick() != 0 or bool(steward.get("enabled", not enabled)) != enabled or not bool(_camera.call("is_paused")):
		_errors.append("warmup changed tick zero or requested mode")
		return
	var phases: Array = _report["phases"]
	if not phases.is_empty() and phases[0]["view"] != stable_view:
		_errors.append("camera/window differs across arms")
		return
	var samples: Array[Dictionary] = []
	var intervals: Array[float] = []
	var boundaries: Array[Dictionary] = []
	var phase: Dictionary = {"label": label, "steward_enabled": enabled, "speed": speed, "expected_ticks_per_second": 12 * speed, "world": info, "view": stable_view, "population_start": _population(), "start_tick": 0, "raw_samples": samples, "planning_boundary_frames": boundaries, "complete": false}
	phases.append(phase)
	var previous_tick: int = 0
	var previous_draw: int = Time.get_ticks_usec()
	var started: int = previous_draw
	var next_boundary: int = 0
	_camera.call("set_paused", false)
	while previous_tick < SAMPLE_TICKS:
		await RenderingServer.frame_post_draw
		var drawn: int = Time.get_ticks_usec()
		var tick: int = _tick()
		var interval: float = float(drawn - previous_draw) / 1000.0
		steward = _world.call("get_steward")
		var crossed: Array[int] = []
		while next_boundary < tick:
			if next_boundary >= previous_tick: crossed.append(next_boundary)
			next_boundary += 360
		var sample: Dictionary = {"tick_before": previous_tick, "tick_after": tick, "tick_delta": tick - previous_tick, "post_draw_usec": drawn, "post_draw_interval_ms": interval, "elapsed_seconds": float(drawn - started) / 1000000.0, "process_frame": Engine.get_process_frames(), "executed_planning_boundaries": crossed}
		samples.append(sample)
		intervals.append(interval)
		if not crossed.is_empty(): boundaries.append(sample)
		if tick < previous_tick or tick - previous_tick > MAX_TICKS_PER_FRAME or not is_equal_approx(Engine.time_scale, 1.0) or paused or _view() != stable_view or bool(_camera.call("is_paused")) or not is_equal_approx(float(_camera.call("get_speed")), float(speed)) or not _world.is_processing() or _world.process_mode != Node.PROCESS_MODE_INHERIT or bool(steward.get("enabled", not enabled)) != enabled:
			_errors.append("arm %s tick/view/cadence/mode guard failed" % label)
			break
		previous_tick = tick
		previous_draw = drawn
	_camera.call("set_paused", true)
	var wall_seconds: float = float(previous_draw - started) / 1000000.0
	phase["end_tick"] = _tick()
	phase["wall_seconds"] = wall_seconds
	phase["observed_ticks_per_second"] = float(previous_tick) / wall_seconds if wall_seconds > 0.0 else 0.0
	phase["cadence_fraction_of_expected"] = float(phase["observed_ticks_per_second"]) / float(12 * speed)
	phase["population_end"] = _population()
	phase["frame_interval_ms"] = _summary(intervals)
	var summary: Dictionary = phase["frame_interval_ms"]
	phase["fps_derivation"] = "mean FPS = frame count / sampled wall seconds; median FPS = 1000 / median frame interval; not physical display refresh proof"
	phase["mean_fps"] = float(samples.size()) / wall_seconds if wall_seconds > 0.0 else 0.0
	phase["median_fps"] = 1000.0 / float(summary.get("median", INF))
	phase["complete"] = previous_tick >= SAMPLE_TICKS and boundaries.size() == 5 and _errors.is_empty()
	if not bool(phase["complete"]) and _errors.is_empty(): _errors.append("arm failed to include all five planning boundaries: " + label)
	_write_report()
	print("MUX_GAMEPLAY_PERF arm=%s complete=%s ticks=%d seconds=%.3f TPS=%.3f frames=%s evidence=%s" % [label, phase["complete"], previous_tick, wall_seconds, phase["observed_ticks_per_second"], summary, _out.path_join("mux-gameplay-perf.json")])

func _view() -> Dictionary:
	return {"camera_position": [_camera.position.x, _camera.position.y], "camera_zoom": [_camera.zoom.x, _camera.zoom.y], "window_size": [root.size.x, root.size.y], "viewport": str(root.get_visible_rect()), "window_mode": DisplayServer.window_get_mode()}

func _environment() -> Dictionary:
	return {"engine": Engine.get_version_info(), "os": OS.get_name(), "os_version": OS.get_version(), "cpu": OS.get_processor_name(), "cpu_count": OS.get_processor_count(), "gpu": RenderingServer.get_video_adapter_name(), "gpu_vendor": RenderingServer.get_video_adapter_vendor(), "render_method": RenderingServer.get_current_rendering_method(), "render_driver": RenderingServer.get_current_rendering_driver_name(), "debug_feature": OS.has_feature("debug"), "editor_feature": OS.has_feature("editor"), "vsync": DisplayServer.window_get_vsync_mode(), "max_fps": Engine.max_fps, "time_scale": Engine.time_scale, "physics_ticks_per_second": Engine.physics_ticks_per_second}

func _source_hashes() -> Dictionary:
	var result: Dictionary = {}
	for path: String in SOURCE_PATHS: result[path] = _required_hash(path)
	return result

func _required_hash(path: String) -> String:
	var digest: String = FileAccess.get_sha256(path) if not path.is_empty() and FileAccess.file_exists(path) else ""
	if digest.length() != 64:
		_errors.append("required SHA256 missing or unreadable: " + path)
		return "MISSING"
	return digest

func _bridge_metadata() -> Dictionary:
	var config: ConfigFile = ConfigFile.new()
	var error: Error = config.load("res://sim_bridge.gdextension")
	var platform: String = "macos" if OS.get_name() == "macOS" else ("windows" if OS.get_name() == "Windows" else "linux")
	var key: String = platform + ".debug"
	if platform != "macos": key += "." + ("x86_64" if OS.has_feature("x86_64") else "arm64")
	var path: String = str(config.get_value("libraries", key, "")) if error == OK else ""
	var loaded: PackedStringArray = GDExtensionManager.get_loaded_extensions()
	var descriptor_loaded: bool = false
	for descriptor: String in loaded:
		if ProjectSettings.globalize_path(descriptor) == ProjectSettings.globalize_path("res://sim_bridge.gdextension"):
			descriptor_loaded = true
	if error != OK or path.is_empty() or not descriptor_loaded:
		_errors.append("required loaded bridge descriptor or feature mapping missing: " + key)
	return {"loaded_extension_descriptors": loaded, "configured_feature_key": key, "configured_library_path": ProjectSettings.globalize_path(path), "configured_library_sha256_on_disk": _required_hash(path), "hash_scope": "Configured debug library on disk, not independently verified process memory. Freeze bridge and source while running."}

func _summary(values: Array[float]) -> Dictionary:
	if values.is_empty(): return {"count": 0}
	var ordered: Array[float] = values.duplicate()
	ordered.sort()
	var total: float = 0.0
	for value: float in ordered: total += value
	var middle: int = floori(float(ordered.size()) / 2.0)
	var median: float = ordered[middle] if ordered.size() % 2 == 1 else (ordered[middle - 1] + ordered[middle]) / 2.0
	return {"count": ordered.size(), "min": ordered[0], "mean": total / float(ordered.size()), "median": median, "p99": ordered[ceili(float(ordered.size()) * 0.99) - 1], "max": ordered[-1]}

func _write_report() -> void:
	var file: FileAccess = FileAccess.open(_out.path_join("mux-gameplay-perf.json"), FileAccess.WRITE)
	if file == null:
		_errors.append("cannot write report")
		_report["complete"] = false
		return
	file.store_string(JSON.stringify(_report, "\t"))
	file.close()

func _closed() -> void:
	_errors.append("native window closed before completion")
	_report["complete"] = false
	_write_report()
	quit(1)

func _timed_out() -> void:
	_errors.append("720 second wall-clock limit reached")
	_report["complete"] = false
	_write_report()
	quit(1)
