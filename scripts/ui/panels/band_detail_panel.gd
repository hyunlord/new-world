extends PanelContainer

const GameCalendar = preload("res://scripts/core/simulation/game_calendar.gd")
const TAB_KEYS: Array[String] = ["UI_TAB_OVERVIEW", "UI_BAND_TAB_MEMBERS", "UI_BAND_TAB_EVENTS"]

var _sim_engine: RefCounted
var _band_id: int = -1
var _detail: Dictionary = {}
var _current_tab: int = 0

var _header_name: Label
var _header_meta: Label
var _tab_buttons: Array[Button] = []
var _content: RichTextLabel


func init(sim_engine: RefCounted) -> void:
	_sim_engine = sim_engine
	mouse_filter = Control.MOUSE_FILTER_STOP
	clip_contents = true
	focus_mode = Control.FOCUS_ALL


func _ready() -> void:
	_build_ui()
	if _band_id >= 0:
		_reload_data()


func set_band_id(band_id: int) -> void:
	_band_id = band_id
	_current_tab = 0
	if is_inside_tree():
		_update_tab_styles()
		_reload_data()


func refresh_locale() -> void:
	_refresh_tab_titles()
	_refresh_header()
	_refresh_content()


func get_band_name() -> String:
	return str(_detail.get("name", ""))


func _build_ui() -> void:
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.05, 0.07, 0.10, 0.95)
	style.border_color = Color(0.12, 0.16, 0.22, 0.80)
	style.border_width_left = 1
	style.border_width_top = 1
	style.corner_radius_top_left = 6
	style.corner_radius_bottom_left = 6
	add_theme_stylebox_override("panel", style)

	var root := VBoxContainer.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
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

	var header_box := VBoxContainer.new()
	header_box.add_theme_constant_override("separation", 2)
	header_bg.add_child(header_box)

	_header_name = Label.new()
	_header_name.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_title"))
	_header_name.add_theme_color_override("font_color", Color(0.78, 0.56, 0.19))
	header_box.add_child(_header_name)

	_header_meta = Label.new()
	_header_meta.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_header_meta.add_theme_font_size_override("font_size", GameConfig.get_font_size("panel_small"))
	_header_meta.add_theme_color_override("font_color", Color(0.31, 0.41, 0.47))
	header_box.add_child(_header_meta)

	var tab_bar := HBoxContainer.new()
	tab_bar.add_theme_constant_override("separation", 0)
	root.add_child(tab_bar)
	for index: int in range(TAB_KEYS.size()):
		var button := Button.new()
		button.focus_mode = Control.FOCUS_NONE
		button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		button.custom_minimum_size.y = 30.0
		button.add_theme_font_size_override("font_size", 9)
		var tab_index: int = index
		button.pressed.connect(func() -> void:
			_switch_tab(tab_index)
		)
		_tab_buttons.append(button)
		tab_bar.add_child(button)

	var scroll := ScrollContainer.new()
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	root.add_child(scroll)

	_content = RichTextLabel.new()
	_content.bbcode_enabled = true
	_content.fit_content = true
	_content.scroll_active = false
	_content.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_content.add_theme_font_size_override("normal_font_size", 10)
	_content.add_theme_color_override("default_color", Color(0.66, 0.73, 0.78))
	_content.meta_clicked.connect(_on_meta_clicked)
	scroll.add_child(_content)

	_refresh_tab_titles()
	_update_tab_styles()


func _refresh_tab_titles() -> void:
	for index: int in range(_tab_buttons.size()):
		_tab_buttons[index].text = Locale.ltr(TAB_KEYS[index])


func _switch_tab(index: int) -> void:
	_current_tab = index
	_update_tab_styles()
	_refresh_content()


