class_name DebugPerfPanel
extends ScrollContainer

## Performance tab: tick budget graph + per-system ms bars.
## Refreshes every 60 frames (~1s at 60fps).

var _provider: DebugDataProvider
var _vbox: VBoxContainer
var _tick_label: Label
var _graph: DebugMiniGraph
var _bars: Dictionary = {}
var _tick_counter: int = 0


func init_provider(provider: DebugDataProvider) -> void:
	_provider = provider


func _ready() -> void:
	_vbox = VBoxContainer.new()
	_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_child(_vbox)

	_tick_label = Label.new()
	_tick_label.text = Locale.ltr("DEBUG_TICK_BUDGET")
	_vbox.add_child(_tick_label)

	_graph = DebugMiniGraph.new()
	_graph.custom_minimum_size = Vector2(360, 60)
	_vbox.add_child(_graph)


func _process(_delta: float) -> void:
	if _provider == null:
		return
	_tick_counter += 1
	if _tick_counter < 60:
		return
	_tick_counter = 0

	var history: PackedFloat32Array = _provider.get_tick_history()
	_graph.set_data(history, 33.3)

	var perf: Dictionary = _provider.get_system_perf()
	var summary: Dictionary = _provider.get_debug_summary()
	var tick_us: int = summary.get("current_tick_us", 0)
	_tick_label.text = "%s: %.2fms" % [Locale.ltr("DEBUG_TICK_BUDGET"), tick_us / 1000.0]

	for key: String in perf:
		var entry: Dictionary = perf[key]
		var ms: float = entry.get("ms", 0.0)
		var section: String = entry.get("section", "Z")
		if not _bars.has(section):
			var bar := DebugStatBar.new()
			_vbox.add_child(bar)
			_bars[section] = bar
		_bars[section].set_data(section, ms, 16.0, Color(0.3, 0.8, 0.3))
