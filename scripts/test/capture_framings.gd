extends SceneTree

const Metadata = preload("res://scripts/test/art/capture_metadata.gd")
const CapturePlans = preload("res://scripts/test/art/capture_plans.gd")

const ORIGIN: Vector2 = Vector2(448, 28)
const TILE_SIZE: float = 16.0
const BASELINE: Array[Array] = [[0, "F1"], [0, "F2"], [1440, "F1"], [1440, "F2"]]
const FINAL: Array[Array] = [
	[1, "12:00", "F1"], [1, "12:00", "F2"],
	[2, "12:00", "F1"], [2, "12:00", "F2"], [2, "12:00", "F3"],
	[2, "12:00", "F4"], [2, "12:00", "F2A"], [2, "22:00", "F2N"],
	[6, "12:00", "F1"], [6, "12:00", "F2"],
]

var _out: String = ""
var _plan: String = ""
var _main: Node = null
var _world: Node = null
var _camera: Camera2D = null
var _manifest: Array[Dictionary] = []
var _selected_seeking: bool = false
var _clock_offset: int = 0
var _failed: bool = false


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = 0
	while i < args.size():
		if args[i] in ["--out", "--plan"] and i + 1 < args.size():
			if args[i] == "--out":
				_out = args[i + 1]
			else:
				_plan = args[i + 1]
			i += 2
		else:
			push_error("capture: unknown or missing argument: " + args[i])
			quit(2)
			return
	if _out.is_empty() or _plan not in ["baseline", "final", "art", "calibration"]:
		push_error("capture: requires --out DIRECTORY --plan baseline|final|art|calibration")
		quit(2)
		return
	call_deferred("_run")


