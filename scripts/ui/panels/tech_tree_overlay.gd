extends Control
class_name TechTreeOverlay

## Fullscreen tech tree overlay with zoom/pan/nodes/edges.

const NODE_SIZE: float = 56.0
const NODE_SPACING_X: float = 120.0
const NODE_SPACING_Y: float = 72.0
const NODE_MARGIN: Vector2 = Vector2(80.0, 80.0)
const ZOOM_MIN: float = 0.4
const ZOOM_MAX: float = 2.0
const ZOOM_STEP: float = 0.1
const EDGE_ALPHA_DEFAULT: float = 0.15
const EDGE_ALPHA_HIGHLIGHT: float = 0.80
const EDGE_WIDTH: float = 2.0

const CATEGORY_COLORS: Dictionary = {
	"food_production": Color(0.85, 0.56, 0.20),
	"materials_crafting": Color(0.18, 0.65, 0.63),
	"construction": Color(0.55, 0.48, 0.42),
	"medicine_health": Color(0.25, 0.66, 0.36),
	"social_organization": Color(0.48, 0.40, 0.85),
	"agriculture_husbandry": Color(0.53, 0.69, 0.29),
	"metallurgy": Color(0.50, 0.55, 0.55),
	"knowledge_science": Color(0.18, 0.37, 0.60),
	"art_culture": Color(0.80, 0.40, 0.60),
	"maritime": Color(0.20, 0.50, 0.72),
}

var _tech_tree_manager: RefCounted
var _sim_engine: RefCounted
var _selected_entity_id: int = -1
var _entity_knowledge: Dictionary = {}
var _style_cache: Dictionary = {}

var _zoom: float = 1.0
var _pan: Vector2 = Vector2.ZERO
var _is_panning: bool = false
var _pan_anchor: Vector2 = Vector2.ZERO
var _pan_start: Vector2 = Vector2.ZERO

var _tech_defs: Dictionary = {}
var _node_buttons: Dictionary = {}
var _node_positions_world: Dictionary = {}
var _edge_pairs: Array[Dictionary] = []

var _hovered_tech_id: String = ""
var _selected_tech_id: String = ""

var _close_button: Button
var _detail_panel: PanelContainer
var _detail_content: VBoxContainer


func setup(tech_tree_manager: RefCounted, sim_engine: RefCounted) -> void:
	_tech_tree_manager = tech_tree_manager
	_sim_engine = sim_engine
	mouse_filter = Control.MOUSE_FILTER_STOP
	_build_ui()
	_load_tech_defs()
	_create_nodes()
	_cache_edges()


func _build_ui() -> void:
	# Top bar
	var top_bar := HBoxContainer.new()
	top_bar.set_anchors_preset(Control.PRESET_TOP_WIDE)
	top_bar.offset_bottom = 36.0
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

	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	top_bar.add_child(spacer)

	_close_button = Button.new()
	_close_button.text = Locale.ltr("UI_CLOSE")
	_close_button.flat = true
	_close_button.add_theme_font_size_override("font_size", 14)
	_close_button.focus_mode = Control.FOCUS_NONE
	_close_button.pressed.connect(_on_close)
	top_bar.add_child(_close_button)

	# Detail panel (right side, hidden)
	_detail_panel = PanelContainer.new()
	_detail_panel.visible = false
	_detail_panel.anchor_left = 1.0
	_detail_panel.anchor_right = 1.0
	_detail_panel.anchor_top = 0.0
	_detail_panel.anchor_bottom = 1.0
	_detail_panel.offset_left = -400.0
	_detail_panel.offset_right = 0.0
	_detail_panel.offset_top = 40.0
	_detail_panel.offset_bottom = 0.0
	var detail_style := StyleBoxFlat.new()
	detail_style.bg_color = Color(0.04, 0.06, 0.09, 0.95)
	detail_style.border_width_left = 1
	detail_style.border_color = Color(0.15, 0.20, 0.28)
	detail_style.content_margin_left = 12
	detail_style.content_margin_right = 12
	detail_style.content_margin_top = 12
	detail_style.content_margin_bottom = 12
	_detail_panel.add_theme_stylebox_override("panel", detail_style)
	_detail_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	add_child(_detail_panel)

	var scroll := ScrollContainer.new()
	scroll.set_anchors_preset(Control.PRESET_FULL_RECT)
	_detail_panel.add_child(scroll)

	_detail_content = VBoxContainer.new()
	_detail_content.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_detail_content.add_theme_constant_override("separation", 6)
	scroll.add_child(_detail_content)


func _load_tech_defs() -> void:
	_tech_defs.clear()
	if _tech_tree_manager == null:
		return
	for tech_id: String in _tech_tree_manager.get_all_ids():
		var def: Dictionary = _tech_tree_manager.get_def(tech_id)
		if str(def.get("era", "")) == "stone_age":
			_tech_defs[tech_id] = def