func _update_tab_styles() -> void:
	for index: int in range(_tab_buttons.size()):
		var button: Button = _tab_buttons[index]
		var style := StyleBoxFlat.new()
		style.bg_color = Color(0.10, 0.14, 0.20, 1.0) if index == _current_tab else Color(0.06, 0.08, 0.12, 1.0)
		style.border_color = Color(0.80, 0.56, 0.18, 1.0) if index == _current_tab else Color(0.16, 0.22, 0.30, 1.0)
		style.border_width_bottom = 2 if index == _current_tab else 1
		style.content_margin_left = 6
		style.content_margin_right = 6
		style.content_margin_top = 4
		style.content_margin_bottom = 4
		button.add_theme_stylebox_override("normal", style)
		button.add_theme_stylebox_override("hover", style)
		button.add_theme_stylebox_override("pressed", style)
		button.add_theme_stylebox_override("focus", style)
		var font_color: Color = Color(0.95, 0.88, 0.52) if index == _current_tab else Color(0.45, 0.50, 0.58)
		button.add_theme_color_override("font_color", font_color)
		button.add_theme_color_override("font_hover_color", font_color)
		button.add_theme_color_override("font_pressed_color", font_color)
		button.add_theme_color_override("font_focus_color", font_color)


func _reload_data() -> void:
	if _sim_engine == null or _band_id < 0:
		_detail = {}
		_refresh_header()
		_refresh_content()
		return
	_detail = _sim_engine.get_band_detail(_band_id) if _sim_engine.has_method("get_band_detail") else {}
	_refresh_header()
	_refresh_content()


func _refresh_header() -> void:
	if _header_name == null or _header_meta == null:
		return
	if _detail.is_empty():
		_header_name.text = Locale.ltr("UI_UNKNOWN")
		_header_meta.text = ""
		return
	var status_key: String = "UI_BAND_PROMOTED" if bool(_detail.get("is_promoted", false)) else "UI_BAND_PROVISIONAL"
	var member_count: int = int(_detail.get("member_count", 0))
	var settlement_name: String = str(_detail.get("settlement_name", ""))
	var meta_parts: PackedStringArray = PackedStringArray()
	meta_parts.append("[%s]" % Locale.ltr(status_key))
	meta_parts.append("%d%s" % [member_count, Locale.ltr("UI_PEOPLE_SUFFIX")])
	if not settlement_name.is_empty():
		meta_parts.append(settlement_name)
	_header_name.text = str(_detail.get("name", Locale.ltr("UI_UNKNOWN")))
	_header_meta.text = " · ".join(meta_parts)


func _refresh_content() -> void:
	if _content == null:
		return
	_content.clear()
	if _detail.is_empty():
		_content.append_text("[color=#354050]%s[/color]" % Locale.ltr("UI_UNKNOWN"))
		return
	match _current_tab:
		0:
			_content.append_text(_format_overview())
		1:
			_content.append_text(_format_members())
		2:
			_content.append_text(_format_events())


func _format_overview() -> String:
	var lines: PackedStringArray = PackedStringArray()
	var leader_raw: Variant = _detail.get("leader", {})
	if leader_raw is Dictionary and not (leader_raw as Dictionary).is_empty():
		var leader: Dictionary = leader_raw
		var leader_id: int = int(leader.get("id", -1))
		var leader_name: String = str(leader.get("name", ""))
		if leader_id >= 0 and not leader_name.is_empty():
			lines.append("[b]%s[/b]: [url=entity:%d]%s[/url] ★" % [Locale.ltr("UI_LEADER"), leader_id, leader_name])
			lines.append("")
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_OVERVIEW_ALERTS"))
	lines.append("  [color=green]✓ %s[/color]" % Locale.ltr("ALERT_ALL_GOOD"))
	lines.append("")
	lines.append("[b]%s[/b]" % Locale.ltr("PANEL_OVERVIEW_INFO"))
	lines.append("  %s: %d%s" % [Locale.ltr("UI_STATS_POP"), int(_detail.get("member_count", 0)), Locale.ltr("UI_PEOPLE_SUFFIX")])
	lines.append("  %s: %s" % [Locale.ltr("UI_STATUS"), Locale.ltr("UI_BAND_PROMOTED") if bool(_detail.get("is_promoted", false)) else Locale.ltr("UI_BAND_PROVISIONAL")])
	var settlement_name: String = str(_detail.get("settlement_name", ""))
	var settlement_id: int = int(_detail.get("settlement_id", -1))
	if settlement_id >= 0 and not settlement_name.is_empty():
		lines.append("  %s: [url=sett:%d]%s[/url]" % [Locale.ltr("UI_STATS_SETTLEMENTS"), settlement_id, settlement_name])
	return "\n".join(lines)


