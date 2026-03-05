class_name DebugSystemPanel
extends ScrollContainer

## Systems tab: lists all simulation systems with their section and ms timing.
## Refreshes every 60 frames (~1s at 60fps).

var _provider: DebugDataProvider
var _vbox: VBoxContainer
var _tick_counter: int = 0


func init_provider(provider: DebugDataProvider) -> void:
	_provider = provider


func _ready() -> void:
	_vbox = VBoxContainer.new()
	_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_child(_vbox)


func _process(_delta: float) -> void:
	if _provider == null:
		return
	_tick_counter += 1
	if _tick_counter < 60:
		return
	_tick_counter = 0
	_refresh()


func _refresh() -> void:
	for child: Node in _vbox.get_children():
		child.queue_free()

	var perf: Dictionary = _provider.get_system_perf()
	var sorted_keys: Array = perf.keys()
	sorted_keys.sort()

	for key: String in sorted_keys:
		var entry: Dictionary = perf[key]
		var lbl := Label.new()
		lbl.text = "%s [%s] %.2fms" % [key, entry.get("section", "?"), entry.get("ms", 0.0)]
		lbl.add_theme_font_size_override("font_size", 10)
		_vbox.add_child(lbl)
