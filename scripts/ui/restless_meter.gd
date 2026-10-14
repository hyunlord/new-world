extends Control
## Three equal presentation segments, driven only by the bridge pressure ratio.
const SIZE := Vector2(26, 6)
var state: Dictionary = {}
var color: Color = Color.WHITE
func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	custom_minimum_size = SIZE
	size = SIZE
func display(value: Dictionary, village_color: Color) -> void:
	state = value.duplicate(true)
	color = village_color
	visible = bool(state.get("visible", false))
	queue_redraw()
func _draw() -> void:
	var ratio: float = clampf(float(state.get("ratio", 0.0)), 0.0, 1.0)
	for index: int in 3:
		var segment := Rect2(Vector2(index * 9, 0), Vector2(8, 6))
		draw_rect(segment, Color("352b23"))
		draw_rect(segment.grow(-1), Color(color, 0.22))
		var fraction: float = clampf(ratio * 3.0 - index, 0.0, 1.0)
		if fraction > 0.0:
			draw_rect(Rect2(segment.position + Vector2.ONE, Vector2(6.0 * fraction, 4)), color)
func capture_metadata() -> Dictionary:
	var result: Dictionary = state.duplicate(true)
	var rect: Rect2 = get_global_transform_with_canvas() * Rect2(Vector2.ZERO, SIZE)
	result["visible"] = is_visible_in_tree()
	result["color"] = color.to_html(false)
	result["screen_rect"] = [rect.position.x, rect.position.y, rect.size.x, rect.size.y]
	return result
