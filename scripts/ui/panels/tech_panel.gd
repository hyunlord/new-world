# tech_panel.gd (UI/TechPanel, created by hud_topbar.gd beside itself): the M4 tech map, toggled with T and opened by
#   the HUD's "⚠ 기술 위험 N" badge (goal 9.2).
# Data-authored lanes as rows and eras as columns (M-SPINE: 10 × 4), each
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
const CARD_SIZE: Vector2 = Vector2(264, 88)
const LANE_WIDTH: int = 160
const DETAIL_WIDTH: int = 392
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
var _headers: GridContainer
var _map_scroll: ScrollContainer
var _jump: OptionButton
var _lane_labels: Dictionary = {}
var _grid: GridContainer
var _cards: Dictionary = {}
var _edges: Array[Vector2i] = []
var _chain: Dictionary = {}
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
	var inert_legend: Label = Label.new()
	inert_legend.name = "InertLegend"
	inert_legend.add_theme_color_override("font_color", MUTED)
	_legend.add_child(inert_legend)
	var body: HBoxContainer = HBoxContainer.new()
	body.add_theme_constant_override("separation", 16)
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	column.add_child(body)
	var map_column: VBoxContainer = VBoxContainer.new()
	map_column.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(map_column)
	_jump = OptionButton.new()
	_jump.name = "LaneJump"
	_jump.item_selected.connect(_jump_lane)
	map_column.add_child(_jump)
	_headers = GridContainer.new()
	_headers.name = "EraHeaders"
	_headers.add_theme_constant_override("h_separation", 16)
	map_column.add_child(_headers)
	var map_scroll: ScrollContainer = ScrollContainer.new()
	_map_scroll = map_scroll
	map_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	map_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	map_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	map_column.add_child(map_scroll)
	_canvas = MarginContainer.new()
	_canvas.name = "Map"
	_canvas.draw.connect(_draw_edges)
	map_scroll.add_child(_canvas)
	_grid = GridContainer.new()
	_grid.columns = 1
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
	_detail.custom_minimum_size.x = DETAIL_WIDTH - 16
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
		neighbours.get("rumoured", PackedInt64Array()), neighbours.get("rumoured_high", PackedInt64Array()), tech.get("version", 0), tech.get("eras", []),
		tech.get("lanes", []), tech.get("connected", PackedInt64Array()), tech.get("debts", PackedStringArray())])
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
	(_legend.get_child(KnowledgeText.STATUS_KEYS.size() + 2) as Label).text = "▧ " + _loc("TECH_STATUS_INERT")
	for child: Node in _headers.get_children():
		_headers.remove_child(child)
		child.queue_free()
	var jump_lane_id: int = _jump.get_selected_id()
	_jump.clear()
	_lane_labels.clear()
	for child: Node in _grid.get_children():
		_grid.remove_child(child)
		child.queue_free()
	_cards.clear()
	var eras: Array[Dictionary] = KnowledgeText.catalogue(tech, "eras")
	var lanes: Array[Dictionary] = KnowledgeText.catalogue(tech, "lanes")
	_grid.columns = 1 + eras.size()
	_headers.columns = _grid.columns
	var corner: Label = _cell_label("")
	corner.custom_minimum_size.x = LANE_WIDTH
	_headers.add_child(corner)
	for era: Dictionary in eras:
		_headers.add_child(_era_header(era))
	var rows: Array[Dictionary] = KnowledgeText.nodes(tech)
	for lane: Dictionary in lanes:
		var lane_order: int = int(lane.get("order", 0))
		var lane_label: Label = _cell_label(_loc(str(lane.get("name_key", "UI_UNKNOWN"))))
		lane_label.custom_minimum_size.x = LANE_WIDTH
		lane_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		lane_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		_grid.add_child(lane_label)
		_lane_labels[lane_order] = lane_label
		_jump.add_item(lane_label.text, lane_order)
		for era: Dictionary in eras:
			var era_order: int = int(era.get("order", 0))
			var cell: PanelContainer = PanelContainer.new()
			cell.name = "Cell_%d_%d" % [lane_order, era_order]
			cell.custom_minimum_size = Vector2(CARD_SIZE.x, 188)
			var boundary: StyleBoxFlat = StyleBoxFlat.new()
			boundary.bg_color = Color(CARD_BG, 0.3)
			boundary.border_color = Color(MUTED, 0.15)
			boundary.set_border_width_all(1)
			cell.add_theme_stylebox_override("panel", boundary)
			_grid.add_child(cell)
			var stack_scroll: ScrollContainer = ScrollContainer.new()
			stack_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
			cell.add_child(stack_scroll)
			var stack: VBoxContainer = VBoxContainer.new()
			stack.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			stack.add_theme_constant_override("separation", GAP)
			stack_scroll.add_child(stack)
			for row: Dictionary in rows:
				if int(row["lane"]) == lane_order and int(row["era"]) == era_order:
					var node_card: Button = _card(row, tech)
					node_card.custom_minimum_size.x = CARD_SIZE.x - 16
					stack.add_child(node_card)
					_cards[int(row["id"])] = node_card
	_jump.select(maxi(0, _jump.get_item_index(jump_lane_id)))
	_edges.clear()
	var froms: PackedInt64Array = tech.get("prereq_from", PackedInt64Array())
	var tos: PackedInt64Array = tech.get("prereq_to", PackedInt64Array())
	for i: int in mini(froms.size(), tos.size()):
		_edges.append(Vector2i(int(froms[i]), int(tos[i])))
	_update_chain()
	_canvas.queue_redraw()


