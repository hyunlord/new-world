extends "res://scripts/ui/panels/entity_detail_panel_v3.gd"
class_name EntityDetailPanelV5

const ValueDefs = preload("res://scripts/core/social/value_defs.gd")

# v5 replaces BBCode RichTextLabel with Godot UI nodes.
# Inherits from v3: _sim_engine, _detail, _mind_tab, _health_tab, etc.
# Inherits from v3: set_entity_id(), _reload_data(), _refresh_all() (overridden)
# Inherits from v3: _safe_panel_scalar(), _localized_action_text(), NEED_ROWS, HEXACO_ROWS, etc.

const SIDEBAR_WIDTH: float = 340.0
const HEADER_HEIGHT: float = 80.0
const TAB_HEIGHT: float = 32.0
const BAR_HEIGHT: float = 18.0
const LABEL_MIN_WIDTH: float = 72.0
const PCT_MIN_WIDTH: float = 38.0
const SECTION_SPACING: float = 12.0
const ROW_SPACING: int = 2

# Colors
const COLOR_BG: Color = Color(0.05, 0.07, 0.10, 0.92)
const COLOR_BORDER: Color = Color(0.20, 0.25, 0.30, 0.70)
const COLOR_LABEL: Color = Color(0.31, 0.41, 0.47)
const COLOR_SECTION_TITLE: Color = Color(0.16, 0.22, 0.28)
const COLOR_ALERT_GOOD: Color = Color(0.28, 0.66, 0.16)
const COLOR_ALERT_BAD: Color = Color(0.78, 0.22, 0.15)
const COLOR_PCT: Color = Color(0.53, 0.60, 0.66)

# Node refs — header
var _scroll: ScrollContainer
var _content: VBoxContainer
var _header_panel: PanelContainer
var _header_name_label: Label
var _header_meta_label: Label
var _header_action_label: Label
var _follow_button: Button
var _favorite_button: Button
var _tab_bar: TabBar
var _tab_container: VBoxContainer

# Tab panels
var _overview_panel: VBoxContainer
var _needs_panel: VBoxContainer

# Overview sub-nodes
var _overview_alert_container: VBoxContainer
var _overview_info_grid: GridContainer
var _overview_info_cells: Array[Label] = []
var _overview_need_rows: Array[Dictionary] = []
var _overview_knowledge_label: Label
var _overview_family_label: Label
var _overview_trait_container: HBoxContainer

# Need rows cache (reusable — update values, don't recreate)
var _v5_need_rows: Array[Dictionary] = []

# Emotion tab
var _emotion_panel: VBoxContainer
var _emotion_rows: Array[Dictionary] = []
var _stress_row: Dictionary = {}

# Personality tab
var _personality_panel: VBoxContainer
var _personality_archetype_label: Label
var _personality_temperament_label: Label
var _tci_rows: Array[Dictionary] = []
var _hexaco_rows: Array[Dictionary] = []
var _trait_tags_label: Label
var _value_rows: Array[Dictionary] = []

# Health tab
var _health_panel: VBoxContainer
var _health_aggregate_row: Dictionary = {}
var _health_group_rows: Array[Dictionary] = []
var _health_derived_rows: Array[Dictionary] = []
var _health_injury_container: VBoxContainer
var _health_injury_rows: Array[Dictionary] = []

# Knowledge tab
var _knowledge_panel: VBoxContainer
var _knowledge_empty_label: Label
var _knowledge_rows: Array[Dictionary] = []
var _knowledge_channel_container: VBoxContainer

# Relationships tab
var _relationships_panel: VBoxContainer
var _relationships_empty_label: Label
var _relationships_container: VBoxContainer

# Inventory tab
var _inventory_panel: VBoxContainer
var _inventory_empty_label: Label
var _inventory_container: GridContainer
var _inv_data_hash: int = 0

# Family tab
var _family_panel: VBoxContainer
var _family_father_label: Label
var _family_mother_label: Label
var _family_spouse_label: Label
var _family_children_label: Label
var _family_kinship_label: Label

# Events tab
var _events_panel: VBoxContainer
var _events_empty_label: Label
var _events_container: VBoxContainer

# Refresh
var _v5_refresh_timer: float = 0.0


# ---------------------------------------------------------------------------
# Lifecycle
# ---------------------------------------------------------------------------

## Override v3.set_entity_id to reset inventory hash on entity change.
func set_entity_id(entity_id: int) -> void:
	_inv_data_hash = -1
	super.set_entity_id(entity_id)


func _process(delta: float) -> void:
	if not visible or _selected_entity_id < 0:
		return
	_v5_refresh_timer += delta
	if _v5_refresh_timer >= 0.5:
		_v5_refresh_timer = 0.0
		if not _is_reloading:
			_reload_data()


# ---------------------------------------------------------------------------
# Data loading — overrides v3._reload_data() (v3 guards on _expand_tabs)
# ---------------------------------------------------------------------------

func _reload_data() -> void:
	if _is_reloading:
		return
	if not is_inside_tree():
		return
	if _sim_engine == null or _selected_entity_id < 0:
		return
	_is_reloading = true
	_detail = _sim_engine.get_entity_detail(_selected_entity_id)
	if _detail.is_empty():
		_is_reloading = false
		visible = false
		return
	_mind_tab = _sim_engine.get_entity_tab(_selected_entity_id, "mind")
	_social_tab = _sim_engine.get_entity_tab(_selected_entity_id, "social")
	_memory_tab = _sim_engine.get_entity_tab(_selected_entity_id, "memory")
	_health_tab = _sim_engine.get_entity_tab(_selected_entity_id, "health")
	_knowledge_tab = _sim_engine.get_entity_tab(_selected_entity_id, "knowledge")
	_family_tab = _sim_engine.get_entity_tab(_selected_entity_id, "family")
	_refresh_all()
	_is_reloading = false


# ---------------------------------------------------------------------------
# UI construction — overrides v3._build_ui() completely
# ---------------------------------------------------------------------------

func _build_ui() -> void:
	var style := StyleBoxFlat.new()
	style.bg_color = COLOR_BG
	style.border_color = COLOR_BORDER
	style.border_width_left = 1
	style.border_width_top = 1
	style.corner_radius_top_left = 6
	style.corner_radius_bottom_left = 6
	style.content_margin_left = 10
	style.content_margin_right = 10
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	add_theme_stylebox_override("panel", style)

	_scroll = ScrollContainer.new()
	_scroll.set_anchors_preset(Control.PRESET_FULL_RECT)
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(_scroll)

	_content = VBoxContainer.new()
	_content.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_content.add_theme_constant_override("separation", 4)
	_scroll.add_child(_content)

	_build_header()
	_build_tab_bar()
	_build_overview_tab()
	_build_needs_tab()
	_build_emotion_tab()
	_build_personality_tab()
	_build_health_tab()
	_build_knowledge_tab()
	_build_relationships_tab()
	_build_inventory_tab()
	_build_family_tab()
	_build_events_tab()

	_switch_tab(0)


# ---------------------------------------------------------------------------
# S2: Header
# ---------------------------------------------------------------------------

func _build_header() -> void:
	_header_panel = PanelContainer.new()
	var header_style := StyleBoxFlat.new()
	header_style.bg_color = Color(0.08, 0.10, 0.14, 0.85)
	header_style.corner_radius_top_left = 4
	header_style.corner_radius_top_right = 4
	header_style.content_margin_left = 8
	header_style.content_margin_right = 8
	header_style.content_margin_top = 6
	header_style.content_margin_bottom = 6
	_header_panel.add_theme_stylebox_override("panel", header_style)
	_content.add_child(_header_panel)

	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 10)
	_header_panel.add_child(hbox)

	# Portrait placeholder (48x48)
	var portrait_container := Control.new()
	portrait_container.custom_minimum_size = Vector2(48, 48)
	hbox.add_child(portrait_container)

	var info_vbox := VBoxContainer.new()
	info_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	info_vbox.add_theme_constant_override("separation", 1)
	hbox.add_child(info_vbox)

	_header_name_label = Label.new()
	_header_name_label.add_theme_font_size_override("font_size", 14)
	_header_name_label.add_theme_color_override("font_color", Color.WHITE)
	info_vbox.add_child(_header_name_label)

	_header_meta_label = Label.new()
	_header_meta_label.add_theme_font_size_override("font_size", 10)
	_header_meta_label.add_theme_color_override("font_color", Color(0.6, 0.65, 0.7))
	info_vbox.add_child(_header_meta_label)

	_header_action_label = Label.new()
	_header_action_label.add_theme_font_size_override("font_size", 10)
	_header_action_label.add_theme_color_override("font_color", Color(0.5, 0.58, 0.65))
	_header_action_label.custom_minimum_size.y = 30.0
	info_vbox.add_child(_header_action_label)

	# Action buttons (right side of header)
	var action_col := VBoxContainer.new()
	action_col.add_theme_constant_override("separation", 4)
	hbox.add_child(action_col)

	_follow_button = Button.new()
	_follow_button.text = Locale.ltr("UI_BTN_FOLLOW")
	_follow_button.custom_minimum_size = Vector2(28, 28)
	_follow_button.add_theme_font_size_override("font_size", 10)
	_follow_button.pressed.connect(_on_follow_pressed)
	action_col.add_child(_follow_button)

	_favorite_button = Button.new()
	_favorite_button.text = Locale.ltr("UI_BTN_FAVORITE")
	_favorite_button.custom_minimum_size = Vector2(28, 28)
	_favorite_button.add_theme_font_size_override("font_size", 10)
	_favorite_button.pressed.connect(_on_favorite_pressed)
	action_col.add_child(_favorite_button)


