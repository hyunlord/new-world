# integ1_review.gd — INTEG1 review videos: JPEG frame sequences of the real game scene for ffmpeg (30 fps).
# Purpose: `--save <wsave> --out <dir> --scene actions|camera|roofs|night [--seconds S] [--width W]`; run windowed
#   (Forward+) with `--fixed-fps 30` so every saved frame is 1/30 s of game time. Scenes:
#   actions — z5 near view that moves (1.5 s pans) to a villager performing a non-idle, non-walk action every 6 s;
#   camera — one smooth zoom z1 → z4 at noon (the per-stage tilt follows the zoom);
#   roofs — z4 on the houses: closed, hover a house (opens after 0.3 s), select it (RoomPanel.select_house), V on, V off;
#   night — z2 at 22:00 with a slow pan;
#   work — queues chop (nearest trees), a hollow log-wall ring, quarry and dig rectangles near the camp through the
#   ordinary cmd_* mutators, lets the colony walk there, then follows working villagers at z5 for `--seconds` (35) and
#   writes `clips.jsonl` (per frame: followed agent, visual key, near clip); `--close chop,build` adds 5 s close shots
#   of one villager per key (z5, `close/`). Prints `INTEG1_REVIEW <json>` per second of actions (visual keys on screen)
#   and `INTEG1_REVIEW_DONE frames=<n>`; any failure quits 1. Moves the real mouse cursor (roofs).
# Bridge methods called: load_world (direct); via SimReads get_agent_snapshot, get_visible_action_snapshot, get_trees,
#   get_terrain_snapshot; mutators advance_ticks, cmd_designate_chop, cmd_place_rect, cmd_designate_quarry, cmd_designate_dig.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const BORING: Array[String] = ["idle", "walk", "sleep", "run", ""]

var _main: Node = null
var _world: Node = null
var _camera: Camera2D = null
var _w3d: Node = null
var _out: String = ""
var _width: int = 1280
var _frame: int = 0


func _init() -> void:
	call_deferred("_run")


func _arg(name: String, fallback: String = "") -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = args.find(name)
	return args[i + 1] if i >= 0 and i + 1 < args.size() else fallback


func _fail(message: String) -> void:
	push_error("integ1_review: " + message)
	quit(1)


func _run() -> void:
	_out = _arg("--out")
	var save: String = _arg("--save")
	var scene: String = _arg("--scene")
	if _out == "" or save == "" or scene == "":
		_fail("--out, --save and --scene are required")
		return
	_width = int(_arg("--width", "1280"))
	DirAccess.make_dir_recursive_absolute(_out)
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D")
	for _i: int in 10:
		await process_frame
	if not bool(_world.call("load_world", save)):
		_fail("load_world failed: " + save)
		return
	for _i: int in 30:
		await process_frame
	_w3d = _main.get_node_or_null("World3D")
	if _w3d == null:
		_fail("no World3D")
		return
	# Review frames compare the grade itself; the settlement hardship tint is pinned to 0 (debug builds).
	_w3d.get_node("Sky").call("set_hardship_override", float(_arg("--hardship", "0")))
	var seconds: float = float(_arg("--seconds", "0"))
	match scene:
		"actions":
			await _actions(seconds if seconds > 0.0 else 25.0)
		"camera":
			await _zoom_sweep(seconds if seconds > 0.0 else 18.0)
		"roofs":
			await _roofs()
		"night":
			await _night(seconds if seconds > 0.0 else 10.0)
		"work":
			await _work(seconds if seconds > 0.0 else 35.0)
		"probe":
			_orders(Vector2i(_camp()))
			for step: int in 30:
				SimReads.mutate(_world, "advance_ticks", [200])
				await process_frame
				var counts: Dictionary = {}
				var rows: Dictionary = _rows()
				for aid: int in rows:
					counts[rows[aid]["key"]] = int(counts.get(rows[aid]["key"], 0)) + 1
				print("INTEG1_PROBE ", (step + 1) * 200, " ", JSON.stringify(counts))
		_:
			_fail("unknown scene " + scene)
			return
	print("INTEG1_REVIEW_DONE frames=%d" % _frame)
	quit(0)


