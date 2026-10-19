extends SceneTree
## Future Part 2 evidence only; requires explicit renderer metadata contract v1.
const SpriteOrder = preload("res://scripts/test/art/capture_sprite_order_p13.gd")
const ReadRoute = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
const FixedDelta = preload("res://scripts/test/art/test_fixed_delta_p9.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const REQUIRED_META: Array[String] = ["activity_oracle_inputs", "p10_activity_schema", "activity_frame_view", "activity_frame_activity", "activity_kind", "activity_place_known", "activity", "pose", "body", "facing", "sprite_id", "cold_pose", "cold_base_pose", "activity_overrides", "entity_bits", "art_id"]
signal frame_settled
var _samples: int = 0
var _valid_delta: bool = true
var _minimum_delta: float = INF
var _maximum_delta: float = 0.0
var _errors: Array[String] = []
var _main: Node
var _world: Node
var _out: String
var _build_identity: Dictionary = {}
var _draw_rows: Array[Dictionary] = []
var _draw_ranks: Dictionary = {}

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	_out = OS.get_environment("P13_ACTIVITY_OUT")
	var profile: String = OS.get_environment("P13_ACTIVITY_BUILD_PROFILE") if OS.has_environment("P13_ACTIVITY_BUILD_PROFILE") else "release"
	var loaded_path: String = "res://rust/target/debug/libsim_bridge.dylib"
	var release_path: String = "res://rust/target/release/libsim_bridge.dylib"
	if profile not in ["debug", "release"] or not FileAccess.file_exists(loaded_path):
		_fail("requires debug or release profile and existing loaded bridge")
		return
	var loaded_sha: String = FileAccess.get_sha256(loaded_path)
	var release_sha: Variant = FileAccess.get_sha256(release_path) if FileAccess.file_exists(release_path) else null
	if profile == "release" and release_sha != loaded_sha:
		_fail("release capture bridge SHA mismatch")
		return
	_build_identity = {"build_profile": profile, "loaded_bridge_sha256": loaded_sha, "release_sha256": release_sha}
	var seed: int = int(OS.get_environment("P13_ACTIVITY_SEED"))
	var day: int = int(OS.get_environment("P13_ACTIVITY_DAY"))
	var hour: int = int(OS.get_environment("P13_ACTIVITY_HOUR"))
	var zoom: int = int(OS.get_environment("P13_ACTIVITY_ZOOM"))
	if _out.is_empty() or seed <= 0 or day not in [30, 60] or hour not in [10, 11, 14, 15] or zoom not in [1, 4] or (day == 30 and seed != 20261010) or (day == 60 and seed != 20261018):
		_fail("invalid explicit capture specification")
		return
	if FileAccess.file_exists(_out.path_join("capture.json")) or FileAccess.file_exists(_out.path_join("capture.png")):
		_fail("refusing to overwrite capture evidence")
		return
	# Renderer oracle metadata is gated by the original P10 capture environment.
	OS.set_environment("P10_ACTIVITY_OUT", _out)
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
	var center_kind: int = int(OS.get_environment("P13_ACTIVITY_CENTER_KIND")) if OS.has_environment("P13_ACTIVITY_CENTER_KIND") else -1
	if center_kind >= 0:
		var center_agents: Dictionary = ReadRoute.read(_world, "get_agent_snapshot")
		var center_activities: Dictionary = ReadRoute.read(_world, "get_activity_snapshot")
		var winner: int = 9223372036854775807
		for i: int in center_activities["ids"].size():
			if int(center_activities["kinds"][i]) != center_kind or int(center_activities["until_tick"][i]) < 0:
				continue
			var agent_index: int = center_agents["ids"].find(center_activities["ids"][i])
			if agent_index >= 0 and int(center_agents["agent_ids"][agent_index]) < winner:
				winner = int(center_agents["agent_ids"][agent_index])
				camera.position = Vector2(448, 28) + Vector2(center_agents["xs"][agent_index], center_agents["ys"][agent_index]) * 16.0 + Vector2(8, 8)
		if winner == 9223372036854775807:
			_fail("no arrived holder of requested added-capture kind")
			return
	camera.zoom = Vector2.ONE * zoom
	camera.reset_smoothing()
	camera.force_update_scroll()
	for name: String in ["TerrainRenderer", "BuildingRenderer", "AgentRenderer"]:
		var renderer: Node = _main.get_node(name)
		if renderer.has_method("refresh_snapshot"):
			renderer.call("refresh_snapshot")
	if OS.has_environment("P13_ACTIVITY_SELECTED_ENTITY"):
		await _until_frame(12)
		var entity: int = int(OS.get_environment("P13_ACTIVITY_SELECTED_ENTITY"))
		var detail: Dictionary = _world.call("get_agent_detail", entity)
		detail["entity_bits"] = entity
		_main.get_node("UI/AgentInspectorPanel").call("display_agent", detail)
	await _until_frame(130)
	RenderingServer.force_draw(false)
	var clock: Dictionary = ReadRoute.read(_world, "get_time_snapshot")
	var agents: Dictionary = ReadRoute.read(_world, "get_agent_snapshot")
	var activities: Dictionary = ReadRoute.read(_world, "get_activity_snapshot")
	var ordering := SpriteOrder.new()
	_draw_rows = ordering.collect(_main.get_node("WorldObjects") as Node2D)
	for error: String in ordering.errors:
		_errors.append(error)
	for row: Dictionary in _draw_rows:
		_draw_ranks[row["node_path"]] = row["draw_rank"]
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
	file.store_string(JSON.stringify(_json_ready({"schema_version": 1, "complete": _errors.is_empty(), "errors": _errors,
		"build_identity": _build_identity, "seed": seed, "display_day": day, "hour": hour, "zoom": zoom, "tick": tick,
		"state_change_frame": 10, "capture_frame": 130, "fixed_fps": 60, "delta_samples": _samples,
		"fixed_delta_valid": _valid_delta, "delta_tolerance": 0.000000001, "minimum_delta": _minimum_delta, "maximum_delta": _maximum_delta,
		"world_info": info, "clock": clock, "settlements": settlements, "population": agents["ids"].size(),
		"selected_entity_bits": _main.get_node("UI/AgentInspectorPanel").call("get_selected_entity_bits"),
		"crowd_centre_tile": [(camera.position.x - 456.0) / 16.0, (camera.position.y - 36.0) / 16.0],
		"camera": [camera.position.x, camera.position.y], "viewport": [1920, 1080],
		"draw_order_schema": 3, "draw_order_complete": ordering.errors.is_empty(), "draw_rects": _draw_rows, "village_labels": _main.get_node("SettlementLabels").call("capture_metadata"), "center_kind": center_kind, "raw_building_snapshot": ReadRoute.read(_world, "get_building_snapshot"),
		"raw_society_snapshot": ReadRoute.read(_world, "get_society_snapshot"), "raw_agent_snapshot": agents, "raw_activity_snapshot": activities, "activity_holders": rows, "holder_count": activities["ids"].size(),
		"pose_oracle_checked": false, "debug_build": OS.is_debug_build()}), "\t") + "\n")
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
		row["draw_rank"] = int(_draw_ranks.get(str(sprite.get_path()), -1))
		row["node_path"] = str(sprite.get_path())
		if sprite.is_visible_in_tree() and int(row["draw_rank"]) < 0:
			_errors.append("visible holder missing draw rank: %d" % entity)
		row.merge(_icon_geometry(sprite, int(row["draw_rank"])))
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
	push_error("activity-capture-p12: " + message)
	quit(1)


# Godot serializes PackedByteArray as a string. Copy the evidence tree so
# normalized JSON arrays never mutate cached bridge snapshots or raw census.
func _json_ready(value: Variant) -> Variant:
	if value is PackedByteArray:
		return Array(value)
	if value is Dictionary:
		var result: Dictionary = {}
		for key: Variant in value:
			result[key] = _json_ready(value[key])
		return result
	if value is Array:
		var result: Array = []
		for item: Variant in value:
			result.append(_json_ready(item))
		return result
	return value


func _icon_geometry(sprite: Sprite2D, body_rank: int) -> Dictionary:
	var renderer: Node2D = _main.get_node("AgentRenderer") as Node2D
	var icon: Sprite2D = null
	for child: Node in sprite.get_children():
		if child is Sprite2D and str(child.get_meta("art_kind", "")) == "activity_icon":
			icon = child as Sprite2D
			break
	var shown: bool = icon != null and icon.is_visible_in_tree() and icon.texture != null
	var rect: Rect2
	var rank: int = body_rank
	var source: String = "head_icon_position"
	if shown:
		rect = icon.get_global_transform_with_canvas() * icon.get_rect()
		rank = int(_draw_ranks.get(str(icon.get_path()), -1))
		source = "actual_sprite"
		if rank < 0:
			_errors.append("visible activity icon missing draw rank")
	else:
		var anchor: Vector2 = renderer.call("head_icon_position", sprite.position)
		# Daily icons have a 16x16 footprint, bottom-centred at the head anchor.
		var size: Vector2 = renderer.get_script().get_script_constant_map()["ICON_SIZE_PX"]
		rect = sprite.get_parent().get_global_transform_with_canvas() * Rect2(anchor - Vector2(size.x / 2.0, size.y), size)
	return {"icon_visible": shown, "icon_viewport_rect": [rect.position.x, rect.position.y, rect.size.x, rect.size.y],
		"icon_draw_rank": rank, "icon_rect_source": source,
		"icon_rank_source": "actual_sprite" if shown else "holder_body_rank"}
