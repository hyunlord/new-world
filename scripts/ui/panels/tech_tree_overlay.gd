extends Control
class_name TechTreeOverlay

signal closed

const NODE_W: float = 64.0
const NODE_H: float = 64.0
const SPACING_X: float = 160.0
const SPACING_Y: float = 68.0
const MARGIN: Vector2 = Vector2(120.0, 50.0)
const ZOOM_MIN: float = 0.3
const ZOOM_MAX: float = 2.5
const ZOOM_STEP: float = 0.08
const RING_RADIUS: float = 35.0
const RING_WIDTH: float = 3.0
const EDGE_WIDTH: float = 1.5
const EDGE_HIGHLIGHT_WIDTH: float = 2.5
const LABEL_FONT_SIZE: int = 9
const TOP_BAR_HEIGHT: float = 42.0
const DETAIL_PANEL_WIDTH: float = 420.0

const CATEGORY_COLORS: Dictionary = {
	"materials_crafting": Color(0.18, 0.65, 0.63),
	"construction": Color(0.55, 0.48, 0.42),
	"food_production": Color(0.85, 0.56, 0.20),
	"animal": Color(0.72, 0.36, 0.23),
	"maritime": Color(0.13, 0.50, 0.69),
	"knowledge_science": Color(0.18, 0.37, 0.60),
	"art_culture": Color(0.63, 0.25, 0.50),
	"social_organization": Color(0.48, 0.40, 0.85),
}

const CATEGORY_NAMES: Dictionary = {
	"materials_crafting": "TECH_CAT_CRAFT",
	"construction": "TECH_CAT_BUILD",
	"food_production": "TECH_CAT_FOOD",
	"animal": "TECH_CAT_HUNT",
	"maritime": "TECH_CAT_MARITIME",
	"knowledge_science": "TECH_CAT_KNOWLEDGE",
	"art_culture": "TECH_CAT_ART",
	"social_organization": "TECH_CAT_SOCIAL",
}

const PROFICIENCY_COLORS: Array[Dictionary] = [
	{"min": 0.76, "color": Color(0.96, 0.62, 0.04)},
	{"min": 0.51, "color": Color(0.06, 0.73, 0.51)},
	{"min": 0.26, "color": Color(0.23, 0.51, 0.96)},
	{"min": 0.01, "color": Color(0.39, 0.56, 1.0)},
]
const PROFICIENCY_COLOR_ZERO: Color = Color(0.42, 0.45, 0.50)

const COLUMN_HEADERS: Array[String] = ["TECH_COL_BASIC", "TECH_COL_APPLIED", "TECH_COL_ADVANCED", "TECH_COL_EXPERT"]
const SOURCE_ICONS: Array[String] = ["🗣️", "👁️", "🔨", "📜", "🏛️", "💡"]

enum NodeState { KNOWN, DISCOVERABLE, LOCKED }

var _tech_tree_manager: RefCounted
var _sim_engine: RefCounted
var _selected_entity_id: int = -1
var _entity_knowledge: Dictionary = {}

var _zoom: float = 1.0
var _pan: Vector2 = Vector2.ZERO
var _is_panning: bool = false
var _pan_anchor: Vector2 = Vector2.ZERO
var _pan_start: Vector2 = Vector2.ZERO

var _tech_defs: Dictionary = {}
var _node_world_pos: Dictionary = {}
var _node_states: Dictionary = {}
var _edge_list: Array[Dictionary] = []
var _category_lanes: Dictionary = {}

var _hovered_id: String = ""
var _selected_id: String = ""

var _icon_labels: Dictionary = {}
var _chain_edges: Dictionary = {}
var _chain_nodes: Dictionary = {}
var _entity_label: Label
var _detail_panel: PanelContainer
var _detail_vbox: VBoxContainer

enum ViewMode { AGENT, SETTLEMENT, BAND }
var _view_mode: int = ViewMode.AGENT
var _settlement_manager: RefCounted
var _current_band_id: int = -1
var _band_knowledge_cache: Dictionary = {}
var _agent_knowledge_backup: Dictionary = {}
var _agent_button: Button
var _settlement_dropdown: MenuButton
var _band_dropdown: MenuButton


func setup(tech_tree_manager: RefCounted, sim_engine: RefCounted, settlement_manager: RefCounted = null) -> void:
	_tech_tree_manager = tech_tree_manager
	_sim_engine = sim_engine
	_settlement_manager = settlement_manager
	mouse_filter = Control.MOUSE_FILTER_STOP
	_build_top_bar()
	_build_detail_panel()
	_load_stone_age()
	_compute_layout()
	_cache_edges()
	_create_icon_labels()


