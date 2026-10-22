# tools/v8/m3_ui_test.gd — headless check of the M3 UI paths (goal §9), asserted through the bridge reads.
# Purpose: boot res://scenes/main.tscn with the clock paused (Space); chop marks around the camp, a stockpile and
#   quarry marks on the 8 nearest boulders (F7); then as a player would: F3 opens the build menu (9 entries), 6 picks
#   the door and a click places it in the south wall of a 7×5 house; F3 5 picks the log wall (its drag hint), F3 9
#   the stone wall, and a Shift+drag places the hollow rectangle — the door tile is refused, so the toast names one
#   refused tile (synthetic mouse events are NOT_OBSERVABLE headless: the headless window does not map them onto the
#   1920×1080 canvas, so the same drag also goes through OrderTool.place_drag); F6 cancels the door site and a
#   one-tile stone line closes the ring (the walls are stone because the live scene's colony does not chop, see
#   /Users/rexxa/work/m3-data/ui/UI_REPORT.md); F3 8 places two bedrolls, F3 7 a packed-floor line; the line /
#   rectangle previews have the expected tile counts. advance_ticks until the house is a room; R shows the room
#   overlay (the order bar's toggle follows); a click on an empty room tile opens the room panel, whose dropdown sets
#   the role to storehouse and back to auto (checked in get_rooms); the inspector's bed line, the hover's room
#   suffix, F10 deconstruct on one wall, F8 dig on the nearest diggable hill tile, the chronicle's RoomFormed text,
#   the priority table's seven columns and the order bar's fit on the 1920×1080 screen.
# Bridge methods called: via SimReads get_building_snapshot, get_colony_overlay, get_rooms, get_space,
#   get_agent_snapshot, get_terrain_snapshot, get_work_priorities, advance_ticks; get_place_preview, get_room_detail,
#   get_agent_economy, get_agent_detail (direct); and the UI's own calls.
# Run: bash tools/v8/run_bounded.sh 900 "$GODOT_BIN" --headless --path . --script res://tools/v8/m3_ui_test.gd
# Output: one "M3_UI <PASS|FAIL|NOT_OBSERVABLE|INFO> <step> <detail>" line per check, then "M3_UI_DONE failures=N";
#   quits 0 / 1.
extends SceneTree

const RoomLayer = preload("res://scripts/world/colony/room_layer.gd")
const OverlayLayer = preload("res://scripts/world/colony/overlay_layer.gd")
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const Inspector = preload("res://scripts/ui/panels/agent_inspector_panel.gd")
const MAX_TICKS: int = 20 * 1440
const STEP_TICKS: int = 720

var _failures: Array[String] = []
var _main: Node
var _world: Node


func _init() -> void:
	call_deferred("_run")


func _check(step: String, ok: bool, detail: String = "") -> void:
	print("M3_UI %s %s %s" % ["PASS" if ok else "FAIL", step, detail])
	if not ok:
		_failures.append(step)


func _frames(n: int) -> void:
	for _i: int in n:
		await process_frame


func _key(code: Key) -> void:
	for pressed: bool in [true, false]:
		var event: InputEventKey = InputEventKey.new()
		event.keycode = code
		event.physical_keycode = code
		event.pressed = pressed
		Input.parse_input_event(event)
		await process_frame


func _screen(tile: Vector2i) -> Vector2:
	return _main.get_viewport().get_canvas_transform() * (Vector2(tile) * 16.0 + Vector2(8, 8))


func _mouse(tile: Vector2i, button: bool, pressed: bool, shift: bool) -> void:
	var at: Vector2 = _screen(tile)
	if button:
		var click: InputEventMouseButton = InputEventMouseButton.new()
		click.button_index = MOUSE_BUTTON_LEFT
		click.pressed = pressed
		click.position = at
		click.global_position = at
		click.shift_pressed = shift
		Input.parse_input_event(click)
	else:
		var motion: InputEventMouseMotion = InputEventMouseMotion.new()
		motion.position = at
		motion.global_position = at
		motion.shift_pressed = shift
		Input.parse_input_event(motion)
	await process_frame


## A left drag from `a` to `b` with Shift held or not.
func _drag(a: Vector2i, b: Vector2i, shift: bool) -> void:
	await _mouse(a, false, false, shift)
	await _mouse(a, true, true, shift)
	await _mouse((a + b) / 2, false, false, shift)
	await _mouse(b, false, false, shift)
	await _mouse(b, true, false, shift)
	await _frames(2)


