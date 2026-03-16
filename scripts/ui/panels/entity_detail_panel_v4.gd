extends "res://scripts/ui/panels/entity_detail_panel_v3.gd"
class_name EntityDetailPanelV4

var _breadcrumb_wrapper: PanelContainer
var _breadcrumb_label: RichTextLabel
var _breadcrumb_back_button: Button
var _follow_button: Button
var _favorite_button: Button
var _nav_stack: Array[Dictionary] = []


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
	style.content_margin_left = 0
	style.content_margin_right = 0
	style.content_margin_top = 0
	style.content_margin_bottom = 0
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

	var header_vbox := VBoxContainer.new()
	header_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header_vbox.add_theme_constant_override("separation", 2)
	header_bg.add_child(header_vbox)

	var title_row := HBoxContainer.new()
	title_row.add_theme_constant_override("separation", 6)
	header_vbox.add_child(title_row)

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
	header_vbox.add_child(_header_meta)

	_summary_label = Label.new()
	_summary_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_summary_label.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_small"))
	_summary_label.add_theme_color_override("font_color", Color(0.70, 0.74, 0.80))
	header_vbox.add_child(_summary_label)

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
		_tab_family_text,
		_tab_events_text,
	]:
		tab_label.meta_clicked.connect(_on_tab_meta_clicked)

	var tab_bar: TabBar = _expand_tabs.get_tab_bar()
	tab_bar.clip_tabs = false
	tab_bar.tab_alignment = TabBar.ALIGNMENT_LEFT
	tab_bar.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_small"))
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
	_expand_tabs.set_tab_title(7, Locale.ltr("PANEL_FAMILY_TITLE"))
	_expand_tabs.set_tab_title(8, Locale.ltr("PANEL_EVENTS_TITLE"))
	if _follow_button != null:
		_follow_button.tooltip_text = Locale.ltr("UI_BTN_FOLLOW")
	if _favorite_button != null:
		_favorite_button.tooltip_text = Locale.ltr("UI_BTN_FAVORITE")
	_update_breadcrumb()


func _refresh_all() -> void:
	if _detail.is_empty():
		return
	_refresh_header()
	_refresh_summary()
	_update_breadcrumb()
	_refresh_expand_tabs()


func _format_overview_tab_text() -> String:
	return super._format_overview_tab_text()


func _format_needs_tab_text() -> String:
	return super._format_needs_tab_text()


func _format_emotion_tab_text() -> String:
	return super._format_emotion_tab_text()


func _format_personality_tab_text() -> String:
	return super._format_personality_tab_text()


func _format_health_tab_text() -> String:
	return super._format_health_tab_text()


func _format_knowledge_tab_text() -> String:
	return super._format_knowledge_tab_text()


func _format_relationships_tab_text() -> String:
	return super._format_relationships_tab_text()


func _format_family_tab_text() -> String:
	var lines: PackedStringArray = PackedStringArray()
	var kinship_names: Array[String] = [
		"KINSHIP_BILATERAL",
		"KINSHIP_PATRILINEAL",
		"KINSHIP_MATRILINEAL",
	]
	var father_raw: Variant = _family_tab.get("father", {})
	var mother_raw: Variant = _family_tab.get("mother", {})
	var spouse_raw: Variant = _family_tab.get("spouse", {})
	var father_name: String = Locale.ltr("PANEL_FAMILY_UNKNOWN")
	if father_raw is Dictionary and not (father_raw as Dictionary).is_empty():
		father_name = _entity_link(
			int((father_raw as Dictionary).get("id", -1)),
			str((father_raw as Dictionary).get("name", father_name))
		)
	var mother_name: String = Locale.ltr("PANEL_FAMILY_UNKNOWN")
	if mother_raw is Dictionary and not (mother_raw as Dictionary).is_empty():
		mother_name = _entity_link(
			int((mother_raw as Dictionary).get("id", -1)),
			str((mother_raw as Dictionary).get("name", mother_name))
		)
	var spouse_name: String = ""
	if spouse_raw is Dictionary and not (spouse_raw as Dictionary).is_empty():
		spouse_name = _entity_link(
			int((spouse_raw as Dictionary).get("id", -1)),
			str((spouse_raw as Dictionary).get("name", ""))
		)
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_FAMILY_TITLE"))
	lines.append("")
	lines.append("  %s: %s" % [Locale.ltr("PANEL_FAMILY_FATHER"), father_name])
	lines.append("  %s: %s" % [Locale.ltr("PANEL_FAMILY_MOTHER"), mother_name])
	if spouse_name.is_empty():
		lines.append("  %s: %s" % [Locale.ltr("PANEL_FAMILY_SPOUSE"), Locale.ltr("PANEL_FAMILY_NONE")])
	else:
		lines.append("  %s: %s" % [Locale.ltr("PANEL_FAMILY_SPOUSE"), spouse_name])
	var children: Array = _family_tab.get("children", [])
	if children.is_empty():
		lines.append("  %s: %s" % [Locale.ltr("PANEL_FAMILY_CHILDREN"), Locale.ltr("PANEL_FAMILY_NONE")])
	else:
		var child_names: PackedStringArray = PackedStringArray()
		for child_raw: Variant in children:
			if not (child_raw is Dictionary):
				continue
			var child: Dictionary = child_raw
			child_names.append(
				"%s(%d)" % [
					_entity_link(int(child.get("id", -1)), str(child.get("name", "?"))),
					int(child.get("age", 0)),
				]
			)
		lines.append("  %s: %s" % [Locale.ltr("PANEL_FAMILY_CHILDREN"), ", ".join(child_names)])
	lines.append("")
	lines.append("  %s: %d" % [Locale.ltr("PANEL_FAMILY_GENERATION"), int(_family_tab.get("generation", 0))])
	var kinship_index: int = clampi(int(_family_tab.get("kinship_type", 0)), 0, kinship_names.size() - 1)
	lines.append(
		"  %s: %s" % [
			Locale.ltr("PANEL_FAMILY_KINSHIP"),
			Locale.ltr(kinship_names[kinship_index]),
		]
	)
	var clan_id: int = int(_family_tab.get("clan_id", -1))
	if clan_id >= 0:
		lines.append("  %s: #%d" % [Locale.ltr("PANEL_FAMILY_CLAN"), clan_id])
	else:
		lines.append("  %s: %s" % [Locale.ltr("PANEL_FAMILY_CLAN"), Locale.ltr("PANEL_FAMILY_NONE_YET")])
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
		int(round(float(entry.get("affinity", 0.0)) * 100.0)),
		Locale.ltr("UI_TRUST"),
		int(round(float(entry.get("trust", 0.0)) * 100.0)),
	]
	headline += "\n%s %s %d" % [
		Locale.ltr("UI_FAMILIARITY"),
		_familiarity_bar(float(entry.get("familiarity", 0.0))),
		int(round(float(entry.get("familiarity", 0.0)) * 100.0)),
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
	style.content_margin_left = 6
	style.content_margin_right = 6
	style.content_margin_top = 4
	style.content_margin_bottom = 4
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
		var leader_id: int = _current_band_leader_id()
		if leader_id >= 0 and leader_id != _selected_entity_id:
			_navigate_to_entity(leader_id)


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


func _current_band_leader_id() -> int:
	var members: Array = _detail.get("band_members", [])
	for member_raw: Variant in members:
		if not (member_raw is Dictionary):
			continue
		var member: Dictionary = member_raw
		if bool(member.get("is_leader", false)):
			return int(member.get("entity_id", -1))
	return -1


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