# ---------------------------------------------------------------------------
# S5: Tab bar
# ---------------------------------------------------------------------------

func _build_tab_bar() -> void:
	_tab_bar = TabBar.new()
	_tab_bar.tab_alignment = TabBar.ALIGNMENT_LEFT
	_tab_bar.add_theme_font_size_override("font_size", 10)
	_tab_bar.add_tab(Locale.ltr("PANEL_OVERVIEW_TITLE"))
	_tab_bar.add_tab(Locale.ltr("PANEL_NEEDS_TITLE"))
	_tab_bar.add_tab(Locale.ltr("PANEL_EMOTION_TITLE"))
	_tab_bar.add_tab(Locale.ltr("PANEL_PERSONALITY_TITLE"))
	_tab_bar.add_tab(Locale.ltr("PANEL_HEALTH_TITLE"))
	_tab_bar.add_tab(Locale.ltr("PANEL_KNOWLEDGE_TITLE"))
	_tab_bar.add_tab(Locale.ltr("PANEL_RELATIONSHIPS_TITLE"))
	_tab_bar.add_tab(Locale.ltr("UI_INVENTORY"))
	_tab_bar.add_tab(Locale.ltr("PANEL_FAMILY_TITLE"))
	_tab_bar.add_tab(Locale.ltr("PANEL_EVENTS_TITLE"))
	_tab_bar.tab_changed.connect(_on_tab_changed)
	_content.add_child(_tab_bar)

	_tab_container = VBoxContainer.new()
	_tab_container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_tab_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_content.add_child(_tab_container)


func _on_tab_changed(tab_index: int) -> void:
	_switch_tab(tab_index)
	_refresh_all()


func _switch_tab(index: int) -> void:
	if _overview_panel != null:
		_overview_panel.visible = (index == 0)
	if _needs_panel != null:
		_needs_panel.visible = (index == 1)
	if _emotion_panel != null:
		_emotion_panel.visible = (index == 2)
	if _personality_panel != null:
		_personality_panel.visible = (index == 3)
	if _health_panel != null:
		_health_panel.visible = (index == 4)
	if _knowledge_panel != null:
		_knowledge_panel.visible = (index == 5)
	if _relationships_panel != null:
		_relationships_panel.visible = (index == 6)
	if _inventory_panel != null:
		_inventory_panel.visible = (index == 7)
	if _family_panel != null:
		_family_panel.visible = (index == 8)
	if _events_panel != null:
		_events_panel.visible = (index == 9)


# ---------------------------------------------------------------------------
# S3: Overview tab
# ---------------------------------------------------------------------------

func _build_overview_tab() -> void:
	_overview_panel = VBoxContainer.new()
	_overview_panel.add_theme_constant_override("separation", 6)
	_tab_container.add_child(_overview_panel)

	# --- Alert Cards Section ---
	_add_section_title(_overview_panel, "PANEL_OVERVIEW_ALERTS")
	_overview_alert_container = VBoxContainer.new()
	_overview_alert_container.add_theme_constant_override("separation", 2)
	_overview_panel.add_child(_overview_alert_container)

	# --- Basic Info 2x2 Grid ---
	_add_section_title(_overview_panel, "PANEL_OVERVIEW_BASIC")
	_overview_info_grid = GridContainer.new()
	_overview_info_grid.columns = 2
	_overview_info_grid.add_theme_constant_override("h_separation", 4)
	_overview_info_grid.add_theme_constant_override("v_separation", 3)
	_overview_panel.add_child(_overview_info_grid)

	_overview_info_cells.clear()
	for _idx: int in range(4):
		var cell := PanelContainer.new()
		var cell_style := StyleBoxFlat.new()
		cell_style.bg_color = Color(0.03, 0.04, 0.06, 1.0)
		cell_style.set_corner_radius_all(2)
		cell_style.content_margin_left = 5
		cell_style.content_margin_right = 5
		cell_style.content_margin_top = 3
		cell_style.content_margin_bottom = 3
		cell.add_theme_stylebox_override("panel", cell_style)
		cell.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		_overview_info_grid.add_child(cell)

		var cell_label := Label.new()
		cell_label.add_theme_font_size_override("font_size", 10)
		cell_label.add_theme_color_override("font_color", Color(0.66, 0.72, 0.78))
		cell.add_child(cell_label)
		_overview_info_cells.append(cell_label)

	# --- Needs Bars ---
	_add_section_title(_overview_panel, "PANEL_OVERVIEW_NEEDS")
	for i in range(4):
		_overview_need_rows.append(_create_bar_row(_overview_panel))

	# --- Knowledge Summary ---
	_add_section_title(_overview_panel, "PANEL_OVERVIEW_KNOWLEDGE")
	_overview_knowledge_label = Label.new()
	_overview_knowledge_label.add_theme_font_size_override("font_size", 10)
	_overview_knowledge_label.add_theme_color_override("font_color", Color(0.44, 0.53, 0.60))
	_overview_panel.add_child(_overview_knowledge_label)

	# --- Family Summary ---
	_add_section_title(_overview_panel, "PANEL_OVERVIEW_FAMILY")
	_overview_family_label = Label.new()
	_overview_family_label.add_theme_font_size_override("font_size", 10)
	_overview_family_label.add_theme_color_override("font_color", Color(0.44, 0.53, 0.60))
	_overview_panel.add_child(_overview_family_label)

	# --- Trait Chips ---
	_add_section_title(_overview_panel, "UI_TRAITS_TITLE")
	_overview_trait_container = HBoxContainer.new()
	_overview_trait_container.add_theme_constant_override("separation", 3)
	_overview_panel.add_child(_overview_trait_container)


# ---------------------------------------------------------------------------
# S4: Needs tab
# ---------------------------------------------------------------------------

func _build_needs_tab() -> void:
	_needs_panel = VBoxContainer.new()
	_needs_panel.add_theme_constant_override("separation", ROW_SPACING)
	_tab_container.add_child(_needs_panel)

	for row: Dictionary in NEED_ROWS:
		_v5_need_rows.append(_create_bar_row(_needs_panel))


# ---------------------------------------------------------------------------
# Shared bar row component — creates HBox with Label + ProgressBar + pct Label
# ---------------------------------------------------------------------------

func _create_bar_row(parent: Control) -> Dictionary:
	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 4)
	parent.add_child(hbox)

	var label := Label.new()
	label.custom_minimum_size.x = LABEL_MIN_WIDTH
	label.add_theme_font_size_override("font_size", 10)
	label.add_theme_color_override("font_color", COLOR_LABEL)
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	hbox.add_child(label)

	var bar: ProgressBar = ProgressBar.new()
	bar.min_value = 0.0
	bar.max_value = 1.0
	bar.step = 0.01
	bar.show_percentage = false
	bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	bar.custom_minimum_size.y = BAR_HEIGHT

	var bar_bg := StyleBoxFlat.new()
	bar_bg.bg_color = Color(0.09, 0.14, 0.19)
	bar_bg.corner_radius_top_left = 2
	bar_bg.corner_radius_top_right = 2
	bar_bg.corner_radius_bottom_left = 2
	bar_bg.corner_radius_bottom_right = 2
	bar.add_theme_stylebox_override("background", bar_bg)

	var bar_fill := StyleBoxFlat.new()
	bar_fill.bg_color = Color(0.35, 0.80, 0.43)
	bar_fill.corner_radius_top_left = 2
	bar_fill.corner_radius_top_right = 2
	bar_fill.corner_radius_bottom_left = 2
	bar_fill.corner_radius_bottom_right = 2
	bar.add_theme_stylebox_override("fill", bar_fill)
	hbox.add_child(bar)

	var pct := Label.new()
	pct.custom_minimum_size.x = PCT_MIN_WIDTH
	pct.add_theme_font_size_override("font_size", 10)
	pct.add_theme_color_override("font_color", COLOR_PCT)
	pct.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	hbox.add_child(pct)

	return {"container": hbox, "label": label, "bar": bar, "pct": pct, "fill_style": bar_fill}


func _update_bar_row(row: Dictionary, entry: Dictionary) -> void:
	var label_node: Label = row.label
	var bar_node: ProgressBar = row.bar
	var pct_node: Label = row.pct
	var fill: StyleBoxFlat = row.fill_style
	var value: float = clampf(float(entry.get("value", 0.0)), 0.0, 1.0)
	label_node.text = str(entry.get("label", ""))
	bar_node.value = value
	pct_node.text = "%d%%" % int(round(value * 100.0))
	if entry.has("color"):
		fill.bg_color = entry.color
	else:
		fill.bg_color = _need_color(value)


# ---------------------------------------------------------------------------
# Refresh — overrides v3._refresh_all()
# ---------------------------------------------------------------------------

func _refresh_all() -> void:
	if not is_inside_tree():
		return
	if _detail.is_empty():
		return
	_refresh_header()
	# Only refresh the currently visible tab — avoids unnecessary queue_free
	# on hidden tabs which causes layout jitter and kills tooltips
	if _tab_bar == null:
		return
	match _tab_bar.current_tab:
		0: _refresh_overview()
		1: _refresh_needs()
		2: _refresh_emotion()
		3: _refresh_personality()
		4: _refresh_health()
		5: _refresh_knowledge()
		6: _refresh_relationships()
		7: _refresh_inventory()
		8: _refresh_family()
		9: _refresh_events()


