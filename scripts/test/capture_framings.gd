extends SceneTree

const ORIGIN: Vector2 = Vector2(448, 28)
const TILE_SIZE: float = 16.0
const BASELINE: Array[Array] = [[0, "F1"], [0, "F2"], [1440, "F1"], [1440, "F2"]]
const FINAL: Array[Array] = [
	[0, "F1"], [0, "F2"], [1440, "F1"], [1440, "F2"], [1440, "F3"],
	[1440, "F4"], [2760, "F2N"], [7200, "F1"], [7200, "F2"],
]

var _out: String = ""
var _plan: String = ""
var _main: Node = null
var _world: Node = null
var _camera: Camera2D = null
var _manifest: Array[Dictionary] = []


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
	var plan: Array[Array] = BASELINE if _plan == "baseline" else FINAL
	for shot: Array in plan:
		var target: int = int(shot[0])
		var framing: String = str(shot[1])
		var before: Dictionary = _world.call("get_time_snapshot")
		var delta_ticks: int = target - int(before["tick"])
		if delta_ticks < 0 or int(_world.call("advance_ticks", delta_ticks)) != target:
			_fail("simulation did not reach requested tick")
			return
		var tile: Vector2 = Vector2(32, 32) if framing == "F1" else _village_tile()
		if framing == "F3":
			tile = _food_tile(tile)
			if tile.x < 0:
				_fail("F3 requires a food source; none exists at this tick")
				return
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
		for frame: int in range(5):
			await process_frame
		await RenderingServer.frame_post_draw
		var now: Dictionary = _world.call("get_time_snapshot")
		if int(now["tick"]) != target:
			_fail("wall-clock advanced a frozen capture")
			return
		var image: Image = root.get_texture().get_image()
		if image == null or image.get_width() != 1920 or image.get_height() != 1080:
			_fail("capture framebuffer must be 1920x1080; no placeholder is written")
			return
		var filename: String = "%s_t%d_%s.png" % [_plan, target, framing]
		if image.save_png(ProjectSettings.globalize_path(_out.path_join(filename))) != OK:
			_fail("cannot save " + filename)
			return
		_manifest.append({
			"file": filename, "tick": target, "framing": framing,
			"tile_x": tile.x, "tile_y": tile.y, "zoom": _camera.zoom.x,
			"time_of_day": now["time_of_day"], "width": 1920, "height": 1080,
		})
	var manifest: FileAccess = FileAccess.open(_out.path_join("manifest.json"), FileAccess.WRITE)
	if manifest == null:
		_fail("cannot write manifest")
		return
	manifest.store_string(JSON.stringify({
		"schema_version": 1, "plan": _plan, "git_sha": OS.get_environment("WORLDSIM_GIT_SHA"),
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
	detail["entity_bits"] = ids[best]
	inspector.call("display_agent", detail)
	return true


func _fail(message: String) -> void:
	push_error("capture: " + message)
	quit(1)
