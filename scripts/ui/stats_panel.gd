class_name StatsPanel
extends Control

var _stats_recorder: RefCounted

const PANEL_W: int = 160
const PANEL_H: int = 200
const GRAPH_H: int = 55
const BAR_H: int = 12


func init(stats_recorder: RefCounted) -> void:
	_stats_recorder = stats_recorder


func _ready() -> void:
	set_anchors_preset(Control.PRESET_TOP_RIGHT)
	offset_right = -10
	offset_left = -(10 + PANEL_W)
	offset_top = 38 + 160 + 4
	offset_bottom = 38 + 160 + 4 + PANEL_H
	custom_minimum_size = Vector2(PANEL_W, PANEL_H)
	mouse_filter = Control.MOUSE_FILTER_IGNORE


func _process(_delta: float) -> void:
	queue_redraw()


func _draw() -> void:
	# Background
	draw_rect(Rect2(Vector2.ZERO, Vector2(PANEL_W, PANEL_H)), Color(0, 0, 0, 0.7))

	var font: Font = ThemeDB.fallback_font

	if _stats_recorder == null or _stats_recorder.history.size() < 2:
		draw_string(font, Vector2(4, 14), "Population", HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color.WHITE)
		draw_string(font, Vector2(4, GRAPH_H + 18), "Resources", HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color.WHITE)
		return

	_draw_population_graph(font)
	_draw_resource_graph(font)
	_draw_job_distribution(font)


func _draw_population_graph(font: Font) -> void:
	var rect := Rect2(0, 0, PANEL_W, GRAPH_H)
	draw_rect(rect, Color(0.1, 0.1, 0.1, 0.5))
	draw_string(font, Vector2(4, 12), "Population", HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color.WHITE)

	var history: Array = _stats_recorder.history
	var max_pop: int = 1
	for i in range(history.size()):
		var p: int = history[i].pop
		if p > max_pop:
			max_pop = p

	draw_string(font, Vector2(PANEL_W - 30, 12), str(max_pop), HORIZONTAL_ALIGNMENT_RIGHT, -1, 9, Color(0.5, 0.5, 0.5))

	var points := PackedVector2Array()
	var count: int = history.size()
	for i in range(count):
		var x: float = float(i) / float(count - 1) * rect.size.x
		var y: float = rect.size.y - (float(history[i].pop) / float(max_pop) * (rect.size.y - 15.0))
		points.append(Vector2(x, y))

	draw_polyline(points, Color(0.2, 1.0, 0.4), 1.5)


func _draw_resource_graph(font: Font) -> void:
	var y_off: float = GRAPH_H + 4
	var rect := Rect2(0, y_off, PANEL_W, GRAPH_H)
	draw_rect(rect, Color(0.1, 0.1, 0.1, 0.5))
	draw_string(font, Vector2(4, y_off + 12), "Resources", HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color.WHITE)

	var history: Array = _stats_recorder.history
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
	draw_circle(Vector2(PANEL_W - 50, y_off + 10), 3, Color(0.9, 0.8, 0.1))
	draw_circle(Vector2(PANEL_W - 34, y_off + 10), 3, Color(0.6, 0.4, 0.2))
	draw_circle(Vector2(PANEL_W - 18, y_off + 10), 3, Color(0.7, 0.7, 0.7))


func _draw_job_distribution(font: Font) -> void:
	var y_off: float = GRAPH_H * 2 + 8
	draw_string(font, Vector2(4, y_off + 12), "Jobs", HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color.WHITE)

	var history: Array = _stats_recorder.history
	if history.is_empty():
		return

	var snap: Dictionary = history[history.size() - 1]
	var total: float = float(snap.pop)
	if total <= 0.0:
		return

	var bar_y: float = y_off + 16
	var x: float = 0.0
	var pw: float = float(PANEL_W)

	var gw: float = float(snap.gatherers) / total * pw
	if gw > 0.0:
		draw_rect(Rect2(x, bar_y, gw, BAR_H), Color(0.3, 0.8, 0.2))
	x += gw

	var lw: float = float(snap.lumberjacks) / total * pw
	if lw > 0.0:
		draw_rect(Rect2(x, bar_y, lw, BAR_H), Color(0.6, 0.35, 0.1))
	x += lw

	var bw: float = float(snap.builders) / total * pw
	if bw > 0.0:
		draw_rect(Rect2(x, bar_y, bw, BAR_H), Color(0.9, 0.6, 0.1))
	x += bw

	var mw: float = float(snap.miners) / total * pw
	if mw > 0.0:
		draw_rect(Rect2(x, bar_y, mw, BAR_H), Color(0.5, 0.6, 0.75))
	x += mw

	var nw: float = float(snap.none_job) / total * pw
	if nw > 0.0:
		draw_rect(Rect2(x, bar_y, nw, BAR_H), Color(0.4, 0.4, 0.4))

	var label_y: float = bar_y + BAR_H + 10
	draw_string(font, Vector2(4, label_y), "G:%d L:%d B:%d M:%d" % [snap.gatherers, snap.lumberjacks, snap.builders, snap.miners], HORIZONTAL_ALIGNMENT_LEFT, -1, 9, Color(0.7, 0.7, 0.7))
