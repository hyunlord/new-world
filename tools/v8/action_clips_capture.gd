# action_clips_capture.gd — INTEG1: which mocap clip each near Style 2 villager plays for its ACTION1 row, frame by frame.
# Purpose: `--save <wsave> --out <dir> [--zoom 4] [--shots 48] [--every 10] [--segment 12] [--settle 30]` loads the
#   save in the real game scene, moves the clock into working hours (09:00) when it is outside 07–18 (then `--advance T`
#   more ticks), lets it run at 1×, and every `--segment` shots frames the tile whose surroundings (Chebyshev 4) hold
#   the most performing (phase 2) visual keys not yet seen near the camera (next candidate while none is near). Every `--every` frames it saves `<out>/frame_NNN.png` and records
#   each near-tier villager ({agent, bits, visual_key, fallback_key, clip, on_screen}); `<out>/action_clips.json` holds
#   all frames and the distinct on-screen (visual_key → clips). Prints `ACTION_CLIPS <json>` per frame and
#   `ACTION_CLIPS_DONE distinct=<n> keys=<…>`; quits 1 when no near villager was ever seen. Windowed only (Forward+):
#   `Godot --path . --resolution 1920x1080 --script res://tools/v8/action_clips_capture.gd -- --save … --out …`
# Bridge methods called: load_world, advance_ticks (direct); via SimReads get_time_snapshot, get_agent_snapshot,
#   get_visible_action_snapshot; World3D/People state_of.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")

var _main: Node = null
var _world: Node = null
var _camera: Camera2D = null


func _init() -> void:
	call_deferred("_run")


func _arg(name: String, fallback: String = "") -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = args.find(name)
	return args[i + 1] if i >= 0 and i + 1 < args.size() else fallback


func _frames(n: int) -> void:
	for _i: int in n:
		await process_frame


func _fail(message: String) -> void:
	push_error("action_clips_capture: " + message)
	quit(1)


func _run() -> void:
	var out: String = _arg("--out")
	var save: String = _arg("--save")
	if out == "" or save == "" or DisplayServer.get_name() == "headless":
		_fail("--out and --save are required, in a window")
		return
	DirAccess.make_dir_recursive_absolute(out)
	var zoom: float = float(_arg("--zoom", "4"))
	var shots: int = int(_arg("--shots", "48"))
	var every: int = maxi(1, int(_arg("--every", "10")))
	var segment: int = maxi(1, int(_arg("--segment", "12")))
	var settle: int = int(_arg("--settle", "30"))
	var advance: int = int(_arg("--advance", "0"))
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D")
	await _frames(10)
	if not bool(_world.call("load_world", save)):
		_fail("load_world failed: " + save)
		return
	await _frames(10)
	var time: Dictionary = SimReads.read(_world, "get_time_snapshot")
	var hour: float = float(time.get("time_of_day", 12.0))
	var per_day: int = int(time.get("ticks_per_day", 1440))
	if per_day > 0 and (hour < 7.0 or hour >= 18.0):
		_world.call("advance_ticks", int(fposmod(9.0 - hour, 24.0) * float(per_day) / 24.0))
		await _frames(10)
	if advance > 0:
		_world.call("advance_ticks", advance)
		await _frames(10)
	var w3d: Node = _main.get_node_or_null("World3D")
	var people: Node = w3d.get_node_or_null("People") if w3d != null else null
	if people == null:
		_fail("no World3D/People")
		return
	(_main.get_node("UI") as CanvasLayer).visible = false
	var seen: Dictionary = {}
	var frames: Array = []
	for shot: int in shots:
		if shot % segment == 0:
			# Near mocap is gated per crowd bin (its chunk centre within the near range), so a framed performer can
			# still be drawn at mid: try the next candidate until one shows an on-screen near villager.
			for tile: Vector2i in _best_tiles(seen).slice(0, 6):
				_focus(tile, zoom)
				w3d.get_node("Terrain").call("build_all")
				await _frames(settle)
				var probe: Array = _near_rows(people, {})
				if probe.any(func(a: Dictionary) -> bool: return bool(a["on_screen"])):
					break
		else:
			await _frames(every)
		await RenderingServer.frame_post_draw
		var path: String = out.path_join("frame_%03d.png" % shot)
		root.get_viewport().get_texture().get_image().save_png(path)
		var row: Dictionary = {"frame": shot, "file": path, "tick": int(SimReads.read(_world, "get_time_snapshot").get("tick", 0)),
			"agents": _near_rows(people, seen)}
		frames.append(row)
		print("ACTION_CLIPS %s" % JSON.stringify(row))
	var distinct: Dictionary = {}
	var any_near: bool = false
	for row: Dictionary in frames:
		for a: Dictionary in row["agents"]:
			any_near = true
			if bool(a["on_screen"]):
				if not distinct.has(a["visual_key"]):
					distinct[a["visual_key"]] = []
				if not (distinct[a["visual_key"]] as Array).has(a["clip"]):
					(distinct[a["visual_key"]] as Array).append(a["clip"])
	var file: FileAccess = FileAccess.open(out.path_join("action_clips.json"), FileAccess.WRITE)
	file.store_string(JSON.stringify({"save": save, "zoom": zoom, "distinct": distinct, "frames": frames}, " "))
	file.close()
	print("ACTION_CLIPS_DONE distinct=%d keys=%s" % [distinct.size(), JSON.stringify(distinct)])
	if not any_near:
		_fail("no near-tier villager was seen")
		return
	quit(0)


