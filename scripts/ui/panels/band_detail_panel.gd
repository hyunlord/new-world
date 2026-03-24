extends PanelContainer

const GameCalendar = preload("res://scripts/core/simulation/game_calendar.gd")
const TAB_KEYS: Array[String] = ["UI_TAB_OVERVIEW", "UI_BAND_TAB_MEMBERS", "UI_BAND_TAB_EVENTS"]

const COLOR_LABEL: Color = Color(0.35, 0.42, 0.50)
const COLOR_VALUE: Color = Color(0.55, 0.62, 0.70)
const COLOR_WHITE: Color = Color(0.90, 0.92, 0.95)
const COLOR_ACCENT: Color = Color(0.80, 0.56, 0.18)
const COLOR_LINK: Color = Color(0.55, 0.70, 0.88)
const COLOR_SECTION: Color = Color(0.45, 0.55, 0.65)

var _sim_engine: RefCounted
var _band_id: int = -1
var _detail: Dictionary = {}
var _current_tab: int = 0
var _refresh_timer: float = 0.0

var _header_name: Label
var _header_meta: Label
var _tab_buttons: Array[Button] = []

# Scroll + tab panels
var _scroll: ScrollContainer
var _tab_content: VBoxContainer

var _overview_panel: VBoxContainer
var _members_panel: VBoxContainer
var _events_panel: VBoxContainer

# Overview tab nodes
var _leader_label: Label
var _status_label: Label
var _pop_stats_label: Label
var _gender_label: Label
var _avg_stress_label: Label
var _settlement_btn: Button

# Members tab nodes
var _members_header: Label
var _members_container: VBoxContainer

# Events tab nodes
var _events_header: Label
var _events_container: VBoxContainer


func init(sim_engine: RefCounted) -> void:
	_sim_engine = sim_engine
	mouse_filter = Control.MOUSE_FILTER_STOP
	clip_contents = true
	focus_mode = Control.FOCUS_ALL


func _ready() -> void:
	_build_ui()
	if _band_id >= 0:
		_reload_data()


func _process(delta: float) -> void:
	if not visible or _band_id < 0:
		return
	_refresh_timer += delta
	if _refresh_timer >= 0.5:
		_refresh_timer = 0.0
		_reload_data()


func set_band_id(band_id: int) -> void:
	_band_id = band_id
	_current_tab = 0
	if is_inside_tree():
		_update_tab_styles()
		_reload_data()


func refresh_locale() -> void:
	_refresh_tab_titles()
	_refresh_all()


func get_band_name() -> String:
	return str(_detail.get("name", ""))


# ---------------------------------------------------------------------------
# UI construction
# ---------------------------------------------------------------------------

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

	# --- Header ---
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

	# --- Tab bar ---
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

	# --- Scroll + tab content ---
	_scroll = ScrollContainer.new()
	_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	root.add_child(_scroll)

	_tab_content = VBoxContainer.new()
	_tab_content.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_tab_content.add_theme_constant_override("separation", 4)
	_scroll.add_child(_tab_content)

	_build_overview_tab()
	_build_members_tab()
	_build_events_tab()

	_refresh_tab_titles()
	_update_tab_styles()
	_show_active_tab()


func _build_overview_tab() -> void:
	_overview_panel = VBoxContainer.new()
	_overview_panel.add_theme_constant_override("separation", 4)
	_tab_content.add_child(_overview_panel)

	var margin := MarginContainer.new()
	margin.add_theme_constant_override("margin_left", 8)
	margin.add_theme_constant_override("margin_right", 8)
	margin.add_theme_constant_override("margin_top", 6)
	margin.add_theme_constant_override("margin_bottom", 4)
	_overview_panel.add_child(margin)

	var inner := VBoxContainer.new()
	inner.add_theme_constant_override("separation", 4)
	margin.add_child(inner)

	_add_section_title(inner, "UI_LEADER")
	_leader_label = _add_info_label(inner)

	_add_section_spacer(inner)
	_add_section_title(inner, "UI_STATUS")
	_status_label = _add_info_label(inner)

	_settlement_btn = Button.new()
	_settlement_btn.flat = true
	_settlement_btn.focus_mode = Control.FOCUS_NONE
	_settlement_btn.add_theme_font_size_override("font_size", 10)
	_settlement_btn.add_theme_color_override("font_color", COLOR_LINK)
	_settlement_btn.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	_settlement_btn.visible = false
	inner.add_child(_settlement_btn)

	_add_section_spacer(inner)
	_add_section_title(inner, "UI_DEMOGRAPHICS")
	_pop_stats_label = _add_info_label(inner)
	_gender_label = _add_info_label(inner)

	_add_section_spacer(inner)
	_add_section_title(inner, "UI_AVG_STRESS")
	_avg_stress_label = _add_info_label(inner)