func _build_top_bar() -> void:
	var top_bar := HBoxContainer.new()
	top_bar.set_anchors_preset(Control.PRESET_TOP_WIDE)
	top_bar.offset_bottom = TOP_BAR_HEIGHT
	top_bar.add_theme_constant_override("separation", 4)
	add_child(top_bar)

	for era_info: Array in [["ERA_STONE_AGE", true], ["ERA_TRIBAL", false], ["ERA_BRONZE_AGE", false]]:
		var btn := Button.new()
		btn.text = Locale.ltr(str(era_info[0]))
		btn.flat = true
		btn.disabled = not bool(era_info[1])
		btn.add_theme_font_size_override("font_size", 12)
		btn.custom_minimum_size = Vector2(80, 28)
		btn.focus_mode = Control.FOCUS_NONE
		top_bar.add_child(btn)

	var sep := VSeparator.new()
	sep.custom_minimum_size.x = 1
	top_bar.add_child(sep)

	_agent_button = Button.new()
	_agent_button.text = "🧑 ?"
	_agent_button.flat = true
	_agent_button.add_theme_font_size_override("font_size", 11)
	_agent_button.focus_mode = Control.FOCUS_NONE
	_agent_button.pressed.connect(_switch_to_agent_view)
	top_bar.add_child(_agent_button)

	_settlement_dropdown = MenuButton.new()
	_settlement_dropdown.text = Locale.ltr("TECH_VIEW_SETTLEMENT")
	_settlement_dropdown.flat = true
	_settlement_dropdown.add_theme_font_size_override("font_size", 11)
	_settlement_dropdown.focus_mode = Control.FOCUS_NONE
	top_bar.add_child(_settlement_dropdown)
	_settlement_dropdown.get_popup().id_pressed.connect(_on_settlement_selected)

	_band_dropdown = MenuButton.new()
	_band_dropdown.text = Locale.ltr("TECH_VIEW_BAND")
	_band_dropdown.flat = true
	_band_dropdown.add_theme_font_size_override("font_size", 11)
	_band_dropdown.focus_mode = Control.FOCUS_NONE
	top_bar.add_child(_band_dropdown)
	_band_dropdown.get_popup().id_pressed.connect(_on_band_selected)

	_entity_label = Label.new()
	_entity_label.text = ""
	_entity_label.add_theme_font_size_override("font_size", 11)
	_entity_label.add_theme_color_override("font_color", Color(0.45, 0.55, 0.68))
	top_bar.add_child(_entity_label)

	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	top_bar.add_child(spacer)

	var close_btn := Button.new()
	close_btn.text = Locale.ltr("UI_CLOSE")
	close_btn.flat = true
	close_btn.add_theme_font_size_override("font_size", 14)
	close_btn.focus_mode = Control.FOCUS_NONE
	close_btn.pressed.connect(func() -> void: visible = false; closed.emit())
	top_bar.add_child(close_btn)


func _build_detail_panel() -> void:
	_detail_panel = PanelContainer.new()
	_detail_panel.visible = false
	_detail_panel.anchor_left = 1.0
	_detail_panel.anchor_right = 1.0
	_detail_panel.anchor_top = 0.0
	_detail_panel.anchor_bottom = 1.0
	_detail_panel.offset_left = -DETAIL_PANEL_WIDTH
	_detail_panel.offset_right = 0.0
	_detail_panel.offset_top = TOP_BAR_HEIGHT
	_detail_panel.offset_bottom = 0.0
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.04, 0.06, 0.09, 0.95)
	style.border_width_left = 1
	style.border_color = Color(0.15, 0.20, 0.28)
	style.content_margin_left = 12
	style.content_margin_right = 12
	style.content_margin_top = 12
	style.content_margin_bottom = 12
	_detail_panel.add_theme_stylebox_override("panel", style)
	_detail_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	add_child(_detail_panel)

	var scroll := ScrollContainer.new()
	scroll.set_anchors_preset(Control.PRESET_FULL_RECT)
	_detail_panel.add_child(scroll)

	_detail_vbox = VBoxContainer.new()
	_detail_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_detail_vbox.add_theme_constant_override("separation", 6)
	scroll.add_child(_detail_vbox)


func _load_stone_age() -> void:
	_tech_defs.clear()
	if _tech_tree_manager == null:
		return
	for tech_id: String in _tech_tree_manager.get_all_ids():
		var def: Dictionary = _tech_tree_manager.get_def(tech_id)
		if str(def.get("era", "")) == "stone_age":
			_tech_defs[tech_id] = def


func _compute_layout() -> void:
	_node_world_pos.clear()
	_category_lanes.clear()

	var cat_order: Array[String] = [
		"materials_crafting", "construction", "food_production",
		"maritime", "knowledge_science", "art_culture", "social_organization"
	]
	var cat_groups: Dictionary = {}
	for cat: String in cat_order:
		cat_groups[cat] = []

	for tech_id: String in _tech_defs:
		var def: Dictionary = _tech_defs[tech_id]
		var cats: Array = def.get("categories", [])
		var primary: String = str(cats[0]) if not cats.is_empty() else "materials_crafting"
		if primary == "animal":
			primary = "food_production"
		if not cat_groups.has(primary):
			primary = "materials_crafting"
		cat_groups[primary].append(tech_id)

	var row: int = 0
	for cat: String in cat_order:
		var group: Array = cat_groups.get(cat, [])
		if group.is_empty():
			continue
		group.sort_custom(func(a: String, b: String) -> bool:
			var da: Dictionary = _tech_defs[a].get("ui", {})
			var db: Dictionary = _tech_defs[b].get("ui", {})
			var xa: int = int(da.get("tree_x", 0))
			var xb: int = int(db.get("tree_x", 0))
			if xa != xb: return xa < xb
			return int(da.get("tree_y", 0)) < int(db.get("tree_y", 0))
		)

		var lane_start: int = row
		for tech_id: String in group:
			var def: Dictionary = _tech_defs[tech_id]
			var ui_data: Dictionary = def.get("ui", {})
			var col: int = int(ui_data.get("tree_x", 0))
			_node_world_pos[tech_id] = Vector2(
				MARGIN.x + float(col) * SPACING_X,
				MARGIN.y + float(row) * SPACING_Y,
			)
			row += 1

		var cat_color: Color = CATEGORY_COLORS.get(cat, Color(0.3, 0.3, 0.3))
		_category_lanes[cat] = {
			"min_row": lane_start,
			"max_row": row - 1,
			"color": cat_color,
		}
		row += 1


