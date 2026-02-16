class_name StatsDetailPanel
extends Control

var _stats_recorder: RefCounted
var _settlement_manager: RefCounted
var _entity_manager: RefCounted
var _relationship_manager: RefCounted

const GRAPH_HEIGHT: float = 120.0
const SECTION_GAP: float = 10.0

## Scroll state
var _scroll_offset: float = 0.0
var _content_height: float = 0.0


func init(stats_recorder: RefCounted, settlement_manager: RefCounted = null, entity_manager: RefCounted = null, relationship_manager: RefCounted = null) -> void:
	_stats_recorder = stats_recorder
	_settlement_manager = settlement_manager
	_entity_manager = entity_manager
	_relationship_manager = relationship_manager


func _process(_delta: float) -> void:
	if visible:
		queue_redraw()


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			_scroll_offset = minf(_scroll_offset + 30.0, maxf(0.0, _content_height - size.y + 40.0))
			accept_event()
		elif event.button_index == MOUSE_BUTTON_WHEEL_UP:
			_scroll_offset = maxf(_scroll_offset - 30.0, 0.0)
			accept_event()


func _draw() -> void:
	if not visible or _stats_recorder == null:
		return

	var panel_w: float = size.x
	var panel_h: float = size.y

	draw_rect(Rect2(0, 0, panel_w, panel_h), Color(0.08, 0.08, 0.12, 0.95))
	draw_rect(Rect2(0, 0, panel_w, panel_h), Color(0.3, 0.3, 0.4), false, 1.0)

	var font: Font = ThemeDB.fallback_font
	var cx: float = 20.0
	var cy: float = 30.0 - _scroll_offset

	# Title
	draw_string(font, Vector2(cx, cy), "World Statistics", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_title"), Color.WHITE)
	cy += 15.0
	draw_line(Vector2(cx, cy), Vector2(panel_w - 20, cy), Color(0.3, 0.3, 0.4), 1.0)
	cy += 10.0

	var content_w: float = panel_w - 40.0
	var half_w: float = content_w * 0.5 - 10.0

	cy = _draw_population_section(font, cx, cy, content_w)
	cy += SECTION_GAP

	# Demographics section (couples, age distribution, happiness)
	if _entity_manager != null:
		cy = _draw_demographics_section(font, cx, cy, content_w)
		cy += SECTION_GAP

	cy = _draw_resource_section(font, cx, cy, content_w)
	cy += SECTION_GAP

	var jobs_y: float = cy
	_draw_jobs_section(font, cx, cy, half_w)
	_draw_settlements_section(font, cx + half_w + 20, cy, half_w)

	# Estimate the end of the two-column section
	var history: Array = _stats_recorder.history
	if not history.is_empty():
		var snap: Dictionary = history[history.size() - 1]
		var job_rows: int = 5
		var settlement_rows: int = 0
		if _settlement_manager != null:
			var active: Array = _settlement_manager.get_active_settlements()
			settlement_rows = active.size() * 2 + 1
		var max_rows: int = maxi(job_rows + 2, settlement_rows + 1)
		cy = jobs_y + 20.0 + float(max_rows) * 14.0

	# Track content height for scrolling
	_content_height = cy + _scroll_offset + 40.0

	# Footer hint
	draw_string(font, Vector2(panel_w * 0.5 - 60, panel_h - 12), "Scroll for more | Click background or G to close", HORIZONTAL_ALIGNMENT_CENTER, -1, GameConfig.get_font_size("popup_small"), Color(0.4, 0.4, 0.4))


func _draw_population_section(font: Font, x: float, y: float, w: float) -> float:
	draw_string(font, Vector2(x, y + 14), "Population", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color(0.2, 1.0, 0.4))
	y += 20.0

	var history: Array = _stats_recorder.history
	if history.size() < 2:
		draw_string(font, Vector2(x, y + 12), "Waiting for data...", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
		return y + 20.0

	var latest: Dictionary = history[history.size() - 1]
	var graph_rect := Rect2(x, y, w, GRAPH_HEIGHT)
	draw_rect(graph_rect, Color(0.05, 0.05, 0.08, 0.8))

	var max_pop: int = 1
	for i in range(history.size()):
		if history[i].pop > max_pop:
			max_pop = history[i].pop

	var points := PackedVector2Array()
	var count: int = history.size()
	for i in range(count):
		var px: float = x + float(i) / float(count - 1) * w
		var py: float = y + GRAPH_HEIGHT - (float(history[i].pop) / float(max_pop) * (GRAPH_HEIGHT - 5.0))
		points.append(Vector2(px, py))
	draw_polyline(points, Color(0.2, 1.0, 0.4), 2.0)

	y += GRAPH_HEIGHT + 5.0

	var pop_line: String = "Current: %d  |  Peak: %d  |  Deaths: %d  |  Births: %d" % [
		latest.pop, _stats_recorder.peak_pop,
		_stats_recorder.total_deaths, _stats_recorder.total_births,
	]
	draw_string(font, Vector2(x, y + 12), pop_line, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
	return y + 18.0


func _draw_demographics_section(font: Font, x: float, y: float, w: float) -> float:
	draw_string(font, Vector2(x, y + 14), "Demographics", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color(0.8, 0.5, 0.9))
	y += 20.0

	var alive: Array = _entity_manager.get_alive_entities()
	if alive.is_empty():
		draw_string(font, Vector2(x, y + 12), "No population", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
		return y + 20.0

	# Count age stages, couples, happiness
	var child_count: int = 0
	var teen_count: int = 0
	var adult_count: int = 0
	var elder_count: int = 0
	var couple_count: int = 0
	var single_adult_count: int = 0
	var total_happiness: float = 0.0
	var male_count: int = 0
	var female_count: int = 0

	for i in range(alive.size()):
		var e: RefCounted = alive[i]
		match e.age_stage:
			"infant", "toddler", "child":
				child_count += 1
			"teen":
				teen_count += 1
			"adult":
				adult_count += 1
			"elder", "ancient":
				elder_count += 1

		if e.gender == "male":
			male_count += 1
		else:
			female_count += 1

		total_happiness += e.emotions.get("happiness", 0.5)

		# Count couples (only count each pair once — count from one side)
		if e.partner_id >= 0 and e.id < e.partner_id:
			couple_count += 1
		if e.partner_id < 0 and (e.age_stage == "adult" or e.age_stage == "elder" or e.age_stage == "ancient"):
			single_adult_count += 1

	var total: int = alive.size()
	var avg_happiness: float = total_happiness / float(total) if total > 0 else 0.0

	# Gender line
	draw_string(font, Vector2(x, y + 12), "Gender: M:%d  F:%d" % [male_count, female_count], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.8))
	y += 16.0

	# Couples line
	draw_string(font, Vector2(x, y + 12), "Couples: %d  |  Single Adults: %d" % [couple_count, single_adult_count], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.9, 0.5, 0.6))
	y += 16.0

	# Average happiness bar
	draw_string(font, Vector2(x, y + 11), "Avg Happiness:", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))
	var bar_x: float = x + 120.0
	var bar_w: float = w - 180.0
	var bar_h: float = 10.0
	draw_rect(Rect2(bar_x, y + 2, bar_w, bar_h), Color(0.2, 0.2, 0.2, 0.8))
	var happy_color: Color = Color(0.9, 0.8, 0.2) if avg_happiness >= 0.4 else Color(0.9, 0.3, 0.2)
	draw_rect(Rect2(bar_x, y + 2, bar_w * clampf(avg_happiness, 0.0, 1.0), bar_h), happy_color)
	draw_string(font, Vector2(bar_x + bar_w + 5, y + 11), "%d%%" % int(avg_happiness * 100), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
	y += 18.0

	# Age distribution bar
	draw_string(font, Vector2(x, y + 12), "Age Distribution:", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.7, 0.7, 0.7))
	y += 16.0

	var age_bar_x: float = x
	var age_bar_w: float = w
	var age_bar_h: float = 16.0
	var age_data: Array = [
		{"label": "Child", "count": child_count, "color": Color(0.5, 0.8, 1.0)},
		{"label": "Teen", "count": teen_count, "color": Color(0.3, 0.7, 0.4)},
		{"label": "Adult", "count": adult_count, "color": Color(0.2, 0.5, 0.9)},
		{"label": "Elder", "count": elder_count, "color": Color(0.7, 0.5, 0.3)},
	]

	var bx: float = age_bar_x
	for idx in range(age_data.size()):
		var ad: Dictionary = age_data[idx]
		var aw: float = float(ad.count) / float(total) * age_bar_w if total > 0 else 0.0
		if aw > 0.0:
			draw_rect(Rect2(bx, y, aw, age_bar_h), ad.color)
		bx += aw

	y += age_bar_h + 6.0

	# Age legend
	for idx in range(age_data.size()):
		var ad: Dictionary = age_data[idx]
		var pct: int = int(float(ad.count) / float(total) * 100.0) if total > 0 else 0
		draw_string(font, Vector2(x + 4, y + 11), "%s: %d (%d%%)" % [ad.label, ad.count, pct], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), ad.color)
		y += 14.0

	return y


