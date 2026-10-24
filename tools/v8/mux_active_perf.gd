# M-UX active-world timing: real main scene, one ordinary engine tick per rendered frame.
# Bridge methods via SimReads: reset_world, cmd_set_steward, advance_ticks, get_time_snapshot,
# get_world_info, get_steward, save_world. No resource grants, person movement, or simulated timing results.
# Windowed debug run after bridge build: godot --path . --script tools/v8/mux_active_perf.gd -- /external/dir [seed]
extends SceneTree

const Hud = preload("res://scripts/ui/mux/mux_hud.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
const WARMUP_FRAMES: int = 120
const SAMPLE_TICKS: int = 1441
const MAX_SECONDS: float = 900.0
const SOURCE_PATHS: Array[String] = [
	"res://tools/v8/mux_active_perf.gd", "res://sim_bridge.gdextension", "res://rust/Cargo.toml",
	"res://rust/crates/sim-systems/src/runtime/steward/planning.rs",
	"res://rust/crates/sim-systems/src/runtime/steward/production.rs",
	"res://rust/crates/sim-systems/src/runtime/steward/lifecycle.rs",
	"res://rust/crates/sim-systems/src/runtime/steward/mod.rs",
	"res://rust/crates/sim-systems/src/commands.rs", "res://rust/crates/sim-bridge/src/ffi/node.rs",
	"res://rust/crates/sim-core/data/colony/steward.ron",
]
var _out: String
var _seed: int = 0
var _main: Node
var _world: Node
var _camera: Camera2D
var _report: Dictionary = {}
var _errors: Array[String] = []

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	_out = args[0] if not args.is_empty() else "/tmp/worldsim-mux-active-perf-%d" % OS.get_process_id()
	if not _out.is_absolute_path() or _out.simplify_path().begins_with(ProjectSettings.globalize_path("res://").simplify_path()):
		push_error("MUX_ACTIVE_PERF needs an absolute evidence directory outside the project")
		quit(1)
		return
	if DirAccess.make_dir_recursive_absolute(_out) != OK:
		push_error("MUX_ACTIVE_PERF cannot create evidence directory")
		quit(1)
		return
	if args.size() > 1:
		if not args[1].is_valid_int():
			push_error("MUX_ACTIVE_PERF seed must be an integer")
			quit(1)
			return
		_seed = int(args[1])
	if DisplayServer.get_name() == "headless" or not OS.has_feature("debug"):
		push_error("MUX_ACTIVE_PERF requires a windowed debug Godot executable")
		quit(1)
		return
	_report = {
		"started_utc": Time.get_datetime_string_from_system(true), "seed": _seed,
		"scope": "Synthetic active simulation: one real advance_ticks(1) per rendered frame, automatic accumulator paused. Not normal-speed gameplay FPS acceptance.",
		"limitations": "Sequential ON then OFF arms; shared renderer caches and uncontrolled background load. Advance CPU includes bridge dispatch and SimReads invalidation. Frame intervals include UI, rendering, pacing and harness overhead. No per-system exclusive timing or loaded-memory binary hash claim.",
		"warmup_rendered_frames": WARMUP_FRAMES, "sample_ticks_per_phase": SAMPLE_TICKS,
		"environment": _environment(), "source_files_on_disk": _source_hashes(),
		"bridge": _bridge_metadata(), "phases": [], "errors": _errors, "complete": false,
	}
	if not _errors.is_empty():
		_write_report()
		push_error("MUX_ACTIVE_PERF provenance validation failed: %s" % _errors)
		quit(1)
		return
	auto_accept_quit = false
	root.close_requested.connect(_closed)
	create_timer(MAX_SECONDS).timeout.connect(_timed_out)
	for enabled: bool in [true, false]:
		await _phase(enabled)
		if not _errors.is_empty(): break
	_report["complete"] = _errors.is_empty() and (_report["phases"] as Array).size() == 2
	_report["finished_utc"] = Time.get_datetime_string_from_system(true)
	_write_report()
	print("MUX_ACTIVE_PERF_DONE complete=%s errors=%s evidence=%s" % [_report["complete"], _errors, _out.path_join("mux-active-perf.json")])
	quit(0 if bool(_report["complete"]) else 1)

func _phase(enabled: bool) -> void:
	if is_instance_valid(_main):
		_main.queue_free()
		await process_frame
		await process_frame
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var hud: Hud = _main.get_node("UI/MuxHud") as Hud
	var label: String = "on" if enabled else "off"
	hud.store = Store.new(_out.path_join("active-settings-%d-%s.cfg" % [OS.get_process_id(), label]))
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D") as Camera2D
	root.add_child(_main)
	_camera.call("set_paused", true)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	if not bool(SimReads.mutate(_world, "reset_world", [_seed])):
		_errors.append("reset_world failed")
		return
	var command: Dictionary = SimReads.mutate(_world, "cmd_set_steward", [enabled, 0])
	if not bool(command.get("accepted", false)):
		_errors.append("ordinary SetSteward command rejected")
		return
	_camera.set_process(false)
	_camera.set_process_input(false)
	_camera.set_process_unhandled_input(false)
	_camera.zoom = Vector2.ONE
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	_camera.call("center_on_tile", Vector2i(floori(float(info.get("width", 0)) / 2.0), floori(float(info.get("height", 0)) / 2.0)))
	for _frame: int in WARMUP_FRAMES: await RenderingServer.frame_post_draw
	var previous_draw: int = Time.get_ticks_usec()
	var stable_view: Dictionary = _view()
	var start: Dictionary = SimReads.read(_world, "get_time_snapshot")
	var steward: Dictionary = SimReads.read(_world, "get_steward")
	if int(start.get("tick", -1)) != 0 or bool(steward.get("enabled", not enabled)) != enabled:
		_errors.append("renderer warmup changed tick zero or requested steward mode")
		return
	var phases: Array = _report["phases"]
	if not phases.is_empty() and phases[0]["view"] != stable_view:
		_errors.append("camera or viewport differs across phases")
		return
	var samples: Array[Dictionary] = []
	var calls: Array[float] = []
	var intervals: Array[float] = []
	var boundary_frames: Array[Dictionary] = []
	var phase: Dictionary = {"steward_enabled": enabled, "world": info, "view": stable_view, "start_tick": 0, "raw_samples": samples, "boundary_windows": boundary_frames, "complete": false}
	phases.append(phase)
	for index: int in SAMPLE_TICKS:
		await process_frame
		var before_tick: int = int(SimReads.read(_world, "get_time_snapshot").get("tick", -1))
		if before_tick != index or _view() != stable_view or _world.process_mode != Node.PROCESS_MODE_DISABLED:
			_errors.append("phase %s: tick/view/automatic process changed before sample %d" % [label, index])
			break
		var started: int = Time.get_ticks_usec()
		var returned_tick: int = int(SimReads.mutate(_world, "advance_ticks", [1]))
		var elapsed: int = Time.get_ticks_usec() - started
		var after_tick: int = int(SimReads.read(_world, "get_time_snapshot").get("tick", -1))
		await RenderingServer.frame_post_draw
		var drawn: int = Time.get_ticks_usec()
		var interval: float = float(drawn - previous_draw) / 1000.0
		steward = SimReads.read(_world, "get_steward")
		var sample: Dictionary = {"executed_tick": before_tick, "next_tick": after_tick, "returned_tick": returned_tick, "advance_call_usec": elapsed, "post_draw_usec": drawn, "post_draw_interval_ms": interval, "process_frame": Engine.get_process_frames(), "steward_sequence": int(steward.get("sequence", -1)), "steward_last_tick": int(steward.get("last_tick", -1)), "steward_interval_due": before_tick % 360 == 0}
		samples.append(sample)
		calls.append(float(elapsed))
		intervals.append(interval)
		if index % 360 <= 5 or index % 360 >= 355: boundary_frames.append(sample)
		previous_draw = drawn
		if returned_tick != index + 1 or after_tick != index + 1 or _view() != stable_view or bool(steward.get("enabled", not enabled)) != enabled:
			_errors.append("phase %s: tick/view/mode mismatch after sample %d" % [label, index])
			break
	phase["complete"] = samples.size() == SAMPLE_TICKS and _errors.is_empty()
	phase["advance_call_summary_usec"] = _summary(calls)
	phase["post_draw_interval_summary_ms"] = _summary(intervals)
	phase["end_tick"] = int(SimReads.read(_world, "get_time_snapshot").get("tick", -1))
	phase["final_steward"] = steward
	if bool(phase["complete"]):
		var save_path: String = _out.path_join("active-%s-tick-%d.save" % [label, int(phase["end_tick"])])
		var save_record: Dictionary = {"path": save_path, "tick": phase["end_tick"], "success": false, "scope": "Full debug world save after all measured samples; save cost excluded from timing summaries. No additional simulation ticks."}
		phase["world_save"] = save_record
		if FileAccess.file_exists(save_path):
			_errors.append("refusing to overwrite world save: " + save_path)
		elif not bool(SimReads.mutate(_world, "save_world", [save_path])):
			_errors.append("post-measurement save_world failed: " + save_path)
		else:
			save_record["sha256"] = _required_hash(save_path)
			var saved_tick: int = int(SimReads.read(_world, "get_time_snapshot").get("tick", -1))
			save_record["tick_after_save"] = saved_tick
			if saved_tick != int(phase["end_tick"]):
				_errors.append("post-measurement save changed simulation tick")
			save_record["success"] = _errors.is_empty()
		phase["complete"] = _errors.is_empty()
	_write_report()
	print("MUX_ACTIVE_PERF phase=%s complete=%s ticks=%d advance_us=%s post_draw_ms=%s" % [label, phase["complete"], samples.size(), phase["advance_call_summary_usec"], phase["post_draw_interval_summary_ms"]])

func _view() -> Dictionary:
	return {"camera_position": [_camera.position.x, _camera.position.y], "camera_zoom": [_camera.zoom.x, _camera.zoom.y], "window_size": [root.size.x, root.size.y], "viewport": str(root.get_visible_rect()), "window_mode": DisplayServer.window_get_mode()}

func _environment() -> Dictionary:
	return {"engine": Engine.get_version_info(), "os": OS.get_name(), "os_version": OS.get_version(), "cpu": OS.get_processor_name(), "cpu_count": OS.get_processor_count(), "gpu": RenderingServer.get_video_adapter_name(), "gpu_vendor": RenderingServer.get_video_adapter_vendor(), "render_method": RenderingServer.get_current_rendering_method(), "render_driver": RenderingServer.get_current_rendering_driver_name(), "debug_feature": OS.has_feature("debug"), "editor_feature": OS.has_feature("editor"), "vsync": DisplayServer.window_get_vsync_mode(), "max_fps": Engine.max_fps}

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
	var file: FileAccess = FileAccess.open(_out.path_join("mux-active-perf.json"), FileAccess.WRITE)
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
	_errors.append("900 second wall-clock limit reached")
	_report["complete"] = false
	_write_report()
	quit(1)
