extends Control

const GameCalendar = preload("res://scripts/core/simulation/game_calendar.gd")

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted
var _sim_engine: RefCounted

## Tab state
var _current_tab: int = 0  # 0=entities, 1=buildings, 2=settlements, 3=bands

## Sort state
var _sort_key: String = "name"
var _sort_ascending: bool = true

## Filter
var _show_deceased: bool = true

## Pagination
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
var _cached_tab_labels: Array[String] = []
var _cached_entity_column_labels: Dictionary = {}
var _cached_building_column_labels: Dictionary = {}
var _cached_deceased_label: String = ""
var _cached_sort_asc_label: String = ""
var _cached_sort_desc_label: String = ""

## Column definitions for entities
const ENTITY_COLUMNS: Array = [
	{"key": "name", "label": "UI_NAME", "min_width": 70, "weight": 15},
	{"key": "age", "label": "UI_AGE", "min_width": 90, "weight": 18},
	{"key": "born", "label": "UI_BORN", "min_width": 72, "weight": 10},
	{"key": "died", "label": "UI_DIED", "min_width": 72, "weight": 10},
	{"key": "job", "label": "UI_JOB", "min_width": 55, "weight": 10},
	{"key": "status", "label": "UI_STATUS", "min_width": 80, "weight": 17},
	{"key": "settlement", "label": "UI_SETTLEMENT", "min_width": 28, "weight": 5},
	{"key": "band", "label": "UI_BAND", "min_width": 28, "weight": 5},
	{"key": "hunger", "label": "UI_HUNGER", "min_width": 42, "weight": 8},
]

const BUILDING_COLUMNS: Array = [
	{"key": "name", "label": "UI_BUILDINGS", "width": 100},
	{"key": "settlement", "label": "UI_SETTLEMENT", "width": 40},
	{"key": "position", "label": "UI_POS", "width": 80},
	{"key": "status", "label": "UI_STATUS", "width": 80},
]


func _ready() -> void:
	_refresh_locale_cache()
	if not Locale.locale_changed.is_connected(_on_locale_changed):
		Locale.locale_changed.connect(_on_locale_changed)