func _run() -> void:
	if DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(_out)) != OK:
		_fail("cannot create capture directory")
		return
	var old_manifest: String = ProjectSettings.globalize_path(_out.path_join("manifest.json"))
	if FileAccess.file_exists(old_manifest) and DirAccess.remove_absolute(old_manifest) != OK:
		_fail("cannot invalidate previous capture manifest")
		return
	if _plan == "calibration":
		await _capture_calibration()
		_write_manifest()
		return
	var packed: PackedScene = load("res://scenes/main.tscn") as PackedScene
	if packed == null:
		_fail("main scene not loadable")
		return
	_main = packed.instantiate()
	_world = _main.get_node_or_null("WorldSim")
	_camera = _main.get_node_or_null("Camera2D") as Camera2D
	if _world == null or _camera == null:
		_main.free()
		_fail("missing WorldSim or Camera2D")
		return
	# Freeze before adding the scene: even the first process frame is tick zero.
	_world.call("set_sim_speed", 0.0)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(_main)
	current_scene = _main
	_camera.call("set_auto_track", false)
	_camera.position_smoothing_enabled = false
	var start: Dictionary = _world.call("get_time_snapshot")
	if int(start["tick"]) != 0:
		_fail("initial simulation tick is not zero")
		return
	var info: Dictionary = _world.call("get_world_info")
	_clock_offset = int(info.get("clock_offset_ticks", 0))
	if int(info.get("ticks_per_day", 0)) != 1440:
		_fail("clock plan requires 1440 ticks per day")
		return
	var plan: Array[Array] = BASELINE if _plan == "baseline" else (CapturePlans.ART if _plan == "art" else FINAL)
	for shot: Array in plan:
		var target: int = int(shot[0])
		var framing: String = str(shot[1])
		var stem: String = "%s_t%d_%s" % [_plan, target, framing]
		if _plan != "baseline":
			var clock: PackedStringArray = str(shot[1]).split(":")
			target = (int(shot[0]) - 1) * 1440 + int(clock[0]) * 60 + int(clock[1]) - _clock_offset
			framing = str(shot[2])
			stem = ("a_" if _plan == "art" else "") + "d%d_%s%s_%s" % [int(shot[0]), clock[0], clock[1], framing]
		var before: Dictionary = _world.call("get_time_snapshot")
		var delta_ticks: int = target - int(before["tick"])
		if delta_ticks < 0 or not _advance_to(target):
			_fail("simulation did not reach requested tick")
			return
		var tile: Vector2 = Vector2(32, 32) if framing == "F1" else _village_tile()
		if framing == "F3":
			tile = _food_tile(tile)
			if tile.x < 0:
				_fail("F3 requires a food source; none exists at this tick")
				return
		_selected_seeking = false
		_set_agent_layers(framing != "F2A")
		var inspector: Node = _main.get_node_or_null("UI/AgentInspectorPanel")
		if inspector != null:
			inspector.call("close")
		if framing == "F4" and not _open_inspector(inspector):
			_fail("inspector capture requires a living agent and inspector")
			return
		_camera.position = ORIGIN + tile * TILE_SIZE + Vector2(8, 8)
		_camera.zoom = Vector2.ONE if framing == "F1" else (Vector2(6, 6) if framing == "F2Z" else Vector2(4, 4))
		_camera.reset_smoothing()
		_camera.force_update_scroll()
		# Reconcile infrequent object renderers after synchronous tooling ticks.
		for node_name: String in ["TerrainRenderer", "BuildingRenderer", "AgentRenderer"]:
			var renderer: Node = _main.get_node_or_null(node_name)
			if renderer != null and renderer.has_method("refresh_snapshot"):
				renderer.call("refresh_snapshot")
		await create_timer(0.4).timeout
		for frame: int in range(5):
			await process_frame
		await RenderingServer.frame_post_draw
		var now: Dictionary = _world.call("get_time_snapshot")
		if int(now["tick"]) != target:
			_fail("wall-clock advanced a frozen capture")
			return
		var image: Image = root.get_texture().get_image()
		if image == null or image.get_width() < 1920 or image.get_height() < 1080 or image.get_width() * 9 != image.get_height() * 16:
			_fail("capture framebuffer must be at least 1920x1080 with 16:9 aspect; no placeholder is written")
			return
		var filename: String = stem + ".png"
		if image.save_png(ProjectSettings.globalize_path(_out.path_join(filename))) != OK:
			_fail("cannot save " + filename)
			return
		if not _write_entities(stem, target, image.get_size()):
			return
		_manifest.append({
			"file": filename, "tick": target, "framing": framing,
			"tile_x": tile.x, "tile_y": tile.y, "zoom": _camera.zoom.x,
			"time_of_day": now["time_of_day"], "width": image.get_width(), "height": image.get_height(),
			"entities": stem + ".entities.json",
		})
		if _plan == "art" and stem == "a_d2_1200_F2":
			_write_forest_evidence()
		if _plan == "art" and stem == "a_d2_1200_F1":
			await _capture_strip("a_strip", true)
			_camera.zoom = Vector2.ONE
			_camera.force_update_scroll()
			await _measure_frame_times()
		if _plan == "art" and stem == "a_d2_1930_F2":
			await _capture_strip("a_evening", false)
	if _plan == "art":
		_main.queue_free()
		await process_frame
		await _capture_calibration()
	_write_manifest()


func _write_manifest() -> void:
	if _failed:
		quit(1)
		return
	var manifest: FileAccess = FileAccess.open(_out.path_join("manifest.json"), FileAccess.WRITE)
	if manifest == null:
		_fail("cannot write manifest")
		return
	manifest.store_string(JSON.stringify({
		"schema_version": 2, "plan": _plan, "git_sha": OS.get_environment("WORLDSIM_GIT_SHA"),
		"captures": _manifest, "worktree_changed_paths": int(OS.get_environment("WORLDSIM_GIT_DIRTY")),
	}, "\t") + "\n")
	manifest.close()
	quit(0)


