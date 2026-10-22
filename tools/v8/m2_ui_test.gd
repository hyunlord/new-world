# tools/v8/m2_ui_test.gd — headless check of the M2 UI paths (goal §9), asserted through the bridge reads.
# Purpose: boot res://scenes/main.tscn with the clock paused, then as a player would: F2 stockpile and F7 quarry
#   (OrderTool area orders) near the camp; the F3 key opens the build menu, key 2 picks 돌깎이 자리 and a placement
#   puts its blueprint down; advance_ticks until it is built; a click on it opens the workshop panel, whose 추가 →
#   "부싯돌 날 깎기" button adds a bill (checked in get_workshops: make 5), + raises it to 6, 유지로 switches it to
#   keep, 멈추기 suspends it; the P key opens the priority table, a click on a cell cycles that level (checked in
#   get_work_priorities) and 모두 기본 resets it; the inspector shows the skill lines of a person. Buttons are
#   pressed by emitting their `pressed` signal (headless has no pixels; layout is not checked here).
# Bridge methods called: via SimReads get_colony_overlay, get_workshops, get_work_priorities, get_agent_snapshot,
#   get_building_snapshot, advance_ticks; can_place_blueprint, get_agent_detail (direct); and the UI's own calls.
# Run: bash tools/v8/run_bounded.sh 900 "$GODOT_BIN" --headless --path . --script res://tools/v8/m2_ui_test.gd
#   Windowed (--rendering-driver opengl3 --resolution 1920x1080) with "-- --shots <abs dir>" it also saves
#   m2_menu.png (build menu), m2_workshop.png (workshop panel with the bill), m2_priorities.png and m2_inspector.png.
#   With "-- --late --shots <abs dir>" (windowed) it instead plays the M2-U orders of tools/v8/m2_u_script.json at
#   tick 360 (20 nearest trees, a 6×6 stockpile east of the camp, quarry the 2 nearest boulders and the nearest
#   outcrop, a knapping spot and a wood bench south of the camp, one adult's chop 0 and another's craft 1; bills
#   knap make 6, haft axe make 2, firewood keep 40 once each workshop stands), advances until blades exist and
#   someone carries one, and saves the goal's 11.3 screens: m2s_1_bills.png (the knapping spot's bill list),
#   m2s_2_priorities.png, m2s_3_inspector.png (a person with a skill tier and the carried item), m2s_4_menu.png
#   (F3 build menu), plus m2s_shots.json with each shot's tick and what it shows.
# Output: one "M2_UI <PASS|FAIL> <step> <detail>" line per check, then "M2_UI_DONE failures=N"; quits 0 / 1.
extends SceneTree

const WorkshopPanel = preload("res://scripts/ui/panels/workshop_panel.gd")
const MAX_BUILD_TICKS: int = 14400
const STEP_TICKS: int = 360

var _failures: Array[String] = []
var _main: Node
var _world: Node
var _shots: String = ""
var _late: bool = false
var _manifest: Dictionary = {}


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--shots" and i + 1 < args.size():
			_shots = args[i + 1]
		elif args[i] == "--late":
			_late = true
	call_deferred("_run_late" if _late else "_run")


## Save the screen to <shots>/<name> after a few frames (windowed runs only).
func _shot(name: String) -> void:
	if _shots == "":
		return
	DirAccess.make_dir_recursive_absolute(_shots)
	await _frames(6)
	root.get_texture().get_image().save_png(_shots.path_join(name))


func _check(step: String, ok: bool, detail: String = "") -> void:
	print("M2_UI %s %s %s" % ["PASS" if ok else "FAIL", step, detail])
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


func _press(panel: Node, node_name: String) -> bool:
	var button: Button = panel.find_child(node_name, true, false) as Button
	if button == null:
		return false
	button.pressed.emit()
	return true


func _hearth() -> Vector2i:
	var b: Dictionary = SimReads.read(_world, "get_building_snapshot")
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	var xs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = b.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[i] == 0:
			return Vector2i(xs[i], ys[i])
	return Vector2i(96, 96)


func _shop_bills(shop: int) -> Array[Dictionary]:
	var shops: Dictionary = SimReads.read(_world, "get_workshops")
	var row: int = (shops.get("ids", PackedInt32Array()) as PackedInt32Array).find(shop)
	if row < 0:
		return []
	return WorkshopPanel.bills_of(shops, row)


