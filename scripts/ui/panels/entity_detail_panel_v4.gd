extends "res://scripts/ui/panels/entity_detail_panel_v3.gd"
class_name EntityDetailPanelV4


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

	_header_name = Label.new()
	_header_name.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_title"))
	_header_name.add_theme_color_override("font_color", Color(0.95, 0.88, 0.52))
	header_vbox.add_child(_header_name)

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

	# Hidden compatibility nodes preserved for inherited refresh/process code paths.
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


func _refresh_all() -> void:
	if _detail.is_empty():
		return
	_refresh_header()
	_refresh_summary()
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
	return super._format_family_tab_text()


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
	return super._format_relationship_entry(entry)


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
