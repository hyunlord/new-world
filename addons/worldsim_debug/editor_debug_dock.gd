@tool
class_name EditorDebugDock
extends Control

## Main dock controller for the WorldSim Debug EditorPlugin.
## Shows live simulation panels when connected (game running),
## and static reference tabs (formulas, architecture, locale) always.

enum ConnectionState { DISCONNECTED, CONNECTED }

const SNAPSHOT_PATH := "user://debug_snapshot.json"
const CHECK_INTERVAL_SEC := 1.0

# Panel preload paths
const _PANEL_PATHS := {
	"perf":      "res://scripts/debug/panels/perf_panel.gd",
	"inspector": "res://scripts/debug/panels/entity_inspector.gd",
	"system":    "res://scripts/debug/panels/system_panel.gd",
	"events":    "res://scripts/debug/panels/event_monitor.gd",
	"balance":   "res://scripts/debug/panels/balance_tuner.gd",
	"world":     "res://scripts/debug/panels/world_stats.gd",
	"guard":     "res://scripts/debug/panels/guardrail_monitor.gd",
	"ffi":       "res://scripts/debug/panels/ffi_monitor.gd",
}

const _LIVE_TAB_LABELS := [
	"DEBUG_TAB_PERF", "DEBUG_TAB_INSPECT", "DEBUG_TAB_SYSTEMS",
	"DEBUG_TAB_EVENTS", "DEBUG_TAB_BALANCE", "DEBUG_TAB_WORLD",
	"DEBUG_TAB_GUARD", "DEBUG_TAB_FFI",
]
const _LIVE_TAB_KEYS := ["perf", "inspector", "system", "events", "balance", "world", "guard", "ffi"]

var _state: ConnectionState = ConnectionState.DISCONNECTED
var _file_provider: FileBasedDebugProvider

# UI nodes
var _status_bar: HBoxContainer
var _status_dot: Label
var _status_text: Label
var _tab_container: TabContainer

# Live panel wrappers — each is a Control containing [placeholder_label, panel]
var _live_wrappers: Array[Control] = []
var _live_panels: Array[Control] = []

# Static tab scripts
var _formula_tab: FormulaReference
var _architecture_tab: ArchitectureViewer
var _locale_tab: LocalePreview

# Snapshot check timer
var _check_timer: Timer

func _ready() -> void:
	if not Engine.is_editor_hint():
		return
	_file_provider = FileBasedDebugProvider.new()
	_build_ui()
	_start_timer()
	_set_state(ConnectionState.DISCONNECTED)

func _build_ui() -> void:
	var vbox := VBoxContainer.new()
	vbox.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(vbox)

	# Status bar
	_status_bar = HBoxContainer.new()
	_status_bar.add_theme_constant_override("separation", 6)
	vbox.add_child(_status_bar)

	_status_dot = Label.new()
	_status_dot.text = "🔴"
	_status_bar.add_child(_status_dot)

	_status_text = Label.new()
	_status_text.text = Locale.ltr("DEBUG_EDITOR_DISCONNECTED")
	_status_bar.add_child(_status_text)

	var sep := HSeparator.new()
	vbox.add_child(sep)

	# Tab container
	_tab_container = TabContainer.new()
	_tab_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	vbox.add_child(_tab_container)

	_build_live_tabs()
	_build_static_tabs()

func _build_live_tabs() -> void:
	for i in _LIVE_TAB_KEYS.size():
		var key: String = _LIVE_TAB_KEYS[i]
		var label_key: String = _LIVE_TAB_LABELS[i]

		# Wrapper control for this tab
		var wrapper := Control.new()
		wrapper.name = Locale.ltr(label_key)
		wrapper.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
		_tab_container.add_child(wrapper)
		_live_wrappers.append(wrapper)

		# Placeholder — shown when disconnected
		var placeholder := _make_disconnect_placeholder()
		placeholder.name = "Placeholder"
		wrapper.add_child(placeholder)

		# Live panel — load script and instantiate
		var panel := _load_live_panel(key)
		if panel:
			panel.name = "Panel"
			panel.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
			wrapper.add_child(panel)
			if panel.has_method("init_provider"):
				panel.call("init_provider", _file_provider)
		_live_panels.append(panel)