func _village_tile() -> Vector2:
	var snap: Dictionary = _world.call("get_settlement_snapshot")
	var ids: PackedInt32Array = snap["settlement_ids"]
	var counts: PackedInt32Array = snap["member_counts"]
	var xs: PackedInt32Array = snap["formation_xs"]
	var ys: PackedInt32Array = snap["formation_ys"]
	var best: int = -1
	for i: int in range(ids.size()):
		if best < 0 or counts[i] > counts[best] or (counts[i] == counts[best] and ids[i] < ids[best]):
			best = i
	return Vector2(32, 32) if best < 0 else Vector2(xs[best], ys[best])


func _food_tile(village: Vector2) -> Vector2:
	var snap: Dictionary = _world.call("get_resource_snapshot")
	var kinds: PackedInt32Array = snap["kinds"]
	var xs: PackedInt32Array = snap["xs"]
	var ys: PackedInt32Array = snap["ys"]
	var best: Vector2 = Vector2(-1, -1)
	var distance: float = INF
	for i: int in range(kinds.size()):
		if kinds[i] != 0:
			continue
		var tile: Vector2 = Vector2(xs[i], ys[i])
		var candidate: float = tile.distance_squared_to(village)
		if candidate < distance or (candidate == distance and (tile.x < best.x or (tile.x == best.x and tile.y < best.y))):
			best = tile
			distance = candidate
	return best


func _open_inspector(inspector: Node) -> bool:
	if inspector == null:
		return false
	var snap: Dictionary = _world.call("get_agent_snapshot")
	var ids: PackedInt64Array = snap["ids"]
	var agent_ids: PackedInt64Array = snap["agent_ids"]
	if ids.is_empty():
		return false
	var best: int = 0
	for i: int in range(agent_ids.size()):
		if agent_ids[i] < agent_ids[best]:
			best = i
	var detail: Dictionary = _world.call("get_agent_detail", ids[best])
	if not bool(detail.get("found", false)):
		return false
	_selected_seeking = int(detail.get("state_tag", -1)) == 1
	detail["entity_bits"] = ids[best]
	inspector.call("display_agent", detail)
	return true


func _fail(message: String) -> void:
	_failed = true
	push_error("capture: " + message)
	quit(1)


func _advance_to(target: int) -> bool:
	var clock: Dictionary = _world.call("get_time_snapshot")
	var tick: int = int(clock["tick"])
	var trail: Node = _main.get_node_or_null("ActivityTrailRenderer")
	while tick < target:
		tick = int(_world.call("advance_ticks", mini(8, target - tick)))
		if trail != null:
			trail.call("_process", 0.0)
	return tick == target


func _set_agent_layers(shown: bool) -> void:
	var agents: Node = _main.get_node_or_null("AgentRenderer")
	if agents != null and agents.has_method("set_capture_visible"):
		agents.call("set_capture_visible", shown)
	Metadata.show_agents(_main, shown)
	for node_name: String in ["SeekVizRenderer", "NeedBarRenderer", "ActivityTrailRenderer", "DeathVizRenderer"]:
		var layer: CanvasItem = _main.get_node_or_null(node_name) as CanvasItem
		if layer != null:
			layer.visible = shown


