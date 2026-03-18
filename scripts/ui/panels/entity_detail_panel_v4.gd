extends "res://scripts/ui/panels/entity_detail_panel_v3.gd"
class_name EntityDetailPanelV4

const ValueDefs = preload("res://scripts/core/social/value_defs.gd")

const PORTRAIT_SIZE: Vector2 = Vector2(48.0, 48.0)
const VALUE_LABELS: Array[String] = ValueDefs.KEYS

var _breadcrumb_wrapper: PanelContainer
var _breadcrumb_label: RichTextLabel
var _breadcrumb_back_button: Button
var _portrait: Control
var _follow_button: Button
var _favorite_button: Button
var _tab_inventory_text: RichTextLabel
var _nav_stack: Array[Dictionary] = []
var _v4_refresh_timer: float = 0.0


func _process(delta: float) -> void:
	# v4 overrides v3._process — eliminates the 3s race condition
	if not visible or _selected_entity_id < 0:
		return
	_v4_refresh_timer += delta
	if _v4_refresh_timer >= 0.5:
		if _is_reloading:
			# Safety: force-reset stuck flag (can happen if _refresh_all crashes)
			_is_reloading = false
			_v4_refresh_timer = 0.0
		else:
			_v4_refresh_timer = 0.0
			_reload_data()


func _build_ui() -> void:
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.05, 0.07, 0.10, 0.95)
	style.border_color = Color(0.12, 0.16, 0.22, 0.80)
	style.border_width_left = 1
	style.border_width_top = 1
	style.border_width_right = 0
	style.border_width_bottom = 1
	style.corner_radius_top_left = 6
	style.corner_radius_bottom_left = 6
	add_theme_stylebox_override("panel", style)

	var root := VBoxContainer.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	root.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_theme_constant_override("separation", 0)
	add_child(root)

	_breadcrumb_wrapper = PanelContainer.new()
	_breadcrumb_wrapper.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_breadcrumb_wrapper.visible = false
	var breadcrumb_style := StyleBoxFlat.new()
	breadcrumb_style.bg_color = Color(0.04, 0.05, 0.08, 1.0)
	breadcrumb_style.content_margin_left = 6
	breadcrumb_style.content_margin_right = 6
	breadcrumb_style.content_margin_top = 2
	breadcrumb_style.content_margin_bottom = 2
	breadcrumb_style.border_width_bottom = 1
	breadcrumb_style.border_color = Color(0.05, 0.06, 0.10)
	_breadcrumb_wrapper.add_theme_stylebox_override("panel", breadcrumb_style)
	root.add_child(_breadcrumb_wrapper)

	var breadcrumb_row := HBoxContainer.new()
	breadcrumb_row.add_theme_constant_override("separation", 4)
	_breadcrumb_wrapper.add_child(breadcrumb_row)

	_breadcrumb_back_button = Button.new()
	_breadcrumb_back_button.text = "←"
	_breadcrumb_back_button.custom_minimum_size = Vector2(24.0, 0.0)
	_breadcrumb_back_button.focus_mode = Control.FOCUS_NONE
	_breadcrumb_back_button.add_theme_font_size_override("font_size", 12)
	_breadcrumb_back_button.pressed.connect(_go_back)
	breadcrumb_row.add_child(_breadcrumb_back_button)

	_breadcrumb_label = RichTextLabel.new()
	_breadcrumb_label.bbcode_enabled = true
	_breadcrumb_label.fit_content = true
	_breadcrumb_label.scroll_active = false
	_breadcrumb_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_breadcrumb_label.mouse_filter = Control.MOUSE_FILTER_STOP
	_breadcrumb_label.add_theme_font_size_override("normal_font_size", GameConfig.get_font_size("panel_small"))
	_breadcrumb_label.add_theme_color_override("default_color", Color(0.31, 0.41, 0.47))
	_breadcrumb_label.meta_clicked.connect(_on_breadcrumb_clicked)
	breadcrumb_row.add_child(_breadcrumb_label)

	var header_bg := PanelContainer.new()
	header_bg.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var header_style := StyleBoxFlat.new()
	header_style.bg_color = Color(0.06, 0.08, 0.12, 1.0)
	header_style.content_margin_left = 12
	header_style.content_margin_right = 12
	header_style.content_margin_top = 8
	header_style.content_margin_bottom = 6
	header_style.border_width_bottom = 1
	header_style.border_color = Color(0.15, 0.20, 0.28)
	header_bg.add_theme_stylebox_override("panel", header_style)
	root.add_child(header_bg)

	var header_hbox := HBoxContainer.new()
	header_hbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header_hbox.add_theme_constant_override("separation", 8)
	header_bg.add_child(header_hbox)

	_portrait = Control.new()
	_portrait.custom_minimum_size = PORTRAIT_SIZE
	_portrait.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_portrait.draw.connect(_draw_portrait)
	header_hbox.add_child(_portrait)

	var header_text_vbox := VBoxContainer.new()
	header_text_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header_text_vbox.add_theme_constant_override("separation", 2)
	header_hbox.add_child(header_text_vbox)

	var title_row := HBoxContainer.new()
	title_row.add_theme_constant_override("separation", 6)
	header_text_vbox.add_child(title_row)

	_header_name = Label.new()
	_header_name.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_header_name.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_title"))
	_header_name.add_theme_color_override("font_color", Color(0.95, 0.88, 0.52))
	title_row.add_child(_header_name)

	var action_row := HBoxContainer.new()
	action_row.add_theme_constant_override("separation", 4)
	title_row.add_child(action_row)

	_follow_button = _make_header_action_button("👁️")
	_follow_button.pressed.connect(_on_follow_pressed)
	action_row.add_child(_follow_button)

	_favorite_button = _make_header_action_button("⭐")
	_favorite_button.pressed.connect(_on_favorite_pressed)
	action_row.add_child(_favorite_button)

	_header_meta = Label.new()
	_header_meta.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_header_meta.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_small"))
	_header_meta.add_theme_color_override("font_color", Color(0.55, 0.60, 0.68))
	header_text_vbox.add_child(_header_meta)

	_summary_label = Label.new()
	_summary_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_summary_label.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_small"))
	_summary_label.add_theme_color_override("font_color", Color(0.70, 0.74, 0.80))
	header_text_vbox.add_child(_summary_label)

	_expand_tabs = TabContainer.new()
	_expand_tabs.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_expand_tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_expand_tabs.add_theme_stylebox_override("panel", _make_tab_panel_style())
	root.add_child(_expand_tabs)

	_tab_overview_text = _add_text_tab()
	_tab_needs_text = _add_text_tab()
	_tab_emotion_text = _add_text_tab()
	_tab_personality_text = _add_text_tab()
	_tab_health_text = _add_text_tab()
	_tab_knowledge_text = _add_text_tab()
	_tab_relationships_text = _add_text_tab()
	_tab_inventory_text = _add_text_tab()
	_tab_family_text = _add_text_tab()
	_tab_events_text = _add_text_tab()

	for tab_label: RichTextLabel in [
		_tab_overview_text,
		_tab_needs_text,
		_tab_emotion_text,
		_tab_personality_text,
		_tab_health_text,
		_tab_knowledge_text,
		_tab_relationships_text,
		_tab_inventory_text,
		_tab_family_text,
		_tab_events_text,
	]:
		tab_label.meta_clicked.connect(_on_tab_meta_clicked)

	var tab_bar: TabBar = _expand_tabs.get_tab_bar()
	_expand_tabs.clip_tabs = true
	tab_bar.clip_tabs = true
	tab_bar.tab_alignment = TabBar.ALIGNMENT_LEFT
	tab_bar.add_theme_font_size_override("font_size", 7)
	tab_bar.add_theme_stylebox_override("tab_selected", _make_tab_style(Color(0.10, 0.14, 0.20, 1.0), Color(0.80, 0.56, 0.18), 2))
	tab_bar.add_theme_stylebox_override("tab_unselected", _make_tab_style(Color(0.06, 0.08, 0.12, 1.0)))
	tab_bar.add_theme_stylebox_override("tab_hovered", _make_tab_style(Color(0.08, 0.11, 0.16, 1.0)))
	tab_bar.add_theme_color_override("font_selected_color", Color(0.95, 0.88, 0.52))
	tab_bar.add_theme_color_override("font_unselected_color", Color(0.45, 0.50, 0.58))
	tab_bar.add_theme_color_override("font_hovered_color", Color(0.70, 0.66, 0.50))

	_forage_label = Label.new()
	_forage_label.visible = false
	add_child(_forage_label)

	_narrative_panel = NarrativePanelScene.instantiate() as Control
	_narrative_panel.visible = false
	add_child(_narrative_panel)

	_thought_label = RichTextLabel.new()
	_thought_label.visible = false
	add_child(_thought_label)

	_expand_tabs.current_tab = 0
	_apply_locale()


