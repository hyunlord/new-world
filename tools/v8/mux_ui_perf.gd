# M-UX rendered-frame/process comparison in the real paused main scene.
# Bridge methods via SimReads: advance_ticks(1), get_time_snapshot, get_agent_snapshot, get_world_info.
# Run windowed after fresh bridge build: godot --path . --script tools/v8/mux_ui_perf.gd -- /absolute/evidence/dir
# Synthetic paused-world A/B measurement; background load is uncontrolled. No FPS acceptance assertion.
extends SceneTree

const Hud = preload("res://scripts/ui/mux/mux_hud.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
const WARMUP_FRAMES: int = 120
const SAMPLE_FRAMES: int = 600
var _main: Node
var _hud: Hud
var _world: Node
var _camera: Camera2D
var _processing_nodes: Array[Node] = []
var _out: String
var _start_utc: String

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	if DisplayServer.get_name() == "headless":
		push_error("MUX_UI_PERF requires a windowed renderer and frame_post_draw evidence")
		quit(1)
		return
	var args: PackedStringArray = OS.get_cmdline_user_args()
	_out = args[0] if not args.is_empty() else "/tmp/worldsim-mux-ui-perf-%d" % OS.get_process_id()
	if not _out.is_absolute_path() or _out.begins_with(ProjectSettings.globalize_path("res://")):
		push_error("MUX_UI_PERF evidence must be an absolute external path")
		quit(1)
		return
	if DirAccess.make_dir_recursive_absolute(_out) != OK:
		push_error("MUX_UI_PERF cannot create evidence directory")
		quit(1)
		return
	_start_utc = Time.get_datetime_string_from_system(true)
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	_hud = _main.get_node("UI/MuxHud") as Hud
	_hud.store = Store.new(_out.path_join("settings-%d.cfg" % OS.get_process_id()))
	_camera = _main.get_node("Camera2D") as Camera2D
	_world = _main.get_node("WorldSim")
	root.add_child(_main)
	_camera.call("set_paused", true)
	SimReads.mutate(_world, "advance_ticks", [1])
	for _frame: int in 20: await process_frame
	_camera.call("center_on_camp")
	_camera.set_process(false)
	_collect_processing(_hud)
	var node_paths: PackedStringArray = PackedStringArray()
	for node: Node in _processing_nodes: node_paths.append(str(_hud.get_path_to(node)))
	var environment: Dictionary = {
		"engine": Engine.get_version_info(), "os": OS.get_name(), "os_version": OS.get_version(),
		"processor": OS.get_processor_name(), "processor_count": OS.get_processor_count(),
		"gpu": RenderingServer.get_video_adapter_name(), "gpu_vendor": RenderingServer.get_video_adapter_vendor(),
		"display_server": DisplayServer.get_name(), "rendering_method": RenderingServer.get_current_rendering_method(),
		"rendering_driver": RenderingServer.get_current_rendering_driver_name(),
		"window_size": root.size, "viewport": root.get_visible_rect(), "window_mode": DisplayServer.window_get_mode(),
		"vsync": DisplayServer.window_get_vsync_mode(), "max_fps": Engine.max_fps,
		"camera_position": _camera.position, "camera_zoom": _camera.zoom,
		"world_info": SimReads.read(_world, "get_world_info"),
		"population": (SimReads.read(_world, "get_agent_snapshot").get("agent_ids", PackedInt64Array()) as PackedInt64Array).size(),
		"processing_nodes_toggled": node_paths,
	}
	var enabled: Dictionary = await _measure("mux_processing_enabled", true)
	var disabled: Dictionary = await _measure("mux_processing_disabled", false)
	for node: Node in _processing_nodes: node.set_process(true)
	var enabled_process: Dictionary = enabled["scene_process_summary_usec"]
	var disabled_process: Dictionary = disabled["scene_process_summary_usec"]
	var stable: bool = bool(enabled["stable_world_camera_viewport"]) and bool(disabled["stable_world_camera_viewport"]) and enabled["state_before"] == disabled["state_before"]
	var report: Dictionary = {
		"started_utc": _start_utc, "finished_utc": Time.get_datetime_string_from_system(true),
		"scope": "Synthetic paused-world comparison; real rendered frames, same visible MuxHud; only its process callbacks are disabled in B. Legacy UI remains enabled.",
		"limitations": "Sequential A/B under uncontrolled background load. Scene process-time difference is an estimate, not exclusive UI profiling. No full-game or FPS acceptance claim. Minimap max includes startup and warmup; other panels expose no dedicated CPU timers.",
		"warmup_rendered_frames_per_phase": WARMUP_FRAMES, "sample_rendered_frames_per_phase": SAMPLE_FRAMES,
		"environment": environment, "stable_world_camera_viewport": stable, "enabled": enabled, "disabled": disabled,
		"estimated_mux_process_difference_usec": {"mean": float(enabled_process["mean"]) - float(disabled_process["mean"]), "median": float(enabled_process["median"]) - float(disabled_process["median"])},
		"exclusive_total_ui_cpu": "NOT_MEASURED: no dedicated legacy/per-panel timers beyond minimap",
	}
	var file: FileAccess = FileAccess.open(_out.path_join("mux-ui-perf.json"), FileAccess.WRITE)
	if file == null:
		push_error("MUX_UI_PERF cannot write result")
		quit(1)
		return
	file.store_string(JSON.stringify(report, "\t"))
	file.close()
	print("MUX_UI_PERF enabled rendered_ms=%s process_us=%s" % [enabled["rendered_interval_summary_ms"], enabled_process])
	print("MUX_UI_PERF disabled rendered_ms=%s process_us=%s" % [disabled["rendered_interval_summary_ms"], disabled_process])
	print("MUX_UI_PERF minimap=%s" % enabled["minimap_after"])
	print("MUX_UI_PERF_DONE stable=%s samples_per_phase=%d evidence=%s" % [stable, SAMPLE_FRAMES, _out])
	quit(0 if stable else 1)

func _collect_processing(node: Node) -> void:
	if node.is_processing(): _processing_nodes.append(node)
	for child: Node in node.get_children(): _collect_processing(child)

func _state() -> Dictionary:
	return {"tick": int(SimReads.read(_world, "get_time_snapshot").get("tick", -1)), "camera_position": _camera.position, "camera_zoom": _camera.zoom, "viewport": root.get_visible_rect(), "window_size": root.size}

func _measure(label: String, processing: bool) -> Dictionary:
	for node: Node in _processing_nodes: node.set_process(processing)
	for _frame: int in WARMUP_FRAMES: await RenderingServer.frame_post_draw
	var previous: int = Time.get_ticks_usec()
	var before: Dictionary = _state()
	var minimap_before: Dictionary = _hud.minimap.performance_stats()
	var samples: Array[Dictionary] = []
	var intervals: Array[float] = []
	var process_usec: Array[float] = []
	var minimap_usec: Array[float] = []
	var previous_minimap: int = int(_hud.minimap.get("_total_usec"))
	for index: int in SAMPLE_FRAMES:
		await RenderingServer.frame_post_draw
		var now: int = Time.get_ticks_usec()
		var interval: float = float(now - previous) / 1000.0
		var process_cost: float = Performance.get_monitor(Performance.TIME_PROCESS) * 1000000.0
		var minimap_total: int = int(_hud.minimap.get("_total_usec"))
		var minimap_cost: float = float(minimap_total - previous_minimap)
		samples.append({"index": index, "rendered_usec": now, "rendered_interval_ms": interval, "scene_process_usec": process_cost, "minimap_process_usec": minimap_cost})
		intervals.append(interval)
		process_usec.append(process_cost)
		minimap_usec.append(minimap_cost)
		previous = now
		previous_minimap = minimap_total
	var after: Dictionary = _state()
	var minimap_after: Dictionary = _hud.minimap.performance_stats()
	var upper_bound: int = int(minimap_after["conservative_frame_usec"])
	return {"phase": label, "state_before": before, "state_after": after, "stable_world_camera_viewport": before == after,
		"raw_frames": samples, "rendered_interval_summary_ms": _summary(intervals), "scene_process_summary_usec": _summary(process_usec),
		"minimap_process_summary_usec": _summary(minimap_usec), "minimap_before": minimap_before, "minimap_after": minimap_after,
		"minimap_lifetime_conservative_budget_usec": 500, "minimap_lifetime_conservative_budget_met": upper_bound <= 500,
		"minimap_cost_note": "Process delta uses the widget's existing cumulative timer. Conservative max adds independent max process and max draw CPU times, including startup/warmup; GPU execution is not included."}

func _summary(values: Array[float]) -> Dictionary:
	var sorted: Array[float] = values.duplicate()
	sorted.sort()
	var total: float = 0.0
	for value: float in sorted: total += value
	var middle: int = floori(float(sorted.size()) / 2.0)
	var median: float = sorted[middle] if sorted.size() % 2 == 1 else (sorted[middle - 1] + sorted[middle]) / 2.0
	return {"count": sorted.size(), "min": sorted[0], "mean": total / sorted.size(), "median": median, "p95": sorted[ceili(sorted.size() * 0.95) - 1], "p99": sorted[ceili(sorted.size() * 0.99) - 1], "max": sorted[-1]}