## Catalogue thresholds are marked independently; adding an era needs no UI code.
func _era_header(era: Dictionary) -> VBoxContainer:
	var column: VBoxContainer = VBoxContainer.new()
	column.name = "Era_%s" % str(era.get("id", ""))
	column.custom_minimum_size.x = CARD_SIZE.x
	column.add_theme_constant_override("separation", GAP)
	var title: Label = _cell_label(_loc(str(era.get("name_key", "UI_UNKNOWN"))))
	title.theme_type_variation = &"Heading"
	title.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	column.add_child(title)
	var progress: int = int(era.get("progress", 0))
	var value: Label = _cell_label("%d ‰" % progress)
	value.name = "Progress"
	value.theme_type_variation = &"SmallNumber"
	column.add_child(value)
	var transition: int = int(era.get("transition_permille", 0))
	var reached: int = int(era.get("era_permille", 0))
	var bar: ProgressBar = ArtUI.proficiency_bar(progress, reached, int(CARD_SIZE.x))
	bar.name = "EraGauge"
	bar.tooltip_text = "%d ‰ / %d ‰" % [transition, reached]
	bar.mouse_filter = Control.MOUSE_FILTER_PASS
	var mark: ColorRect = ColorRect.new()
	mark.name = "TransitionThreshold"
	mark.color = MUTED
	mark.mouse_filter = Control.MOUSE_FILTER_IGNORE
	mark.anchor_left = float(transition) / 1000.0
	mark.anchor_right = mark.anchor_left
	mark.anchor_bottom = 1.0
	mark.offset_left = -1.0
	mark.offset_right = 1.0
	mark.offset_top = -2.0
	mark.offset_bottom = 2.0
	bar.add_child(mark)
	column.add_child(bar)
	return column


func reset_overview() -> void:
	_map_scroll.scroll_vertical = 0
	_map_scroll.scroll_horizontal = 0
	_jump.select(0)
	for cell: Node in _grid.get_children():
		if cell is PanelContainer:
			(cell.get_child(0) as ScrollContainer).scroll_vertical = 0


func _jump_lane(index: int) -> void:
	var lane_id: int = _jump.get_item_id(index)
	var label: Control = _lane_labels.get(lane_id)
	if label != null:
		_map_scroll.ensure_control_visible(label)


func _cell_label(text: String) -> Label:
	var label: Label = Label.new()
	label.text = text
	label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	return label