func _refresh_header() -> void:
	if _header_name_label == null or _detail.is_empty():
		return
	var name_text: String = str(_detail.get("name", "???"))
	var archetype_key: String = str(_detail.get("archetype_key", "ARCHETYPE_QUIET_OBSERVER"))
	_header_name_label.text = "%s [%s]" % [name_text, Locale.ltr(archetype_key)]

	var age: int = int(round(_safe_panel_scalar(_detail.get("age_years", 0.0), 0.0)))
	var sex_key: String = "UI_MALE" if str(_detail.get("sex", "")).to_lower() == "male" else "UI_FEMALE"
	var stage_key: String = "STAGE_" + str(_detail.get("growth_stage", "adult")).to_upper()
	var occ_raw: String = str(_detail.get("occupation", "none")).strip_edges()
	if occ_raw.is_empty():
		occ_raw = "none"
	var occ_key: String = "OCCUPATION_" + occ_raw.to_upper()
	_header_meta_label.text = "%d%s · %s · %s · %s" % [age, Locale.ltr("UI_AGE_UNIT"), Locale.ltr(stage_key), Locale.ltr(sex_key), Locale.ltr(occ_key)]

	var action_text: String = _localized_action_text(str(_detail.get("current_action", "Idle")))
	var motivation: String = _localized_need_text(str(_detail.get("top_need_key", "NEED_ENERGY")))
	var timer_current: int = int(_safe_panel_scalar(_detail.get("action_timer", 0), 0))
	var timer_total: int = int(_safe_panel_scalar(_detail.get("action_duration", 0), 0))
	var action_line: String = action_text + " — " + motivation
	if timer_total > 0:
		action_line += "\n" + Locale.trf2("UI_ACTION_TIMER_FMT", "current", timer_current, "total", timer_total)
	else:
		action_line += "\n "
	_header_action_label.text = action_line

	# Update follow button icon based on camera state
	if _follow_button != null:
		var cam: Camera2D = get_viewport().get_camera_2d()
		var following: bool = cam != null and cam.has_method("is_following") and cam.is_following()
		_follow_button.text = Locale.ltr("UI_BTN_FOLLOWING") if following else Locale.ltr("UI_BTN_FOLLOW")


func _refresh_overview() -> void:
	if _overview_panel == null or _detail.is_empty():
		return

	# --- 1. Alert Cards ---
	for child in _overview_alert_container.get_children():
		child.queue_free()

	var alerts: Array[Dictionary] = []
	var hunger: float = _safe_float(_detail, "need_hunger", 1.0)
	var sleep_val: float = _safe_float(_detail, "need_sleep", 1.0)
	var stress: float = _normalized_stress()
	var health_tab_data: Dictionary = _health_tab if not _health_tab.is_empty() else {}
	var damaged: Array = health_tab_data.get("damaged_parts", []) if health_tab_data is Dictionary else []

	if hunger < 0.35:
		alerts.append({"text": Locale.ltr("ALERT_HUNGRY"), "detail": Locale.ltr("ALERT_HUNGRY_DETAIL"), "color": COLOR_ALERT_BAD})
	if sleep_val < 0.30:
		alerts.append({"text": Locale.ltr("ALERT_TIRED"), "detail": Locale.ltr("ALERT_TIRED_DETAIL"), "color": Color(0.78, 0.55, 0.10)})
	if stress > 0.30:
		alerts.append({"text": Locale.ltr("ALERT_STRESSED"), "detail": Locale.ltr("ALERT_STRESSED_DETAIL"), "color": COLOR_ALERT_BAD})
	if damaged is Array and not damaged.is_empty():
		alerts.append({"text": Locale.ltr("ALERT_INJURED"), "detail": Locale.trf1("ALERT_INJURED_DETAIL", "n", damaged.size()), "color": COLOR_ALERT_BAD})

	if alerts.is_empty():
		_overview_alert_container.add_child(_create_alert_card(Locale.ltr("ALERT_ALL_GOOD"), "", COLOR_ALERT_GOOD))
	else:
		for alert: Dictionary in alerts:
			_overview_alert_container.add_child(_create_alert_card(str(alert["text"]), str(alert["detail"]), alert["color"]))

	# --- 2. Info Grid ---
	if _overview_info_cells.size() >= 4:
		var occ_raw: String = str(_detail.get("occupation", "none")).strip_edges()
		if occ_raw.is_empty():
			occ_raw = "none"
		var occ: String = Locale.ltr("OCCUPATION_" + occ_raw.to_upper())
		var age: int = int(round(_safe_panel_scalar(_detail.get("age_years", 0.0), 0.0)))
		var action: String = _localized_action_text(str(_detail.get("current_action", "Idle")))
		var band: String = _band_label()
		_overview_info_cells[0].text = "%s %s" % [Locale.ltr("UI_JOB"), occ]
		_overview_info_cells[1].text = "%s %d%s" % [Locale.ltr("UI_AGE"), age, Locale.ltr("UI_AGE_UNIT")]
		_overview_info_cells[2].text = "%s %s" % [Locale.ltr("UI_ACTION"), action]
		_overview_info_cells[3].text = "%s %s" % [Locale.ltr("UI_BAND"), band]

	# --- 3. Need Bars ---
	var entries: Array[Dictionary] = _build_sorted_need_entries(4)
	for i in range(4):
		if i < entries.size():
			_update_bar_row(_overview_need_rows[i], entries[i])
			_overview_need_rows[i].container.visible = true
		else:
			_overview_need_rows[i].container.visible = false

	# --- 4. Knowledge Summary ---
	if _overview_knowledge_label != null:
		var k_count: int = int(_safe_panel_scalar(_detail.get("knowledge_count", 0), 0))
		var is_learning: bool = bool(_detail.get("is_learning", false))
		var k_text: String = Locale.trf1("OVERVIEW_KNOWLEDGE_FMT", "n", k_count)
		if is_learning:
			k_text += " · " + Locale.ltr("OVERVIEW_LEARNING")
		_overview_knowledge_label.text = k_text

	# --- 5. Family Summary ---
	if _overview_family_label != null:
		var has_spouse: bool = bool(_detail.get("has_spouse", false))
		var child_count: int = int(_safe_panel_scalar(_detail.get("children_count", 0), 0))
		var spouse_text: String = Locale.ltr("OVERVIEW_MARRIED") if has_spouse else Locale.ltr("OVERVIEW_SINGLE")
		var f_text: String = spouse_text
		if child_count > 0:
			f_text += " · " + Locale.trf1("OVERVIEW_CHILDREN_FMT", "n", child_count)
		_overview_family_label.text = f_text

	# --- 6. Trait Chips ---
	if _overview_trait_container != null:
		for child in _overview_trait_container.get_children():
			child.queue_free()
		var tags: PackedStringArray = _build_trait_tag_texts()
		if tags.is_empty():
			var none_label := Label.new()
			none_label.text = "—"
			none_label.add_theme_font_size_override("font_size", 9)
			none_label.add_theme_color_override("font_color", Color(0.22, 0.28, 0.31))
			_overview_trait_container.add_child(none_label)
		else:
			for tag_text: String in tags:
				_overview_trait_container.add_child(_create_trait_chip(tag_text))


func _refresh_needs() -> void:
	if _needs_panel == null or _detail.is_empty():
		return
	var sorted_entries: Array[Dictionary] = []
	for row: Dictionary in NEED_ROWS:
		sorted_entries.append({
			"label": Locale.ltr(str(row["key"])),
			"value": _safe_float(_detail, str(row["field"]), 0.0),
		})
	sorted_entries.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		return float(a.get("value", 0.0)) < float(b.get("value", 0.0)))
	for i in range(_v5_need_rows.size()):
		if i < sorted_entries.size():
			_update_bar_row(_v5_need_rows[i], sorted_entries[i])
		else:
			_v5_need_rows[i].container.visible = false


# ---------------------------------------------------------------------------
# Emotion tab
# ---------------------------------------------------------------------------

func _build_emotion_tab() -> void:
	_emotion_panel = VBoxContainer.new()
	_emotion_panel.add_theme_constant_override("separation", ROW_SPACING)
	_tab_container.add_child(_emotion_panel)

	var title := Label.new()
	title.text = Locale.ltr("PANEL_EMOTION_TITLE")
	title.add_theme_font_size_override("font_size", 11)
	title.add_theme_color_override("font_color", COLOR_SECTION_TITLE)
	_emotion_panel.add_child(title)

	for row: Dictionary in EMOTION_ROWS:
		_emotion_rows.append(_create_bar_row(_emotion_panel))

	var spacer := Control.new()
	spacer.custom_minimum_size.y = 6.0
	_emotion_panel.add_child(spacer)

	_stress_row = _create_bar_row(_emotion_panel)


func _refresh_emotion() -> void:
	if _emotion_panel == null or _detail.is_empty():
		return
	for i in range(EMOTION_ROWS.size()):
		if i >= _emotion_rows.size():
			break
		var row_def: Dictionary = EMOTION_ROWS[i]
		var value: float = _safe_float(_detail, str(row_def["field"]), 0.0)
		var key_str: String = str(row_def["key"])
		_update_bar_row(_emotion_rows[i], {
			"label": Locale.ltr(key_str),
			"value": value,
			"color": _emotion_to_color(key_str),
		})
	var stress: float = _normalized_stress()
	_update_bar_row(_stress_row, {
		"label": Locale.ltr("UI_STRESS"),
		"value": stress,
		"color": Color(0.78, 0.34, 0.28),
	})


# ---------------------------------------------------------------------------
# Personality tab
# ---------------------------------------------------------------------------