func _apply_locale() -> void:
	_expand_tabs.set_tab_title(0, Locale.ltr("PANEL_OVERVIEW_TITLE"))
	_expand_tabs.set_tab_title(1, Locale.ltr("PANEL_NEEDS_TITLE"))
	_expand_tabs.set_tab_title(2, Locale.ltr("PANEL_EMOTION_TITLE"))
	_expand_tabs.set_tab_title(3, Locale.ltr("PANEL_PERSONALITY_TITLE"))
	_expand_tabs.set_tab_title(4, Locale.ltr("PANEL_HEALTH_TITLE"))
	_expand_tabs.set_tab_title(5, Locale.ltr("PANEL_KNOWLEDGE_TITLE"))
	_expand_tabs.set_tab_title(6, Locale.ltr("PANEL_RELATIONSHIPS_TITLE"))
	_expand_tabs.set_tab_title(7, Locale.ltr("UI_INVENTORY"))
	_expand_tabs.set_tab_title(8, Locale.ltr("PANEL_FAMILY_TITLE"))
	_expand_tabs.set_tab_title(9, Locale.ltr("PANEL_EVENTS_TITLE"))
	if _follow_button != null:
		_follow_button.tooltip_text = Locale.ltr("UI_BTN_FOLLOW")
	if _favorite_button != null:
		_favorite_button.tooltip_text = Locale.ltr("UI_BTN_FAVORITE")
	_update_breadcrumb()


func _refresh_all() -> void:
	if not is_inside_tree():
		return
	if _expand_tabs == null:
		return
	if _detail.is_empty():
		return
	_refresh_header()
	_refresh_summary()
	_update_breadcrumb()
	_refresh_expand_tabs()
	if _portrait != null:
		_portrait.queue_redraw()


func _refresh_expand_tabs() -> void:
	if _tab_overview_text == null:
		return
	var overview_text: String = _format_overview_tab_text()
	_set_text_tab_content(_tab_overview_text, overview_text, "overview")
	var needs_text: String = _format_needs_tab_text()
	_set_text_tab_content(_tab_needs_text, needs_text, "needs")
	var emotion_text: String = _format_emotion_tab_text()
	_set_text_tab_content(_tab_emotion_text, emotion_text, "emotion")
	var personality_text: String = _format_personality_tab_text()
	_set_text_tab_content(_tab_personality_text, personality_text, "personality")
	var health_text: String = _format_health_tab_text()
	_set_text_tab_content(_tab_health_text, health_text, "health")
	var knowledge_text: String = _format_knowledge_tab_text()
	_set_text_tab_content(_tab_knowledge_text, knowledge_text, "knowledge")
	var relationships_text: String = _format_relationships_tab_text()
	_set_text_tab_content(_tab_relationships_text, relationships_text, "relationships")
	var inventory_text: String = _format_inventory_tab_text()
	_set_text_tab_content(_tab_inventory_text, inventory_text, "inventory")
	var family_text: String = _format_family_tab_text()
	_set_text_tab_content(_tab_family_text, family_text, "family")
	var events_text: String = _format_events_tab_text()
	_set_text_tab_content(_tab_events_text, events_text, "events")