func _build_members_tab() -> void:
	_members_panel = VBoxContainer.new()
	_members_panel.add_theme_constant_override("separation", 2)
	_tab_content.add_child(_members_panel)

	var margin := MarginContainer.new()
	margin.add_theme_constant_override("margin_left", 8)
	margin.add_theme_constant_override("margin_right", 8)
	margin.add_theme_constant_override("margin_top", 6)
	margin.add_theme_constant_override("margin_bottom", 4)
	_members_panel.add_child(margin)

	var inner := VBoxContainer.new()
	inner.add_theme_constant_override("separation", 2)
	margin.add_child(inner)

	_members_header = Label.new()
	_members_header.add_theme_font_size_override("font_size", 11)
	_members_header.add_theme_color_override("font_color", COLOR_WHITE)
	inner.add_child(_members_header)

	_members_container = VBoxContainer.new()
	_members_container.add_theme_constant_override("separation", 2)
	inner.add_child(_members_container)


func _build_events_tab() -> void:
	_events_panel = VBoxContainer.new()
	_events_panel.add_theme_constant_override("separation", 2)
	_tab_content.add_child(_events_panel)

	var margin := MarginContainer.new()
	margin.add_theme_constant_override("margin_left", 8)
	margin.add_theme_constant_override("margin_right", 8)
	margin.add_theme_constant_override("margin_top", 6)
	margin.add_theme_constant_override("margin_bottom", 4)
	_events_panel.add_child(margin)

	var inner := VBoxContainer.new()
	inner.add_theme_constant_override("separation", 4)
	margin.add_child(inner)

	_events_header = Label.new()
	_events_header.add_theme_font_size_override("font_size", 11)
	_events_header.add_theme_color_override("font_color", COLOR_WHITE)
	inner.add_child(_events_header)

	_events_container = VBoxContainer.new()
	_events_container.add_theme_constant_override("separation", 2)
	inner.add_child(_events_container)


# ---------------------------------------------------------------------------
# Tab management
# ---------------------------------------------------------------------------

func _refresh_tab_titles() -> void:
	for index: int in range(_tab_buttons.size()):
		_tab_buttons[index].text = Locale.ltr(TAB_KEYS[index])


func _switch_tab(index: int) -> void:
	_current_tab = index
	_update_tab_styles()
	_show_active_tab()
	match _current_tab:
		0: _refresh_overview()
		1: _refresh_members()
		2: _refresh_events()


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


func _show_active_tab() -> void:
	if _overview_panel:
		_overview_panel.visible = (_current_tab == 0)
	if _members_panel:
		_members_panel.visible = (_current_tab == 1)
	if _events_panel:
		_events_panel.visible = (_current_tab == 2)


# ---------------------------------------------------------------------------
# Data loading
# ---------------------------------------------------------------------------

func _reload_data() -> void:
	if _sim_engine == null or _band_id < 0:
		_detail = {}
		_refresh_header()
		return
	_detail = _sim_engine.get_band_detail(_band_id) if _sim_engine.has_method("get_band_detail") else {}
	_refresh_all()


# ---------------------------------------------------------------------------
# Refresh
# ---------------------------------------------------------------------------

func _refresh_all() -> void:
	_refresh_header()
	_show_active_tab()
	match _current_tab:
		0: _refresh_overview()
		1: _refresh_members()
		2: _refresh_events()


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
	meta_parts.append("%d" % member_count)
	if not settlement_name.is_empty():
		meta_parts.append(settlement_name)
	_header_name.text = str(_detail.get("name", Locale.ltr("UI_UNKNOWN")))
	_header_meta.text = " · ".join(meta_parts)