func _format_members() -> String:
	var lines: PackedStringArray = PackedStringArray()
	var members: Array = _detail.get("members", [])
	lines.append("[b]%s (%d%s)[/b]" % [Locale.ltr("UI_BAND_TAB_MEMBERS"), members.size(), Locale.ltr("UI_PEOPLE_SUFFIX")])
	lines.append("")
	if members.is_empty():
		lines.append("[color=#354050]%s[/color]" % Locale.ltr("UI_BAND_NO_MEMBERS"))
		return "\n".join(lines)
	for member_raw: Variant in members:
		if not (member_raw is Dictionary):
			continue
		var member: Dictionary = member_raw
		var member_id: int = int(member.get("id", -1))
		var member_name: String = str(member.get("name", Locale.ltr("UI_UNKNOWN")))
		var leader_prefix: String = "[color=#c89828]★[/color] " if bool(member.get("is_leader", false)) else "  "
		lines.append("%s[url=entity:%d]%s[/url]" % [leader_prefix, member_id, member_name])
		lines.append(
			"    [color=#486070]%d%s · %s · %s[/color]" % [
				int(round(float(member.get("age_years", 0.0)))),
				Locale.ltr("UI_AGE_UNIT"),
				_localized_sex_text(str(member.get("sex", ""))),
				_localized_action_text(str(member.get("current_action", ""))),
			]
		)
		lines.append("")
	return "\n".join(lines)


func _format_events() -> String:
	var lines: PackedStringArray = PackedStringArray()
	var events: Array = _detail.get("events", [])
	lines.append("[b]%s[/b]" % Locale.ltr("UI_BAND_TAB_EVENTS"))
	lines.append("")
	if events.is_empty():
		lines.append("[color=#354050]%s[/color]" % Locale.ltr("UI_HISTORY_EMPTY"))
		return "\n".join(lines)
	for event_raw: Variant in events:
		if not (event_raw is Dictionary):
			continue
		var event: Dictionary = event_raw
		var tick: int = int(event.get("tick", 0))
		var text_key: String = str(event.get("text_key", event.get("text", "")))
		var event_text: String = Locale.ltr(text_key) if Locale.has_key(text_key) else text_key
		lines.append("[color=#506878]%s[/color]  %s" % [GameCalendar.format_short_date(tick), event_text])
		lines.append("")
	return "\n".join(lines)


func _on_meta_clicked(meta: Variant) -> void:
	var target: String = str(meta)
	if target.begins_with("entity:"):
		var entity_id: int = int(target.substr(7))
		if entity_id >= 0:
			SimulationBus.entity_selected.emit(entity_id)
	elif target.begins_with("sett:"):
		var settlement_id: int = int(target.substr(5))
		if settlement_id >= 0:
			SimulationBus.settlement_panel_requested.emit(settlement_id)


func _localized_sex_text(sex: String) -> String:
	match sex:
		"male":
			return Locale.ltr("GENDER_M")
		"female":
			return Locale.ltr("GENDER_F")
		_:
			return Locale.ltr("UI_UNKNOWN")


func _localized_action_text(action_raw: String) -> String:
	if action_raw.is_empty():
		return Locale.ltr("UI_UNKNOWN")
	var status_key: String = "STATUS_" + _camel_to_upper_snake(action_raw)
	if Locale.has_key(status_key):
		return Locale.ltr(status_key)
	return action_raw


func _camel_to_upper_snake(value: String) -> String:
	var chars: PackedStringArray = PackedStringArray()
	for index: int in range(value.length()):
		var letter: String = value.substr(index, 1)
		if index > 0 and letter == letter.to_upper() and letter != letter.to_lower():
			chars.append("_")
		chars.append(letter.to_upper())
	return "".join(chars)
