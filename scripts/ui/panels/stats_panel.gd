class_name StatsPanel
extends Control

var _stats_recorder: RefCounted

var panel_w: int = 250
var panel_h: int = 220
var graph_h: int = 70
var bar_h: int = 14


func init(stats_recorder: RefCounted) -> void:
	_stats_recorder = stats_recorder


func _ready() -> void:
	set_anchors_preset(Control.PRESET_BOTTOM_RIGHT)
	offset_right = -10
	offset_left = -(10 + panel_w)
	offset_bottom = -30  # Above key hints
	offset_top = -(30 + panel_h)
	custom_minimum_size = Vector2(panel_w, panel_h)
	mouse_filter = Control.MOUSE_FILTER_STOP
	Locale.locale_changed.connect(func(_l): queue_redraw())


func _process(_delta: float) -> void:
	queue_redraw()


func _draw() -> void:
	# Background
	draw_rect(Rect2(Vector2.ZERO, Vector2(panel_w, panel_h)), Color(0, 0, 0, 0.7))

	var font: Font = ThemeDB.fallback_font

	if _stats_recorder == null or _stats_recorder.history.size() < 2:
		draw_string(font, Vector2(4, 14), Locale.ltr("UI_STAT_POPULATION"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("stats_title"), Color.WHITE)
		draw_string(font, Vector2(4, graph_h + 18), Locale.ltr("UI_RESOURCES"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("stats_title"), Color.WHITE)
		return

	_draw_population_graph(font)
	_draw_resource_graph(font)
	_draw_job_distribution(font)


func _draw_population_graph(font: Font) -> void:
	var rect := Rect2(0, 0, panel_w, graph_h)
	draw_rect(rect, Color(0.1, 0.1, 0.1, 0.5))

	var history: Array = _stats_recorder.history
	var latest: Dictionary = history[history.size() - 1]
	draw_string(font, Vector2(4, 12), Locale.ltr("UI_STAT_POPULATION") + ": %d" % latest.pop, HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("stats_title"), Color.WHITE)

	var max_pop: int = 1
	for i in range(history.size()):
		var p: int = history[i].pop
		if p > max_pop:
			max_pop = p

	draw_string(font, Vector2(panel_w - 4, 12), str(max_pop), HORIZONTAL_ALIGNMENT_RIGHT, -1, GameConfig.get_font_size("stats_body"), Color(0.5, 0.5, 0.5))

	var points := PackedVector2Array()
	var count: int = history.size()
	for i in range(count):
		var x: float = float(i) / float(count - 1) * rect.size.x
		var y: float = rect.size.y - (float(history[i].pop) / float(max_pop) * (rect.size.y - 15.0))
		points.append(Vector2(x, y))

	draw_polyline(points, Color(0.2, 1.0, 0.4), 1.5)


func _draw_resource_graph(font: Font) -> void:
	var y_off: float = graph_h + 4
	var rect := Rect2(0, y_off, panel_w, graph_h)
	draw_rect(rect, Color(0.1, 0.1, 0.1, 0.5))

	var history: Array = _stats_recorder.history
	var latest: Dictionary = history[history.size() - 1]
	draw_string(font, Vector2(4, y_off + 12), "F:%d W:%d S:%d" % [int(latest.food), int(latest.wood), int(latest.stone)], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("stats_body"), Color.WHITE)
	var max_res: float = 1.0
	for i in range(history.size()):
		var s: Dictionary = history[i]
		max_res = maxf(max_res, s.food)
		max_res = maxf(max_res, s.wood)
		max_res = maxf(max_res, s.stone)

	var food_pts := PackedVector2Array()
	var wood_pts := PackedVector2Array()
	var stone_pts := PackedVector2Array()
	var count: int = history.size()
	var graph_h: float = rect.size.y - 15.0

	for i in range(count):
		var x: float = float(i) / float(count - 1) * rect.size.x
		var s: Dictionary = history[i]
		food_pts.append(Vector2(x, y_off + rect.size.y - (s.food / max_res * graph_h)))
		wood_pts.append(Vector2(x, y_off + rect.size.y - (s.wood / max_res * graph_h)))
		stone_pts.append(Vector2(x, y_off + rect.size.y - (s.stone / max_res * graph_h)))

	draw_polyline(food_pts, Color(0.9, 0.8, 0.1), 1.5)
	draw_polyline(wood_pts, Color(0.6, 0.4, 0.2), 1.5)
	draw_polyline(stone_pts, Color(0.7, 0.7, 0.7), 1.5)

	# Legend dots
	draw_circle(Vector2(panel_w - 50, y_off + 10), 3, Color(0.9, 0.8, 0.1))
	draw_circle(Vector2(panel_w - 34, y_off + 10), 3, Color(0.6, 0.4, 0.2))
	draw_circle(Vector2(panel_w - 18, y_off + 10), 3, Color(0.7, 0.7, 0.7))


func _draw_job_distribution(font: Font) -> void:
	var y_off: float = graph_h * 2 + 8
	draw_string(font, Vector2(4, y_off + 12), Locale.ltr("UI_JOBS"), HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("stats_title"), Color.WHITE)

	var history: Array = _stats_recorder.history
	if history.is_empty():
		return

	var snap: Dictionary = history[history.size() - 1]
	var total: float = float(snap.pop)
	if total <= 0.0:
		return

	var bar_y: float = y_off + 16
	var x: float = 0.0
	var pw: float = float(panel_w)

	var gw: float = float(snap.gatherers) / total * pw
	if gw > 0.0:
		draw_rect(Rect2(x, bar_y, gw, bar_h), Color(0.3, 0.8, 0.2))
	x += gw

	var lw: float = float(snap.lumberjacks) / total * pw
	if lw > 0.0:
		draw_rect(Rect2(x, bar_y, lw, bar_h), Color(0.6, 0.35, 0.1))
	x += lw

	var bw: float = float(snap.builders) / total * pw
	if bw > 0.0:
		draw_rect(Rect2(x, bar_y, bw, bar_h), Color(0.9, 0.6, 0.1))
	x += bw

	var mw: float = float(snap.miners) / total * pw
	if mw > 0.0:
		draw_rect(Rect2(x, bar_y, mw, bar_h), Color(0.5, 0.6, 0.75))
	x += mw

	var nw: float = float(snap.none_job) / total * pw
	if nw > 0.0:
		draw_rect(Rect2(x, bar_y, nw, bar_h), Color(0.4, 0.4, 0.4))

	var label_y: float = bar_y + bar_h + 10
	draw_string(font, Vector2(4, label_y), "G:%d L:%d B:%d M:%d" % [snap.gatherers, snap.lumberjacks, snap.builders, snap.miners], HORIZONTAL_ALIGNMENT_LEFT, -1, GameConfig.get_font_size("stats_body"), Color(0.7, 0.7, 0.7))

	# Click hint
	draw_string(font, Vector2(panel_w * 0.5 - 20, panel_h - 4), Locale.ltr("UI_STAT_DETAILS_HINT"), HORIZONTAL_ALIGNMENT_CENTER, -1, GameConfig.get_font_size("stats_body"), Color(0.5, 0.5, 0.5))


func apply_ui_scale() -> void:
	panel_w = GameConfig.get_ui_size("mini_stats_width")
	panel_h = GameConfig.get_ui_size("mini_stats_height")
	graph_h = int(panel_h * 0.32)
	bar_h = int(14 * GameConfig.ui_scale)
	offset_right = -10
	offset_left = -(10 + panel_w)
	offset_bottom = -30
	offset_top = -(30 + panel_h)
	custom_minimum_size = Vector2(panel_w, panel_h)


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		SimulationBus.ui_notification.emit("open_stats_detail", "command")
		accept_event()
