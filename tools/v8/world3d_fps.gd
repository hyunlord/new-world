# world3d_fps.gd — M3D-1 frame-rate runs of the real game scene (windowed only; headless draws nothing).
# Purpose: `--stress N` (reset_world_stress(0, N); the bridge caps N at 5,000) or `--load <wsave>`, camera at `--zoom Z`
#   on the map centre (stress) or the save's camp, vsync off, no frame cap, the simulation running at 1×; `--warmup S`
#   seconds discarded, then frames counted for `--seconds S` (RenderingServer.frame_post_draw). `--replicate K` asks the
#   3D people layer to draw every person K times (a render-only crowd beyond the bridge cap; recorded as such).
#   `--paused 1` stops the clock (a renderer-only measurement); `--hide Layer,…` or `Env:ssao+glow+fog+shadows` switches
#   parts off for profiling. Works with `--renderer=2d` too (the 2D reference). Prints one `W3D_FPS <json>` line; the caller records the load.
# Bridge methods called: load_world, reset_world_stress (direct); via SimReads get_agent_snapshot, get_world_info,
#   get_building_snapshot.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")

var _drawn_frames: int = 0


func _init() -> void:
	RenderingServer.frame_post_draw.connect(func() -> void: _drawn_frames += 1)
	call_deferred("_run")


func _arg(name: String, fallback: String) -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = args.find(name)
	return args[i + 1] if i >= 0 and i + 1 < args.size() else fallback


func _run() -> void:
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	Engine.max_fps = 0
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	var camera: Camera2D = main.get_node("Camera2D")
	for _i: int in 10:
		await process_frame
	var save: String = _arg("--load", "")
	var stress: int = int(_arg("--stress", "0"))
	if save != "":
		world.call("load_world", save)
	elif stress > 0:
		world.call("reset_world_stress", 0, stress)
	var w3d: Node = main.get_node_or_null("World3D")
	var replicate: int = int(_arg("--replicate", "1"))
	if w3d != null and replicate > 1:
		w3d.get_node("People").set("replicate", replicate)
	for _i: int in 30:
		await process_frame
	if _arg("--paused", "") == "1":
		world.process_mode = Node.PROCESS_MODE_DISABLED
	# Profiling: hide 3D layers by name (`--hide Flora,Props`), or `Env:ssao,glow,fog,shadows`.
	for name: String in _arg("--hide", "").split(",", false):
		if w3d == null:
			break
		if name.begins_with("Env:"):
			var env: Environment = w3d.get_node("Sky").get("environment")
			for f: String in name.trim_prefix("Env:").split("+"):
				if f == "shadows":
					(w3d.get_node("Sky").get("sun") as DirectionalLight3D).shadow_enabled = false
				else:
					env.set(f + "_enabled", false)
			continue
		var n: Node = w3d.get_node_or_null(name)
		if n is Node3D:
			(n as Node3D).visible = false
			n.process_mode = Node.PROCESS_MODE_DISABLED
	for name2: String in _arg("--noprocess", "").split(",", false):
		if w3d != null and w3d.get_node_or_null(name2) != null:
			w3d.get_node(name2).process_mode = Node.PROCESS_MODE_DISABLED
	var zoom: float = float(_arg("--zoom", "1"))
	camera.zoom = Vector2(zoom, zoom)
	var info: Dictionary = SimReads.read(world, "get_world_info")
	var centre: Vector2i = Vector2i(int(info.get("width", 64)) >> 1, int(info.get("height", 64)) >> 1)
	if stress <= 0:
		var b: Dictionary = SimReads.read(world, "get_building_snapshot")
		var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
		for i: int in kinds.size():
			if bool(ArtData.binding("buildings", kinds[i]).get("light", false)):
				centre = Vector2i(b["xs"][i], b["ys"][i])
				break
	camera.position = View.tile_centre(centre)
	camera.call("_clamp_position")
	var warmup: float = float(_arg("--warmup", "15"))
	var seconds: float = float(_arg("--seconds", "30"))
	var t0: int = Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < int(warmup * 1000.0):
		await process_frame
	var vp: RID = root.get_viewport_rid()
	RenderingServer.viewport_set_measure_render_time(vp, true)
	var gpu_ms: float = 0.0
	var cpu_ms: float = 0.0
	var f0: int = _drawn_frames
	var s0: int = Time.get_ticks_usec()
	var process_ms: float = 0.0
	var draw_calls: float = 0.0
	var prims: float = 0.0
	var samples: int = 0
	while Time.get_ticks_usec() - s0 < int(seconds * 1e6):
		await process_frame
		process_ms += Performance.get_monitor(Performance.TIME_PROCESS) * 1000.0
		draw_calls += Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME)
		prims += Performance.get_monitor(Performance.RENDER_TOTAL_PRIMITIVES_IN_FRAME)
		gpu_ms += RenderingServer.viewport_get_measured_render_time_gpu(vp)
		cpu_ms += RenderingServer.viewport_get_measured_render_time_cpu(vp) + RenderingServer.get_frame_setup_time_cpu()
		samples += 1
	var elapsed: float = float(Time.get_ticks_usec() - s0) / 1e6
	var frames: int = _drawn_frames - f0
	var people: int = (SimReads.read(world, "get_agent_snapshot").get("ids", PackedInt64Array()) as PackedInt64Array).size()
	var result: Dictionary = {"renderer": "3d" if w3d != null else "2d", "scene": "stress %d" % stress if stress > 0 else save.get_file(),
		"zoom": zoom, "fps": float(frames) / elapsed, "frames": frames, "seconds": elapsed, "people": people,
		"drawn_people": people * replicate if w3d != null else people, "replicate": replicate,
		"process_ms_mean": process_ms / maxi(1, samples), "draw_calls_mean": draw_calls / maxi(1, samples),
		"primitives_mean": prims / maxi(1, samples),
		"render_gpu_ms_mean": gpu_ms / maxi(1, samples), "render_cpu_ms_mean": cpu_ms / maxi(1, samples), "video_mem_bytes": Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED),
		"focus": root.has_focus(), "size": [root.size.x, root.size.y], "paused": _arg("--paused", "") == "1", "hide": _arg("--hide", "")}
	print("W3D_FPS " + JSON.stringify(result))
	quit(0)
