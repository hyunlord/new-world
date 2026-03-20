extends PanelContainer

const GameCalendar = preload("res://scripts/core/simulation/game_calendar.gd")

var _ui_built: bool = false
var _refresh_timer: float = 0.0
var _cached_event_count: int = -1

var _scroll: ScrollContainer
var _content: VBoxContainer
var _title_label: Label
var _events_container: VBoxContainer

const COLOR_BG: Color = Color(0.05, 0.07, 0.10, 0.92)
const COLOR_LABEL: Color = Color(0.50, 0.58, 0.65)
const COLOR_VALUE: Color = Color(0.85, 0.82, 0.75)


func _ensure_ui() -> void:
	if _ui_built:
		return
	_build_ui()
	_ui_built = true


func _ready() -> void:
	_ensure_ui()


func _process(delta: float) -> void:
	if not visible:
		return
	_ensure_ui()
	_refresh_timer += delta
	if _refresh_timer >= 3.0:
		_refresh_timer = 0.0
		_refresh()


func _build_ui() -> void:
	var style := StyleBoxFlat.new()
	style.bg_color = COLOR_BG
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

	_title_label = Label.new()
	_title_label.text = Locale.ltr("UI_HISTORY_TITLE")
	_title_label.add_theme_font_size_override("font_size", 12)
	_title_label.add_theme_color_override("font_color", Color.WHITE)
	_content.add_child(_title_label)

	_events_container = VBoxContainer.new()
	_events_container.add_theme_constant_override("separation", 4)
	_content.add_child(_events_container)


func _refresh() -> void:
	if _events_container == null:
		return

	var response: Dictionary = SimBridge.runtime_get_chronicle_feed(200)
	var events_raw: Variant = response.get("items", [])
	var events: Array = events_raw if events_raw is Array else []

	# Sort by tick descending (newest first)
	events.sort_custom(func(a: Variant, b: Variant) -> bool:
		var tick_a: int = int(a.get("tick", a.get("end_tick", 0))) if a is Dictionary else 0
		var tick_b: int = int(b.get("tick", b.get("end_tick", 0))) if b is Dictionary else 0
		return tick_a > tick_b
	)

	# Cache check BEFORE destroying children
	if events.size() == _cached_event_count:
		return
	_cached_event_count = events.size()

	for child in _events_container.get_children():
		child.queue_free()

	if events.is_empty():
		var empty := Label.new()
		empty.text = Locale.ltr("UI_HISTORY_EMPTY")
		empty.add_theme_font_size_override("font_size", 10)
		empty.add_theme_color_override("font_color", COLOR_LABEL)
		_events_container.add_child(empty)
		return

	for event_raw: Variant in events:
		if not (event_raw is Dictionary):
			continue
		var event: Dictionary = event_raw
		var tick: int = int(event.get("tick", event.get("end_tick", 0)))
		var date: String = GameCalendar.format_short_date(tick)
		var desc: String = _event_text(event)

		var card := PanelContainer.new()
		var card_style := StyleBoxFlat.new()
		card_style.bg_color = Color(0.08, 0.10, 0.14, 0.5)
		card_style.corner_radius_top_left = 3
		card_style.corner_radius_top_right = 3
		card_style.corner_radius_bottom_left = 3
		card_style.corner_radius_bottom_right = 3
		card_style.content_margin_left = 6
		card_style.content_margin_right = 6
		card_style.content_margin_top = 3
		card_style.content_margin_bottom = 3
		card.add_theme_stylebox_override("panel", card_style)
		_events_container.add_child(card)

		var vbox := VBoxContainer.new()
		vbox.add_theme_constant_override("separation", 1)
		card.add_child(vbox)

		var time_label := Label.new()
		time_label.text = date
		time_label.add_theme_font_size_override("font_size", 9)
		time_label.add_theme_color_override("font_color", Color(0.40, 0.46, 0.55))
		vbox.add_child(time_label)

		# Make card clickable if event has a subject
		var subjects: Variant = event.get("primary_subjects", [])
		var first_subject_id: int = -1
		if subjects is Array and not subjects.is_empty():
			var first: Variant = subjects[0]
			if first is Dictionary:
				first_subject_id = int(first.get("entity_id", first.get("id", -1)))
		var settlement_id: int = int(event.get("settlement_id", -1))

		var msg_label := Label.new()
		msg_label.text = desc
		msg_label.add_theme_font_size_override("font_size", 10)
		msg_label.autowrap_mode = TextServer.AUTOWRAP_WORD
		vbox.add_child(msg_label)

		if first_subject_id >= 0:
			card.mouse_filter = Control.MOUSE_FILTER_STOP
			card.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
			var captured_id: int = first_subject_id
			card.gui_input.connect(func(ev: InputEvent) -> void:
				if ev is InputEventMouseButton and ev.pressed and ev.button_index == MOUSE_BUTTON_LEFT:
					SimulationBus.entity_selected.emit(captured_id)
					SimulationBus.ui_notification.emit("focus_entity_%d" % captured_id, "command")
			)
			msg_label.add_theme_color_override("font_color", Color(0.7, 0.8, 1.0))
		elif settlement_id >= 0:
			card.mouse_filter = Control.MOUSE_FILTER_STOP
			card.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
			var captured_sett_id: int = settlement_id
			card.gui_input.connect(func(ev: InputEvent) -> void:
				if ev is InputEventMouseButton and ev.pressed and ev.button_index == MOUSE_BUTTON_LEFT:
					SimulationBus.settlement_panel_requested.emit(captured_sett_id)
			)
			msg_label.add_theme_color_override("font_color", Color(0.7, 0.8, 1.0))
		else:
			msg_label.add_theme_color_override("font_color", COLOR_VALUE)


func _event_text(event: Dictionary) -> String:
	var capsule: String = _localized_text(event, "capsule_key", "capsule_params", "description")
	if not capsule.is_empty():
		return capsule
	var headline: String = _localized_text(event, "headline_key", "headline_params", "description")
	if not headline.is_empty():
		return headline
	return str(event.get("description", Locale.ltr("UI_HISTORY_EMPTY")))


func _localized_text(event: Dictionary, key_field: String, params_field: String, fallback_field: String) -> String:
	var locale_key: String = str(event.get(key_field, ""))
	var params_raw: Variant = event.get(params_field, {})
	var params: Dictionary = params_raw if params_raw is Dictionary else {}
	if not locale_key.is_empty():
		return Locale.trf(locale_key, params)
	var fallback: String = str(event.get(fallback_field, ""))
	if fallback.is_empty():
		return ""
	return Locale.ltr(fallback) if Locale.has_key(fallback) else fallback
