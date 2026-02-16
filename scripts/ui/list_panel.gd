class_name ListPanel
extends Control

const GameCalendar = preload("res://scripts/core/game_calendar.gd")

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _settlement_manager: RefCounted

## Tab state
var _current_tab: int = 0  # 0=entities, 1=buildings
const TAB_LABELS: Array = ["Entities", "Buildings"]

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

## Scroll
var _scroll_offset: float = 0.0
var _content_height: float = 0.0

## Click regions
var _click_regions: Array = []  # [{rect: Rect2, entity_id: int}]
var _tab_rects: Array = []
var _sort_rects: Array = []
var _page_rects: Array = []  # [{rect: Rect2, action: String}]
var _toggle_deceased_rect: Rect2 = Rect2()

## Column definitions for entities
const ENTITY_COLUMNS: Array = [
	{"key": "name", "label": "Name", "width": 90},
	{"key": "age", "label": "Age", "width": 80},
	{"key": "job", "label": "Job", "width": 70},
	{"key": "status", "label": "Status", "width": 70},
	{"key": "settlement", "label": "Sett", "width": 35},
	{"key": "hunger", "label": "Hunger", "width": 50},
]

const BUILDING_COLUMNS: Array = [
	{"key": "name", "label": "Type", "width": 100},
	{"key": "settlement", "label": "Sett", "width": 40},
	{"key": "position", "label": "Position", "width": 80},
	{"key": "status", "label": "Status", "width": 80},
]


