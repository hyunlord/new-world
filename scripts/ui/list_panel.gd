class_name ListPanel
extends Control

const GameCalendar = preload("res://scripts/core/game_calendar.gd")

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted

## Tab state
var _current_tab: int = 0  # 0=entities, 1=buildings

## Sort state
var _sort_key: String = "name"
var _sort_ascending: bool = true

## Filter
var _search_text: String = ""
var _show_deceased: bool = true

## Pagination
var _page: int = 0
const ITEMS_PER_PAGE: int = 30
const ROW_HEIGHT: float = 18.0
const COL_PAD: float = 6.0

## Scroll
var _scroll_offset: float = 0.0
var _content_height: float = 0.0

## Scrollbar drag state
var _scrollbar_dragging: bool = false
var _scrollbar_rect: Rect2 = Rect2()

## Click regions
var _click_regions: Array = []  # [{rect: Rect2, entity_id: int}]
var _tab_rects: Array = []
var _sort_rects: Array = []
var _page_rects: Array = []  # [{rect: Rect2, action: String}]
var _toggle_deceased_rect: Rect2 = Rect2()

## Column definitions for entities
const ENTITY_COLUMNS: Array = [
	{"key": "name", "label": "UI_NAME", "min_width": 70, "weight": 15},
	{"key": "age", "label": "UI_AGE", "min_width": 90, "weight": 18},
	{"key": "born", "label": "UI_BORN", "min_width": 72, "weight": 10},
	{"key": "died", "label": "UI_DIED", "min_width": 72, "weight": 10},
	{"key": "job", "label": "UI_JOB", "min_width": 55, "weight": 10},
	{"key": "status", "label": "UI_STATUS", "min_width": 80, "weight": 17},
	{"key": "settlement", "label": "UI_SETTLEMENT", "min_width": 28, "weight": 5},
	{"key": "hunger", "label": "UI_HUNGER", "min_width": 42, "weight": 8},
]

const BUILDING_COLUMNS: Array = [
	{"key": "name", "label": "UI_BUILDINGS", "width": 100},
	{"key": "settlement", "label": "UI_SETTLEMENT", "width": 40},
	{"key": "position", "label": "UI_POS", "width": 80},
	{"key": "status", "label": "UI_STATUS", "width": 80},
]


func _ready() -> void:
	if not Locale.locale_changed.is_connected(_on_locale_changed):
		Locale.locale_changed.connect(_on_locale_changed)