func _cache_edges() -> void:
	_edge_list.clear()
	for tech_id: String in _tech_defs:
		var def: Dictionary = _tech_defs[tech_id]
		var prereqs: Dictionary = def.get("prereq_logic", {})
		var to_pos: Vector2 = _node_world_pos.get(tech_id, Vector2.ZERO)

		for prereq_id: String in prereqs.get("all_of", []):
			if _node_world_pos.has(prereq_id):
				_edge_list.append({
					"from_id": prereq_id, "to_id": tech_id,
					"from_pos": _node_world_pos[prereq_id], "to_pos": to_pos,
					"is_soft": false,
				})

		for prereq_id: String in prereqs.get("soft", []):
			if _node_world_pos.has(prereq_id):
				_edge_list.append({
					"from_id": prereq_id, "to_id": tech_id,
					"from_pos": _node_world_pos[prereq_id], "to_pos": to_pos,
					"is_soft": true,
				})


func _create_icon_labels() -> void:
	for tech_id: String in _tech_defs:
		var def: Dictionary = _tech_defs[tech_id]
		var ui_data: Dictionary = def.get("ui", {})
		var icon_lbl := Label.new()
		icon_lbl.text = _tech_icon(str(ui_data.get("icon", "")))
		icon_lbl.add_theme_font_size_override("font_size", 20)
		icon_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		icon_lbl.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		icon_lbl.mouse_filter = Control.MOUSE_FILTER_IGNORE
		add_child(icon_lbl)
		_icon_labels[tech_id] = icon_lbl
	_update_label_positions()


func _update_label_positions() -> void:
	for tech_id: String in _icon_labels:
		var label: Label = _icon_labels[tech_id]
		var pos: Vector2 = _node_world_pos.get(tech_id, Vector2.ZERO)
		var sp := _world_to_screen(pos)
		var sz := Vector2(NODE_W, NODE_H) * _zoom
		label.position = sp
		label.size = sz
		label.add_theme_font_size_override("font_size", int(20.0 * _zoom))
		var state: int = _node_states.get(tech_id, NodeState.LOCKED)
		match state:
			NodeState.KNOWN:
				label.modulate = Color(1, 1, 1, 1)
			NodeState.DISCOVERABLE:
				label.modulate = Color(0.7, 0.7, 0.7, 0.45)
			_:
				label.modulate = Color(0.5, 0.5, 0.5, 0.25)


func _compute_node_states() -> void:
	_node_states.clear()
	for tech_id: String in _tech_defs:
		if _entity_knowledge.has(tech_id):
			_node_states[tech_id] = NodeState.KNOWN
			continue
		var def: Dictionary = _tech_defs[tech_id]
		var prereqs: Dictionary = def.get("prereq_logic", {})
		var all_of: Array = prereqs.get("all_of", [])
		var tech_prereqs_met: bool = true
		for pid: String in all_of:
			if not _entity_knowledge.has(pid):
				tech_prereqs_met = false
				break
		if tech_prereqs_met:
			_node_states[tech_id] = NodeState.DISCOVERABLE
		else:
			_node_states[tech_id] = NodeState.LOCKED


func _draw() -> void:
	# 1. Background
	draw_rect(Rect2(Vector2.ZERO, size), Color(0.024, 0.031, 0.047, 0.95))

	# 2. Category lanes
	var max_col: int = _get_max_column()
	for cat: String in _category_lanes:
		var lane: Dictionary = _category_lanes[cat]
		var cat_color: Color = lane["color"]
		var y0: float = MARGIN.y + float(int(lane["min_row"])) * SPACING_Y - 18.0
		var h: float = (float(int(lane["max_row"]) - int(lane["min_row"])) + 1.0) * SPACING_Y + 20.0
		var w: float = MARGIN.x * 2.0 + float(max_col) * SPACING_X + NODE_W + 40.0
		var lane_rect := Rect2(_world_to_screen(Vector2(8.0, y0)), Vector2(w - 16.0, h) * _zoom)
		draw_rect(lane_rect, Color(cat_color.r, cat_color.g, cat_color.b, 0.04))
		var label_pos := _world_to_screen(Vector2(22.0, y0 + 15.0))
		draw_string(ThemeDB.fallback_font, label_pos,
			Locale.ltr(CATEGORY_NAMES.get(cat, "")).to_upper(),
			HORIZONTAL_ALIGNMENT_LEFT, -1, int(10.0 * _zoom),
			Color(cat_color.r, cat_color.g, cat_color.b, 0.4))

	# 3. Column headers
	for col: int in range(mini(4, max_col + 1)):
		var header_pos := _world_to_screen(Vector2(MARGIN.x + float(col) * SPACING_X + NODE_W * 0.5, MARGIN.y - 20.0))
		if col < COLUMN_HEADERS.size():
			draw_string(ThemeDB.fallback_font, header_pos,
				Locale.ltr(COLUMN_HEADERS[col]),
				HORIZONTAL_ALIGNMENT_CENTER, -1, int(9.0 * _zoom),
				Color(0.10, 0.14, 0.21))

	# 4. Bezier edges
	var active_id: String = _selected_id if _selected_id != "" else _hovered_id
	for edge: Dictionary in _edge_list:
		var from_pos: Vector2 = edge["from_pos"]
		var to_pos: Vector2 = edge["to_pos"]
		var p0 := _world_to_screen(Vector2(from_pos.x + NODE_W, from_pos.y + NODE_H * 0.5))
		var p3 := _world_to_screen(Vector2(to_pos.x, to_pos.y + NODE_H * 0.5))
		var mx: float = (p0.x + p3.x) * 0.5
		var p1 := Vector2(mx, p0.y)
		var p2 := Vector2(mx, p3.y)
		var edge_key: String = str(edge["from_id"]) + "->" + str(edge["to_id"])
		var is_highlight: bool = _chain_edges.has(edge_key)
		var edge_color: Color
		var edge_w: float
		if is_highlight:
			edge_color = Color(0.35, 0.60, 0.82, 0.85)
			edge_w = EDGE_HIGHLIGHT_WIDTH * _zoom
		else:
			edge_color = Color(0.13, 0.19, 0.25, 0.18)
			edge_w = EDGE_WIDTH * _zoom
		var prev: Vector2 = p0
		for i: int in range(1, 13):
			var t: float = float(i) / 12.0
			var mt: float = 1.0 - t
			var pt := mt * mt * mt * p0 + 3.0 * mt * mt * t * p1 + 3.0 * mt * t * t * p2 + t * t * t * p3
			draw_line(prev, pt, edge_color, edge_w)
			prev = pt

	# 5. Nodes
	for tech_id: String in _tech_defs:
		_draw_node(tech_id)


