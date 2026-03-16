extends PanelContainer

signal oracle_action(action_type: String)

const ACTIONS: Array[Dictionary] = [
	{
		"icon": "🔮",
		"type": "prophecy",
		"label": "UI_ORACLE_PROPHECY",
		"tip": "UI_ORACLE_PROPHECY_TIP",
		"cost": 10,
	},
	{
		"icon": "✨",
		"type": "miracle",
		"label": "UI_ORACLE_MIRACLE",
		"tip": "UI_ORACLE_MIRACLE_TIP",
		"cost": 25,
	},
	{
		"icon": "🌊",
		"type": "disaster",
		"label": "UI_ORACLE_DISASTER",
		"tip": "UI_ORACLE_DISASTER_TIP",
		"cost": 50,
	},
	{
		"icon": "🌿",
		"type": "blessing",
		"label": "UI_ORACLE_BLESSING",
		"tip": "UI_ORACLE_BLESSING_TIP",
		"cost": 30,
	},
	{
		"icon": "⚖️",
		"type": "worldrule",
		"label": "UI_ORACLE_WORLDRULE",
		"tip": "UI_ORACLE_WORLDRULE_TIP",
		"cost": 100,
	},
]

var _buttons: Array[Button] = []
var _faith_label: Label
var _faith_value: int = 0
var _tooltip_panel: PanelContainer
var _tooltip_label: Label


func _ready() -> void:
	_build_ui()
	visible = false


func _build_ui() -> void:
	var panel_style := StyleBoxFlat.new()
	panel_style.bg_color = Color(0.03, 0.04, 0.06, 0.95)
	panel_style.border_color = Color(0.16, 0.22, 0.30)
	panel_style.set_border_width_all(1)
	panel_style.set_corner_radius_all(8)
	panel_style.content_margin_left = 12
	panel_style.content_margin_right = 12
	panel_style.content_margin_top = 8
	panel_style.content_margin_bottom = 8
	add_theme_stylebox_override("panel", panel_style)

	var root := VBoxContainer.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.add_theme_constant_override("separation", 6)
	add_child(root)

	_tooltip_panel = PanelContainer.new()
	_tooltip_panel.visible = false
	_tooltip_panel.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var tooltip_style := StyleBoxFlat.new()
	tooltip_style.bg_color = Color(0.06, 0.09, 0.12, 0.95)
	tooltip_style.border_color = Color(0.16, 0.22, 0.30)
	tooltip_style.set_border_width_all(1)
	tooltip_style.set_corner_radius_all(4)
	tooltip_style.content_margin_left = 6
	tooltip_style.content_margin_right = 6
	tooltip_style.content_margin_top = 3
	tooltip_style.content_margin_bottom = 3
	_tooltip_panel.add_theme_stylebox_override("panel", tooltip_style)
	_tooltip_label = Label.new()
	_tooltip_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_tooltip_label.add_theme_font_size_override("font_size", 9)
	_tooltip_label.add_theme_color_override("font_color", Color(0.66, 0.73, 0.78))
	_tooltip_panel.add_child(_tooltip_label)
	_tooltip_panel.offset_left = 0.0
	_tooltip_panel.offset_top = -44.0
	_tooltip_panel.offset_right = 280.0
	_tooltip_panel.offset_bottom = -4.0
	add_child(_tooltip_panel)

	var button_row := HBoxContainer.new()
	button_row.alignment = BoxContainer.ALIGNMENT_CENTER
	button_row.add_theme_constant_override("separation", 6)
	root.add_child(button_row)

	for action: Dictionary in ACTIONS:
		var button := Button.new()
		button.text = str(action.get("icon", "?"))
		button.custom_minimum_size = Vector2(40.0, 40.0)
		button.focus_mode = Control.FOCUS_NONE
		button.add_theme_font_size_override("font_size", 18)
		button.set_meta("action_type", str(action.get("type", "")))
		button.set_meta("tip_key", str(action.get("tip", "")))
		button.set_meta("label_key", str(action.get("label", "")))
		button.set_meta("cost", int(action.get("cost", 0)))

		var button_style := StyleBoxFlat.new()
		button_style.bg_color = Color(0.04, 0.06, 0.10)
		button_style.border_color = Color(0.16, 0.22, 0.30)
		button_style.set_border_width_all(1)
		button_style.set_corner_radius_all(6)
		button.add_theme_stylebox_override("normal", button_style)

		var hover_style: StyleBoxFlat = button_style.duplicate()
		hover_style.border_color = Color(0.35, 0.53, 0.69)
		hover_style.bg_color = Color(0.06, 0.09, 0.16)
		button.add_theme_stylebox_override("hover", hover_style)
		button.add_theme_stylebox_override("pressed", hover_style.duplicate())
		button.add_theme_stylebox_override("focus", hover_style.duplicate())

		var action_type: String = str(action.get("type", ""))
		var tip_key: String = str(action.get("tip", ""))
		var cost: int = int(action.get("cost", 0))
		button.pressed.connect(func() -> void:
			oracle_action.emit(action_type)
		)
		button.mouse_entered.connect(func() -> void:
			_show_tooltip(Locale.ltr(tip_key), cost)
		)
		button.mouse_exited.connect(_hide_tooltip)

		button_row.add_child(button)
		_buttons.append(button)

	_faith_label = Label.new()
	_faith_label.add_theme_font_size_override("font_size", 9)
	_faith_label.add_theme_color_override("font_color", Color(0.72, 0.76, 0.84))
	button_row.add_child(_faith_label)

	set_faith(0)
	refresh_locale()


func set_faith(value: int) -> void:
	_faith_value = value
	if _faith_label != null:
		_faith_label.text = "%s: %d" % [Locale.ltr("UI_ORACLE_FAITH"), value]


func refresh_locale() -> void:
	for button: Button in _buttons:
		if button == null:
			continue
		var tip_key: String = str(button.get_meta("tip_key", ""))
		var cost: int = int(button.get_meta("cost", 0))
		button.tooltip_text = "%s\n%s: %d" % [
			Locale.ltr(tip_key),
			Locale.ltr("UI_ORACLE_FAITH_COST"),
			cost,
		]
	set_faith(_faith_value)
	if _tooltip_panel != null and _tooltip_panel.visible:
		var active_tip: String = str(_tooltip_panel.get_meta("active_tip", ""))
		var active_cost: int = int(_tooltip_panel.get_meta("active_cost", 0))
		if not active_tip.is_empty():
			_show_tooltip(Locale.ltr(active_tip), active_cost)


func _show_tooltip(text: String, cost: int) -> void:
	if _tooltip_panel == null or _tooltip_label == null:
		return
	_tooltip_panel.set_meta("active_tip", _tip_key_for_text(text))
	_tooltip_panel.set_meta("active_cost", cost)
	_tooltip_label.text = "%s\n%s: %d" % [text, Locale.ltr("UI_ORACLE_FAITH_COST"), cost]
	_tooltip_panel.visible = true


func _hide_tooltip() -> void:
	if _tooltip_panel == null:
		return
	_tooltip_panel.visible = false
	_tooltip_panel.set_meta("active_tip", "")
	_tooltip_panel.set_meta("active_cost", 0)


func _tip_key_for_text(text: String) -> String:
	for action: Dictionary in ACTIONS:
		var tip_key: String = str(action.get("tip", ""))
		if Locale.ltr(tip_key) == text:
			return tip_key
	return ""
