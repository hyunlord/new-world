extends Control

const PopulationTab = preload("res://scripts/ui/panels/world_stats_tabs/world_stats_population_tab.gd")
const TechTab = preload("res://scripts/ui/panels/world_stats_tabs/world_stats_tech_tab.gd")
const ResourcesTab = preload("res://scripts/ui/panels/world_stats_tabs/world_stats_resources_tab.gd")
const SocialTab = preload("res://scripts/ui/panels/world_stats_tabs/world_stats_social_tab.gd")

## Manager references (injected via init())
var _stats_recorder: RefCounted
var _settlement_manager: RefCounted
var _entity_manager: RefCounted
var _relationship_manager: RefCounted
var _tech_tree_manager: RefCounted

## State
var _current_tab: int = 0  # 0=Population, 1=Tech, 2=Resources, 3=Social
var _cached_data: Dictionary = {}
var _refresh_counter: int = 0

## Tab instances
var _population_tab: RefCounted
var _tech_tab: RefCounted
var _resources_tab: RefCounted
var _social_tab: RefCounted

## Scroll state
var _scroll_offset: float = 0.0
var _content_height: float = 0.0
var _scrollbar_dragging: bool = false
var _scrollbar_rect: Rect2 = Rect2()

## Click regions
var _click_regions: Array = []  # [{rect: Rect2, action: String, id: int}]
var _tab_rects: Array = []      # [{rect: Rect2, tab_index: int}]
var _close_rect: Rect2 = Rect2()

## Style constants (consistent with settlement_detail_panel)
const PANEL_BG: Color = Color(0.08, 0.08, 0.12, 0.95)
const HEADER_BG: Color = Color(0.10, 0.10, 0.16, 1.0)
const TAB_ACTIVE: Color = Color(0.3, 0.5, 0.8)
const TAB_INACTIVE: Color = Color(0.3, 0.3, 0.35)
const SECTION_HEADER_COLOR: Color = Color(0.9, 0.85, 0.6)
const POSITIVE_COLOR: Color = Color(0.4, 0.8, 0.4)
const NEGATIVE_COLOR: Color = Color(0.8, 0.4, 0.4)
const NEUTRAL_COLOR: Color = Color(0.7, 0.7, 0.7)
const GOLD_COLOR: Color = Color(1.0, 0.85, 0.3)
const SEPARATOR_COLOR: Color = Color(0.3, 0.3, 0.4)
const HEADER_HEIGHT: float = 55.0
const TAB_BAR_HEIGHT: float = 35.0
const TAB_NAMES: Array = ["UI_TAB_POP", "UI_TAB_TECH", "UI_TAB_RESOURCES", "UI_TAB_SOCIAL"]


## Initializes the panel with manager references and creates tab instances.
func init(stats_recorder: RefCounted, settlement_manager: RefCounted = null, entity_manager: RefCounted = null, relationship_manager: RefCounted = null) -> void:
	_stats_recorder = stats_recorder
	_settlement_manager = settlement_manager
	_entity_manager = entity_manager
	_relationship_manager = relationship_manager
	_population_tab = PopulationTab.new()
	_tech_tab = TechTab.new()
	_resources_tab = ResourcesTab.new()
	_social_tab = SocialTab.new()


## Sets the TechTreeManager reference for tech section display.
func set_tech_tree_manager(ttm: RefCounted) -> void:
	_tech_tree_manager = ttm


func _ready() -> void:
	Locale.locale_changed.connect(func(_l): queue_redraw())


func _process(_delta: float) -> void:
	if not visible:
		return
	queue_redraw()
	_refresh_counter += 1
	if _refresh_counter >= GameConfig.STATS_PANEL_REFRESH_INTERVAL:
		_refresh_counter = 0
		_load_data()