func _draw_node(tech_id: String) -> void:
	var pos: Vector2 = _node_world_pos.get(tech_id, Vector2.ZERO)
	var screen_pos := _world_to_screen(pos)
	var screen_size := Vector2(NODE_W, NODE_H) * _zoom
	var def: Dictionary = _tech_defs.get(tech_id, {})
	var state: int = _node_states.get(tech_id, NodeState.LOCKED)
	var cats: Array = def.get("categories", [])
	var cat_color: Color = _get_cat_color(cats)
	var entry: Dictionary = _entity_knowledge.get(tech_id, {})
	var prof: float = clampf(_safe_float(entry, "proficiency", 0.0), 0.0, 1.0)
	var is_active: bool = tech_id == _selected_id or tech_id == _hovered_id

	var fill_color: Color
	var stroke_color: Color
	match state:
		NodeState.KNOWN:
			fill_color = Color(cat_color.r, cat_color.g, cat_color.b, 0.08)
			stroke_color = Color(cat_color.r, cat_color.g, cat_color.b, 0.22)
		NodeState.DISCOVERABLE:
			fill_color = Color(cat_color.r, cat_color.g, cat_color.b, 0.03)
			stroke_color = Color(cat_color.r, cat_color.g, cat_color.b, 0.12)
		_:
			fill_color = Color(0.05, 0.06, 0.08, 1.0)
			stroke_color = Color(0.10, 0.12, 0.16, 1.0)

	var in_chain: bool = _chain_nodes.has(tech_id)
	if is_active:
		stroke_color = Color(0.42, 0.63, 0.85, 0.8)
	elif in_chain:
		stroke_color = Color(0.23, 0.42, 0.56, 0.4)

	var node_rect := Rect2(screen_pos, screen_size)
	draw_rect(node_rect, fill_color)
	draw_rect(node_rect, stroke_color, false, 1.0 * _zoom)

	if is_active:
		var glow_rect := Rect2(screen_pos - Vector2(2, 2) * _zoom, screen_size + Vector2(4, 4) * _zoom)
		draw_rect(glow_rect, Color(0.31, 0.56, 0.75, 0.2), false, 1.0 * _zoom)
	elif in_chain:
		var chain_rect := Rect2(screen_pos - Vector2(1, 1) * _zoom, screen_size + Vector2(2, 2) * _zoom)
		draw_rect(chain_rect, Color(0.23, 0.42, 0.56, 0.15), false, 0.8 * _zoom)

	# Proficiency / ratio ring
	if state == NodeState.KNOWN:
		var center := screen_pos + screen_size * 0.5
		var r: float = RING_RADIUS * _zoom
		if _view_mode == ViewMode.AGENT and prof > 0.0:
			draw_arc(center, r, 0.0, TAU, 32, Color(0.08, 0.10, 0.14), RING_WIDTH * _zoom)
			draw_arc(center, r, -PI * 0.5, -PI * 0.5 + TAU * prof, 32, _prof_color(prof), RING_WIDTH * _zoom)
		elif _view_mode == ViewMode.BAND:
			var cache: Dictionary = _band_knowledge_cache.get(tech_id, {})
			var count: int = int(cache.get("count", 0))
			var btotal: int = maxi(int(cache.get("total", 1)), 1)
			var ratio: float = float(count) / float(btotal)
			draw_arc(center, r, 0.0, TAU, 32, Color(0.08, 0.10, 0.14), RING_WIDTH * _zoom)
			var ratio_color: Color = Color(0.30, 0.69, 0.31) if ratio >= 0.8 else (Color(0.96, 0.62, 0.04) if ratio >= 0.4 else Color(0.86, 0.15, 0.15))
			draw_arc(center, r, -PI * 0.5, -PI * 0.5 + TAU * ratio, 32, ratio_color, RING_WIDTH * _zoom)
		elif _view_mode == ViewMode.SETTLEMENT:
			draw_rect(Rect2(screen_pos - Vector2(1, 1) * _zoom, screen_size + Vector2(2, 2) * _zoom),
				Color(0.30, 0.69, 0.31, 0.4), false, 2.0 * _zoom)

	# Name label
	var label_y: float = screen_pos.y + screen_size.y + 10.0 * _zoom
	var label_color: Color
	match state:
		NodeState.KNOWN: label_color = Color(0.56, 0.63, 0.69)
		NodeState.DISCOVERABLE: label_color = Color(0.23, 0.29, 0.35)
		_: label_color = Color(0.15, 0.18, 0.22)

	var display_key: String = str(def.get("display_key", tech_id))
	var display_name: String = Locale.ltr(display_key)
	draw_string(ThemeDB.fallback_font,
		Vector2(screen_pos.x + screen_size.x * 0.5, label_y),
		display_name, HORIZONTAL_ALIGNMENT_CENTER,
		int(screen_size.x + 32.0 * _zoom),
		int(LABEL_FONT_SIZE * _zoom), label_color)

	if state == NodeState.KNOWN:
		var sub_text: String = ""
		if _view_mode == ViewMode.BAND:
			var cache: Dictionary = _band_knowledge_cache.get(tech_id, {})
			sub_text = "%d/%d" % [int(cache.get("count", 0)), int(cache.get("total", 0))]
		else:
			sub_text = "%d%%" % int(prof * 100.0)
		draw_string(ThemeDB.fallback_font,
			Vector2(screen_pos.x + screen_size.x * 0.5, label_y + 12.0 * _zoom),
			sub_text, HORIZONTAL_ALIGNMENT_CENTER,
			int(screen_size.x), int(8.0 * _zoom), _prof_color(prof))
	elif state == NodeState.DISCOVERABLE:
		draw_string(ThemeDB.fallback_font,
			Vector2(screen_pos.x + screen_size.x * 0.5, label_y + 12.0 * _zoom),
			Locale.ltr("TECH_DISCOVERABLE"), HORIZONTAL_ALIGNMENT_CENTER,
			int(screen_size.x), int(7.0 * _zoom),
			Color(cat_color.r, cat_color.g, cat_color.b, 0.3))


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.ctrl_pressed:
			if mb.button_index == MOUSE_BUTTON_WHEEL_UP and mb.pressed:
				_set_zoom(clampf(_zoom + ZOOM_STEP, ZOOM_MIN, ZOOM_MAX), mb.position)
				accept_event()
				return
			if mb.button_index == MOUSE_BUTTON_WHEEL_DOWN and mb.pressed:
				_set_zoom(clampf(_zoom - ZOOM_STEP, ZOOM_MIN, ZOOM_MAX), mb.position)
				accept_event()
				return
		if mb.button_index == MOUSE_BUTTON_WHEEL_UP and mb.pressed:
			_pan.y += 40.0
			_on_view_changed()
			accept_event()
			return
		if mb.button_index == MOUSE_BUTTON_WHEEL_DOWN and mb.pressed:
			_pan.y -= 40.0
			_on_view_changed()
			accept_event()
			return
		if mb.button_index == MOUSE_BUTTON_WHEEL_LEFT and mb.pressed:
			_pan.x += 40.0
			_on_view_changed()
			accept_event()
			return
		if mb.button_index == MOUSE_BUTTON_WHEEL_RIGHT and mb.pressed:
			_pan.x -= 40.0
			_on_view_changed()
			accept_event()
			return
		if mb.button_index == MOUSE_BUTTON_MIDDLE or mb.button_index == MOUSE_BUTTON_RIGHT:
			if mb.pressed:
				_is_panning = true
				_pan_anchor = mb.position
				_pan_start = _pan
			else:
				_is_panning = false
			accept_event()
		if mb.button_index == MOUSE_BUTTON_LEFT and mb.pressed:
			var clicked: String = _hit_test(mb.position)
			if clicked != "":
				_selected_id = clicked
				_update_active_chain()
				_show_detail_panel(clicked)
				queue_redraw()
				accept_event()
				return
			_selected_id = ""
			_update_active_chain()
			_detail_panel.visible = false
			queue_redraw()

	if event is InputEventMouseMotion and _is_panning:
		var mm := event as InputEventMouseMotion
		_pan = _pan_start + (mm.position - _pan_anchor)
		_on_view_changed()
		accept_event()

	if event is InputEventKey:
		var key := event as InputEventKey
		if key.pressed and key.keycode == KEY_ESCAPE:
			visible = false
			closed.emit()
			accept_event()


