# tools/v8/m4_ui_test.gd — headless check of the M4 UI paths (goal §9), asserted through the bridge reads.
# Purpose: boot res://scenes/main.tscn with the clock paused (Space); the HUD era line reads "구석기 n %" (get_tech
#   `progress` ‰ / 10; 2 % at the start: the founders' timber-wall practitioners count), the badge
#   "⚠ 기술 위험 2" (get_tech at_risk) and the warning line names the first at-risk node; T opens the tech map with
#   24 cards, the doorway card (node 10) in status 1 (위험); a click on it shows the detail (one holder, learners,
#   gates) and its 가르치기 button opens the teach dialog (K) on node 10, whose 명령 sends cmd_order_teach (accepted;
#   the open order is listed in get_teach_orders and in the dialog); K closes and reopens it; the inspector's 지식
#   tab lists a person's knowledge (no raw key) and, for the teacher, the lesson line once the lesson is assigned;
#   chronicle kinds 24–28 and the loss toast text are not raw keys (ko and en); F7 opens the submenu and 2 enters
#   the clay-pit mode (and marks the nearest clay bank); the build menu has 11 entries (0 토기대, - 갈돌); the
#   hearth stays drawn and its fire_pot workshop opens the workshop panel, where a fire_pot bill shows "기다리는 중:
#   토기 굽기를 아는 사람이 없음"; a stone-wall site's "why?" panel shows its wait line; the priority table has the
#   Pottery column.
# Bridge methods called: via SimReads get_tech, get_teach_orders, get_knowledge_waits, get_colony_overlay,
#   get_building_snapshot, get_workshops, get_work_priorities, get_agent_snapshot, advance_ticks; get_tech_node,
#   get_agent_knowledge, get_agent_detail, get_place_preview (direct); and the UI's own calls.
# Run: bash tools/v8/run_bounded.sh 900 "$GODOT_BIN" --headless --path . --script res://tools/v8/m4_ui_test.gd
# Output: one "M4_UI <PASS|FAIL|INFO> <step> <detail>" line per check, then "M4_UI_DONE failures=N"; quits 0 / 1.
extends SceneTree

const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const KnowledgeText = preload("res://scripts/ui/knowledge_text.gd")
const KEY_RE: String = "[A-Z][A-Z0-9]*(_[A-Z0-9]+)+"
const DOORWAY: int = 10
const POT_FIRING: int = 19

var _failures: Array[String] = []
var _main: Node
var _world: Node
var _locale: Node
var _raw: RegEx


func _init() -> void:
	call_deferred("_run")


func _check(step: String, ok: bool, detail: String = "") -> void:
	print("M4_UI %s %s %s" % ["PASS" if ok else "FAIL", step, detail])
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


## Whether `text` is non-empty and holds no key-shaped token.
func _readable(text: String) -> bool:
	return text.strip_edges() != "" and _raw.search(text) == null


func _hearth() -> Vector2i:
	var b: Dictionary = SimReads.read(_world, "get_building_snapshot")
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	var xs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = b.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[i] == 0:
			return Vector2i(xs[i], ys[i])
	return Vector2i(96, 96)