func _format_overview_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()

	# --- Section 1: Alerts ---
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_OVERVIEW_ALERTS"))
	var hunger: float = _safe_float(_detail, "need_hunger", 1.0)
	var sleep_default: float = _safe_scalar(_detail.get("energy", 1.0), 1.0)
	var sleep_need: float = _safe_float(_detail, "need_sleep", sleep_default)
	var stress_value: float = _normalized_stress()
	if hunger < 0.35:
		lines.append("  [color=#c83838]⚠ %s[/color]" % Locale.ltr("ALERT_HUNGRY"))
	if sleep_need < 0.30:
		lines.append("  [color=#c88818]⚠ %s[/color]" % Locale.ltr("ALERT_TIRED"))
	if stress_value > 0.30:
		lines.append("  [color=#c84a32]⚠ %s[/color]" % Locale.ltr("ALERT_STRESSED"))
	if hunger >= 0.35 and sleep_need >= 0.30 and stress_value <= 0.30:
		lines.append("  [color=#48a828]✓ %s[/color]" % Locale.ltr("ALERT_ALL_GOOD"))

	# --- Section 2: Info ---
	lines.append("")
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_OVERVIEW_INFO"))
	var occupation_text: String = _localized_action_text(str(_detail.get("occupation", "none")))
	var age_val: int = int(round(_safe_scalar(_detail.get("age_years", 0.0), 0.0)))
	var action_text: String = _localized_action_text(str(_detail.get("current_action", "Idle")))
	var band_text: String = _band_label()
	lines.append("  %s: %s  |  %s: %d%s" % [Locale.ltr("UI_JOB"), occupation_text, Locale.ltr("UI_AGE"), age_val, Locale.ltr("UI_AGE_UNIT")])
	lines.append("  %s: %s  |  %s: %s" % [Locale.ltr("UI_ACTION"), action_text, Locale.ltr("UI_BAND"), band_text])

	# --- Section 3: Needs bar ---
	lines.append("")
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_OVERVIEW_NEEDS"))
	var warmth_need: float = _safe_float(_detail, "need_warmth", 0.5)
	var safety_need: float = _safe_float(_detail, "need_safety", 0.5)
	lines.append(_format_bar_table([
		{"label": Locale.ltr("NEED_HUNGER"), "value": hunger, "color": _need_color(hunger)},
		{"label": Locale.ltr("NEED_SLEEP"), "value": sleep_need, "color": _need_color(sleep_need)},
		{"label": Locale.ltr("NEED_WARMTH"), "value": warmth_need, "color": _need_color(warmth_need)},
		{"label": Locale.ltr("NEED_SAFETY"), "value": safety_need, "color": _need_color(safety_need)},
	]))
	return "\n".join(lines)


func _format_needs_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	var entries: Array[Dictionary] = []
	for entry: Dictionary in _build_need_entries():
		var key_name: String = str(entry.get("key", "UI_UNKNOWN"))
		var value: float = _sanitize_unit_float(entry.get("value", 0.0), 0.0)
		entries.append({
			"label": Locale.ltr(key_name),
			"value": value,
			"color": _need_color(value),
		})
	lines.append(_format_bar_table(entries))
	return "\n".join(lines)


func _format_emotion_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	var entries: Array[Dictionary] = []
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_EMOTION_TITLE"))
	lines.append("")
	for row: Dictionary in EMOTION_ROWS:
		var field_name: String = str(row.get("field", ""))
		var value: float = _sanitize_unit_float(_detail.get(field_name, 0.0), 0.0)
		var key_str: String = str(row.get("key", "UI_UNKNOWN"))
		entries.append({
			"label": Locale.ltr(key_str),
			"value": value,
			"color": _emotion_to_color(key_str),
		})
	entries.sort_custom(func(left: Dictionary, right: Dictionary) -> bool:
		return float(left.get("value", 0.0)) > float(right.get("value", 0.0))
	)
	var stress_val: float = _normalized_stress()
	lines.append("")
	entries.append({"label": Locale.ltr("UI_STRESS"), "value": stress_val, "color": Color(0.78, 0.34, 0.28)})
	lines.append(_format_bar_table(entries))
	return "\n".join(lines)


func _format_personality_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	var archetype_key: String = str(_detail.get("archetype_key", "ARCHETYPE_QUIET_OBSERVER"))
	lines.append("[b]%s[/b]: %s" % [Locale.ltr("PANEL_PERSONALITY_TITLE"), Locale.ltr(archetype_key)])
	var temperament_key: String = str(_detail.get("temperament_label_key", ""))
	if not temperament_key.is_empty():
		lines.append(Locale.ltr(temperament_key))
	lines.append("")

	lines.append("[b]%s[/b]" % Locale.ltr("UI_TCI_TITLE"))
	var tci_ns: float = _safe_float(_detail, "tci_ns", 0.5)
	var tci_ha: float = _safe_float(_detail, "tci_ha", 0.5)
	var tci_rd: float = _safe_float(_detail, "tci_rd", 0.5)
	var tci_p: float = _safe_float(_detail, "tci_p", 0.5)
	lines.append("%s %d%%  %s %d%%  %s %d%%  %s %d%%" % [
		Locale.ltr("UI_TCI_NS"), int(round(tci_ns * 100.0)),
		Locale.ltr("UI_TCI_HA"), int(round(tci_ha * 100.0)),
		Locale.ltr("UI_TCI_RD"), int(round(tci_rd * 100.0)),
		Locale.ltr("UI_TCI_P"), int(round(tci_p * 100.0)),
	])
	lines.append("")

	lines.append("[b]%s[/b]" % Locale.ltr("UI_HEXACO_TITLE"))
	for axis_row: Dictionary in HEXACO_ROWS:
		var field_name: String = str(axis_row.get("field", ""))
		var axis_value: float = _sanitize_unit_float(_detail.get(field_name, 0.0), 0.0)
		lines.append("%s %d%%" % [Locale.ltr(str(axis_row.get("key", "UI_UNKNOWN"))), int(round(axis_value * 100.0))])
	lines.append("")

	lines.append("[b]%s[/b]" % Locale.ltr("UI_TRAITS_TITLE"))
	var trait_tags: PackedStringArray = _trait_tags()
	if trait_tags.is_empty():
		lines.append("—")
	else:
		lines.append(" ".join(trait_tags))
		lines.append(str(trait_tags.size()) + Locale.ltr("UI_TRAITS_TOTAL"))
	lines.append("")

	lines.append("[b]%s[/b]" % Locale.ltr("UI_VALUES_TITLE"))
	var values_ranked: Array[Dictionary] = _value_rankings()
	if values_ranked.is_empty():
		lines.append("—")
	else:
		for value_entry: Dictionary in values_ranked:
			var val_key: String = str(value_entry.get("key", "UI_UNKNOWN"))
			if val_key.is_empty():
				val_key = "UI_UNKNOWN"
			var val_score: float = _safe_scalar(value_entry.get("value", 0.0), 0.0)
			lines.append("%s %d%%" % [Locale.ltr(val_key), int(round(val_score * 100.0))])
	return "\n".join(lines)