func _notification(what: int) -> void:
	if what == NOTIFICATION_VISIBILITY_CHANGED and visible:
		_populate_dropdowns()


func _process(_delta: float) -> void:
	if not visible:
		return
	var mouse_pos: Vector2 = get_local_mouse_position()
	var new_hover: String = _hit_test(mouse_pos)
	if new_hover != _hovered_id:
		_hovered_id = new_hover
		_update_active_chain()
		queue_redraw()


func _update_active_chain() -> void:
	var active: String = _selected_id if _selected_id != "" else _hovered_id
	if active == "":
		_chain_edges.clear()
		_chain_nodes.clear()
	else:
		var chain: Dictionary = _compute_chain(active)
		_chain_edges = chain["edges"]
		_chain_nodes = chain["nodes"]


func _compute_chain(tech_id: String) -> Dictionary:
	var chain_edges: Dictionary = {}
	var chain_nodes: Dictionary = {}
	chain_nodes[tech_id] = true
	# Walk UP: all ancestors
	var up_queue: Array[String] = [tech_id]
	while not up_queue.is_empty():
		var current: String = up_queue.pop_back()
		var def: Dictionary = _tech_defs.get(current, {})
		var prereqs: Array = def.get("prereq_logic", {}).get("all_of", [])
		for pid_raw: Variant in prereqs:
			var pid: String = str(pid_raw)
			var ek: String = pid + "->" + current
			if not chain_edges.has(ek) and _tech_defs.has(pid):
				chain_edges[ek] = true
				chain_nodes[pid] = true
				up_queue.append(pid)
	# Walk DOWN: all descendants
	var down_queue: Array[String] = [tech_id]
	while not down_queue.is_empty():
		var current: String = down_queue.pop_back()
		for tid: String in _tech_defs:
			var def: Dictionary = _tech_defs[tid]
			var prereqs: Array = def.get("prereq_logic", {}).get("all_of", [])
			for pid_raw: Variant in prereqs:
				if str(pid_raw) == current:
					var ek: String = current + "->" + tid
					if not chain_edges.has(ek):
						chain_edges[ek] = true
						chain_nodes[tid] = true
						down_queue.append(tid)
					break
	return {"edges": chain_edges, "nodes": chain_nodes}


