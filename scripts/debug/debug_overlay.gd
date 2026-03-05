extends CanvasLayer

## In-game debug overlay. F3 cycles: OFF -> COMPACT -> FULL -> OFF.

enum Mode { OFF, COMPACT, FULL }

var _mode: Mode = Mode.OFF
var _provider: DebugDataProvider
var _label: Label
var _update_counter: int = 0

# FULL mode
var _panel_container: PanelContainer
var _tab_container: TabContainer
var _panels: Array = []


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
			_set_mode(Mode.FULL)
		Mode.FULL:
			_set_mode(Mode.OFF)


func _set_mode(new_mode: Mode) -> void:
	_mode = new_mode
	match _mode:
		Mode.OFF:
			if _label != null:
				_label.visible = false
			if _panel_container != null:
				_panel_container.visible = false
			_provider.enable_debug(false)
		Mode.COMPACT:
			if _label != null:
				_label.visible = true
			if _panel_container != null:
				_panel_container.visible = false
			_provider.enable_debug(true)
		Mode.FULL:
			if _label != null:
				_label.visible = false
			_build_full_panel_if_needed()
			if _panel_container != null:
				_panel_container.visible = true
			_provider.enable_debug(true)


func _build_ui() -> void:
	layer = 100

	# COMPACT: 3-line green HUD
	_label = Label.new()
	_label.position = Vector2(10, 10)
	_label.add_theme_font_size_override("font_size", 14)
	_label.add_theme_color_override("font_color", Color(0.0, 1.0, 0.0))
	_label.visible = false
	add_child(_label)


func _build_full_panel_if_needed() -> void:
	if _panel_container != null:
		return

	var viewport_size: Vector2 = get_viewport().get_visible_rect().size
	var panel_width: float = viewport_size.x * 0.35

	_panel_container = PanelContainer.new()
	_panel_container.position = Vector2(0, 0)
	_panel_container.size = Vector2(panel_width, viewport_size.y)
	_panel_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	add_child(_panel_container)

	var scroll := ScrollContainer.new()
	scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_panel_container.add_child(scroll)

	_tab_container = TabContainer.new()
	_tab_container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_tab_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_tab_container.custom_minimum_size = Vector2(panel_width - 10, 0)
	scroll.add_child(_tab_container)

	_add_tab("Perf",    DebugPerfPanel.new())
	_add_tab("Inspect", DebugInspectPanel.new())
	_add_tab("Systems", DebugSystemPanel.new())
	_add_tab("Events",  DebugEventPanel.new())
	_add_tab("Balance", DebugBalancePanel.new())
	_add_tab("World",   DebugWorldPanel.new())
	_add_tab("Guard",   DebugGuardPanel.new())
	_add_tab("FFI",     DebugFFIPanel.new())


func _add_tab(tab_name: String, panel: Control) -> void:
	panel.name = tab_name
	_tab_container.add_child(panel)
	panel.init(_provider)
	_panels.append(panel)


func _process(_delta: float) -> void:
	if _mode == Mode.OFF:
		return
	_update_counter += 1
	if _mode == Mode.COMPACT:
		if _update_counter % 10 == 0:
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
	var line3: String = "Debug: COMPACT | F3 for FULL"

	_label.text = line1 + "\n" + line2 + "\n" + line3
