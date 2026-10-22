# tools/v8/m1_gui_test.gd — M1 GUI test (goal 9.8): the real scene driven by synthetic input events.
# Purpose: with Input.parse_input_event only (button clicks, drags, clicks — never the bridge's command calls):
#   1. 베기: click the bar button, drag over the trees nearest the camp (≥ 15);
#   2. 비축 구역: drag a 4×4 block next to the hearth;
#   3. 오두막: the F3 button opens the build menu (M2), key 1 picks the hut, then click a spot where
#      can_place_blueprint is true (the ghost is green there);
#   4. run 3 game days at 4× speed (live loop, key 4).
#   After each step it asserts the command's effect through the colony reads, and it saves the shots
#   before, after_orders, day1, hut_done and far. With --fps-seconds N it then measures the mean FPS at 4× for
#   N seconds at 2× zoom and at 1× zoom (U7); --no-vsync turns vsync and the frame cap off for that phase, so the
#   number is not clipped at the display's refresh rate (M1-ART 8.5). Prints M1_GUI_PASS / M1_GUI_FAIL lines and a
#   JSON summary.
# Tooling only: the game never runs this script.
# Bridge methods called: get_time_snapshot, get_building_snapshot, get_trees, get_terrain_snapshot,
#   get_colony_overlay, can_place_blueprint, get_tallies (all reads).
# usage: Godot --path . --rendering-driver opengl3 --resolution 1920x1080 \
#          --script res://tools/v8/m1_gui_test.gd -- --out <abs dir> [--days 3] [--fps-seconds 120] [--no-vsync]
extends SceneTree

const TILE: float = 16.0
const DAY: int = 1440

var _out: String = ""
var _days: int = 3
var _fps_seconds: float = 0.0
var _no_vsync: bool = false
var _failures: Array[String] = []
var _summary: Dictionary = {}
var _main: Node
var _world: Node
var _camera: Camera2D


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--out" and i + 1 < args.size():
			_out = args[i + 1]
		elif args[i] == "--days" and i + 1 < args.size():
			_days = int(args[i + 1])
		elif args[i] == "--fps-seconds" and i + 1 < args.size():
			_fps_seconds = float(args[i + 1])
		elif args[i] == "--no-vsync":
			_no_vsync = true
	call_deferred("_run")


func _check(label: String, ok: bool, detail: String = "") -> void:
	print("M1_GUI_%s %s %s" % ["PASS" if ok else "FAIL", label, detail])
	if not ok:
		_failures.append(label + " " + detail)


func _run() -> void:
	if _out.is_empty():
		push_error("m1_gui_test: --out is required")
		quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	for _i: int in 20:
		await process_frame
	_world = _main.get_node_or_null("WorldSim")
	_camera = _main.get_node_or_null("Camera2D") as Camera2D
	if _world == null or _camera == null:
		_check("scene", false, "missing WorldSim or Camera2D")
		_finish()
		return
	# Freeze the clock while ordering (the player may pause with Space since M2; orders apply at once).
	await _key(KEY_SPACE)
	var hearth: Vector2i = _hearth()
	_camera.call("center_on_tile", hearth)
	for _i: int in 10:
		await process_frame
	await _shot("before")
	await _step_chop(hearth)
	await _step_stockpile(hearth)
	await _step_hut(hearth)
	await _shot("after_orders")
	await _step_run(hearth)
	if _fps_seconds > 0.0:
		await _fps(hearth)
	_finish()


func _finish() -> void:
	_summary["failures"] = _failures
	var file: FileAccess = FileAccess.open(_out.path_join("m1_gui_summary.json"), FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify(_summary, "  ", true))
		file.close()
	print("M1_GUI_DONE failures=%d" % _failures.size())
	quit(0 if _failures.is_empty() else 1)


func _tick() -> int:
	return int((_world.call("get_time_snapshot") as Dictionary).get("tick", 0))


func _hearth() -> Vector2i:
	var b: Dictionary = _world.call("get_building_snapshot")
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	var xs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = b.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[i] == 0:
			return Vector2i(xs[i], ys[i])
	return Vector2i(96, 96)


func _screen(tile: Vector2i) -> Vector2:
	return root.get_canvas_transform() * (Vector2(tile) * TILE + Vector2(8, 8))


func _mouse(button: MouseButton, pressed: bool, at: Vector2) -> void:
	var event: InputEventMouseButton = InputEventMouseButton.new()
	event.button_index = button
	event.pressed = pressed
	event.position = at
	event.global_position = at
	Input.parse_input_event(event)
	await process_frame


func _move(at: Vector2) -> void:
	var event: InputEventMouseMotion = InputEventMouseMotion.new()
	event.position = at
	event.global_position = at
	Input.parse_input_event(event)
	await process_frame


func _key(code: Key) -> void:
	for pressed: bool in [true, false]:
		var event: InputEventKey = InputEventKey.new()
		event.keycode = code
		event.physical_keycode = code
		event.pressed = pressed
		Input.parse_input_event(event)
		await process_frame