func _switch_to_agent_view() -> void:
	_view_mode = ViewMode.AGENT
	_entity_knowledge = _agent_knowledge_backup.duplicate()
	_compute_node_states()
	_update_active_chain()
	queue_redraw()
	_update_label_positions()


func _on_settlement_selected(index: int) -> void:
	if _settlement_manager == null:
		return
	if not _settlement_manager.has_method("get_all_settlements"):
		return
	var settlements: Array = _settlement_manager.get_all_settlements()
	if index < 0 or index >= settlements.size():
		return
	_view_mode = ViewMode.SETTLEMENT
	_load_settlement_knowledge(settlements[index])
	_compute_node_states()
	_update_active_chain()
	queue_redraw()
	_update_label_positions()
	if _entity_label != null:
		_entity_label.text = "🏘️ " + str(index)


func _on_band_selected(index: int) -> void:
	if _sim_engine == null or not _sim_engine.has_method("runtime_get_band_list"):
		return
	var bands: Array = _sim_engine.runtime_get_band_list()
	if index < 0 or index >= bands.size():
		return
	var band: Dictionary = bands[index] if bands[index] is Dictionary else {}
	_current_band_id = int(band.get("id", -1))
	_view_mode = ViewMode.BAND
	_load_band_knowledge()
	_compute_node_states()
	_update_active_chain()
	queue_redraw()
	_update_label_positions()
	if _entity_label != null:
		_entity_label.text = "👥 " + str(band.get("name", "?"))


func _load_settlement_knowledge(settlement: Variant) -> void:
	_entity_knowledge.clear()
	_band_knowledge_cache.clear()
	if settlement == null:
		return
	if settlement is Dictionary:
		var tech_states: Dictionary = settlement.get("tech_states", {})
		for tech_id: String in tech_states:
			_entity_knowledge[tech_id] = {
				"proficiency": 1.0,
				"source": 0,
			}


func _load_band_knowledge() -> void:
	_entity_knowledge.clear()
	_band_knowledge_cache.clear()
	if _sim_engine == null or _current_band_id < 0:
		return
	if not _sim_engine.has_method("runtime_get_band_detail"):
		return
	var band_detail: Dictionary = _sim_engine.runtime_get_band_detail(_current_band_id)
	var member_ids_raw: Variant = band_detail.get("member_ids", [])
	var member_ids: Array = member_ids_raw if member_ids_raw is Array else []
	var total: int = member_ids.size()
	if total == 0:
		return
	var tech_counts: Dictionary = {}
	for member_id: Variant in member_ids:
		var mid: int = int(member_id)
		var k_tab: Dictionary = _sim_engine.get_entity_tab(mid, "knowledge")
		var known_raw: Variant = k_tab.get("known", [])
		var known: Array = known_raw if known_raw is Array else []
		for entry_raw: Variant in known:
			if entry_raw is Dictionary:
				var kid: String = str((entry_raw as Dictionary).get("id", ""))
				if not kid.is_empty():
					tech_counts[kid] = tech_counts.get(kid, 0) + 1
	for tech_id: String in tech_counts:
		var count: int = tech_counts[tech_id]
		_entity_knowledge[tech_id] = {
			"proficiency": float(count) / float(total),
			"source": 0,
		}
		_band_knowledge_cache[tech_id] = {"count": count, "total": total}


func _populate_dropdowns() -> void:
	# Settlements
	var s_popup: PopupMenu = _settlement_dropdown.get_popup()
	s_popup.clear()
	if _settlement_manager != null and _settlement_manager.has_method("get_all_settlements"):
		var settlements: Array = _settlement_manager.get_all_settlements()
		for i: int in range(settlements.size()):
			var s: Variant = settlements[i]
			var s_name: String = "Settlement %d" % i
			if s is Dictionary:
				s_name = str(s.get("name", s_name))
			s_popup.add_item("🏘️ " + s_name, i)
	_settlement_dropdown.disabled = s_popup.item_count == 0

	# Bands
	var b_popup: PopupMenu = _band_dropdown.get_popup()
	b_popup.clear()
	if _sim_engine != null and _sim_engine.has_method("runtime_get_band_list"):
		var bands: Array = _sim_engine.runtime_get_band_list()
		for i: int in range(bands.size()):
			var b: Dictionary = bands[i] if bands[i] is Dictionary else {}
			var b_name: String = str(b.get("name", "Band %d" % i))
			var b_count: int = int(b.get("member_count", 0))
			b_popup.add_item("👥 %s (%d)" % [b_name, b_count], i)
	_band_dropdown.disabled = b_popup.item_count == 0


func _world_to_screen(p: Vector2) -> Vector2:
	return (p * _zoom) + _pan


