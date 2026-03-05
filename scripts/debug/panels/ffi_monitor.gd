class_name DebugFFIMonitor
extends VBoxContainer

## FFI tab: shows bridge bandwidth (tick time history) as a sparkline graph.

var _provider: DebugDataProvider
var _label: Label
var _graph: DebugMiniGraph
var _tick_counter: int = 0
const REFRESH_INTERVAL: int = 60


func init_provider(provider: DebugDataProvider) -> void:
	_provider = provider


func _ready() -> void:
	_label = Label.new()
	_label.text = Locale.ltr("DEBUG_BANDWIDTH")
	add_child(_label)

	_graph = DebugMiniGraph.new()
	_graph.custom_minimum_size = Vector2(360, 60)
	add_child(_graph)


func _process(_delta: float) -> void:
	if _provider == null:
		return
	_tick_counter += 1
	if _tick_counter < REFRESH_INTERVAL:
		return
	_tick_counter = 0
	var history: PackedFloat32Array = _provider.get_tick_history()
	_graph.set_data(history, 33.3)
	var summary: Dictionary = _provider.get_debug_summary()
	var tick_us: int = summary.get("current_tick_us", 0)
	_label.text = "%s: %.2fms" % [Locale.ltr("DEBUG_BANDWIDTH"), tick_us / 1000.0]