## Rebuilds the cached world data from all managers.
func _load_data() -> void:
	if _stats_recorder == null:
		_cached_data = {}
		return

	var settlements: Array = []
	var settlement_summaries: Array = []
	var total_pop: int = 0
	var total_male: int = 0
	var total_female: int = 0
	var total_age_ticks: float = 0.0
	var age_dist: Dictionary = {"child": 0, "teen": 0, "adult": 0, "elder": 0}
	var total_happiness: float = 0.0
	var total_stress: float = 0.0

	if _settlement_manager != null:
		settlements = _settlement_manager.get_active_settlements()

	for settlement in settlements:
		var members: Array = []
		var s_adults: int = 0
		var s_children: int = 0
		var s_elders: int = 0
		var s_teens: int = 0
		var s_male: int = 0
		var s_female: int = 0
		var s_happiness: float = 0.0
		var s_stress: float = 0.0

		if _entity_manager != null:
			for member_id in settlement.member_ids:
				var entity = _entity_manager.get_entity(member_id)
				if entity == null or not entity.is_alive:
					continue
				members.append(entity)

				var age_years: float = float(entity.age) / float(GameConfig.TICKS_PER_YEAR)
				total_age_ticks += entity.age

				if age_years < 12.0:
					s_children += 1
					age_dist["child"] += 1
				elif age_years < 18.0:
					s_teens += 1
					age_dist["teen"] += 1
				elif age_years < 55.0:
					s_adults += 1
					age_dist["adult"] += 1
				else:
					s_elders += 1
					age_dist["elder"] += 1

				if entity.gender == "male":
					s_male += 1
					total_male += 1
				else:
					s_female += 1
					total_female += 1

				s_happiness += entity.emotions.get("happiness", 0.5)
				s_stress += entity.emotions.get("stress", 0.0)
				total_happiness += entity.emotions.get("happiness", 0.5)
				total_stress += entity.emotions.get("stress", 0.0)

		var s_pop: int = members.size()
		total_pop += s_pop
		var s_avg_happiness: float = s_happiness / float(s_pop) if s_pop > 0 else 0.0
		var s_avg_stress: float = s_stress / float(s_pop) if s_pop > 0 else 0.0

		var leader = null
		if _entity_manager != null and settlement.leader_id > -1:
			leader = _entity_manager.get_entity(settlement.leader_id)

		# Get settlement resource data from stats_recorder
		var s_food: float = 0.0
		var s_wood: float = 0.0
		var s_stone: float = 0.0
		var s_stats: Array = _stats_recorder.get_settlement_stats()
		for ss in s_stats:
			if ss.id == settlement.id:
				s_food = ss.get("food", 0.0)
				s_wood = ss.get("wood", 0.0)
				s_stone = ss.get("stone", 0.0)
				break

		settlement_summaries.append({
			"id": settlement.id,
			"settlement": settlement,
			"pop": s_pop,
			"adults": s_adults,
			"children": s_children,
			"elders": s_elders,
			"teens": s_teens,
			"male": s_male,
			"female": s_female,
			"avg_happiness": s_avg_happiness,
			"avg_stress": s_avg_stress,
			"leader": leader,
			"tech_era": settlement.tech_era,
			"food": s_food,
			"wood": s_wood,
			"stone": s_stone,
		})

	# Recent births/deaths from history
	var total_births_recent: int = 0
	var total_deaths_recent: int = 0
	var history: Array = _stats_recorder.history
	if history.size() >= 2:
		var _latest: Dictionary = history[history.size() - 1]
		total_births_recent = _stats_recorder.total_births
		total_deaths_recent = _stats_recorder.total_deaths

	var avg_age_years: float = (total_age_ticks / float(total_pop) / float(GameConfig.TICKS_PER_YEAR)) if total_pop > 0 else 0.0

	# Global resource totals from latest history snapshot
	var global_food: float = 0.0
	var global_wood: float = 0.0
	var global_stone: float = 0.0
	var resource_deltas: Dictionary = {"food": 0.0, "wood": 0.0, "stone": 0.0}
	if not history.is_empty():
		var latest: Dictionary = history[history.size() - 1]
		global_food = latest.get("food", 0.0)
		global_wood = latest.get("wood", 0.0)
		global_stone = latest.get("stone", 0.0)
		resource_deltas = _stats_recorder.get_resource_deltas()

	_cached_data = {
		"settlements": settlements,
		"settlement_summaries": settlement_summaries,
		"total_population": total_pop,
		"total_births": total_births_recent,
		"total_deaths": total_deaths_recent,
		"age_distribution": age_dist,
		"total_male": total_male,
		"total_female": total_female,
		"avg_age_years": avg_age_years,
		"avg_happiness": total_happiness / float(total_pop) if total_pop > 0 else 0.0,
		"avg_stress": total_stress / float(total_pop) if total_pop > 0 else 0.0,
		"global_food": global_food,
		"global_wood": global_wood,
		"global_stone": global_stone,
		"resource_deltas": resource_deltas,
		"peak_pop": _stats_recorder.peak_pop,
		"history": history,
		"tech_tree_manager": _tech_tree_manager,
		"entity_manager": _entity_manager,
		"settlement_manager": _settlement_manager,
		"relationship_manager": _relationship_manager,
		"stats_recorder": _stats_recorder,
	}