func _set_zoom(new_zoom: float, pivot: Vector2) -> void:
	var world_before: Vector2 = (pivot - _pan) / _zoom
	_zoom = new_zoom
	_pan = pivot - world_before * _zoom
	_on_view_changed()


func _on_view_changed() -> void:
	queue_redraw()
	_update_label_positions()


func _hit_test(screen_pos: Vector2) -> String:
	for tech_id: String in _tech_defs:
		var world_pos: Vector2 = _node_world_pos.get(tech_id, Vector2.ZERO)
		var sp := _world_to_screen(world_pos)
		var sz := Vector2(NODE_W, NODE_H) * _zoom
		if Rect2(sp, sz).has_point(screen_pos):
			return tech_id
	return ""


func _show_detail_panel(tech_id: String) -> void:
	for child in _detail_vbox.get_children():
		child.queue_free()

	var def: Dictionary = _tech_defs.get(tech_id, {})
	if def.is_empty():
		_detail_panel.visible = false
		return

	var display_name: String = Locale.ltr(str(def.get("display_key", tech_id)))
	var desc_key: String = str(def.get("description_key", ""))
	var desc: String = Locale.ltr(desc_key) if not desc_key.is_empty() else ""
	var prereqs: Dictionary = def.get("prereq_logic", {})
	var unlocks: Dictionary = def.get("unlocks", {})
	var requirements: Dictionary = def.get("requirements", {})
	var discovery: Dictionary = def.get("discovery", {})

	# Title
	var title := Label.new()
	title.text = display_name
	title.add_theme_font_size_override("font_size", 16)
	title.add_theme_color_override("font_color", Color.WHITE)
	_detail_vbox.add_child(title)

	# Category
	var cats: Array = def.get("categories", [])
	if not cats.is_empty():
		var cat_label := Label.new()
		cat_label.text = Locale.ltr(CATEGORY_NAMES.get(str(cats[0]), ""))
		cat_label.add_theme_font_size_override("font_size", 9)
		cat_label.add_theme_color_override("font_color", _get_cat_color(cats))
		_detail_vbox.add_child(cat_label)

	# Description
	if not desc.is_empty() and desc != desc_key:
		var desc_label := Label.new()
		desc_label.text = desc
		desc_label.add_theme_font_size_override("font_size", 10)
		desc_label.add_theme_color_override("font_color", Color(0.50, 0.58, 0.66))
		desc_label.autowrap_mode = TextServer.AUTOWRAP_WORD
		_detail_vbox.add_child(desc_label)

	# Agent proficiency
	var entry: Dictionary = _entity_knowledge.get(tech_id, {})
	if not entry.is_empty():
		var prof: float = clampf(_safe_float(entry, "proficiency", 0.0), 0.0, 1.0)
		var source_code: int = int(entry.get("source", 0))
		var source_icon: String = SOURCE_ICONS[clampi(source_code, 0, SOURCE_ICONS.size() - 1)]
		var prof_label := Label.new()
		prof_label.text = "%s: %d%% — %s: %s" % [
			Locale.ltr("UI_PROFICIENCY"), int(prof * 100),
			Locale.ltr("UI_SOURCE"), source_icon,
		]
		prof_label.add_theme_font_size_override("font_size", 11)
		prof_label.add_theme_color_override("font_color", _prof_color(prof))
		_detail_vbox.add_child(prof_label)
	else:
		var state: int = _node_states.get(tech_id, NodeState.LOCKED)
		var state_label := Label.new()
		if state == NodeState.DISCOVERABLE:
			state_label.text = Locale.ltr("TECH_DISCOVERABLE")
			state_label.add_theme_color_override("font_color", Color(0.25, 0.55, 0.35))
		else:
			state_label.text = Locale.ltr("TECH_STATE_LOCKED")
			state_label.add_theme_color_override("font_color", Color(0.35, 0.40, 0.48))
		state_label.add_theme_font_size_override("font_size", 11)
		_detail_vbox.add_child(state_label)

	# Tech prerequisites
	var all_of: Array = prereqs.get("all_of", [])
	if not all_of.is_empty():
		_add_detail_section(Locale.ltr("TECH_DETAIL_PREREQS"))
		for pid: String in all_of:
			var pdef: Dictionary = _tech_defs.get(pid, {})
			var pname: String = Locale.ltr(str(pdef.get("display_key", pid)))
			var has_it: bool = _entity_knowledge.has(pid)
			var check := Label.new()
			check.text = "%s %s" % ["✓" if has_it else "✗", pname]
			check.add_theme_font_size_override("font_size", 10)
			check.add_theme_color_override("font_color", Color(0.28, 0.66, 0.16) if has_it else Color(0.78, 0.22, 0.15))
			_detail_vbox.add_child(check)

	# Non-tech requirements
	var env_items: Array[String] = []
	var biomes_raw: Variant = requirements.get("biomes_nearby", [])
	var biomes: Array = biomes_raw if biomes_raw is Array else []
	for b: String in biomes:
		env_items.append(Locale.ltr("TECH_DETAIL_BIOME_REQ").replace("{name}", b))
	var resources_raw: Variant = requirements.get("resources_nearby", [])
	var resources: Array = resources_raw if resources_raw is Array else []
	for r: String in resources:
		env_items.append(Locale.ltr("TECH_DETAIL_RESOURCE_REQ").replace("{name}", r))
	var req_pop: int = int(discovery.get("required_population", 0))
	if req_pop > 0:
		env_items.append(Locale.ltr("TECH_DETAIL_POP_REQ").replace("{n}", str(req_pop)))
	var req_skills_raw: Variant = discovery.get("required_skills", [])
	var req_skills: Array = req_skills_raw if req_skills_raw is Array else []
	for skill_raw: Variant in req_skills:
		if skill_raw is Dictionary:
			var s: Dictionary = skill_raw
			var sname: String = str(s.get("skill", ""))
			var slevel: int = int(s.get("level", 0))
			env_items.append(Locale.ltr("TECH_DETAIL_SKILL_REQ").replace("{name}", sname).replace("{n}", str(slevel)))

	if not env_items.is_empty():
		_add_detail_section(Locale.ltr("TECH_DETAIL_ENV_REQS"))
		for item_text: String in env_items:
			var item_label := Label.new()
			item_label.text = "• " + item_text
			item_label.add_theme_font_size_override("font_size", 10)
			item_label.add_theme_color_override("font_color", Color(0.45, 0.52, 0.60))
			_detail_vbox.add_child(item_label)

	# Unlocks
	var unlock_lists: Array = [unlocks.get("skills", []), unlocks.get("buildings", []), unlocks.get("actions", [])]
	var all_unlocks: Array = []
	for ul: Variant in unlock_lists:
		if ul is Array:
			all_unlocks.append_array(ul)
	if not all_unlocks.is_empty():
		_add_detail_section(Locale.ltr("TECH_DETAIL_UNLOCKS"))
		for unlock_id: String in all_unlocks:
			var u_key: String = unlock_id.to_upper()
			var u_text: String = Locale.ltr(u_key)
			if u_text == u_key:
				u_text = unlock_id.replace("_", " ").capitalize()
			var u_label := Label.new()
			u_label.text = "• " + u_text
			u_label.add_theme_font_size_override("font_size", 10)
			u_label.add_theme_color_override("font_color", Color(0.55, 0.62, 0.70))
			_detail_vbox.add_child(u_label)

	_detail_panel.visible = true


