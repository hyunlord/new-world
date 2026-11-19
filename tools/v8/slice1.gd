# Slice and Style 2 captures of the normal colony scene.
# Bridge methods: load/reset world, read world/people/time, draft and order a diagnostic walk.
# SimReads calls: load_world, reset_world_stress and time, world and people snapshots.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const TripoAnimals = {"goat": "tripo_goat", "wild_goat": "tripo_goat"}
const Features: Array[String] = ["pbr", "vegetation", "people", "sdfgi", "ssao", "ssil", "volumetric", "taa", "shadows", "sun", "resolution"]

var _world: Node
var _w3d: Node
var _camera: Camera2D
var _style2: bool = false
var _rect: Rect2i
var _focus: Vector2
var _close: Vector2
var _ultra: bool = false
var _omit: String
var _draws: int = 0


func _initialize() -> void:
	RenderingServer.frame_post_draw.connect(func() -> void: _draws += 1)
	_run.call_deferred()


func _arg(key: String, default: String = "") -> String:
	for arg: String in OS.get_cmdline_user_args():
		if arg.begins_with(key + "="):
			return arg.trim_prefix(key + "=")
	return default


func _fail(reason: String) -> void:
	push_error("SLICE1: " + reason)
	quit(1)


func _frames(count: int) -> void:
	for _i: int in count:
		await process_frame


func _has(feature: String) -> bool:
	return _ultra and _omit != feature


func _run() -> void:
	_ultra = _arg("--quality") == "ultra"
	_style2 = OS.get_cmdline_user_args().has("--style=s2")
	_omit = _arg("--without")
	var action: String = _arg("--mode", "play")
	var stress: bool = _arg("--stress") == "500"
	if action not in ["play", "check", "capture", "fps", "walk"] or (_omit != "" and not Features.has(_omit)):
		_fail("unknown mode or ablation")
		return
	if action in ["capture", "fps", "walk"]:
		if DisplayServer.get_name() == "headless" or not OS.has_environment("WORLDSIM_MUX2_LANE_FD"):
			_fail("capture and FPS need a window and the shared GPU lock")
			return
		if RenderingServer.get_current_rendering_method() != "forward_plus":
			_fail("Forward+ required")
			return
		DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
		Engine.max_fps = 0
	if not stress and not _read_framing():
		_fail("expected --save and --crop=x,y plus optional --focus=x,y and --close=x,y")
		return
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	_world = main.get_node_or_null("WorldSim")
	_w3d = main.get_node_or_null("World3D")
	_camera = main.get_node_or_null("Camera2D")
	if _world == null or _w3d == null:
		_fail("normal game scene failed to boot")
		return
	await _frames(24)
	if stress:
		if not bool(SimReads.mutate(_world, "reset_world_stress", [0, 500])):
			_fail("stress reset failed")
			return
		var size: Dictionary = SimReads.read(_world, "get_world_info")
		_focus = Vector2(float(size["width"]), float(size["height"])) * 0.5
		_close = _focus
	elif not bool(SimReads.mutate(_world, "load_world", [_arg("--save")])):
		_fail("could not load day-120 save")
		return
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	var people: Node = _w3d.get_node("People")
	if not bool(people.call("configure_slice", _rect, "tripo_woman" if _has("people") and not _style2 else "")):
		_fail("Tripo woman VAT did not load")
		return
	if _has("people") and not _style2:
		_w3d.get_node("Animals").set("slice_models", TripoAnimals)
	if not _configure():
		_fail("ultra asset setup failed")
		return
	if action != "play":
		(main.get_node("UI") as CanvasLayer).hide()
		(_w3d.get_node("LabelLayer") as CanvasLayer).hide()
	_configure_view("z1")
	_w3d.get_node("Terrain").call("build_all")
	await _frames(30)
	var drawn: int = int(people.call("count"))
	var time: Dictionary = SimReads.read(_world, "get_time_snapshot")
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	var all: PackedInt64Array = SimReads.read(_world, "get_agent_snapshot").get("ids", PackedInt64Array())
	if (stress and (drawn != 500 or all.size() != 500)) or (not stress and (drawn < 6 or drawn > 10)):
		_fail("unexpected population in view: %d of %d" % [drawn, all.size()])
		return
	if not stress and (int(info.get("seed", -1)) != 0 or int(time.get("tick", -1)) != int(time.get("ticks_per_day", -1)) * 120):
		_fail("save is not the day-120 boundary: %s" % JSON.stringify(time))
		return
	if _has("people") or _style2:
		var expected_model: String = "s2_woman" if _style2 else "tripo_woman"
		for body: Dictionary in people.call("bodies", 0):
			var shown: Dictionary = people.call("state_of", int(body["id"]))
			if shown.get("model") != expected_model:
				_fail("a villager is missing " + expected_model)
				return
	var proof: Dictionary = {"tick": time.get("tick"), "seed": info.get("seed"), "rendered_people": drawn,
		"world_people": all.size(), "ultra": _ultra, "style2": _style2, "omitted": _omit, "stress": stress}
	print("SLICE1_READY " + JSON.stringify(proof))
	if action == "check":
		quit(0)
	elif action == "capture":
		await _capture()
	elif action == "fps":
		await _fps(proof)
	elif action == "walk":
		await _walk()