func _build_personality_tab() -> void:
	_personality_panel = VBoxContainer.new()
	_personality_panel.add_theme_constant_override("separation", ROW_SPACING)
	_tab_container.add_child(_personality_panel)

	_personality_archetype_label = Label.new()
	_personality_archetype_label.add_theme_font_size_override("font_size", 11)
	_personality_archetype_label.add_theme_color_override("font_color", Color.WHITE)
	_personality_archetype_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	_personality_panel.add_child(_personality_archetype_label)

	_personality_temperament_label = Label.new()
	_personality_temperament_label.add_theme_font_size_override("font_size", 10)
	_personality_temperament_label.add_theme_color_override("font_color", Color(0.44, 0.53, 0.63))
	_personality_panel.add_child(_personality_temperament_label)

	_add_section_spacer(_personality_panel)

	_add_section_title(_personality_panel, "UI_TCI_TITLE")
	for i in range(4):
		_tci_rows.append(_create_bar_row(_personality_panel))

	_add_section_spacer(_personality_panel)

	_add_section_title(_personality_panel, "UI_HEXACO_TITLE")
	for row: Dictionary in HEXACO_ROWS:
		_hexaco_rows.append(_create_bar_row(_personality_panel))

	_add_section_spacer(_personality_panel)

	_add_section_title(_personality_panel, "UI_TRAITS_TITLE")
	_trait_tags_label = Label.new()
	_trait_tags_label.add_theme_font_size_override("font_size", 10)
	_trait_tags_label.add_theme_color_override("font_color", Color(0.41, 0.53, 0.66))
	_trait_tags_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	_personality_panel.add_child(_trait_tags_label)

	_add_section_spacer(_personality_panel)

	_add_section_title(_personality_panel, "UI_VALUES_TITLE")
	for i in range(5):
		_value_rows.append(_create_bar_row(_personality_panel))


func _refresh_personality() -> void:
	if _personality_panel == null or _detail.is_empty():
		return

	var archetype_key: String = str(_detail.get("archetype_key", "ARCHETYPE_QUIET_OBSERVER"))
	_personality_archetype_label.text = "%s: %s" % [Locale.ltr("PANEL_PERSONALITY_TITLE"), Locale.ltr(archetype_key)]
	var temp_key: String = str(_detail.get("temperament_label_key", ""))
	_personality_temperament_label.text = Locale.ltr(temp_key) if not temp_key.is_empty() else ""
	_personality_temperament_label.visible = not temp_key.is_empty()

	var tci_fields: Array[String] = ["tci_ns", "tci_ha", "tci_rd", "tci_p"]
	var tci_keys: Array[String] = ["UI_TCI_NS", "UI_TCI_HA", "UI_TCI_RD", "UI_TCI_P"]
	var tci_colors: Array[Color] = [Color(0.51, 0.62, 0.78), Color(0.68, 0.58, 0.80), Color(0.62, 0.74, 0.48), Color(0.80, 0.65, 0.36)]
	for i in range(mini(_tci_rows.size(), 4)):
		_update_bar_row(_tci_rows[i], {
			"label": Locale.ltr(tci_keys[i]),
			"value": _safe_float(_detail, tci_fields[i], 0.5),
			"color": tci_colors[i],
		})

	for i in range(mini(_hexaco_rows.size(), HEXACO_ROWS.size())):
		var field: String = str(HEXACO_ROWS[i]["field"])
		_update_bar_row(_hexaco_rows[i], {
			"label": Locale.ltr(str(HEXACO_ROWS[i]["key"])),
			"value": _safe_float(_detail, field, 0.0),
			"color": Color(0.41, 0.53, 0.66),
		})

	var tags: PackedStringArray = _build_trait_tag_texts()
	_trait_tags_label.text = ", ".join(tags) if not tags.is_empty() else "—"

	var ranked: Array[Dictionary] = _value_rankings()
	for i in range(5):
		if i < ranked.size():
			_update_bar_row(_value_rows[i], {
				"label": Locale.ltr(str(ranked[i].get("key", "UI_UNKNOWN"))),
				"value": clampf(float(ranked[i].get("value", 0.0)), 0.0, 1.0),
				"color": Color(0.66, 0.60, 0.28),
			})
			_value_rows[i].container.visible = true
		else:
			_value_rows[i].container.visible = false


# ---------------------------------------------------------------------------
# Health tab
# ---------------------------------------------------------------------------

func _build_health_tab() -> void:
	_health_panel = VBoxContainer.new()
	_health_panel.add_theme_constant_override("separation", ROW_SPACING)
	_tab_container.add_child(_health_panel)

	_add_section_title(_health_panel, "PANEL_HEALTH_AGGREGATE")
	_health_aggregate_row = _create_bar_row(_health_panel)

	_add_section_spacer(_health_panel)

	_add_section_title(_health_panel, "PANEL_HEALTH_GROUPS")
	for i in range(8):
		_health_group_rows.append(_create_bar_row(_health_panel))

	_add_section_spacer(_health_panel)

	_add_section_title(_health_panel, "UI_DERIVED_STATS")
	for i in range(4):
		_health_derived_rows.append(_create_bar_row(_health_panel))

	_health_injury_container = VBoxContainer.new()
	_health_injury_container.add_theme_constant_override("separation", ROW_SPACING)
	_health_panel.add_child(_health_injury_container)


func _refresh_health() -> void:
	if _health_panel == null or _health_tab.is_empty():
		return

	var agg_hp: float = _safe_float(_health_tab, "aggregate_hp", 1.0)
	_update_bar_row(_health_aggregate_row, {
		"label": Locale.ltr("PANEL_HEALTH_AGGREGATE"),
		"value": agg_hp,
		"color": _need_color(agg_hp),
	})

	var groups: Array[Dictionary] = _merged_health_groups()
	for i in range(mini(_health_group_rows.size(), groups.size())):
		var hp: float = clampf(float(groups[i].get("value", 1.0)), 0.0, 1.0)
		_update_bar_row(_health_group_rows[i], {
			"label": Locale.ltr(str(groups[i].get("label", ""))),
			"value": hp,
			"color": _need_color(hp),
		})

	var move_mult: float = clampf(_safe_scalar(_health_tab.get("move_mult", 1.0), 1.0) / 1.5, 0.0, 1.0)
	var work_mult: float = clampf(_safe_scalar(_health_tab.get("work_mult", 1.0), 1.0) / 1.5, 0.0, 1.0)
	var combat_mult: float = clampf(_safe_scalar(_health_tab.get("combat_mult", 1.0), 1.0) / 1.5, 0.0, 1.0)
	var pain: float = _safe_float(_health_tab, "pain", 0.0)
	var derived: Array[Dictionary] = [
		{"label": Locale.ltr("UI_MOVE"), "value": move_mult, "color": Color(0.36, 0.76, 0.48)},
		{"label": Locale.ltr("UI_WORK"), "value": work_mult, "color": Color(0.42, 0.56, 0.82)},
		{"label": Locale.ltr("UI_COMBAT"), "value": combat_mult, "color": Color(0.82, 0.34, 0.28)},
		{"label": Locale.ltr("UI_PAIN"), "value": pain, "color": Color(0.86, 0.68, 0.24)},
	]
	for i in range(mini(_health_derived_rows.size(), derived.size())):
		_update_bar_row(_health_derived_rows[i], derived[i])

	for child in _health_injury_container.get_children():
		child.queue_free()
	_health_injury_rows.clear()
	var damaged: Array = _health_tab.get("damaged_parts", [])
	if not damaged.is_empty():
		_add_section_title(_health_injury_container, "PANEL_HEALTH_INJURIES")
		for part_raw: Variant in damaged:
			if not (part_raw is Dictionary):
				continue
			var part: Dictionary = part_raw
			var hp: float = clampf(_safe_scalar(part.get("hp", 0), 0.0) / 100.0, 0.0, 1.0)
			var vital: bool = bool(part.get("vital", false))
			var part_name: String = _localized_body_part_name(str(part.get("name", "")))
			if vital:
				part_name = "⚠ " + part_name
			var row: Dictionary = _create_bar_row(_health_injury_container)
			_update_bar_row(row, {"label": part_name, "value": hp, "color": _need_color(hp)})
			_health_injury_rows.append(row)


# ---------------------------------------------------------------------------
# Knowledge tab
# ---------------------------------------------------------------------------

func _build_knowledge_tab() -> void:
	_knowledge_panel = VBoxContainer.new()
	_knowledge_panel.add_theme_constant_override("separation", ROW_SPACING)
	_tab_container.add_child(_knowledge_panel)

	_add_section_title(_knowledge_panel, "PANEL_KNOWLEDGE_TITLE")

	_knowledge_empty_label = Label.new()
	_knowledge_empty_label.text = Locale.ltr("UI_NO_KNOWLEDGE")
	_knowledge_empty_label.add_theme_font_size_override("font_size", 10)
	_knowledge_empty_label.add_theme_color_override("font_color", Color(0.22, 0.28, 0.31))
	_knowledge_panel.add_child(_knowledge_empty_label)

	_knowledge_channel_container = VBoxContainer.new()
	_knowledge_channel_container.add_theme_constant_override("separation", ROW_SPACING)
	_knowledge_panel.add_child(_knowledge_channel_container)


