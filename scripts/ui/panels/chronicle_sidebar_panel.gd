extends PanelContainer

const GameCalendar = preload("res://scripts/core/simulation/game_calendar.gd")

var _sim_engine: RefCounted
@warning_ignore("unused_private_class_variable")
var _entity_manager: RefCounted  # Set by hud.gd init
var _refresh_timer: float = 0.0

var _scroll: ScrollContainer
var _content: VBoxContainer
var _title_label: Label
var _count_label: Label
var _filter_bar: HBoxContainer
var _events_container: VBoxContainer

var _current_filter: String = "all"
var _cached_event_count: int = -1

const COLOR_BG: Color = Color(0.05, 0.07, 0.10, 0.92)
const COLOR_SECTION: Color = Color(0.16, 0.22, 0.28)
static var FILTER_KEYS: Array[String] = ["all", "food", "danger", "shelter", "social"]
static var FILTER_LOCALE: Array[String] = ["UI_FILTER_ALL", "UI_FILTER_FOOD", "UI_FILTER_DANGER", "UI_FILTER_SHELTER", "UI_FILTER_SOCIAL"]
## Maps UI filter keys to the Rust category_id values they should match.
static var FILTER_CATEGORY_MAP: Dictionary = {
	"food": ["food"],
	"danger": ["danger"],
	"shelter": ["warmth"],
	"social": ["social", "social_group"],
}


func init(sim_engine, entity_manager = null) -> void:
	if sim_engine is RefCounted:
		_sim_engine = sim_engine
	if entity_manager is RefCounted:
		_entity_manager = entity_manager


func _ready() -> void:
	_build_ui()


func _process(delta: float) -> void:
	if not visible:
		return
	_refresh_timer += delta
	if _refresh_timer >= 2.0:
		_refresh_timer = 0.0
		_refresh()


func force_redraw() -> void:
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

	var header := HBoxContainer.new()
	header.add_theme_constant_override("separation", 8)
	_content.add_child(header)

	_title_label = Label.new()
	_title_label.text = Locale.ltr("UI_CHRONICLE_TITLE")
	_title_label.add_theme_font_size_override("font_size", 12)
	_title_label.add_theme_color_override("font_color", Color.WHITE)
	header.add_child(_title_label)

	_count_label = Label.new()
	_count_label.add_theme_font_size_override("font_size", 10)
	_count_label.add_theme_color_override("font_color", Color(0.5, 0.58, 0.65))
	header.add_child(_count_label)

	_filter_bar = HBoxContainer.new()
	_filter_bar.add_theme_constant_override("separation", 4)
	_content.add_child(_filter_bar)
	for i in range(FILTER_KEYS.size()):
		var btn := Button.new()
		btn.text = Locale.ltr(FILTER_LOCALE[i])
		btn.add_theme_font_size_override("font_size", 9)
		btn.pressed.connect(_on_filter_pressed.bind(FILTER_KEYS[i]))
		_filter_bar.add_child(btn)

	_events_container = VBoxContainer.new()
	_events_container.add_theme_constant_override("separation", 6)
	_content.add_child(_events_container)


func _on_filter_pressed(filter: String) -> void:
	_current_filter = filter
	_cached_event_count = -1
	_refresh()


