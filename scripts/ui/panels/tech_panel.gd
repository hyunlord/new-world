# tech_panel.gd (UI/TechPanel, created by hud_topbar.gd beside itself): the M4 tech map, toggled with T and opened by
#   the HUD's "⚠ 기술 위험 N" badge (goal 9.2).
# Lanes as rows (생존 · 제작 · 건설 · 채굴 · 토기 · 사회, get_tech `lanes` 0–5), eras as columns (구석기 | 신석기), each
#   node a card: its name, its practitioners ("3명") and ◆ with the marker weight for an era marker, bordered in its
#   status colour (미지 grey, 위험 red, 알려짐 yellow, 안정 green, 소실 a black border on a darker card). Prerequisite
#   edges (`prereq_from` → `prereq_to`) are lines drawn under the cards. A click on a card shows its detail on the
#   right (get_tech_node): status and threshold, the holders with proficiency bars (the threshold marked; "가르치기
#   숙련" for those who know instruction), who could learn it, the discovery conditions (● met / ○ not yet), what it
#   gates (recipes, buildings, marks, channels, lore), how many orders wait for it (get_knowledge_waits), and a
#   가르치기 button that opens the teach dialog (UI/TeachPanel) on this node. The cards are rebuilt only when the
#   statuses or counts change; the detail is read again when the selection, the tech read or the hour changes.
#   Esc, 닫기 or a world reset closes it.
# M5.5 (goal §9.4): a card whose node an artifact the colony holds hints at (get_neighbours `clue_nodes`) carries the
#   eureka glyph ★ (legend "★ 단서 (유레카)"); the detail adds "★ 단서: 토기 — 발견 확률 ×3" (`clue_items`) and one line
#   "이웃 붉은바위가 앎 (소문)" per neighbour whose `rumoured` bits hold the node.
# Bridge methods: get_tech_node (direct, for the selected node); via SimReads get_tech, get_knowledge_waits,
#   get_time_snapshot, get_neighbours; signal world_reset.
extends Control

