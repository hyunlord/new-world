extends Control

const GameCalendar = preload("res://scripts/core/simulation/game_calendar.gd")

var _entity_manager: RefCounted

## Filter state
var _filter_type: String = ""  # "" = all
const RUNTIME_FILTER_OPTIONS: Array = ["", "food", "danger", "warmth", "social"]
const RUNTIME_FILTER_LABEL_KEYS: Array = [
	"UI_FILTER_ALL",
	"UI_FILTER_FOOD",
	"UI_FILTER_DANGER",
	"UI_FILTER_SHELTER",
	"UI_FILTER_SOCIAL",
]
var _filter_index: int = 0

## Scroll state
var _scroll_offset: float = 0.0
var _content_height: float = 0.0

## Scrollbar drag state
var _scrollbar_dragging: bool = false
var _scrollbar_rect: Rect2 = Rect2()

## Clickable regions
var _click_regions: Array = []  # [{rect: Rect2, entity_id: int}]
var _entry_click_regions: Array = []  # [{rect: Rect2, entry_id: int}]
var _headline_cache: PackedStringArray = PackedStringArray()
var _capsule_cache: PackedStringArray = PackedStringArray()
var _text_cache_signature: String = ""
var _legacy_fallback_logged: bool = false
var _feed_snapshot_revision: int = -1
var _selected_entry_id: int = -1
var _selected_entry_snapshot_revision: int = -1
var _selected_entry_detail: Dictionary = {}

## Event type icons and colors
const EVENT_STYLES: Dictionary = {
	"birth": {"icon": "B", "color": Color(0.3, 0.9, 0.3)},
	"death": {"icon": "D", "color": Color(0.9, 0.3, 0.3)},
	"marriage": {"icon": "C", "color": Color(0.9, 0.4, 0.6)},
	"settlement_founded": {"icon": "S", "color": Color(0.9, 0.7, 0.2)},
	"population_milestone": {"icon": "P", "color": Color(0.3, 0.8, 0.9)},
	"famine": {"icon": "!", "color": Color(0.9, 0.2, 0.1)},
	"became_adult": {"icon": "A", "color": Color(0.5, 0.7, 0.9)},
	"trait_violation": {"icon": "V", "color": Color(0.9, 0.5, 0.1)},
	"food": {"icon": "F", "color": Color(0.50, 0.85, 0.35)},
	"danger": {"icon": "!", "color": Color(0.95, 0.30, 0.28)},
	"warmth": {"icon": "H", "color": Color(0.95, 0.70, 0.30)},
	"social": {"icon": "G", "color": Color(0.40, 0.75, 0.95)},
}

## Filter button rects for click detection
var _filter_rects: Array = []  # [{rect: Rect2, index: int}]


## Initializes the panel with the EntityManager reference for entity name resolution and navigation.
func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager


func _ready() -> void:
	Locale.locale_changed.connect(func(_l):
		_invalidate_text_cache()
		queue_redraw()
	)


func _process(_delta: float) -> void:
	if visible:
		queue_redraw()


func _gui_input(event: InputEvent) -> void:
	# Scrollbar drag handling
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		if event.pressed:
			if _scrollbar_rect.size.x > 0 and _scrollbar_rect.has_point(event.position):
				_scrollbar_dragging = true
				_update_scroll_from_mouse(event.position.y)
				accept_event()
				return
		else:
			if _scrollbar_dragging:
				_scrollbar_dragging = false
				accept_event()
				return

	if event is InputEventMouseMotion and _scrollbar_dragging:
		_update_scroll_from_mouse(event.position.y)
		accept_event()
		return

	# Existing input handling
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			_scroll_offset = minf(_scroll_offset + 30.0, maxf(0.0, _content_height - size.y + 60.0))
			accept_event()
		elif event.button_index == MOUSE_BUTTON_WHEEL_UP:
			_scroll_offset = maxf(_scroll_offset - 30.0, 0.0)
			accept_event()
		elif event.button_index == MOUSE_BUTTON_LEFT:
			# Check filter buttons
			for fr in _filter_rects:
				if fr.rect.has_point(event.position):
					_filter_index = fr.index
					_filter_type = _current_filter_options()[_filter_index]
					_scroll_offset = 0.0
					_invalidate_text_cache()
					accept_event()
					return
			# Check entity click regions
			for region in _click_regions:
				if region.rect.has_point(event.position):
					_navigate_to_entity(region.entity_id)
					accept_event()
					return
			for region in _entry_click_regions:
				if region.rect.has_point(event.position):
					_toggle_entry_detail(region.entry_id)
					accept_event()
					return
	elif event is InputEventPanGesture:
		_scroll_offset += event.delta.y * 15.0
		_scroll_offset = clampf(_scroll_offset, 0.0, maxf(0.0, _content_height - size.y + 60.0))
		accept_event()


