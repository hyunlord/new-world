extends CanvasLayer

## In-game debug overlay. F3 toggles: OFF -> COMPACT -> OFF.
## FULL mode (with tabs) will be added in Prompt 3.

enum Mode { OFF, COMPACT }

var _mode: Mode = Mode.OFF
var _provider: DebugDataProvider
var _label: Label
var _update_counter: int = 0


func init(bridge: Node) -> void:
	_provider = DebugDataProvider.new(bridge)
	_provider.enable_debug(false)
	_build_ui()
	_set_mode(Mode.OFF)


func cycle_mode() -> void:
	match _mode:
		Mode.OFF:
			_set_mode(Mode.COMPACT)
		Mode.COMPACT:
			_set_mode(Mode.OFF)


func _set_mode(new_mode: Mode) -> void:
	_mode = new_mode
	match _mode:
		Mode.OFF:
			if _label != null:
				_label.visible = false
			_provider.enable_debug(false)
		Mode.COMPACT:
			if _label != null:
				_label.visible = true
			_provider.enable_debug(true)


func _build_ui() -> void:
	layer = 100
	_label = Label.new()
	_label.position = Vector2(10, 10)
	_label.add_theme_font_size_override("font_size", 14)
	_label.add_theme_color_override("font_color", Color(0.0, 1.0, 0.0))
	_label.visible = false
	add_child(_label)


func _process(_delta: float) -> void:
	if _mode == Mode.OFF:
		return
	_update_counter += 1
	if _update_counter % 10 != 0:
		return
	_update_compact_hud()


func _update_compact_hud() -> void:
	var fps: int = Engine.get_frames_per_second()
	var perf: Dictionary = _provider.get_system_perf()
	var tick_us: float = 0.0
	for key in perf:
		var entry: Dictionary = perf[key]
		tick_us += float(entry.get("us", 0))
	var tick_ms: float = tick_us / 1000.0
	var tick_pct: float = (tick_ms / 16.0) * 100.0
	var entity_count: int = _provider.get_entity_count()
	var tick_num: int = _provider.get_tick()

	var line1: String = "FPS: %d | Tick: %.2fms / 16ms (%.0f%%)" % [fps, tick_ms, tick_pct]
	var line2: String = "Pop: %d | Tick#: %d" % [entity_count, tick_num]
	var line3: String = "Debug: COMPACT | F3 to toggle"

	_label.text = line1 + "\n" + line2 + "\n" + line3