func _build_static_tabs() -> void:
	_formula_tab = FormulaReference.new()
	_formula_tab.name = Locale.ltr("DEBUG_TAB_FORMULAS")
	_tab_container.add_child(_formula_tab)

	_architecture_tab = ArchitectureViewer.new()
	_architecture_tab.name = Locale.ltr("DEBUG_TAB_ARCHITECTURE")
	_tab_container.add_child(_architecture_tab)

	_locale_tab = LocalePreview.new()
	_locale_tab.name = Locale.ltr("DEBUG_TAB_LOCALE")
	_tab_container.add_child(_locale_tab)

func _load_live_panel(key: String) -> Control:
	var path: String = _PANEL_PATHS.get(key, "")
	if path.is_empty() or not ResourceLoader.exists(path):
		return null
	var script = load(path)
	if not script:
		return null
	var node = script.new()
	if node is Control:
		return node as Control
	node.queue_free()
	return null

func _make_disconnect_placeholder() -> Control:
	var container := CenterContainer.new()
	container.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)

	var vbox := VBoxContainer.new()
	container.add_child(vbox)

	var dot_label := Label.new()
	dot_label.text = "🔴 " + Locale.ltr("DEBUG_EDITOR_DISCONNECTED")
	dot_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	vbox.add_child(dot_label)

	var hint_label := Label.new()
	hint_label.text = Locale.ltr("DEBUG_EDITOR_PRESS_PLAY")
	hint_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	hint_label.add_theme_color_override("font_color", Color(0.6, 0.6, 0.6))
	vbox.add_child(hint_label)

	var static_hint := Label.new()
	static_hint.text = Locale.ltr("DEBUG_EDITOR_STATIC_HINT")
	static_hint.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	static_hint.add_theme_color_override("font_color", Color(0.5, 0.7, 0.9))
	vbox.add_child(static_hint)

	return container

func _start_timer() -> void:
	_check_timer = Timer.new()
	_check_timer.wait_time = CHECK_INTERVAL_SEC
	_check_timer.autostart = true
	_check_timer.timeout.connect(_check_snapshot)
	add_child(_check_timer)

func _check_snapshot() -> void:
	if not Engine.is_editor_hint():
		return
	if not FileAccess.file_exists(SNAPSHOT_PATH):
		if _state == ConnectionState.CONNECTED:
			_set_state(ConnectionState.DISCONNECTED)
		return

	var file := FileAccess.open(SNAPSHOT_PATH, FileAccess.READ)
	if not file:
		return
	var text := file.get_as_text()
	file.close()

	var parsed := JSON.parse_string(text)
	if not parsed is Dictionary:
		if _state == ConnectionState.CONNECTED:
			_set_state(ConnectionState.DISCONNECTED)
		return

	var snapshot: Dictionary = parsed
	if not snapshot.has("tick") or not snapshot.has("timestamp_msec"):
		if _state == ConnectionState.CONNECTED:
			_set_state(ConnectionState.DISCONNECTED)
		return

	# Check freshness: snapshot must be < 3 seconds old
	var now_msec: int = Time.get_ticks_msec()
	# timestamp_msec in snapshot is wall-clock (Unix epoch ms), Godot ticks are uptime ms.
	# We check by comparing file modification time instead.
	var file_time := FileAccess.get_modified_time(SNAPSHOT_PATH)
	var wall_time := int(Time.get_unix_time_from_system())
	if wall_time - int(file_time) > 3:
		if _state == ConnectionState.CONNECTED:
			_set_state(ConnectionState.DISCONNECTED)
		return

	# Fresh snapshot — update provider and set connected
	_file_provider.update_snapshot(snapshot)

	if _state == ConnectionState.DISCONNECTED:
		_set_state(ConnectionState.CONNECTED)

	# Update status tick
	var tick: int = int(snapshot.get("tick", 0))
	_status_text.text = "🟢 %s (tick %d)" % [Locale.ltr("DEBUG_EDITOR_CONNECTED"), tick]

func _set_state(new_state: ConnectionState) -> void:
	_state = new_state

	match _state:
		ConnectionState.DISCONNECTED:
			_status_dot.text = "🔴"
			_status_text.text = Locale.ltr("DEBUG_EDITOR_DISCONNECTED")
			_show_placeholders(true)

		ConnectionState.CONNECTED:
			_status_dot.text = "🟢"
			_status_text.text = Locale.ltr("DEBUG_EDITOR_CONNECTED")
			_show_placeholders(false)

func _show_placeholders(show: bool) -> void:
	for wrapper in _live_wrappers:
		var placeholder := wrapper.get_node_or_null("Placeholder")
		var panel := wrapper.get_node_or_null("Panel")
		if placeholder:
			placeholder.visible = show
		if panel:
			panel.visible = not show