func _write_entities(stem: String, tick: int, image_size: Vector2i) -> bool:
	var viewport: Vector2 = Vector2(image_size)
	var pixel_scale: Vector2 = viewport / root.get_visible_rect().size
	var rows: Array[Dictionary] = []
	rows = Metadata.entities(_main, viewport, pixel_scale)
	var inspector: Node = _main.get_node("UI/AgentInspectorPanel")
	var data: Dictionary = {
		"schema_version": 3, "camera_position": [_camera.position.x, _camera.position.y],
		"zoom": [_camera.zoom.x, _camera.zoom.y], "viewport_size": [image_size.x, image_size.y],
		"clock_offset_ticks": _clock_offset, "tick": tick, "entities": rows,
		"framebuffer_scale": pixel_scale.x, "labels": Metadata.labels(_main.get_node_or_null("UI"), pixel_scale),
		"agent_snapshot": _json_arrays(_world.call("get_agent_snapshot")),
		"time_snapshot": _json_arrays(_world.call("get_time_snapshot")),
		"tallies": _json_arrays(_world.call("get_tallies")),
		"recent_deaths": _json_arrays(_world.call("get_recent_deaths")),
		"head_dots": _draw_counter("SeekVizRenderer", "get_drawn_head_dot_count"),
		"need_icons": _draw_counter("NeedBarRenderer", "get_drawn_icon_count"),
		"need_icon_agent_ids": _draw_counter("NeedBarRenderer", "get_drawn_icon_agent_ids"),
		"selected_entity_bits": int(inspector.call("get_selected_entity_bits")), "selected_seeking": _selected_seeking,
		"seek_segments": int(_main.get_node("SeekVizRenderer").call("get_drawn_segment_count")),
		"trail_segments": int(_main.get_node("ActivityTrailRenderer").call("get_drawn_segment_count")),
	}
	var file: FileAccess = FileAccess.open(_out.path_join(stem + ".entities.json"), FileAccess.WRITE)
	if file == null:
		_fail("cannot write entity sidecar")
		return false
	file.store_string(JSON.stringify(data, "\t") + "\n")
	file.close()
	return true


func _measure_frame_times() -> void:
	_world.process_mode = Node.PROCESS_MODE_INHERIT
	_world.call("set_sim_speed", 1.0)
	var elapsed: Array[float] = []
	await RenderingServer.frame_post_draw
	var previous: int = Time.get_ticks_usec()
	for frame: int in range(300):
		await RenderingServer.frame_post_draw
		var current: int = Time.get_ticks_usec()
		elapsed.append(float(current - previous) / 1000.0)
		previous = current
	_world.call("set_sim_speed", 0.0)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	var ordered: Array[float] = elapsed.duplicate()
	ordered.sort()
	if not CapturePlans.write_json(_out.path_join("frame_times.json"), {
		"frames": 300, "zoom": 1, "median_ms": (ordered[149] + ordered[150]) / 2.0,
		"p95_ms": ordered[284], "samples_ms": elapsed, "simulation_frozen": false,
	}):
		_fail("cannot write frame timing evidence")


func _capture_strip(prefix: String, follow: bool) -> void:
	var followed: int = CapturePlans.moving_agent(_world) if follow else -1
	var frames: Array[Dictionary] = []
	var images: Array[Image] = []
	var clip_start_usec: int = Time.get_ticks_usec()
	var original_position: Vector2 = _camera.position
	_camera.zoom = Vector2(6, 6) if follow else Vector2(4, 4)
	_world.process_mode = Node.PROCESS_MODE_INHERIT
	_world.call("set_sim_speed", 1.0)
	for index: int in range(40):
		var start_usec: int = Time.get_ticks_usec()
		if followed >= 0:
			var position: Vector2 = CapturePlans.agent_position(_main, followed)
			if position != Vector2.INF:
				_camera.position = position
		_camera.force_update_scroll()
		await RenderingServer.frame_post_draw
		var now: Dictionary = _world.call("get_time_snapshot")
		var stem: String = "%s_%02d" % [prefix, index]
		var image: Image = root.get_texture().get_image()
		if image == null:
			_fail("cannot read animation framebuffer")
			return
		images.append(image)
		if not _write_entities(stem, int(now["tick"]), image.get_size()):
			return
		var entry: Dictionary = {"file": stem + ".png", "entities": stem + ".entities.json",
			"tick": int(now["tick"]), "framing": prefix, "zoom": _camera.zoom.x,
			"width": image.get_width(), "height": image.get_height(),
			"real_time_usec": start_usec, "followed_agent_id": followed}
		frames.append(entry)
		_manifest.append(entry)
		var remaining: float = float(clip_start_usec + (index + 1) * 100000 - Time.get_ticks_usec()) / 1000000.0
		if remaining > 0:
			await create_timer(remaining).timeout
	_world.call("set_sim_speed", 0.0)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	_camera.position = original_position
	# Encode after freezing so PNG compression cannot change the sampling cadence.
	for index: int in images.size():
		if images[index].save_png(ProjectSettings.globalize_path(_out.path_join(str(frames[index]["file"])))) != OK:
			_fail("cannot save animation frame")
			return
	images.clear()
	if not CapturePlans.write_json(_out.path_join("strip.json" if follow else "evening.json"), {
		"frames": frames, "followed_agent_id": followed, "moving_agent_found": followed >= 0 if follow else null,
		"requested_interval_s": 0.1, "speed": 1.0,
	}):
		_fail("cannot write animation manifest")


