class_name DebugWorldStats
extends VBoxContainer

## World tab: population, season, FPS summary. Refreshes every 10 frames.

var _provider: DebugDataProvider
var _label: Label
var _tick_counter: int = 0


func init_provider(provider: DebugDataProvider) -> void:
	_provider = provider


func _ready() -> void:
	_label = Label.new()
	_label.text = Locale.ltr("DEBUG_NO_DATA")
	_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	add_child(_label)


func _process(_delta: float) -> void:
	if _provider == null:
		return
	_tick_counter += 1
	if _tick_counter < 10:
		return
	_tick_counter = 0

	var summary: Dictionary = _provider.get_debug_summary()
	var lines: PackedStringArray = []
	lines.append("%s: %d" % [Locale.ltr("DEBUG_POPULATION"), summary.get("population", 0)])
	lines.append("%s: %d" % [Locale.ltr("DEBUG_SEASON"), summary.get("season", 0)])
	lines.append("%s: %d" % [Locale.ltr("DEBUG_FPS"), Engine.get_frames_per_second()])
	_label.text = "\n".join(lines)
