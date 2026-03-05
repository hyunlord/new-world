class_name DebugEventMonitor
extends VBoxContainer

## Events tab: display-only event log. Data arrives via SimulationBus signals.

var _provider: DebugDataProvider
var _list: ItemList


func init_provider(provider: DebugDataProvider) -> void:
	_provider = provider


func _ready() -> void:
	var lbl := Label.new()
	lbl.text = Locale.ltr("DEBUG_EVENTS_PER_TICK")
	add_child(lbl)

	_list = ItemList.new()
	_list.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_list.add_theme_font_size_override("font_size", 10)
	add_child(_list)


func _process(_delta: float) -> void:
	pass  # Events tab is display-only; data arrives via SimulationBus signals