func _refresh_knowledge() -> void:
	if _knowledge_panel == null:
		return
	for child in _knowledge_channel_container.get_children():
		child.queue_free()
	_knowledge_rows.clear()

	var known_raw: Variant = _knowledge_tab.get("known", [])
	var known: Array = known_raw if known_raw is Array else []
	_knowledge_empty_label.visible = known.is_empty()
	if known.is_empty():
		return

	for knowledge_raw: Variant in known:
		if not (knowledge_raw is Dictionary):
			continue
		var knowledge: Dictionary = knowledge_raw
		var knowledge_id: String = str(knowledge.get("id", ""))
		var display_name: String = _display_token(knowledge_id)
		var proficiency: float = clampf(_safe_scalar(knowledge.get("proficiency", 0.0), 0.0), 0.0, 1.0)
		var row: Dictionary = _create_bar_row(_knowledge_channel_container)
		_update_bar_row(row, {
			"label": display_name,
			"value": proficiency,
			"color": Color(0.45, 0.62, 0.84),
		})
		_knowledge_rows.append(row)

	_add_section_spacer(_knowledge_channel_container)
	_add_section_title(_knowledge_channel_container, "PANEL_KNOWLEDGE_CHANNELS")
	for channel: Dictionary in _knowledge_channels():
		var channel_label := Label.new()
		var icon: String = str(channel.get("icon", ""))
		var name_text: String = Locale.ltr(str(channel.get("label", "")))
		var status_text: String = Locale.ltr(str(channel.get("status", "")))
		var locked: bool = bool(channel.get("locked", false))
		channel_label.text = "%s %s — %s" % [icon, name_text, status_text]
		channel_label.add_theme_font_size_override("font_size", 10)
		channel_label.add_theme_color_override("font_color", Color(0.35, 0.40, 0.48) if locked else Color(0.50, 0.60, 0.70))
		_knowledge_channel_container.add_child(channel_label)


# ---------------------------------------------------------------------------
# Relationships tab
# ---------------------------------------------------------------------------

func _build_relationships_tab() -> void:
	_relationships_panel = VBoxContainer.new()
	_relationships_panel.add_theme_constant_override("separation", 4)
	_tab_container.add_child(_relationships_panel)

	_add_section_title(_relationships_panel, "PANEL_RELATIONSHIPS_TITLE")

	_relationships_empty_label = Label.new()
	_relationships_empty_label.text = Locale.ltr("UI_NO_RELATIONSHIPS")
	_relationships_empty_label.add_theme_font_size_override("font_size", 10)
	_relationships_empty_label.add_theme_color_override("font_color", Color(0.22, 0.28, 0.31))
	_relationships_panel.add_child(_relationships_empty_label)

	_relationships_container = VBoxContainer.new()
	_relationships_container.add_theme_constant_override("separation", 6)
	_relationships_panel.add_child(_relationships_container)


func _refresh_relationships() -> void:
	if _relationships_panel == null:
		return
	for child in _relationships_container.get_children():
		child.queue_free()

	var entries: Array[Dictionary] = _build_relationship_entries(15)
	_relationships_empty_label.visible = entries.is_empty()
	if entries.is_empty():
		return

	for entry: Dictionary in entries:
		var row_panel := PanelContainer.new()
		var row_style := StyleBoxFlat.new()
		row_style.bg_color = Color(0.08, 0.10, 0.14, 0.6)
		row_style.corner_radius_top_left = 3
		row_style.corner_radius_top_right = 3
		row_style.corner_radius_bottom_left = 3
		row_style.corner_radius_bottom_right = 3
		row_style.content_margin_left = 6
		row_style.content_margin_right = 6
		row_style.content_margin_top = 4
		row_style.content_margin_bottom = 4
		row_panel.add_theme_stylebox_override("panel", row_style)
		_relationships_container.add_child(row_panel)

		var vbox := VBoxContainer.new()
		vbox.add_theme_constant_override("separation", 1)
		row_panel.add_child(vbox)

		var name_label := Label.new()
		var target_name: String = _resolve_entity_name(int(entry.get("target_id", -1)))
		var relation_type: String = str(entry.get("relation_type", ""))
		var relation_text: String = _localized_relation_text(relation_type)
		var markers: PackedStringArray = PackedStringArray()
		if bool(entry.get("is_band_mate", false)):
			markers.append("[B]")
		var marker_str: String = (" ".join(markers) + " ") if not markers.is_empty() else ""
		var display: String = marker_str + target_name
		if not relation_text.is_empty():
			display += " (%s)" % relation_text
		name_label.text = display
		name_label.add_theme_font_size_override("font_size", 10)
		name_label.add_theme_color_override("font_color", Color.WHITE)
		_make_clickable_label(name_label, int(entry.get("target_id", -1)))
		vbox.add_child(name_label)

		var stats_label := Label.new()
		var affinity: int = int(round(_safe_panel_scalar(entry.get("affinity", 0.0), 0.0) * 100.0))
		var trust: int = int(round(_safe_panel_scalar(entry.get("trust", 0.0), 0.0) * 100.0))
		stats_label.text = "%s %+d   %s %d" % [Locale.ltr("UI_AFFINITY"), affinity, Locale.ltr("UI_TRUST"), trust]
		stats_label.add_theme_font_size_override("font_size", 9)
		stats_label.add_theme_color_override("font_color", Color(0.45, 0.55, 0.65))
		vbox.add_child(stats_label)


# ---------------------------------------------------------------------------
# Inventory tab
# ---------------------------------------------------------------------------

const INV_SLOT_SIZE: float = 36.0
const INV_MAX_DISPLAY_SLOTS: int = 10


func _build_inventory_tab() -> void:
	_inventory_panel = VBoxContainer.new()
	_inventory_panel.add_theme_constant_override("separation", ROW_SPACING)
	_tab_container.add_child(_inventory_panel)

	_add_section_title(_inventory_panel, "PANEL_INVENTORY_TITLE")

	_inventory_empty_label = Label.new()
	_inventory_empty_label.text = Locale.ltr("UI_NO_ITEMS")
	_inventory_empty_label.add_theme_font_size_override("font_size", 10)
	_inventory_empty_label.add_theme_color_override("font_color", Color(0.22, 0.28, 0.31))
	_inventory_panel.add_child(_inventory_empty_label)

	# Grid container — 5 columns
	_inventory_container = GridContainer.new()
	_inventory_container.columns = 5
	_inventory_container.add_theme_constant_override("h_separation", 3)
	_inventory_container.add_theme_constant_override("v_separation", 3)
	_inventory_panel.add_child(_inventory_container)


func _refresh_inventory() -> void:
	if _inventory_panel == null:
		return

	var items_raw: Variant = _detail.get("inv_items", [])
	var items: Array = items_raw if items_raw is Array else []

	# Skip rebuild if data hasn't changed — preserves tooltips
	var new_hash: int = _compute_inv_fingerprint(items)
	if new_hash == _inv_data_hash and _inventory_container.get_child_count() > 0:
		return
	_inv_data_hash = new_hash

	for child in _inventory_container.get_children():
		child.queue_free()
	# Remove any previously added equipped labels below the grid
	var grid_idx: int = _inventory_container.get_index()
	while _inventory_panel.get_child_count() > grid_idx + 1:
		var extra: Node = _inventory_panel.get_child(grid_idx + 1)
		_inventory_panel.remove_child(extra)
		extra.queue_free()

	_inventory_empty_label.visible = items.is_empty()
	if items.is_empty():
		for i in range(INV_MAX_DISPLAY_SLOTS):
			_inventory_container.add_child(_create_empty_slot())
		return

	# Separate stackable vs non-stackable
	var stackable_groups: Dictionary = {}
	var individual_items: Array[Dictionary] = []
	var equipped_items: Array[Dictionary] = []

	for item_raw: Variant in items:
		if not (item_raw is Dictionary):
			continue
		var item: Dictionary = item_raw
		var equipped_slot: String = str(item.get("equipped_slot", ""))
		var cur_dur: float = float(item.get("current_durability", 100.0))
		var max_dur: float = float(item.get("max_durability", 100.0))
		# Display grouping: equipped → individual, worn → individual, rest → group
		var is_worn: bool = max_dur > 0.0 and cur_dur < max_dur * 0.99

		if not equipped_slot.is_empty():
			equipped_items.append(item)
		elif is_worn:
			individual_items.append(item)
		else:
			var tid: String = str(item.get("template_id", "unknown"))
			var mid: String = str(item.get("material_id", ""))
			var group_key: String = tid + "|" + mid
			var stack_count: int = maxi(1, int(item.get("stack_count", 1)))
			if stackable_groups.has(group_key):
				stackable_groups[group_key]["count"] += stack_count
			else:
				stackable_groups[group_key] = {
					"count": stack_count,
					"template_id": tid,
					"material_id": mid,
				}

	# Build grid slots: equipped first, then tools, then stacked materials
	var slot_count: int = 0

	for item: Dictionary in equipped_items:
		_inventory_container.add_child(_create_tool_slot(item, true))
		slot_count += 1

	for item: Dictionary in individual_items:
		_inventory_container.add_child(_create_tool_slot(item, false))
		slot_count += 1

	for group_key_variant: Variant in stackable_groups.keys():
		var group: Dictionary = stackable_groups[str(group_key_variant)]
		_inventory_container.add_child(_create_stack_slot(group))
		slot_count += 1

	# Fill remaining with empty slots
	for i in range(slot_count, INV_MAX_DISPLAY_SLOTS):
		_inventory_container.add_child(_create_empty_slot())

	# Equipped summary below grid (simple text, Phase 3 will add silhouette)
	if not equipped_items.is_empty():
		_add_section_spacer(_inventory_panel)
		_add_section_title(_inventory_panel, "PANEL_EQUIPPED_TITLE")
		for item: Dictionary in equipped_items:
			var tid: String = str(item.get("template_id", ""))
			var mid: String = str(item.get("material_id", ""))
			var slot_name: String = str(item.get("equipped_slot", ""))
			var display_name: String = _display_token(tid)
			var material_name: String = _display_token(mid)
			var slot_label: String = _equip_slot_label(slot_name)
			var equip_label := Label.new()
			equip_label.text = "%s %s (%s) — %s" % [_inventory_icon(tid), display_name, material_name, slot_label]
			equip_label.add_theme_font_size_override("font_size", 9)
			equip_label.add_theme_color_override("font_color", Color(0.78, 0.60, 0.15))
			_inventory_panel.add_child(equip_label)