func _run() -> void:
	_raw = RegEx.new()
	_raw.compile(KEY_RE)
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	await _frames(20)
	_world = _main.get_node_or_null("WorldSim")
	_locale = root.get_node_or_null("Locale")
	var hud: Node = _main.get_node_or_null("UI/HudTopbar")
	var tech_panel: Node = _main.get_node_or_null("UI/TechPanel")
	var teach_panel: Node = _main.get_node_or_null("UI/TeachPanel")
	var inspector: Node = _main.get_node_or_null("UI/AgentInspectorPanel")
	var bar: Node = _main.get_node_or_null("UI/OrderBar")
	var tool: Node = _main.get_node_or_null("ColonyView/OrderTool")
	var shop_panel: Node = _main.get_node_or_null("UI/WorkshopPanel")
	var causal: Node = _main.get_node_or_null("UI/CausalPanel")
	if _world == null or hud == null or tech_panel == null or teach_panel == null or inspector == null or bar == null or tool == null \
			or shop_panel == null or causal == null or _locale == null:
		_check("scene", false, "missing nodes (tech %s teach %s)" % [tech_panel != null, teach_panel != null])
		_finish()
		return
	_locale.call("set_language", "ko")
	await _key(KEY_SPACE)
	await _frames(2)
	var tech: Dictionary = SimReads.read(_world, "get_tech")
	var at_risk: int = int(tech.get("at_risk", -1))
	# HUD: era line, badge, warning line.
	# The founders' two timber-wall practitioners already count (marker weight 2): progress 28 ‰ shows as "구석기 2 %".
	@warning_ignore("integer_division")
	var percent: int = int(tech.get("progress", 0)) / 10
	var era_want: String = str(_locale.call("trf", "M4_HUD_ERA_FMT", {"era": _locale.call("ltr", "M4_ERA_PALEOLITHIC"), "percent": percent}))
	var era: String = str(hud.call("era_text"))
	_check("hud_era_line", era == era_want and _readable(era), "'%s' (want '%s', progress %d era %d)" % [era, era_want, int(tech.get("progress", -1)), int(tech.get("era", -1))])
	var badge: String = str(hud.call("risk_text"))
	_check("hud_badge", at_risk == 2 and badge.contains("2") and _readable(badge), "'%s' at_risk %d" % [badge, at_risk])
	var warning: String = str(hud.call("warning_text"))
	var first: Dictionary = KnowledgeText.first_at_risk(tech)
	_check("hud_warning", _readable(warning) and warning.contains(KnowledgeText.node_name(_locale, int(first.get("id", -1)), tech)), "'%s'" % warning)
	var hud_panel: Control = null
	for node: Node in (hud as Control).get_children():
		if node is PanelContainer:
			hud_panel = node as Control
	_check("hud_fits", hud_panel != null and hud_panel.get_combined_minimum_size().x <= 1896.0, "top bar min size %s" % (hud_panel.get_combined_minimum_size() if hud_panel != null else Vector2.ZERO))
	# T: the tech map.
	await _key(KEY_T)
	await _frames(3)
	var map_panel: Control = null
	for node: Node in (tech_panel as Control).get_children():
		if node is PanelContainer:
			map_panel = node as Control
	_check("map_fits", map_panel != null and map_panel.get_combined_minimum_size().x <= 1896.0 and map_panel.get_combined_minimum_size().y <= 830.0,
		"tech map min size %s" % (map_panel.get_combined_minimum_size() if map_panel != null else Vector2.ZERO))
	var cards: int = 0
	for id: int in KnowledgeText.NODE_KEYS.size():
		if tech_panel.call("card", id) != null:
			cards += 1
	var door_card: Button = tech_panel.call("card", DOORWAY)
	_check("t_opens_map", (tech_panel as Control).visible and cards == 24, "visible %s cards %d" % [(tech_panel as Control).visible, cards])
	_check("doorway_card_at_risk", door_card != null and int(door_card.get_meta("status", -1)) == 1, "status %s" % (str(door_card.get_meta("status", -1)) if door_card != null else "no card"))
	var card_texts: Array[String] = []
	for label: Node in (tech_panel as Control).find_children("*", "Label", true, false):
		card_texts.append((label as Label).text)
	var raw_map: int = 0
	for text: String in card_texts:
		if text != "" and not _readable(text):
			raw_map += 1
	_check("map_text_localized", raw_map == 0, "labels %d raw %d" % [card_texts.size(), raw_map])
	# Click the doorway card: the detail.
	door_card.emit_signal("pressed")
	await _frames(2)
	var detail: Dictionary = tech_panel.call("detail")
	var holders: PackedInt64Array = detail.get("holder_ids", PackedInt64Array())
	var learners: PackedInt64Array = detail.get("learner_ids", PackedInt64Array())
	_check("node_detail", int(tech_panel.call("selected_node")) == DOORWAY and bool(detail.get("found", false)) and holders.size() == 1 and learners.size() > 0,
		"holders %s learners %d gates %s" % [holders, learners.size(), detail.get("gate_names", [])])
	var detail_box: Node = (tech_panel as Control).find_child("Detail", true, false)
	var detail_lines: Array[String] = []
	for label: Node in detail_box.find_children("*", "Label", true, false):
		detail_lines.append((label as Label).text)
	print("M4_UI INFO detail: %s" % " | ".join(detail_lines))
	var teach_button: Button = detail_box.find_child("Teach", true, false) as Button
	_check("detail_teach_button", teach_button != null and teach_button.visible and _readable(teach_button.text), teach_button.text if teach_button != null else "")
	# 가르치기 -> the teach dialog on node 10.
	teach_button.emit_signal("pressed")
	await _frames(2)
	var listed: Dictionary = teach_panel.call("listed")
	_check("teach_prefilled", (teach_panel as Control).visible and int(teach_panel.call("current_node")) == DOORWAY and (listed["teachers"] as Array).size() == 1,
		"node %d listed %s" % [int(teach_panel.call("current_node")), listed])
	var teacher: int = int(holders[0])
	var learner: int = int(learners[0])
	_check("choose_people", bool(teach_panel.call("choose_people", teacher, learner)), "teacher %d learner %d" % [teacher, learner])
	var order: Dictionary = teach_panel.call("submit")
	var toast: Label = bar.get("_toast")
	_check("teach_order_accepted", bool(order.get("accepted", false)), "%s toast '%s'" % [order, toast.text if toast != null else ""])
	await _frames(2)
	var orders: Dictionary = SimReads.read(_world, "get_teach_orders")
	_check("teach_order_listed", (orders.get("ids", PackedInt64Array()) as PackedInt64Array).size() == 1 and int(teach_panel.call("order_rows")) == 1,
		"orders %s rows %d" % [orders.get("ids", []), int(teach_panel.call("order_rows"))])
	# A second, identical order is refused with a reason (the toast is not a raw key).
	var again: Dictionary = teach_panel.call("submit")
	_check("teach_refusal_toast", not bool(again.get("accepted", true)) and _readable(toast.text), "%s toast '%s'" % [again, toast.text])
	# K closes and reopens the dialog.
	await _key(KEY_K)
	var closed: bool = not (teach_panel as Control).visible
	await _key(KEY_K)
	_check("k_toggles", closed and (teach_panel as Control).visible and int(teach_panel.call("current_node")) == DOORWAY, "node %d" % int(teach_panel.call("current_node")))
	await _key(KEY_K)
	await _key(KEY_T)
	_check("t_closes", not (tech_panel as Control).visible)
	# Run a little so the lesson is assigned, then the inspector's 지식 tab for the teacher.
	for _step: int in 12:
		SimReads.mutate(_world, "advance_ticks", [120])
		await _frames(2)
		if int((_world.call("get_agent_knowledge", teacher) as Dictionary).get("lesson_role", -1)) >= 0:
			break
	var agents: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var ids: PackedInt64Array = agents.get("agent_ids", PackedInt64Array())
	var entities: PackedInt64Array = agents.get("ids", PackedInt64Array())
	var index: int = ids.find(teacher)
	var bits: int = entities[index] if index >= 0 and index < entities.size() else -1
	inspector.call("display_agent", _world.call("get_agent_detail", bits))
	inspector.call("select_tab", 1)
	await _frames(3)
	var lines: Array = inspector.call("knowledge_texts")
	var raw_lines: int = 0
	for text: String in lines:
		if not _readable(text):
			raw_lines += 1
	var read: Dictionary = _world.call("get_agent_knowledge", teacher)
	_check("inspector_knowledge_tab", int(inspector.call("current_tab")) == 1 and lines.size() >= (read.get("ids", PackedInt64Array()) as PackedInt64Array).size() and raw_lines == 0,
		"lines %d entries %d raw %d: %s" % [lines.size(), (read.get("ids", PackedInt64Array()) as PackedInt64Array).size(), raw_lines, " | ".join(lines)])
	var door_line: String = KnowledgeText.node_name(_locale, DOORWAY)
	var has_door: bool = false
	for text: String in lines:
		if text.begins_with(door_line):
			has_door = true
	_check("inspector_doorway_entry", has_door, door_line)
	print("M4_UI INFO teacher lesson: role %d partner %d node %d" % [int(read.get("lesson_role", -1)), int(read.get("lesson_partner", -1)), int(read.get("lesson_node", -1))])
	if int(read.get("lesson_role", -1)) >= 0:
		var lesson: String = str(lines[0]) if not lines.is_empty() else ""
		_check("inspector_lesson_line", _readable(lesson) and lesson.contains(door_line), "'%s'" % lesson)
	inspector.call("select_tab", 0)
	inspector.call("close")
	# Chronicle kinds 24–28 (ko and en) and the loss toast text.
	var rows: Array[Dictionary] = [
		{"kind": 24, "subject": teacher, "other": -1, "cause": 5, "value": 40},
		{"kind": 25, "subject": learner, "other": teacher, "cause": 4, "value": DOORWAY},
		{"kind": 25, "subject": learner, "other": -1, "cause": 2, "value": 3},
		{"kind": 26, "subject": teacher, "other": -1, "cause": -1, "value": 7},
		{"kind": 27, "subject": teacher, "other": -1, "cause": -1, "value": DOORWAY},
		{"kind": 28, "subject": teacher, "other": -1, "cause": -1, "value": DOORWAY},
		{"kind": 28, "subject": -1, "other": -1, "cause": -1, "value": 23},
	]
	for language: String in ["ko", "en"]:
		_locale.call("set_language", language)
		var texts: Array[String] = []
		var ok: bool = true
		for row: Dictionary in rows:
			var line: String = Chronicle.line(row, -1, _locale, SimReads.read(_world, "get_world_info"), true)
			texts.append(line)
			var label: RichTextLabel = RichTextLabel.new()
			label.bbcode_enabled = true
			label.text = line
			ok = ok and _readable(label.get_parsed_text())
			label.free()
		var toast_text: String = Chronicle.lost_plain(rows[5], _locale)
		texts.append(toast_text)
		ok = ok and _readable(toast_text)
		_check("chronicle_m4_kinds_%s" % language, ok, " || ".join(texts))
	_locale.call("set_language", "ko")
	var notable: Array = (load("res://scripts/ui/panels/chronicle_panel.gd") as GDScript).get_script_constant_map().get("NOTABLE", [])
	_check("chronicle_notable", notable.has(23) and notable.has(24) and notable.has(28), str(notable))
	# F7, then 2: the clay-pit mode; mark the nearest clay bank.
	await _key(KEY_F7)
	var dig_open: bool = bool(bar.call("dig_menu_open")) and int(tool.get("mode")) == 6
	await _key(KEY_2)
	_check("f7_2_clay_mode", dig_open and int(tool.get("mode")) == 9 and not bool(bar.call("dig_menu_open")), "menu %s mode %d hint '%s'" % [dig_open, int(tool.get("mode")), (bar.get("_hint") as Label).text])
	var hearth: Vector2i = _hearth()
	var overlay: Dictionary = SimReads.read(_world, "get_colony_overlay")
	var nxs: PackedInt32Array = overlay.get("node_xs", PackedInt32Array())
	var nys: PackedInt32Array = overlay.get("node_ys", PackedInt32Array())
	var nkinds: PackedInt32Array = overlay.get("node_kinds", PackedInt32Array())
	var banks: Array[Vector2i] = []
	for i: int in mini(nxs.size(), mini(nys.size(), nkinds.size())):
		if nkinds[i] == 2:
			banks.append(Vector2i(nxs[i], nys[i]))
	banks.sort_custom(func(p: Vector2i, q: Vector2i) -> bool: return (p - hearth).length_squared() < (q - hearth).length_squared())
	var ground: Node = _main.get_node_or_null("ColonyView/OverlayLayer")
	_check("clay_banks_drawn", banks.size() > 0 and int(ground.call("node_count")) == nkinds.size(), "banks %d nodes drawn %d of %d" % [banks.size(), int(ground.call("node_count")), nkinds.size()])
	if not banks.is_empty():
		var clay: Dictionary = tool.call("area", banks[0], banks[0])
		var marks: int = 0
		for k: int in SimReads.read(_world, "get_colony_overlay").get("designation_kinds", PackedInt32Array()):
			if k == 4:
				marks += 1
		print("M4_UI INFO clay mark at %s: %s, clay marks %d, toast '%s'" % [banks[0], clay, marks, toast.text])
		_check("clay_mark_toast", _readable(toast.text), "'%s'" % toast.text)
	await _key(KEY_ESCAPE)
	await _key(KEY_ESCAPE)
	# F3: eleven entries, 0 = pottery bench, - = grinding stone.
	await _key(KEY_F3)
	var menu_buttons: Array = bar.get("_menu_buttons")
	var menu_ok: bool = bool(bar.call("build_menu_open")) and menu_buttons.size() == 11
	await _key(KEY_0)
	var bench: String = str(tool.get("build_kind"))
	await _key(KEY_F3)
	await _key(KEY_MINUS)
	_check("build_menu_m4", menu_ok and bench == "pottery_bench" and str(tool.get("build_kind")) == "grinding_stone",
		"buttons %d (%s / %s) kinds %s, %s" % [menu_buttons.size(), (menu_buttons[9] as Button).text if menu_buttons.size() > 10 else "", (menu_buttons[10] as Button).text if menu_buttons.size() > 10 else "",
		bench, str(tool.get("build_kind"))])
	await _key(KEY_ESCAPE)
	# The hearth stays drawn; its fire_pot workshop opens the workshop panel; a fire_pot bill waits.
	var buildings: Node = _main.get_node_or_null("ColonyView/BuildingLayer")
	_check("hearth_drawn", bool(buildings.get("_has_hearth")), "")
	var shops: Dictionary = SimReads.read(_world, "get_workshops")
	var shop_ids: PackedInt32Array = shops.get("ids", PackedInt32Array())
	var shop_kinds: PackedInt32Array = shops.get("kinds", PackedInt32Array())
	var fire_shop: int = -1
	for i: int in mini(shop_ids.size(), shop_kinds.size()):
		if shop_kinds[i] == 11:
			fire_shop = shop_ids[i]
	shop_panel.call("display_workshop", fire_shop)
	await _frames(2)
	var title: Label = shop_panel.get("_title")
	_check("hearth_workshop_panel", fire_shop >= 0 and (shop_panel as Control).visible and _readable(title.text), "shop %d title '%s'" % [fire_shop, title.text])
	shop_panel.call("_add_bill", "fire_pot")
	await _frames(3)
	var waits: Dictionary = SimReads.read(_world, "get_knowledge_waits")
	var wait_label: Label = (shop_panel as Control).find_child("Wait", true, false) as Label
	var want_wait: String = KnowledgeText.wait_line(_locale, POT_FIRING)
	_check("bill_wait_line", wait_label != null and wait_label.text == want_wait, "'%s' (want '%s', waits %s)" % [wait_label.text if wait_label != null else "", want_wait, waits])
	shop_panel.call("close")
	# A stone-wall site waits for stone walling: the why? panel says so.
	var site: Vector2i = Vector2i(-1, -1)
	for r: int in range(3, 12):
		for off: Vector2i in [Vector2i(r, 0), Vector2i(-r, 0), Vector2i(0, r), Vector2i(0, -r), Vector2i(r, r), Vector2i(-r, -r)]:
			var p: Dictionary = _world.call("get_place_preview", "stone_wall", hearth.x + off.x, hearth.y + off.y, hearth.x + off.x, hearth.y + off.y, 0)
			var ok: PackedByteArray = p.get("ok", PackedByteArray())
			if site.x < 0 and ok.size() == 1 and ok[0] == 1:
				site = hearth + off
	tool.call("set_build", "stone_wall")
	var placed: Dictionary = tool.call("place_drag", site, site, false)
	tool.call("set_mode", 0)
	await _frames(2)
	causal.call("display_history", [], site.x, site.y)
	var why: Label = (causal as Control).find_child("Wait", true, false) as Label
	var want_why: String = KnowledgeText.wait_line(_locale, 13)
	_check("site_why_line", bool(placed.get("accepted", false)) and why != null and why.text == want_why, "site %s %s why '%s' (want '%s')" % [site, placed, why.text if why != null else "", want_why])
	# P: the priority table's Pottery column.
	await _key(KEY_P)
	await _frames(2)
	var priorities: Control = _main.get_node_or_null("UI/PriorityPanel") as Control
	var grid: GridContainer = null
	for node: Node in priorities.find_children("*", "GridContainer", true, false):
		grid = node as GridContainer
	var works: PackedStringArray = SimReads.read(_world, "get_work_priorities").get("work_types", PackedStringArray())
	var pottery_title: Label = priorities.find_child("PotteryTitle", true, false) as Label
	_check("priority_pottery_column", grid != null and grid.columns == works.size() + 2 and pottery_title != null and _readable(pottery_title.text),
		"columns %d works %d title '%s' width %d" % [grid.columns if grid != null else -1, works.size(), pottery_title.text if pottery_title != null else "", int(grid.get_combined_minimum_size().x) if grid != null else -1])
	await _key(KEY_P)
	_finish()


func _finish() -> void:
	print("M4_UI_DONE failures=%d %s" % [_failures.size(), ",".join(_failures)])
	quit(0 if _failures.is_empty() else 1)