func _format_health_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_HEALTH_TITLE"))
	lines.append("")
	lines.append("[b][color=#283848]%s[/color][/b]" % Locale.ltr("PANEL_HEALTH_AGGREGATE"))
	var aggregate_hp: float = _safe_float(_health_tab, "aggregate_hp", 1.0)
	lines.append(_format_bar_table([
		{"label": Locale.ltr("PANEL_HEALTH_AGGREGATE"), "value": aggregate_hp, "color": _need_color(aggregate_hp)}
	]))
	lines.append("")
	lines.append("[b][color=#283848]%s[/color][/b]" % Locale.ltr("PANEL_HEALTH_GROUPS"))
	var health_entries: Array[Dictionary] = []
	for group_entry: Dictionary in _merged_health_groups():
		var hp_value: float = clampf(_safe_scalar(group_entry.get("value", 0.0), 0.0), 0.0, 1.0)
		health_entries.append({
			"label": Locale.ltr(str(group_entry.get("label", "UI_UNKNOWN"))),
			"value": hp_value,
			"color": _need_color(hp_value),
		})
	lines.append(_format_bar_table(health_entries))
	lines.append("")
	lines.append("[b][color=#283848]%s[/color][/b]" % Locale.ltr("UI_DERIVED_STATS"))
	var move_mult: float = clampf(_safe_scalar(_health_tab.get("move_mult", 1.0), 1.0) / 1.5, 0.0, 1.0)
	var work_mult: float = clampf(_safe_scalar(_health_tab.get("work_mult", 1.0), 1.0) / 1.5, 0.0, 1.0)
	var combat_mult: float = clampf(_safe_scalar(_health_tab.get("combat_mult", 1.0), 1.0) / 1.5, 0.0, 1.0)
	var pain_value: float = _safe_float(_health_tab, "pain", 0.0)
	lines.append(_format_bar_table([
		{"label": Locale.ltr("UI_MOVE"), "value": move_mult, "color": Color(0.36, 0.76, 0.48)},
		{"label": Locale.ltr("UI_WORK"), "value": work_mult, "color": Color(0.42, 0.56, 0.82)},
		{"label": Locale.ltr("UI_COMBAT"), "value": combat_mult, "color": Color(0.82, 0.34, 0.28)},
		{"label": Locale.ltr("UI_PAIN"), "value": pain_value, "color": Color(0.86, 0.68, 0.24)},
	]))
	var damaged_parts: Array = _health_tab.get("damaged_parts", [])
	if not damaged_parts.is_empty():
		lines.append("")
		lines.append("[b][color=#c84a32]%s[/color][/b]" % Locale.ltr("PANEL_HEALTH_INJURIES"))
		var injury_entries: Array[Dictionary] = []
		for part_raw: Variant in damaged_parts:
			if not (part_raw is Dictionary):
				continue
			var part: Dictionary = part_raw
			var part_hp: float = clampf(_safe_scalar(part.get("hp", 0), 0.0) / 100.0, 0.0, 1.0)
			injury_entries.append({
				"label": ("%s%s" % ["⚠ " if bool(part.get("vital", false)) else "", _localized_body_part_name(str(part.get("name", "")))]).strip_edges(),
				"value": part_hp,
				"color": _need_color(part_hp),
			})
		lines.append(_format_bar_table(injury_entries))
	return "\n".join(lines)


func _format_knowledge_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_KNOWLEDGE_TITLE"))
	lines.append("")
	var known: Array = _knowledge_tab.get("known", [])
	if known.is_empty():
		lines.append("[color=#384850]%s[/color]" % Locale.ltr("UI_NO_KNOWLEDGE"))
	else:
		var knowledge_entries: Array[Dictionary] = []
		var source_lines: PackedStringArray = PackedStringArray()
		for knowledge_raw: Variant in known:
			if not (knowledge_raw is Dictionary):
				continue
			var knowledge: Dictionary = knowledge_raw
			var knowledge_id: String = str(knowledge.get("id", Locale.ltr("UI_UNKNOWN")))
			var display_name: String = Locale.ltr(knowledge_id) if Locale.has_key(knowledge_id) else _display_token(knowledge_id)
			var proficiency: float = clampf(_safe_scalar(knowledge.get("proficiency", 0.0), 0.0), 0.0, 1.0)
			var source_index: int = clampi(int(knowledge.get("source", 0)), 0, 5)
			var source_key: Array[String] = [
				"KNOWLEDGE_SRC_SELF",
				"KNOWLEDGE_SRC_ORAL",
				"KNOWLEDGE_SRC_OBSERVED",
				"KNOWLEDGE_SRC_APPRENTICE",
				"KNOWLEDGE_SRC_RECORDED",
				"KNOWLEDGE_SRC_SCHOOL",
			]
			knowledge_entries.append({
				"label": display_name,
				"value": proficiency,
				"color": Color(0.45, 0.62, 0.84),
			})
			source_lines.append("  [color=#506878]%s[/color] %s" % [
				Locale.ltr("UI_KNOWLEDGE_SOURCE"),
				Locale.ltr(source_key[source_index]),
			])
		lines.append(_format_bar_table(knowledge_entries))
		for source_line: String in source_lines:
			lines.append(source_line)
		lines.append("")

	var skill_tokens: PackedStringArray = PackedStringArray()
	for knowledge_raw: Variant in known:
		if not (knowledge_raw is Dictionary):
			continue
		var knowledge_dict: Dictionary = knowledge_raw
		var knowledge_id: String = str(knowledge_dict.get("id", ""))
		if knowledge_id.is_empty():
			continue
		skill_tokens.append("[color=#7088a0]%s[/color]" % (Locale.ltr(knowledge_id) if Locale.has_key(knowledge_id) else _display_token(knowledge_id)))
	if not skill_tokens.is_empty():
		lines.append("[b][color=#283848]%s[/color][/b]" % Locale.ltr("UI_SKILLS"))
		lines.append("  ".join(skill_tokens))
		lines.append("")

	lines.append("[b][color=#283848]%s[/color][/b]" % Locale.ltr("UI_TRANSMISSION_CHANNELS"))
	for channel_entry: Dictionary in _knowledge_channels():
		var locked: bool = bool(channel_entry.get("locked", false))
		var tint: String = "#384850" if locked else "#7088a0"
		lines.append(
			"[color=%s]%s %s — %s[/color]" % [
				tint,
				str(channel_entry.get("icon", "")),
				Locale.ltr(str(channel_entry.get("label", "UI_UNKNOWN"))),
				Locale.ltr(str(channel_entry.get("status", "UI_UNKNOWN"))),
			]
		)
	lines.append("")
	lines.append("[b][color=#283848]%s[/color][/b]" % Locale.ltr("UI_RECORDS_TITLE"))
	lines.append("[color=#384850]%s[/color]" % Locale.ltr("UI_RECORDS_PLACEHOLDER"))
	lines.append("")
	lines.append(_format_bar_table([
		{"label": Locale.ltr("UI_INNOVATION"), "value": _safe_float(_knowledge_tab, "innovation_potential", 0.0), "color": Color(0.78, 0.56, 0.19)}
	]))
	return "\n".join(lines)