func _shoot() -> void:
	await RenderingServer.frame_post_draw
	var image: Image = root.get_viewport().get_texture().get_image()
	if image.get_width() != _width:
		image.resize(_width, int(round(float(image.get_height()) * _width / image.get_width())), Image.INTERPOLATE_BILINEAR)
	image.save_jpg(_out.path_join("f_%05d.jpg" % _frame), 0.92)
	_frame += 1


func _look(tile: Vector2, zoom: float) -> void:
	_camera.zoom = Vector2(zoom, zoom)
	_camera.position = tile * float(View.TILE)


func _camp() -> Vector2:
	var hearth: Variant = _arg("--tile", "")
	if str(hearth) != "":
		var p: PackedStringArray = str(hearth).split(",")
		return Vector2(float(p[0]), float(p[1]))
	return _camera.position / float(View.TILE)


## Villagers performing a shown action, nearest to `near` first: [{tile, key}].
func _workers(near: Vector2) -> Array[Dictionary]:
	var acts: Dictionary = SimReads.read(_world, "get_visible_action_snapshot")
	var agents: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var at: Dictionary = {}
	var ids: PackedInt64Array = agents.get("agent_ids", PackedInt64Array())
	for i: int in ids.size():
		at[ids[i]] = Vector2((agents["xs"] as PackedInt32Array)[i] + 0.5, (agents["ys"] as PackedInt32Array)[i] + 0.5)
	var out: Array[Dictionary] = []
	var keys: PackedStringArray = acts.get("visual_keys", PackedStringArray())
	var aids: PackedInt64Array = acts.get("agent_ids", PackedInt64Array())
	for i: int in mini(keys.size(), aids.size()):
		if not BORING.has(keys[i]) and at.has(aids[i]):
			out.append({"tile": at[aids[i]], "key": keys[i]})
	out.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return (a["tile"] as Vector2).distance_to(near) < (b["tile"] as Vector2).distance_to(near))
	return out


func _actions(seconds: float) -> void:
	(_main.get_node("UI") as CanvasLayer).visible = false
	var focus: Vector2 = _camp()
	_look(focus, 4.0)
	var frames: int = int(seconds * 30.0)
	var seen: Dictionary = {}
	var target: Vector2 = focus
	var from: Vector2 = focus
	var pan: int = 45
	for f: int in frames:
		if f % 180 == 0:
			var workers: Array[Dictionary] = _workers(focus)
			# Prefer an action not shown yet; keep the camera if nobody works.
			for w: Dictionary in workers:
				if not seen.has(w["key"]):
					target = w["tile"]
					break
			from = focus
			pan = 0
		if pan < 45:
			pan += 1
			var e: float = float(pan) / 45.0
			focus = from.lerp(target, e * e * (3.0 - 2.0 * e))
			_look(focus, 4.0)
		if f % 30 == 0:
			var shown: Array = []
			for w: Dictionary in _workers(focus):
				if (w["tile"] as Vector2).distance_to(focus) < 6.0:
					shown.append(w["key"])
					seen[w["key"]] = true
			print("INTEG1_REVIEW " + JSON.stringify({"second": floori(f / 30.0), "focus": [focus.x, focus.y], "near_keys": shown}))
		await _shoot()
	print("INTEG1_REVIEW_ACTIONS " + JSON.stringify(seen.keys()))