## Click the order bar button with index `index` (0 = 베기 … 4 = 취소).
func _click_bar(index: int) -> void:
	var bar: Node = _main.get_node("UI/OrderBar")
	var buttons: Array = bar.get("_buttons")
	var button: Button = buttons[index]
	var center: Vector2 = button.get_global_rect().get_center()
	await _move(center)
	await _mouse(MOUSE_BUTTON_LEFT, true, center)
	await _mouse(MOUSE_BUTTON_LEFT, false, center)
	for _i: int in 3:
		await process_frame


func _drag(a: Vector2i, b: Vector2i) -> void:
	await _move(_screen(a))
	await _mouse(MOUSE_BUTTON_LEFT, true, _screen(a))
	await _move(_screen((a + b) / 2))
	await _move(_screen(b))
	await _mouse(MOUSE_BUTTON_LEFT, false, _screen(b))
	for _i: int in 3:
		await process_frame


func _overlay() -> Dictionary:
	return _world.call("get_colony_overlay")


func _last_result() -> Dictionary:
	var tool: Node = _main.get_node("ColonyView/OrderTool")
	return tool.call("last_result")


func _step_chop(hearth: Vector2i) -> void:
	# The smallest square around the hearth whose ring holds ≥ 15 trees (Chebyshev, like the U-script).
	var trees: Dictionary = _world.call("get_trees")
	var txs: PackedInt32Array = trees.get("xs", PackedInt32Array())
	var tys: PackedInt32Array = trees.get("ys", PackedInt32Array())
	var r: int = 1
	var inside: int = 0
	while r < 60:
		inside = 0
		for i: int in txs.size():
			if absi(txs[i] - hearth.x) <= r and absi(tys[i] - hearth.y) <= r:
				inside += 1
		if inside >= 15:
			break
		r += 1
	_camera.call("center_on_tile", hearth)
	while _camera.zoom.x > 1.0 and (r * 2 + 1) * TILE * _camera.zoom.x > 1000.0:
		_camera.call("step_zoom", -1)
	for _i: int in 5:
		await process_frame
	await _click_bar(0)
	_check("chop mode", int(_main.get_node("ColonyView/OrderTool").get("mode")) == 1)
	await _drag(hearth - Vector2i(r, r), hearth + Vector2i(r, r))
	var result: Dictionary = _last_result()
	var marks: int = (_overlay().get("designation_xs", PackedInt32Array()) as PackedInt32Array).size()
	_summary["chop"] = {"radius": r, "trees_in_rect": inside, "result": result, "marked": marks}
	_check("chop accepted", bool(result.get("accepted", false)) and int(result.get("count", 0)) == inside, str(result))
	_check("chop marks", marks == inside and marks >= 15, "marked %d of %d" % [marks, inside])
	await _key(KEY_ESCAPE)


func _zoneable(t: Vector2i, terrain: Dictionary, buildings: Dictionary) -> bool:
	var w: int = int(terrain.get("width", 0))
	var codes: PackedByteArray = terrain.get("terrain", PackedByteArray())
	if t.x < 0 or t.y < 0 or t.x >= w or t.y * w + t.x >= codes.size():
		return false
	var code: int = codes[t.y * w + t.x]
	return code != 2 and code != 4 and not buildings.has(t)


func _step_stockpile(hearth: Vector2i) -> void:
	var terrain: Dictionary = _world.call("get_terrain_snapshot")
	var b: Dictionary = _world.call("get_building_snapshot")
	var buildings: Dictionary = {}
	var bxs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var bys: PackedInt32Array = b.get("ys", PackedInt32Array())
	for i: int in mini(bxs.size(), bys.size()):
		buildings[Vector2i(bxs[i], bys[i])] = true
	var corner: Vector2i = hearth + Vector2i(3, 0)
	var found: bool = false
	for ring: int in 12:
		for dy: int in range(-ring, ring + 1):
			for dx: int in range(-ring, ring + 1):
				if found or maxi(absi(dx), absi(dy)) != ring:
					continue
				var c: Vector2i = hearth + Vector2i(3 + dx, dy)
				var ok: bool = true
				for y: int in 4:
					for x: int in 4:
						ok = ok and _zoneable(c + Vector2i(x, y), terrain, buildings)
				if ok:
					corner = c
					found = true
	await _click_bar(1)
	await _drag(corner, corner + Vector2i(3, 3))
	var result: Dictionary = _last_result()
	var zoned: int = (_overlay().get("zone_xs", PackedInt32Array()) as PackedInt32Array).size()
	_summary["stockpile"] = {"corner": [corner.x, corner.y], "result": result, "zone_tiles": zoned}
	_check("stockpile accepted", bool(result.get("accepted", false)) and int(result.get("count", 0)) == 16, str(result))
	_check("stockpile tiles", zoned == 16, "zone tiles %d" % zoned)
	await _key(KEY_ESCAPE)


