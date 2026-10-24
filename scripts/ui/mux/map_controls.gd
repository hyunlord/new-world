# map_controls.gd — visible speed, pause, zoom and camp controls sharing the keyboard camera API.
# Bridge methods: none directly; the camera owns set_sim_speed and pause handling.
extends PanelContainer

signal action_used(action: String)

const THEME = preload("res://assets/ui/worldsim_theme.tres")

var _camera: Camera2D = null
var _catalog: RefCounted = null
var _buttons: Dictionary = {}
var _zoom_label: Label = null

func setup(camera: Camera2D, catalog: RefCounted) -> void:
	_camera = camera
	_catalog = catalog
	if _buttons.is_empty():
		_build()
	for action: String in _buttons:
		var button: Button = _buttons[action]
		button.tooltip_text = str(_catalog.call("tooltip", action)) if _catalog != null else ""
	refresh_state()

func _build() -> void:
	theme = THEME
	mouse_filter = Control.MOUSE_FILTER_STOP
	var rows: VBoxContainer = VBoxContainer.new()
	rows.add_theme_constant_override("separation", 8)
	add_child(rows)
	var speeds: HBoxContainer = HBoxContainer.new()
	rows.add_child(speeds)
	_add_button(speeds, "pause", "⏸", true)
	for level: int in range(1, 5):
		_add_button(speeds, "speed_%d" % level, "▶".repeat(level), true)
	var zoom_row: HBoxContainer = HBoxContainer.new()
	rows.add_child(zoom_row)
	_add_button(zoom_row, "zoom_out", "−")
	_zoom_label = Label.new()
	_zoom_label.custom_minimum_size.x = 64
	_zoom_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_zoom_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	zoom_row.add_child(_zoom_label)
	_add_button(zoom_row, "zoom_in", "+")
	var camp: Button = _add_button(zoom_row, "home", _text("UI_MUX_CAMP"))
	camp.size_flags_horizontal = Control.SIZE_EXPAND_FILL

func _add_button(row: HBoxContainer, action: String, label: String, toggle: bool = false) -> Button:
	var button: Button = Button.new()
	button.name = action.to_pascal_case()
	button.text = label
	button.toggle_mode = toggle
	button.focus_mode = Control.FOCUS_ALL
	button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	button.pressed.connect(_activate.bind(action))
	row.add_child(button)
	_buttons[action] = button
	return button

func _activate(action: String) -> void:
	if _camera == null:
		return
	match action:
		"pause": _camera.call("set_paused", not bool(_camera.call("is_paused")))
		"zoom_in": _camera.call("step_zoom", 1)
		"zoom_out": _camera.call("step_zoom", -1)
		"home": _camera.call("center_on_camp")
		_:
			if action.begins_with("speed_"):
				_camera.call("set_speed", float(action.trim_prefix("speed_")))
				_camera.call("set_paused", false)
	refresh_state()
	action_used.emit(action)

func refresh_state() -> void:
	if _camera == null or _buttons.is_empty():
		return
	var paused: bool = bool(_camera.call("is_paused"))
	var speed: int = int(_camera.call("get_speed"))
	for action: String in _buttons:
		var button: Button = _buttons[action]
		if button.toggle_mode:
			button.set_pressed_no_signal(paused if action == "pause" else not paused and action == "speed_%d" % speed)
	var level: float = float(_camera.call("get_zoom_level")) if _camera.has_method("get_zoom_level") else _camera.zoom.x
	_zoom_label.text = "¼×" if is_equal_approx(level, 0.25) else ("½×" if is_equal_approx(level, 0.5) else "%d×" % int(level))

func button_for(action: String) -> Button:
	return _buttons.get(action) as Button

func _process(_delta: float) -> void:
	refresh_state()

func _text(key: String) -> String:
	var locale: Node = get_node_or_null("/root/Locale")
	return str(locale.call("ltr", key)) if locale != null else key