func init(entity_manager: RefCounted, building_manager: RefCounted = null, settlement_manager: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_settlement_manager = settlement_manager


func _on_locale_changed(_locale: String) -> void:
	queue_redraw()


func _get_tab_label(index: int) -> String:
	if index == 0:
		return Locale.tr("UI_ENTITIES")
	return Locale.tr("UI_BUILDINGS")


static func _format_date_compact(date: Dictionary) -> String:
	if date.is_empty():
		return "?"
	return "Y%d.%d.%d" % [date.get("year", 0), date.get("month", 1), date.get("day", 1)]


func _compute_column_widths(columns: Array, available_width: float) -> Array:
	var total_weight: float = 0.0
	var total_min: float = 0.0
	for col in columns:
		total_weight += float(col.get("weight", 10))
		total_min += float(col.get("min_width", 50))
	var pad_total: float = float(columns.size() - 1) * COL_PAD
	var usable: float = available_width - pad_total
	var extra: float = maxf(0.0, usable - total_min)
	var widths: Array = []
	for col in columns:
		var min_w: float = float(col.get("min_width", 50))
		var w: float = float(col.get("weight", 10))
		widths.append(min_w + extra * (w / total_weight))
	return widths


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

	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			_scroll_offset = minf(_scroll_offset + 30.0, maxf(0.0, _content_height - size.y + 60.0))
			accept_event()
		elif event.button_index == MOUSE_BUTTON_WHEEL_UP:
			_scroll_offset = maxf(_scroll_offset - 30.0, 0.0)
			accept_event()
		elif event.button_index == MOUSE_BUTTON_LEFT:
			# Tab clicks
			for tr in _tab_rects:
				if tr.rect.has_point(event.position):
					_current_tab = tr.index
					_scroll_offset = 0.0
					accept_event()
					return
			# Sort header clicks
			for sr in _sort_rects:
				if sr.rect.has_point(event.position):
					if _sort_key == sr.key:
						_sort_ascending = not _sort_ascending
					else:
						_sort_key = sr.key
						_sort_ascending = true
					accept_event()
					return
			# Toggle deceased
			if _toggle_deceased_rect.has_point(event.position):
				_show_deceased = not _show_deceased
				_scroll_offset = 0.0
				accept_event()
				return
			# Entity click
			for cr in _click_regions:
				if cr.rect.has_point(event.position):
					_on_entity_clicked(cr.entity_id, cr.get("deceased", false))
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


func _draw() -> void:
	if not visible:
		return
	_click_regions.clear()
	_tab_rects.clear()
	_sort_rects.clear()
	_page_rects.clear()

	var panel_w: float = size.x
	var panel_h: float = size.y

	# Background
	draw_rect(Rect2(0, 0, panel_w, panel_h), Color(0.05, 0.07, 0.05, 0.95))
	draw_rect(Rect2(0, 0, panel_w, panel_h), Color(0.3, 0.4, 0.3), false, 1.0)

	var font: Font = ThemeDB.fallback_font
	var cx: float = 15.0
	var cy: float = 10.0
	var fs_title: int = GameConfig.get_font_size("popup_title")
	var fs_body: int = GameConfig.get_font_size("popup_body")
	var fs_small: int = GameConfig.get_font_size("popup_small")

	# Tabs
	var tab_x: float = cx
	for i in range(2):
		var label: String = _get_tab_label(i)
		var tw: float = font.get_string_size(label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_body).x + 16
		var tab_rect := Rect2(tab_x, cy, tw, 24)
		if i == _current_tab:
			draw_rect(tab_rect, Color(0.2, 0.35, 0.2, 0.9))
		else:
			draw_rect(tab_rect, Color(0.15, 0.15, 0.15, 0.9))
		draw_rect(tab_rect, Color(0.4, 0.4, 0.4), false, 1.0)
		draw_string(font, Vector2(tab_x + 8, cy + 17), label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_body, Color.WHITE)
		_tab_rects.append({"rect": tab_rect, "index": i})
		tab_x += tw + 2

	# Deceased toggle (entities tab only)
	if _current_tab == 0:
		var toggle_label: String = "[%s] %s" % [("x" if _show_deceased else " "), Locale.tr("UI_DECEASED")]
		var toggle_w: float = font.get_string_size(toggle_label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small).x + 8
		_toggle_deceased_rect = Rect2(panel_w - toggle_w - 15, cy + 2, toggle_w, 20)
		draw_string(font, Vector2(panel_w - toggle_w - 11, cy + 17), toggle_label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, Color(0.6, 0.6, 0.6))

	cy += 30.0

	if _current_tab == 0:
		_draw_entity_list(font, cx, cy, panel_w, panel_h, fs_body, fs_small)
	else:
		_draw_building_list(font, cx, cy, panel_w, panel_h, fs_body, fs_small)
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


func _draw_entity_list(font: Font, cx: float, start_cy: float, panel_w: float, panel_h: float, fs_body: int, fs_small: int) -> void:
	var cy: float = start_cy
	var col_widths: Array = _compute_column_widths(ENTITY_COLUMNS, panel_w - 30.0)

	# Gather data
	var rows: Array = []
	if _entity_manager != null:
		var alive: Array = _entity_manager.get_alive_entities()
		for i in range(alive.size()):
			var e: RefCounted = alive[i]
			var current_tick: int = e.birth_tick + e.age
			var ref_d: Dictionary = GameCalendar.tick_to_date(current_tick)
			var ref_date: Dictionary = {"year": ref_d.year, "month": ref_d.month, "day": ref_d.day}
			var age_short: String = GameCalendar.format_age_short(e.birth_date, ref_date)
			var age_detail: Dictionary = GameCalendar.calculate_detailed_age(e.birth_date, ref_date)
			var born_display: String = _format_date_compact(e.birth_date)
			var born_days: int = 0
			if not e.birth_date.is_empty():
				born_days = GameCalendar.to_julian_day(e.birth_date)
			rows.append({
				"id": e.id, "name": e.entity_name, "age": age_detail.total_days,
				"age_display": age_short,
				"born": born_days, "born_display": born_display,
				"died": 9999999, "died_display": "-",
				"job": e.job, "status": e.current_action, "settlement": e.settlement_id,
				"hunger": e.hunger, "deceased": false,
			})

	# Add deceased
	if _show_deceased:
		var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
		if registry != null:
			var deceased: Array = registry.get_all()
			for i in range(deceased.size()):
				var r: Dictionary = deceased[i]
				var bd: Dictionary = r.get("birth_date", {})
				var dd: Dictionary = r.get("death_date", {})
				var d_age_short: String = "?"
				var d_total_days: int = int(r.get("death_age_days", 0))
				if not bd.is_empty() and not dd.is_empty():
					d_age_short = GameCalendar.format_age_short(bd, dd)
					var detail: Dictionary = GameCalendar.calculate_detailed_age(bd, dd)
					d_total_days = detail.total_days
				else:
					d_age_short = "%dy" % int(r.get("death_age_years", 0.0))
				var cause_raw: String = r.get("death_cause", "unknown")
				var cause_loc: String = Locale.tr_id("DEATH", cause_raw)
				var d_born_display: String = _format_date_compact(bd)
				var d_born_days: int = 0
				if not bd.is_empty():
					d_born_days = GameCalendar.to_julian_day(bd)
				var d_died_display: String = _format_date_compact(dd)
				var d_died_days: int = 0
				if not dd.is_empty():
					d_died_days = GameCalendar.to_julian_day(dd)
				rows.append({
					"id": r.get("id", -1), "name": r.get("name", "?"),
					"age": d_total_days, "age_display": d_age_short,
					"born": d_born_days, "born_display": d_born_display,
					"died": d_died_days, "died_display": d_died_display,
					"job": r.get("job", ""),
					"status": Locale.trf("UI_DECEASED_STATUS_FMT", {"cause": cause_loc}), "settlement": r.get("settlement_id", 0),
					"hunger": 0.0, "deceased": true,
				})

	# Sort
	rows.sort_custom(func(a, b):
		var va = a.get(_sort_key, "")
		var vb = b.get(_sort_key, "")
		if va is String and vb is String:
			if _sort_ascending:
				return va.naturalnocasecmp_to(vb) < 0
			return va.naturalnocasecmp_to(vb) > 0
		if va is float or va is int:
			if _sort_ascending:
				return va < vb
			return va > vb
		return false
	)

	# Column headers (sortable)
	var col_x: float = cx + 5
	for idx in range(ENTITY_COLUMNS.size()):
		var col: Dictionary = ENTITY_COLUMNS[idx]
		var cw: float = col_widths[idx]
		var label: String = Locale.tr(col.label)
		if _sort_key == col.key:
			label += " %s" % (Locale.tr("UI_SORT_ASC") if _sort_ascending else Locale.tr("UI_SORT_DESC"))
		var header_rect := Rect2(col_x, cy, cw, 16)
		draw_string(font, Vector2(col_x, cy + 12), label, HORIZONTAL_ALIGNMENT_LEFT, int(cw) - 2, fs_small, Color(0.8, 0.8, 0.3))
		_sort_rects.append({"rect": header_rect, "key": col.key})
		col_x += cw + COL_PAD
	cy += 18.0
	draw_line(Vector2(cx, cy), Vector2(panel_w - 15, cy), Color(0.3, 0.3, 0.3), 1.0)
	cy += 4.0

	# Scrollable rows
	var row_area_top: float = cy
	var row_area_height: float = panel_h - cy - 30.0
	_content_height = float(rows.size()) * ROW_HEIGHT + 80.0

	var row_y: float = 0.0
	for i in range(rows.size()):
		var row: Dictionary = rows[i]
		if row_y + ROW_HEIGHT < _scroll_offset:
			row_y += ROW_HEIGHT
			continue
		if row_y - _scroll_offset > row_area_height:
			break

		var draw_y: float = row_area_top + row_y - _scroll_offset
		if draw_y < row_area_top:
			row_y += ROW_HEIGHT
			continue
		var is_deceased: bool = row.get("deceased", false)
		var text_color: Color = Color(0.5, 0.5, 0.5) if is_deceased else Color(0.8, 0.8, 0.8)
		var row_rect := Rect2(cx, draw_y, panel_w - 30, ROW_HEIGHT)

		# Hover highlight (alternating rows)
		if (i % 2) == 1:
			draw_rect(row_rect, Color(0.1, 0.1, 0.1, 0.3))

		col_x = cx + 5
		# Name (with deceased marker)
		var display_name: String = row.name
		if is_deceased:
			display_name = "☠ " + display_name
		draw_string(font, Vector2(col_x, draw_y + 14), display_name, HORIZONTAL_ALIGNMENT_LEFT, int(col_widths[0]) - 2, fs_small, text_color if not is_deceased else Color(0.6, 0.4, 0.4))
		col_x += col_widths[0] + COL_PAD

		# Age (short format)
		var age_text: String = row.get("age_display", "%d" % int(row.age))
		draw_string(font, Vector2(col_x, draw_y + 14), age_text, HORIZONTAL_ALIGNMENT_LEFT, int(col_widths[1]) - 2, fs_small, text_color)
		col_x += col_widths[1] + COL_PAD

		# Born
		draw_string(font, Vector2(col_x, draw_y + 14), row.get("born_display", "?"), HORIZONTAL_ALIGNMENT_LEFT, int(col_widths[2]) - 2, fs_small, text_color)
		col_x += col_widths[2] + COL_PAD

		# Died
		var died_text: String = row.get("died_display", "-")
		var died_color: Color = Color(0.6, 0.3, 0.3) if is_deceased else text_color
		draw_string(font, Vector2(col_x, draw_y + 14), died_text, HORIZONTAL_ALIGNMENT_LEFT, int(col_widths[3]) - 2, fs_small, died_color)
		col_x += col_widths[3] + COL_PAD

		# Job
		draw_string(font, Vector2(col_x, draw_y + 14), str(row.job), HORIZONTAL_ALIGNMENT_LEFT, int(col_widths[4]) - 2, fs_small, text_color)
		col_x += col_widths[4] + COL_PAD

		# Status
		var status_text: String = str(row.status)
		var status_color: Color = text_color
		if is_deceased:
			status_color = Color(0.6, 0.3, 0.3)
		draw_string(font, Vector2(col_x, draw_y + 14), status_text, HORIZONTAL_ALIGNMENT_LEFT, int(col_widths[5]) - 2, fs_small, status_color)
		col_x += col_widths[5] + COL_PAD

		# Settlement
		var sett_text: String = "S%d" % row.settlement if row.settlement > 0 else "-"
		draw_string(font, Vector2(col_x, draw_y + 14), sett_text, HORIZONTAL_ALIGNMENT_LEFT, int(col_widths[6]) - 2, fs_small, text_color)
		col_x += col_widths[6] + COL_PAD

		# Hunger
		if not is_deceased:
			var h: float = row.hunger
			var h_color: Color = Color(0.9, 0.2, 0.2) if h < 0.3 else (Color(0.9, 0.8, 0.2) if h < 0.6 else Color(0.3, 0.8, 0.3))
			draw_string(font, Vector2(col_x, draw_y + 14), "%d%%" % int(h * 100), HORIZONTAL_ALIGNMENT_LEFT, int(col_widths[7]) - 2, fs_small, h_color)

		# Register click region
		_click_regions.append({"rect": row_rect, "entity_id": row.id, "deceased": is_deceased})
		row_y += ROW_HEIGHT

	# Footer: total count
	var footer_y: float = panel_h - 24
	var count_text: String = Locale.trf("UI_ENTITIES_COUNT_FMT", {"n": rows.size()})
	draw_string(font, Vector2(panel_w * 0.5 - 40, footer_y + 12), count_text, HORIZONTAL_ALIGNMENT_CENTER, -1, fs_small, Color(0.6, 0.6, 0.6))


func _draw_building_list(font: Font, cx: float, start_cy: float, panel_w: float, panel_h: float, fs_body: int, fs_small: int) -> void:
	var cy: float = start_cy

	if _building_manager == null:
		var missing_manager_text: String = "%s: %s" % [Locale.tr("UI_BUILDINGS"), Locale.tr("UI_UNKNOWN")]
		draw_string(font, Vector2(cx, cy + 14), missing_manager_text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_body, Color(0.5, 0.5, 0.5))
		_content_height = cy + 40.0
		return

	var buildings: Array = _building_manager.get_all_buildings()

	# Column headers
	var col_x: float = cx + 5
	for col in BUILDING_COLUMNS:
		var label: String = Locale.tr(col.label)
		draw_string(font, Vector2(col_x, cy + 12), label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, Color(0.8, 0.8, 0.3))
		col_x += col.width + COL_PAD
	cy += 18.0
	draw_line(Vector2(cx, cy), Vector2(panel_w - 15, cy), Color(0.3, 0.3, 0.3), 1.0)
	cy += 4.0

	for i in range(buildings.size()):
		var b = buildings[i]
		var text_color := Color(0.8, 0.8, 0.8)
		if (i % 2) == 1:
			draw_rect(Rect2(cx, cy, panel_w - 30, ROW_HEIGHT), Color(0.1, 0.1, 0.1, 0.3))

		col_x = cx + 5
		draw_string(font, Vector2(col_x, cy + 14), b.building_type.capitalize(), HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, text_color)
		col_x += BUILDING_COLUMNS[0].width + COL_PAD

		var sett_text: String = "S%d" % b.settlement_id if b.settlement_id > 0 else "-"
		draw_string(font, Vector2(col_x, cy + 14), sett_text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, text_color)
		col_x += BUILDING_COLUMNS[1].width + COL_PAD

		draw_string(font, Vector2(col_x, cy + 14), "(%d,%d)" % [b.tile_x, b.tile_y], HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, text_color)
		col_x += BUILDING_COLUMNS[2].width + COL_PAD

		var status: String = Locale.tr("UI_BUILT_LABEL") if b.is_built else "%d%%" % int(b.build_progress * 100)
		draw_string(font, Vector2(col_x, cy + 14), status, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, Color(0.3, 0.8, 0.3) if b.is_built else Color(0.9, 0.7, 0.2))
		cy += ROW_HEIGHT

	_content_height = cy + 40.0


func _on_entity_clicked(entity_id: int, is_deceased: bool) -> void:
	if is_deceased:
		SimulationBus.ui_notification.emit("open_entity_%d" % entity_id, "command")
	else:
		SimulationBus.entity_selected.emit(entity_id)
		SimulationBus.ui_notification.emit("open_entity_detail", "command")