func _run() -> void:
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	await _frames(20)
	_world = _main.get_node_or_null("WorldSim")
	var tool: Node = _main.get_node_or_null("ColonyView/OrderTool")
	var bar: Node = _main.get_node_or_null("UI/OrderBar")
	var shop_panel: Node = _main.get_node_or_null("UI/WorkshopPanel")
	var priorities: Node = _main.get_node_or_null("UI/PriorityPanel")
	var inspector: Node = _main.get_node_or_null("UI/AgentInspectorPanel")
	if _world == null or tool == null or bar == null or shop_panel == null or priorities == null or inspector == null:
		_check("scene", false, "missing nodes")
		_finish()
		return
	# Space pauses (M2: P is the priority table); only advance_ticks moves the world from here.
	await _key(KEY_SPACE)
	var camera: Node = _main.get_node_or_null("Camera2D")
	_check("space_pauses", camera != null and bool(camera.call("is_paused")))
	var hearth: Vector2i = _hearth()
	# Stockpile (F2) south-east of the camp; the first offset that is accepted.
	tool.call("set_mode", 2)
	var stock: Dictionary = {}
	for offset: Vector2i in [Vector2i(3, 3), Vector2i(-8, 3), Vector2i(3, -8), Vector2i(-8, -8)]:
		stock = tool.call("area", hearth + offset, hearth + offset + Vector2i(5, 5))
		if bool(stock.get("accepted", false)):
			break
	_check("stockpile", bool(stock.get("accepted", false)), str(stock))
	# Quarry (F7) the two boulders nearest the camp.
	var overlay: Dictionary = SimReads.read(_world, "get_colony_overlay")
	var nxs: PackedInt32Array = overlay.get("node_xs", PackedInt32Array())
	var nys: PackedInt32Array = overlay.get("node_ys", PackedInt32Array())
	var nkinds: PackedInt32Array = overlay.get("node_kinds", PackedInt32Array())
	var boulders: Array[Vector2i] = []
	for i: int in mini(nxs.size(), mini(nys.size(), nkinds.size())):
		if nkinds[i] == 0:
			boulders.append(Vector2i(nxs[i], nys[i]))
	boulders.sort_custom(func(a: Vector2i, b: Vector2i) -> bool: return (a - hearth).length_squared() < (b - hearth).length_squared())
	_check("boulders_exist", boulders.size() >= 2, "boulders=%d" % boulders.size())
	await _key(KEY_F7)
	_check("f7_quarry_mode", int(tool.get("mode")) == 6)
	var marked: int = 0
	for b: Vector2i in boulders.slice(0, 2):
		marked += int((tool.call("area", b, b) as Dictionary).get("count", 0))
	await _frames(2)
	overlay = SimReads.read(_world, "get_colony_overlay")
	var dkinds: PackedInt32Array = overlay.get("designation_kinds", PackedInt32Array())
	_check("quarry_marked", marked >= 2 and dkinds.has(1), "count=%d" % marked)
	await _key(KEY_ESCAPE)
	# F3 opens the build menu; 2 picks the knapping spot.
	await _key(KEY_F3)
	_check("f3_opens_build_menu", bool(bar.call("build_menu_open")))
	await _shot("m2_menu.png")
	await _key(KEY_2)
	_check("key2_picks_knapping_spot", not bool(bar.call("build_menu_open")) and int(tool.get("mode")) == 3 and str(tool.get("build_kind")) == "knapping_spot",
		"mode=%d kind=%s speed=%s" % [int(tool.get("mode")), str(tool.get("build_kind")), str(camera.call("get_speed"))])
	var spot: Vector2i = Vector2i(-1, -1)
	for radius: int in range(2, 12):
		for dy: int in range(-radius, radius + 1):
			for dx: int in [-radius, radius]:
				var t: Vector2i = hearth + Vector2i(dx, dy)
				if spot.x < 0 and bool(_world.call("can_place_blueprint", "knapping_spot", t.x, t.y)) and bool(_world.call("can_place_blueprint", "knapping_spot", t.x, t.y + 1)):
					spot = t
	var camera2d: Camera2D = camera as Camera2D
	if camera2d != null:
		camera2d.position = Vector2(spot) * 16.0 + Vector2(8, 8)
	var placed: Dictionary = tool.call("place_at", spot)
	await _frames(2)
	overlay = SimReads.read(_world, "get_colony_overlay")
	_check("knapping_spot_blueprint", bool(placed.get("accepted", false)) and (overlay.get("site_kinds", PackedInt32Array()) as PackedInt32Array).has(1), "at=%s %s" % [spot, placed])
	tool.call("set_mode", 0)
	# Advance until a workshop exists.
	var shop: int = -1
	var shop_tile: Vector2i = Vector2i.ZERO
	var waited: int = 0
	while shop < 0 and waited < MAX_BUILD_TICKS:
		SimReads.mutate(_world, "advance_ticks", [STEP_TICKS])
		waited += STEP_TICKS
		await process_frame
		var shops: Dictionary = SimReads.read(_world, "get_workshops")
		var ids: PackedInt32Array = shops.get("ids", PackedInt32Array())
		# M4: the camp hearth is a workshop too (kind 11, fire_pot only) from the start; find the knapping spot (kind 1).
		var row: int = (shops.get("kinds", PackedInt32Array()) as PackedInt32Array).find(1)
		if row >= 0 and row < ids.size():
			shop = ids[row]
			shop_tile = Vector2i((shops["xs"] as PackedInt32Array)[row], (shops["ys"] as PackedInt32Array)[row])
	_check("knapping_spot_built", shop >= 0, "after %d ticks" % waited)
	if shop >= 0:
		await _frames(2)
		var layer: Node = _main.get_node("ColonyView/BuildingLayer")
		_check("building_layer_draws_it", int(layer.call("workshop_count")) >= 1)
		_check("no_hearth_on_workshop", int(layer.call("hearth_count")) == 1, "hearths drawn=%d" % int(layer.call("hearth_count")))
		tool.call("click_world", Vector2(shop_tile) * 16.0 + Vector2(8, 8), shop_tile)
		await _frames(2)
		_check("click_opens_workshop_panel", int(shop_panel.call("workshop_id")) == shop and (shop_panel as Control).visible)
		var added: bool = _press(shop_panel, "Add") and _press(shop_panel, "Recipe_knap_blade")
		await _frames(2)
		var bills: Array[Dictionary] = _shop_bills(shop)
		_check("add_bill_make_5", added and bills.size() == 1 and int(bills[0]["mode"]) == 0 and int(bills[0]["count"]) == 5, str(bills))
		_press(shop_panel, "Plus_0")
		await _frames(2)
		bills = _shop_bills(shop)
		_check("plus_raises_to_6", bills.size() == 1 and int(bills[0]["count"]) == 6, str(bills))
		_press(shop_panel, "Mode_0")
		await _frames(2)
		bills = _shop_bills(shop)
		_check("switch_to_keep", bills.size() == 1 and int(bills[0]["mode"]) == 1 and int(bills[0]["count"]) == 6, str(bills))
		_press(shop_panel, "Suspend_0")
		await _frames(2)
		bills = _shop_bills(shop)
		_check("suspend", bills.size() == 1 and int(bills[0]["suspended"]) == 1, str(bills))
		var status: Label = shop_panel.find_child("Bill_0", true, false).get_child(1) as Label
		_check("bill_status_text", status != null and status.text.length() > 0, status.text if status != null else "")
		_press(shop_panel, "Add")
		await _shot("m2_workshop.png")
		shop_panel.call("close")
	# P opens the priority table; a cell click cycles the level; 모두 기본 resets.
	await _key(KEY_P)
	_check("p_opens_priorities", (priorities as Control).visible)
	await _frames(2)
	var before: Dictionary = SimReads.read(_world, "get_work_priorities")
	var levels: PackedByteArray = before.get("levels", PackedByteArray())
	var ids64: PackedInt64Array = before.get("agent_ids", PackedInt64Array())
	if levels.size() >= 4 and not ids64.is_empty():
		var old: int = int(levels[3])
		_check("cell_button_exists", _press(priorities, "Cell_0_3"))
		await _frames(2)
		var after: PackedByteArray = SimReads.read(_world, "get_work_priorities").get("levels", PackedByteArray())
		_check("cell_click_cycles", int(after[3]) == (old + 1) % 5, "%d -> %d" % [old, int(after[3])])
		_press(priorities, "Reset")
		await _frames(2)
		after = SimReads.read(_world, "get_work_priorities").get("levels", PackedByteArray())
		_check("reset_to_default", int(after[3]) == 3, "chop=%d" % int(after[3]))
		_press(priorities, "Cell_1_0")
		await _shot("m2_priorities.png")
	else:
		_check("priorities_rows", false, "no adults")
	await _key(KEY_P)
	_check("p_closes_priorities", not (priorities as Control).visible)
	# Inspector: skill lines for the first person (six since M3: mining).
	var snap: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var bits_list: PackedInt64Array = snap.get("ids", PackedInt64Array())
	if not bits_list.is_empty():
		var detail: Dictionary = _world.call("get_agent_detail", bits_list[0])
		inspector.call("display_agent", detail)
		await _frames(2)
		await _shot("m2_inspector.png")
		var skills: Label = inspector.get("_skills_label") as Label
		var tool_line: Label = inspector.get("_tool_label") as Label
		var locale: Node = root.get_node_or_null("Locale")
		var tier0: String = str(locale.call("ltr", "SKILL_TIER_0")) if locale != null else "SKILL_TIER_0"
		# Seven skills since M4 (6 = pottery).
		_check("inspector_skills", skills != null and skills.visible and skills.text.split("\n").size() == 7 and tier0 in skills.text, skills.text.replace("\n", " | ") if skills != null else "")
		_check("inspector_tool_line", tool_line != null and tool_line.visible, tool_line.text if tool_line != null else "")
	_finish()