## Creates a 36x36 inventory slot for a non-stackable tool/weapon item.
func _create_tool_slot(item: Dictionary, is_equipped: bool) -> Button:
	var slot := Button.new()
	slot.flat = true
	slot.focus_mode = Control.FOCUS_NONE
	slot.text = ""
	slot.custom_minimum_size = Vector2(INV_SLOT_SIZE, INV_SLOT_SIZE)

	var quality: float = clampf(_safe_scalar(item.get("quality", 0.5), 0.5), 0.0, 1.0)
	var border_color: Color = _quality_border_color(quality)
	_apply_slot_style(slot, Color(0.04, 0.06, 0.09, 1.0), border_color, 2, 3)

	var tid: String = str(item.get("template_id", ""))
	var mid: String = str(item.get("material_id", ""))
	var icon: String = _inventory_icon(tid)
	var display_name: String = _display_token(tid)
	var material_name: String = _display_token(mid)
	var cur_dur: float = _safe_scalar(item.get("current_durability", 100.0), 100.0)
	var max_dur: float = _safe_scalar(item.get("max_durability", 100.0), 100.0)
	var damage: float = _safe_scalar(item.get("damage", 1.0), 1.0)
	var speed: float = _safe_scalar(item.get("speed", 1.0), 1.0)
	var quality_pct: int = int(quality * 100.0)

	# Center icon
	var icon_label := Label.new()
	icon_label.text = icon
	icon_label.add_theme_font_size_override("font_size", 16)
	icon_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	icon_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	icon_label.position = Vector2(0, 0)
	icon_label.size = Vector2(INV_SLOT_SIZE, INV_SLOT_SIZE)
	icon_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	slot.add_child(icon_label)

	# Durability mini-bar (bottom of slot)
	var dur_ratio: float = clampf(cur_dur / maxf(max_dur, 1.0), 0.0, 1.0)
	var dur_color: Color
	if dur_ratio > 0.6:
		dur_color = Color(0.28, 0.66, 0.16)
	elif dur_ratio > 0.3:
		dur_color = Color(0.80, 0.65, 0.12)
	else:
		dur_color = Color(0.78, 0.22, 0.15)
	var dur_bar := ColorRect.new()
	dur_bar.color = dur_color
	dur_bar.position = Vector2(2, INV_SLOT_SIZE - 5)
	dur_bar.size = Vector2((INV_SLOT_SIZE - 4) * dur_ratio, 3)
	dur_bar.mouse_filter = Control.MOUSE_FILTER_IGNORE
	slot.add_child(dur_bar)

	# Equipped marker
	if is_equipped:
		var star := Label.new()
		star.text = "★"
		star.add_theme_font_size_override("font_size", 8)
		star.add_theme_color_override("font_color", Color(0.78, 0.60, 0.15))
		star.position = Vector2(2, 1)
		star.size = Vector2(16, 12)
		star.mouse_filter = Control.MOUSE_FILTER_IGNORE
		slot.add_child(star)

	# Tooltip on slot — hash check prevents rebuild so tooltip survives
	slot.tooltip_text = "%s (%s)\n%s: %d%%\n%s: %d/%d\n%s: %.1f  %s: %.1f" % [
		display_name, material_name,
		Locale.ltr("UI_QUALITY"), quality_pct,
		Locale.ltr("UI_DURABILITY"), int(cur_dur), int(max_dur),
		Locale.ltr("UI_DAMAGE"), damage,
		Locale.ltr("UI_SPEED"), speed,
	]
	slot.mouse_filter = Control.MOUSE_FILTER_STOP

	return slot


## Creates a 36x36 inventory slot for a stack of identical raw materials.
func _create_stack_slot(group: Dictionary) -> Button:
	var slot := Button.new()
	slot.flat = true
	slot.focus_mode = Control.FOCUS_NONE
	slot.text = ""
	slot.custom_minimum_size = Vector2(INV_SLOT_SIZE, INV_SLOT_SIZE)

	_apply_slot_style(slot, Color(0.04, 0.06, 0.09, 1.0), Color(0.09, 0.14, 0.19), 1, 3)

	var tid: String = str(group.get("template_id", ""))
	var mid: String = str(group.get("material_id", ""))
	var count: int = int(group.get("count", 1))
	var icon: String = _inventory_icon(tid)
	var display_name: String = _display_token(tid)
	var material_name: String = _display_token(mid)

	# Center icon
	var icon_label := Label.new()
	icon_label.text = icon
	icon_label.add_theme_font_size_override("font_size", 16)
	icon_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	icon_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	icon_label.position = Vector2(0, 0)
	icon_label.size = Vector2(INV_SLOT_SIZE, INV_SLOT_SIZE)
	icon_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	slot.add_child(icon_label)

	# Quantity badge (bottom-right) — only if count > 1
	if count > 1:
		var qty_label := Label.new()
		qty_label.text = "×%d" % count
		qty_label.add_theme_font_size_override("font_size", 8)
		qty_label.add_theme_color_override("font_color", Color(0.72, 0.78, 0.85))
		qty_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		qty_label.position = Vector2(0, INV_SLOT_SIZE - 12)
		qty_label.size = Vector2(INV_SLOT_SIZE - 2, 12)
		qty_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
		slot.add_child(qty_label)

	# Tooltip on slot — hash check prevents rebuild so tooltip survives
	slot.tooltip_text = "%s (%s)\n%s: %d" % [
		display_name, material_name,
		Locale.ltr("UI_QUANTITY"), count,
	]
	slot.mouse_filter = Control.MOUSE_FILTER_STOP

	return slot


## Creates a dim 36x36 empty inventory slot.
func _create_empty_slot() -> Button:
	var slot := Button.new()
	slot.flat = true
	slot.focus_mode = Control.FOCUS_NONE
	slot.text = ""
	slot.custom_minimum_size = Vector2(INV_SLOT_SIZE, INV_SLOT_SIZE)

	_apply_slot_style(slot, Color(0.04, 0.06, 0.09, 0.3), Color(0.09, 0.14, 0.19, 0.3), 1, 3)
	slot.mouse_filter = Control.MOUSE_FILTER_IGNORE

	return slot


## Returns a border color based on item quality tier.
func _quality_border_color(quality: float) -> Color:
	if quality >= 0.85:
		return Color(0.75, 0.60, 0.15)  # Gold — excellent
	elif quality >= 0.6:
		return Color(0.25, 0.55, 0.20)  # Green — good
	elif quality >= 0.3:
		return Color(0.20, 0.25, 0.30)  # Default — normal
	else:
		return Color(0.35, 0.35, 0.38)  # Gray — poor


## Creates a colored alert card with left border accent.
func _create_alert_card(title: String, detail: String, color: Color) -> PanelContainer:
	var card := PanelContainer.new()
	var card_style := StyleBoxFlat.new()
	card_style.bg_color = Color(color.r, color.g, color.b, 0.06)
	card_style.border_color = color
	card_style.border_width_left = 3
	card_style.border_width_top = 0
	card_style.border_width_right = 0
	card_style.border_width_bottom = 0
	card_style.set_corner_radius_all(3)
	card_style.content_margin_left = 6
	card_style.content_margin_right = 6
	card_style.content_margin_top = 3
	card_style.content_margin_bottom = 3
	card.add_theme_stylebox_override("panel", card_style)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 1)
	card.add_child(vbox)

	var title_label := Label.new()
	title_label.text = title
	title_label.add_theme_font_size_override("font_size", 10)
	title_label.add_theme_color_override("font_color", color)
	vbox.add_child(title_label)

	if not detail.is_empty():
		var detail_label := Label.new()
		detail_label.text = detail
		detail_label.add_theme_font_size_override("font_size", 9)
		detail_label.add_theme_color_override("font_color", Color(0.53, 0.60, 0.66))
		vbox.add_child(detail_label)

	return card


## Creates a small colored tag chip for trait display.
func _create_trait_chip(text: String) -> PanelContainer:
	var chip := PanelContainer.new()
	var chip_style := StyleBoxFlat.new()
	chip_style.bg_color = Color(0.06, 0.10, 0.18)
	chip_style.border_color = Color(0.09, 0.14, 0.19)
	chip_style.set_border_width_all(1)
	chip_style.set_corner_radius_all(2)
	chip_style.content_margin_left = 4
	chip_style.content_margin_right = 4
	chip_style.content_margin_top = 1
	chip_style.content_margin_bottom = 1
	chip.add_theme_stylebox_override("panel", chip_style)

	var label := Label.new()
	label.text = text
	label.add_theme_font_size_override("font_size", 9)
	label.add_theme_color_override("font_color", Color(0.41, 0.53, 0.66))
	chip.add_child(label)

	return chip


## Applies the same StyleBox to all button states so the slot looks static.
func _apply_slot_style(button: Button, bg_color: Color, border_color: Color, border_width: int, corner_radius: int) -> void:
	for state: String in ["normal", "hover", "pressed", "disabled", "focus"]:
		var sb := StyleBoxFlat.new()
		sb.bg_color = bg_color
		sb.border_color = border_color
		sb.set_border_width_all(border_width)
		sb.set_corner_radius_all(corner_radius)
		button.add_theme_stylebox_override(state, sb)