func _refresh() -> void:
	if _events_container == null:
		return

	var all_events: Array = []
	if _sim_engine != null and _sim_engine.has_method("get_chronicle_events"):
		all_events = _sim_engine.get_chronicle_events()

	var filtered: Array = all_events
	if _current_filter != "all":
		filtered = []
		var match_cats: Array = FILTER_CATEGORY_MAP.get(_current_filter, [_current_filter])
		for ev: Variant in all_events:
			if ev is Dictionary:
				var ev_cat: String = str((ev as Dictionary).get("category_id", (ev as Dictionary).get("category", ""))).to_lower()
				if ev_cat in match_cats:
					filtered.append(ev)

	_count_label.text = "%d %s" % [filtered.size(), Locale.ltr("UI_EVENTS")]

	# Cache check BEFORE destroying children
	if filtered.size() == _cached_event_count:
		return
	_cached_event_count = filtered.size()

	for child in _events_container.get_children():
		child.queue_free()

	if filtered.is_empty():
		var empty := Label.new()
		empty.text = Locale.ltr("UI_NO_EVENTS")
		empty.add_theme_font_size_override("font_size", 10)
		empty.add_theme_color_override("font_color", Color(0.40, 0.48, 0.55))
		_events_container.add_child(empty)
		return

	for ev_raw: Variant in filtered:
		if not (ev_raw is Dictionary):
			continue
		var ev: Dictionary = ev_raw
		var event_panel := PanelContainer.new()
		var ev_style := StyleBoxFlat.new()
		ev_style.bg_color = Color(0.08, 0.10, 0.14, 0.6)
		ev_style.corner_radius_top_left = 3
		ev_style.corner_radius_top_right = 3
		ev_style.corner_radius_bottom_left = 3
		ev_style.corner_radius_bottom_right = 3
		ev_style.content_margin_left = 6
		ev_style.content_margin_right = 6
		ev_style.content_margin_top = 3
		ev_style.content_margin_bottom = 3
		event_panel.add_theme_stylebox_override("panel", ev_style)
		_events_container.add_child(event_panel)

		var vbox := VBoxContainer.new()
		vbox.add_theme_constant_override("separation", 1)
		event_panel.add_child(vbox)

		var tick: int = int(ev.get("tick", ev.get("end_tick", 0)))
		var time_text: String = GameCalendar.format_short_date(tick) if tick > 0 else ""
		var time_label := Label.new()
		time_label.text = time_text
		time_label.add_theme_font_size_override("font_size", 9)
		time_label.add_theme_color_override("font_color", Color(0.45, 0.50, 0.58))
		vbox.add_child(time_label)

		var cat: String = str(ev.get("category_id", ev.get("category", "")))
		var headline_key: String = str(ev.get("headline_key", ev.get("title_key", "")))
		var headline_params_raw: Variant = ev.get("headline_params", {})
		var headline_params: Dictionary = headline_params_raw if headline_params_raw is Dictionary else {}
		var title_text: String = ""
		if not headline_key.is_empty():
			if not headline_params.is_empty():
				title_text = Locale.trf(headline_key, headline_params)
			elif Locale.has_key(headline_key):
				title_text = Locale.ltr(headline_key)
			else:
				title_text = headline_key
		if title_text.is_empty():
			var capsule_key_fb: String = str(ev.get("capsule_key", ev.get("description", "")))
			var capsule_params_fb_raw: Variant = ev.get("capsule_params", ev.get("l10n_params", {}))
			var capsule_params_fb: Dictionary = capsule_params_fb_raw if capsule_params_fb_raw is Dictionary else {}
			if not capsule_key_fb.is_empty():
				if not capsule_params_fb.is_empty():
					title_text = Locale.trf(capsule_key_fb, capsule_params_fb)
				elif Locale.has_key(capsule_key_fb):
					title_text = Locale.ltr(capsule_key_fb)
				else:
					title_text = capsule_key_fb
		var title_label := Label.new()
		var badge: String = _category_badge(cat)
		title_label.text = "%s %s" % [badge, title_text] if not badge.is_empty() else title_text
		title_label.add_theme_font_size_override("font_size", 10)
		title_label.add_theme_color_override("font_color", _category_color(cat))
		title_label.autowrap_mode = TextServer.AUTOWRAP_WORD
		vbox.add_child(title_label)

		var detail_capsule_key: String = str(ev.get("capsule_key", ""))
		if not detail_capsule_key.is_empty() and detail_capsule_key != headline_key:
			var detail_params_raw: Variant = ev.get("capsule_params", {})
			var detail_params: Dictionary = detail_params_raw if detail_params_raw is Dictionary else {}
			var detail_text: String = ""
			if not detail_params.is_empty():
				detail_text = Locale.trf(detail_capsule_key, detail_params)
			elif Locale.has_key(detail_capsule_key):
				detail_text = Locale.ltr(detail_capsule_key)
			if not detail_text.is_empty():
				var detail_label := Label.new()
				detail_label.text = detail_text
				detail_label.add_theme_font_size_override("font_size", 9)
				detail_label.add_theme_color_override("font_color", Color(0.55, 0.60, 0.68))
				detail_label.autowrap_mode = TextServer.AUTOWRAP_WORD
				vbox.add_child(detail_label)

		var actor_id: int = int(ev.get("actor_id", ev.get("entity_id", -1)))
		if actor_id >= 0:
			event_panel.mouse_filter = Control.MOUSE_FILTER_STOP
			event_panel.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
			var captured: int = actor_id
			event_panel.gui_input.connect(func(input_event: InputEvent) -> void:
				if input_event is InputEventMouseButton and input_event.pressed and input_event.button_index == MOUSE_BUTTON_LEFT:
					SimulationBus.entity_selected.emit(captured)
			)


func _category_badge(cat: String) -> String:
	match cat.to_lower():
		"danger": return "!"
		"food": return "🌾"
		"shelter": return "🏠"
		"social": return "G"
		_: return ""


func _category_color(cat: String) -> Color:
	match cat.to_lower():
		"danger": return Color(0.88, 0.30, 0.24)
		"food": return Color(0.35, 0.75, 0.40)
		"shelter": return Color(0.70, 0.58, 0.35)
		"social": return Color(0.45, 0.62, 0.82)
		_: return Color(0.75, 0.75, 0.75)
