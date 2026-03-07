class_name DebugPerfPanel
extends VBoxContainer

## Perf tab: 10 section bars (A-J) with ms values + tick history total.

const SECTIONS: Array[Dictionary] = [
	{"label": "A: Survival",    "color": Color(0.94, 0.27, 0.27), "pri_min": 10,  "pri_max": 53},
	{"label": "B: Psychology",  "color": Color(0.66, 0.33, 0.97), "pri_min": 55,  "pri_max": 105},
	{"label": "I: Economy",     "color": Color(0.92, 0.70, 0.03), "pri_min": 87,  "pri_max": 95},
	{"label": "C: Tech",        "color": Color(0.98, 0.45, 0.09), "pri_min": 120, "pri_max": 140},
	{"label": "H: Environment", "color": Color(0.13, 0.77, 0.37), "pri_min": 135, "pri_max": 441},
	{"label": "D: Derived",     "color": Color(0.39, 0.40, 0.95), "pri_min": 145, "pri_max": 185},
	{"label": "E: Social",      "color": Color(0.23, 0.51, 0.98), "pri_min": 200, "pri_max": 240},
	{"label": "F: Politics",    "color": Color(0.93, 0.27, 0.60), "pri_min": 250, "pri_max": 310},
	{"label": "G: Culture",     "color": Color(0.08, 0.72, 0.65), "pri_min": 350, "pri_max": 400},
	{"label": "J: Guardrails",  "color": Color(0.47, 0.44, 0.40), "pri_min": 450, "pri_max": 490},
]

var _provider: DebugDataProvider
var _update_counter: int = 0
var _bars: Array[StatBar] = []
var _total_label: Label


func init(provider: DebugDataProvider) -> void:
	_provider = provider
	_build_ui()


func _build_ui() -> void:
	var title := Label.new()
	title.text = "System Performance"
	title.add_theme_font_size_override("font_size", 13)
	add_child(title)

	_bars.clear()
	for sec in SECTIONS:
		var bar := StatBar.new()
		bar.label_text = sec["label"]
		bar.bar_color = sec["color"]
		bar.max_value = 4.0
		bar.custom_minimum_size = Vector2(0, 20)
		bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		add_child(bar)
		_bars.append(bar)

	var sep := HSeparator.new()
	add_child(sep)

	_total_label = Label.new()
	_total_label.text = "Total: --ms"
	_total_label.add_theme_font_size_override("font_size", 12)
	add_child(_total_label)


func _process(_delta: float) -> void:
	_update_counter += 1
	if _update_counter % 60 != 0:
		return
	_update_data()


func _update_data() -> void:
	if _provider == null or not is_instance_valid(_provider):
		return
	var perf: Dictionary = _provider.get_system_perf()
	if perf.is_empty():
		return

	var section_ms: Array[float] = []
	for i in SECTIONS.size():
		section_ms.append(0.0)

	var total_us: float = 0.0
	for key in perf:
		var entry: Dictionary = perf[key]
		var us: float = float(entry.get("us", 0))
		var pri: int = int(entry.get("priority", -1))
		total_us += us
		for i in SECTIONS.size():
			var sec: Dictionary = SECTIONS[i]
			if pri >= sec["pri_min"] and pri <= sec["pri_max"]:
				section_ms[i] += us / 1000.0
				break

	var total_ms: float = total_us / 1000.0
	var budget_ms: float = maxf(total_ms, 1.0)

	for i in _bars.size():
		_bars[i].max_value = budget_ms
		_bars[i].value = section_ms[i]
		_bars[i].queue_redraw()

	_total_label.text = "Total: %.2fms / 16ms" % total_ms