## Returns a localized equipment slot label.
func _equip_slot_label(slot_key: String) -> String:
	match slot_key:
		"main_hand":
			return Locale.ltr("UI_SLOT_MAIN_HAND")
		"off_hand":
			return Locale.ltr("UI_SLOT_OFF_HAND")
		_:
			return slot_key


## Computes a fast fingerprint of inventory data to detect changes.
func _compute_inv_fingerprint(items: Array) -> int:
	var h: int = items.size()
	for item_raw: Variant in items:
		if not (item_raw is Dictionary):
			continue
		var item: Dictionary = item_raw
		h = h * 31 + int(item.get("id", 0))
		h = h * 31 + int(item.get("stack_count", 0))
		h = h * 31 + int(float(item.get("current_durability", 0)) * 10.0)
		h = h * 31 + int(float(item.get("quality", 0)) * 100.0)
		var equip_str: String = str(item.get("equipped_slot", ""))
		h = h * 31 + equip_str.hash()
	return h


# ---------------------------------------------------------------------------
# Family tab
# ---------------------------------------------------------------------------

func _build_family_tab() -> void:
	_family_panel = VBoxContainer.new()
	_family_panel.add_theme_constant_override("separation", 4)
	_tab_container.add_child(_family_panel)

	_add_section_title(_family_panel, "PANEL_FAMILY_TITLE")

	var pairs: Array[Array] = [
		["UI_FATHER", "_family_father_label"],
		["UI_MOTHER", "_family_mother_label"],
		["UI_SPOUSE", "_family_spouse_label"],
		["UI_CHILDREN", "_family_children_label"],
	]
	for pair: Array in pairs:
		var hbox := HBoxContainer.new()
		hbox.add_theme_constant_override("separation", 8)
		_family_panel.add_child(hbox)
		var key_label := Label.new()
		key_label.text = Locale.ltr(str(pair[0]))
		key_label.custom_minimum_size.x = 50.0
		key_label.add_theme_font_size_override("font_size", 10)
		key_label.add_theme_color_override("font_color", COLOR_LABEL)
		hbox.add_child(key_label)
		var val_label := Label.new()
		val_label.add_theme_font_size_override("font_size", 10)
		val_label.add_theme_color_override("font_color", Color.WHITE)
		val_label.autowrap_mode = TextServer.AUTOWRAP_WORD
		val_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		hbox.add_child(val_label)
		set(str(pair[1]), val_label)

	_add_section_spacer(_family_panel)
	_family_kinship_label = Label.new()
	_family_kinship_label.add_theme_font_size_override("font_size", 10)
	_family_kinship_label.add_theme_color_override("font_color", Color(0.40, 0.48, 0.55))
	_family_panel.add_child(_family_kinship_label)


func _refresh_family() -> void:
	if _family_panel == null:
		return
	if _family_tab.is_empty():
		_family_father_label.text = Locale.ltr("PANEL_FAMILY_UNKNOWN")
		_family_mother_label.text = Locale.ltr("PANEL_FAMILY_UNKNOWN")
		_family_spouse_label.text = Locale.ltr("UI_NONE")
		_family_children_label.text = Locale.ltr("UI_NONE")
		_family_kinship_label.text = "%s: %s" % [Locale.ltr("PANEL_FAMILY_KINSHIP"), Locale.ltr("KINSHIP_BILATERAL")]
		return

	var father_raw: Variant = _family_tab.get("father", {})
	var mother_raw: Variant = _family_tab.get("mother", {})
	var spouse_raw: Variant = _family_tab.get("spouse", {})
	var children_raw: Variant = _family_tab.get("children", [])
	var children: Array = children_raw if children_raw is Array else []

	_family_father_label.text = _family_member_name(father_raw)
	_make_clickable_label(_family_father_label, _family_member_id(father_raw))
	_family_mother_label.text = _family_member_name(mother_raw)
	_make_clickable_label(_family_mother_label, _family_member_id(mother_raw))
	_family_spouse_label.text = _family_member_name(spouse_raw, Locale.ltr("UI_NONE"))
	_make_clickable_label(_family_spouse_label, _family_member_id(spouse_raw))

	if children.is_empty():
		_family_children_label.text = Locale.ltr("UI_NONE")
	else:
		var names: PackedStringArray = PackedStringArray()
		for child_raw: Variant in children:
			if child_raw is Dictionary:
				names.append(str((child_raw as Dictionary).get("name", "?")))
		_family_children_label.text = ", ".join(names) if not names.is_empty() else Locale.ltr("UI_NONE")

	var kinship_index: int = clampi(int(_family_tab.get("kinship_system", 0)), 0, 2)
	var kinship_keys: Array[String] = ["KINSHIP_BILATERAL", "KINSHIP_PATRILINEAL", "KINSHIP_MATRILINEAL"]
	_family_kinship_label.text = "%s: %s" % [Locale.ltr("PANEL_FAMILY_KINSHIP"), Locale.ltr(kinship_keys[kinship_index])]


func _family_member_id(raw: Variant) -> int:
	if raw is Dictionary and not (raw as Dictionary).is_empty():
		return int((raw as Dictionary).get("id", -1))
	return -1


func _family_member_name(raw: Variant, fallback: String = "") -> String:
	if raw is Dictionary and not (raw as Dictionary).is_empty():
		return str((raw as Dictionary).get("name", fallback if not fallback.is_empty() else Locale.ltr("PANEL_FAMILY_UNKNOWN")))
	return fallback if not fallback.is_empty() else Locale.ltr("PANEL_FAMILY_UNKNOWN")


func _on_follow_pressed() -> void:
	if _selected_entity_id < 0:
		return
	var camera: Camera2D = get_viewport().get_camera_2d()
	if camera != null and camera.has_method("is_following") and camera.is_following():
		camera.stop_following()
		return
	SimulationBus.follow_entity_requested.emit(_selected_entity_id)


func _on_favorite_pressed() -> void:
	if _selected_entity_id < 0:
		return
	SimulationBus.favorite_toggled.emit(_selected_entity_id)

func update_favorite_state(is_fav: bool) -> void:
	if _favorite_button != null:
		_favorite_button.text = Locale.ltr("UI_BTN_FAVORITED") if is_fav else Locale.ltr("UI_BTN_FAVORITE")


var _clickable_callables: Dictionary = {}

func _make_clickable_label(label: Label, entity_id: int) -> void:
	var key: int = label.get_instance_id()
	if entity_id < 0:
		label.mouse_default_cursor_shape = Control.CURSOR_ARROW
		label.remove_theme_color_override("font_color")
		label.mouse_filter = Control.MOUSE_FILTER_IGNORE
		if _clickable_callables.has(key):
			if label.gui_input.is_connected(_clickable_callables[key]):
				label.gui_input.disconnect(_clickable_callables[key])
			_clickable_callables.erase(key)
		return

	label.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	label.add_theme_color_override("font_color", Color(0.7, 0.8, 1.0))
	label.mouse_filter = Control.MOUSE_FILTER_STOP

	# Disconnect old callable if exists
	if _clickable_callables.has(key):
		if label.gui_input.is_connected(_clickable_callables[key]):
			label.gui_input.disconnect(_clickable_callables[key])

	var captured_id: int = entity_id
	var callable: Callable = func(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			_navigate_to_entity(captured_id)
	_clickable_callables[key] = callable
	label.gui_input.connect(callable)


func _navigate_to_entity(entity_id: int) -> void:
	if entity_id < 0:
		return
	SimulationBus.entity_selected.emit(entity_id)
	SimulationBus.ui_notification.emit("focus_entity_%d" % entity_id, "command")


# ---------------------------------------------------------------------------
# Events tab
# ---------------------------------------------------------------------------

func _build_events_tab() -> void:
	_events_panel = VBoxContainer.new()
	_events_panel.add_theme_constant_override("separation", 4)
	_tab_container.add_child(_events_panel)

	_add_section_title(_events_panel, "PANEL_EVENTS_TITLE")

	_events_empty_label = Label.new()
	_events_empty_label.text = Locale.ltr("UI_NO_EVENTS")
	_events_empty_label.add_theme_font_size_override("font_size", 10)
	_events_empty_label.add_theme_color_override("font_color", Color(0.22, 0.28, 0.31))
	_events_panel.add_child(_events_empty_label)

	_events_container = VBoxContainer.new()
	_events_container.add_theme_constant_override("separation", 4)
	_events_panel.add_child(_events_container)


func _refresh_events() -> void:
	if _events_panel == null:
		return
	for child in _events_container.get_children():
		child.queue_free()

	var story_events_raw: Variant = _memory_tab.get("story_events", [])
	var story_events: Array = story_events_raw if story_events_raw is Array else []
	_events_empty_label.visible = story_events.is_empty()
	if story_events.is_empty():
		return

	for entry_raw: Variant in story_events:
		if not (entry_raw is Dictionary):
			continue
		var entry: Dictionary = entry_raw
		var event_label := Label.new()
		event_label.text = _story_event_display(entry)
		event_label.add_theme_font_size_override("font_size", 10)
		event_label.add_theme_color_override("font_color", Color(0.55, 0.62, 0.70))
		event_label.autowrap_mode = TextServer.AUTOWRAP_WORD
		_events_container.add_child(event_label)


func _story_event_display(entry: Dictionary) -> String:
	var message_key: String = str(entry.get("message_key", ""))
	if not message_key.is_empty():
		var params_raw: Variant = entry.get("message_params", {})
		if params_raw is Dictionary:
			var localized: String = Locale.trf(message_key, params_raw as Dictionary)
			if localized != message_key:
				return localized
		var fallback: String = Locale.ltr(message_key)
		if fallback != message_key:
			return fallback
	var raw_text: String = str(entry.get("text", str(entry.get("description", ""))))
	return raw_text if not raw_text.is_empty() else Locale.ltr("UI_UNKNOWN")


# ---------------------------------------------------------------------------
# Helpers (from v4, needed by v5 since we extend v3 directly)
# ---------------------------------------------------------------------------

func _build_sorted_need_entries(limit: int) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for row: Dictionary in NEED_ROWS:
		var value: float = _safe_float(_detail, str(row["field"]), 0.0)
		result.append({"label": Locale.ltr(str(row["key"])), "value": value})
	result.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		return float(a.get("value", 0.0)) < float(b.get("value", 0.0)))
	if result.size() > limit:
		result.resize(limit)
	return result