func _hearth() -> Vector2i:
	var b: Dictionary = SimReads.read(_world, "get_building_snapshot")
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	var xs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = b.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[i] == 0:
			return Vector2i(xs[i], ys[i])
	return Vector2i(96, 96)


func _site_kinds() -> Dictionary:
	var overlay: Dictionary = SimReads.read(_world, "get_colony_overlay")
	var counts: Dictionary = {}
	for kind: int in overlay.get("site_kinds", PackedInt32Array()):
		counts[kind] = int(counts.get(kind, 0)) + 1
	return counts


func _designations(kind: int) -> int:
	var n: int = 0
	for k: int in SimReads.read(_world, "get_colony_overlay").get("designation_kinds", PackedInt32Array()):
		if k == kind:
			n += 1
	return n


func _all_ok(preview: Dictionary) -> bool:
	var ok: PackedByteArray = preview.get("ok", PackedByteArray())
	if ok.is_empty():
		return false
	for flag: int in ok:
		if flag == 0:
			return false
	return true


func _pick_menu(key: Key, bar: Node) -> void:
	await _key(KEY_F3)
	_check("menu_open_%d" % (int(key) - int(KEY_0)), bool(bar.call("build_menu_open")))
	await _key(key)


func _run() -> void:
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	await _frames(20)
	_world = _main.get_node_or_null("WorldSim")
	var tool: Node = _main.get_node_or_null("ColonyView/OrderTool")
	var bar: Node = _main.get_node_or_null("UI/OrderBar")
	var overlay: Node = _main.get_node_or_null("ColonyView/OverlayLayer")
	var buildings: Node = _main.get_node_or_null("ColonyView/BuildingLayer")
	var inspector: Node = _main.get_node_or_null("UI/AgentInspectorPanel")
	var room_panel: Node = _main.get_node_or_null("UI/RoomPanel")
	var rooms: Node = _main.get_node_or_null("ColonyView/OverlayLayer/Rooms")
	if _world == null or tool == null or bar == null or overlay == null or buildings == null or inspector == null or room_panel == null or rooms == null:
		_check("scene", false, "missing nodes (room panel %s, rooms %s)" % [room_panel != null, rooms != null])
		_finish()
		return
	var locale: Node = root.get_node_or_null("Locale")
	await _key(KEY_SPACE)
	var hearth: Vector2i = _hearth()
	# Logs for the walls: chop the trees around the camp.
	tool.call("set_mode", 1)
	var chop: Dictionary = tool.call("area", hearth - Vector2i(14, 14), hearth + Vector2i(14, 14))
	_check("chop_marks", bool(chop.get("accepted", false)), str(chop))
	# A stockpile for the logs (as the M3-U script), the first offset that is accepted.
	tool.call("set_mode", 2)
	var stock: Dictionary = {}
	for offset: Vector2i in [Vector2i(3, -3), Vector2i(-9, -3), Vector2i(3, -10), Vector2i(-9, -10)]:
		stock = tool.call("area", hearth + offset, hearth + offset + Vector2i(5, 5))
		if bool(stock.get("accepted", false)):
			break
	_check("stockpile", bool(stock.get("accepted", false)), str(stock))
	# Stone for the house's stone walls: quarry the 8 boulders nearest the camp (F7).
	var ov0: Dictionary = SimReads.read(_world, "get_colony_overlay")
	var nxs: PackedInt32Array = ov0.get("node_xs", PackedInt32Array())
	var nys: PackedInt32Array = ov0.get("node_ys", PackedInt32Array())
	var nkinds: PackedInt32Array = ov0.get("node_kinds", PackedInt32Array())
	var boulders: Array[Vector2i] = []
	for i: int in mini(nxs.size(), mini(nys.size(), nkinds.size())):
		if nkinds[i] == 0:
			boulders.append(Vector2i(nxs[i], nys[i]))
	boulders.sort_custom(func(p: Vector2i, q: Vector2i) -> bool: return (p - hearth).length_squared() < (q - hearth).length_squared())
	tool.call("set_mode", 6)
	var quarried: int = 0
	for i: int in mini(8, boulders.size()):
		var q: Dictionary = tool.call("area", boulders[i], boulders[i])
		quarried += int(q.get("count", 0))
	_check("quarry_marks", quarried >= 4, "marked %d" % quarried)
	tool.call("set_mode", 0)
	# A 7×5 house whose whole border fits (preview shape 2 all ok).
	var a: Vector2i = Vector2i(-1, -1)
	for off: Vector2i in [Vector2i(-3, 4), Vector2i(-3, -9), Vector2i(4, -2), Vector2i(-11, -2), Vector2i(-3, 7), Vector2i(5, 5), Vector2i(-12, 5), Vector2i(5, -10)]:
		var p: Dictionary = _world.call("get_place_preview", "log_wall", hearth.x + off.x, hearth.y + off.y, hearth.x + off.x + 6, hearth.y + off.y + 4, 2)
		var inner: Dictionary = _world.call("get_place_preview", "packed_floor", hearth.x + off.x + 1, hearth.y + off.y + 1, hearth.x + off.x + 5, hearth.y + off.y + 3, 3)
		if _all_ok(p) and _all_ok(inner):
			a = hearth + off
			break
	_check("house_site", a.x >= 0, "origin %s hearth %s" % [a, hearth])
	if a.x < 0:
		_finish()
		return
	var b: Vector2i = a + Vector2i(6, 4)
	var door: Vector2i = a + Vector2i(3, 4)
	# Preview shapes: line 7, hollow 20, filled 35.
	var line_p: Dictionary = _world.call("get_place_preview", "log_wall", a.x, a.y, b.x, a.y, 1)
	var hollow_p: Dictionary = _world.call("get_place_preview", "log_wall", a.x, a.y, b.x, b.y, 2)
	var filled_p: Dictionary = _world.call("get_place_preview", "packed_floor", a.x, a.y, b.x, b.y, 3)
	_check("preview_counts", (line_p["xs"] as PackedInt32Array).size() == 7 and (hollow_p["xs"] as PackedInt32Array).size() == 20 and (filled_p["xs"] as PackedInt32Array).size() == 35,
		"line %d hollow %d filled %d" % [(line_p["xs"] as PackedInt32Array).size(), (hollow_p["xs"] as PackedInt32Array).size(), (filled_p["xs"] as PackedInt32Array).size()])
	# F3: nine entries; 6 = door, one click.
	await _key(KEY_F3)
	var menu_buttons: Array = bar.get("_menu_buttons")
	_check("menu_nine", bool(bar.call("build_menu_open")) and menu_buttons.size() == 9, "buttons=%d %s" % [menu_buttons.size(), (menu_buttons[8] as Button).text if menu_buttons.size() == 9 else ""])
	await _key(KEY_6)
	_check("menu_6_door", str(tool.get("build_kind")) == "door" and int(tool.get("mode")) == 3, str(tool.get("build_kind")))
	var placed_door: Dictionary = tool.call("place_at", door)
	_check("door_placed", bool(placed_door.get("accepted", false)), str(placed_door))
	# F3 5 = log wall and its drag hint (no placement: the live scene's colony does not chop; see UI_REPORT.md).
	await _pick_menu(KEY_5, bar)
	_check("menu_5_log_wall", str(tool.get("build_kind")) == "log_wall", str(tool.get("build_kind")))
	var hint: Label = bar.get("_hint")
	var log_hint: String = hint.text if hint != null else ""
	# F3 9 = stone wall; Shift+drag the hollow rectangle (the door tile is refused).
	await _pick_menu(KEY_9, bar)
	_check("menu_9_stone_wall", str(tool.get("build_kind")) == "stone_wall", "%s (log wall hint '%s')" % [str(tool.get("build_kind")), log_hint])
	var line_word: String = str(locale.call("ltr", "UI_HINT_DRAG_LINE")) if locale != null else "UI_HINT_DRAG_LINE"
	_check("drag_hint", hint != null and line_word in hint.text, hint.text if hint != null else "")
	await _drag(a, b, true)
	var walls: Dictionary = tool.call("last_result")
	var toast: Label = bar.get("_toast")
	var headless: bool = DisplayServer.get_name() == "headless"
	var drag_ok: bool = bool(walls.get("accepted", false)) and int(walls.get("count", 0)) == 19
	if headless and not drag_ok:
		# The headless window does not map event positions onto the 1920×1080 canvas (godot skill, procedure 9).
		print("M3_UI NOT_OBSERVABLE wall_rect_drag headless (%s toast '%s'); the same drag goes through place_drag below" % [str(walls), toast.text if toast != null else ""])
	else:
		_check("wall_rect_drag", drag_ok, "%s toast '%s'" % [str(walls), toast.text if toast != null else ""])
	var preview: Dictionary = tool.call("preview")
	_check("drag_preview_red", int(tool.call("refused_count", preview)) >= 1, "preview tiles %d refused %d" % [(preview.get("xs", PackedInt32Array()) as PackedInt32Array).size(), int(tool.call("refused_count", preview))])
	# Fallback when synthetic mouse events do not reach the tool headless: place through the tool's API.
	if not bool(walls.get("accepted", false)):
		walls = tool.call("place_drag", a, b, true)
		_check("wall_rect_api", bool(walls.get("accepted", false)), "%s toast '%s'" % [str(walls), toast.text if toast != null else ""])
	# F6 cancels the door site (a door needs a log), then a one-tile stone line closes the ring.
	tool.call("set_mode", 5)
	var cancelled: Dictionary = tool.call("area", door, door)
	_check("f6_cancel_door", bool(cancelled.get("accepted", false)), str(cancelled))
	await _pick_menu(KEY_9, bar)
	var closing: Dictionary = tool.call("place_drag", door, door, false)
	_check("close_ring", bool(closing.get("accepted", false)) and int(closing.get("count", 0)) == 1, str(closing))
	# F3 8 = bedroll (two), F3 7 = packed floor line along the middle row.
	await _pick_menu(KEY_8, bar)
	var bed1: Dictionary = tool.call("place_at", a + Vector2i(1, 1))
	var bed2: Dictionary = tool.call("place_at", a + Vector2i(2, 1))
	_check("bedrolls", bool(bed1.get("accepted", false)) and bool(bed2.get("accepted", false)), "%s %s" % [bed1, bed2])
	await _pick_menu(KEY_7, bar)
	var floors: Dictionary = tool.call("place_drag", a + Vector2i(1, 3), a + Vector2i(5, 3), false)
	_check("floor_line", bool(floors.get("accepted", false)) and int(floors.get("count", 0)) == 5, "%s toast '%s'" % [floors, toast.text if toast != null else ""])
	await _key(KEY_ESCAPE)
	var kinds: Dictionary = _site_kinds()
	_check("sites", int(kinds.get(5, 0)) == 20 and int(kinds.get(6, 0)) == 0 and int(kinds.get(8, 0)) == 2 and int(kinds.get(7, 0)) == 5, str(kinds))
	# Build: advance until the house is a room.
	var room: int = -1
	var tick: int = 0
	while tick < MAX_TICKS:
		SimReads.mutate(_world, "advance_ticks", [STEP_TICKS])
		tick += STEP_TICKS
		await _frames(3)
		room = int(rooms.call("room_at", a + Vector2i(3, 2)))
		if room >= 0 and int(buildings.call("wall_count")) >= 20:
			break
	var ov: Dictionary = SimReads.read(_world, "get_colony_overlay")
	var by_kind: Dictionary = {}
	var ik: PackedInt32Array = ov.get("item_kinds", PackedInt32Array())
	var ic: PackedInt32Array = ov.get("item_counts", PackedInt32Array())
	for i: int in mini(ik.size(), ic.size()):
		by_kind[ik[i]] = int(by_kind.get(ik[i], 0)) + ic[i]
	print("M3_UI INFO after %d ticks: ground items by kind %s, trees marked %d, site delivered %s needed %s" % [tick, by_kind, _designations(0), str(ov.get("site_delivered", PackedInt32Array())), str(ov.get("site_needed", PackedInt32Array()))])
	_check("house_room", room >= 0, "after %d ticks: room %d, walls drawn %d, floor pieces %d, sites left %s" % [tick, room, int(buildings.call("wall_count")), int(buildings.call("floor_piece_count")), str(_site_kinds())])
	if room < 0:
		_finish()
		return
	var space: Dictionary = SimReads.read(_world, "get_space")
	_check("space_drawn", int(buildings.call("wall_count")) == (space.get("wall_xs", PackedInt32Array()) as PackedInt32Array).size(), "walls %d" % int(buildings.call("wall_count")))
	# R: the overlay and the bar's toggle.
	await _key(KEY_R)
	var toggle: Button = bar.find_child("RoomsToggle", true, false) as Button
	_check("r_toggle", bool(rooms.call("is_shown")) and toggle != null and toggle.button_pressed, "shown %s button %s" % [rooms.call("is_shown"), toggle.button_pressed if toggle != null else false])
	var info: Dictionary = rooms.call("room", room)
	print("M3_UI INFO room_label '%s' (%d rooms)" % [RoomLayer.label_text(info, locale), int(rooms.call("room_count"))])
	print("M3_UI INFO get_room_detail(%d) = %s" % [room, str(_world.call("get_room_detail", room))])
	# Click a room tile: the room panel.
	var inside: Vector2i = a + Vector2i(4, 2)
	var agent_layer: Node = _main.get_node("ColonyView/AgentLayer")
	var people_inside: int = 0
	for y: int in range(a.y + 1, b.y):
		for x: int in range(a.x + 1, b.x):
			if int(agent_layer.call("entity_at", Vector2(x, y) * 16.0 + Vector2(8, 8))) >= 0:
				people_inside += 1
			else:
				inside = Vector2i(x, y)
	var snap_people: int = 0
	var sxs: PackedInt32Array = SimReads.read(_world, "get_agent_snapshot").get("xs", PackedInt32Array())
	var sys: PackedInt32Array = SimReads.read(_world, "get_agent_snapshot").get("ys", PackedInt32Array())
	for i: int in mini(sxs.size(), sys.size()):
		if sxs[i] > a.x and sxs[i] < b.x and sys[i] > a.y and sys[i] < b.y:
			snap_people += 1
	print("M3_UI INFO people inside the doorless ring: %d by the snapshot (%d tiles hit by the agent layer); clicking %s" % [snap_people, people_inside, inside])
	tool.call("click_world", Vector2(inside) * 16.0 + Vector2(8, 8), inside)
	await _frames(2)
	print("M3_UI INFO after click: room panel visible %s, inspector visible %s, causal visible %s, rooms shown %s, room_at %d" % [(room_panel as Control).visible,
		(inspector as Control).visible, (_main.get_node("UI/CausalPanel") as Control).visible, rooms.call("is_shown"), int(rooms.call("room_at", inside))])
	var lines: Label = room_panel.get("_lines")
	var why: Label = room_panel.get("_why")
	var title: Label = room_panel.get("_title")
	_check("room_panel", (room_panel as Control).visible and int(room_panel.call("current_room")) == room, "'%s' | '%s' | '%s'" % [title.text, lines.text.replace("\n", " / "), why.text])
	# Override the role: storehouse, then auto.
	var set_store: Dictionary = room_panel.call("choose_role", 2)
	SimReads.mutate(_world, "advance_ticks", [2])
	await _frames(3)
	var after: Dictionary = rooms.call("room", room)
	_check("role_override", bool(set_store.get("accepted", false)) and int(after.get("role", -1)) == 2 and bool(after.get("overridden", false)), "%s -> %s label '%s'" % [set_store, after, RoomLayer.label_text(after, locale)])
	var set_auto: Dictionary = room_panel.call("choose_role", 0)
	SimReads.mutate(_world, "advance_ticks", [2])
	await _frames(3)
	var back: Dictionary = rooms.call("room", room)
	_check("role_auto", bool(set_auto.get("accepted", false)) and not bool(back.get("overridden", true)), "%s -> %s" % [set_auto, back])
	room_panel.call("close")
	# Inspector bed line: someone whose bed is in the room (after the night), else the first person.
	var agents: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var ids: PackedInt64Array = agents.get("agent_ids", PackedInt64Array())
	var chosen: int = ids[0] if ids.size() > 0 else -1
	for id: int in ids:
		var eco: Dictionary = _world.call("get_agent_economy", id)
		if int(eco.get("bed_room", -1)) == room:
			chosen = id
			break
	var person_eco: Dictionary = _world.call("get_agent_economy", chosen)
	var bed_text: String = Inspector.bed_line(person_eco, rooms, false, locale)
	print("M3_UI INFO bed agent %d bed (%d,%d) room %d -> '%s'" % [chosen, int(person_eco.get("bed_x", -1)), int(person_eco.get("bed_y", -1)), int(person_eco.get("bed_room", -1)), bed_text])
	var bits: int = -1
	var cache_ids: PackedInt64Array = agents.get("agent_ids", PackedInt64Array())
	var entities: Variant = agents.get("entity_bits", agents.get("ids", null))
	if entities is PackedInt64Array:
		bits = (entities as PackedInt64Array)[cache_ids.find(chosen)] if cache_ids.find(chosen) >= 0 else -1
	if bits >= 0:
		var detail: Dictionary = _world.call("get_agent_detail", bits)
		inspector.call("display_agent", detail)
		await _frames(2)
	var bed_label: Label = inspector.get("_bed_label")
	_check("inspector_bed_line", bed_label != null and bed_label.text != "" and not bed_label.text.begins_with("UI_"), "'%s' (visible %s)" % [bed_label.text if bed_label != null else "", bed_label.visible if bed_label != null else false])
	inspector.call("close")
	# The overlay hover suffix on a room tile.
	var suffix: String = OverlayLayer.room_suffix(rooms, inside, true, locale)
	_check("hover_room_suffix", suffix.length() > 3, "'%s'" % suffix)
	# F10 deconstruct one wall tile.
	await _key(KEY_F10)
	var decon: Dictionary = tool.call("area", a, a)
	_check("f10_deconstruct", int(tool.get("mode")) == 8 and bool(decon.get("accepted", false)) and _designations(3) >= 1, "%s marks %d" % [decon, _designations(3)])
	# F8 dig: the nearest hill tile with a non-hill, non-water neighbour.
	var terrain: Dictionary = SimReads.read(_world, "get_terrain_snapshot")
	var width: int = int(terrain.get("width", 0))
	var height: int = int(terrain.get("height", 0))
	var types: PackedByteArray = terrain.get("terrain", PackedByteArray())
	var hill: Vector2i = Vector2i(-1, -1)
	var best: int = 1 << 30
	for y: int in range(1, height - 1):
		for x: int in range(1, width - 1):
			if types[y * width + x] != 3:
				continue
			var d: int = (Vector2i(x, y) - hearth).length_squared()
			if d >= best:
				continue
			for step: Vector2i in [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 0)]:
				var t: int = types[(y + step.y) * width + x + step.x]
				if t == 0 or t == 1:
					hill = Vector2i(x, y)
					best = d
					break
	await _key(KEY_F8)
	if hill.x >= 0:
		var dig: Dictionary = tool.call("area", hill, hill)
		_check("f8_dig", int(tool.get("mode")) == 7 and bool(dig.get("accepted", false)) and _designations(2) >= 1, "hill %s %s marks %d toast '%s'" % [hill, dig, _designations(2), toast.text])
	else:
		_check("f8_dig", int(tool.get("mode")) == 7, "no hill on this map; mode only")
	await _key(KEY_ESCAPE)
	# Chronicle kind 23 text.
	var formed: String = Chronicle.sentence({"kind": 23, "cause": 1, "value": room, "subject": -1}, -1, locale, {})
	_check("chronicle_room_formed", formed != "" and not formed.begins_with("CHRONICLE"), "'%s'" % formed)
	# Priority table: seven work types.
	var works: PackedStringArray = SimReads.read(_world, "get_work_priorities").get("work_types", PackedStringArray())
	await _key(KEY_P)
	var priorities: Control = _main.get_node_or_null("UI/PriorityPanel") as Control
	await _frames(2)
	var grid: GridContainer = null
	for node: Node in priorities.find_children("*", "GridContainer", true, false):
		grid = node as GridContainer
	_check("priority_columns", works.size() == 7 and grid != null and grid.columns == 8, "works %s columns %d grid width %d" % [",".join(works), grid.columns if grid != null else -1, int(grid.get_combined_minimum_size().x) if grid != null else -1])
	await _key(KEY_P)
	# The bar fits the screen.
	var bar_panel: Control = bar.find_child("Bar", true, false) as Control
	if bar_panel != null:
		_check("bar_fits", bar_panel.position.x + bar_panel.get_combined_minimum_size().x <= 1920.0 and bar_panel.position.y + bar_panel.get_combined_minimum_size().y <= 1080.0,
			"bar at %s min size %s" % [bar_panel.position, bar_panel.get_combined_minimum_size()])
	_finish()


func _finish() -> void:
	print("M3_UI_DONE failures=%d %s" % [_failures.size(), ",".join(_failures)])
	quit(0 if _failures.is_empty() else 1)