func _zoom_sweep(seconds: float) -> void:
	(_main.get_node("UI") as CanvasLayer).visible = false
	_w3d.get_node("Sky").call("set_hour_override", 12.0)
	var focus: Vector2 = _camp()
	var frames: int = int(seconds * 30.0)
	for f: int in frames:
		var t: float = clampf((float(f) / frames - 0.08) / 0.84, 0.0, 1.0)
		var e: float = t * t * (3.0 - 2.0 * t)
		# z1 0.25 → z4 2.0, even in log zoom.
		_look(focus, pow(2.0, lerpf(-2.0, 1.0, e)))
		await _shoot()


func _night(seconds: float) -> void:
	(_main.get_node("UI") as CanvasLayer).visible = false
	_w3d.get_node("Sky").call("set_hour_override", 22.0)
	var focus: Vector2 = _camp()
	var frames: int = int(seconds * 30.0)
	for f: int in frames:
		_look(focus + Vector2(lerpf(-4.0, 4.0, float(f) / frames), 0.0), 0.5)
		await _shoot()


func _screen_of(tile: Vector2, height: float) -> Vector2:
	var cam: Camera3D = _w3d.get("rig").get("camera")
	var ground: float = float(_w3d.get_node("Terrain").call("height_at", tile.x, tile.y))
	return cam.unproject_position(Vector3(tile.x, ground + height, tile.y))


func _hold(frames: int) -> void:
	for _i: int in frames:
		await _shoot()


func _key(code: Key) -> void:
	for pressed: bool in [true, false]:
		var key: InputEventKey = InputEventKey.new()
		key.keycode = code
		key.physical_keycode = code
		key.pressed = pressed
		Input.parse_input_event(key)
		await _shoot()


func _move_mouse(screen: Vector2) -> void:
	Input.warp_mouse(screen)
	var motion: InputEventMouseMotion = InputEventMouseMotion.new()
	motion.position = screen
	motion.global_position = screen
	Input.parse_input_event(motion)


func _roofs() -> void:
	_w3d.get_node("Sky").call("set_hour_override", 12.0)
	var focus: Vector2 = _camp()
	_look(focus, 2.0)
	var structures: Node = _w3d.get_node("Structures")
	var house: int = -1
	var house_tile: Vector2 = focus
	var best: float = INF
	for dy: int in range(-4, 5):
		for dx: int in range(-4, 5):
			var t: Vector2i = Vector2i(int(focus.x) + dx, int(focus.y) + dy)
			var key: int = int(structures.call("roof_key_at", t))
			if key >= 0 and Vector2(dx, dy).length() < best:
				best = Vector2(dx, dy).length()
				house = key
				house_tile = Vector2(t) + Vector2(0.5, 0.5)
	if house < 0:
		_fail("no roofed house within 4 tiles of the focus")
		return
	var empty: Vector2 = Vector2(root.get_viewport().get_visible_rect().size.x * 0.08, root.get_viewport().get_visible_rect().size.y * 0.3)
	_move_mouse(empty)
	await _hold(60)
	print("INTEG1_REVIEW " + JSON.stringify({"step": "closed", "fade": structures.call("roof_fade", house)}))
	_move_mouse(_screen_of(house_tile, 0.0))
	await _hold(75)
	var hovered: Control = root.get_viewport().gui_get_hovered_control()
	print("INTEG1_REVIEW " + JSON.stringify({"step": "hover", "fade": structures.call("roof_fade", house), "mouse": str(root.get_viewport().get_mouse_position()),
		"aim": str(_screen_of(house_tile, 0.0)), "gui": str(hovered.get_path()) if hovered != null else "", "world": str(_w3d.call("screen_to_world", root.get_viewport().get_mouse_position()))}))
	# Select the house the pointer opened, so hover and selection show the same roof.
	if int(structures.get("_hover_house")) >= 0:
		house = int(structures.get("_hover_house"))
	var room_panel: Node = _main.get_node("UI/RoomPanel")
	room_panel.call("select_house", house)
	_move_mouse(empty)
	await _hold(90)
	print("INTEG1_REVIEW " + JSON.stringify({"step": "selected", "house": room_panel.call("selected_house"), "fade": structures.call("roof_fade", house)}))
	(room_panel as Control).visible = false
	await _hold(30)
	await _key(KEY_V)
	await _hold(75)
	print("INTEG1_REVIEW " + JSON.stringify({"step": "inside_view", "fade": structures.call("roof_fade", house)}))
	await _key(KEY_V)
	await _hold(60)
	print("INTEG1_REVIEW " + JSON.stringify({"step": "closed_again", "fade": structures.call("roof_fade", house)}))