func _finish() -> void:
	print("M2_UI_DONE failures=%d %s" % [_failures.size(), ",".join(_failures)])
	quit(0 if _failures.is_empty() else 1)


# ---------------------------------------------------------------------------------------------- late (11.3 screens)
const LATE_MAX_TICKS: int = 4 * 1440


func _tick() -> int:
	return int(SimReads.read(_world, "get_time_snapshot").get("tick", -1))


func _advance(n: int) -> void:
	SimReads.mutate(_world, "advance_ticks", [n])
	await process_frame


func _nearest(points: Array[Vector2i], from: Vector2i, count: int) -> Array[Vector2i]:
	var sorted: Array[Vector2i] = points.duplicate()
	sorted.sort_custom(func(a: Vector2i, b: Vector2i) -> bool:
		var da: int = (a - from).length_squared()
		var db: int = (b - from).length_squared()
		return da < db or (da == db and (a.y < b.y or (a.y == b.y and a.x < b.x))))
	return sorted.slice(0, count)


## The first tile south of `hearth` (rows 2..14, columns -4..4) where `kind` fits and does not overlap `taken`.
func _south_spot(kind: String, hearth: Vector2i, size: Vector2i, taken: Array[Rect2i]) -> Vector2i:
	for dy: int in range(2, 15):
		for dx: int in [0, -1, 1, -2, 2, -3, 3, -4, 4]:
			var t: Vector2i = hearth + Vector2i(dx, dy)
			var rect: Rect2i = Rect2i(t, size).grow(1)
			var free: bool = true
			for other: Rect2i in taken:
				free = free and not other.intersects(rect)
			if free and bool(_world.call("can_place_blueprint", kind, t.x, t.y)):
				return t
	return Vector2i(-1, -1)


