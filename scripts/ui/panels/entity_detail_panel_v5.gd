extends "res://scripts/ui/panels/entity_detail_panel_v3.gd"
class_name EntityDetailPanelV5

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
const ROW_SPACING: float = 2.0

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
var _tab_bar: TabBar
var _tab_container: VBoxContainer

# Tab panels
var _overview_panel: VBoxContainer
var _needs_panel: VBoxContainer

# Overview sub-nodes
var _overview_alert_label: Label
var _overview_info_label: Label
var _overview_need_rows: Array[Dictionary] = []

# Need rows cache (reusable — update values, don't recreate)
var _v5_need_rows: Array[Dictionary] = []

# Refresh
var _v5_refresh_timer: float = 0.0


# ---------------------------------------------------------------------------
# Lifecycle
# ---------------------------------------------------------------------------

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
	info_vbox.add_child(_header_action_label)


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
	_tab_bar.tab_changed.connect(_on_tab_changed)
	_content.add_child(_tab_bar)

	_tab_container = VBoxContainer.new()
	_tab_container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_tab_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_content.add_child(_tab_container)


func _on_tab_changed(tab_index: int) -> void:
	_switch_tab(tab_index)


func _switch_tab(index: int) -> void:
	if _overview_panel != null:
		_overview_panel.visible = (index == 0)
	if _needs_panel != null:
		_needs_panel.visible = (index == 1)


# ---------------------------------------------------------------------------
# S3: Overview tab
# ---------------------------------------------------------------------------

func _build_overview_tab() -> void:
	_overview_panel = VBoxContainer.new()
	_overview_panel.add_theme_constant_override("separation", 6)
	_tab_container.add_child(_overview_panel)

	_overview_alert_label = Label.new()
	_overview_alert_label.add_theme_font_size_override("font_size", 11)
	_overview_alert_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	_overview_panel.add_child(_overview_alert_label)

	_overview_info_label = Label.new()
	_overview_info_label.add_theme_font_size_override("font_size", 10)
	_overview_info_label.add_theme_color_override("font_color", COLOR_LABEL)
	_overview_info_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	_overview_panel.add_child(_overview_info_label)

	var needs_title := Label.new()
	needs_title.text = Locale.ltr("PANEL_OVERVIEW_NEEDS")
	needs_title.add_theme_font_size_override("font_size", 11)
	needs_title.add_theme_color_override("font_color", COLOR_SECTION_TITLE)
	_overview_panel.add_child(needs_title)

	for i in range(4):
		_overview_need_rows.append(_create_bar_row(_overview_panel))


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
	_refresh_overview()
	_refresh_needs()


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
	_header_action_label.text = action_line


func _refresh_overview() -> void:
	if _overview_alert_label == null or _detail.is_empty():
		return

	var alerts: PackedStringArray = PackedStringArray()
	var hunger: float = _safe_float(_detail, "need_hunger", 1.0)
	var sleep_val: float = _safe_float(_detail, "need_sleep", 1.0)
	var stress: float = _normalized_stress()
	if hunger < 0.35:
		alerts.append("⚠ " + Locale.ltr("ALERT_HUNGRY"))
	if sleep_val < 0.30:
		alerts.append("⚠ " + Locale.ltr("ALERT_TIRED"))
	if stress > 0.30:
		alerts.append("⚠ " + Locale.ltr("ALERT_STRESSED"))
	if alerts.is_empty():
		_overview_alert_label.text = "✓ " + Locale.ltr("ALERT_ALL_GOOD")
		_overview_alert_label.add_theme_color_override("font_color", COLOR_ALERT_GOOD)
	else:
		_overview_alert_label.text = "\n".join(alerts)
		_overview_alert_label.add_theme_color_override("font_color", COLOR_ALERT_BAD)

	var occ: String = _localized_action_text(str(_detail.get("occupation", "none")))
	var age: int = int(round(_safe_panel_scalar(_detail.get("age_years", 0.0), 0.0)))
	var action: String = _localized_action_text(str(_detail.get("current_action", "Idle")))
	var band: String = _band_label()
	_overview_info_label.text = "%s %s   %s %d%s\n%s %s   %s %s" % [
		Locale.ltr("UI_JOB"), occ,
		Locale.ltr("UI_AGE"), age, Locale.ltr("UI_AGE_UNIT"),
		Locale.ltr("UI_ACTION"), action,
		Locale.ltr("UI_BAND"), band,
	]

	var entries: Array[Dictionary] = _build_sorted_need_entries(4)
	for i in range(4):
		if i < entries.size():
			_update_bar_row(_overview_need_rows[i], entries[i])
			_overview_need_rows[i].container.visible = true
		else:
			_overview_need_rows[i].container.visible = false


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