func _safe_scalar(raw: Variant, default_value: float) -> float:
	if raw is float or raw is int:
		var numeric_value: float = float(raw)
		if is_nan(numeric_value) or is_inf(numeric_value):
			return default_value
		return numeric_value
	if raw is String:
		var text: String = raw.strip_edges()
		if text.is_empty():
			return default_value
		if not text.is_valid_float():
			return default_value
		var parsed_value: float = text.to_float()
		if is_nan(parsed_value) or is_inf(parsed_value):
			return default_value
		return parsed_value
	return default_value


func _safe_float(dict: Dictionary, key: String, default_value: float) -> float:
	var scalar_value: float = _safe_scalar(dict.get(key, default_value), default_value)
	if is_nan(scalar_value) or is_inf(scalar_value):
		return clampf(default_value, 0.0, 1.0)
	return clampf(scalar_value, 0.0, 1.0)


func _normalized_stress() -> float:
	var raw_value: float = _safe_scalar(_detail.get("stress_level", 0.0), 0.0)
	if raw_value <= 1.0:
		return clampf(raw_value, 0.0, 1.0)
	return clampf(raw_value / 1000.0, 0.0, 1.0)


func _band_label() -> String:
	var band_name: String = str(_detail.get("band_name", ""))
	return band_name if not band_name.is_empty() else Locale.ltr("UI_NONE")


func _emotion_to_color(emotion: String) -> Color:
	match _emotion_category(emotion):
		"joy":
			return Color(0.28, 0.66, 0.16)
		"fear":
			return Color(0.78, 0.55, 0.10)
		"anger":
			return Color(0.78, 0.22, 0.22)
		"sad":
			return Color(0.34, 0.41, 0.66)
		_:
			return Color(0.38, 0.44, 0.50)


func _emotion_category(emotion: String) -> String:
	var ek: String = emotion.strip_edges().to_lower()
	if ek.contains("joy") or ek.contains("trust"):
		return "joy"
	if ek.contains("fear") or ek.contains("surprise"):
		return "fear"
	if ek.contains("anger") or ek.contains("disgust"):
		return "anger"
	if ek.contains("sad"):
		return "sad"
	return "neutral"


func _merged_health_groups() -> Array[Dictionary]:
	var hp_raw: Variant = _health_tab.get("group_hp", PackedByteArray())
	var values: Array[float] = []
	if hp_raw is PackedByteArray:
		for hp_value: int in hp_raw:
			values.append(clampf(float(hp_value) / 100.0, 0.0, 1.0))
	while values.size() < 10:
		values.append(1.0)
	return [
		{"label": "BODY_GROUP_HEAD", "value": values[0]},
		{"label": "BODY_GROUP_NECK", "value": values[1]},
		{"label": "BODY_GROUP_UPPER_TORSO", "value": values[2]},
		{"label": "BODY_GROUP_LOWER_TORSO", "value": values[3]},
		{"label": "BODY_GROUP_ARM_L", "value": (values[4] + values[8]) * 0.5},
		{"label": "BODY_GROUP_ARM_R", "value": (values[5] + values[9]) * 0.5},
		{"label": "BODY_GROUP_LEG_L", "value": values[6]},
		{"label": "BODY_GROUP_LEG_R", "value": values[7]},
	]


func _knowledge_channels() -> Array[Dictionary]:
	return [
		{"icon": "🗣️", "label": "UI_CHANNEL_ORAL", "status": "UI_ACTIVE", "locked": false},
		{"icon": "👁️", "label": "UI_CHANNEL_OBSERVE", "status": "UI_ACTIVE", "locked": false},
		{"icon": "🔨", "label": "UI_CHANNEL_APPRENTICE", "status": "UI_ACTIVE", "locked": false},
		{"icon": "📜", "label": "UI_CHANNEL_RECORD", "status": "UI_REQUIRES_WRITING", "locked": true},
		{"icon": "🏛️", "label": "UI_CHANNEL_SCHOOL", "status": "UI_REQUIRES_WRITING", "locked": true},
		{"icon": "💡", "label": "UI_CHANNEL_DISCOVERY", "status": "UI_ACTIVE", "locked": false},
	]


func _inventory_icon(template_id: String) -> String:
	var token: String = template_id.to_lower()
	# Tools
	if token.contains("axe"): return "🪓"
	if token.contains("spear"): return "🗡️"
	if token.contains("knife"): return "🔪"
	if token.contains("pick") or token.contains("pickaxe"): return "⛏️"
	if token.contains("hammer"): return "🔨"
	if token.contains("needle"): return "🪡"
	if token.contains("pot") or token.contains("vessel"): return "🏺"
	# Raw materials
	if token.contains("stone") or token.contains("flint") or token.contains("obsidian"): return "🪨"
	if token.contains("wood") or token.contains("log") or token.contains("stick"): return "🪵"
	if token.contains("bone"): return "🦴"
	if token.contains("hide") or token.contains("leather") or token.contains("pelt"): return "🦴"
	if token.contains("fiber") or token.contains("rope") or token.contains("cordage"): return "🧵"
	if token.contains("clay"): return "🧱"
	# Food
	if token.contains("berries") or token.contains("berry") or token.contains("fruit"): return "🫐"
	if token.contains("mushroom") or token.contains("fungus"): return "🍄"
	if token.contains("herb") or token.contains("plant"): return "🌿"
	if token.contains("fish"): return "🐟"
	if token.contains("meat") or token.contains("food"): return "🍖"
	if token.contains("seed") or token.contains("grain"): return "🌾"
	# Default
	return "📦"


func _display_token(token: String) -> String:
	if token.is_empty():
		return Locale.ltr("UI_UNKNOWN")
	var locale_key: String = token.to_upper()
	if Locale.has_key(locale_key):
		return Locale.ltr(locale_key)
	var display_text: String = token.replace("_", " ")
	return display_text.capitalize()


func _build_trait_tag_texts() -> PackedStringArray:
	var tags: PackedStringArray = PackedStringArray()
	if _safe_float(_detail, "hex_c", 0.0) >= 0.65:
		tags.append(Locale.ltr("VALUE_HARD_WORK"))
	if _safe_float(_detail, "hex_a", 0.0) >= 0.65:
		tags.append(Locale.ltr("VALUE_HARMONY"))
	if _safe_float(_detail, "hex_o", 0.0) >= 0.65:
		tags.append(Locale.ltr("VALUE_KNOWLEDGE"))
	if _safe_float(_detail, "hex_x", 0.0) >= 0.65:
		tags.append(Locale.ltr("VALUE_FRIENDSHIP"))
	if _safe_float(_detail, "hex_h", 0.0) >= 0.65:
		tags.append(Locale.ltr("VALUE_TRUTH"))
	if _safe_float(_detail, "hex_e", 0.0) >= 0.65:
		tags.append(Locale.ltr("VALUE_FAMILY"))
	if _safe_float(_detail, "tci_ns", 0.0) >= 0.65:
		tags.append(Locale.ltr("VALUE_INDEPENDENCE"))
	if _safe_float(_detail, "tci_p", 0.0) >= 0.65:
		tags.append(Locale.ltr("VALUE_PERSEVERANCE"))
	return tags


func _value_rankings() -> Array[Dictionary]:
	var values_raw: Variant = _mind_tab.get("values_all", null)
	var ranked: Array[Dictionary] = []
	if values_raw == null:
		return ranked
	var labels: Array = ValueDefs.KEYS
	var count: int = 0
	if values_raw is PackedFloat32Array:
		var pf: PackedFloat32Array = values_raw
		count = mini(pf.size(), labels.size())
		for index: int in range(count):
			ranked.append({"key": str(labels[index]), "value": float(pf[index])})
	elif values_raw is Array:
		count = mini(values_raw.size(), labels.size())
		for index: int in range(count):
			var v: float = float(values_raw[index]) if (values_raw[index] is float or values_raw[index] is int) else 0.0
			ranked.append({"key": str(labels[index]), "value": v})
	ranked.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		return float(a.get("value", 0.0)) > float(b.get("value", 0.0)))
	if ranked.size() > 5:
		ranked.resize(5)
	return ranked


func _add_section_title(parent: Control, locale_key: String) -> void:
	var title := Label.new()
	title.text = Locale.ltr(locale_key)
	title.add_theme_font_size_override("font_size", 11)
	title.add_theme_color_override("font_color", COLOR_SECTION_TITLE)
	parent.add_child(title)


func _add_section_spacer(parent: Control) -> void:
	var spacer := Control.new()
	spacer.custom_minimum_size.y = SECTION_SPACING
	parent.add_child(spacer)