func _step_hut(hearth: Vector2i) -> void:
	# M2: the F3 button opens the build menu; 1 picks the hut.
	await _click_bar(2)
	await _key(KEY_1)
	var spot: Vector2i = Vector2i(-1, -1)
	for dy: int in range(2, 30):
		for dx: int in range(-8, 9):
			var top_left: Vector2i = hearth + Vector2i(dx - 1, dy - 1)
			if spot.x < 0 and bool(_world.call("can_place_blueprint", "hut", top_left.x, top_left.y)):
				spot = top_left
	_check("hut spot", spot.x >= 0)
	var centre: Vector2i = spot + Vector2i(1, 1)
	await _move(_screen(centre))
	for _i: int in 3:
		await process_frame
	var tool: Node = _main.get_node("ColonyView/OrderTool")
	_check("ghost green", bool(tool.get("_ghost_ok")))
	await _shot("hut_ghost")
	await _mouse(MOUSE_BUTTON_LEFT, true, _screen(centre))
	await _mouse(MOUSE_BUTTON_LEFT, false, _screen(centre))
	var result: Dictionary = _last_result()
	var sites: PackedInt32Array = _overlay().get("site_xs", PackedInt32Array())
	_summary["hut"] = {"top_left": [spot.x, spot.y], "result": result, "sites": sites.size()}
	_check("hut accepted", bool(result.get("accepted", false)), str(result))
	_check("hut site", sites.size() == 1)
	# A second click on the same spot is refused (the site blocks itself): the ghost turns red.
	await _move(_screen(centre + Vector2i(1, 0)))
	await _move(_screen(centre))
	_check("ghost red on the site", not bool(tool.get("_ghost_ok")))
	await _key(KEY_ESCAPE)


func _step_run(hearth: Vector2i) -> void:
	_camera.call("center_on_tile", hearth)
	_camera.zoom = Vector2(2, 2)
	await _key(KEY_4)
	await _key(KEY_SPACE)
	_check("speed 4x", is_equal_approx(float(_camera.call("get_speed")), 4.0) and not bool(_camera.call("is_paused")))
	var start: int = _tick()
	var day1_done: bool = false
	var hut_tick: int = -1
	var guard: int = Time.get_ticks_msec() + int(float(_days * DAY) / 48.0 * 1000.0 * 2.5) + 30000
	while _tick() - start < _days * DAY and Time.get_ticks_msec() < guard:
		await process_frame
		if not day1_done and _tick() - start >= DAY:
			day1_done = true
			await _shot("day1")
		if hut_tick < 0 and int(_main.get_node("ColonyView/BuildingLayer").call("hut_count")) > 0:
			hut_tick = _tick()
			_camera.call("center_on_tile", hearth + Vector2i(0, 4))
			for _i: int in 5:
				await process_frame
			await _shot("hut_done")
			_camera.call("center_on_tile", hearth)
	var ran: int = _tick() - start
	var overlay: Dictionary = _overlay()
	var tallies: Dictionary = _world.call("get_tallies")
	_summary["run"] = {
		"ticks": ran,
		"hut_tick": hut_tick,
		"marks_left": (overlay.get("designation_xs", PackedInt32Array()) as PackedInt32Array).size(),
		"stacks": (overlay.get("item_xs", PackedInt32Array()) as PackedInt32Array).size(),
		"deaths": int(tallies.get("deaths_starvation", 0)) + int(tallies.get("deaths_dehydration", 0)) + int(tallies.get("deaths_exposure", 0)),
	}
	_check("ran %d days" % _days, ran >= _days * DAY, "ticks %d" % ran)
	_check("hut built", hut_tick >= 0, "hut tick %d" % hut_tick)
	_check("trees felled", int(_summary["run"]["marks_left"]) == 0, "marks left %d" % int(_summary["run"]["marks_left"]))
	_camera.zoom = Vector2(1, 1)
	_camera.call("center_on_tile", Vector2i(96, 96))
	for _i: int in 10:
		await process_frame
	await _shot("far")


func _fps(hearth: Vector2i) -> void:
	var results: Dictionary = {"vsync": not _no_vsync}
	if _no_vsync:
		DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
		Engine.max_fps = 0
	for zoom: float in [2.0, 1.0]:
		_camera.zoom = Vector2.ONE * zoom
		_camera.call("center_on_tile", hearth)
		for _i: int in 30:
			await process_frame
		var started: int = Time.get_ticks_usec()
		var frames: int = 0
		var start_tick: int = _tick()
		while Time.get_ticks_usec() - started < int(_fps_seconds * 1000000.0):
			await process_frame
			frames += 1
		var seconds: float = float(Time.get_ticks_usec() - started) / 1000000.0
		results["zoom_%d" % int(zoom)] = {"mean_fps": float(frames) / seconds, "seconds": seconds, "ticks": _tick() - start_tick}
	_summary["fps_at_4x"] = results
	_check("fps 2x zoom >= 60", float(results["zoom_2"]["mean_fps"]) >= 60.0, str(results["zoom_2"]))


func _shot(name: String) -> void:
	for _i: int in 3:
		await process_frame
	await RenderingServer.frame_post_draw
	var image: Image = root.get_texture().get_image()
	var path: String = _out.path_join("%s.png" % name)
	if image == null or image.save_png(path) != OK:
		push_error("m1_gui_test: could not save " + path)
	else:
		print("m1_gui_test: saved ", path)
