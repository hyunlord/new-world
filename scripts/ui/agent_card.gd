extends PanelContainer
class_name AgentCard

signal selected(entity_id: int)
signal follow_requested(entity_id: int)
signal pin_toggled(entity_id: int, is_pinned: bool)

const CARD_SIZE: Vector2 = Vector2(44.0, 44.0)
const CARD_CENTER: Vector2 = Vector2(22.0, 22.0)
const RING_RADIUS: float = 18.0
const INNER_RADIUS: float = 14.0
const ACTION_RECT: Rect2 = Rect2(30.0, 30.0, 10.0, 10.0)
const MOOD_COLORS: Array[Color] = [
	Color("#B71C1C"),
	Color("#F44336"),
	Color("#FF9800"),
	Color("#CDDC39"),
	Color("#4CAF50"),
]
const ACTION_COLORS: Array[Color] = [
	Color(0.35, 0.35, 0.4),
	Color(0.45, 0.7, 1.0),
	Color(1.0, 0.75, 0.2),
	Color(0.6, 0.9, 0.45),
	Color(0.85, 0.45, 0.9),
]
const JOB_COLORS: Dictionary = {
	0: Color(0.45, 0.45, 0.48),
	1: Color(0.35, 0.72, 0.28),
	2: Color(0.55, 0.35, 0.18),
	3: Color(0.86, 0.64, 0.2),
	4: Color(0.55, 0.62, 0.78),
}
const SEX_TINTS: Dictionary = {
	0: Color(0.35, 0.45, 0.9),
	1: Color(0.92, 0.42, 0.58),
}

var entity_id: int = -1
var agent_name: String = ""
var mood_color_idx: int = 2
var stress_phase: int = 0
var action_state: int = 0
var is_in_break: bool = false
var is_pinned: bool = false
var is_selected: bool = false
var job_icon: int = 0
var sex: int = 0


func _ready() -> void:
	custom_minimum_size = CARD_SIZE
	size = CARD_SIZE
	mouse_filter = Control.MOUSE_FILTER_STOP


func apply_state(state: Dictionary) -> void:
	entity_id = int(state.get("entity_id", -1))
	agent_name = str(state.get("name", ""))
	mood_color_idx = clampi(int(state.get("mood_color_idx", 2)), 0, 4)
	stress_phase = clampi(int(state.get("stress_phase", 0)), 0, 4)
	action_state = max(int(state.get("action_state", 0)), 0)
	is_in_break = bool(state.get("is_in_break", false))
	is_pinned = bool(state.get("is_pinned", false))
	is_selected = bool(state.get("is_selected", false))
	job_icon = int(state.get("job_icon", 0))
	sex = int(state.get("sex", 0))
	tooltip_text = str(state.get("tooltip_text", ""))
	visible = entity_id >= 0
	queue_redraw()


func _process(_delta: float) -> void:
	if stress_phase >= 2 or is_in_break:
		queue_redraw()


func _draw() -> void:
	var pulse_strength: float = 0.0
	if stress_phase >= 2 or is_in_break:
		pulse_strength = 0.4 + 0.3 * sin(float(Time.get_ticks_msec()) / 140.0)

	var background_color: Color = Color(0.10, 0.10, 0.14, 0.96)
	if is_selected:
		background_color = background_color.lerp(Color(0.18, 0.24, 0.34, 0.98), 0.65)
	draw_rect(Rect2(Vector2.ZERO, CARD_SIZE), background_color, true)
	draw_rect(Rect2(Vector2.ZERO, CARD_SIZE), Color(1.0, 1.0, 1.0, 0.08), false, 1.0)

	var ring_color: Color = MOOD_COLORS[mood_color_idx]
	if is_in_break:
		var flash_phase: float = fmod(float(Time.get_ticks_msec()) / 125.0, 2.0)
		ring_color = Color.RED if flash_phase < 1.0 else Color.WHITE
	elif stress_phase >= 2:
		ring_color = ring_color.lerp(Color(1.0, 0.3, 0.2), minf(0.6, pulse_strength))
	draw_arc(CARD_CENTER, RING_RADIUS, 0.0, TAU, 32, ring_color, 4.0)

	var base_inner: Color = JOB_COLORS.get(job_icon, JOB_COLORS[0])
	var tint: Color = SEX_TINTS.get(sex, SEX_TINTS[0])
	var inner_color: Color = base_inner.lerp(tint, 0.22)
	draw_circle(CARD_CENTER, INNER_RADIUS, inner_color)
	draw_circle(CARD_CENTER, INNER_RADIUS - 5.0, inner_color.darkened(0.18))

	var action_color: Color = ACTION_COLORS[min(action_state, ACTION_COLORS.size() - 1)]
	if action_state == 0:
		action_color = action_color.lightened(pulse_strength * 0.15)
	else:
		action_color = action_color.lightened(pulse_strength * 0.25)
	draw_rect(ACTION_RECT, action_color, true)
	draw_rect(ACTION_RECT, Color(1.0, 1.0, 1.0, 0.15), false, 1.0)

	if stress_phase >= 2:
		var stress_alpha: float = clampf(0.18 + pulse_strength * 0.35, 0.0, 0.8)
		draw_rect(
			Rect2(Vector2(2.0, 2.0), CARD_SIZE - Vector2(4.0, 4.0)),
			Color(1.0, 0.3, 0.2, stress_alpha),
			false,
			2.0
		)

	if is_pinned:
		var font: Font = ThemeDB.fallback_font
		if font != null:
			draw_string(
				font,
				Vector2(3.0, 12.0),
				"★",
				HORIZONTAL_ALIGNMENT_LEFT,
				-1.0,
				ThemeDB.fallback_font_size,
				Color(1.0, 0.9, 0.35)
			)

	if is_selected:
		draw_rect(
			Rect2(Vector2(1.0, 1.0), CARD_SIZE - Vector2(2.0, 2.0)),
			Color(0.5, 0.8, 1.0, 0.95),
			false,
			2.0
		)


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_LEFT:
			if event.double_click:
				follow_requested.emit(entity_id)
			else:
				selected.emit(entity_id)
			accept_event()
		elif event.button_index == MOUSE_BUTTON_RIGHT:
			is_pinned = not is_pinned
			pin_toggled.emit(entity_id, is_pinned)
			queue_redraw()
			accept_event()
