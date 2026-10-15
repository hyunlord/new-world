extends SceneTree
## Future Part 2 evidence only; requires explicit renderer metadata contract v1.
const ReadRoute = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
const FixedDelta = preload("res://scripts/test/art/test_fixed_delta_p9.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const REQUIRED_META: Array[String] = ["p10_activity_schema", "activity_frame_view", "activity_frame_activity", "activity_kind", "activity_place_known", "activity", "pose", "body", "facing", "sprite_id", "cold_pose", "cold_base_pose", "activity_overrides", "entity_bits", "art_id"]
signal frame_settled
var _samples: int = 0
var _valid_delta: bool = true
var _minimum_delta: float = INF
var _maximum_delta: float = 0.0
var _errors: Array[String] = []
var _main: Node
var _world: Node
var _out: String

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	_out = OS.get_environment("P10_ACTIVITY_OUT")
	var seed: int = int(OS.get_environment("P10_ACTIVITY_SEED"))
	var day: int = int(OS.get_environment("P10_ACTIVITY_DAY"))
	var hour: int = int(OS.get_environment("P10_ACTIVITY_HOUR"))
	var zoom: int = int(OS.get_environment("P10_ACTIVITY_ZOOM"))
	if _out.is_empty() or seed <= 0 or day not in [30, 60] or hour not in [10, 15] or zoom not in [1, 4] or (day == 60 and hour != 10) or (day == 30 and seed != 20261010):
		_fail("invalid explicit capture specification")
		return
	if FileAccess.file_exists(_out.path_join("capture.json")) or FileAccess.file_exists(_out.path_join("capture.png")):
		_fail("refusing to overwrite capture evidence")
		return
	root.size = Vector2i(1920, 1080)
	var scene: PackedScene = load("res://scenes/main.tscn") as PackedScene
	_main = scene.instantiate()
	_world = _main.get_node("WorldSim")
	var camera: Camera2D = _main.get_node("Camera2D") as Camera2D
	_world.call("set_sim_speed", 0.0)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(_main)
	current_scene = _main
	camera.call("set_auto_track", false)
	camera.position_smoothing_enabled = false
	camera.process_mode = Node.PROCESS_MODE_DISABLED
	await _until_frame(10)
	if Engine.get_process_frames() != 10:
		_fail("startup missed state-change frame 10")
		return
	ReadRoute.mutate(_world, "reset_world", [seed])
	var info: Dictionary = ReadRoute.read(_world, "get_world_info")
	var ticks_per_day: int = int(info["ticks_per_day"])
	var tick: int = (day - 1) * ticks_per_day + int(float(hour) * ticks_per_day / 24.0) - int(info["clock_offset_ticks"])
	if int(ReadRoute.mutate(_world, "advance_ticks", [tick])) != tick:
		_fail("advance_ticks failed exact target")
		return
	Selection.set_layers_enabled(self, true)
	var settlements: Dictionary = ReadRoute.read(_world, "get_settlement_snapshot")
	if settlements["settlement_ids"].is_empty():
		_fail("capture needs an existing settlement anchor")
		return
	var first: int = 0
	for i: int in settlements["settlement_ids"].size():
		if int(settlements["settlement_ids"][i]) < int(settlements["settlement_ids"][first]):
			first = i
	camera.position = Vector2(448, 28) + Vector2(settlements["formation_xs"][first], settlements["formation_ys"][first]) * 16.0 + Vector2(8, 8)
	camera.zoom = Vector2.ONE * zoom
	camera.reset_smoothing()
	camera.force_update_scroll()
	for name: String in ["TerrainRenderer", "BuildingRenderer", "AgentRenderer"]:
		var renderer: Node = _main.get_node(name)
		if renderer.has_method("refresh_snapshot"):
			renderer.call("refresh_snapshot")
	await _until_frame(130)
	RenderingServer.force_draw(false)
	var clock: Dictionary = ReadRoute.read(_world, "get_time_snapshot")
	var agents: Dictionary = ReadRoute.read(_world, "get_agent_snapshot")
	var activities: Dictionary = ReadRoute.read(_world, "get_activity_snapshot")
	var rows: Array[Dictionary] = _holders(agents, activities)
	var image: Image = root.get_texture().get_image()
	if not _valid_delta or _samples == 0 or Engine.get_process_frames() != 130 or int(clock["tick"]) != tick or image.get_size() != Vector2i(1920, 1080):
		_errors.append("frame, delta, tick or image invariant failed")
	if DirAccess.make_dir_recursive_absolute(_out) != OK:
		_fail("cannot create evidence directory")
		return
	if image.save_png(_out.path_join("capture.png")) != OK:
		_errors.append("image write failed")
	var file: FileAccess = FileAccess.open(_out.path_join("capture.json"), FileAccess.WRITE)
	if file == null:
		_fail("cannot write sidecar")
		return
	file.store_string(JSON.stringify({"schema_version": 1, "complete": _errors.is_empty(), "errors": _errors,
		"seed": seed, "display_day": day, "hour": hour, "zoom": zoom, "tick": tick,
		"state_change_frame": 10, "capture_frame": 130, "fixed_fps": 60, "delta_samples": _samples,
		"fixed_delta_valid": _valid_delta, "delta_tolerance": 0.000000001, "minimum_delta": _minimum_delta, "maximum_delta": _maximum_delta,
		"world_info": info, "clock": clock, "settlements": settlements, "population": agents["ids"].size(),
		"camera": [camera.position.x, camera.position.y], "viewport": [1920, 1080],
		"raw_agent_snapshot": agents, "raw_activity_snapshot": activities, "activity_holders": rows, "holder_count": activities["ids"].size(),
		"pose_oracle_checked": false, "debug_build": OS.is_debug_build()}, "\t") + "\n")
	file.close()
	quit(0 if _errors.is_empty() else 1)

func _holders(agents: Dictionary, activities: Dictionary) -> Array[Dictionary]:
	var by_entity: Dictionary = {}
	for i: int in agents["ids"].size():
		by_entity[int(agents["ids"][i])] = i
	var sprites: Dictionary = {}
	for node: Node in _main.get_node("WorldObjects").get_children():
		if node is Sprite2D and str(node.get_meta("art_kind", "")) == "agent":
			sprites[int(node.get_meta("entity_bits", -1))] = node
	var rows: Array[Dictionary] = []
	for i: int in activities["ids"].size():
		var entity: int = int(activities["ids"][i])
		var index: int = int(by_entity.get(entity, -1))
		var row: Dictionary = {"entity_bits": entity, "kind": int(activities["kinds"][i]),
			"place": [activities["place_x"][i], activities["place_y"][i]], "until_tick": activities["until_tick"][i],
			"agent_id": int(agents["agent_ids"][index]) if index >= 0 else -1,
			"state_tag": int(agents["states"][index]) if index >= 0 else -1, "renderer_present": sprites.has(entity)}
		if index < 0 or not sprites.has(entity):
			_errors.append("activity holder lacks live agent/sprite: %d" % entity)
			rows.append(row)
			continue
		var sprite: Sprite2D = sprites[entity]
		var missing: Array[String] = []
		for key: String in REQUIRED_META:
			if not sprite.has_meta(key):
				missing.append(key)
		if not missing.is_empty() or int(sprite.get_meta("p10_activity_schema", 0)) != 1:
			_errors.append("incomplete final activity metadata: %d" % entity)
		if int(sprite.get_meta("activity_kind", -1)) != int(row["kind"]) or bool(sprite.get_meta("activity_place_known", false)) != (int(row["until_tick"]) >= 0) or int(sprite.get_meta("art_id", -1)) != int(row["agent_id"]):
			_errors.append("raw/render metadata identity mismatch: %d" % entity)
		row["missing_metadata"] = missing
		var metadata: Dictionary = {}
		for key: StringName in sprite.get_meta_list():
			metadata[str(key)] = sprite.get_meta(key)
		row["metadata"] = metadata
		var rect: Rect2 = sprite.get_global_transform_with_canvas() * sprite.get_rect()
		row["viewport_rect"] = [rect.position.x, rect.position.y, rect.size.x, rect.size.y]
		row["visible"] = sprite.visible
		row["visible_in_tree"] = sprite.is_visible_in_tree()
		row["intersects_viewport"] = rect.intersects(Rect2(0, 0, 1920, 1080))
		row["fully_inside_viewport"] = Rect2(0, 0, 1920, 1080).encloses(rect)
		row["flip_h"] = sprite.flip_h
		row["texture_path"] = sprite.texture.resource_path if sprite.texture != null else ""
		row["region_enabled"] = sprite.region_enabled
		row["region_rect"] = [sprite.region_rect.position.x, sprite.region_rect.position.y, sprite.region_rect.size.x, sprite.region_rect.size.y]
		rows.append(row)
	return rows

func _until_frame(target: int) -> void:
	while Engine.get_process_frames() < target:
		await process_frame
		call_deferred("_settled")
		await frame_settled

func _settled() -> void:
	frame_settled.emit()

func _process(delta: float) -> bool:
	_samples += 1
	_minimum_delta = minf(_minimum_delta, delta)
	_maximum_delta = maxf(_maximum_delta, delta)
	_valid_delta = _valid_delta and FixedDelta.accepts_delta(delta)
	return false

func _fail(message: String) -> void:
	push_error("activity-capture-p10: " + message)
	quit(1)
