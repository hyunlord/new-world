extends Control

const OverviewTab = preload("res://scripts/ui/panels/settlement_tabs/settlement_overview_tab.gd")
const TechTab = preload("res://scripts/ui/panels/settlement_tabs/settlement_tech_tab.gd")
const PopulationTab = preload("res://scripts/ui/panels/settlement_tabs/settlement_population_tab.gd")
const EconomyTab = preload("res://scripts/ui/panels/settlement_tabs/settlement_economy_tab.gd")

## Manager references (injected via init())
var _settlement_manager: RefCounted
var _entity_manager: RefCounted
var _building_manager: RefCounted
var _tech_tree_manager: RefCounted

## State
var _settlement_id: int = -1
var _current_tab: int = 0  # 0=Overview, 1=Tech, 2=Population, 3=Economy
var _cached_data: Dictionary = {}
var _refresh_counter: int = 0

## Tab instances
var _overview_tab: RefCounted
var _tech_tab: RefCounted
var _population_tab: RefCounted
var _economy_tab: RefCounted

## Scroll state (same as stats_detail_panel.gd)
var _scroll_offset: float = 0.0
var _content_height: float = 0.0
var _scrollbar_dragging: bool = false
var _scrollbar_rect: Rect2 = Rect2()

## Click regions
var _click_regions: Array = []  # [{rect: Rect2, entity_id: int}]
var _tab_rects: Array = []      # [{rect: Rect2, tab_index: int}]
var _close_rect: Rect2 = Rect2()

## Style constants
const PANEL_BG: Color = Color(0.08, 0.08, 0.12, 0.95)
const HEADER_BG: Color = Color(0.10, 0.10, 0.16, 1.0)
const TAB_ACTIVE: Color = Color(0.3, 0.5, 0.8)
const TAB_INACTIVE: Color = Color(0.3, 0.3, 0.35)
const TAB_HOVER: Color = Color(0.35, 0.4, 0.5)
const SECTION_HEADER_COLOR: Color = Color(0.9, 0.85, 0.6)
const POSITIVE_COLOR: Color = Color(0.4, 0.8, 0.4)
const NEGATIVE_COLOR: Color = Color(0.8, 0.4, 0.4)
const NEUTRAL_COLOR: Color = Color(0.7, 0.7, 0.7)
const SEPARATOR_COLOR: Color = Color(0.3, 0.3, 0.4)
const HEADER_HEIGHT: float = 55.0
const TAB_BAR_HEIGHT: float = 35.0
const TAB_NAMES: Array = ["UI_TAB_OVERVIEW", "UI_TAB_TECHNOLOGY", "UI_TAB_POPULATION", "UI_TAB_ECONOMY"]


## Initializes the panel with manager references and creates tab instances.
func init(settlement_manager: RefCounted, entity_manager: RefCounted, building_manager: RefCounted, tech_tree_manager: RefCounted) -> void:
	_settlement_manager = settlement_manager
	_entity_manager = entity_manager
	_building_manager = building_manager
	_tech_tree_manager = tech_tree_manager
	_overview_tab = OverviewTab.new()
	_tech_tab = TechTab.new()
	_population_tab = PopulationTab.new()
	_economy_tab = EconomyTab.new()


## Sets the active settlement and reloads data.
func set_settlement_id(id: int) -> void:
	_settlement_id = id
	_scroll_offset = 0.0
	_current_tab = 0
	_load_data()


func _load_data() -> void:
	if _settlement_manager == null or _settlement_id < 0:
		_cached_data = {}
		return

	var settlement: RefCounted = _settlement_manager.get_settlement(_settlement_id)
	if settlement == null:
		_cached_data = {}
		return

	var members: Array = []
	var adults: int = 0
	var children: int = 0
	var elders: int = 0
	var teens: int = 0
	var male_count: int = 0
	var female_count: int = 0
	var total_happiness: float = 0.0
	var total_stress: float = 0.0

	for member_id in settlement.member_ids:
		var entity: RefCounted = _entity_manager.get_entity(member_id)
		if entity == null or not entity.is_alive:
			continue
		members.append(entity)

		var age_years: float = float(entity.age) / float(GameConfig.TICKS_PER_YEAR)
		if age_years < 12.0:
			children += 1
		elif age_years < 18.0:
			teens += 1
		elif age_years < 55.0:
			adults += 1
		else:
			elders += 1

		if entity.gender == "male":
			male_count += 1
		else:
			female_count += 1

		total_happiness += entity.emotions.get("happiness", 0.5)
		total_stress += entity.emotions.get("stress", 0.0)

	var pop: int = members.size()
	var avg_happiness: float = total_happiness / float(pop) if pop > 0 else 0.0
	var avg_stress: float = total_stress / float(pop) if pop > 0 else 0.0

	var leader: RefCounted = null
	if settlement.leader_id > -1:
		leader = _entity_manager.get_entity(settlement.leader_id)

	_cached_data = {
		"settlement": settlement,
		"members": members,
		"leader": leader,
		"population": pop,
		"adults": adults,
		"children": children,
		"elders": elders,
		"teens": teens,
		"male_count": male_count,
		"female_count": female_count,
		"avg_happiness": avg_happiness,
		"avg_stress": avg_stress,
		"tech_tree_manager": _tech_tree_manager,
		"entity_manager": _entity_manager,
		"building_manager": _building_manager,
		"settlement_manager": _settlement_manager,
	}