func _shops() -> Dictionary:
	return SimReads.read(_world, "get_workshops")


## The id of the workshop of kind `code`, or -1.
func _shop_of(code: int) -> int:
	var shops: Dictionary = _shops()
	var ids: PackedInt32Array = shops.get("ids", PackedInt32Array())
	var kinds: PackedInt32Array = shops.get("kinds", PackedInt32Array())
	for i: int in mini(ids.size(), kinds.size()):
		if kinds[i] == code:
			return ids[i]
	return -1


func _economy(agent: int) -> Dictionary:
	return _world.call("get_agent_economy", agent)


## The best person for the inspector shot: carries a flint blade and has a skill level > 0 (best), else carries a
## blade, else carries any M2 item with a material, else has a skill level > 0. Returns {agent, score, economy}.
func _pick_person() -> Dictionary:
	var ids: PackedInt64Array = SimReads.read(_world, "get_agent_snapshot").get("agent_ids", PackedInt64Array())
	var best: Dictionary = {"agent": -1, "score": -1}
	for id: int in ids:
		var e: Dictionary = _economy(id)
		if not bool(e.get("found", false)):
			continue
		var skilled: bool = false
		for level: int in (e.get("skill_levels", PackedInt32Array()) as PackedInt32Array):
			skilled = skilled or level > 0
		var carry: int = int(e.get("carry_kind", -1))
		var score: int = (4 if carry == 4 else (2 if carry > 0 and str(e.get("carry_material", "")) != "" else 0)) + (1 if skilled else 0)
		if score > int(best["score"]):
			best = {"agent": id, "score": score, "economy": e}
	return best


