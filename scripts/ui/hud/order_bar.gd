# order_bar.gd (UI/OrderBar in scenes/main.tscn): the M1 order bar at the bottom of the screen.
# Five modes — 베기 / 비축 구역 / 오두막 / 구역 지우기 / 취소 — chosen by a click or F1–F5 (the number keys
#   1–4 set the speed; M1 DECISIONS D24); Esc or a right click leaves the mode. Shows the mode's hint (with no
#   mode: the order keys and Home/H = centre on the camp, camera_controller.gd) and, on a backing panel above the
#   bar, a short toast with each order's result (count with its noun, or the reject reason).
# Bridge methods: none (OrderTool sends the commands).
extends Control

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const MODES: Array[Dictionary] = [
	{"mode": 1, "key": "UI_ORDER_CHOP", "hotkey": KEY_F1, "hint": "UI_ORDER_HINT_AREA"},
	{"mode": 2, "key": "UI_ORDER_STOCKPILE", "hotkey": KEY_F2, "hint": "UI_ORDER_HINT_AREA"},
	{"mode": 3, "key": "UI_ORDER_HUT", "hotkey": KEY_F3, "hint": "UI_ORDER_HINT_PLACE"},
	{"mode": 4, "key": "UI_ORDER_REMOVE_ZONE", "hotkey": KEY_F4, "hint": "UI_ORDER_HINT_AREA"},
	{"mode": 5, "key": "UI_ORDER_CANCEL", "hotkey": KEY_F5, "hint": "UI_ORDER_HINT_AREA"},
]
const TOAST_SECONDS: float = 2.0

var _tool: Node = null
var _buttons: Array[Button] = []
var _hint: Label
var _toast: Label
var _toast_panel: PanelContainer
var _toast_left: float = 0.0
var _locale: Node = null


func _loc(key: String) -> String:
	return str(_locale.call("ltr", key)) if _locale != null else key


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	set_anchors_preset(Control.PRESET_FULL_RECT)
	_locale = get_node_or_null("/root/Locale")
	var panel: PanelContainer = ArtUI.panel(self, Rect2(12, 1080 - 76, 1100, 64))
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var row: HBoxContainer = HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	panel.add_child(row)
	for entry: Dictionary in MODES:
		var button: Button = Button.new()
		button.toggle_mode = true
		button.focus_mode = Control.FOCUS_NONE
		button.text = "%s %s" % [OS.get_keycode_string(int(entry["hotkey"])), _loc(str(entry["key"]))]
		button.pressed.connect(_on_button.bind(int(entry["mode"])))
		row.add_child(button)
		_buttons.append(button)
	_hint = Label.new()
	_hint.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_hint.add_theme_color_override("font_color", Color("c8b89a"))
	row.add_child(_hint)
	_hint.text = _loc("UI_ORDER_HINT_IDLE")
	# The toast sits on its own panel just above the bar; the panel shrinks to the text on every show.
	_toast_panel = ArtUI.panel(self, Rect2(12, 1080 - 76 - 8 - 48, 0, 0))
	_toast = Label.new()
	_toast.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_toast.add_theme_color_override("font_color", Color("f2e6c8"))
	_toast_panel.add_child(_toast)
	_toast_panel.visible = false
	_tool = get_node_or_null("/root/Main/ColonyView/OrderTool")
	if _tool != null:
		_tool.connect("mode_changed", _on_mode_changed)


func _on_button(mode: int) -> void:
	select_mode(mode if _tool == null or int(_tool.get("mode")) != mode else 0)


## Choose mode `mode` (0 = none) on the order tool.
func select_mode(mode: int) -> void:
	if _tool != null:
		_tool.call("set_mode", mode)


## The order tool's current mode (0 = none).
func current_mode() -> int:
	return int(_tool.get("mode")) if _tool != null else 0


func _on_mode_changed(mode: int) -> void:
	for i: int in _buttons.size():
		_buttons[i].set_pressed_no_signal(int(MODES[i]["mode"]) == mode)
	_hint.text = _loc("UI_ORDER_HINT_IDLE") if mode == 0 else ""
	for entry: Dictionary in MODES:
		if int(entry["mode"]) == mode:
			_hint.text = _loc(str(entry["hint"]))


## Show `text` above the bar for a short while.
func show_toast(text: String) -> void:
	_toast.text = text
	_toast_panel.visible = true
	_toast_panel.size = Vector2.ZERO
	_toast_left = TOAST_SECONDS


func _process(delta: float) -> void:
	if _toast_left > 0.0:
		_toast_left -= delta
		if _toast_left <= 0.0:
			_toast_panel.visible = false


func _unhandled_input(event: InputEvent) -> void:
	if not (event is InputEventKey and event.pressed and not event.echo):
		return
	var key: InputEventKey = event
	for entry: Dictionary in MODES:
		if key.keycode == int(entry["hotkey"]):
			select_mode(int(entry["mode"]))
			get_viewport().set_input_as_handled()
			return