func _read_framing() -> bool:
	var crop: PackedStringArray = _arg("--crop").split(",")
	if _arg("--save") == "" or crop.size() != 2 or not crop[0].is_valid_int() or not crop[1].is_valid_int():
		return false
	var size: Vector2i = Vector2i(_point("--crop-size", Vector2(24, 24)))
	if size.x < 20 or size.y < 20 or size.x > 28 or size.y > 28:
		return false
	_rect = Rect2i(Vector2i(int(crop[0]), int(crop[1])), size)
	_focus = _point("--focus", Vector2(_rect.get_center()))
	_close = _point("--close", _focus)
	return true


func _point(key: String, fallback: Vector2) -> Vector2:
	var parts: PackedStringArray = _arg(key).split(",")
	if parts.size() == 2 and parts[0].is_valid_float() and parts[1].is_valid_float():
		return Vector2(float(parts[0]), float(parts[1]))
	return fallback


func _configure() -> bool:
	if _has("pbr"):
		for layer: String in ["Terrain", "Structures", "Flora", "Props"]:
			if not bool(_w3d.get_node(layer).call("set_slice_pbr", true)):
				return false
	if _has("vegetation"):
		_w3d.get_node("Flora").call("configure_slice_vegetation", _rect)
	if not _ultra:
		return true
	var sky: Node = _w3d.get_node("Sky")
	var env: Environment = sky.get("environment")
	var sun: DirectionalLight3D = sky.get("sun")
	env.sdfgi_enabled = _has("sdfgi")
	env.sdfgi_cascades = 4
	env.sdfgi_min_cell_size = 0.25
	env.sdfgi_energy = 0.75
	env.ssao_enabled = _has("ssao")
	env.ssil_enabled = _has("ssil")
	env.ssil_radius = 2.0
	env.ssil_intensity = 0.65
	env.volumetric_fog_enabled = _has("volumetric")
	env.volumetric_fog_density = 0.0024
	env.volumetric_fog_length = 56.0
	env.volumetric_fog_anisotropy = 0.55
	if _has("sun"):
		sky.set("slice_day_elevation", 35.0)
	if _has("resolution"):
		root.scaling_3d_mode = Viewport.SCALING_3D_MODE_BILINEAR
		root.scaling_3d_scale = 1.0
	root.use_taa = _has("taa")
	if _has("shadows"):
		sun.directional_shadow_mode = DirectionalLight3D.SHADOW_PARALLEL_4_SPLITS
		RenderingServer.directional_soft_shadow_filter_set_quality(RenderingServer.SHADOW_QUALITY_SOFT_ULTRA)
	return true


func _set_light(hour: float) -> void:
	var sky: Node = _w3d.get_node("Sky")
	sky.call("set_hour_override", hour)
	if _arg("--anchor-shots") == "1" or _style2:
		sky.call("set_hardship_override", 0.0)
	await process_frame
	await process_frame


