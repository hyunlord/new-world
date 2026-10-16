extends SceneTree

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
	if _out.is_empty() or _plan not in ["baseline", "final"]:
		push_error("capture: requires --out DIRECTORY --plan baseline|final")
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
	var plan: Array[Array] = BASELINE if _plan == "baseline" else FINAL
	for shot: Array in plan:
		var target: int = int(shot[0])
		var framing: String = str(shot[1])
		var stem: String = "%s_t%d_%s" % [_plan, target, framing]
		if _plan == "final":
			var clock: PackedStringArray = str(shot[1]).split(":")
			target = (int(shot[0]) - 1) * 1440 + int(clock[0]) * 60 + int(clock[1]) - _clock_offset
			framing = str(shot[2])
			stem = "d%d_%s%s_%s" % [int(shot[0]), clock[0], clock[1], framing]
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
		_camera.zoom = Vector2.ONE if framing == "F1" else Vector2(4, 4)
		_camera.reset_smoothing()
		_camera.force_update_scroll()
		# Reconcile infrequent object renderers after synchronous tooling ticks.
		for node_name: String in ["TerrainRenderer", "BuildingRenderer"]:
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
	var manifest: FileAccess = FileAccess.open(_out.path_join("manifest.json"), FileAccess.WRITE)
	if manifest == null:
		_fail("cannot write manifest")
		return
	manifest.store_string(JSON.stringify({
		"schema_version": 2, "plan": _plan, "git_sha": OS.get_environment("WORLDSIM_GIT_SHA"),
		"captures": _manifest,
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
	for node_name: String in ["AgentRenderer", "SeekVizRenderer", "NeedBarRenderer", "ActivityTrailRenderer", "DeathVizRenderer"]:
		var layer: CanvasItem = _main.get_node_or_null(node_name) as CanvasItem
		if layer != null:
			layer.visible = shown


func _screen_rect(local: Rect2, transform: Transform2D, pixel_scale: Vector2) -> Rect2:
	var transformed: Rect2 = transform * local
	return Rect2(transformed.position * pixel_scale, transformed.size * pixel_scale)


func _entity(kind: String, id: int, rect: Rect2, shown: bool, viewport: Vector2) -> Dictionary:
	return {"kind": kind, "id": id, "rect": [rect.position.x, rect.position.y, rect.size.x, rect.size.y],
		"visible": shown and rect.intersects(Rect2(Vector2.ZERO, viewport))}


func _node_bounds(node: Node, pixel_scale: Vector2) -> Rect2:
	var bounds: Rect2 = Rect2()
	if node is Sprite2D:
		var sprite: Sprite2D = node as Sprite2D
		return _screen_rect(sprite.get_rect(), sprite.get_global_transform_with_canvas(), pixel_scale)
	if node is Control:
		var control: Control = node as Control
		if control.size.x > 0 and control.size.y > 0:
			bounds = _screen_rect(Rect2(Vector2.ZERO, control.size), control.get_global_transform_with_canvas(), pixel_scale)
	for child: Node in node.get_children():
		if child is CanvasItem and not (child as CanvasItem).is_visible_in_tree():
			continue
		var child_rect: Rect2 = _node_bounds(child, pixel_scale)
		if child_rect.has_area():
			bounds = bounds.merge(child_rect) if bounds.has_area() else child_rect
	return bounds


func _write_entities(stem: String, tick: int, image_size: Vector2i) -> bool:
	var viewport: Vector2 = Vector2(image_size)
	var pixel_scale: Vector2 = viewport / root.get_visible_rect().size
	var rows: Array[Dictionary] = []
	var agents: Node2D = _main.get_node("AgentRenderer") as Node2D
	var mesh_node: MultiMeshInstance2D = agents.get("multi_mesh_inst") as MultiMeshInstance2D
	var mesh: MultiMesh = agents.get("multi_mesh") as MultiMesh
	var snap: Dictionary = _world.call("get_agent_snapshot")
	var agent_ids: PackedInt64Array = snap["agent_ids"]
	var entity_ids: PackedInt64Array = snap["ids"]
	var mesh_bounds: AABB = mesh.mesh.get_aabb()
	var local: Rect2 = Rect2(Vector2(mesh_bounds.position.x, mesh_bounds.position.y), Vector2(mesh_bounds.size.x, mesh_bounds.size.y))
	for i: int in range(mesh.instance_count):
		var transform: Transform2D = mesh_node.get_global_transform_with_canvas() * mesh.get_instance_transform_2d(i)
		var row: Dictionary = _entity("agent", agent_ids[i], _screen_rect(local, transform, pixel_scale), mesh_node.is_visible_in_tree(), viewport)
		row["entity_bits"] = entity_ids[i]
		rows.append(row)
	var buildings: Node = _main.get_node("BuildingRenderer")
	var building_snap: Dictionary = _world.call("get_building_snapshot")
	var building_ids: PackedInt64Array = building_snap["ids"]
	var kinds: PackedByteArray = building_snap["kinds"]
	var building_kinds: Dictionary = {}
	for i: int in building_ids.size():
		building_kinds[building_ids[i]] = int(kinds[i])
	for pair: Array in [["building", "_buildings"], ["site", "_sites"], ["bush", "_bushes"], ["stockpile_label", "_labels"]]:
		var nodes: Dictionary = buildings.get(str(pair[1]))
		var ids: Array = nodes.keys()
		ids.sort()
		for id: int in ids:
			var item: CanvasItem = nodes[id] as CanvasItem
			var row: Dictionary = _entity(str(pair[0]), id, _node_bounds(item, pixel_scale), item.is_visible_in_tree(), viewport)
			if str(pair[0]) == "building":
				row["building_kind"] = int(building_kinds.get(id, -1))
			rows.append(row)
	for child: Node in _main.get_node("UI").get_children():
		if child is Control and (child as Control).is_visible_in_tree():
			var row: Dictionary = _entity("panel", child.get_instance_id(), _node_bounds(child, pixel_scale), true, viewport)
			row["name"] = str(child.name)
			rows.append(row)
	var inspector: Node = _main.get_node("UI/AgentInspectorPanel")
	var data: Dictionary = {
		"schema_version": 1, "camera_position": [_camera.position.x, _camera.position.y],
		"zoom": [_camera.zoom.x, _camera.zoom.y], "viewport_size": [image_size.x, image_size.y],
		"clock_offset_ticks": _clock_offset, "tick": tick, "entities": rows,
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