func _update_scroll_from_mouse(mouse_y: float) -> void:
	var track_top: float = _scrollbar_rect.position.y
	var track_height: float = _scrollbar_rect.size.y
	if track_height <= 0.0:
		return
	var ratio: float = clampf((mouse_y - track_top) / track_height, 0.0, 1.0)
	var scroll_max: float = maxf(0.0, _content_height - size.y + 60.0)
	_scroll_offset = ratio * scroll_max


## Draw the fixed header (title, filters, separator, event count).
## Returns the Y where scrollable content begins.
func _draw_header(font: Font, panel_w: float, events_count: int) -> float:
	var cx: float = 20.0
	var cy: float = 28.0
	var filter_label_keys: Array = _current_filter_label_keys()

	# Opaque header background
	draw_rect(Rect2(0, 0, panel_w, 76.0), Color(0.05, 0.05, 0.08, 0.95))

	# Title
	draw_string(font, Vector2(cx, cy), Locale.ltr("UI_CHRONICLE"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_title"), Color.WHITE)
	cy += 10.0

	# Filter buttons
	_filter_rects.clear()
	var btn_x: float = cx
	var btn_y: float = cy + 4.0
	for i in range(filter_label_keys.size()):
		var label: String = Locale.ltr(filter_label_keys[i])
		var label_size: Vector2 = font.get_string_size(label, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"))
		var btn_w: float = label_size.x + 12
		var btn_h: float = 20.0
		var btn_rect := Rect2(btn_x, btn_y, btn_w, btn_h)
		if i == _filter_index:
			draw_rect(btn_rect, Color(0.3, 0.5, 0.3, 0.8))
		else:
			draw_rect(btn_rect, Color(0.2, 0.2, 0.25, 0.8))
		draw_rect(btn_rect, Color(0.4, 0.4, 0.4), false, 1.0)
		draw_string(font, Vector2(btn_x + 6, btn_y + 15), label, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), Color.WHITE)
		_filter_rects.append({"rect": btn_rect, "index": i})
		btn_x += btn_w + 4
	cy += 30.0

	# Separator
	draw_line(Vector2(cx, cy), Vector2(panel_w - 20, cy), Color(0.3, 0.3, 0.3), 1.0)
	cy += 8.0

	# Event count
	if events_count > 0:
		draw_string(font, Vector2(panel_w - 120, 28), Locale.trf1("UI_EVENTS_COUNT", "n", events_count), HORIZONTAL_ALIGNMENT_RIGHT, -1, GameConfig.get_font_size("popup_small"), Color(0.5, 0.5, 0.5))

	return cy


func _draw() -> void:
	if not visible:
		return
	_click_regions.clear()
	_entry_click_regions.clear()

	var panel_w: float = size.x
	var panel_h: float = size.y

	# Background
	draw_rect(Rect2(0, 0, panel_w, panel_h), Color(0.05, 0.05, 0.08, 0.95))
	draw_rect(Rect2(0, 0, panel_w, panel_h), Color(0.3, 0.3, 0.4), false, 1.0)

	var font: Font = ThemeDB.fallback_font

	var events: Array = _get_display_events()

	# Draw header first pass (establishes cy)
	var cy: float = _draw_header(font, panel_w, events.size())
	var cx: float = 20.0

	if events.is_empty() and not _chronicle_data_available():
		draw_string(font, Vector2(cx, cy + 14), Locale.ltr("UI_CHRONICLE_UNAVAILABLE"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
		_content_height = cy + 40.0
		return

	if events.size() == 0:
		draw_string(font, Vector2(cx, cy + 14), Locale.ltr("UI_NO_EVENTS"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
		_content_height = cy + 40.0
		return

	# Header bottom boundary for clipping
	var header_bottom: float = cy

	# Apply scroll offset
	cy -= _scroll_offset

	# Draw events grouped by year
	_ensure_text_cache(events)
	var current_year: int = -1
	var deceased_registry: Node = get_node_or_null("/root/DeceasedRegistry")
	for i in range(events.size()):
		var evt: Dictionary = events[i]
		var entry_id: int = int(evt.get("entry_id", -1))
		var year: int = int(evt.get("year", -1))
		if year < 0 and evt.has("tick"):
			var tick_date: Dictionary = GameCalendar.tick_to_date(int(evt.get("tick", 0)))
			year = int(tick_date.get("year", 0))

		# Year header
		if year != current_year:
			current_year = year
			if cy + 14 > header_bottom and cy < panel_h + 20:
				draw_string(font, Vector2(cx, cy + 14), "── Y%d ──" % year, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color(0.7, 0.7, 0.8))
			cy += 22.0

		var headline: String = _headline_cache[i] if i < _headline_cache.size() else _resolve_event_headline(evt)
		var capsule: String = _capsule_cache[i] if i < _capsule_cache.size() else _resolve_event_capsule(evt)
		var is_selected: bool = entry_id >= 0 and entry_id == _selected_entry_id
		var detail_snapshot: Dictionary = _selected_entry_detail_for(entry_id) if is_selected else {}
		if not detail_snapshot.is_empty():
			var detail_capsule: String = _resolve_event_capsule(detail_snapshot)
			if not detail_capsule.is_empty():
				capsule = detail_capsule
		var dossier_stub: String = _resolve_event_dossier_stub(detail_snapshot) if is_selected else ""
		var entry_height: float = 28.0 if not capsule.is_empty() else 16.0
		if not dossier_stub.is_empty():
			entry_height += 12.0

		# Skip if off screen (use header_bottom instead of -20)
		if cy + entry_height < header_bottom:
			cy += entry_height
			continue
		if cy > panel_h + 20:
			cy += entry_height
			continue

		# Event entry
		var style: Dictionary = _event_style(evt)
		var icon_color: Color = style.color
		if is_selected:
			draw_rect(
				Rect2(cx - 4.0, cy - 2.0, panel_w - (cx * 2.0) + 8.0, entry_height),
				Color(0.18, 0.22, 0.28, 0.75)
			)

		# Date
		var date_str: String
		var event_tick: int = int(evt.get("tick", -1))
		if evt.has("tick") and event_tick >= 0:
			date_str = GameCalendar.format_short_datetime_with_year(event_tick)
		elif evt.has("hour"):
			date_str = "M%d D%d %02d:00" % [int(evt.get("month", 0)), int(evt.get("day", 0)), int(evt.get("hour", 0))]
		else:
			date_str = Locale.trf2(
				"UI_SHORT_DATE",
				"month",
				int(evt.get("month", 0)),
				"day",
				int(evt.get("day", 0))
			)
		draw_string(font, Vector2(cx + 5, cy + 12), date_str, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), Color(0.5, 0.5, 0.5))

		# Icon
		var date_width: float = font.get_string_size(date_str, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small")).x
		var icon_x: float = cx + 10 + date_width + 8.0
		draw_string(font, Vector2(icon_x, cy + 12), style.icon, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), icon_color)

		# Headline / capsule layered text
		draw_string(font, Vector2(icon_x + 18, cy + 12), headline, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), Color(0.92, 0.92, 0.92))
		if not capsule.is_empty():
			draw_string(font, Vector2(icon_x + 18, cy + 24), capsule, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), Color(0.62, 0.62, 0.68))
		if not dossier_stub.is_empty():
			draw_string(font, Vector2(icon_x + 18, cy + 36), dossier_stub, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), Color(0.48, 0.56, 0.66))

		if entry_id >= 0:
			_entry_click_regions.append({
				"rect": Rect2(cx - 4.0, cy - 2.0, panel_w - (cx * 2.0) + 8.0, entry_height),
				"entry_id": entry_id,
			})

		# Make entity name clickable
		var entity_id: int = evt.get("entity_id", -1)
		if entity_id >= 0:
			var entity_name: String = evt.get("entity_name", "")
			if entity_name.length() > 0:
				var clickable_text: String = capsule
				var clickable_y: float = cy + 24
				if clickable_text.find(entity_name) < 0:
					clickable_text = headline
					clickable_y = cy + 12
				# Find name position in the most descriptive visible line.
				var name_start: int = clickable_text.find(entity_name)
				if name_start >= 0:
					var pre_text: String = clickable_text.substr(0, name_start)
					var pre_w: float = font.get_string_size(pre_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small")).x
					var name_w: float = font.get_string_size(entity_name, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small")).x
					var name_pos := Vector2(icon_x + 18 + pre_w, clickable_y)
					_click_regions.append({"rect": Rect2(name_pos.x, name_pos.y - GameConfig.get_font_size("popup_small"), name_w, GameConfig.get_font_size("popup_small") + 4), "entity_id": entity_id})

		# Make related entity names clickable
		var related_ids: Array = evt.get("related_ids", [])
		for rid in related_ids:
			if rid == entity_id:
				continue
			var rname: String = ""
			var rentity: RefCounted = _entity_manager.get_entity(rid) if _entity_manager != null else null
			if rentity != null:
				rname = rentity.entity_name
			else:
				if deceased_registry != null:
					rname = deceased_registry.get_record(rid).get("name", "")
			if rname.length() == 0:
				continue
			var related_clickable_text: String = capsule
			var related_clickable_y: float = cy + 24
			if related_clickable_text.find(rname) < 0:
				related_clickable_text = headline
				related_clickable_y = cy + 12
			var rstart: int = related_clickable_text.find(rname)
			if rstart < 0:
				continue
			var rpre_text: String = related_clickable_text.substr(0, rstart)
			var rpre_w: float = font.get_string_size(rpre_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small")).x
			var rname_w: float = font.get_string_size(rname, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small")).x
			var rname_pos := Vector2(icon_x + 18 + rpre_w, related_clickable_y)
			_click_regions.append({"rect": Rect2(rname_pos.x, rname_pos.y - GameConfig.get_font_size("popup_small"), rname_w, GameConfig.get_font_size("popup_small") + 4), "entity_id": rid})

		cy += entry_height

	_content_height = cy + _scroll_offset + 40.0

	# Redraw header on top to clip any content that bled into header zone
	_draw_header(font, panel_w, events.size())

	# Footer
	draw_string(font, Vector2(panel_w * 0.5 - 60, panel_h - 12), Locale.ltr("UI_SCROLL_HINT"), HORIZONTAL_ALIGNMENT_CENTER, -1, GameConfig.get_font_size("popup_small"), Color(0.4, 0.4, 0.4))
	_draw_scrollbar()


func _draw_scrollbar() -> void:
	# Only show when content overflows
	if _content_height <= size.y:
		_scrollbar_rect = Rect2()
		return

	var panel_h: float = size.y
	var scrollbar_width: float = 6.0
	var scrollbar_margin: float = 3.0
	var scrollbar_x: float = size.x - scrollbar_width - scrollbar_margin
	var track_top: float = 4.0
	var track_bottom: float = panel_h - 4.0
	var track_height: float = track_bottom - track_top

	# Store track rect for drag input (wider hit area)
	_scrollbar_rect = Rect2(scrollbar_x - 4.0, track_top, scrollbar_width + 8.0, track_height)

	# Draw track background
	draw_rect(Rect2(scrollbar_x, track_top, scrollbar_width, track_height), Color(0.15, 0.15, 0.15, 0.3))

	# Calculate grabber size and position
	var visible_ratio: float = clampf(panel_h / _content_height, 0.05, 1.0)
	var grabber_height: float = maxf(track_height * visible_ratio, 20.0)
	var scroll_max: float = maxf(0.0, _content_height - panel_h)
	var scroll_ratio: float = clampf(_scroll_offset / scroll_max, 0.0, 1.0) if scroll_max > 0.0 else 0.0
	var grabber_y: float = track_top + (track_height - grabber_height) * scroll_ratio

	# Draw grabber
	draw_rect(Rect2(scrollbar_x, grabber_y, scrollbar_width, grabber_height), Color(0.6, 0.6, 0.6, 0.5))


func _navigate_to_entity(entity_id: int) -> void:
	if _entity_manager == null:
		return
	var entity: RefCounted = _entity_manager.get_entity(entity_id)
	if entity != null and entity.is_alive:
		SimulationBus.entity_selected.emit(entity_id)
		# Request opening entity detail through UI notification
		SimulationBus.ui_notification.emit("open_entity_detail", "command")
	else:
		# Try to open deceased detail
		SimulationBus.ui_notification.emit("open_deceased_%d" % entity_id, "command")


func _invalidate_text_cache() -> void:
	_text_cache_signature = ""
	_headline_cache.resize(0)
	_capsule_cache.resize(0)


func _ensure_text_cache(events: Array) -> void:
	var signature: String = _compute_text_cache_signature(events)
	if _text_cache_signature == signature and _capsule_cache.size() == events.size() and _headline_cache.size() == events.size():
		return
	_text_cache_signature = signature
	_headline_cache.resize(events.size())
	_capsule_cache.resize(events.size())
	for i in range(events.size()):
		_headline_cache[i] = _resolve_event_headline(events[i])
		_capsule_cache[i] = _resolve_event_capsule(events[i])


func _compute_text_cache_signature(events: Array) -> String:
	var locale_key: String = str(Locale.current_locale)
	if events.is_empty():
		return "%s|%s|0|%d|%d" % [locale_key, _filter_type, _selected_entry_id, _selected_entry_snapshot_revision]
	var first_evt: Dictionary = events[0]
	var last_evt: Dictionary = events[events.size() - 1]
	return "%s|%s|%d|%d|%s|%d|%s|%d|%d|%d" % [
		locale_key,
		_filter_type,
		events.size(),
		int(first_evt.get("tick", -1)),
		str(first_evt.get("event_type", "")),
		int(last_evt.get("tick", -1)),
		str(last_evt.get("event_type", "")),
		int(last_evt.get("entity_id", -1)),
		_selected_entry_id,
		_selected_entry_snapshot_revision
	]


## Temporary migration fallback:
## if the layered snapshot fields are missing, use legacy flat fields derived by the bridge.
func _resolve_chronicle_text(evt: Dictionary, key_field: String, params_field: String, fallback_field: String) -> String:
	var desc: String = str(evt.get(fallback_field, ""))
	if evt.has(key_field):
		var l10n_key: String = str(evt.get(key_field, ""))
		if not l10n_key.is_empty():
			var l10n_params: Dictionary = evt.get(params_field, {})
			if l10n_params.has("cause_id"):
				var l10n_params_with_cause: Dictionary = l10n_params.duplicate()
				l10n_params_with_cause["cause"] = Locale.tr_id("DEATH", l10n_params["cause_id"])
				desc = Locale.trf(l10n_key, l10n_params_with_cause)
			else:
				desc = Locale.trf(l10n_key, l10n_params)
	return desc


func _resolve_event_headline(evt: Dictionary) -> String:
	var headline: String = _resolve_chronicle_text(evt, "headline_key", "headline_params", "title_key")
	var legacy_title_key: String = str(evt.get("title_key", ""))
	if headline == legacy_title_key and not legacy_title_key.is_empty():
		var legacy_title_params: Dictionary = {}
		if evt.has("headline_params"):
			legacy_title_params = evt.get("headline_params", {})
		elif evt.has("l10n_params"):
			legacy_title_params = evt.get("l10n_params", {})
		headline = Locale.trf(legacy_title_key, legacy_title_params)
	if headline == legacy_title_key and evt.has("l10n_key"):
		headline = _resolve_chronicle_text(evt, "l10n_key", "l10n_params", "description")
	return _trim_chronicle_line(headline, 34)


func _resolve_event_capsule(evt: Dictionary) -> String:
	var capsule: String = _resolve_chronicle_text(evt, "capsule_key", "capsule_params", "description")
	if capsule.is_empty() or capsule == str(evt.get("description", "")):
		capsule = _resolve_chronicle_text(evt, "l10n_key", "l10n_params", "description")
	return _trim_chronicle_line(capsule, 55)


func _resolve_event_dossier_stub(evt: Dictionary) -> String:
	if evt.is_empty():
		return ""
	var dossier: String = _resolve_chronicle_text(evt, "dossier_stub_key", "dossier_stub_params", "")
	return _trim_chronicle_line(dossier, 72)


func _trim_chronicle_line(text: String, max_chars: int) -> String:
	if text.length() > max_chars:
		return text.substr(0, max_chars - 3) + "..."
	return text


func _using_runtime_chronicle() -> bool:
	return SimBridge.runtime_is_initialized()


func _current_filter_options() -> Array:
	return RUNTIME_FILTER_OPTIONS


func _current_filter_label_keys() -> Array:
	return RUNTIME_FILTER_LABEL_KEYS


func _chronicle_data_available() -> bool:
	return _using_runtime_chronicle()


func _get_display_events() -> Array:
	if _using_runtime_chronicle():
		return _runtime_chronicle_events()
	_log_legacy_chronicle_fallback()
	_feed_snapshot_revision = -1
	_clear_selected_entry_detail()
	return []


func _runtime_chronicle_events() -> Array:
	var response: Dictionary = SimBridge.runtime_get_chronicle_feed(200)
	var revision: int = int(response.get("snapshot_revision", -1))
	if bool(response.get("revision_unavailable", false)) and revision < 0:
		if not _legacy_fallback_logged:
			_legacy_fallback_logged = true
			push_warning("[Chronicle] runtime chronicle feed unavailable; legacy timeline adapter is compatibility-only and not used by the live panel")
		_feed_snapshot_revision = -1
		_clear_selected_entry_detail()
		return []
	_legacy_fallback_logged = false
	_feed_snapshot_revision = revision
	var events: Array = response.get("items", [])
	var selected_entry_visible: bool = false
	if _filter_type.is_empty():
		for evt in events:
			if evt is Dictionary:
				var dict: Dictionary = evt
				if int(dict.get("entry_id", -1)) == _selected_entry_id:
					selected_entry_visible = true
					break
		if not selected_entry_visible:
			_clear_selected_entry_detail()
		return events
	var filtered: Array = []
	for evt in events:
		if not (evt is Dictionary):
			continue
		var dict: Dictionary = evt
		if str(dict.get("cause_id", "")) == _filter_type:
			filtered.append(dict)
			if int(dict.get("entry_id", -1)) == _selected_entry_id:
				selected_entry_visible = true
	if not selected_entry_visible:
		_clear_selected_entry_detail()
	return filtered


func _event_style(evt: Dictionary) -> Dictionary:
	var cause_id: String = str(evt.get("cause_id", ""))
	if not cause_id.is_empty():
		return EVENT_STYLES.get(cause_id, {"icon": "?", "color": Color(0.6, 0.6, 0.6)})
	var event_type: String = str(evt.get("event_type", ""))
	return EVENT_STYLES.get(event_type, {"icon": "?", "color": Color(0.6, 0.6, 0.6)})


func _log_legacy_chronicle_fallback() -> void:
	if _legacy_fallback_logged:
		return
	_legacy_fallback_logged = true
	push_warning("[Chronicle] runtime timeline unavailable; legacy ChronicleSystem fallback is disabled")


func _toggle_entry_detail(entry_id: int) -> void:
	if entry_id < 0:
		return
	if _selected_entry_id == entry_id:
		_clear_selected_entry_detail()
	else:
		_selected_entry_id = entry_id
		_selected_entry_snapshot_revision = -1
		_selected_entry_detail = {}
		_refresh_selected_entry_detail()
	_invalidate_text_cache()
	queue_redraw()


func _selected_entry_detail_for(entry_id: int) -> Dictionary:
	if entry_id < 0 or entry_id != _selected_entry_id:
		return {}
	if _selected_entry_detail.is_empty() or _selected_entry_snapshot_revision != _feed_snapshot_revision:
		_refresh_selected_entry_detail()
	return _selected_entry_detail


func _refresh_selected_entry_detail() -> void:
	if _selected_entry_id < 0 or not _using_runtime_chronicle():
		_clear_selected_entry_detail()
		return
	var response: Dictionary = SimBridge.runtime_get_chronicle_entry_detail(_selected_entry_id, _feed_snapshot_revision)
	if bool(response.get("revision_unavailable", false)):
		_selected_entry_detail = {}
		_selected_entry_snapshot_revision = -1
		return
	if not bool(response.get("available", false)):
		_selected_entry_detail = {}
		_selected_entry_snapshot_revision = int(response.get("snapshot_revision", -1))
		return
	_selected_entry_detail = response
	_selected_entry_snapshot_revision = int(response.get("snapshot_revision", -1))


func _clear_selected_entry_detail() -> void:
	_selected_entry_id = -1
	_selected_entry_snapshot_revision = -1
	_selected_entry_detail = {}