## Initializes the panel with EntityManager, BuildingManager, SettlementManager, and SimEngine references for list display.
func init(entity_manager: RefCounted, building_manager: RefCounted = null, settlement_manager: RefCounted = null, sim_engine: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_settlement_manager = settlement_manager
	_sim_engine = sim_engine


func _on_locale_changed(_locale: String) -> void:
	_refresh_locale_cache()
	queue_redraw()


func _get_tab_label(index: int) -> String:
	if index >= 0 and index < _cached_tab_labels.size():
		return _cached_tab_labels[index]
	return Locale.ltr("UI_ENTITIES") if index == 0 else Locale.ltr("UI_BUILDINGS")


func _refresh_locale_cache() -> void:
	_cached_tab_labels = [
		Locale.ltr("UI_ENTITIES"),
		Locale.ltr("UI_BUILDINGS"),
		Locale.ltr("UI_TAB_SETTLEMENTS"),
		Locale.ltr("UI_TAB_BANDS"),
	]
	_cached_deceased_label = Locale.ltr("UI_DECEASED")
	_cached_sort_asc_label = Locale.ltr("UI_SORT_ASC")
	_cached_sort_desc_label = Locale.ltr("UI_SORT_DESC")

	_cached_entity_column_labels.clear()
	for i in range(ENTITY_COLUMNS.size()):
		var col: Dictionary = ENTITY_COLUMNS[i]
		var label_key: String = str(col.get("label", ""))
		_cached_entity_column_labels[label_key] = Locale.ltr(label_key)

	_cached_building_column_labels.clear()
	for i in range(BUILDING_COLUMNS.size()):
		var col: Dictionary = BUILDING_COLUMNS[i]
		var label_key: String = str(col.get("label", ""))
		_cached_building_column_labels[label_key] = Locale.ltr(label_key)


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
			for tr_item in _tab_rects:
				if tr_item.rect.has_point(event.position):
					_current_tab = tr_item.index
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
			# Row click (entity, settlement, or band)
			for cr in _click_regions:
				if cr.rect.has_point(event.position):
					match cr.get("type", "entity"):
						"settlement":
							SimulationBus.ui_notification.emit("open_settlement_%d" % int(cr.get("id", -1)), "command")
						"band":
							SimulationBus.band_selected.emit(int(cr.get("id", -1)))
						_:
							_on_entity_clicked(int(cr.get("entity_id", cr.get("id", -1))), cr.get("deceased", false))
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
	var _fs_title: int = GameConfig.get_font_size("popup_title")
	var fs_body: int = GameConfig.get_font_size("popup_body")
	var fs_small: int = GameConfig.get_font_size("popup_small")

	# Tabs
	var tab_x: float = cx
	for i in range(4):
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
		var toggle_label: String = "[%s] %s" % [("x" if _show_deceased else " "), _cached_deceased_label]
		var toggle_w: float = font.get_string_size(toggle_label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small).x + 8
		_toggle_deceased_rect = Rect2(panel_w - toggle_w - 15, cy + 2, toggle_w, 20)
		draw_string(font, Vector2(panel_w - toggle_w - 11, cy + 17), toggle_label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, Color(0.6, 0.6, 0.6))

	cy += 30.0

	match _current_tab:
		0:
			_draw_entity_list(font, cx, cy, panel_w, panel_h, fs_body, fs_small)
		1:
			_draw_building_list(font, cx, cy, panel_w, panel_h, fs_body, fs_small)
		2:
			_draw_settlement_list(font, cx, cy, panel_w, panel_h, fs_body, fs_small)
		3:
			_draw_band_list(font, cx, cy, panel_w, panel_h, fs_body, fs_small)
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


func _draw_entity_list(font: Font, cx: float, start_cy: float, panel_w: float, panel_h: float, _fs_body: int, fs_small: int) -> void:
	var cy: float = start_cy
	var col_widths: Array = _compute_column_widths(ENTITY_COLUMNS, panel_w - 30.0)

	# Gather data
	var rows: Array = _get_entity_rows_from_bridge()

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
					d_age_short = Locale.trf1("UI_AGE_YEARS_SHORT_FMT", "n", int(r.get("death_age_years", 0.0)))
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
						"job_display": Locale.tr_id("JOB", str(r.get("job", ""))),
						"status": Locale.trf1("UI_DECEASED_STATUS_FMT", "cause", cause_loc), "settlement": r.get("settlement_id", -1),
						"band_name": "", "hunger": 0.0, "deceased": true, "is_leader": false,
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
		var label_key: String = str(col.get("label", ""))
		var label: String = str(_cached_entity_column_labels.get(label_key, Locale.ltr(label_key)))
		if _sort_key == col.key:
			label += " %s" % (_cached_sort_asc_label if _sort_ascending else _cached_sort_desc_label)
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
		# Name (with leader crown and/or deceased marker)
		var display_name: String = row.name
		var name_color: Color = text_color if not is_deceased else Color(0.6, 0.4, 0.4)
		if row.get("is_leader", false):
			display_name = "\u265B " + display_name
			if not is_deceased:
				name_color = Color(1.0, 0.82, 0.1)
		elif is_deceased:
			display_name = "☠ " + display_name
		draw_string(font, Vector2(col_x, draw_y + 14), display_name, HORIZONTAL_ALIGNMENT_LEFT, int(col_widths[0]) - 2, fs_small, name_color)
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
		var job_text: String = str(row.get("job_display", Locale.tr_id("JOB", str(row.job))))
		draw_string(font, Vector2(col_x, draw_y + 14), job_text, HORIZONTAL_ALIGNMENT_LEFT, int(col_widths[4]) - 2, fs_small, text_color)
		col_x += col_widths[4] + COL_PAD

		# Status
		var status_text: String = str(row.status)
		var status_color: Color = text_color
		if is_deceased:
			status_color = Color(0.6, 0.3, 0.3)
		draw_string(font, Vector2(col_x, draw_y + 14), status_text, HORIZONTAL_ALIGNMENT_LEFT, int(col_widths[5]) - 2, fs_small, status_color)
		col_x += col_widths[5] + COL_PAD

		# Settlement
		var sett_text: String = "S%d" % row.settlement if row.settlement >= 0 else "-"
		draw_string(font, Vector2(col_x, draw_y + 14), sett_text, HORIZONTAL_ALIGNMENT_LEFT, int(col_widths[6]) - 2, fs_small, text_color)
		col_x += col_widths[6] + COL_PAD

		# Band
		var band_text: String = str(row.get("band_name", ""))
		if band_text.is_empty():
			band_text = "-"
		draw_string(font, Vector2(col_x, draw_y + 14), band_text, HORIZONTAL_ALIGNMENT_LEFT, int(col_widths[7]) - 2, fs_small, text_color)
		col_x += col_widths[7] + COL_PAD

		# Hunger
		if not is_deceased:
			var h: float = row.hunger
			var h_color: Color = Color(0.9, 0.2, 0.2) if h < 0.3 else (Color(0.9, 0.8, 0.2) if h < 0.6 else Color(0.3, 0.8, 0.3))
			draw_string(font, Vector2(col_x, draw_y + 14), "%d%%" % int(h * 100), HORIZONTAL_ALIGNMENT_LEFT, int(col_widths[8]) - 2, fs_small, h_color)

		# Register click region
		_click_regions.append({"rect": row_rect, "entity_id": row.id, "deceased": is_deceased})
		row_y += ROW_HEIGHT

	# Footer: total count
	var footer_y: float = panel_h - 24
	var count_text: String = Locale.trf1("UI_ENTITIES_COUNT_FMT", "n", rows.size())
	draw_string(font, Vector2(panel_w * 0.5 - 40, footer_y + 12), count_text, HORIZONTAL_ALIGNMENT_CENTER, -1, fs_small, Color(0.6, 0.6, 0.6))


func _draw_building_list(font: Font, cx: float, start_cy: float, panel_w: float, _panel_h: float, fs_body: int, fs_small: int) -> void:
	var cy: float = start_cy

	var buildings: Array = []
	if _building_manager != null:
		buildings = _building_manager.get_all_buildings()
	if buildings.is_empty():
		buildings = _get_building_rows_from_bridge()
	if buildings.is_empty():
		var missing_manager_text: String = "%s: %s" % [Locale.ltr("UI_BUILDINGS"), Locale.ltr("UI_UNKNOWN")]
		draw_string(font, Vector2(cx, cy + 14), missing_manager_text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_body, Color(0.5, 0.5, 0.5))
		_content_height = cy + 40.0
		return
	var building_type_cache: Dictionary = {}
	var built_label: String = Locale.ltr("UI_BUILT_LABEL")

	# Column headers
	var col_x: float = cx + 5
	for col in BUILDING_COLUMNS:
		var label_key: String = str(col.get("label", ""))
		var label: String = str(_cached_building_column_labels.get(label_key, Locale.ltr(label_key)))
		draw_string(font, Vector2(col_x, cy + 12), label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, Color(0.8, 0.8, 0.3))
		col_x += col.width + COL_PAD
	cy += 18.0
	draw_line(Vector2(cx, cy), Vector2(panel_w - 15, cy), Color(0.3, 0.3, 0.3), 1.0)
	cy += 4.0

	for i in range(buildings.size()):
		var b: Variant = buildings[i]
		var text_color := Color(0.8, 0.8, 0.8)
		if (i % 2) == 1:
			draw_rect(Rect2(cx, cy, panel_w - 30, ROW_HEIGHT), Color(0.1, 0.1, 0.1, 0.3))

		col_x = cx + 5
		var building_type: String = str(_building_row_value(b, "building_type", ""))
		var building_type_key: String = "BUILDING_TYPE_" + building_type.to_upper()
		var building_type_name: String = str(building_type_cache.get(building_type_key, ""))
		if building_type_name.is_empty():
			building_type_name = Locale.ltr(building_type_key)
			building_type_cache[building_type_key] = building_type_name
		draw_string(font, Vector2(col_x, cy + 14), building_type_name, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, text_color)
		col_x += BUILDING_COLUMNS[0].width + COL_PAD

		var settlement_id: int = int(_building_row_value(b, "settlement_id", 0))
		var sett_text: String = "S%d" % settlement_id if settlement_id > 0 else "-"
		draw_string(font, Vector2(col_x, cy + 14), sett_text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, text_color)
		col_x += BUILDING_COLUMNS[1].width + COL_PAD

		var tile_x: int = int(_building_row_value(b, "tile_x", 0))
		var tile_y: int = int(_building_row_value(b, "tile_y", 0))
		draw_string(font, Vector2(col_x, cy + 14), "(%d,%d)" % [tile_x, tile_y], HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, text_color)
		col_x += BUILDING_COLUMNS[2].width + COL_PAD

		var is_built: bool = bool(_building_row_value(b, "is_built", _building_row_value(b, "is_constructed", false)))
		var build_progress: float = float(_building_row_value(b, "build_progress", _building_row_value(b, "construction_progress", 0.0)))
		var status: String = built_label if is_built else "%d%%" % int(build_progress * 100)
		draw_string(font, Vector2(col_x, cy + 14), status, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, Color(0.3, 0.8, 0.3) if is_built else Color(0.9, 0.7, 0.2))
		cy += ROW_HEIGHT

	_content_height = cy + 40.0


func _draw_settlement_list(font: Font, cx: float, start_cy: float, panel_w: float, panel_h: float, _fs_body: int, fs_small: int) -> void:
	var cy: float = start_cy

	var sim_bridge: Object = _get_sim_bridge()
	var settlements: Array = []
	if sim_bridge != null and sim_bridge.has_method("runtime_get_world_summary"):
		var raw: Variant = sim_bridge.call("runtime_get_world_summary")
		if raw is Dictionary:
			for ss: Variant in raw.get("settlement_summaries", []):
				if ss is Dictionary:
					settlements.append(ss)

	if settlements.is_empty():
		draw_string(font, Vector2(cx, cy + 14), Locale.ltr("UI_NONE"), HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, Color(0.5, 0.5, 0.5))
		_content_height = cy + 40.0
		return

	# Column headers
	draw_string(font, Vector2(cx + 5, cy + 12), Locale.ltr("UI_NAME"), HORIZONTAL_ALIGNMENT_LEFT, 100, fs_small, Color(0.8, 0.8, 0.3))
	draw_string(font, Vector2(cx + 130, cy + 12), Locale.ltr("UI_POPULATION"), HORIZONTAL_ALIGNMENT_LEFT, 70, fs_small, Color(0.8, 0.8, 0.3))
	draw_string(font, Vector2(cx + 210, cy + 12), Locale.ltr("UI_ERA"), HORIZONTAL_ALIGNMENT_LEFT, 80, fs_small, Color(0.8, 0.8, 0.3))
	cy += 18.0
	draw_line(Vector2(cx, cy), Vector2(panel_w - 15, cy), Color(0.3, 0.3, 0.3), 1.0)
	cy += 4.0

	var row_area_top: float = cy
	var row_area_height: float = panel_h - cy - 30.0
	_content_height = float(settlements.size()) * ROW_HEIGHT + 80.0

	var row_y: float = 0.0
	for i in range(settlements.size()):
		var ss: Dictionary = settlements[i]
		var sett: Dictionary = ss.get("settlement", {}) if ss.get("settlement") is Dictionary else {}
		var sett_id: int = int(ss.get("id", sett.get("id", -1)))
		var sett_name: String = str(ss.get("name", sett.get("name", "S%d" % sett_id)))
		var pop: int = int(ss.get("pop", sett.get("population", 0)))
		var era: String = str(ss.get("tech_era", sett.get("tech_era", "")))

		if row_y + ROW_HEIGHT < _scroll_offset:
			row_y += ROW_HEIGHT
			continue
		if row_y - _scroll_offset > row_area_height:
			break

		var draw_y: float = row_area_top + row_y - _scroll_offset
		if draw_y < row_area_top:
			row_y += ROW_HEIGHT
			continue

		var row_rect := Rect2(cx, draw_y, panel_w - 30, ROW_HEIGHT)
		if (i % 2) == 1:
			draw_rect(row_rect, Color(0.1, 0.1, 0.1, 0.3))

		draw_string(font, Vector2(cx + 5, draw_y + 14), sett_name, HORIZONTAL_ALIGNMENT_LEFT, 120, fs_small, Color(0.8, 0.8, 0.8))
		draw_string(font, Vector2(cx + 130, draw_y + 14), str(pop), HORIZONTAL_ALIGNMENT_LEFT, 70, fs_small, Color(0.6, 0.8, 0.6))
		draw_string(font, Vector2(cx + 210, draw_y + 14), era, HORIZONTAL_ALIGNMENT_LEFT, 80, fs_small, Color(0.6, 0.6, 0.8))

		if sett_id >= 0:
			_click_regions.append({"rect": row_rect, "type": "settlement", "id": sett_id})
		row_y += ROW_HEIGHT

	var footer_y: float = panel_h - 24
	draw_string(font, Vector2(panel_w * 0.5 - 40, footer_y + 12), Locale.trf1("UI_ENTITIES_COUNT_FMT", "n", settlements.size()), HORIZONTAL_ALIGNMENT_CENTER, -1, fs_small, Color(0.6, 0.6, 0.6))


func _draw_band_list(font: Font, cx: float, start_cy: float, panel_w: float, panel_h: float, _fs_body: int, fs_small: int) -> void:
	var cy: float = start_cy

	var bands: Array = []
	if _sim_engine != null and _sim_engine.has_method("get_band_list"):
		bands = _sim_engine.get_band_list()
	if bands.is_empty():
		var sim_bridge: Object = _get_sim_bridge()
		if sim_bridge != null and sim_bridge.has_method("runtime_get_band_list"):
			var raw: Variant = sim_bridge.call("runtime_get_band_list")
			if raw is Array:
				bands = raw

	if bands.is_empty():
		draw_string(font, Vector2(cx, cy + 14), Locale.ltr("UI_POPUP_NO_BANDS"), HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, Color(0.5, 0.5, 0.5))
		_content_height = cy + 40.0
		return

	# Column headers
	draw_string(font, Vector2(cx + 5, cy + 12), Locale.ltr("UI_NAME"), HORIZONTAL_ALIGNMENT_LEFT, 100, fs_small, Color(0.8, 0.8, 0.3))
	draw_string(font, Vector2(cx + 130, cy + 12), Locale.ltr("UI_MEMBERS"), HORIZONTAL_ALIGNMENT_LEFT, 50, fs_small, Color(0.8, 0.8, 0.3))
	draw_string(font, Vector2(cx + 190, cy + 12), Locale.ltr("UI_STATUS"), HORIZONTAL_ALIGNMENT_LEFT, 60, fs_small, Color(0.8, 0.8, 0.3))
	draw_string(font, Vector2(cx + 260, cy + 12), Locale.ltr("UI_LEADER"), HORIZONTAL_ALIGNMENT_LEFT, 80, fs_small, Color(0.8, 0.8, 0.3))
	cy += 18.0
	draw_line(Vector2(cx, cy), Vector2(panel_w - 15, cy), Color(0.3, 0.3, 0.3), 1.0)
	cy += 4.0

	var row_area_top: float = cy
	var row_area_height: float = panel_h - cy - 30.0
	_content_height = float(bands.size()) * ROW_HEIGHT + 80.0

	var row_y: float = 0.0
	for i in range(bands.size()):
		if not (bands[i] is Dictionary):
			continue
		var band: Dictionary = bands[i]
		var band_id: int = int(band.get("id", -1))
		var band_name: String = str(band.get("name", "?"))
		var member_count: int = int(band.get("member_count", 0))
		var is_promoted: bool = bool(band.get("is_promoted", false))
		var leader_name: String = str(band.get("leader_name", ""))
		var status_text: String = Locale.ltr("UI_BAND_PROMOTED") if is_promoted else Locale.ltr("UI_BAND_PROVISIONAL")

		if row_y + ROW_HEIGHT < _scroll_offset:
			row_y += ROW_HEIGHT
			continue
		if row_y - _scroll_offset > row_area_height:
			break

		var draw_y: float = row_area_top + row_y - _scroll_offset
		if draw_y < row_area_top:
			row_y += ROW_HEIGHT
			continue

		var row_rect := Rect2(cx, draw_y, panel_w - 30, ROW_HEIGHT)
		if (i % 2) == 1:
			draw_rect(row_rect, Color(0.1, 0.1, 0.1, 0.3))

		draw_string(font, Vector2(cx + 5, draw_y + 14), band_name, HORIZONTAL_ALIGNMENT_LEFT, 120, fs_small, Color(0.8, 0.8, 0.8))
		draw_string(font, Vector2(cx + 130, draw_y + 14), str(member_count), HORIZONTAL_ALIGNMENT_LEFT, 50, fs_small, Color(0.6, 0.8, 0.6))
		var status_color: Color = Color(0.4, 0.7, 0.4) if is_promoted else Color(0.7, 0.7, 0.4)
		draw_string(font, Vector2(cx + 190, draw_y + 14), status_text, HORIZONTAL_ALIGNMENT_LEFT, 60, fs_small, status_color)
		draw_string(font, Vector2(cx + 260, draw_y + 14), leader_name, HORIZONTAL_ALIGNMENT_LEFT, 80, fs_small, Color(0.7, 0.7, 0.7))

		if band_id >= 0:
			_click_regions.append({"rect": row_rect, "type": "band", "id": band_id})
		row_y += ROW_HEIGHT

	var footer_y: float = panel_h - 24
	draw_string(font, Vector2(panel_w * 0.5 - 40, footer_y + 12), Locale.trf1("UI_ENTITIES_COUNT_FMT", "n", bands.size()), HORIZONTAL_ALIGNMENT_CENTER, -1, fs_small, Color(0.6, 0.6, 0.6))


func _on_entity_clicked(entity_id: int, is_deceased: bool) -> void:
	if is_deceased:
		SimulationBus.ui_notification.emit("open_deceased_%d" % entity_id, "command")
	else:
		SimulationBus.entity_selected.emit(entity_id)
		SimulationBus.ui_notification.emit("open_entity_detail", "command")


## Returns row dicts from SimBridge runtime_get_entity_list (fallback when entity_manager is null).
func _get_entity_rows_from_bridge() -> Array:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("runtime_get_entity_list"):
		return []
	var raw: Variant = sim_bridge.call("runtime_get_entity_list")
	if not (raw is Array):
		return []
	var rows: Array = []
	for item_raw: Variant in raw:
		if not (item_raw is Dictionary):
			continue
		var item: Dictionary = item_raw
		if not item.get("alive", true):
			continue
		var age_years: float = float(item.get("age_years", 0.0))
		var job_raw: String = str(item.get("job", ""))
		var settlement_id: int = int(item.get("settlement_id", -1))
		var action_raw: String = str(item.get("current_action", "Idle"))
		rows.append({
			"id": int(item.get("entity_id", 0)),
			"name": str(item.get("name", "")),
			"age": int(age_years * 365.0),
			"age_display": Locale.trf1("UI_AGE_YEARS_SHORT_FMT", "n", int(age_years)),
			"born": 0, "born_display": "-",
			"died": 9999999, "died_display": "-",
			"job": job_raw, "job_display": Locale.tr_id("JOB", job_raw),
			"status": _localized_status_text(action_raw), "settlement": settlement_id,
			"band_name": str(item.get("band_name", "")),
			"hunger": float(item.get("hunger", 0.0)),
			"deceased": false,
			"is_leader": bool(item.get("is_leader", false)),
		})
	return rows


func _localized_status_text(action_raw: String) -> String:
	if action_raw.is_empty():
		return Locale.ltr("STATUS_IDLE")
	return Locale.ltr("STATUS_" + _camel_to_upper_snake(action_raw))


func _camel_to_upper_snake(value: String) -> String:
	if value.is_empty():
		return ""
	var chars: PackedStringArray = PackedStringArray()
	for index: int in range(value.length()):
		var current: String = value.substr(index, 1)
		var lower: String = current.to_lower()
		var upper: String = current.to_upper()
		var is_uppercase: bool = current == upper and current != lower
		if index > 0 and is_uppercase:
			chars.append("_")
		chars.append(upper)
	return "".join(chars)


func _get_building_rows_from_bridge() -> Array:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("runtime_get_world_summary"):
		return []
	var raw: Variant = sim_bridge.call("runtime_get_world_summary")
	if not (raw is Dictionary):
		return []
	var summary: Dictionary = raw
	var rows: Array = []
	var settlements: Array = summary.get("settlement_summaries", [])
	for settlement_summary: Variant in settlements:
		if not (settlement_summary is Dictionary):
			continue
		var settlement_detail: Variant = settlement_summary.get("settlement", {})
		if not (settlement_detail is Dictionary):
			continue
		var buildings: Array = settlement_detail.get("buildings", [])
		for building_raw: Variant in buildings:
			if building_raw is Dictionary:
				rows.append(building_raw)
	return rows


func _building_row_value(building: Variant, key: String, default_value: Variant) -> Variant:
	if building is Dictionary:
		return building.get(key, default_value)
	if building == null:
		return default_value
	return building.get(key)


## Programmatically switch to a tab (0=entities, 1=buildings, 2=settlements, 3=bands).
func set_tab(tab_index: int) -> void:
	_current_tab = clampi(tab_index, 0, 3)
	_scroll_offset = 0.0
	queue_redraw()


func _get_sim_bridge() -> Object:
	if Engine.has_singleton("SimBridge"):
		var bridge: Object = Engine.get_singleton("SimBridge")
		if bridge != null:
			return bridge
	var tree: SceneTree = Engine.get_main_loop() as SceneTree
	if tree == null or tree.root == null:
		return null
	return tree.root.get_node_or_null("SimBridge")