func _draw_resource_section(font: Font, x: float, y: float, w: float) -> float:
	draw_string(font, Vector2(x, y + 14), "Resources", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color(0.9, 0.8, 0.1))
	y += 20.0

	var history: Array = _stats_recorder.history
	if history.size() < 2:
		return y + 20.0

	var latest: Dictionary = history[history.size() - 1]
	var graph_rect := Rect2(x, y, w, GRAPH_HEIGHT)
	draw_rect(graph_rect, Color(0.05, 0.05, 0.08, 0.8))

	var max_res: float = 1.0
	for i in range(history.size()):
		var s: Dictionary = history[i]
		max_res = maxf(max_res, maxf(s.food, maxf(s.wood, s.stone)))

	var food_pts := PackedVector2Array()
	var wood_pts := PackedVector2Array()
	var stone_pts := PackedVector2Array()
	var count: int = history.size()
	for i in range(count):
		var px: float = x + float(i) / float(count - 1) * w
		var s: Dictionary = history[i]
		food_pts.append(Vector2(px, y + GRAPH_HEIGHT - (s.food / max_res * (GRAPH_HEIGHT - 5.0))))
		wood_pts.append(Vector2(px, y + GRAPH_HEIGHT - (s.wood / max_res * (GRAPH_HEIGHT - 5.0))))
		stone_pts.append(Vector2(px, y + GRAPH_HEIGHT - (s.stone / max_res * (GRAPH_HEIGHT - 5.0))))

	draw_polyline(food_pts, Color(0.9, 0.8, 0.1), 2.0)
	draw_polyline(wood_pts, Color(0.6, 0.4, 0.2), 2.0)
	draw_polyline(stone_pts, Color(0.7, 0.7, 0.7), 2.0)

	y += GRAPH_HEIGHT + 5.0

	var deltas: Dictionary = _stats_recorder.get_resource_deltas()
	var res_line: String = "Food: %d (%+.0f/100t)  |  Wood: %d (%+.0f/100t)  |  Stone: %d (%+.0f/100t)" % [
		int(latest.food), deltas.food, int(latest.wood), deltas.wood, int(latest.stone), deltas.stone,
	]
	draw_string(font, Vector2(x, y + 12), res_line, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
	return y + 18.0


func _draw_jobs_section(font: Font, x: float, y: float, w: float) -> void:
	draw_string(font, Vector2(x, y + 14), "Jobs", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color(0.5, 0.7, 1.0))
	y += 20.0

	var history: Array = _stats_recorder.history
	if history.is_empty():
		return
	var snap: Dictionary = history[history.size() - 1]
	var total: int = snap.pop
	if total <= 0:
		return

	# Job bar
	var bar_h: float = 16.0
	var bx: float = x
	var job_data: Array = [
		{"name": "Gatherer", "count": snap.gatherers, "color": Color(0.3, 0.8, 0.2)},
		{"name": "Lumberjack", "count": snap.lumberjacks, "color": Color(0.6, 0.35, 0.1)},
		{"name": "Builder", "count": snap.builders, "color": Color(0.9, 0.6, 0.1)},
		{"name": "Miner", "count": snap.miners, "color": Color(0.5, 0.6, 0.75)},
		{"name": "None", "count": snap.none_job, "color": Color(0.4, 0.4, 0.4)},
	]

	for idx in range(job_data.size()):
		var jd: Dictionary = job_data[idx]
		var jw: float = float(jd.count) / float(total) * w
		if jw > 0.0:
			draw_rect(Rect2(bx, y, jw, bar_h), jd.color)
		bx += jw

	y += bar_h + 6.0

	for idx in range(job_data.size()):
		var jd: Dictionary = job_data[idx]
		var pct: int = int(float(jd.count) / float(total) * 100.0)
		draw_string(font, Vector2(x + 4, y + 11), "%s: %d (%d%%)" % [jd.name, jd.count, pct], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), jd.color)
		y += 14.0


func _draw_settlements_section(font: Font, x: float, y: float, w: float) -> void:
	draw_string(font, Vector2(x, y + 14), "Settlements", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_heading"), Color(0.9, 0.6, 0.2))
	y += 20.0

	var settlements: Array = _stats_recorder.get_settlement_stats()
	if settlements.is_empty():
		draw_string(font, Vector2(x, y + 12), "No settlements", HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.5, 0.5, 0.5))
		return

	for i in range(settlements.size()):
		var s: Dictionary = settlements[i]
		draw_string(font, Vector2(x + 4, y + 11), "S%d: Pop %d, Bld %d" % [s.id, s.pop, s.buildings], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_body"), Color(0.8, 0.8, 0.8))
		y += 14.0
		draw_string(font, Vector2(x + 4, y + 11), "  F:%d W:%d S:%d" % [int(s.food), int(s.wood), int(s.stone)], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("popup_small"), Color(0.6, 0.6, 0.6))
		y += 14.0
