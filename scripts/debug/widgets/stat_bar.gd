class_name StatBar
extends Control

## Horizontal bar widget: label + colored fill + value text.
## Usage: set label_text, value, max_value, bar_color, then queue_redraw().

var label_text: String = ""
var value: float = 0.0
var max_value: float = 1.0
var bar_color: Color = Color.GREEN
var show_value_text: bool = true

const _MIN_SIZE := Vector2(200, 20)


func _init() -> void:
	custom_minimum_size = _MIN_SIZE


func set_data(label: String, val: float, max_val: float, color: Color) -> void:
	label_text = label
	value = val
	max_value = max_val
	bar_color = color
	queue_redraw()


func _draw() -> void:
	var w: float = size.x
	var h: float = size.y
	var fill_w: float = 0.0
	if max_value > 0.0:
		fill_w = clampf((value / max_value) * w, 0.0, w)

	# Background
	draw_rect(Rect2(0.0, 0.0, w, h), Color(0.15, 0.15, 0.15))
	# Fill
	if fill_w > 0.0:
		draw_rect(Rect2(0.0, 0.0, fill_w, h), bar_color)
	# Label
	draw_string(ThemeDB.fallback_font, Vector2(4.0, h - 4.0),
		label_text, HORIZONTAL_ALIGNMENT_LEFT, -1, 11, Color.WHITE)
	# Value
	if show_value_text:
		var val_str: String = "%.2fms" % value
		draw_string(ThemeDB.fallback_font, Vector2(w - 56.0, h - 4.0),
			val_str, HORIZONTAL_ALIGNMENT_RIGHT, -1, 10, Color(0.7, 0.7, 0.7))