func _ready() -> void:
	Locale.locale_changed.connect(func(_l): queue_redraw())


func _process(_delta: float) -> void:
	if not visible:
		return
	queue_redraw()
	_refresh_counter += 1
	if _refresh_counter >= GameConfig.SETTLEMENT_PANEL_REFRESH_TICKS:
		_refresh_counter = 0
		_load_data()


func _draw() -> void:
	if not visible or _settlement_id < 0 or _cached_data.is_empty():
		return

	var panel_w: float = size.x
	var panel_h: float = size.y
	var font: Font = ThemeDB.fallback_font

	# Panel background and border
	draw_rect(Rect2(0.0, 0.0, panel_w, panel_h), PANEL_BG)
	draw_rect(Rect2(0.0, 0.0, panel_w, panel_h), SEPARATOR_COLOR, false, 1.0)

	_click_regions.clear()
	_tab_rects.clear()

	# Header
	draw_rect(Rect2(0.0, 0.0, panel_w, HEADER_HEIGHT), HEADER_BG)

	var settlement: RefCounted = _cached_data.get("settlement")
	var settlement_name: String = ""
	var era_key: String = ""
	if settlement != null:
		settlement_name = Locale.ltr("UI_SETTLEMENT") + " %d" % settlement.id
		era_key = "ERA_" + settlement.tech_era.to_upper()

	# Settlement name
	draw_string(font, Vector2(20.0, 28.0), settlement_name,
		HORIZONTAL_ALIGNMENT_LEFT, -1,
		GameConfig.get_font_size("popup_title"), Color.WHITE)

	# Era badge
	if settlement != null and settlement.tech_era != "":
		var era_text: String = "[" + Locale.ltr(era_key) + "]"
		var era_color: Color = _era_color(settlement.tech_era)
		var name_width: float = font.get_string_size(settlement_name, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_title")).x
		draw_string(font, Vector2(24.0 + name_width, 28.0), era_text,
			HORIZONTAL_ALIGNMENT_LEFT, -1,
			GameConfig.get_font_size("popup_body"), era_color)

	# Population count
	var pop_text: String = Locale.trf("UI_STAT_POP_FMT", {"n": _cached_data.get("population", 0)})
	draw_string(font, Vector2(20.0, 47.0), pop_text,
		HORIZONTAL_ALIGNMENT_LEFT, -1,
		GameConfig.get_font_size("popup_small"), NEUTRAL_COLOR)

	# Close button
	var close_size: float = 24.0
	var close_x: float = panel_w - close_size - 8.0
	var close_y: float = 8.0
	_close_rect = Rect2(close_x, close_y, close_size, close_size)
	draw_rect(_close_rect, Color(0.4, 0.15, 0.15, 0.8))
	draw_string(font, Vector2(close_x + 5.0, close_y + 17.0), "\u00d7",
		HORIZONTAL_ALIGNMENT_LEFT, -1,
		GameConfig.get_font_size("popup_heading"), Color(0.9, 0.5, 0.5))

	# Tab bar
	var tab_y: float = HEADER_HEIGHT
	var tab_w: float = panel_w / float(TAB_NAMES.size())
	for i in range(TAB_NAMES.size()):
		var tab_rect := Rect2(float(i) * tab_w, tab_y, tab_w, TAB_BAR_HEIGHT)
		var tab_bg: Color = TAB_ACTIVE if i == _current_tab else TAB_INACTIVE
		draw_rect(tab_rect, tab_bg)
		draw_rect(tab_rect, SEPARATOR_COLOR, false, 1.0)
		var label: String = Locale.ltr(TAB_NAMES[i])
		draw_string(font, Vector2(tab_rect.position.x + 8.0, tab_y + 22.0), label,
			HORIZONTAL_ALIGNMENT_LEFT, tab_w - 10.0,
			GameConfig.get_font_size("popup_body"), Color.WHITE)
		_tab_rects.append({"rect": tab_rect, "tab_index": i})

	# Separator line below tab bar
	var content_top: float = HEADER_HEIGHT + TAB_BAR_HEIGHT
	draw_line(Vector2(0.0, content_top), Vector2(panel_w, content_top), SEPARATOR_COLOR, 1.0)

	# Delegate to active tab
	var content_x: float = 20.0
	var content_y: float = content_top + 10.0 - _scroll_offset
	var content_width: float = panel_w - 40.0

	var end_y: float = content_y
	match _current_tab:
		0:
			end_y = _overview_tab.draw_content(self, _cached_data, font, content_x, content_y, content_width, _click_regions)
		1:
			end_y = _tech_tab.draw_content(self, _cached_data, font, content_x, content_y, content_width, _click_regions)
		2:
			end_y = _population_tab.draw_content(self, _cached_data, font, content_x, content_y, content_width, _click_regions)
		3:
			end_y = _economy_tab.draw_content(self, _cached_data, font, content_x, content_y, content_width, _click_regions)

	_content_height = end_y + _scroll_offset + 40.0

	_draw_scrollbar()


func _draw_scrollbar() -> void:
	var content_top: float = HEADER_HEIGHT + TAB_BAR_HEIGHT
	var available_h: float = size.y - content_top
	if _content_height <= available_h + content_top:
		_scrollbar_rect = Rect2()
		return

	var panel_h: float = size.y
	var scrollbar_width: float = 6.0
	var scrollbar_margin: float = 3.0
	var scrollbar_x: float = size.x - scrollbar_width - scrollbar_margin
	var track_top: float = content_top + 4.0
	var track_bottom: float = panel_h - 4.0
	var track_height: float = track_bottom - track_top

	_scrollbar_rect = Rect2(scrollbar_x - 4.0, track_top, scrollbar_width + 8.0, track_height)

	draw_rect(Rect2(scrollbar_x, track_top, scrollbar_width, track_height), Color(0.15, 0.15, 0.15, 0.3))

	var visible_ratio: float = clampf(panel_h / _content_height, 0.05, 1.0)
	var grabber_height: float = maxf(track_height * visible_ratio, 20.0)
	var scroll_max: float = maxf(0.0, _content_height - panel_h)
	var scroll_ratio: float = clampf(_scroll_offset / scroll_max, 0.0, 1.0) if scroll_max > 0.0 else 0.0
	var grabber_y: float = track_top + (track_height - grabber_height) * scroll_ratio

	draw_rect(Rect2(scrollbar_x, grabber_y, scrollbar_width, grabber_height), Color(0.6, 0.6, 0.6, 0.5))


func _gui_input(event: InputEvent) -> void:
	# Scrollbar drag
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

	# Mouse wheel scroll
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			_scroll_offset = minf(_scroll_offset + 30.0, maxf(0.0, _content_height - size.y + 40.0))
			accept_event()
			return
		elif event.button_index == MOUSE_BUTTON_WHEEL_UP:
			_scroll_offset = maxf(_scroll_offset - 30.0, 0.0)
			accept_event()
			return

	# Trackpad pan
	if event is InputEventPanGesture:
		_scroll_offset += event.delta.y * 15.0
		_scroll_offset = clampf(_scroll_offset, 0.0, maxf(0.0, _content_height - size.y + 40.0))
		accept_event()
		return

	# Left-click hit testing
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		# Close button
		if _close_rect.has_point(event.position):
			hide()
			accept_event()
			return

		# Tab clicks
		for tab_entry in _tab_rects:
			if tab_entry.rect.has_point(event.position):
				_current_tab = tab_entry.tab_index
				_scroll_offset = 0.0
				accept_event()
				return

		# Tech tab toggle clicks (practitioner list expand/collapse)
		if _current_tab == 1 and _tech_tab.handle_click(event.position):
			_scroll_offset = _scroll_offset  # preserve scroll
			accept_event()
			return

		# Entity name clicks
		for region in _click_regions:
			if region.rect.has_point(event.position):
				SimulationBus.emit_signal("entity_selected", region.entity_id)
				accept_event()
				return


func _update_scroll_from_mouse(mouse_y: float) -> void:
	var track_top: float = _scrollbar_rect.position.y
	var track_height: float = _scrollbar_rect.size.y
	if track_height <= 0.0:
		return
	var ratio: float = clampf((mouse_y - track_top) / track_height, 0.0, 1.0)
	var scroll_max: float = maxf(0.0, _content_height - size.y + 40.0)
	_scroll_offset = ratio * scroll_max


## Returns a color for the given era string.
func _era_color(era: String) -> Color:
	match era:
		"stone": return Color(0.75, 0.7, 0.6)
		"bronze": return Color(0.9, 0.65, 0.2)
		"iron": return Color(0.6, 0.65, 0.75)
		"medieval": return Color(0.4, 0.7, 0.5)
		"renaissance": return Color(0.7, 0.5, 0.8)
		_: return Color(0.8, 0.8, 0.8)