func _capture_calibration() -> void:
	var path: String = "res://scenes/test/art_calibration.tscn"
	if not ResourceLoader.exists(path):
		_fail("missing required art calibration scene")
		return
	var packed: PackedScene = load(path) as PackedScene
	var scene: Node = packed.instantiate()
	root.add_child(scene)
	current_scene = scene
	var camera: Camera2D = scene.get_node_or_null("Camera2D") as Camera2D
	if camera == null:
		_fail("calibration scene requires Camera2D")
		return
	for capture_spec: Array in [[4, false], [6, false], [4, true], [2, true]]:
		var zoom_level: int = int(capture_spec[0])
		var lineup: bool = bool(capture_spec[1])
		scene.call("set_lineup", lineup)
		camera.zoom = Vector2(zoom_level, zoom_level)
		camera.force_update_scroll()
		for frame: int in range(5):
			await process_frame
		await RenderingServer.frame_post_draw
		var image: Image = root.get_texture().get_image()
		var stem: String = ("a_lineup_z%d" if lineup else "a_calib_z%d") % zoom_level
		if image == null or image.save_png(ProjectSettings.globalize_path(_out.path_join(stem + ".png"))) != OK:
			_fail("cannot save calibration framebuffer")
			return
		var viewport: Vector2 = Vector2(image.get_size())
		var pixel_scale: Vector2 = viewport / root.get_visible_rect().size
		if not CapturePlans.write_json(_out.path_join(stem + ".entities.json"), {
			"schema_version": 3, "entities": Metadata.entities(scene, viewport, pixel_scale),
			"zoom": [zoom_level, zoom_level], "viewport_size": [image.get_width(), image.get_height()],
			"framebuffer_scale": pixel_scale.x,
		}):
			_fail("cannot write calibration metadata")
			return
		_manifest.append({"file": stem + ".png", "entities": stem + ".entities.json", "framing": "a_calib",
			"zoom": zoom_level, "width": image.get_width(), "height": image.get_height()})
	scene.queue_free()


func _json_arrays(value: Variant) -> Variant:
	if value is Dictionary:
		var converted: Dictionary = {}
		for key: Variant in value:
			converted[key] = _json_arrays(value[key])
		return converted
	if value is PackedByteArray or value is PackedInt32Array or value is PackedInt64Array or value is PackedFloat32Array or value is PackedFloat64Array:
		return Array(value)
	return value


func _write_forest_evidence() -> void:
	var evidence: Dictionary = {}
	for domain: String in ["terrain", "building", "construction", "settlement", "resource"]:
		evidence[domain] = _json_arrays(_world.call("get_" + domain + "_snapshot"))
	var viewport: Vector2 = Vector2(root.get_texture().get_size())
	evidence["entities"] = Metadata.entities(_main, viewport, viewport / root.get_visible_rect().size)
	evidence["tick"] = _world.call("get_time_snapshot")["tick"]
	if not CapturePlans.write_json(_out.path_join("forest_evidence.json"), evidence):
		_fail("cannot write forest evidence")


func _draw_counter(node_name: String, method: String) -> Variant:
	var node: Node = _main.get_node_or_null(NodePath(node_name))
	return node.call(method) if node != null and node.has_method(method) else null
