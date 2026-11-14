# world3d_gui_test.gd — M3D-1 interaction test of the 3D view with synthetic input (windowed; procedure 9 of
#   worldsim-godot). Loads `--save`, stops the clock and checks, through bridge reads and panel state, never through
#   the 3D layers' own bookkeeping alone:
#   1 a click on a person's body opens the inspector on that person (AgentInspectorPanel.get_selected_entity_bits);
#   2 a click on a workshop's roof opens the workshop panel (the ray stops at the roof, not the ground behind);
#   3 R (room overlay) dissolves every roof, and a click on a room tile then opens the room panel;
#   4 E turns the view 90°, and W then pans "up on screen" (the 2D camera moves along the rotated forward);
#   5 the wheel steps the 3D zoom ladder;
#   6 an F1 chop drag over trees in 3D marks trees (get_colony_overlay designations grow).
#   Saves a frame after each step to `--out`. Prints `W3D_GUI <name> PASS|FAIL <detail>` and `W3D_GUI_DONE failures=<n>`;
#   quits 1 on any failure. `Godot --path . --resolution 1920x1080 --script res://tools/v8/world3d_gui_test.gd --
#   --renderer=3d --save <wsave> --out <dir>`
# Bridge methods called: load_world (direct); via SimReads get_agent_snapshot, get_workshops, get_rooms,
#   get_colony_overlay, get_trees.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")

var _main: Node = null
var _world: Node = null
var _camera: Camera2D = null
var _w3d: Node = null
var _out: String = ""
var _failures: int = 0
var _shot: int = 0


func _init() -> void:
	call_deferred("_run")


func _arg(name: String) -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = args.find(name)
	return args[i + 1] if i >= 0 and i + 1 < args.size() else ""


func _frames(n: int) -> void:
	for _i: int in n:
		await process_frame


func _check(name: String, ok: bool, detail: String) -> void:
	print("W3D_GUI %s %s %s" % [name, "PASS" if ok else "FAIL", detail])
	if not ok:
		_failures += 1


func _save(name: String) -> void:
	await RenderingServer.frame_post_draw
	root.get_viewport().get_texture().get_image().save_png(_out.path_join("%02d_%s.png" % [_shot, name]))
	_shot += 1


func _motion(p: Vector2) -> void:
	var m: InputEventMouseMotion = InputEventMouseMotion.new()
	m.position = p
	m.global_position = p
	Input.parse_input_event(m)
	await process_frame


func _button(p: Vector2, button: MouseButton, pressed: bool) -> void:
	var b: InputEventMouseButton = InputEventMouseButton.new()
	b.position = p
	b.global_position = p
	b.button_index = button
	b.pressed = pressed
	Input.parse_input_event(b)
	await process_frame


func _click(p: Vector2) -> void:
	await _motion(p)
	await _button(p, MOUSE_BUTTON_LEFT, true)
	await _button(p, MOUSE_BUTTON_LEFT, false)
	await _frames(3)


func _key(code: Key, frames_held: int = 1) -> void:
	var e: InputEventKey = InputEventKey.new()
	e.keycode = code
	e.physical_keycode = code
	e.pressed = true
	Input.parse_input_event(e)
	await _frames(frames_held)
	var r: InputEventKey = InputEventKey.new()
	r.keycode = code
	r.physical_keycode = code
	r.pressed = false
	Input.parse_input_event(r)
	await process_frame


## Move the 2D camera so world point `p3` (3D) lands on screen point `s`.
func _bring(p3: Vector3, s: Vector2) -> void:
	var cam: Camera3D = _w3d.call("camera")
	for _i: int in 6:
		_w3d.get_node("CameraRig").call("update_camera")
		var at: Vector2 = cam.unproject_position(p3)
		var rot: float = float(_camera.get("screen_rotation"))
		_camera.position += ((at - s) / _camera.zoom.x).rotated(rot)
		await process_frame
	_w3d.get_node("CameraRig").call("update_camera")
	await _frames(2)


func _free_spot(candidates: Array[Vector2]) -> Vector2:
	for c: Vector2 in candidates:
		await _motion(c)
		if root.gui_get_hovered_control() == null:
			return c
	return candidates[0]