func _late_shot(name: String, what: String) -> void:
	await _shot(name)
	_manifest[name] = {"tick": _tick(), "shows": what}
	print("M2_SHOT %s tick=%d %s" % [name, _tick(), what])


func _run_late() -> void:
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	await _frames(20)
	_world = _main.get_node_or_null("WorldSim")
	var tool: Node = _main.get_node_or_null("ColonyView/OrderTool")
	var bar: Node = _main.get_node_or_null("UI/OrderBar")
	var shop_panel: Node = _main.get_node_or_null("UI/WorkshopPanel")
	var priorities: Node = _main.get_node_or_null("UI/PriorityPanel")
	var inspector: Node = _main.get_node_or_null("UI/AgentInspectorPanel")
	var camera: Camera2D = _main.get_node_or_null("Camera2D") as Camera2D
	await _key(KEY_SPACE)
	_check("late_paused", camera != null and bool(camera.call("is_paused")))
	if _tick() < 360:
		await _advance(360 - _tick())
	var hearth: Vector2i = _hearth()
	# 20 nearest trees.
	var trees: Dictionary = SimReads.read(_world, "get_trees")
	var tree_tiles: Array[Vector2i] = []
	var txs: PackedInt32Array = trees.get("xs", PackedInt32Array())
	var tys: PackedInt32Array = trees.get("ys", PackedInt32Array())
	for i: int in mini(txs.size(), tys.size()):
		tree_tiles.append(Vector2i(txs[i], tys[i]))
	var chop_x: PackedInt32Array = PackedInt32Array()
	var chop_y: PackedInt32Array = PackedInt32Array()
	for t: Vector2i in _nearest(tree_tiles, hearth, 20):
		chop_x.append(t.x)
		chop_y.append(t.y)
	var chop: Dictionary = SimReads.mutate(_world, "cmd_designate_chop_tiles", [chop_x, chop_y])
	_check("late_chop", bool(chop.get("accepted", false)), str(chop))
	# Workshops south of the camp first (so the stockpile cannot take their ground).
	var taken: Array[Rect2i] = []
	var spot: Vector2i = _south_spot("knapping_spot", hearth, Vector2i(1, 1), taken)
	tool.call("set_build", "knapping_spot")
	var placed_spot: Dictionary = tool.call("place_at", spot)
	taken.append(Rect2i(spot, Vector2i(1, 2)))
	var bench: Vector2i = _south_spot("wood_bench", hearth, Vector2i(2, 1), taken)
	tool.call("set_build", "wood_bench")
	var placed_bench: Dictionary = tool.call("place_at", bench)
	_check("late_workshops_placed", bool(placed_spot.get("accepted", false)) and bool(placed_bench.get("accepted", false)), "%s %s / %s %s" % [spot, placed_spot, bench, placed_bench])
	# A 6×6 stockpile east of the camp (offset 3), else the first accepted fallback.
	tool.call("set_mode", 2)
	var stock: Dictionary = {}
	for offset: Vector2i in [Vector2i(3, -2), Vector2i(3, 3), Vector2i(-8, -2), Vector2i(3, -8)]:
		stock = tool.call("area", hearth + offset, hearth + offset + Vector2i(5, 5))
		if bool(stock.get("accepted", false)):
			break
	_check("late_stockpile", bool(stock.get("accepted", false)), str(stock))
	# Quarry the 2 nearest boulders and the nearest outcrop.
	var overlay: Dictionary = SimReads.read(_world, "get_colony_overlay")
	var nxs: PackedInt32Array = overlay.get("node_xs", PackedInt32Array())
	var nys: PackedInt32Array = overlay.get("node_ys", PackedInt32Array())
	var nkinds: PackedInt32Array = overlay.get("node_kinds", PackedInt32Array())
	var boulders: Array[Vector2i] = []
	var outcrops: Array[Vector2i] = []
	for i: int in mini(nxs.size(), mini(nys.size(), nkinds.size())):
		(boulders if nkinds[i] == 0 else outcrops).append(Vector2i(nxs[i], nys[i]))
	tool.call("set_mode", 6)
	var quarried: int = 0
	for t: Vector2i in _nearest(boulders, hearth, 2) + _nearest(outcrops, hearth, 1):
		quarried += int((tool.call("area", t, t) as Dictionary).get("count", 0))
	_check("late_quarry", quarried >= 3, "marked=%d" % quarried)
	tool.call("set_mode", 0)
	# Priorities: adult rank 0 chop 0, rank 1 craft 1.
	var adults: PackedInt64Array = SimReads.read(_world, "get_work_priorities").get("agent_ids", PackedInt64Array())
	if adults.size() >= 2:
		SimReads.mutate(_world, "cmd_set_work_priority", [adults[0], "chop", 0])
		SimReads.mutate(_world, "cmd_set_work_priority", [adults[1], "craft", 1])
	# Run; add bills as the workshops appear; stop once blades exist and someone carries one (or the cap).
	var knap_bill: bool = false
	var bench_bills: bool = false
	var start: int = _tick()
	var made: int = 0
	var carrier: Dictionary = {}
	while _tick() - start < LATE_MAX_TICKS:
		await _advance(30)
		var knap: int = _shop_of(1)
		if knap >= 0 and not knap_bill:
			knap_bill = bool((SimReads.mutate(_world, "cmd_add_bill", [knap, "knap_blade", "make", 6]) as Dictionary).get("accepted", false))
		var wood: int = _shop_of(2)
		if wood >= 0 and not bench_bills:
			SimReads.mutate(_world, "cmd_add_bill", [wood, "haft_axe", "make", 2])
			SimReads.mutate(_world, "cmd_add_bill", [wood, "split_firewood", "maintain", 40])
			bench_bills = true
		if knap >= 0:
			var bills: Array[Dictionary] = _shop_bills(knap)
			made = int(bills[0]["done"]) if not bills.is_empty() else made
		if made >= 2:
			carrier = _pick_person()
			if int(carrier["score"]) >= 5:
				break
	_check("late_blades_made", made >= 1, "made=%d at tick %d" % [made, _tick()])
	carrier = _pick_person()
	_check("late_person", int(carrier["agent"]) >= 0, "score=%d %s" % [int(carrier["score"]), str(carrier.get("economy", {}))])
	# (1) The knapping spot with its bill list.
	var knap_id: int = _shop_of(1)
	var shops: Dictionary = _shops()
	var row: int = (shops.get("ids", PackedInt32Array()) as PackedInt32Array).find(knap_id)
	if row >= 0:
		var at: Vector2i = Vector2i((shops["xs"] as PackedInt32Array)[row], (shops["ys"] as PackedInt32Array)[row])
		camera.call("center_on_tile", at)
		camera.zoom = Vector2(3, 3)
		shop_panel.call("display_workshop", knap_id)
		await _late_shot("m2s_1_bills.png", "knapping spot bill list: %s" % str(_shop_bills(knap_id)))
		shop_panel.call("close")
	# (2) The priorities table.
	camera.zoom = Vector2(2, 2)
	camera.call("center_on_tile", hearth)
	await _key(KEY_P)
	await _late_shot("m2s_2_priorities.png", "priorities; adults %s" % str(adults.slice(0, 2)))
	await _key(KEY_P)
	# (3) The inspector of the chosen person, scrolled so the carried item, tool and skills show.
	var agent: int = int(carrier["agent"])
	var snap: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var aids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var index: int = aids.find(agent)
	if index >= 0:
		var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
		var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
		camera.call("center_on_tile", Vector2i(xs[index], ys[index]))
		inspector.call("display_agent", _world.call("get_agent_detail", bits[index]))
		await _frames(4)
		var scroll: ScrollContainer = inspector.get("_scroll") as ScrollContainer
		var carried: Label = inspector.get("_carried_label") as Label
		if scroll != null and carried != null:
			scroll.scroll_vertical = int(carried.position.y) - 8
		await _late_shot("m2s_3_inspector.png", "inspector of %d: %s" % [agent, str(carrier.get("economy", {}))])
		inspector.call("close")
	# (4) The build menu (F3).
	await _key(KEY_F3)
	await _late_shot("m2s_4_menu.png", "F3 build menu open=%s" % str(bar.call("build_menu_open")))
	await _key(KEY_ESCAPE)
	if _shots != "":
		var file: FileAccess = FileAccess.open(_shots.path_join("m2s_shots.json"), FileAccess.WRITE)
		if file != null:
			file.store_string(JSON.stringify(_manifest, "  ", true))
			file.close()
	_finish()
