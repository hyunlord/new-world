class_name DebugOverlay
extends CanvasLayer

## In-game debug overlay. F3 toggles: OFF → COMPACT → FULL → OFF.
## Uses DebugDataProvider for all data access — never calls SimBridge directly.

enum Mode { OFF, COMPACT, FULL }

var _mode: Mode = Mode.OFF
var _provider: DebugDataProvider

var _compact_label: Label
var _full_panel: Control
var _tab_container: TabContainer


func init(sim_bridge) -> void:
	_provider = DebugDataProvider.new(sim_bridge)
	_setup_compact()
	_setup_full_panel()
	_apply_mode()


func cycle_mode() -> void:
	_mode = (_mode + 1) % 3 as Mode
	_apply_mode()
	if _mode != Mode.OFF:
		_provider.enable_debug(true)
	else:
		_provider.enable_debug(false)


func _apply_mode() -> void:
	match _mode:
		Mode.OFF:
			visible = false
		Mode.COMPACT:
			visible = true
			_compact_label.visible = true
			_full_panel.visible = false
		Mode.FULL:
			visible = true
			_compact_label.visible = false
			_full_panel.visible = true


func _process(_delta: float) -> void:
	if _mode == Mode.OFF:
		return
	if _mode == Mode.COMPACT:
		_update_compact()
	# Full panel tabs self-update via their own _process


func _setup_compact() -> void:
	_compact_label = Label.new()
	_compact_label.add_theme_color_override("font_color", Color.WHITE)
	_compact_label.add_theme_font_size_override("font_size", 12)
	_compact_label.position = Vector2(8, 8)
	add_child(_compact_label)


func _update_compact() -> void:
	var summary: Dictionary = _provider.get_debug_summary()
	var fps: int = Engine.get_frames_per_second()
	var tick_us: int = summary.get("tick_us", summary.get("current_tick_us", 0))
	var pop: int = summary.get("population", 0)
	_compact_label.text = (
		"%s: %d | %s: %.2fms | %s: %d" % [
			Locale.ltr("DEBUG_FPS"), fps,
			Locale.ltr("DEBUG_TICK_BUDGET"), tick_us / 1000.0,
			Locale.ltr("DEBUG_POPULATION"), pop
		]
	)


func _setup_full_panel() -> void:
	_full_panel = Control.new()
	_full_panel.set_anchors_preset(Control.PRESET_LEFT_WIDE)
	_full_panel.custom_minimum_size = Vector2(380, 0)

	var bg := ColorRect.new()
	bg.color = Color(0.05, 0.05, 0.05, 0.88)
	bg.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_full_panel.add_child(bg)

	var vbox := VBoxContainer.new()
	vbox.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	vbox.add_theme_constant_override("separation", 4)
	_full_panel.add_child(vbox)

	var title_label := Label.new()
	title_label.text = Locale.ltr("DEBUG_TITLE")
	title_label.add_theme_color_override("font_color", Color(0.8, 1.0, 0.8))
	title_label.add_theme_font_size_override("font_size", 14)
	vbox.add_child(title_label)

	_tab_container = TabContainer.new()
	_tab_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	vbox.add_child(_tab_container)

	var panels: Array = [
		[Locale.ltr("DEBUG_TAB_PERF"), DebugPerfPanel.new()],
		[Locale.ltr("DEBUG_TAB_INSPECT"), DebugEntityInspector.new()],
		[Locale.ltr("DEBUG_TAB_SYSTEMS"), DebugSystemPanel.new()],
		[Locale.ltr("DEBUG_TAB_EVENTS"), DebugEventMonitor.new()],
		[Locale.ltr("DEBUG_TAB_BALANCE"), DebugBalanceTuner.new()],
		[Locale.ltr("DEBUG_TAB_WORLD"), DebugWorldStats.new()],
		[Locale.ltr("DEBUG_TAB_GUARD"), DebugGuardrailMonitor.new()],
		[Locale.ltr("DEBUG_TAB_FFI"), DebugFFIMonitor.new()],
	]

	for pair: Array in panels:
		var panel: Control = pair[1]
		panel.name = pair[0]
		_tab_container.add_child(panel)
		if panel.has_method("init_provider"):
			panel.init_provider(_provider)

	add_child(_full_panel)