func _refresh_overview() -> void:
	if _overview_panel == null or _leader_label == null:
		return

	# --- Leader ---
	var leader_raw: Variant = _detail.get("leader", {})
	if leader_raw is Dictionary and not (leader_raw as Dictionary).is_empty():
		var ld: Dictionary = leader_raw as Dictionary
		var leader_name: String = str(ld.get("name", ""))
		var leader_id: int = int(ld.get("id", -1))
		_leader_label.text = "★ %s" % leader_name
		_leader_label.add_theme_color_override("font_color", COLOR_LINK)
		_leader_label.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
		_leader_label.mouse_filter = Control.MOUSE_FILTER_STOP
		for conn: Dictionary in _leader_label.gui_input.get_connections():
			_leader_label.gui_input.disconnect(conn["callable"])
		if leader_id >= 0:
			var captured: int = leader_id
			_leader_label.gui_input.connect(func(event: InputEvent) -> void:
				if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
					SimulationBus.entity_selected.emit(captured)
					SimulationBus.ui_notification.emit("focus_entity_%d" % captured, "command")
			)
	else:
		_leader_label.text = Locale.ltr("UI_NO_LEADER")
		_leader_label.add_theme_color_override("font_color", COLOR_LABEL)
		_leader_label.mouse_default_cursor_shape = Control.CURSOR_ARROW
		_leader_label.mouse_filter = Control.MOUSE_FILTER_IGNORE

	# --- Status ---
	var is_promoted: bool = bool(_detail.get("is_promoted", false))
	_status_label.text = Locale.ltr("UI_BAND_PROMOTED") if is_promoted else Locale.ltr("UI_BAND_PROVISIONAL")

	# --- Settlement button ---
	var settlement_id: int = int(_detail.get("settlement_id", -1))
	var settlement_name: String = str(_detail.get("settlement_name", ""))
	if settlement_id >= 0 and not settlement_name.is_empty():
		_settlement_btn.text = "🏘 %s" % settlement_name
		_settlement_btn.visible = true
		for conn: Dictionary in _settlement_btn.pressed.get_connections():
			_settlement_btn.pressed.disconnect(conn["callable"])
		var captured_sid: int = settlement_id
		_settlement_btn.pressed.connect(func() -> void:
			SimulationBus.ui_notification.emit("open_settlement_%d" % captured_sid, "command")
		)
	else:
		_settlement_btn.visible = false

	# --- Demographics ---
	var members: Array = _detail.get("members", [])
	var adults: int = 0
	var teens: int = 0
	var children: int = 0
	var elders: int = 0
	var male: int = 0
	var female: int = 0
	var total_stress: float = 0.0

	for m_raw: Variant in members:
		if not (m_raw is Dictionary):
			continue
		var md: Dictionary = m_raw as Dictionary
		var age: float = _safe_float(md, "age_years", 0.0)
		var sex: String = str(md.get("sex", ""))
		if sex == "male":
			male += 1
		else:
			female += 1
		if age < 13.0:
			children += 1
		elif age < 18.0:
			teens += 1
		elif age >= 55.0:
			elders += 1
		else:
			adults += 1
		var mid: int = int(md.get("id", -1))
		if mid >= 0 and _sim_engine != null and _sim_engine.has_method("get_entity_detail"):
			var ed: Dictionary = _sim_engine.get_entity_detail(mid)
			var s_raw: Variant = ed.get("stress_level", 0.0)
			total_stress += float(s_raw) if (s_raw is float or s_raw is int) else 0.0

	_pop_stats_label.text = "%s %d · %s %d · %s %d · %s %d" % [
		Locale.ltr("UI_ADULTS"), adults,
		Locale.ltr("UI_TEENS"), teens,
		Locale.ltr("UI_CHILDREN"), children,
		Locale.ltr("UI_ELDERS"), elders,
	]
	_gender_label.text = "♂ %d / ♀ %d" % [male, female]

	var avg_stress: float = total_stress / float(maxi(members.size(), 1))
	_avg_stress_label.text = "%d%%" % int(avg_stress * 100.0)


func _refresh_members() -> void:
	if _members_panel == null or _members_container == null:
		return
	for child in _members_container.get_children():
		child.queue_free()

	var members: Array = _detail.get("members", [])
	_members_header.text = "%s (%d)" % [Locale.ltr("UI_BAND_TAB_MEMBERS"), members.size()]

	if members.is_empty():
		var empty := Label.new()
		empty.text = Locale.ltr("UI_BAND_NO_MEMBERS")
		empty.add_theme_font_size_override("font_size", 10)
		empty.add_theme_color_override("font_color", COLOR_LABEL)
		_members_container.add_child(empty)
		return

	# Sort: leader first, then age descending
	var sorted: Array = members.duplicate()
	sorted.sort_custom(func(a: Variant, b: Variant) -> bool:
		var ad: Dictionary = a as Dictionary if a is Dictionary else {}
		var bd: Dictionary = b as Dictionary if b is Dictionary else {}
		var a_leader: bool = bool(ad.get("is_leader", false))
		var b_leader: bool = bool(bd.get("is_leader", false))
		if a_leader != b_leader:
			return a_leader
		return _safe_float(ad, "age_years", 0.0) > _safe_float(bd, "age_years", 0.0)
	)

	for m_raw: Variant in sorted:
		if not (m_raw is Dictionary):
			continue
		var m: Dictionary = m_raw as Dictionary
		var m_id: int = int(m.get("id", -1))
		var m_name: String = str(m.get("name", "?"))
		var m_age: float = _safe_float(m, "age_years", 0.0)
		var m_sex: String = str(m.get("sex", ""))
		var m_action: String = str(m.get("current_action", ""))
		var m_job: String = str(m.get("job", ""))
		var is_leader: bool = bool(m.get("is_leader", false))
		var gender_icon: String = "♂" if m_sex == "male" else "♀"

		var row := VBoxContainer.new()
		row.add_theme_constant_override("separation", 1)

		# Name / job line
		var name_label := Label.new()
		var prefix: String = "★ " if is_leader else "  "
		var job_display: String = ""
		if not m_job.is_empty() and m_job != "none":
			var jk: String = "OCCUPATION_" + m_job.to_upper()
			job_display = Locale.ltr(jk)
			if job_display == jk:
				job_display = m_job.capitalize()
		var job_text: String = job_display if not job_display.is_empty() else Locale.ltr("UI_NO_JOB")
		name_label.text = "%s%s — %s · %d세 %s" % [prefix, m_name, job_text, int(m_age), gender_icon]
		name_label.add_theme_font_size_override("font_size", 10)
		name_label.add_theme_color_override("font_color", COLOR_ACCENT if is_leader else COLOR_VALUE)
		name_label.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
		name_label.mouse_filter = Control.MOUSE_FILTER_STOP
		if m_id >= 0:
			var captured: int = m_id
			name_label.gui_input.connect(func(event: InputEvent) -> void:
				if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
					SimulationBus.entity_selected.emit(captured)
					SimulationBus.ui_notification.emit("focus_entity_%d" % captured, "command")
			)
		row.add_child(name_label)

		# Action line
		var action_label := Label.new()
		action_label.text = "  🎯 %s" % _localized_action_text(m_action)
		action_label.add_theme_font_size_override("font_size", 9)
		action_label.add_theme_color_override("font_color", Color(0.40, 0.48, 0.56))
		row.add_child(action_label)

		_members_container.add_child(row)

		var sep := HSeparator.new()
		var sep_style := StyleBoxFlat.new()
		sep_style.bg_color = Color(0.12, 0.16, 0.22)
		sep.add_theme_stylebox_override("separator", sep_style)
		_members_container.add_child(sep)


