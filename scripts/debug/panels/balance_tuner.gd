class_name DebugBalanceTuner
extends ScrollContainer

## Balance tab: runtime sliders for simulation config constants.
## Changes are pushed to Rust via DebugDataProvider.set_config_value().

var _provider: DebugDataProvider
var _sliders: Dictionary = {}
var _defaults: Dictionary = {}
var _changed_labels: Dictionary = {}
var _vbox: VBoxContainer

const CONFIG_RANGES: Dictionary = {
	"need_decay_rate": [0.001, 0.05],
	"stress_drain_rate": [0.0001, 0.005],
	"emotion_decay_rate": [0.01, 0.2],
	"contagion_radius": [1.0, 20.0],
	"contagion_strength": [0.01, 0.5],
	"resource_regen_r": [0.001, 0.2],
	"allee_threshold": [0.01, 0.2],
	"harvest_gamma": [1.0, 5.0],
	"fallow_half_life": [5.0, 50.0],
	"surplus_threshold_days": [5.0, 30.0],
	"legitimacy_tradition_w": [0.0, 1.0],
	"legitimacy_charisma_w": [0.0, 1.0],
	"rebel_base_threshold": [0.3, 0.9],
	"tedium_schism_threshold": [0.5, 1.0],
}


func init_provider(provider: DebugDataProvider) -> void:
	_provider = provider


func _ready() -> void:
	_vbox = VBoxContainer.new()
	_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_child(_vbox)

	var reset_btn := Button.new()
	reset_btn.text = Locale.ltr("DEBUG_RESET_ALL")
	reset_btn.pressed.connect(_reset_all)
	_vbox.add_child(reset_btn)

	_build_sliders()


func _build_sliders() -> void:
	if _provider == null:
		return
	var config: Dictionary = _provider.get_config_all()
	for key: String in CONFIG_RANGES:
		var range_arr: Array = CONFIG_RANGES[key]
		var current: float = config.get(key, range_arr[0])
		_defaults[key] = current

		var row := HBoxContainer.new()
		_vbox.add_child(row)

		var lbl := Label.new()
		lbl.text = key
		lbl.custom_minimum_size = Vector2(160, 0)
		lbl.add_theme_font_size_override("font_size", 10)
		row.add_child(lbl)

		var slider := HSlider.new()
		slider.min_value = range_arr[0]
		slider.max_value = range_arr[1]
		slider.step = (range_arr[1] - range_arr[0]) / 200.0
		slider.value = current
		slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		slider.value_changed.connect(_on_slider_changed.bind(key))
		row.add_child(slider)
		_sliders[key] = slider

		var changed_lbl := Label.new()
		changed_lbl.text = ""
		changed_lbl.add_theme_color_override("font_color", Color.YELLOW)
		changed_lbl.add_theme_font_size_override("font_size", 10)
		row.add_child(changed_lbl)
		_changed_labels[key] = changed_lbl


func _on_slider_changed(value: float, key: String) -> void:
	if _provider == null:
		return
	_provider.set_config_value(key, value)
	var lbl: Label = _changed_labels[key]
	if absf(value - _defaults.get(key, value)) > 1e-9:
		lbl.text = Locale.ltr("DEBUG_CHANGED")
	else:
		lbl.text = ""


func _reset_all() -> void:
	if _provider == null:
		return
	for key: String in _sliders:
		var default_val: float = _defaults.get(key, 0.0)
		_sliders[key].value = default_val
		_provider.set_config_value(key, default_val)
		_changed_labels[key].text = ""