func _focus(tile: Vector2i, zoom: float) -> void:
	_camera.zoom = Vector2(zoom, zoom)
	_camera.position = Vector2(tile) * View.TILE + Vector2(View.TILE, View.TILE) * 0.5
	_camera.call("_clamp_position")


## Performer tiles, best first: most performing visual keys not yet seen within Chebyshev 4, then most performers.
func _best_tiles(seen: Dictionary) -> Array[Vector2i]:
	var agents: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var acts: Dictionary = SimReads.read(_world, "get_visible_action_snapshot")
	var at: Dictionary = {}
	var aids: PackedInt64Array = agents.get("agent_ids", PackedInt64Array())
	var xs: PackedInt32Array = agents.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = agents.get("ys", PackedInt32Array())
	for i: int in mini(aids.size(), mini(xs.size(), ys.size())):
		at[aids[i]] = Vector2i(xs[i], ys[i])
	var performers: Array = []
	var act_ids: PackedInt64Array = acts.get("agent_ids", PackedInt64Array())
	var phases: PackedByteArray = acts.get("phases", PackedByteArray())
	var keys: PackedStringArray = acts.get("visual_keys", PackedStringArray())
	for i: int in mini(act_ids.size(), mini(phases.size(), keys.size())):
		if phases[i] == 2 and at.has(act_ids[i]):
			performers.append([at[act_ids[i]], keys[i]])
	var scored: Array = []
	for p: Array in performers:
		var centre: Vector2i = p[0]
		var fresh: Dictionary = {}
		var count: int = 0
		for q: Array in performers:
			var d: Vector2i = (q[0] as Vector2i) - centre
			if maxi(absi(d.x), absi(d.y)) <= 4:
				count += 1
				if not seen.has(q[1]):
					fresh[q[1]] = true
		scored.append([fresh.size() * 1000 + count, centre])
	scored.sort_custom(func(a: Array, b: Array) -> bool: return int(a[0]) > int(b[0]))
	var out: Array[Vector2i] = []
	for row: Array in scored:
		if not out.has(row[1]):
			out.append(row[1])
	if out.is_empty() and not at.is_empty():
		out.append(at.values()[0])
	return out


## Every villager whose near mocap tier is written: AgentId, entity bits, ACTION1 keys and the near clip drawn.
func _near_rows(people: Node, seen: Dictionary) -> Array:
	var agents: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var acts: Dictionary = SimReads.read(_world, "get_visible_action_snapshot")
	var row_of: Dictionary = {}
	var act_ids: PackedInt64Array = acts.get("agent_ids", PackedInt64Array())
	for i: int in act_ids.size():
		row_of[act_ids[i]] = i
	var keys: PackedStringArray = acts.get("visual_keys", PackedStringArray())
	var falls: PackedStringArray = acts.get("fallback_keys", PackedStringArray())
	var ids: PackedInt64Array = agents.get("ids", PackedInt64Array())
	var aids: PackedInt64Array = agents.get("agent_ids", PackedInt64Array())
	var cam: Camera3D = root.get_viewport().get_camera_3d()
	var out: Array = []
	for i: int in mini(ids.size(), aids.size()):
		var st: Dictionary = people.call("state_of", ids[i])
		if st.is_empty() or not bool(st.get("near_live", false)) or str(st.get("near_clip", "")).is_empty():
			continue
		var r: int = int(row_of.get(aids[i], -1))
		var feet: Vector3 = st["feet"]
		var on_screen: bool = cam != null and cam.is_position_in_frustum(feet + Vector3(0, 0.8, 0))
		var key: String = keys[r] if r >= 0 and r < keys.size() else ""
		if on_screen:
			seen[key] = true
		out.append({"agent": aids[i], "bits": ids[i], "visual_key": key,
			"fallback_key": falls[r] if r >= 0 and r < falls.size() else "", "clip": st["near_clip"], "on_screen": on_screen})
	return out
