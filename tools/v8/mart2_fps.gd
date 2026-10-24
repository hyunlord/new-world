# mart2_fps.gd — M-ART2 frame-rate runs (goal §4, MART2_DECISIONS D4). Windowed only (headless draws nothing).
# Purpose: one measured run of the real game scene: `--load <abs .wsave>` (the seed 0 colony at day 20) or
#   `--stress N` (reset_world_stress(0, N), the 1,000-person world), camera at `--zoom Z` on the hearth (the stress
#   world: its centre), vsync off and no frame cap, the simulation running at 1×; `--warmup S` seconds are discarded,
#   then frames are counted for `--seconds S` (Engine.get_frames_drawn). Prints one MART2_FPS JSON line: fps, frames,
#   seconds, people, zoom, LOD state, the largest chunk bake seen (µs), texture and video memory (Performance
#   monitors, bytes), the load average is recorded by the caller. Exits 0.
#   `Godot --path . --rendering-driver opengl3 --resolution 1920x1080 --script res://tools/v8/mart2_fps.gd -- --load F --zoom 1`
# Bridge methods called: load_world, reset_world_stress (direct); via SimReads get_agent_snapshot,
#   get_building_snapshot, get_world_info.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")


func _init() -> void:
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
	for _i: int in 5:
		await process_frame
	var save: String = _arg("--load", "")
	var stress: int = int(_arg("--stress", "0"))
	if save != "":
		world.call("load_world", save)
	elif stress > 0:
		world.call("reset_world_stress", 0, stress)
	for _i: int in 30:
		await process_frame
	var zoom: float = float(_arg("--zoom", "1"))
	camera.zoom = Vector2(zoom, zoom)
	var centre: Vector2i = Vector2i(-1, -1)
	var b: Dictionary = SimReads.read(world, "get_building_snapshot")
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	for i: int in kinds.size():
		if bool(ArtData.binding("buildings", kinds[i]).get("light", false)):
			centre = Vector2i(b["xs"][i], b["ys"][i])
			break
	if stress > 0 or centre.x < 0:
		var info: Dictionary = SimReads.read(world, "get_world_info")
		centre = Vector2i(int(info.get("width", 64)) >> 1, int(info.get("height", 64)) >> 1)
	camera.position = View.tile_centre(centre)
	camera.call("_clamp_position")
	var ground: Node = main.get_node("ColonyView/GroundLayer")
	var warmup: float = float(_arg("--warmup", "30"))
	var seconds: float = float(_arg("--seconds", "60"))
	var t0: int = Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < int(warmup * 1000.0):
		await process_frame
	var worst_bake: int = 0
	var f0: int = Engine.get_frames_drawn()
	var s0: int = Time.get_ticks_usec()
	var process_ms: float = 0.0
	var draw_calls: float = 0.0
	var samples: int = 0
	while Time.get_ticks_usec() - s0 < int(seconds * 1e6):
		await process_frame
		worst_bake = maxi(worst_bake, int(ground.get("last_bake_usec")))
		process_ms += Performance.get_monitor(Performance.TIME_PROCESS) * 1000.0
		draw_calls += Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME)
		samples += 1
	var frames: int = Engine.get_frames_drawn() - f0
	var elapsed: float = float(Time.get_ticks_usec() - s0) / 1e6
	var people: int = (SimReads.read(world, "get_agent_snapshot").get("ids", PackedInt64Array()) as PackedInt64Array).size()
	var agents: Node = main.get_node("ColonyView/AgentLayer")
	var result: Dictionary = {
		"scene": "stress %d" % stress if stress > 0 else save.get_file(),
		"zoom": zoom, "fps": float(frames) / elapsed, "frames": frames, "seconds": elapsed, "people": people,
		"lod": bool(agents.call("lod_active")), "worst_chunk_bake_usec": worst_bake,
		"texture_mem_bytes": Performance.get_monitor(Performance.RENDER_TEXTURE_MEM_USED),
		"video_mem_bytes": Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED),
		"vsync": DisplayServer.window_get_vsync_mode(), "max_fps": Engine.max_fps,
		"process_ms_mean": process_ms / maxi(1, samples), "draw_calls_mean": draw_calls / maxi(1, samples),
		"nodes": Performance.get_monitor(Performance.OBJECT_NODE_COUNT),
		"worst_bake_breakdown": str(ground.get("worst_bake")),
	}
	print("MART2_FPS " + JSON.stringify(result))
	quit(0)
