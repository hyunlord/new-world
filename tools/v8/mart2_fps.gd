# mart2_fps.gd — M-ART2 frame-rate runs (goal §4, MART2_DECISIONS D4). Windowed only (headless draws nothing).
# Purpose: one measured run of the real game scene: `--load <abs .wsave>` (the seed 0 colony at day 20) or
#   `--stress N` (reset_world_stress(0, N), the 1,000-person world), camera at `--zoom Z` on the hearth (the stress
#   world: its centre), vsync off and no frame cap, the simulation running at 1×; `--warmup S` seconds are discarded,
#   then frames are counted for `--seconds S` (RenderingServer.frame_post_draw). Prints one MART2_FPS JSON line: fps, frames,
#   seconds, people, zoom, LOD state, the largest chunk bake seen (µs), texture and video memory (Performance
#   monitors, bytes), the load average is recorded by the caller. Exits 0.
#   `Godot --path . --rendering-driver opengl3 --resolution 1920x1080 --script res://tools/v8/mart2_fps.gd -- --load F --zoom 1`
# Bridge methods called: load_world, reset_world_stress (direct); via SimReads get_agent_snapshot,
#   get_building_snapshot, get_world_info.
extends SceneTree

var _drawn_frames: int = 0

func _on_frame_post_draw() -> void:
	_drawn_frames += 1

func _window_state() -> Dictionary:
	return {"mode": root.mode, "always_on_top": root.always_on_top, "visible": root.visible, "focus": root.has_focus(), "size": [root.size.x, root.size.y]}


const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")


func _init() -> void:
	RenderingServer.frame_post_draw.connect(_on_frame_post_draw)
	call_deferred("_run")


func _arg(name: String, fallback: String) -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = args.find(name)
	return args[i + 1] if i >= 0 and i + 1 < args.size() else fallback


func _run() -> void:
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	Engine.max_fps = 0
	var receipt_path: String = _arg("--bake-events", OS.get_user_data_dir().path_join("mart2-fps-bakes-%d.jsonl" % OS.get_process_id())).simplify_path()
	var project: String = ProjectSettings.globalize_path("res://").simplify_path().trim_suffix("/")
	if not receipt_path.is_absolute_path() or receipt_path == project or receipt_path.begins_with(project + "/") or FileAccess.file_exists(receipt_path):
		push_error("MART2_FPS bake receipt needs a new external absolute path")
		quit(2)
		return
	var receipt: FileAccess = FileAccess.open(receipt_path, FileAccess.WRITE)
	if receipt == null:
		push_error("MART2_FPS cannot create bake receipt")
		quit(2)
		return
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var ground: Node = main.get_node("ColonyView/GroundLayer")
	ground.set("capture_bake_events", true)
	ground.set("bake_capture_context", "bootstrap")
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	var camera: Camera2D = main.get_node("Camera2D")
	for _i: int in 5:
		await process_frame
	var save: String = _arg("--load", "")
	var stress: int = int(_arg("--stress", "0"))
	ground.set("bake_capture_context", "world_setup")
	if save != "":
		world.call("load_world", save)
	elif stress > 0:
		world.call("reset_world_stress", 0, stress)
	for _i: int in 30:
		await process_frame
	ground.set("bake_capture_context", "camera_setup")
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
	var warmup: float = float(_arg("--warmup", "30"))
	var seconds: float = float(_arg("--seconds", "60"))
	ground.set("bake_capture_context", "warmup")
	var t0: int = Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < int(warmup * 1000.0):
		await process_frame
	ground.set("bake_capture_context", "measurement")
	var worst_bake: int = 0
	var window_start: Dictionary = _window_state()
	var f0: int = _drawn_frames
	var engine_f0: int = Engine.get_frames_drawn()
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
	var frames: int = _drawn_frames - f0
	var engine_frames: int = Engine.get_frames_drawn() - engine_f0
	var window_end: Dictionary = _window_state()
	var s1: int = Time.get_ticks_usec()
	var elapsed: float = float(s1 - s0) / 1e6
	ground.set("capture_bake_events", false)
	var people: int = (SimReads.read(world, "get_agent_snapshot").get("ids", PackedInt64Array()) as PackedInt64Array).size()
	var agents: Node = main.get_node("ColonyView/AgentLayer")
	var result: Dictionary = {
		"frame_counter": "RenderingServer.frame_post_draw", "engine_frames": engine_frames,
		"window_start": window_start, "window_end": window_end,
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
	var events: Array[Dictionary] = ground.get("bake_events")
	for event: Dictionary in events:
		receipt.store_line(JSON.stringify(event))
	receipt.flush()
	var receipt_ok: bool = receipt.get_error() == OK
	receipt.close()
	result["bake_receipt"] = {"path": receipt_path, "sha256": FileAccess.get_sha256(receipt_path), "write_ok": receipt_ok,
		"measurement_start_usec": s0, "measurement_end_usec": s1, "summary": summarize_bakes(events, s0, s1),
		"legacy_warning": "worst_chunk_bake_usec samples a persistent last value; worst_bake_breakdown is the lifetime maximum across reset epochs"}
	print("MART2_FPS " + JSON.stringify(result))
	if not receipt_ok:
		quit(2)
		return
	quit(0)


## Classify actual event intervals, never sampled/stale last values. Boundary-overlapping work remains measured.
static func summarize_bakes(events: Array[Dictionary], start_usec: int, end_usec: int) -> Dictionary:
	var groups: Dictionary = {}
	var valid: bool = end_usec > start_usec
	var previous_start: int = -1
	for i: int in events.size():
		var event: Dictionary = events[i]
		var begin: int = int(event["start_usec"])
		var finish: int = int(event["end_usec"])
		valid = valid and int(event["sequence"]) == i and begin >= previous_start and finish >= begin
		valid = valid and int(event["total_usec"]) == finish - begin
		valid = valid and int(event["compose_usec"]) + int(event["upload_usec"]) + int(event["strips_usec"]) == int(event["total_usec"])
		previous_start = begin
		var window: String = "before_measurement" if finish <= start_usec else ("after_measurement" if begin >= end_usec else "measurement")
		var kind: String = str(event["kind"])
		for key: String in ["all", window, window + "/" + kind, "context/" + str(event["context"]) + "/" + kind]:
			if not groups.has(key):
				groups[key] = {"count": 0, "max_total_usec": 0, "max_compose_upload_usec": 0, "over_4000_total": 0, "worst_sequence": -1}
			var group: Dictionary = groups[key]
			group["count"] = int(group["count"]) + 1
			if int(event["total_usec"]) > int(group["max_total_usec"]):
				group["max_total_usec"] = int(event["total_usec"])
				group["worst_sequence"] = i
			group["max_compose_upload_usec"] = maxi(int(group["max_compose_upload_usec"]), int(event["compose_upload_usec"]))
			group["over_4000_total"] = int(group["over_4000_total"]) + (1 if int(event["total_usec"]) > 4000 else 0)
	var rebakes: Dictionary = groups.get("measurement/rebake", {})
	var status: String = "NOT_MEASURED" if rebakes.is_empty() else ("FAIL" if int(rebakes["max_total_usec"]) > 4000 else "PASS")
	return {"valid_intervals": valid, "event_count": events.size(), "groups": groups,
		"measured_rebake_budget_usec": 4000, "measured_rebake_status": status if valid else "INVALID",
		"budget_scope": "Full per-chunk total including compose, upload-side metadata and wall strips; fresh events remain separately reported, not erased"}