func _refresh_events() -> void:
	if _events_panel == null or _events_container == null:
		return
	for child in _events_container.get_children():
		child.queue_free()

	var events: Array = _detail.get("events", [])
	_events_header.text = "%s (%d)" % [Locale.ltr("UI_BAND_TAB_EVENTS"), events.size()]

	if events.is_empty():
		var empty := Label.new()
		empty.text = Locale.ltr("UI_HISTORY_EMPTY")
		empty.add_theme_font_size_override("font_size", 10)
		empty.add_theme_color_override("font_color", COLOR_LABEL)
		_events_container.add_child(empty)
		return

	# Newest first
	var sorted_events: Array = events.duplicate()
	sorted_events.sort_custom(func(a: Variant, b: Variant) -> bool:
		var at: int = int((a as Dictionary).get("tick", 0)) if a is Dictionary else 0
		var bt: int = int((b as Dictionary).get("tick", 0)) if b is Dictionary else 0
		return at > bt
	)

	for ev_raw: Variant in sorted_events:
		if not (ev_raw is Dictionary):
			continue
		var ev: Dictionary = ev_raw as Dictionary
		var tick: int = int(ev.get("tick", 0))
		var text_key: String = str(ev.get("text_key", ev.get("text", "")))
		var display_text: String = Locale.ltr(text_key) if Locale.has_key(text_key) else text_key

		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 8)

		var date_label := Label.new()
		date_label.text = GameCalendar.format_short_date(tick)
		date_label.custom_minimum_size.x = 80.0
		date_label.add_theme_font_size_override("font_size", 9)
		date_label.add_theme_color_override("font_color", COLOR_LABEL)
		row.add_child(date_label)

		var text_label := Label.new()
		text_label.text = display_text
		text_label.add_theme_font_size_override("font_size", 10)
		text_label.add_theme_color_override("font_color", COLOR_VALUE)
		text_label.autowrap_mode = TextServer.AUTOWRAP_WORD
		text_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		row.add_child(text_label)

		_events_container.add_child(row)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

func _add_section_title(parent: Control, key: String) -> void:
	var lbl := Label.new()
	lbl.text = Locale.ltr(key)
	lbl.add_theme_font_size_override("font_size", 11)
	lbl.add_theme_color_override("font_color", COLOR_SECTION)
	parent.add_child(lbl)


func _add_info_label(parent: Control) -> Label:
	var lbl := Label.new()
	lbl.add_theme_font_size_override("font_size", 10)
	lbl.add_theme_color_override("font_color", COLOR_VALUE)
	lbl.autowrap_mode = TextServer.AUTOWRAP_WORD
	parent.add_child(lbl)
	return lbl


func _add_section_spacer(parent: Control) -> void:
	var sp := Control.new()
	sp.custom_minimum_size.y = 6.0
	parent.add_child(sp)


func _safe_float(dict: Dictionary, key: String, default_value: float) -> float:
	var raw: Variant = dict.get(key, default_value)
	if raw is float or raw is int:
		return float(raw)
	return default_value


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
