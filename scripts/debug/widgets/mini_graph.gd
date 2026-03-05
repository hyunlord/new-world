class_name DebugMiniGraph
extends Control

## Sparkline graph for debug panels. Draws PackedFloat32Array as a line graph.
## Used in perf_panel for tick history and ffi_monitor for bandwidth history.

var _data: PackedFloat32Array = PackedFloat32Array()
var _max_value: float = 16.0  ## Expected max (e.g. 16ms for 60fps target)
var _line_color: Color = Color(0.3, 0.8, 0.4)
var _bg_color: Color = Color(0.05, 0.05, 0.05, 0.8)
var _target_color: Color = Color(0.8, 0.3, 0.3, 0.5)

func _ready() -> void:
	custom_minimum_size = Vector2(0, 48)
	size_flags_horizontal = Control.SIZE_EXPAND_FILL


## Update graph data and trigger redraw.
func set_data(data: PackedFloat32Array, max_val: float = 16.0) -> void:
	_data = data
	_max_value = max(max_val, 0.001)
	queue_redraw()


## Set display colors.
func set_colors(line: Color, bg: Color = Color(0.05, 0.05, 0.05, 0.8), target: Color = Color(0.8, 0.3, 0.3, 0.5)) -> void:
	_line_color = line
	_bg_color = bg
	_target_color = target
	queue_redraw()


func _draw() -> void:
	var rect := Rect2(Vector2.ZERO, size)

	# Background
	draw_rect(rect, _bg_color)

	# Target line (16ms = 60fps budget)
	var target_y: float = rect.size.y * (1.0 - clampf(_max_value / max(_max_value * 1.2, 0.001), 0.0, 1.0))
	draw_line(
		Vector2(0, target_y),
		Vector2(rect.size.x, target_y),
		_target_color,
		1.0
	)

	if _data.is_empty():
		return

	var count: int = _data.size()
	var step: float = rect.size.x / max(count - 1, 1)

	var points: PackedVector2Array = PackedVector2Array()
	for i: int in range(count):
		var norm: float = clampf(_data[i] / _max_value, 0.0, 1.5)
		var px: float = i * step
		var py: float = rect.size.y * (1.0 - norm)
		points.append(Vector2(px, py))

	# Draw polyline
	for i: int in range(points.size() - 1):
		var col: Color = Color(0.9, 0.3, 0.3) if _data[i] > _max_value else _line_color
		draw_line(points[i], points[i + 1], col, 1.5)
