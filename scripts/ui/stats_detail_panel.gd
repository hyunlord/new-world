class_name StatsDetailPanel
extends Control

var _stats_recorder: RefCounted
var _settlement_manager: RefCounted
var _sim_engine: RefCounted
var _was_paused: bool = false

const GRAPH_HEIGHT: float = 120.0
const SECTION_GAP: float = 10.0


func init(stats_recorder: RefCounted, settlement_manager: RefCounted = null, sim_engine: RefCounted = null) -> void:
	_stats_recorder = stats_recorder
	_settlement_manager = settlement_manager
	_sim_engine = sim_engine


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP


func show_panel() -> void:
	if _sim_engine != null:
		_was_paused = _sim_engine.is_paused
		_sim_engine.is_paused = true
		SimulationBus.pause_changed.emit(true)
	visible = true


func hide_panel() -> void:
	visible = false
	if _sim_engine != null and not _was_paused:
		_sim_engine.is_paused = false
		SimulationBus.pause_changed.emit(false)


func _process(_delta: float) -> void:
	if visible:
		queue_redraw()


func _gui_input(event: InputEvent) -> void:
	if not visible:
		return
	if event is InputEventKey and event.pressed:
		if event.keycode == KEY_G or event.keycode == KEY_ESCAPE:
			hide_panel()
			accept_event()
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		var vp_size := get_viewport_rect().size
		var panel_w: float = vp_size.x * 0.75
		var panel_h: float = vp_size.y * 0.8
		var panel_x: float = (vp_size.x - panel_w) * 0.5
		var panel_y: float = (vp_size.y - panel_h) * 0.5
		var panel_rect := Rect2(panel_x, panel_y, panel_w, panel_h)
		var mb: InputEventMouseButton = event as InputEventMouseButton
		var close_area := Rect2(panel_x + panel_w - 30, panel_y + 5, 25, 25)
		if close_area.has_point(mb.position) or not panel_rect.has_point(mb.position):
			hide_panel()
			accept_event()
		else:
			accept_event()  # Consume click inside panel to prevent passthrough


func _draw() -> void:
	if not visible or _stats_recorder == null:
		return

	var vp_size := get_viewport_rect().size
	# Dim overlay
	draw_rect(Rect2(Vector2.ZERO, vp_size), Color(0, 0, 0, 0.7))

	# Panel area
	var panel_w: float = vp_size.x * 0.75
	var panel_h: float = vp_size.y * 0.8
	var panel_x: float = (vp_size.x - panel_w) * 0.5
	var panel_y: float = (vp_size.y - panel_h) * 0.5
	var panel_rect := Rect2(panel_x, panel_y, panel_w, panel_h)

	draw_rect(panel_rect, Color(0.08, 0.08, 0.12, 0.95))
	draw_rect(panel_rect, Color(0.3, 0.3, 0.4), false, 1.0)

	var font: Font = ThemeDB.fallback_font
	var cx: float = panel_x + 20.0
	var cy: float = panel_y + 30.0

	# Title
	draw_string(font, Vector2(cx, cy), "World Statistics", HORIZONTAL_ALIGNMENT_LEFT, -1, 22, Color.WHITE)
	draw_string(font, Vector2(panel_x + panel_w - 28, panel_y + 20), "[X]", HORIZONTAL_ALIGNMENT_LEFT, -1, 16, Color(0.7, 0.7, 0.7))
	cy += 15.0
	draw_line(Vector2(cx, cy), Vector2(panel_x + panel_w - 20, cy), Color(0.3, 0.3, 0.4), 1.0)
	cy += 10.0

	var content_w: float = panel_w - 40.0
	var half_w: float = content_w * 0.5 - 10.0

	cy = _draw_population_section(font, cx, cy, content_w)
	cy += SECTION_GAP
	cy = _draw_resource_section(font, cx, cy, content_w)
	cy += SECTION_GAP

	_draw_jobs_section(font, cx, cy, half_w)
	_draw_settlements_section(font, cx + half_w + 20, cy, half_w)

	# Footer
	draw_string(font, Vector2(vp_size.x * 0.5 - 30, panel_y + panel_h - 12), "[G: Close]", HORIZONTAL_ALIGNMENT_CENTER, -1, 14, Color(0.5, 0.5, 0.5))


func _draw_population_section(font: Font, x: float, y: float, w: float) -> float:
	draw_string(font, Vector2(x, y + 14), "Population", HORIZONTAL_ALIGNMENT_LEFT, -1, 16, Color(0.2, 1.0, 0.4))
	y += 20.0

	var history: Array = _stats_recorder.history
	if history.size() < 2:
		draw_string(font, Vector2(x, y + 12), "Waiting for data...", HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(0.5, 0.5, 0.5))
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
	draw_string(font, Vector2(x, y + 12), pop_line, HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(0.8, 0.8, 0.8))
	return y + 18.0


func _draw_resource_section(font: Font, x: float, y: float, w: float) -> float:
	draw_string(font, Vector2(x, y + 14), "Resources", HORIZONTAL_ALIGNMENT_LEFT, -1, 16, Color(0.9, 0.8, 0.1))
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
	draw_string(font, Vector2(x, y + 12), res_line, HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(0.8, 0.8, 0.8))
	return y + 18.0


func _draw_jobs_section(font: Font, x: float, y: float, w: float) -> void:
	draw_string(font, Vector2(x, y + 14), "Jobs", HORIZONTAL_ALIGNMENT_LEFT, -1, 16, Color(0.5, 0.7, 1.0))
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
		draw_string(font, Vector2(x + 4, y + 11), "%s: %d (%d%%)" % [jd.name, jd.count, pct], HORIZONTAL_ALIGNMENT_LEFT, -1, 13, jd.color)
		y += 14.0


func _draw_settlements_section(font: Font, x: float, y: float, w: float) -> void:
	draw_string(font, Vector2(x, y + 14), "Settlements", HORIZONTAL_ALIGNMENT_LEFT, -1, 16, Color(0.9, 0.6, 0.2))
	y += 20.0

	var settlements: Array = _stats_recorder.get_settlement_stats()
	if settlements.is_empty():
		draw_string(font, Vector2(x, y + 12), "No settlements", HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(0.5, 0.5, 0.5))
		return

	for i in range(settlements.size()):
		var s: Dictionary = settlements[i]
		draw_string(font, Vector2(x + 4, y + 11), "S%d: Pop %d, Bld %d" % [s.id, s.pop, s.buildings], HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(0.8, 0.8, 0.8))
		y += 14.0
		draw_string(font, Vector2(x + 4, y + 11), "  F:%d W:%d S:%d" % [int(s.food), int(s.wood), int(s.stone)], HORIZONTAL_ALIGNMENT_LEFT, -1, 13, Color(0.6, 0.6, 0.6))
		y += 14.0