func _run() -> void:
	_out = _arg("--out")
	DirAccess.make_dir_recursive_absolute(_out)
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D")
	await _frames(10)
	_w3d = _main.get_node_or_null("World3D")
	if _w3d == null or not bool(_world.call("load_world", _arg("--save"))):
		push_error("world3d_gui_test: needs --renderer=3d and a loadable --save")
		quit(1)
		return
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	_w3d.get_node("Sky").call("set_hour_override", 10.0)
	_camera.zoom = Vector2(2, 2)
	await _frames(30)
	_w3d.get_node("Terrain").call("build_all")
	await _frames(30)
	var spot: Vector2 = await _free_spot([Vector2(420, 760), Vector2(300, 600), Vector2(1700, 400), Vector2(500, 880)])
	print("W3D_GUI spot %s" % spot)
	# 1 person.
	var agents: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var ids: PackedInt64Array = agents.get("ids", PackedInt64Array())
	var target_bits: int = ids[0] if ids.size() > 0 else -1
	var people: Node = _w3d.get_node("People")
	var feet: Vector3 = people.call("feet_of", target_bits)
	await _bring(feet + Vector3(0, 0.9, 0), spot)
	var layer: Node = _main.get_node("ColonyView/AgentLayer")
	print("W3D_GUI debug feet=%s screen=%s pick=%s rect=%s world=%s" % [feet, (_w3d.call("camera") as Camera3D).unproject_position(feet + Vector3(0, 0.9, 0)),
		people.call("pick", spot, _w3d.call("camera"), 0), layer.call("drawn_rect", target_bits), _w3d.call("screen_to_world", spot)])
	var wpos: Vector2 = _w3d.call("screen_to_world", spot)
	print("W3D_GUI debug entity_at=%s hover_cam=%s" % [layer.call("entity_at", wpos, true), root.gui_get_hovered_control()])
	await _motion(spot)
	print("W3D_GUI debug after_motion hover_world=%s cam=%s pick=%s" % [_main.get_node("ColonyView/OrderTool").get("_hover_world"), _camera.position, people.call("pick", spot, _w3d.call("camera"), 0)])
	await _click(spot)
	var tool0: Node = _main.get_node("ColonyView/OrderTool")
	print("W3D_GUI debug after hover_world=%s hover=%s mode=%s why=%s" % [tool0.get("_hover_world"), tool0.get("_hover"), tool0.get("mode"), tool0.get("why_selecting")])
	var inspector: Node = _main.get_node("UI/AgentInspectorPanel")
	var got: int = int(inspector.call("get_selected_entity_bits"))
	_check("person_click", got == target_bits, "want %d got %d" % [target_bits, got])
	await _frames(10)
	await _save("person_selected")
	await _key(KEY_ESCAPE)
	# 2 workshop roof: the first workshop whose tile lies in a roofed room.
	var shops: Dictionary = SimReads.read(_world, "get_workshops")
	var rooms: Dictionary = SimReads.read(_world, "get_rooms")
	var room_of: Dictionary = {}
	var txs: PackedInt32Array = rooms.get("tile_xs", PackedInt32Array())
	var tys: PackedInt32Array = rooms.get("tile_ys", PackedInt32Array())
	var trs: PackedInt32Array = rooms.get("tile_rooms", PackedInt32Array())
	for i: int in trs.size():
		room_of[Vector2i(txs[i], tys[i])] = trs[i]
	var shop_tile: Vector2i = Vector2i(-1, -1)
	var sx: PackedInt32Array = shops.get("xs", PackedInt32Array())
	var sy: PackedInt32Array = shops.get("ys", PackedInt32Array())
	for i: int in sx.size():
		if room_of.has(Vector2i(sx[i], sy[i])):
			shop_tile = Vector2i(sx[i], sy[i])
			break
	var structures: Node = _w3d.get_node("Structures")
	if shop_tile.x >= 0:
		var top: float = float(_w3d.get_node("Terrain").call("stand_height", shop_tile)) + 2.2
		await _bring(Vector3(shop_tile.x + 0.5, top, shop_tile.y + 0.5), spot)
		var w: Vector2 = _w3d.call("screen_to_world", spot)
		print("W3D_GUI debug roof world=%s tile=%s shop_at=%s" % [w, View.world_to_tile(w), _main.get_node("ColonyView/OrderTool").call("workshop_at", View.world_to_tile(w))])
		await _click(spot)
		var shop_panel: Control = _main.get_node_or_null("UI/WorkshopPanel") as Control
		_check("workshop_roof_click", shop_panel != null and shop_panel.visible, "tile %s roofs %d" % [shop_tile, int(structures.call("roof_count"))])
		await _save("workshop_panel")
		await _key(KEY_ESCAPE)
	else:
		_check("workshop_roof_click", false, "no workshop inside a room")
	# 3 room overlay → roofs dissolve; click a room tile → room panel.
	await _key(KEY_R)
	await _frames(40)
	var faded: bool = true
	for i: int in (rooms.get("ids", PackedInt32Array()) as PackedInt32Array).size():
		var id: int = (rooms["ids"] as PackedInt32Array)[i]
		if float(structures.call("roof_fade", id)) > 0.01:
			faded = false
	_check("room_overlay_cutaway", faded and int(structures.call("roof_count")) > 0, "roofs %d" % int(structures.call("roof_count")))
	if shop_tile.x >= 0:
		var g: float = float(_w3d.get_node("Terrain").call("stand_height", shop_tile))
		await _bring(Vector3(shop_tile.x + 0.5, g, shop_tile.y + 0.5), spot)
		await _click(spot)
		var room_panel: Control = _main.get_node_or_null("UI/RoomPanel") as Control
		_check("room_click", room_panel != null and room_panel.visible, "")
		await _save("room_cutaway")
		await _key(KEY_ESCAPE)
	await _key(KEY_R)
	# 4 rotate and pan.
	await _key(KEY_E)
	await _frames(40)
	var yaw: float = float(_w3d.get_node("CameraRig").call("yaw"))
	_check("rotate_e", absf(yaw - PI * 0.5) < 0.02, "yaw %.3f" % yaw)
	var before: Vector2 = _camera.position
	await _key(KEY_W, 20)
	var moved: Vector2 = _camera.position - before
	_check("pan_rotated", moved.x < -10.0 and absf(moved.y) < absf(moved.x) * 0.2, "moved %s" % moved)
	await _save("rotated")
	# 5 zoom wheel.
	var z0: float = float(_camera.call("get_zoom_level"))
	await _motion(spot)
	await _button(spot, MOUSE_BUTTON_WHEEL_DOWN, true)
	await _button(spot, MOUSE_BUTTON_WHEEL_DOWN, false)
	await _frames(20)
	var z1: float = float(_camera.call("get_zoom_level"))
	_check("zoom_wheel", z1 < z0, "%.2f -> %.2f" % [z0, z1])
	await _key(KEY_Q)
	await _frames(40)
	# 6 chop drag over the nearest trees.
	var trees: Dictionary = SimReads.read(_world, "get_trees")
	var tx: PackedInt32Array = trees.get("xs", PackedInt32Array())
	var ty: PackedInt32Array = trees.get("ys", PackedInt32Array())
	var before_marks: int = (SimReads.read(_world, "get_colony_overlay").get("designation_xs", PackedInt32Array()) as PackedInt32Array).size()
	if tx.size() > 0:
		var centre: Vector2i = Vector2i(tx[0], ty[0])
		var best: int = 1 << 30
		for i: int in tx.size():
			var d: int = absi(tx[i] - 95) + absi(ty[i] - 105)
			if d < best:
				best = d
				centre = Vector2i(tx[i], ty[i])
		var tool: Node = _main.get_node("ColonyView/OrderTool")
		tool.call("set_mode", 1)
		var terrain: Node = _w3d.get_node("Terrain")
		var a3: Vector3 = Vector3(centre.x - 1.5, float(terrain.call("stand_height", centre)), centre.y - 1.5)
		var b3: Vector3 = Vector3(centre.x + 2.5, float(terrain.call("stand_height", centre)), centre.y + 2.5)
		await _bring(Vector3(centre.x + 0.5, a3.y, centre.y + 0.5), spot)
		var cam: Camera3D = _w3d.call("camera")
		var sa: Vector2 = cam.unproject_position(a3)
		var sb: Vector2 = cam.unproject_position(b3)
		await _motion(sa)
		await _button(sa, MOUSE_BUTTON_LEFT, true)
		await _motion((sa + sb) * 0.5)
		await _motion(sb)
		await _save("chop_drag")
		await _button(sb, MOUSE_BUTTON_LEFT, false)
		await _frames(5)
		var after_marks: int = (SimReads.read(_world, "get_colony_overlay").get("designation_xs", PackedInt32Array()) as PackedInt32Array).size()
		_check("chop_drag", after_marks > before_marks, "marks %d -> %d around %s result %s drag %s→%s" % [before_marks, after_marks, centre,
			tool.call("last_result"), tool.get("_drag_from"), tool.get("_drag_to")])
		tool.call("set_mode", 0)
	print("W3D_GUI_DONE failures=%d" % _failures)
	quit(1 if _failures > 0 else 0)