## One node card: name, practitioners and the marker weight, bordered in the status colour.
func _card(row: Dictionary, tech: Dictionary) -> Button:
	var id: int = int(row["id"])
	var code: int = int(row["status"])
	var inert: bool = not bool(row.get("connected", true))
	var card: Button = Button.new()
	card.name = "Card_%d" % id
	card.set_meta("status", code)
	card.set_meta("inert", inert)
	card.focus_mode = Control.FOCUS_NONE
	card.toggle_mode = true
	card.button_pressed = id == _selected
	card.custom_minimum_size = CARD_SIZE
	card.tooltip_text = "%s · %s" % [KnowledgeText.node_name(_locale, id, tech),
		_loc("TECH_STATUS_INERT") if inert else KnowledgeText.status(_locale, code)]
	if inert:
		card.tooltip_text += "\n" + str(row.get("debt", ""))
		var hatch: Control = Control.new()
		hatch.name = "Hatch"
		hatch.mouse_filter = Control.MOUSE_FILTER_IGNORE
		hatch.set_anchors_preset(Control.PRESET_FULL_RECT)
		hatch.draw.connect(_draw_hatch.bind(hatch))
		card.add_child(hatch)
	PanelTooltips.attach(card, "tech")
	card.pressed.connect(select_node.bind(id))
	var colour: Color = MUTED if inert else KnowledgeText.status_color(code)
	for state: String in ["normal", "hover", "pressed", "hover_pressed", "disabled"]:
		var box: StyleBoxFlat = StyleBoxFlat.new()
		box.bg_color = LOST_BG if code == 4 or inert else CARD_BG
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
	name_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	name_label.custom_minimum_size.y = 48
	name_label.max_lines_visible = 2
	lines.add_child(name_label)
	var sub: String = _loc("TECH_STATUS_INERT") if inert else KnowledgeText.trf(_locale, "M4_TECH_PRACTITIONERS_FMT", {"count": int(row["practitioners"])})
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
		if a == null or b == null or not _card_on_screen(a) or not _card_on_screen(b):
			continue
		var from: Vector2 = a.get_global_rect().get_center() - origin
		var to: Vector2 = b.get_global_rect().get_center() - origin
		var highlighted: bool = _chain.has(edge.x) and _chain.has(edge.y)
		if not highlighted:
			continue
		_canvas.draw_line(from, to, EUREKA, 3.0)


func _card_on_screen(node_card: Button) -> bool:
	if not node_card.is_visible_in_tree():
		return false
	var centre: Vector2 = node_card.get_global_rect().get_center()
	var ancestor: Node = node_card.get_parent()
	while ancestor != null:
		if ancestor is ScrollContainer and not ancestor.get_global_rect().has_point(centre):
			return false
		ancestor = ancestor.get_parent()
	return true


## Hatching stays beneath the readable name and status label.
func _draw_hatch(node_card: Control) -> void:
	var bounds: Rect2 = Rect2(Vector2(4, 4), node_card.size - Vector2(8, 8))
	var ink: Color = Color(MUTED, 0.12)
	var offset: float = -bounds.size.y
	while offset < bounds.size.x:
		var from: Vector2 = Vector2(maxf(offset, 0.0), maxf(-offset, 0.0))
		var to: Vector2 = Vector2(minf(offset + bounds.size.y, bounds.size.x), minf(bounds.size.y, bounds.size.x - offset))
		node_card.draw_line(bounds.position + from, bounds.position + to, ink, 2.0)
		offset += 12.0


## Walk all ancestors once, preserving data-owned ids and prerequisite links.
func _update_chain() -> void:
	_chain.clear()
	if _selected >= 0:
		var pending: Array[int] = [_selected]
		while not pending.is_empty():
			var node_id: int = pending.pop_back()
			if _chain.has(node_id):
				continue
			_chain[node_id] = true
			for edge: Vector2i in _edges:
				if edge.y == node_id:
					pending.append(edge.x)
	for card_id: int in _cards:
		var node_card: Button = _cards[card_id]
		node_card.modulate = Color(1, 1, 1, 1.0 if _selected < 0 or _chain.has(card_id) else 0.5)
	_canvas.queue_redraw()


## Show node `id`'s detail (a card's click; also used by tools).
func select_node(id: int) -> void:
	_selected = id
	for card_id: int in _cards.keys():
		(_cards[card_id] as Button).set_pressed_no_signal(card_id == id)
	_update_chain()
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
	var lane_era: String = KnowledgeText.trf(_locale, "M4_TECH_LANE_ERA_FMT", {"lane": KnowledgeText.lane(_locale, int(_detail_read.get("lane", 0)), tech),
		"era": KnowledgeText.era_column(_locale, int(_detail_read.get("era", 0)), tech)})
	if int(_detail_read.get("marker_weight", 0)) > 0:
		lane_era += " · " + KnowledgeText.trf(_locale, "M4_TECH_MARKER_FMT", {"weight": int(_detail_read.get("marker_weight", 0))})
	_add_line(lane_era, MUTED)
	if not bool(_detail_read.get("connected", true)):
		_add_line(_loc("TECH_STATUS_INERT"), MUTED)
		_add_line(str(_detail_read.get("debt", "")), MUTED)
		_conditions()
		_gates()
		return
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
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
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


## Adapt the existing scroll viewport; fixed header content still contributes its real minimum.
func set_available_height(value: float) -> void:
	for child: Node in get_children():
		if child is PanelContainer:
			var panel: PanelContainer = child as PanelContainer
			var height: float = maxf(panel.get_combined_minimum_size().y, minf(PANEL_RECT.size.y, value))
			panel.size = Vector2(PANEL_RECT.size.x, height)
			size = panel.size
			return