func _format_relationships_tab_text() -> String:
	return super._format_relationships_tab_text()


func _format_inventory_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_INVENTORY_TITLE"))
	lines.append("")
	var items: Array = _detail.get("inv_items", [])
	if items.is_empty():
		lines.append("[color=#384850]%s[/color]" % Locale.ltr("UI_NO_ITEMS"))
	else:
		var row_tokens: PackedStringArray = PackedStringArray()
		var item_quality_entries: Array[Dictionary] = []
		for index: int in range(items.size()):
			if not (items[index] is Dictionary):
				continue
			var item: Dictionary = items[index]
			var icon: String = _inventory_icon(str(item.get("template_id", "")))
			var display_name: String = _display_token(str(item.get("template_id", Locale.ltr("UI_UNKNOWN"))))
			var stack_count: int = maxi(1, int(item.get("stack_count", 1)))
			row_tokens.append("[color=#d8d0a0]%s[/color]%s" % [icon, "×%d" % stack_count if stack_count > 1 else ""])
			if row_tokens.size() == 5:
				lines.append(" ".join(row_tokens))
				row_tokens.clear()
			item_quality_entries.append({
				"label": display_name,
				"value": clampf(_safe_scalar(item.get("quality", 0.5), 0.5), 0.0, 1.0),
				"color": Color(0.58, 0.68, 0.32),
			})
		if not row_tokens.is_empty():
			lines.append(" ".join(row_tokens))
		if not item_quality_entries.is_empty():
			lines.append("")
			lines.append(_format_bar_table(item_quality_entries))
	lines.append("")
	lines.append("[b][color=#283848]%s[/color][/b]" % Locale.ltr("PANEL_EQUIPMENT_TITLE"))
	var equipment: Dictionary = _equipment_slots()
	lines.append("[center][color=#506878]    %s: %s[/color]" % [Locale.ltr("UI_EQUIP_HEAD"), str(equipment.get("head", "—"))])
	lines.append("[color=#506878]%s: %s[/color]    [color=#7088a0]◉[/color]    [color=#506878]%s: %s[/color]" % [
		Locale.ltr("UI_EQUIP_WEAPON"),
		str(equipment.get("weapon", "—")),
		Locale.ltr("UI_EQUIP_OFFHAND"),
		str(equipment.get("offhand", "—")),
	])
	lines.append("[color=#506878]    %s: %s[/color]" % [Locale.ltr("UI_EQUIP_BODY"), str(equipment.get("body", "—"))])
	lines.append("[color=#506878]    %s: %s[/color][/center]" % [Locale.ltr("UI_EQUIP_LEGS"), str(equipment.get("legs", "—"))])
	return "\n".join(lines)


func _format_family_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_FAMILY_TITLE"))
	lines.append("")

	var father_raw: Variant = _family_tab.get("father", {})
	var mother_raw: Variant = _family_tab.get("mother", {})
	var spouse_raw: Variant = _family_tab.get("spouse", {})
	var children: Array = _family_tab.get("children", [])
	var father_text: String = _family_member_text(father_raw)
	var mother_text: String = _family_member_text(mother_raw)
	var spouse_text: String = _family_member_text(spouse_raw, Locale.ltr("UI_NONE"))
	lines.append("[color=#506878]%s[/color]  %s" % [Locale.ltr("UI_FATHER"), father_text])
	lines.append("[color=#506878]%s[/color]  %s" % [Locale.ltr("UI_MOTHER"), mother_text])
	lines.append("[color=#506878]%s[/color]  %s" % [Locale.ltr("UI_SPOUSE"), spouse_text])
	if children.is_empty():
		lines.append("[color=#506878]%s[/color]  %s" % [Locale.ltr("UI_CHILDREN"), Locale.ltr("UI_NONE")])
	else:
		var child_links: PackedStringArray = PackedStringArray()
		for child_raw: Variant in children:
			if not (child_raw is Dictionary):
				continue
			var child: Dictionary = child_raw
			child_links.append(
				_entity_link(int(child.get("id", -1)), str(child.get("name", Locale.ltr("UI_UNKNOWN"))))
			)
		lines.append("[color=#506878]%s[/color]  %s" % [Locale.ltr("UI_CHILDREN"), ", ".join(child_links)])
	lines.append("")
	lines.append("[b][color=#283848]%s[/color][/b]" % Locale.ltr("UI_FAMILY_TREE"))
	lines.append("[center][color=#384850]%s ═ %s[/color]" % [_tree_member_text(father_raw), _tree_member_text(mother_raw)])
	lines.append("[color=#384850]      │[/color]")
	var spouse_center: String = ""
	if spouse_raw is Dictionary:
		var spouse_dict: Dictionary = spouse_raw
		if not spouse_dict.is_empty():
			spouse_center = " ═ " + _entity_link(int(spouse_dict.get("id", -1)), str(spouse_dict.get("name", Locale.ltr("UI_UNKNOWN"))))
	lines.append("[color=#c89030][b]★ %s[/b][/color]%s" % [str(_detail.get("name", Locale.ltr("UI_UNKNOWN"))), spouse_center])
	if not children.is_empty():
		lines.append("[color=#384850]      │[/color]")
		var child_names: PackedStringArray = PackedStringArray()
		for child_raw: Variant in children:
			if not (child_raw is Dictionary):
				continue
			var child: Dictionary = child_raw
			child_names.append(_entity_link(int(child.get("id", -1)), str(child.get("name", Locale.ltr("UI_UNKNOWN")))))
		lines.append("[color=#587080]%s[/color]" % " · ".join(child_names))
	lines.append("[/center]")
	lines.append("")
	lines.append("[b][color=#283848]%s[/color][/b] [color=#384850][%s][/color]" % [Locale.ltr("UI_CLAN"), Locale.ltr("UI_CIV_PLACEHOLDER_META")])
	lines.append("[color=#384850]%s[/color]" % Locale.ltr("UI_CLAN_PLACEHOLDER"))
	return "\n".join(lines)


func _format_events_tab_text() -> String:
	return super._format_events_tab_text()


func _localized_action_text(raw: String) -> String:
	return super._localized_action_text(raw)


func _localized_need_text(raw: String) -> String:
	return super._localized_need_text(raw)


func _localized_body_part_name(raw: String) -> String:
	return super._localized_body_part_name(raw)


func _build_need_entries() -> Array[Dictionary]:
	return super._build_need_entries()


func _build_emotion_entries() -> Array[Dictionary]:
	return super._build_emotion_entries()


func _build_relationship_entries(limit: int) -> Array[Dictionary]:
	return super._build_relationship_entries(limit)