func _create_nodes() -> void:
	for tech_id: String in _tech_defs:
		var def: Dictionary = _tech_defs[tech_id]
		var ui_data: Dictionary = def.get("ui", {})
		var tree_x: int = int(ui_data.get("tree_x", 0))
		var tree_y: int = int(ui_data.get("tree_y", 0))

		var world_pos := Vector2(
			NODE_MARGIN.x + float(tree_x) * NODE_SPACING_X,
			NODE_MARGIN.y + float(tree_y) * NODE_SPACING_Y,
		)
		_node_positions_world[tech_id] = world_pos

		var btn := Button.new()
		btn.flat = true
		btn.focus_mode = Control.FOCUS_NONE
		btn.text = ""
		btn.custom_minimum_size = Vector2(NODE_SIZE, NODE_SIZE)
		btn.size = Vector2(NODE_SIZE, NODE_SIZE)

		var cat_color: Color = _category_color(def.get("categories", []))
		_apply_node_style(btn, cat_color)

		var display_name: String = Locale.ltr(str(def.get("display_key", tech_id)))
		btn.tooltip_text = display_name
		btn.mouse_filter = Control.MOUSE_FILTER_STOP
		btn.mouse_entered.connect(_on_node_hover.bind(tech_id))
		btn.mouse_exited.connect(_on_node_unhover)
		btn.pressed.connect(_on_node_click.bind(tech_id))
		add_child(btn)
		_node_buttons[tech_id] = btn

		# Icon
		var icon_label := Label.new()
		icon_label.text = _tech_icon(str(ui_data.get("icon", "")))
		icon_label.add_theme_font_size_override("font_size", 20)
		icon_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		icon_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		icon_label.position = Vector2(0, 0)
		icon_label.size = Vector2(NODE_SIZE, NODE_SIZE)
		icon_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
		btn.add_child(icon_label)

		# Name below node
		var name_label := Label.new()
		name_label.text = display_name
		name_label.add_theme_font_size_override("font_size", 9)
		name_label.add_theme_color_override("font_color", Color(0.55, 0.62, 0.70))
		name_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		name_label.position = Vector2(-16, NODE_SIZE + 2)
		name_label.size = Vector2(NODE_SIZE + 32, 14)
		name_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
		btn.add_child(name_label)

	_update_node_positions()


func _cache_edges() -> void:
	_edge_pairs.clear()
	for tech_id: String in _tech_defs:
		var def: Dictionary = _tech_defs[tech_id]
		var prereqs: Dictionary = def.get("prereq_logic", {})
		var all_of: Array = prereqs.get("all_of", [])
		var any_of: Array = prereqs.get("any_of", [])
		var soft: Array = prereqs.get("soft", [])

		var to_center: Vector2 = _node_positions_world.get(tech_id, Vector2.ZERO) + Vector2(NODE_SIZE * 0.5, NODE_SIZE * 0.5)

		for prereq_id: String in all_of:
			if _node_positions_world.has(prereq_id):
				var from_center: Vector2 = _node_positions_world[prereq_id] + Vector2(NODE_SIZE * 0.5, NODE_SIZE * 0.5)
				_edge_pairs.append({"from": from_center, "to": to_center, "from_id": prereq_id, "to_id": tech_id, "alpha": EDGE_ALPHA_DEFAULT})

		for prereq_id: String in any_of + soft:
			if _node_positions_world.has(prereq_id):
				var from_center: Vector2 = _node_positions_world[prereq_id] + Vector2(NODE_SIZE * 0.5, NODE_SIZE * 0.5)
				_edge_pairs.append({"from": from_center, "to": to_center, "from_id": prereq_id, "to_id": tech_id, "alpha": EDGE_ALPHA_DEFAULT * 0.5})


# --- Zoom / Pan ---

func _world_to_screen(p: Vector2) -> Vector2:
	return (p * _zoom) + _pan


func _update_node_positions() -> void:
	for tech_id: String in _node_buttons:
		var btn: Button = _node_buttons[tech_id]
		var world_pos: Vector2 = _node_positions_world.get(tech_id, Vector2.ZERO)
		btn.position = _world_to_screen(world_pos)
		btn.scale = Vector2.ONE * _zoom