func _draw() -> void:
	if not visible or _stats_recorder == null:
		return

	# Force initial load
	if _cached_data.is_empty():
		_load_data()
		if _cached_data.is_empty():
			return

	var panel_w: float = size.x
	var panel_h: float = size.y
	var font: Font = ThemeDB.fallback_font

	# Panel background and border
	draw_rect(Rect2(0.0, 0.0, panel_w, panel_h), PANEL_BG)
	draw_rect(Rect2(0.0, 0.0, panel_w, panel_h), SEPARATOR_COLOR, false, 1.0)

	_click_regions.clear()
	_tab_rects.clear()

	# Header background
	draw_rect(Rect2(0.0, 0.0, panel_w, HEADER_HEIGHT), HEADER_BG)

	# Title
	draw_string(font, Vector2(20.0, 28.0), Locale.ltr("UI_WORLD_STATISTICS"),
		HORIZONTAL_ALIGNMENT_LEFT, -1,
		GameConfig.get_font_size("popup_title"), Color.WHITE)

	# Settlement count
	var settlement_count: int = _cached_data.get("settlement_summaries", []).size()
	var settlement_count_text: String = Locale.trf1("UI_SETTLEMENT_COUNT_FMT", "n", settlement_count)
	draw_string(font, Vector2(20.0, 47.0),
		settlement_count_text,
		HORIZONTAL_ALIGNMENT_LEFT, -1,
		GameConfig.get_font_size("popup_small"), NEUTRAL_COLOR)

	# Total population in header (right side)
	var pop_text: String = Locale.ltr("UI_TOTAL_POP") + ": " + str(_cached_data.get("total_population", 0))
	draw_string(font, Vector2(panel_w - 180.0, 28.0), pop_text,
		HORIZONTAL_ALIGNMENT_LEFT, -1,
		GameConfig.get_font_size("popup_body"), GOLD_COLOR)

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
			end_y = _population_tab.draw_content(self, _cached_data, font, content_x, content_y, content_width, _click_regions)
		1:
			end_y = _tech_tab.draw_content(self, _cached_data, font, content_x, content_y, content_width, _click_regions)
		2:
			end_y = _resources_tab.draw_content(self, _cached_data, font, content_x, content_y, content_width, _click_regions)
		3:
			end_y = _social_tab.draw_content(self, _cached_data, font, content_x, content_y, content_width, _click_regions)

	_content_height = end_y + _scroll_offset + 40.0

	# Footer
	var footer_text: String = settlement_count_text
	draw_string(font, Vector2(panel_w * 0.5 - 40.0, panel_h - 12.0), footer_text,
		HORIZONTAL_ALIGNMENT_CENTER, -1,
		GameConfig.get_font_size("popup_small"), Color(0.4, 0.4, 0.4))

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

		# Click regions (settlement clicks, entity clicks)
		for region in _click_regions:
			if region.rect.has_point(event.position):
				var action: String = region.get("action", "")
				if action == "open_settlement":
					SimulationBus.emit_signal("ui_notification", "open_settlement_%d" % region.id, "panel")
				elif action == "open_entity":
					SimulationBus.emit_signal("entity_selected", region.id)
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