func _format_relationship_entry(entry: Dictionary) -> String:
	var relation_type: String = str(entry.get("relation_type", ""))
	var marker_parts: PackedStringArray = PackedStringArray()
	if bool(entry.get("is_band_mate", false)):
		marker_parts.append("[B]")
	var relation_marker: String = _relationship_marker(relation_type)
	if not relation_marker.is_empty():
		marker_parts.append(relation_marker)
	var prefix: String = ""
	if not marker_parts.is_empty():
		prefix = " ".join(marker_parts) + " "
	var target_id: int = int(entry.get("target_id", -1))
	var headline: String = prefix + _entity_link(target_id, _resolve_entity_name(target_id))
	var relation_text: String = _localized_relation_text(relation_type)
	if not relation_text.is_empty():
		headline += " (%s)" % relation_text
	headline += "  %+d / %s %d" % [
		int(round(_safe_scalar(entry.get("affinity", 0.0), 0.0) * 100.0)),
		Locale.ltr("UI_TRUST"),
		int(round(_safe_scalar(entry.get("trust", 0.0), 0.0) * 100.0)),
	]
	headline += "\n%s %d" % [
		Locale.ltr("UI_FAMILIARITY"),
		int(round(_safe_scalar(entry.get("familiarity", 0.0), 0.0) * 100.0)),
	]
	return headline


func _story_event_text(entry: Dictionary) -> String:
	return super._story_event_text(entry)


func _add_text_tab() -> RichTextLabel:
	return super._add_text_tab()


func _make_tab_panel_style() -> StyleBoxFlat:
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.05, 0.07, 0.10, 1.0)
	style.content_margin_left = 8
	style.content_margin_right = 8
	style.content_margin_top = 6
	style.content_margin_bottom = 6
	return style


func _make_tab_style(bg_color: Color, border_color: Color = Color.TRANSPARENT, border_bottom: int = 0) -> StyleBoxFlat:
	var style := StyleBoxFlat.new()
	style.bg_color = bg_color
	style.content_margin_left = 3
	style.content_margin_right = 3
	style.content_margin_top = 3
	style.content_margin_bottom = 3
	style.border_color = border_color
	style.border_width_bottom = border_bottom
	return style


func _make_header_action_button(icon_text: String) -> Button:
	var button := Button.new()
	button.text = icon_text
	button.focus_mode = Control.FOCUS_NONE
	button.custom_minimum_size = Vector2(28.0, 24.0)
	button.add_theme_font_size_override("font_size", 12)
	return button


func _draw_portrait() -> void:
	if _portrait == null:
		return
	var rect := Rect2(Vector2.ZERO, PORTRAIT_SIZE)
	var emotion: String = str(_detail.get("dominant_emotion", ""))
	var emotion_category: String = _emotion_category(emotion)
	var stress: float = _normalized_stress()
	var sex: String = str(_detail.get("sex", "male")).to_lower()
	var age: float = _safe_scalar(_detail.get("age_years", 20.0), 20.0)
	var emotion_color: Color = _emotion_to_color(emotion)
	var skin: Color = Color(0.78, 0.66, 0.53) if sex == "female" else Color(0.72, 0.60, 0.47)
	var hair: Color = Color(0.66, 0.41, 0.19) if age < 40.0 else Color(0.50, 0.48, 0.45)

	_portrait.draw_rect(rect, Color(0.04, 0.06, 0.10), true)
	_portrait.draw_rect(rect, Color(emotion_color.r, emotion_color.g, emotion_color.b, 0.32), false, 2.0)
	_portrait.draw_circle(Vector2(24.0, 14.0), 13.0, hair)
	_portrait.draw_circle(Vector2(24.0, 22.0), 11.0, skin)

	if emotion_category == "joy" or emotion_category == "positive":
		_portrait.draw_arc(Vector2(19.0, 20.0), 2.0, 0.0, PI, 6, Color(0.22, 0.16, 0.09), 1.5)
		_portrait.draw_arc(Vector2(29.0, 20.0), 2.0, 0.0, PI, 6, Color(0.22, 0.16, 0.09), 1.5)
	elif emotion_category == "fear" or emotion_category == "sad":
		_portrait.draw_circle(Vector2(19.0, 20.0), 2.5, Color.WHITE)
		_portrait.draw_circle(Vector2(19.0, 20.0), 1.3, Color(0.22, 0.16, 0.09))
		_portrait.draw_circle(Vector2(29.0, 20.0), 2.5, Color.WHITE)
		_portrait.draw_circle(Vector2(29.0, 20.0), 1.3, Color(0.22, 0.16, 0.09))
	else:
		_portrait.draw_circle(Vector2(19.0, 20.0), 1.8, Color(0.22, 0.16, 0.09))
		_portrait.draw_circle(Vector2(29.0, 20.0), 1.8, Color(0.22, 0.16, 0.09))

	if emotion_category == "joy" or emotion_category == "positive":
		_portrait.draw_arc(Vector2(24.0, 27.0), 4.0, 0.0, PI, 8, Color(0.50, 0.31, 0.25), 1.2)
	elif emotion_category == "anger":
		_portrait.draw_line(Vector2(20.0, 28.0), Vector2(28.0, 28.0), Color(0.50, 0.31, 0.25), 1.2)
	else:
		_portrait.draw_line(Vector2(21.0, 28.0), Vector2(27.0, 28.0), Color(0.50, 0.31, 0.25), 1.0)

	_portrait.draw_rect(Rect2(16.0, 34.0, 16.0, 12.0), Color(skin.r, skin.g, skin.b, 0.52), true)
	var status_color: Color = Color(0.28, 0.66, 0.16) if stress < 0.30 else Color(0.78, 0.55, 0.10) if stress < 0.55 else Color(0.78, 0.22, 0.22)
	_portrait.draw_circle(Vector2(42.0, 42.0), 4.0, status_color)
	_portrait.draw_arc(Vector2(42.0, 42.0), 4.5, 0.0, TAU, 16, Color(0.04, 0.06, 0.10), 1.0)


func _emotion_to_color(emotion: String) -> Color:
	match _emotion_category(emotion):
		"joy", "positive":
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
	var emotion_key: String = emotion.strip_edges().to_lower()
	if emotion_key.contains("joy") or emotion_key.contains("기쁨") or emotion_key.contains("trust") or emotion_key.contains("신뢰"):
		return "joy"
	if emotion_key.contains("fear") or emotion_key.contains("공포") or emotion_key.contains("surprise") or emotion_key.contains("놀람"):
		return "fear"
	if emotion_key.contains("anger") or emotion_key.contains("분노") or emotion_key.contains("disgust") or emotion_key.contains("혐오"):
		return "anger"
	if emotion_key.contains("sad") or emotion_key.contains("슬픔"):
		return "sad"
	if emotion_key.contains("positive") or emotion_key.contains("행복"):
		return "positive"
	return "neutral"