func _set_zoom(new_zoom: float, pivot: Vector2) -> void:
	var world_before: Vector2 = (pivot - _pan) / _zoom
	_zoom = new_zoom
	_pan = pivot - world_before * _zoom
	queue_redraw()
	_update_node_positions()


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.button_index == MOUSE_BUTTON_WHEEL_UP and mb.pressed:
			_set_zoom(clampf(_zoom + ZOOM_STEP, ZOOM_MIN, ZOOM_MAX), mb.position)
			accept_event()
		elif mb.button_index == MOUSE_BUTTON_WHEEL_DOWN and mb.pressed:
			_set_zoom(clampf(_zoom - ZOOM_STEP, ZOOM_MIN, ZOOM_MAX), mb.position)
			accept_event()
		elif mb.button_index == MOUSE_BUTTON_MIDDLE or mb.button_index == MOUSE_BUTTON_RIGHT:
			if mb.pressed:
				_is_panning = true
				_pan_anchor = mb.position
				_pan_start = _pan
			else:
				_is_panning = false
			accept_event()
		elif mb.button_index == MOUSE_BUTTON_LEFT and mb.pressed:
			if _selected_tech_id != "":
				_selected_tech_id = ""
				_detail_panel.visible = false
				queue_redraw()

	if event is InputEventMouseMotion and _is_panning:
		var mm := event as InputEventMouseMotion
		_pan = _pan_start + (mm.position - _pan_anchor)
		queue_redraw()
		_update_node_positions()
		accept_event()


func _unhandled_input(event: InputEvent) -> void:
	if visible and event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_ESCAPE:
			_on_close()
			get_viewport().set_input_as_handled()


# --- Drawing ---

func _draw() -> void:
	# Background
	draw_rect(Rect2(Vector2.ZERO, size), Color(0.02, 0.03, 0.05, 0.95))

	# Edges
	var active_id: String = _selected_tech_id if _selected_tech_id != "" else _hovered_tech_id
	for edge: Dictionary in _edge_pairs:
		var from_screen: Vector2 = _world_to_screen(edge["from"])
		var to_screen: Vector2 = _world_to_screen(edge["to"])
		var base_alpha: float = float(edge["alpha"])
		var is_connected: bool = false
		if active_id != "":
			is_connected = str(edge["from_id"]) == active_id or str(edge["to_id"]) == active_id
		var color: Color
		if is_connected:
			color = Color(0.45, 0.70, 0.95, EDGE_ALPHA_HIGHLIGHT)
		else:
			color = Color(0.35, 0.55, 0.75, base_alpha)
		draw_line(from_screen, to_screen, color, EDGE_WIDTH)


# --- Node Interaction ---

func _on_node_hover(tech_id: String) -> void:
	_hovered_tech_id = tech_id
	queue_redraw()


func _on_node_unhover() -> void:
	_hovered_tech_id = ""
	queue_redraw()


func _on_node_click(tech_id: String) -> void:
	_selected_tech_id = tech_id
	_show_detail_panel(tech_id)
	queue_redraw()


func _on_close() -> void:
	visible = false


# --- Detail Panel ---

func _show_detail_panel(tech_id: String) -> void:
	for child in _detail_content.get_children():
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

	# Title
	var title := Label.new()
	title.text = display_name
	title.add_theme_font_size_override("font_size", 16)
	title.add_theme_color_override("font_color", Color.WHITE)
	_detail_content.add_child(title)

	# Description
	if not desc.is_empty() and desc != desc_key:
		var desc_label := Label.new()
		desc_label.text = desc
		desc_label.add_theme_font_size_override("font_size", 10)
		desc_label.add_theme_color_override("font_color", Color(0.50, 0.58, 0.66))
		desc_label.autowrap_mode = TextServer.AUTOWRAP_WORD
		_detail_content.add_child(desc_label)

	# Agent proficiency
	var entry: Dictionary = _entity_knowledge.get(tech_id, {})
	if not entry.is_empty():
		var prof: float = clampf(_safe_float(entry.get("proficiency", 0.0)), 0.0, 1.0)
		var source_code: int = int(entry.get("source", 0))
		var prof_label := Label.new()
		prof_label.text = "%s: %d%% — %s: %s" % [
			Locale.ltr("UI_PROFICIENCY"), int(prof * 100),
			Locale.ltr("UI_SOURCE"), _source_icon(source_code),
		]
		prof_label.add_theme_font_size_override("font_size", 11)
		prof_label.add_theme_color_override("font_color", Color(0.45, 0.62, 0.84))
		_detail_content.add_child(prof_label)
	else:
		var unknown_label := Label.new()
		unknown_label.text = Locale.ltr("TECH_NOT_KNOWN")
		unknown_label.add_theme_font_size_override("font_size", 11)
		unknown_label.add_theme_color_override("font_color", Color(0.35, 0.40, 0.48))
		_detail_content.add_child(unknown_label)

	# Prerequisites
	var all_of: Array = prereqs.get("all_of", [])
	if not all_of.is_empty():
		_add_detail_section(Locale.ltr("TECH_PREREQS"))
		for pid: String in all_of:
			var pdef: Dictionary = _tech_defs.get(pid, {})
			var pname: String = Locale.ltr(str(pdef.get("display_key", pid)))
			var has_it: bool = _entity_knowledge.has(pid)
			var check := Label.new()
			check.text = "%s %s" % ["✓" if has_it else "✗", pname]
			check.add_theme_font_size_override("font_size", 10)
			check.add_theme_color_override("font_color", Color(0.28, 0.66, 0.16) if has_it else Color(0.78, 0.22, 0.15))
			_detail_content.add_child(check)

	# Unlocks
	var unlock_lists: Array = [unlocks.get("skills", []), unlocks.get("buildings", []), unlocks.get("actions", [])]
	var all_unlocks: Array = []
	for ul: Array in unlock_lists:
		all_unlocks.append_array(ul)
	if not all_unlocks.is_empty():
		_add_detail_section(Locale.ltr("TECH_UNLOCKS"))
		for unlock_id: String in all_unlocks:
			var u_label := Label.new()
			var u_key: String = unlock_id.to_upper()
			var u_text: String = Locale.ltr(u_key)
			if u_text == u_key:
				u_text = unlock_id.replace("_", " ").capitalize()
			u_label.text = "• " + u_text
			u_label.add_theme_font_size_override("font_size", 10)
			u_label.add_theme_color_override("font_color", Color(0.55, 0.62, 0.70))
			_detail_content.add_child(u_label)

	_detail_panel.visible = true


