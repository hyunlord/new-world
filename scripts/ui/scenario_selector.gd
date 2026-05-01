class_name ScenarioSelector
extends CanvasLayer

signal scenario_confirmed(scenario_name: String)

const _SCENARIOS: Array[Dictionary] = [
	{"id": "default", "name_key": "SCENARIO_DEFAULT_NAME", "desc_key": "SCENARIO_DEFAULT_DESC"},
	{"id": "eternal_winter", "name_key": "SCENARIO_ETERNAL_WINTER_NAME", "desc_key": "SCENARIO_ETERNAL_WINTER_DESC"},
	{"id": "perpetual_summer", "name_key": "SCENARIO_PERPETUAL_SUMMER_NAME", "desc_key": "SCENARIO_PERPETUAL_SUMMER_DESC"},
	{"id": "barren_world", "name_key": "SCENARIO_BARREN_WORLD_NAME", "desc_key": "SCENARIO_BARREN_WORLD_DESC"},
	{"id": "abundance", "name_key": "SCENARIO_ABUNDANCE_NAME", "desc_key": "SCENARIO_ABUNDANCE_DESC"},
]

var _selected_scenario: String = "default"
var _buttons: Dictionary = {}
var _desc_label: Label
var _start_button: Button
var _button_group: ButtonGroup

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		return
	_build_ui()

func _build_ui() -> void:
	layer = 10

	var bg := ColorRect.new()
	bg.color = Color(0.0, 0.0, 0.0, 0.75)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(bg)

	var center := CenterContainer.new()
	center.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(center)

	var panel := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.08, 0.08, 0.12, 0.96)
	style.border_width_all = 1
	style.border_color = Color(0.35, 0.35, 0.45, 0.8)
	style.corner_radius_top_left = 6
	style.corner_radius_top_right = 6
	style.corner_radius_bottom_left = 6
	style.corner_radius_bottom_right = 6
	panel.add_theme_stylebox_override("panel", style)
	center.add_child(panel)

	var vbox := VBoxContainer.new()
	vbox.custom_minimum_size = Vector2(420, 0)
	vbox.add_theme_constant_override("separation", 10)
	panel.add_child(vbox)

	var margin := MarginContainer.new()
	margin.add_theme_constant_override("margin_left", 20)
	margin.add_theme_constant_override("margin_right", 20)
	margin.add_theme_constant_override("margin_top", 18)
	margin.add_theme_constant_override("margin_bottom", 18)
	vbox.add_child(margin)

	var inner := VBoxContainer.new()
	inner.add_theme_constant_override("separation", 12)
	margin.add_child(inner)

	var title := Label.new()
	title.text = Locale.ltr("SCENARIO_SELECTOR_TITLE")
	title.add_theme_font_size_override("font_size", 16)
	title.add_theme_color_override("font_color", Color(1.0, 0.9, 0.7))
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	inner.add_child(title)

	_button_group = ButtonGroup.new()

	for scenario: Dictionary in _SCENARIOS:
		var btn := CheckBox.new()
		btn.button_group = _button_group
		btn.text = Locale.ltr(scenario["name_key"])
		btn.toggle_mode = true
		btn.button_pressed = (scenario["id"] == _selected_scenario)
		btn.focus_mode = Control.FOCUS_NONE
		var sid: String = scenario["id"]
		btn.toggled.connect(func(pressed: bool) -> void:
			if pressed:
				_on_scenario_toggled(sid)
		)
		inner.add_child(btn)
		_buttons[scenario["id"]] = btn

	_desc_label = Label.new()
	_desc_label.text = _get_desc_for(_selected_scenario)
	_desc_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_desc_label.add_theme_font_size_override("font_size", 11)
	_desc_label.add_theme_color_override("font_color", Color(0.72, 0.72, 0.72))
	_desc_label.custom_minimum_size = Vector2(0, 36)
	inner.add_child(_desc_label)

	_start_button = Button.new()
	_start_button.text = Locale.ltr("SCENARIO_START_GAME")
	_start_button.focus_mode = Control.FOCUS_NONE
	_start_button.pressed.connect(_on_start_pressed)
	inner.add_child(_start_button)

func _on_scenario_toggled(scenario_id: String) -> void:
	_selected_scenario = scenario_id
	if _desc_label != null:
		_desc_label.text = _get_desc_for(scenario_id)

func _on_start_pressed() -> void:
	scenario_confirmed.emit(_selected_scenario)
	hide()

func _get_desc_for(scenario_id: String) -> String:
	for s: Dictionary in _SCENARIOS:
		if s["id"] == scenario_id:
			return Locale.ltr(s["desc_key"])
	return ""

func get_selected_scenario() -> String:
	return _selected_scenario
