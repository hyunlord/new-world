class_name DebugBalancePanel
extends VBoxContainer

## Balance tab: config sliders with live apply via set_config_value().

var _provider: DebugDataProvider
var _sliders: Dictionary = {}  # key -> HSlider
var _value_labels: Dictionary = {}  # key -> Label
var _built: bool = false


func init(provider: DebugDataProvider) -> void:
	_provider = provider
	_build_ui()


func _build_ui() -> void:
	var title := Label.new()
	title.text = "Balance Tuner"
	title.add_theme_font_size_override("font_size", 13)
	add_child(title)

	add_child(HSeparator.new())

	var config: Dictionary = _provider.get_config_all()
	if config.is_empty():
		var no_data := Label.new()
		no_data.text = "No config data"
		no_data.add_theme_font_size_override("font_size", 11)
		add_child(no_data)
		return

	_sliders.clear()
	_value_labels.clear()

	for key in config:
		var entry = config[key]
		var cur_val: float = 0.0
		var min_val: float = 0.0
		var max_val: float = 1.0

		if entry is Dictionary:
			cur_val = float(entry.get("value", 0.0))
			min_val = float(entry.get("min", 0.0))
			max_val = float(entry.get("max", 1.0))
		elif entry is float or entry is int:
			cur_val = float(entry)
			max_val = maxf(cur_val * 2.0, 1.0)

		var vbox := VBoxContainer.new()

		var hbox := HBoxContainer.new()
		var key_lbl := Label.new()
		key_lbl.text = key
		key_lbl.custom_minimum_size = Vector2(140, 0)
		key_lbl.add_theme_font_size_override("font_size", 10)
		var val_lbl := Label.new()
		val_lbl.text = "%.3f" % cur_val
		val_lbl.add_theme_font_size_override("font_size", 10)
		val_lbl.add_theme_color_override("font_color", Color(0.8, 1.0, 0.8))
		hbox.add_child(key_lbl)
		hbox.add_child(val_lbl)
		vbox.add_child(hbox)

		var slider := HSlider.new()
		slider.min_value = min_val
		slider.max_value = max_val
		slider.value = cur_val
		slider.step = (max_val - min_val) / 100.0
		slider.custom_minimum_size = Vector2(0, 16)
		slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL

		var captured_key: String = key
		slider.value_changed.connect(func(v: float) -> void:
			_on_slider_changed(captured_key, v)
		)

		vbox.add_child(slider)
		add_child(vbox)

		_sliders[key] = slider
		_value_labels[key] = val_lbl

	add_child(HSeparator.new())

	var reset_btn := Button.new()
	reset_btn.text = "Reset All"
	reset_btn.add_theme_font_size_override("font_size", 11)
	reset_btn.pressed.connect(_on_reset_all)
	add_child(reset_btn)

	_built = true


func _on_slider_changed(key: String, value: float) -> void:
	_provider.set_config_value(key, value)
	if key in _value_labels:
		_value_labels[key].text = "%.3f" % value


func _on_reset_all() -> void:
	# Re-fetch defaults after reset
	var config: Dictionary = _provider.get_config_all()
	for key in _sliders:
		if key in config:
			var entry = config[key]
			var def_val: float = 0.0
			if entry is Dictionary:
				def_val = float(entry.get("default", entry.get("value", 0.0)))
			_sliders[key].value = def_val