func _add_detail_section(title_text: String) -> void:
	var spacer := Control.new()
	spacer.custom_minimum_size.y = 6
	_detail_content.add_child(spacer)
	var label := Label.new()
	label.text = title_text
	label.add_theme_font_size_override("font_size", 11)
	label.add_theme_color_override("font_color", Color(0.35, 0.42, 0.50))
	_detail_content.add_child(label)


# --- Agent Knowledge ---

func set_entity(entity_id: int) -> void:
	_selected_entity_id = entity_id
	_entity_knowledge.clear()
	if _sim_engine == null or entity_id < 0:
		_refresh_node_overlays()
		return
	var knowledge_tab: Dictionary = _sim_engine.get_entity_tab(entity_id, "knowledge")
	var known_raw: Variant = knowledge_tab.get("known", [])
	var known: Array = known_raw if known_raw is Array else []
	for entry_raw: Variant in known:
		if entry_raw is Dictionary:
			var entry: Dictionary = entry_raw
			var kid: String = str(entry.get("id", ""))
			if not kid.is_empty():
				_entity_knowledge[kid] = entry
	_refresh_node_overlays()


func _refresh_node_overlays() -> void:
	for tech_id: String in _node_buttons:
		var btn: Button = _node_buttons[tech_id]
		var def: Dictionary = _tech_defs.get(tech_id, {})
		var entry: Dictionary = _entity_knowledge.get(tech_id, {})
		var cat_color: Color = _category_color(def.get("categories", []))

		if not entry.is_empty():
			var prof: float = clampf(_safe_float(entry.get("proficiency", 0.0)), 0.0, 1.0)
			var bright: float = 0.3 + prof * 0.7
			_apply_node_style(btn, Color(cat_color.r * bright, cat_color.g * bright, cat_color.b * bright))
		else:
			_apply_node_style(btn, Color(0.12, 0.14, 0.18))
	queue_redraw()


# --- Helpers ---

func _category_color(categories: Array) -> Color:
	if categories.is_empty():
		return Color(0.35, 0.40, 0.50)
	return CATEGORY_COLORS.get(str(categories[0]), Color(0.35, 0.40, 0.50))


func _apply_node_style(btn: Button, bg_color: Color) -> void:
	var key: String = "%d_%d_%d" % [int(bg_color.r * 20), int(bg_color.g * 20), int(bg_color.b * 20)]
	var sb: StyleBoxFlat
	if _style_cache.has(key):
		sb = _style_cache[key]
	else:
		sb = StyleBoxFlat.new()
		sb.bg_color = bg_color
		sb.border_color = Color(minf(bg_color.r + 0.1, 1.0), minf(bg_color.g + 0.1, 1.0), minf(bg_color.b + 0.1, 1.0), 0.6)
		sb.set_border_width_all(1)
		sb.set_corner_radius_all(6)
		_style_cache[key] = sb
	for state: String in ["normal", "hover", "pressed", "disabled", "focus"]:
		btn.add_theme_stylebox_override(state, sb)


func _safe_float(raw: Variant) -> float:
	if raw is float or raw is int:
		return float(raw)
	return 0.0


func _source_icon(source_code: int) -> String:
	match source_code:
		0: return "🗣️"
		1: return "👁️"
		2: return "🔨"
		3: return "📜"
		4: return "🏛️"
		5: return "💡"
		_: return "❓"


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