const PanelTooltips = preload("res://scripts/ui/panels/panel_tooltips.gd")

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const KnowledgeText = preload("res://scripts/ui/knowledge_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const NeighbourText = preload("res://scripts/ui/neighbour_text.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const TOGGLE_KEY: Key = KEY_T
## M5.5: the eureka glyph and its colour; the rumour lines' colour.
const EUREKA_GLYPH: String = "★"
const EUREKA: Color = Color("f0c040")
const RUMOUR: Color = Color("a8c8f0")
const PANEL_RECT: Rect2 = Rect2(12, 124, 1896, 830)
const CARD_SIZE: Vector2 = Vector2(196, 64)
const CARDS_PER_ROW: int = 3
const LANE_WIDTH: int = 120
const DETAIL_WIDTH: int = 440
const GAP: int = 8
const MUTED: Color = Color("c8b89a")
const TEXT: Color = Color("f2e6c8")
const CARD_BG: Color = Color(0.13, 0.115, 0.1, 1.0)
const LOST_BG: Color = Color(0.06, 0.055, 0.05, 1.0)
const EDGE: Color = Color(0.78, 0.72, 0.6, 0.55)
## Re-read the detail at least every this many ticks (proficiency moves slowly).
const DETAIL_TICKS: int = 60
## Learner names listed before "외 n명".
const LEARNERS_SHOWN: int = 8

var _world_sim: Node = null
var _locale: Node = null
var _title: Label
var _hint: Label
var _close: Button
var _legend: HBoxContainer
var _canvas: MarginContainer
var _grid: GridContainer
var _cards: Dictionary = {}
var _edges: Array[Vector2i] = []
var _key: int = 0
var _selected: int = -1
var _detail: VBoxContainer
var _detail_key: Array = []
var _detail_read: Dictionary = {}
var _teach: Button
## M5.5: the rumour and clue lines of the detail shown (for tests and tools).
var _neighbour_lines: Array[String] = []


func _ready() -> void:
	_locale = get_node_or_null("/root/Locale")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", close)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	position = PANEL_RECT.position
	size = PANEL_RECT.size
	var panel: PanelContainer = ArtUI.panel(self, Rect2(Vector2.ZERO, PANEL_RECT.size))
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", GAP)
	panel.add_child(column)
	var head: HBoxContainer = HBoxContainer.new()
	head.add_theme_constant_override("separation", 16)
	column.add_child(head)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	head.add_child(_title)
	_hint = Label.new()
	_hint.add_theme_color_override("font_color", MUTED)
	_hint.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(_hint)
	_close = Button.new()
	PanelTooltips.attach(_close, "close")
	_close.name = "Close"
	_close.focus_mode = Control.FOCUS_NONE
	_close.pressed.connect(close)
	head.add_child(_close)
	_legend = HBoxContainer.new()
	_legend.add_theme_constant_override("separation", 24)
	column.add_child(_legend)
	for code: int in KnowledgeText.STATUS_KEYS.size():
		var swatch: Label = Label.new()
		swatch.add_theme_color_override("font_color", KnowledgeText.status_color(code) if code != 4 else MUTED)
		_legend.add_child(swatch)
	var marker: Label = Label.new()
	marker.add_theme_color_override("font_color", Color("e8c46e"))
	_legend.add_child(marker)
	var eureka: Label = Label.new()
	eureka.name = "EurekaLegend"
	eureka.add_theme_color_override("font_color", EUREKA)
	_legend.add_child(eureka)
	var body: HBoxContainer = HBoxContainer.new()
	body.add_theme_constant_override("separation", 16)
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	column.add_child(body)
	var map_scroll: ScrollContainer = ScrollContainer.new()
	map_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	map_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	map_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body.add_child(map_scroll)
	_canvas = MarginContainer.new()
	_canvas.name = "Map"
	_canvas.draw.connect(_draw_edges)
	map_scroll.add_child(_canvas)
	_grid = GridContainer.new()
	_grid.columns = 1 + KnowledgeText.ERAS
	_grid.add_theme_constant_override("h_separation", 16)
	_grid.add_theme_constant_override("v_separation", 12)
	_canvas.add_child(_grid)
	var detail_scroll: ScrollContainer = ScrollContainer.new()
	detail_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	detail_scroll.custom_minimum_size.x = DETAIL_WIDTH
	detail_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body.add_child(detail_scroll)
	_detail = VBoxContainer.new()
	_detail.name = "Detail"
	_detail.add_theme_constant_override("separation", 6)
	_detail.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	detail_scroll.add_child(_detail)
	_teach = Button.new()
	PanelTooltips.attach(_teach, "tech")
	_teach.name = "Teach"
	_teach.focus_mode = Control.FOCUS_NONE
	_teach.pressed.connect(_open_teach)
	_detail.add_child(_teach)
	_teach.visible = false


func _loc(key: String) -> String:
	return KnowledgeText.ltr(_locale, key)


## Open the map (the HUD badge's action).
func open() -> void:
	visible = true
	_key = 0
	_detail_key = []
	_refresh()


func close() -> void:
	visible = false


## Show or hide the map (T).
func toggle_visible() -> void:
	if visible:
		close()
	else:
		open()


func _unhandled_input(event: InputEvent) -> void:
	if not (event is InputEventKey and event.pressed and not event.echo):
		return
	var key: InputEventKey = event
	if key.keycode == TOGGLE_KEY:
		toggle_visible()
		get_viewport().set_input_as_handled()
	elif key.keycode == KEY_ESCAPE and visible:
		close()
		get_viewport().set_input_as_handled()


func _process(_delta: float) -> void:
	if visible:
		_refresh()
		_canvas.queue_redraw()


func _refresh() -> void:
	if _world_sim == null:
		return
	var tech: Dictionary = SimReads.read(_world_sim, "get_tech")
	var neighbours: Dictionary = _neighbours()
	var key: int = hash([tech.get("statuses", PackedInt64Array()), tech.get("practitioners", PackedInt64Array()),
		tech.get("holders", PackedInt64Array()), KnowledgeText.language(_locale), neighbours.get("clue_nodes", PackedInt32Array()),
		neighbours.get("rumoured", PackedInt64Array())])
	if key != _key:
		_key = key
		_rebuild(tech)
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", 0))
	@warning_ignore("integer_division")
	var detail_key: Array = [_selected, key, tick / DETAIL_TICKS]
	if detail_key != _detail_key:
		_detail_key = detail_key
		_fill_detail(tech)


func _rebuild(tech: Dictionary) -> void:
	_title.text = _loc("M4_TECH_TITLE")
	_hint.text = _loc("M4_TECH_HINT")
	_close.text = _loc("UI_CLOSE")
	_teach.text = _loc("M4_TECH_TEACH")
	for code: int in KnowledgeText.STATUS_KEYS.size():
		var swatch: Label = _legend.get_child(code) as Label
		swatch.text = ("□ " if code == 4 else "■ ") + KnowledgeText.status(_locale, code)
	(_legend.get_child(KnowledgeText.STATUS_KEYS.size()) as Label).text = "◆ " + _loc("M4_TECH_MARKER_LEGEND")
	(_legend.get_child(KnowledgeText.STATUS_KEYS.size() + 1) as Label).text = EUREKA_GLYPH + " " + _loc("M55_TECH_EUREKA_LEGEND")
	for child: Node in _grid.get_children():
		_grid.remove_child(child)
		child.queue_free()
	_cards.clear()
	_grid.add_child(_cell_label(""))
	for era: int in KnowledgeText.ERAS:
		var head: Label = _cell_label(KnowledgeText.era_column(_locale, era))
		head.theme_type_variation = &"Heading"
		_grid.add_child(head)
	var rows: Array[Dictionary] = KnowledgeText.nodes(tech)
	for lane: int in KnowledgeText.LANES:
		var lane_label: Label = _cell_label(KnowledgeText.lane(_locale, lane))
		lane_label.custom_minimum_size.x = LANE_WIDTH
		lane_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		_grid.add_child(lane_label)
		for era: int in KnowledgeText.ERAS:
			var cell: HFlowContainer = HFlowContainer.new()
			cell.name = "Cell_%d_%d" % [lane, era]
			cell.add_theme_constant_override("h_separation", GAP)
			cell.add_theme_constant_override("v_separation", GAP)
			cell.custom_minimum_size.x = CARDS_PER_ROW * CARD_SIZE.x + (CARDS_PER_ROW - 1) * GAP
			_grid.add_child(cell)
			for row: Dictionary in rows:
				if int(row["lane"]) == lane and int(row["era"]) == era:
					var card: Button = _card(row, tech)
					cell.add_child(card)
					_cards[int(row["id"])] = card
	_edges.clear()
	var froms: PackedInt64Array = tech.get("prereq_from", PackedInt64Array())
	var tos: PackedInt64Array = tech.get("prereq_to", PackedInt64Array())
	for i: int in mini(froms.size(), tos.size()):
		_edges.append(Vector2i(int(froms[i]), int(tos[i])))
	_canvas.queue_redraw()


func _cell_label(text: String) -> Label:
	var label: Label = Label.new()
	label.text = text
	label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return label


## One node card: name, practitioners and the marker weight, bordered in the status colour.
func _card(row: Dictionary, tech: Dictionary) -> Button:
	var id: int = int(row["id"])
	var code: int = int(row["status"])
	var card: Button = Button.new()
	card.name = "Card_%d" % id
	card.set_meta("status", code)
	card.focus_mode = Control.FOCUS_NONE
	card.toggle_mode = true
	card.button_pressed = id == _selected
	card.custom_minimum_size = CARD_SIZE
	card.tooltip_text = "%s · %s" % [KnowledgeText.node_name(_locale, id, tech), KnowledgeText.status(_locale, code)]
	PanelTooltips.attach(card, "tech")
	card.pressed.connect(select_node.bind(id))
	var colour: Color = KnowledgeText.status_color(code)
	for state: String in ["normal", "hover", "pressed", "hover_pressed", "disabled"]:
		var box: StyleBoxFlat = StyleBoxFlat.new()
		box.bg_color = LOST_BG if code == 4 else CARD_BG
		if state == "hover" or state == "hover_pressed":
			box.bg_color = box.bg_color.lightened(0.08)
		box.border_color = colour
		var width: int = 4 if state.begins_with("pressed") or state == "hover_pressed" else (3 if code == 4 else 2)
		box.set_border_width_all(width)
		box.content_margin_left = 8
		box.content_margin_right = 8
		box.content_margin_top = 4
		box.content_margin_bottom = 4
		card.add_theme_stylebox_override(state, box)
	var lines: VBoxContainer = VBoxContainer.new()
	lines.mouse_filter = Control.MOUSE_FILTER_IGNORE
	lines.set_anchors_preset(Control.PRESET_FULL_RECT)
	lines.offset_left = 8
	lines.offset_top = 4
	lines.offset_right = -8
	lines.offset_bottom = -4
	lines.add_theme_constant_override("separation", 0)
	card.add_child(lines)
	var name_label: Label = _cell_label(KnowledgeText.node_name(_locale, id, tech))
	name_label.name = "Name"
	name_label.add_theme_color_override("font_color", MUTED if code == 0 or code == 4 else TEXT)
	name_label.clip_text = true
	lines.add_child(name_label)
	var sub: String = KnowledgeText.trf(_locale, "M4_TECH_PRACTITIONERS_FMT", {"count": int(row["practitioners"])})
	if int(row["marker"]) > 0:
		sub += "  ◆%d" % int(row["marker"])
	var clue: bool = NeighbourText.clue_item(_neighbours(), id) >= 0
	if clue:
		sub += "  " + EUREKA_GLYPH
	card.set_meta("eureka", clue)
	var count_label: Label = _cell_label(sub)
	count_label.name = "Count"
	count_label.add_theme_color_override("font_color", colour if code != 4 else MUTED)
	lines.add_child(count_label)
	return card


## M5.5: the get_neighbours read ({} without the bridge read).
func _neighbours() -> Dictionary:
	if _world_sim == null or not _world_sim.has_method("get_neighbours"):
		return {}
	return SimReads.read(_world_sim, "get_neighbours")


## Prerequisite lines between card centres, under the cards.
func _draw_edges() -> void:
	var origin: Vector2 = _canvas.global_position
	for edge: Vector2i in _edges:
		var a: Button = _cards.get(edge.x, null)
		var b: Button = _cards.get(edge.y, null)
		if a == null or b == null or not a.is_visible_in_tree() or not b.is_visible_in_tree():
			continue
		var from: Vector2 = a.get_global_rect().get_center() - origin
		var to: Vector2 = b.get_global_rect().get_center() - origin
		_canvas.draw_line(from, to, EDGE, 2.0)


## Show node `id`'s detail (a card's click; also used by tools).
func select_node(id: int) -> void:
	_selected = id
	for card_id: int in _cards.keys():
		(_cards[card_id] as Button).set_pressed_no_signal(card_id == id)
	_detail_key = []
	if visible:
		_refresh()


## The node shown in the detail (-1 none).
func selected_node() -> int:
	return _selected


## The card of node `id` (null when the map has not been built).
func card(id: int) -> Button:
	return _cards.get(id, null)


## The last get_tech_node read of the detail.
func detail() -> Dictionary:
	return _detail_read


func _fill_detail(tech: Dictionary) -> void:
	for child: Node in _detail.get_children():
		if child != _teach:
			_detail.remove_child(child)
			child.queue_free()
	_teach.visible = false
	_neighbour_lines.clear()
	if _selected < 0:
		_detail_read = {}
		_add_line(_loc("M4_TECH_SELECT_HINT"), MUTED)
		return
	var read: Variant = _world_sim.call("get_tech_node", _selected)
	_detail_read = read if read is Dictionary else {}
	if not bool(_detail_read.get("found", false)):
		_add_line(_loc("M4_TECH_SELECT_HINT"), MUTED)
		return
	var code: int = int(_detail_read.get("status", 0))
	var heading: Label = _add_line(KnowledgeText.node_name(_locale, _selected, tech), TEXT)
	heading.theme_type_variation = &"Heading"
	@warning_ignore("integer_division")
	var threshold_percent: int = int(_detail_read.get("threshold", 0)) / 10
	_add_line(KnowledgeText.trf(_locale, "M4_TECH_STATUS_FMT", {"status": KnowledgeText.status(_locale, code),
		"threshold": threshold_percent, "holders": int(_detail_read.get("holders", 0)),
		"practitioners": int(_detail_read.get("practitioners", 0))}), KnowledgeText.status_color(code) if code != 4 else MUTED)
	var lane_era: String = KnowledgeText.trf(_locale, "M4_TECH_LANE_ERA_FMT", {"lane": KnowledgeText.lane(_locale, int(_detail_read.get("lane", 0))),
		"era": KnowledgeText.era_column(_locale, int(_detail_read.get("era", 0)))})
	if int(_detail_read.get("marker_weight", 0)) > 0:
		lane_era += " · " + KnowledgeText.trf(_locale, "M4_TECH_MARKER_FMT", {"weight": int(_detail_read.get("marker_weight", 0))})
	_add_line(lane_era, MUTED)
	_neighbour_info()
	_holders(tech)
	_learners()
	_conditions()
	_gates()
	var waiting: int = 0
	for row: Dictionary in KnowledgeText.waits(SimReads.read(_world_sim, "get_knowledge_waits")):
		if int(row["node"]) == _selected:
			waiting += 1
	if waiting > 0:
		_add_line(KnowledgeText.trf(_locale, "M4_TECH_WAITING_FMT", {"count": waiting}), KnowledgeText.status_color(1))
	_detail.move_child(_teach, -1)
	_teach.visible = true


## M5.5: the clue line (an artifact the colony holds hints at this node) and one rumour line per neighbour knowing it.
func _neighbour_info() -> void:
	_neighbour_lines.clear()
	var read: Dictionary = _neighbours()
	var item: int = NeighbourText.clue_item(read, _selected)
	if item >= 0:
		var clue: Label = _add_line(KnowledgeText.trf(_locale, "M55_TECH_EUREKA_FMT", {"item": EconomyText.item(_locale, item)}), EUREKA)
		clue.name = "Eureka"
		_neighbour_lines.append(clue.text)
	for id: int in NeighbourText.rumour_holders(read, _selected):
		var line: Label = _add_line(NeighbourText.rumour_line(_locale, read, id), RUMOUR)
		line.name = "Rumour_%d" % id
		_neighbour_lines.append(line.text)


## M5.5: the detail's rumour and clue lines (for tests and tools).
func neighbour_lines() -> Array[String]:
	return _neighbour_lines


func _add_line(text: String, colour: Color) -> Label:
	var label: Label = Label.new()
	label.text = text
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.custom_minimum_size.x = DETAIL_WIDTH - 24
	label.add_theme_color_override("font_color", colour)
	_detail.add_child(label)
	return label


func _heading(key: String) -> void:
	var label: Label = _add_line(_loc(key), TEXT)
	label.theme_type_variation = &"Heading"


func _holders(_tech: Dictionary) -> void:
	_heading("M4_TECH_HOLDERS")
	var ids: PackedInt64Array = _detail_read.get("holder_ids", PackedInt64Array())
	var levels: PackedInt64Array = _detail_read.get("holder_proficiency", PackedInt64Array())
	var teaches: PackedInt64Array = _detail_read.get("holder_instruction", PackedInt64Array())
	var threshold: int = int(_detail_read.get("threshold", 0))
	if ids.is_empty():
		_add_line(_loc("M4_TECH_NO_HOLDERS"), MUTED)
		return
	var language: String = KnowledgeText.language(_locale)
	for i: int in mini(ids.size(), levels.size()):
		var name: String = Names.present(int(ids[i]), language)
		if i < teaches.size() and teaches[i] != 0:
			name += " · " + _loc("M4_TECH_INSTRUCTOR")
		_add_line(name, TEXT)
		_detail.add_child(_bar_row(int(levels[i]), threshold))


## A proficiency bar with its percent ("65 %").
func _bar_row(value: int, threshold: int) -> HBoxContainer:
	var row: HBoxContainer = HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.add_child(ArtUI.proficiency_bar(value, threshold, 280))
	var label: Label = Label.new()
	@warning_ignore("integer_division")
	var percent: int = value / 10
	label.text = KnowledgeText.trf(_locale, "M4_PERCENT_FMT", {"percent": percent})
	label.add_theme_color_override("font_color", MUTED)
	row.add_child(label)
	return row


func _learners() -> void:
	_heading("M4_TECH_LEARNERS")
	var ids: PackedInt64Array = _detail_read.get("learner_ids", PackedInt64Array())
	if ids.is_empty():
		_add_line(_loc("M4_TEACH_NO_LEARNER"), MUTED)
		return
	var language: String = KnowledgeText.language(_locale)
	var names: Array[String] = []
	for i: int in mini(ids.size(), LEARNERS_SHOWN):
		names.append(Names.present(int(ids[i]), language))
	var text: String = ", ".join(names)
	if ids.size() > LEARNERS_SHOWN:
		text += " " + KnowledgeText.trf(_locale, "M4_TECH_MORE_FMT", {"count": ids.size() - LEARNERS_SHOWN})
	_add_line(text, MUTED)


func _conditions() -> void:
	_heading("M4_TECH_DISCOVERY")
	var met: PackedInt64Array = _detail_read.get("cond_met", PackedInt64Array())
	if met.is_empty():
		_add_line(_loc("M4_TECH_NO_DISCOVERY"), MUTED)
		return
	for i: int in met.size():
		var ok: bool = met[i] != 0
		_add_line(("● " if ok else "○ ") + KnowledgeText.condition(_locale, _detail_read, i), KnowledgeText.status_color(3) if ok else MUTED)


func _gates() -> void:
	_heading("M4_TECH_GATES")
	var kinds: PackedInt64Array = _detail_read.get("gate_kinds", PackedInt64Array())
	if kinds.is_empty():
		_add_line(_loc("M4_GATE_NONE"), MUTED)
		return
	for i: int in kinds.size():
		_add_line(KnowledgeText.gate(_locale, _detail_read, i), TEXT)


func _open_teach() -> void:
	var dialog: Node = get_parent().get_node_or_null("TeachPanel") if get_parent() != null else null
	if dialog != null and dialog.has_method("open_for"):
		dialog.call("open_for", _selected)