func init(entity_manager: RefCounted, building_manager: RefCounted = null, settlement_manager: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_settlement_manager = settlement_manager


func _process(_delta: float) -> void:
	if visible:
		queue_redraw()


func _gui_input(event: InputEvent) -> void:
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
	for i in range(TAB_LABELS.size()):
		var label: String = TAB_LABELS[i]
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
		var toggle_label: String = "[%s] Deceased" % ("x" if _show_deceased else " ")
		var toggle_w: float = font.get_string_size(toggle_label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small).x + 8
		_toggle_deceased_rect = Rect2(panel_w - toggle_w - 15, cy + 2, toggle_w, 20)
		draw_string(font, Vector2(panel_w - toggle_w - 11, cy + 17), toggle_label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, Color(0.6, 0.6, 0.6))

	cy += 30.0

	if _current_tab == 0:
		_draw_entity_list(font, cx, cy, panel_w, panel_h, fs_body, fs_small)
	else:
		_draw_building_list(font, cx, cy, panel_w, panel_h, fs_body, fs_small)


func _draw_entity_list(font: Font, cx: float, start_cy: float, panel_w: float, panel_h: float, fs_body: int, fs_small: int) -> void:
	var cy: float = start_cy

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
			rows.append({
				"id": e.id, "name": e.entity_name, "age": age_detail.total_days,
				"age_display": age_short,
				"job": e.job, "status": e.current_action, "settlement": e.settlement_id,
				"hunger": e.hunger, "deceased": false,
			})

	# Add deceased
	if _show_deceased:
		var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
		if registry != null:
			var deceased: Array = registry.get_all()
			var cause_map: Dictionary = {"starvation": "아사", "old_age": "노령", "infant_mortality": "영아 사망", "background": "사고/질병", "maternal_death": "출산 사망", "stillborn": "사산"}
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
				var cause_kr: String = cause_map.get(cause_raw, cause_raw)
				rows.append({
					"id": r.get("id", -1), "name": r.get("name", "?"),
					"age": d_total_days, "age_display": d_age_short,
					"job": r.get("job", ""),
					"status": "사망-%s" % cause_kr, "settlement": r.get("settlement_id", 0),
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
	for col in ENTITY_COLUMNS:
		var label: String = col.label
		if _sort_key == col.key:
			label += " %s" % ("v" if _sort_ascending else "^")
		var header_rect := Rect2(col_x, cy, col.width, 16)
		draw_string(font, Vector2(col_x, cy + 12), label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, Color(0.8, 0.8, 0.3))
		_sort_rects.append({"rect": header_rect, "key": col.key})
		col_x += col.width
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
		draw_string(font, Vector2(col_x, draw_y + 14), display_name, HORIZONTAL_ALIGNMENT_LEFT, col_x + ENTITY_COLUMNS[0].width - 5, fs_small, text_color if not is_deceased else Color(0.6, 0.4, 0.4))
		col_x += ENTITY_COLUMNS[0].width

		# Age (short format)
		var age_text: String = row.get("age_display", "%d" % int(row.age))
		draw_string(font, Vector2(col_x, draw_y + 14), age_text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, text_color)
		col_x += ENTITY_COLUMNS[1].width

		# Job
		draw_string(font, Vector2(col_x, draw_y + 14), str(row.job).substr(0, 8), HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, text_color)
		col_x += ENTITY_COLUMNS[2].width

		# Status
		var status_text: String = str(row.status).substr(0, 10)
		var status_color: Color = text_color
		if is_deceased:
			status_color = Color(0.6, 0.3, 0.3)
		draw_string(font, Vector2(col_x, draw_y + 14), status_text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, status_color)
		col_x += ENTITY_COLUMNS[3].width

		# Settlement
		var sett_text: String = "S%d" % row.settlement if row.settlement > 0 else "-"
		draw_string(font, Vector2(col_x, draw_y + 14), sett_text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, text_color)
		col_x += ENTITY_COLUMNS[4].width

		# Hunger
		if not is_deceased:
			var h: float = row.hunger
			var h_color: Color = Color(0.9, 0.2, 0.2) if h < 0.3 else (Color(0.9, 0.8, 0.2) if h < 0.6 else Color(0.3, 0.8, 0.3))
			draw_string(font, Vector2(col_x, draw_y + 14), "%d%%" % int(h * 100), HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, h_color)

		# Register click region
		_click_regions.append({"rect": row_rect, "entity_id": row.id, "deceased": is_deceased})
		row_y += ROW_HEIGHT

	# Footer: total count
	var footer_y: float = panel_h - 24
	var count_text: String = "%d entities" % rows.size()
	draw_string(font, Vector2(panel_w * 0.5 - 40, footer_y + 12), count_text, HORIZONTAL_ALIGNMENT_CENTER, -1, fs_small, Color(0.6, 0.6, 0.6))


func _draw_building_list(font: Font, cx: float, start_cy: float, panel_w: float, panel_h: float, fs_body: int, fs_small: int) -> void:
	var cy: float = start_cy

	if _building_manager == null:
		draw_string(font, Vector2(cx, cy + 14), "No building manager", HORIZONTAL_ALIGNMENT_LEFT, -1, fs_body, Color(0.5, 0.5, 0.5))
		_content_height = cy + 40.0
		return

	var buildings: Array = _building_manager.get_all_buildings()

	# Column headers
	var col_x: float = cx + 5
	for col in BUILDING_COLUMNS:
		var label: String = col.label
		draw_string(font, Vector2(col_x, cy + 12), label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, Color(0.8, 0.8, 0.3))
		col_x += col.width
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
		col_x += BUILDING_COLUMNS[0].width

		var sett_text: String = "S%d" % b.settlement_id if b.settlement_id > 0 else "-"
		draw_string(font, Vector2(col_x, cy + 14), sett_text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, text_color)
		col_x += BUILDING_COLUMNS[1].width

		draw_string(font, Vector2(col_x, cy + 14), "(%d,%d)" % [b.tile_x, b.tile_y], HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, text_color)
		col_x += BUILDING_COLUMNS[2].width

		var status: String = "Built" if b.is_built else "%d%%" % int(b.build_progress * 100)
		draw_string(font, Vector2(col_x, cy + 14), status, HORIZONTAL_ALIGNMENT_LEFT, -1, fs_small, Color(0.3, 0.8, 0.3) if b.is_built else Color(0.9, 0.7, 0.2))
		cy += ROW_HEIGHT

	_content_height = cy + 40.0


func _on_entity_clicked(entity_id: int, is_deceased: bool) -> void:
	if is_deceased:
		SimulationBus.ui_notification.emit("open_deceased_%d" % entity_id, "command")
	else:
		SimulationBus.entity_selected.emit(entity_id)
		SimulationBus.ui_notification.emit("open_entity_detail", "command")