const SHOWN: Array[String] = ["chop", "build", "dig", "mine", "cook", "carry", "pick_up", "set_down", "talk", "harvest", "craft"]


## AgentId → {tile, key, bits} for every villager with an action row.
func _rows() -> Dictionary:
	var acts: Dictionary = SimReads.read(_world, "get_visible_action_snapshot")
	var agents: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var at: Dictionary = {}
	var aids: PackedInt64Array = agents.get("agent_ids", PackedInt64Array())
	for i: int in aids.size():
		at[aids[i]] = i
	var out: Dictionary = {}
	var keys: PackedStringArray = acts.get("visual_keys", PackedStringArray())
	var ids: PackedInt64Array = acts.get("agent_ids", PackedInt64Array())
	for k: int in mini(keys.size(), ids.size()):
		if at.has(ids[k]):
			var i: int = at[ids[k]]
			out[ids[k]] = {"tile": Vector2((agents["xs"] as PackedInt32Array)[i] + 0.5, (agents["ys"] as PackedInt32Array)[i] + 0.5),
				"key": keys[k], "bits": (agents["ids"] as PackedInt64Array)[i]}
	return out


func _orders(camp: Vector2i) -> void:
	var trees: Dictionary = SimReads.read(_world, "get_trees")
	var near: Array[Vector2i] = []
	for i: int in (trees.get("xs", PackedInt32Array()) as PackedInt32Array).size():
		near.append(Vector2i(trees["xs"][i], trees["ys"][i]))
	near.sort_custom(func(a: Vector2i, b: Vector2i) -> bool: return a.distance_squared_to(camp) < b.distance_squared_to(camp))
	for tree: Vector2i in near.slice(0, 8):
		print("INTEG1_ORDER chop ", JSON.stringify(SimReads.mutate(_world, "cmd_designate_chop", [tree.x, tree.y, tree.x, tree.y])))
	var ring: Vector2i = camp + Vector2i(-5, 3)
	print("INTEG1_ORDER wall ", JSON.stringify(SimReads.mutate(_world, "cmd_place_rect", ["log_wall", ring.x, ring.y, ring.x + 3, ring.y + 3, true])))
	print("INTEG1_ORDER floor ", JSON.stringify(SimReads.mutate(_world, "cmd_place_rect", ["packed_floor", camp.x + 3, camp.y - 6, camp.x + 6, camp.y - 4, false])))
	print("INTEG1_ORDER quarry ", JSON.stringify(SimReads.mutate(_world, "cmd_designate_quarry", [camp.x - 12, camp.y - 12, camp.x + 12, camp.y + 12])))
	var terrain: Dictionary = SimReads.read(_world, "get_terrain_snapshot")
	var w: int = int(terrain.get("width", 0))
	var codes: PackedByteArray = terrain.get("terrain", PackedByteArray())
	var hill: Vector2i = Vector2i(-1, -1)
	for i: int in codes.size():
		if codes[i] == 3:
			var t: Vector2i = Vector2i(i % w, floori(float(i) / w))
			if hill.x < 0 or t.distance_squared_to(camp) < hill.distance_squared_to(camp):
				hill = t
	if hill.x >= 0:
		print("INTEG1_ORDER dig ", hill, " ", JSON.stringify(SimReads.mutate(_world, "cmd_designate_dig", [hill.x - 1, hill.y - 1, hill.x + 1, hill.y + 1])))


