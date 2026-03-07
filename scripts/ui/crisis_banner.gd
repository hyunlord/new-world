extends PanelContainer
class_name CrisisBanner

signal activated(entity_id: int, position: Vector2)
signal dismissed(notification_id: int)

var _entity_id: int = -1
var _position: Vector2 = Vector2.ZERO
var _notification_id: int = 0

@onready var _message_label: Label = get_node("HBox/MessageLabel") as Label
@onready var _hint_label: Label = get_node("HBox/HintLabel") as Label
@onready var _dismiss_button: Button = get_node("HBox/DismissButton") as Button


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_STOP
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.46, 0.08, 0.08, 0.96)
	style.border_color = Color(0.95, 0.58, 0.58, 0.7)
	style.border_width_left = 1
	style.border_width_top = 1
	style.border_width_right = 1
	style.border_width_bottom = 1
	style.content_margin_left = 8
	style.content_margin_right = 8
	style.content_margin_top = 8
	style.content_margin_bottom = 8
	add_theme_stylebox_override("panel", style)
	_message_label.add_theme_font_size_override("font_size", GameConfig.get_font_size("popup_body"))
	_message_label.add_theme_color_override("font_color", Color(1.0, 0.97, 0.97))
	_hint_label.add_theme_font_size_override("font_size", GameConfig.get_font_size("popup_small"))
	_hint_label.add_theme_color_override("font_color", Color(1.0, 0.86, 0.86))
	_dismiss_button.pressed.connect(_on_dismiss_pressed)
	refresh_locale()


func show_notification(notif_data: Dictionary) -> void:
	_notification_id = int(notif_data.get("tick", 0))
	_entity_id = int(notif_data.get("primary_entity", -1))
	_position = Vector2(
		float(notif_data.get("position_x", 0.0)),
		float(notif_data.get("position_y", 0.0))
	)
	_message_label.text = str(notif_data.get("message", ""))
	refresh_locale()
	visible = true


func notification_id() -> int:
	return _notification_id


func refresh_locale() -> void:
	if _dismiss_button != null:
		_dismiss_button.text = Locale.ltr("UI_NOTIF_DISMISS")
	if _hint_label != null:
		_hint_label.text = Locale.ltr("UI_CRISIS_BANNER_DISMISS")


func hide_banner() -> void:
	visible = false


func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		activated.emit(_entity_id, _position)
		accept_event()


func _on_dismiss_pressed() -> void:
	visible = false
	dismissed.emit(_notification_id)
