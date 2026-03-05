class_name DebugStatBar
extends HBoxContainer

## Horizontal stat bar for debug panels. Shows label, progress bar, and value text.
## Used in perf_panel to display per-system execution time as a ms bar.

var _label: Label
var _bar: ProgressBar
var _value_label: Label

func _ready() -> void:
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_theme_constant_override("separation", 6)

	_label = Label.new()
	_label.custom_minimum_size = Vector2(120, 0)
	_label.add_theme_font_size_override("font_size", 11)
	add_child(_label)

	_bar = ProgressBar.new()
	_bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_bar.custom_minimum_size = Vector2(80, 14)
	_bar.show_percentage = false
	add_child(_bar)

	_value_label = Label.new()
	_value_label.custom_minimum_size = Vector2(60, 0)
	_value_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	_value_label.add_theme_font_size_override("font_size", 11)
	add_child(_value_label)


## Set bar data. label_text is already-localized string. value and max in same unit.
func set_data(label_text: String, value: float, max_value: float, bar_color: Color = Color(0.2, 0.7, 0.3)) -> void:
	_label.text = label_text
	_bar.max_value = max(max_value, 0.001)
	_bar.value = clampf(value, 0.0, max_value)

	var style := StyleBoxFlat.new()
	style.bg_color = bar_color
	_bar.add_theme_stylebox_override("fill", style)

	_value_label.text = "%.2fms" % (value / 1000.0) if value >= 0 else "—"


## Set value label format override (e.g. for non-time values).
func set_value_text(text: String) -> void:
	_value_label.text = text