func _configure_view(shot: String) -> bool:
	var rig: Node = _w3d.get_node("CameraRig")
	rig.set_process(shot != "low")
	var target: Vector2 = _focus if shot == "z1" else _close
	var zoom: float = 1.0 if shot == "z1" else 4.0
	if shot == "forest" or shot == "roof":
		target = _point("--" + shot, _focus)
		zoom = 2.0 if shot == "roof" else 2.8
		if _arg("--anchor-shots") == "1":
			zoom = 4.0 if shot == "roof" else 2.0
	_camera.zoom = Vector2.ONE * zoom
	_camera.position = target * float(View.TILE)
	_camera.call("_clamp_position")
	rig.call("update_camera")
	if shot != "low":
		return true
	var people: Node = _w3d.get_node("People")
	var look_at: Vector2 = _point("--low-look", _focus)
	var bodies: Array = people.call("bodies", 0)
	var nearest: float = INF
	var feet: Vector3
	var forward: Vector3
	for body: Dictionary in bodies:
		var p: Vector3 = body["feet"]
		var distance: float = Vector2(p.x, p.z).distance_squared_to(_close)
		var state: Dictionary = people.call("state_of", int(body["id"]))
		var look: Vector3 = Vector3.BACK.rotated(Vector3.UP, float(state.get("yaw", 0.0)))
		if _arg("--low-look") != "" and look.dot(Vector3(look_at.x - p.x, 0.0, look_at.y - p.z)) < 0.0:
			distance += 100.0
		if distance < nearest:
			nearest = distance
			feet = p
			forward = look
	if not is_finite(nearest):
		return false
	var cam: Camera3D = rig.get("camera")
	var pos: Vector3 = feet - forward * 3.6 + forward.cross(Vector3.UP) * 0.65
	var height: float = float(_w3d.get_node("Terrain").call("height_at", pos.x, pos.z))
	pos.y = maxf(height, feet.y) + 1.72
	var aim: Vector3 = feet + forward * 2.8 + Vector3.UP * 0.8
	if _arg("--low-look") != "":
		aim = Vector3(look_at.x, feet.y + 0.9, look_at.y)
	cam.fov = 60.0
	cam.near = 0.15
	cam.look_at_from_position(pos, aim, Vector3.UP)
	RenderingServer.global_shader_parameter_set("w3d_camera", pos)
	RenderingServer.global_shader_parameter_set("w3d_focus", aim)
	RenderingServer.global_shader_parameter_set("w3d_xray", 0.0)
	return true


func _capture() -> void:
	var out: String = _arg("--out")
	if out == "" or DirAccess.make_dir_recursive_absolute(out) != OK:
		_fail("could not create capture directory")
		return
	var anchor_shots: bool = _arg("--anchor-shots") == "1"
	var periods: Array[String] = ["day"]
	if not anchor_shots:
		periods.append("dusk")
	for period: String in periods:
		await _set_light(12.0 if period == "day" else 18.8)
		var cam: Camera3D = _w3d.call("camera")
		var fov: float = cam.fov
		var near: float = cam.near
		var shots: Array[String] = ["z1", "z4", "low"]
		if anchor_shots:
			shots.erase("z4")
		if period == "day":
			for detail: String in ["forest", "roof"]:
				if _arg("--" + detail) != "":
					shots.append(detail)
		for shot: String in shots:
			cam.fov = fov
			cam.near = near
			if not _configure_view(shot):
				_fail("no villager for behind-the-back shot")
				return
			await _frames(96)
			await RenderingServer.frame_post_draw
			var arm: String = "anchor-s2" if _style2 else ("slice" if _ultra else "current")
			var name: String = "%s_%s_%s.png" % [arm, period, shot]
			var image: Image = root.get_texture().get_image()
			if image.get_size() != Vector2i(1920, 1080) or image.save_png(out.path_join(name)) != OK:
				_fail("capture dimensions or write failed: %s %s" % [name, image.get_size()])
				return
			print("SLICE1_SHOT " + name)
		cam.fov = fov
		cam.near = near
	print("SLICE1_CAPTURE_DONE")
	quit(0)


