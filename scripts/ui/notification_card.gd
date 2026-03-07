extends PanelContainer
class_name NotificationCard

const GameCalendar = preload("res://scripts/core/simulation/game_calendar.gd")

signal clicked(entity_id: int, position: Vector2)

const TIER_COLORS: Array[Color] = [
	Color("#B71C1C"),
	Color("#F44336"),
	Color("#CDDC39"),
	Color("#78909C"),
]

var _entity_id: int = -1
var _position: Vector2 = Vector2.ZERO
var _notification_id: int = 0

@onready var _tier_pip: ColorRect = get_node("TierPip") as ColorRect
@onready var _message_label: RichTextLabel = get_node("VBox/MessageLabel") as RichTextLabel
@onready var _time_label: Label = get_node("VBox/TimeLabel") as Label


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_STOP
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.07, 0.09, 0.13, 0.94)
	style.border_color = Color(0.32, 0.36, 0.44, 0.6)
	style.border_width_left = 1
	style.border_width_top = 1
	style.border_width_right = 1
	style.border_width_bottom = 1
	style.corner_radius_top_left = 6
	style.corner_radius_top_right = 6
	style.corner_radius_bottom_left = 6
	style.corner_radius_bottom_right = 6
	add_theme_stylebox_override("panel", style)


func setup(notif_data: Dictionary) -> void:
	_notification_id = int(notif_data.get("tick", 0))
	_entity_id = int(notif_data.get("primary_entity", -1))
	_position = Vector2(
		float(notif_data.get("position_x", 0.0)),
		float(notif_data.get("position_y", 0.0))
	)
	var tier: int = clampi(int(notif_data.get("tier", 1)), 0, TIER_COLORS.size() - 1)
	_tier_pip.color = TIER_COLORS[tier]
	_message_label.add_theme_font_size_override("normal_font_size", GameConfig.get_font_size("popup_body"))
	_message_label.add_theme_color_override("default_color", Color(0.94, 0.95, 0.98))
	_time_label.add_theme_font_size_override("font_size", GameConfig.get_font_size("popup_small"))
	_time_label.add_theme_color_override("font_color", Color(0.65, 0.7, 0.78))
	_message_label.text = str(notif_data.get("message", ""))
	_time_label.text = GameCalendar.format_short_datetime(int(notif_data.get("tick", 0)))
	visible = true


func reset() -> void:
	_notification_id = 0
	_entity_id = -1
	_position = Vector2.ZERO
	_message_label.text = ""
	_time_label.text = ""
	visible = false


func notification_id() -> int:
	return _notification_id


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		clicked.emit(_entity_id, _position)
		accept_event()