func _work(_seconds: float) -> void:
	(_main.get_node("UI") as CanvasLayer).visible = false
	_w3d.get_node("Sky").call("set_hour_override", 12.0)
	var camp: Vector2 = _camp()
	_orders(Vector2i(camp))
	var people: Node = _w3d.get_node("People")
	var log_file: FileAccess = FileAccess.open(_out.path_join("clips.jsonl"), FileAccess.WRITE)
	var seen: Dictionary = {}
	var focus: Vector2 = camp
	var main_frames: int = 0
	# `--segments lead:seconds,…`: advance `lead` ticks (villagers take the orders), then follow for `seconds`.
	for segment: String in _arg("--segments", "150:18,1400:17").split(",", false):
		var parts: PackedStringArray = segment.split(":")
		SimReads.mutate(_world, "advance_ticks", [int(parts[0])])
		for _i: int in 20:
			await process_frame
		var main_out: String = _out
		_frame = main_frames
		await _close_shots(seen)
		_out = main_out
		_frame = main_frames
		var followed: int = -1
		_look(focus, 4.0)
		for f: int in int(float(parts[1]) * 30.0):
			var rows: Dictionary = _rows()
			var row: Dictionary = rows.get(followed, {})
			# Every 4 s (or when the followed villager stops working) take the nearest villager doing an unseen action.
			if f % 120 == 0 or row.is_empty() or not SHOWN.has(str(row.get("key", ""))):
				var best: float = INF
				for aid: int in rows:
					var r: Dictionary = rows[aid]
					if SHOWN.has(r["key"]):
						var d: float = (r["tile"] as Vector2).distance_to(focus) + float(seen.get(r["key"], 0)) * 0.2
						if d < best:
							best = d
							followed = aid
				row = rows.get(followed, {})
			if not row.is_empty():
				focus = focus.lerp(row["tile"], 0.06)
				_look(focus, 4.0)
			var clip: String = str((people.call("state_of", int(row.get("bits", -1))) as Dictionary).get("near_clip", "")) if not row.is_empty() else ""
			if not row.is_empty() and clip != "":
				seen[row["key"]] = int(seen.get(row["key"], 0)) + 1
			log_file.store_line(JSON.stringify({"frame": _frame, "agent": followed, "visual_key": row.get("key", ""), "clip": clip}))
			await _shoot()
		main_frames = _frame
	log_file.close()
	print("INTEG1_REVIEW_WORK " + JSON.stringify(seen))

## 5 s close shots (z5) of one villager per `--close` key, before the follow cam; frames in `close/<key>/`.
func _close_shots(seen: Dictionary) -> void:
	var base: String = _out
	for key: String in _arg("--close", "").split(",", false):
		if seen.has("close_" + key):
			continue
		var target: int = -1
		var rows: Dictionary = _rows()
		var people: Node = _w3d.get_node("People")
		# A villager whose near clip already plays the action (not still walking to it).
		for _wait: int in 240:
			for aid: int in rows:
				if rows[aid]["key"] == key and str((people.call("state_of", int(rows[aid]["bits"])) as Dictionary).get("near_clip", "")) == key:
					target = aid
					break
			if target >= 0:
				break
			# Near clips play only near the camera: look at the first villager with the key while waiting.
			for aid: int in rows:
				if rows[aid]["key"] == key:
					_look(rows[aid]["tile"], 4.0)
					break
			await process_frame
			rows = _rows()
		print("INTEG1_REVIEW_CLOSE %s agent %d" % [key, target])
		if target < 0:
			continue
		seen["close_" + key] = true
		_out = base.path_join("close").path_join(key)
		_frame = 0
		DirAccess.make_dir_recursive_absolute(_out)
		for _f: int in 150:
			var r: Dictionary = _rows().get(target, {})
			if not r.is_empty():
				_look(r["tile"], float(_arg("--close-zoom", "3")))
			await _shoot()
	_out = base
