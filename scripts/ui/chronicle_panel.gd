class_name ChroniclePanel
extends Control

var _entity_manager: RefCounted

## Filter state
var _filter_type: String = ""  # "" = all
const FILTER_OPTIONS: Array = ["", "birth", "death", "marriage", "settlement_founded", "population_milestone"]
const FILTER_LABELS: Array = ["All", "Birth", "Death", "Marriage", "Settlement", "Milestone"]
var _filter_index: int = 0

## Scroll state
var _scroll_offset: float = 0.0
var _content_height: float = 0.0

## Scrollbar drag state
var _scrollbar_dragging: bool = false
var _scrollbar_rect: Rect2 = Rect2()

## Clickable regions
var _click_regions: Array = []  # [{rect: Rect2, entity_id: int}]

## Event type icons and colors
const EVENT_STYLES: Dictionary = {
	"birth": {"icon": "B", "color": Color(0.3, 0.9, 0.3)},
	"death": {"icon": "D", "color": Color(0.9, 0.3, 0.3)},
	"marriage": {"icon": "C", "color": Color(0.9, 0.4, 0.6)},
	"orphaned": {"icon": "O", "color": Color(0.6, 0.5, 0.7)},
	"partner_died": {"icon": "W", "color": Color(0.6, 0.4, 0.5)},
	"settlement_founded": {"icon": "S", "color": Color(0.9, 0.7, 0.2)},
	"population_milestone": {"icon": "P", "color": Color(0.3, 0.8, 0.9)},
	"famine": {"icon": "!", "color": Color(0.9, 0.2, 0.1)},
	"became_adult": {"icon": "A", "color": Color(0.5, 0.7, 0.9)},
}

## Filter button rects for click detection
var _filter_rects: Array = []  # [{rect: Rect2, index: int}]


func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager


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
					_filter_type = FILTER_OPTIONS[_filter_index]
					_scroll_offset = 0.0
					accept_event()
					return
			# Check entity click regions
			for region in _click_regions:
				if region.rect.has_point(event.position):
					_navigate_to_entity(region.entity_id)
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

	# Opaque header background
	draw_rect(Rect2(0, 0, panel_w, 76.0), Color(0.05, 0.05, 0.08, 0.95))

	# Title
	draw_string(font, Vector2(cx, cy), "Chronicle", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_title"), Color.WHITE)
	cy += 10.0

	# Filter buttons
	_filter_rects.clear()
	var btn_x: float = cx
	var btn_y: float = cy + 4.0
	for i in range(FILTER_LABELS.size()):
		var label: String = FILTER_LABELS[i]
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
		draw_string(font, Vector2(panel_w - 120, 28), "%d events" % events_count, HORIZONTAL_ALIGNMENT_RIGHT, -1, GameConfig.get_font_size("popup_small"), Color(0.5, 0.5, 0.5))

	return cy


func _draw() -> void:
	if not visible:
		return
	_click_regions.clear()

	var panel_w: float = size.x
	var panel_h: float = size.y

	# Background
	draw_rect(Rect2(0, 0, panel_w, panel_h), Color(0.05, 0.05, 0.08, 0.95))
	draw_rect(Rect2(0, 0, panel_w, panel_h), Color(0.3, 0.3, 0.4), false, 1.0)

	var font: Font = ThemeDB.fallback_font

	# Get events from ChronicleSystem
	var chronicle: Node = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
	var events: Array = []
	if chronicle != null:
		events = chronicle.get_world_events(_filter_type, 200)

	# Draw header first pass (establishes cy)
	var cy: float = _draw_header(font, panel_w, events.size())
	var cx: float = 20.0

	if chronicle == null:
		draw_string(font, Vector2(cx, cy + 14), "ChronicleSystem not available", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
		_content_height = cy + 40.0
		return

	if events.size() == 0:
		draw_string(font, Vector2(cx, cy + 14), "No events recorded yet", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
		_content_height = cy + 40.0
		return

	# Header bottom boundary for clipping
	var header_bottom: float = cy

	# Apply scroll offset
	cy -= _scroll_offset

	# Draw events grouped by year
	var current_year: int = -1
	for i in range(events.size()):
		var evt: Dictionary = events[i]
		var year: int = evt.get("year", 0)

		# Year header
		if year != current_year:
			current_year = year
			if cy + 14 > header_bottom and cy < panel_h + 20:
				draw_string(font, Vector2(cx, cy + 14), "── Y%d ──" % year, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color(0.7, 0.7, 0.8))
			cy += 22.0

		# Skip if off screen (use header_bottom instead of -20)
		if cy + 16 < header_bottom:
			cy += 16.0
			continue
		if cy > panel_h + 20:
			cy += 16.0
			continue

		# Event entry
		var event_type: String = evt.get("event_type", "")
		var style: Dictionary = EVENT_STYLES.get(event_type, {"icon": "?", "color": Color(0.6, 0.6, 0.6)})
		var icon_color: Color = style.color

		# Date
		var date_str: String = "%d월 %d일" % [evt.get("month", 0), evt.get("day", 0)]
		draw_string(font, Vector2(cx + 5, cy + 12), date_str, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), Color(0.5, 0.5, 0.5))

		# Icon
		var icon_x: float = cx + 75
		draw_string(font, Vector2(icon_x, cy + 12), style.icon, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), icon_color)

		# Description (truncated)
		var desc: String = evt.get("description", "?")
		if desc.length() > 55:
			desc = desc.substr(0, 52) + "..."
		draw_string(font, Vector2(icon_x + 18, cy + 12), desc, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), Color(0.8, 0.8, 0.8))

		# Make entity name clickable
		var entity_id: int = evt.get("entity_id", -1)
		if entity_id >= 0:
			var entity_name: String = evt.get("entity_name", "")
			if entity_name.length() > 0:
				# Find name position in description
				var name_start: int = desc.find(entity_name)
				if name_start >= 0:
					var pre_text: String = desc.substr(0, name_start)
					var pre_w: float = font.get_string_size(pre_text, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small")).x
					var name_w: float = font.get_string_size(entity_name, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small")).x
					var name_pos := Vector2(icon_x + 18 + pre_w, cy + 12)
					_click_regions.append({"rect": Rect2(name_pos.x, name_pos.y - GameConfig.get_font_size("popup_small"), name_w, GameConfig.get_font_size("popup_small") + 4), "entity_id": entity_id})

		cy += 16.0

	_content_height = cy + _scroll_offset + 40.0

	# Redraw header on top to clip any content that bled into header zone
	_draw_header(font, panel_w, events.size())

	# Footer
	draw_string(font, Vector2(panel_w * 0.5 - 60, panel_h - 12), "Scroll for more | Click background to close", HORIZONTAL_ALIGNMENT_CENTER, -1, GameConfig.get_font_size("popup_small"), Color(0.4, 0.4, 0.4))
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