func _add_detail_section(title_text: String) -> void:
	var spacer := Control.new()
	spacer.custom_minimum_size.y = 6
	_detail_vbox.add_child(spacer)
	var label := Label.new()
	label.text = title_text
	label.add_theme_font_size_override("font_size", 11)
	label.add_theme_color_override("font_color", Color(0.35, 0.42, 0.50))
	_detail_vbox.add_child(label)


func set_entity(entity_id: int) -> void:
	_selected_entity_id = entity_id
	_entity_knowledge.clear()
	if _sim_engine == null or entity_id < 0:
		if _entity_label != null:
			_entity_label.text = ""
		_compute_node_states()
		queue_redraw()
		_update_label_positions()
		return
	# Show entity name in top bar
	if _entity_label != null:
		var detail: Dictionary = _sim_engine.get_entity_detail(entity_id)
		var entity_name: String = str(detail.get("name", "?"))
		_entity_label.text = "  %s" % entity_name
		_agent_button.text = "🧑 %s" % entity_name
	var knowledge_tab: Dictionary = _sim_engine.get_entity_tab(entity_id, "knowledge")
	var known_raw: Variant = knowledge_tab.get("known", [])
	var known: Array = known_raw if known_raw is Array else []
	for entry_raw: Variant in known:
		if entry_raw is Dictionary:
			var entry: Dictionary = entry_raw
			var kid: String = str(entry.get("id", ""))
			if not kid.is_empty():
				_entity_knowledge[kid] = entry
	_agent_knowledge_backup = _entity_knowledge.duplicate()
	_view_mode = ViewMode.AGENT
	_compute_node_states()
	queue_redraw()
	_update_label_positions()


func _prof_color(prof: float) -> Color:
	for entry: Dictionary in PROFICIENCY_COLORS:
		if prof >= float(entry["min"]):
			return entry["color"]
	return PROFICIENCY_COLOR_ZERO


func _get_cat_color(cats: Array) -> Color:
	if cats.is_empty():
		return Color(0.35, 0.40, 0.50)
	return CATEGORY_COLORS.get(str(cats[0]), Color(0.35, 0.40, 0.50))


func _get_max_column() -> int:
	var max_col: int = 0
	for tech_id: String in _tech_defs:
		var ui: Dictionary = _tech_defs[tech_id].get("ui", {})
		max_col = maxi(max_col, int(ui.get("tree_x", 0)))
	return max_col


func _safe_float(dict: Dictionary, key: String, default: float) -> float:
	var val: Variant = dict.get(key, default)
	if val is float: return val
	if val is int: return float(val)
	return default


func _tech_icon(icon_key: String) -> String:
	match icon_key:
		"icon_fire": return "🔥"
		"icon_cooking": return "🍳"
		"icon_stone": return "🪨"
		"icon_tools": return "🔧"
		"icon_bone": return "🦴"
		"icon_shelter": return "🛖"
		"icon_hide": return "🦌"
		"icon_cordage": return "🧵"
		"icon_fishing": return "🐟"
		"icon_hunting": return "🏹"
		"icon_tracking": return "🐾"
		"icon_trapping": return "🪤"
		"icon_swimming": return "🏊"
		"icon_weather": return "⛅"
		"icon_painting": return "🎨"
		"icon_flute": return "🎵"
		"icon_oral": return "🗣️"
		"icon_kinship": return "👥"
		"icon_animism": return "🌿"
		"icon_seed": return "🌾"
		"icon_nut": return "🥜"
		"icon_root": return "🥕"
		"icon_microlith": return "🔪"
		"icon_burn": return "🔥"
		"icon_gathering": return "🫐"
		_: return "📦"