func _bbcode_bar(label: String, value: float, color: Color) -> String:
	return _format_bar_table([{"label": label, "value": value, "color": color}])


func _bbcode_bar_inline(value: float, color: Color) -> String:
	return _format_bar_table([{"label": "", "value": value, "color": color}], 8)


func _format_bar_table(entries: Array[Dictionary], block_count: int = 12) -> String:
	if entries.is_empty():
		return ""
	var lines: PackedStringArray = PackedStringArray()
	lines.append("[table=3]")
	for entry: Dictionary in entries:
		var label: String = str(entry.get("label", ""))
		var clamped: float = _sanitize_unit_float(entry.get("value", 0.0), 0.0)
		var filled: int = clampi(int(round(clamped * float(block_count))), 0, block_count)
		var raw_color: Variant = entry.get("color", Color(0.5, 0.6, 0.7))
		var color: Color = raw_color if raw_color is Color else Color(0.5, 0.6, 0.7)
		var bar_filled: String = "█".repeat(filled)
		var bar_empty: String = "░".repeat(block_count - filled)
		var pct: String = str(int(round(clamped * 100.0)))
		lines.append("[cell][color=#506878]%s[/color][/cell][cell][color=%s]%s[/color][color=#182430]%s[/color][/cell][cell] [color=#8898a8]%s%%[/color][/cell]" % [
			label, _color_hex(color), bar_filled, bar_empty, pct])
	lines.append("[/table]")
	return "\n".join(lines)


func _format_percent_list(entries: Array[Dictionary]) -> String:
	if entries.is_empty():
		return ""
	var lines: PackedStringArray = PackedStringArray()
	for entry: Dictionary in entries:
		var label: String = str(entry.get("label", ""))
		var clamped: float = _sanitize_unit_float(entry.get("value", 0.0), 0.0)
		lines.append(label + " " + str(int(round(clamped * 100.0))) + "%")
	return "\n".join(lines)


func _set_text_tab_content(label: RichTextLabel, content: String, _tag: String) -> void:
	if label == null:
		return
	label.clear()
	label.append_text(content)


func _color_hex(color: Color) -> String:
	return "#" + color.to_html(false)


func _card_line(color_hex: String, title: String, detail: String) -> String:
	return "[color=%s]▎ [b]%s[/b][/color]  [color=#8898a8]%s[/color]" % [color_hex, title, detail]


func _band_label() -> String:
	var band_name: String = str(_detail.get("band_name", ""))
	return band_name if not band_name.is_empty() else Locale.ltr("UI_NONE")


func _normalized_stress() -> float:
	var raw_value: float = _safe_scalar(_detail.get("stress_level", 0.0), 0.0)
	if raw_value <= 1.0:
		return clampf(raw_value, 0.0, 1.0)
	return clampf(raw_value / 1000.0, 0.0, 1.0)


func _safe_locale_text(key: String) -> String:
	if key.is_empty():
		return ""
	var key_id: int = Locale.key_id(key)
	if key_id >= 0:
		var direct_text: String = Locale.ltr_id(key_id)
		if not direct_text.is_empty():
			return direct_text
	if Locale.has_key(key):
		return key
	return key


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
	# Non-numeric Variant from FFI — this would have crashed with raw float()
	return default_value


func _safe_float(dict: Dictionary, key: String, default_value: float) -> float:
	return _sanitize_unit_float(dict.get(key, default_value), default_value)


func _sanitize_unit_float(raw: Variant, default_value: float) -> float:
	var scalar_value: float = _safe_scalar(raw, default_value)
	if is_nan(scalar_value) or is_inf(scalar_value):
		return clampf(default_value, 0.0, 1.0)
	return clampf(scalar_value, 0.0, 1.0)


func _trait_tags() -> PackedStringArray:
	var tags: PackedStringArray = PackedStringArray()
	var hex_c: float = _sanitize_unit_float(_detail.get("hex_c", 0.0), 0.0)
	var hex_a: float = _sanitize_unit_float(_detail.get("hex_a", 0.0), 0.0)
	var hex_o: float = _sanitize_unit_float(_detail.get("hex_o", 0.0), 0.0)
	var hex_x: float = _sanitize_unit_float(_detail.get("hex_x", 0.0), 0.0)
	var hex_h: float = _sanitize_unit_float(_detail.get("hex_h", 0.0), 0.0)
	var hex_e: float = _sanitize_unit_float(_detail.get("hex_e", 0.0), 0.0)
	var tci_ns: float = _sanitize_unit_float(_detail.get("tci_ns", 0.0), 0.0)
	var tci_p: float = _sanitize_unit_float(_detail.get("tci_p", 0.0), 0.0)
	if hex_c >= 0.65:
		tags.append("[color=#6888a8][%s][/color]" % Locale.ltr("VALUE_HARD_WORK"))
	if hex_a >= 0.65:
		tags.append("[color=#6888a8][%s][/color]" % Locale.ltr("VALUE_HARMONY"))
	if hex_o >= 0.65:
		tags.append("[color=#6888a8][%s][/color]" % Locale.ltr("VALUE_KNOWLEDGE"))
	if hex_x >= 0.65:
		tags.append("[color=#6888a8][%s][/color]" % Locale.ltr("VALUE_FRIENDSHIP"))
	if hex_h >= 0.65:
		tags.append("[color=#6888a8][%s][/color]" % Locale.ltr("VALUE_TRUTH"))
	if hex_e >= 0.65:
		tags.append("[color=#6888a8][%s][/color]" % Locale.ltr("VALUE_FAMILY"))
	if tci_ns >= 0.65:
		tags.append("[color=#6888a8][%s][/color]" % Locale.ltr("VALUE_INDEPENDENCE"))
	if tci_p >= 0.65:
		tags.append("[color=#6888a8][%s][/color]" % Locale.ltr("VALUE_PERSEVERANCE"))
	return tags


func _value_rankings() -> Array[Dictionary]:
	var values_raw: Variant = _mind_tab.get("values_all", null)
	var ranked: Array[Dictionary] = []
	if values_raw == null:
		return ranked
	# Build label lookup from ValueDefs.KEYS (StringName→String safe)
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
			ranked.append({"key": str(labels[index]), "value": _safe_scalar(values_raw[index], 0.0)})
	else:
		return ranked
	ranked.sort_custom(func(left: Dictionary, right: Dictionary) -> bool:
		return _safe_scalar(left.get("value", 0.0), 0.0) > _safe_scalar(right.get("value", 0.0), 0.0)
	)
	if ranked.size() > 5:
		ranked.resize(5)
	return ranked