func _walk() -> void:
	var out: String = _arg("--out")
	if not _style2 or out.is_empty() or DirAccess.make_dir_recursive_absolute(out) != OK:
		_fail("walk capture requires Style 2 and an output directory")
		return
	var aid: int = _arg("--walk-agent", "20").to_int()
	var target: Vector2i = Vector2i(_point("--walk-to", Vector2(89, 110)))
	var people: Node = _w3d.get_node("People")
	var ids: PackedInt64Array = SimReads.read(_world, "get_agent_snapshot").get("ids", PackedInt64Array())
	var agent_ids: PackedInt64Array = SimReads.read(_world, "get_agent_snapshot").get("agent_ids", PackedInt64Array())
	var i: int = agent_ids.find(aid)
	if i < 0 or i >= ids.size():
		_fail("walk subject missing from save")
		return
	var bits: int = ids[i]
	var draft: Dictionary = SimReads.mutate(_world, "cmd_draft", [aid, true])
	var order: Dictionary = SimReads.mutate(_world, "cmd_draft_move", [aid, target.x, target.y])
	if not bool(draft.get("accepted", false)) or not bool(order.get("accepted", false)):
		_fail("walk order failed: %s %s" % [draft, order])
		return
	await _set_light(12.0)
	var rig: Node = _w3d.get_node("CameraRig")
	rig.set_process(false)
	var cam: Camera3D = rig.get("camera")
	cam.fov = 41.0
	cam.near = 0.15
	RenderingServer.global_shader_parameter_set("w3d_xray", 0.0)
	_world.process_mode = Node.PROCESS_MODE_INHERIT
	var origin: Vector3 = (people.call("state_of", bits) as Dictionary)["feet"]
	var feet: Vector3 = origin
	for _sample: int in 96:
		await process_frame
		feet = (people.call("state_of", bits) as Dictionary)["feet"]
		if feet.distance_to(origin) > 0.12:
			break
	if feet.distance_to(origin) <= 0.12:
		_fail("drafted villager did not start moving")
		return
	origin = feet
	var walking: int = 0
	for frame: int in 36:
		var body: Dictionary = people.call("state_of", bits)
		feet = body["feet"]
		if body["clip"] == "walk":
			walking += 1
		var forward: Vector3 = Vector3.BACK.rotated(Vector3.UP, float(body["yaw"]))
		var camera_position: Vector3 = feet - forward * 1.35 + forward.cross(Vector3.UP) * 3.55 + Vector3.UP * 1.40
		cam.look_at_from_position(camera_position, feet + Vector3.UP * 0.86)
		RenderingServer.global_shader_parameter_set("w3d_camera", camera_position)
		await RenderingServer.frame_post_draw
		if root.get_texture().get_image().save_png(out.path_join("walk-%02d.png" % frame)) != OK:
			_fail("could not save walk frame")
			return
		await _frames(4) # Run with --fixed-fps 48 for 12 sampled poses per second.
	if feet.distance_to(origin) < 2.0 or walking < 20:
		_fail("walk sequence had %.2f metres and %d walk poses" % [feet.distance_to(origin), walking])
		return
	print("ANCHOR2_WALK_DONE " + JSON.stringify({"agent": aid, "distance": feet.distance_to(origin), "frames": 36, "walk_poses": walking, "fps": 12}))
	quit(0)


func _fps(proof: Dictionary) -> void:
	var shot: String = _arg("--shot", "z1")
	if shot not in ["z1", "z4"]:
		_fail("expected z1 or z4 FPS shot")
		return
	_configure_view(shot)
	await _set_light(12.0)
	var live: bool = _arg("--live") == "1"
	if live:
		_world.process_mode = Node.PROCESS_MODE_INHERIT
	var end: int = Time.get_ticks_msec() + 3500
	while Time.get_ticks_msec() < end:
		await process_frame
	var viewport: RID = root.get_viewport_rid()
	RenderingServer.viewport_set_measure_render_time(viewport, true)
	var start: int = Time.get_ticks_usec()
	var n0: int = _draws
	var gpu: float = 0.0
	var samples: int = 0
	while Time.get_ticks_usec() - start < 6000000:
		await process_frame
		gpu += RenderingServer.viewport_get_measured_render_time_gpu(viewport)
		samples += 1
	var seconds: float = float(Time.get_ticks_usec() - start) / 1000000.0
	var frames: int = _draws - n0
	if frames <= 0 or root.get_texture().get_size() != Vector2(1920, 1080):
		_fail("FPS run was not rendered at 1920×1080")
		return
	print("SLICE1_FPS " + JSON.stringify(proof.merged({
		"shot": shot, "frames": frames, "seconds": seconds, "fps": float(frames) / seconds,
		"gpu_ms": gpu / float(samples), "size": [1920, 1080], "focus": root.has_focus(),
		"paused": not live, "end_tick": SimReads.read(_world, "get_time_snapshot").get("tick"), "driver": RenderingServer.get_current_rendering_driver_name(),
	})))
	quit(0)