func _merged_health_groups() -> Array[Dictionary]:
	var hp_raw: PackedByteArray = _health_tab.get("group_hp", PackedByteArray())
	var values: Array[float] = []
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
	if token.contains("axe"):
		return "🪓"
	if token.contains("spear") or token.contains("knife"):
		return "🗡️"
	if token.contains("stone"):
		return "🪨"
	if token.contains("wood") or token.contains("log"):
		return "🪵"
	if token.contains("food") or token.contains("meat"):
		return "🍖"
	if token.contains("fiber") or token.contains("cloth"):
		return "🧵"
	return "📦"


func _equipment_slots() -> Dictionary:
	var slots: Dictionary = {
		"head": "—",
		"body": "—",
		"weapon": "—",
		"offhand": "—",
		"legs": "—",
	}
	var items: Array = _detail.get("inv_items", [])
	for item_raw: Variant in items:
		if not (item_raw is Dictionary):
			continue
		var item: Dictionary = item_raw
		var item_name: String = _display_token(str(item.get("template_id", Locale.ltr("UI_UNKNOWN"))))
		var token: String = str(item.get("template_id", "")).to_lower()
		if slots["weapon"] == "—" and (token.contains("axe") or token.contains("knife") or token.contains("spear") or token.contains("club")):
			slots["weapon"] = item_name
		elif slots["offhand"] == "—" and (token.contains("torch") or token.contains("basket") or token.contains("shield")):
			slots["offhand"] = item_name
		elif slots["head"] == "—" and (token.contains("hood") or token.contains("hat") or token.contains("helm")):
			slots["head"] = item_name
		elif slots["body"] == "—" and (token.contains("shirt") or token.contains("coat") or token.contains("armor")):
			slots["body"] = item_name
		elif slots["legs"] == "—" and (token.contains("pants") or token.contains("shoe") or token.contains("boot")):
			slots["legs"] = item_name
	return slots


func _display_token(token: String) -> String:
	if token.is_empty():
		return Locale.ltr("UI_UNKNOWN")
	var locale_key: String = token.to_upper()
	if Locale.has_key(locale_key):
		return Locale.ltr(locale_key)
	var display_text: String = token.replace("_", " ")
	return display_text.capitalize()


func _family_member_text(member_raw: Variant, fallback: String = "") -> String:
	if member_raw is Dictionary:
		var member: Dictionary = member_raw
		if not member.is_empty():
			return _entity_link(int(member.get("id", -1)), str(member.get("name", Locale.ltr("UI_UNKNOWN"))))
	return fallback if not fallback.is_empty() else Locale.ltr("UI_UNKNOWN")


func _tree_member_text(member_raw: Variant) -> String:
	if member_raw is Dictionary:
		var member: Dictionary = member_raw
		if not member.is_empty():
			return _entity_link(int(member.get("id", -1)), str(member.get("name", Locale.ltr("UI_UNKNOWN"))))
	return Locale.ltr("UI_UNKNOWN")


func _entity_link(entity_id: int, entity_name: String) -> String:
	if entity_id < 0 or entity_name.is_empty():
		return entity_name
	return "[url=entity:%d]%s[/url]" % [entity_id, entity_name]


func _on_tab_meta_clicked(meta: Variant) -> void:
	var meta_text: String = str(meta)
	if meta_text.begins_with("entity:"):
		_navigate_to_entity(int(meta_text.substr(7)))
	elif meta_text.begins_with("sett:"):
		var settlement_id: int = int(meta_text.substr(5))
		if settlement_id >= 0:
			SimulationBus.settlement_panel_requested.emit(settlement_id)
	elif meta_text.begins_with("band:"):
		var band_id: int = int(meta_text.substr(5))
		if band_id >= 0:
			SimulationBus.band_selected.emit(band_id)


func _on_breadcrumb_clicked(meta: Variant) -> void:
	_on_tab_meta_clicked(meta)


func _navigate_to_entity(entity_id: int) -> void:
	if entity_id < 0 or entity_id == _selected_entity_id:
		return
	if _selected_entity_id >= 0:
		_nav_stack.append({"type": "entity", "id": _selected_entity_id})
		if _nav_stack.size() > 10:
			_nav_stack.pop_front()
	_update_breadcrumb()
	SimulationBus.entity_selected.emit(entity_id)


func _go_back() -> void:
	if _nav_stack.is_empty():
		return
	var previous: Dictionary = _nav_stack.pop_back()
	if str(previous.get("type", "")) == "entity":
		var entity_id: int = int(previous.get("id", -1))
		if entity_id >= 0:
			SimulationBus.entity_selected.emit(entity_id)
	_update_breadcrumb()


func _update_breadcrumb() -> void:
	if _breadcrumb_wrapper == null or _breadcrumb_label == null:
		return
	var parts: PackedStringArray = PackedStringArray()
	var settlement_id: int = int(_detail.get("settlement_id", -1))
	var settlement_name: String = ""
	if settlement_id >= 0 and _sim_engine != null:
		var settlement_detail: Dictionary = _sim_engine.get_settlement_detail(settlement_id)
		settlement_name = str(settlement_detail.get("name", ""))
	if not settlement_name.is_empty():
		parts.append("[url=sett:%d]%s[/url]" % [settlement_id, settlement_name])
	var band_name: String = str(_detail.get("band_name", ""))
	var band_id: int = int(_detail.get("band_id", -1))
	if not band_name.is_empty() and band_id >= 0:
		parts.append("[url=band:%d]%s[/url]" % [band_id, band_name])
	var entity_name: String = str(_detail.get("name", ""))
	if not entity_name.is_empty():
		parts.append("[color=#b8c8d8]%s[/color]" % entity_name)
	_breadcrumb_back_button.visible = not _nav_stack.is_empty()
	_breadcrumb_back_button.disabled = _nav_stack.is_empty()
	_breadcrumb_wrapper.visible = parts.size() > 1 or not _nav_stack.is_empty()
	_breadcrumb_label.clear()
	if not parts.is_empty():
		_breadcrumb_label.append_text(" › ".join(parts))


func _on_follow_pressed() -> void:
	if _selected_entity_id >= 0:
		SimulationBus.follow_entity_requested.emit(_selected_entity_id)


func _on_favorite_pressed() -> void:
	pass
