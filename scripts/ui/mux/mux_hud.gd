# M-UX screen coordinator: controls, alerts, learning, goals and recorded steward commands.
# Direct bridge: get_steward (attach/reset baseline), get_chronicle_since (independent incremental notification cursor).
# Bridge methods through SimReads: get_ui_facts, get_steward, cmd_set_steward, cmd_set_steward_defence and learning facts reads.
extends Control

const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")

const Catalog = preload("res://scripts/ui/mux/controls_catalog.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
const Controls = preload("res://scripts/ui/mux/map_controls.gd")
const Map = preload("res://scripts/ui/mux/minimap.gd")
const Help = preload("res://scripts/ui/mux/help_panel.gd")
const Lessons = preload("res://scripts/ui/mux/learning_helper.gd")
const Goals = preload("res://scripts/ui/mux/goals_panel.gd")
const Settings = preload("res://scripts/ui/mux/settings_panel.gd")
const Alerts = preload("res://scripts/ui/mux/alerts_panel.gd")
const LearningFacts = preload("res://scripts/ui/mux/learning_facts.gd")
const ChronicleText = preload("res://scripts/ui/chronicle_text.gd")
const LessonHighlight = preload("res://scripts/ui/mux/lesson_highlight.gd")
const LegacyLayout = preload("res://scripts/ui/mux/legacy_layout.gd")
const THEME = preload("res://assets/ui/worldsim_theme.tres")
const LEARNING_PANELS: Dictionary = {"AgentInspectorPanel": "person_selected", "PriorityPanel": "priorities_opened", "RoomPanel": "rooms_opened", "TechPanel": "tech_opened", "NeighboursPanel": "neighbours_opened"}
var store: Store = Store.new()
var catalog: Catalog = Catalog.new()
var controls: Controls
var minimap: Map
var helper: Lessons
var help: Help
var goals: Goals
var settings: Settings
var alerts: Alerts
var _world: Node
var _camera: Camera2D
var _right: VBoxContainer
var _actions: HBoxContainer
var _steward: OptionButton
var _poll_left: float = 0.0
var _sequence: int = -1
var _chronicle_cursor: int = -1
var _pending_native_resets: int = 0
var _highlight: LessonHighlight
var _why_button: Button
var _help_button: Button
var _goals_button: Button
var _settings_button: Button
var _legacy: LegacyLayout
var _mux2_profile: bool = OS.get_environment("WORLDSIM_MUX2_PROFILE") == "1"
var _mux2_frame: int = -1
var _mux2_usec: Dictionary = {}

func _ready() -> void:
	theme = THEME
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_world = get_node("/root/Main/WorldSim")
	_camera = get_node("/root/Main/Camera2D") as Camera2D
	_legacy = LegacyLayout.new()
	add_child(_legacy)
	_legacy.setup(get_parent())
	_right = VBoxContainer.new()
	_right.name = "MapDock"
	_right.add_theme_constant_override("separation", 8)
	add_child(_right)
	minimap = Map.new()
	minimap.name = "Minimap"
	_right.add_child(minimap)
	minimap.setup(_world, _camera)
	controls = Controls.new()
	controls.name = "MapControls"
	_right.add_child(controls)
	controls.setup(_camera, catalog)
	helper = Lessons.new()
	helper.name = "LearningHelper"
	helper.setup(store)
	add_child(helper)
	alerts = Alerts.new()
	alerts.name = "Alerts"
	alerts.setup(_world)
	add_child(alerts)
	_actions = HBoxContainer.new()
	_actions.name = "Actions"
	_actions.add_theme_constant_override("separation", 8)
	add_child(_actions)
	_steward = OptionButton.new()
	_steward.name = "Steward"
	_steward.item_selected.connect(_set_steward)
	_actions.add_child(_steward)
	help = Help.new()
	help.name = "Help"
	help.catalog = catalog
	add_child(help)
	goals = Goals.new()
	goals.name = "Goals"
	add_child(goals)
	settings = Settings.new()
	settings.name = "Settings"
	settings.setup(store)
	add_child(settings)
	settings.defence_toggled.connect(_set_defence)
	_why_button = _button("WhyButton", func() -> void: get_node("/root/Main/ColonyView/OrderTool").call("select_why"))
	_goals_button = _button("GoalsButton", goals.toggle_panel)
	_settings_button = _button("SettingsButton", settings.toggle_panel)
	_help_button = _button("HelpButton", help.toggle_panel)
	for entry: Array in [["PrioritiesButton", "P", "PriorityPanel", "toggle_visible", "priorities"], ["TechButton", "T", "TechPanel", "open", "tech"], ["NeighboursButton", "N", "NeighboursPanel", "open", "neighbours"]]:
		var button: Button = _button(str(entry[0]), _open_panel.bind(str(entry[2]), str(entry[3])))
		button.text = str(entry[1])
		button.set_meta("mux_control", str(entry[4]))
		button.tooltip_text = catalog.tooltip(str(entry[4]))
	minimap.map_used.connect(func() -> void: helper.observe("minimap_used"))
	_camera.connect("camera_activity", _camera_activity)
	alerts.alert_activated.connect(func(_id: String) -> void: helper.observe("alerts_used"))
	alerts.jump_requested.connect(jump_to)
	_highlight = LessonHighlight.new()
	add_child(_highlight)
	_highlight.setup(get_parent())
	helper.highlight_requested.connect(_highlight.show_target)
	get_parent().child_entered_tree.connect(_learning_panel_entered)
	for child: Node in get_parent().get_children():
		_learning_panel_entered(child)
	store.changed.connect(_setting_changed)
	get_node("/root/Locale").connect("language_changed", _locale_changed)
	get_viewport().size_changed.connect(_layout)
	_world.connect("world_reset", _world_reset)
	var reads: Node = SimReads.resolve(_world)
	if reads != null:
		reads.connect("world_replaced", _world_replaced)
		reads.connect("new_game_started", _new_game_preferences)
	_locale_changed("")
	_apply_preferences()
	_new_game_preferences()
	_baseline_notifications()
	_layout.call_deferred()

func _button(node_name: String, action: Callable) -> Button:
	var button: Button = Button.new()
	button.name = node_name
	button.pressed.connect(action)
	_actions.add_child(button)
	return button

func _open_panel(node_name: String, method: String) -> void:
	var panel: Node = get_parent().get_node_or_null(node_name)
	if panel != null:
		panel.call(method)

func _locale_changed(_language: String) -> void:
	var selected: int = _steward.selected
	_steward.clear()
	for key: String in ["MUX_STEWARD_ALL", "MUX_STEWARD_HOUSING", "MUX_STEWARD_FOOD", "MUX_STEWARD_OFF"]:
		_steward.add_item(_loc(key))
	_steward.add_separator()
	_steward.add_item(_loc("MUX_STEWARD_DEFENCE"))
	_steward.get_popup().set_item_as_checkable(5, true)
	_steward.select(clampi(selected, 0, 3))
	_why_button.toggle_mode = true
	_why_button.text = _loc("MUX_WHY")
	_why_button.tooltip_text = catalog.tooltip("why")
	_steward.tooltip_text = _loc("MUX_STEWARD_TIP")
	_help_button.text = "?"
	_help_button.tooltip_text = catalog.tooltip("help")
	_goals_button.text = _loc("MUX_GOALS_TITLE")
	_goals_button.tooltip_text = catalog.tooltip("goals")
	_settings_button.text = _loc("MUX_SETTINGS_TITLE")
	_settings_button.tooltip_text = _loc("MUX_SETTINGS_TITLE")
	for child: Node in _actions.get_children():
		if child is Button and child.has_meta("mux_control"):
			(child as Button).tooltip_text = catalog.tooltip(str(child.get_meta("mux_control")))
			for entry: Dictionary in catalog.entries:
				if str(entry["id"]) == str(child.get_meta("mux_control")):
					(child as Button).text = "%s (%s)" % [_loc(str(entry["text_key"])), Catalog.input_text(entry)]
	_layout.call_deferred()

func _layout() -> void:
	if not _mux2_profile:
		_layout_widgets()
		return
	var started: int = Time.get_ticks_usec()
	_layout_widgets()
	_mux2_record("layout", Time.get_ticks_usec() - started)

func _layout_widgets() -> void:
	if _right == null:
		return
	var area: Vector2 = get_viewport_rect().size
	var factor: float = float(store.value("ui_scale"))
	_legacy.apply_scale(factor, area)
	_right.size = Vector2.ZERO
	_actions.size = Vector2.ZERO
	var actions_factor: float = minf(factor, (area.x - 24) / maxf(1, _actions.get_combined_minimum_size().x))
	_actions.scale = Vector2.ONE * actions_factor
	helper.size = Vector2.ZERO
	_right.position = Vector2(area.x - _right.get_combined_minimum_size().x * factor - 12, _legacy.bar_top() - _right.get_combined_minimum_size().y * factor - 12)
	_actions.position = Vector2(area.x - _actions.get_combined_minimum_size().x * actions_factor - 12, _legacy.top_bottom() + 8)
	alerts.position = Vector2(area.x - alerts.get_combined_minimum_size().x * factor - 12, _actions.position.y + _actions.get_combined_minimum_size().y * actions_factor + 8)
	alerts.set_available_height((_right.position.y - alerts.position.y - 12) / factor)
	var bottom: float = _legacy.bar_top() - 64
	helper.position = Vector2(12, bottom - helper.get_combined_minimum_size().y * factor)
	_legacy.set_content_top(alerts.position.y)
	_legacy.apply_scale(factor, area, minf(alerts.position.x, _right.position.x) - 12)
	for panel: Control in [help, goals, settings]:
		if panel.visible:
			panel.call("center_in_viewport")

func _camera_activity(action: String) -> void:
	var events: Dictionary = {"pan": "camera_moved", "zoom": "camera_zoomed", "speed": "speed_changed"}
	if events.has(action):
		helper.observe(str(events[action]))

func _setting_changed(_key: String, _value: Variant) -> void:
	_apply_preferences()

func _apply_preferences() -> void:
	_camera.call("set_edge_pan", bool(store.value("edge_pan")))
	_camera.call("set_invert_pan", bool(store.value("invert_pan")))
	var factor: float = float(store.value("ui_scale"))
	for panel: Control in [helper, alerts, help, goals, settings, _right, _actions]:
		panel.scale = Vector2.ONE * factor
	_layout.call_deferred()

func _set_steward(index: int) -> void:
	if index == 5:
		var current: Dictionary = SimReads.read(_world, "get_steward")
		_set_defence(not bool(current.get("defence_enabled", true)))
		_steward.select(int(current.get("scope", 0)) if bool(current.get("enabled", false)) else 3)
		return
	SimReads.mutate(_world, "cmd_set_steward", [index != 3, index if index < 3 else 0])
	_poll_left = 0.0

func _set_defence(on: bool) -> void:
	SimReads.mutate(_world, "cmd_set_steward_defence", [on])
	_poll_left = 0.0

func _world_replaced() -> void:
	_pending_native_resets += 1
	_baseline_notifications()
	_poll_left = 0.0

func _world_reset() -> void:
	if _pending_native_resets > 0:
		_pending_native_resets -= 1
		return
	# Direct bridge replacement fallback; normal UI commands use SimReads receipts.
	_baseline_notifications()
	_poll_left = 0.0

func _baseline_notifications() -> void:
	var state: Dictionary = _world.get_steward()
	var page: Dictionary = _world.get_chronicle_since(0, 1)
	_sequence = int(state.get("sequence", 0))
	_chronicle_cursor = int(page.get("total", 0))
	var bar: Node = get_parent().get_node_or_null("OrderBar")
	if bar != null:
		bar.call("clear_background_toasts")

func _new_game_preferences() -> void:
	if not bool(store.value("steward_default")) and _world.has_method("cmd_set_steward"):
		SimReads.mutate(_world, "cmd_set_steward", [false, 0])

func _process(delta: float) -> void:
	if not _mux2_profile:
		_process_hud(delta)
		return
	var started: int = Time.get_ticks_usec()
	_process_hud(delta)
	_mux2_record("process", Time.get_ticks_usec() - started)

## Read-only frame-local inclusive spans; facts/layout/steward overlap whole process.
func mux2_frame_timings() -> Dictionary:
	if not _mux2_profile: return {"status": "DISABLED"}
	_mux2_sync_frame()
	return {"status": "MEASURED", "frame": _mux2_frame, "usec": _mux2_usec.duplicate()}

func _mux2_sync_frame() -> void:
	var frame: int = Engine.get_process_frames()
	if frame != _mux2_frame:
		_mux2_frame = frame
		_mux2_usec = {"process": 0, "steward": 0, "facts": 0, "learning": 0, "layout": 0}

func _mux2_record(metric: String, elapsed: int) -> void:
	_mux2_sync_frame()
	_mux2_usec[metric] = int(_mux2_usec.get(metric, 0)) + elapsed

func _process_hud(delta: float) -> void:
	_sync_foreground()
	_poll_left -= delta
	if _poll_left > 0.0:
		return
	_poll_left = 0.5
	var started: int = Time.get_ticks_usec() if _mux2_profile else 0
	var state: Dictionary = {}
	if _world.has_method("get_steward"):
		state = SimReads.read(_world, "get_steward")
		ChronicleText.steward_labels = state.get("rules", {})
		_steward.select(int(state.get("scope", 0)) if bool(state.get("enabled", false)) else 3)
		_steward.get_popup().set_item_checked(5, bool(state.get("defence_enabled", true)))
		settings.set_defence_state(bool(state.get("defence_enabled", true)))
		_poll_steward_notifications(state)
	if _mux2_profile:
		_mux2_record("steward", Time.get_ticks_usec() - started)
		started = Time.get_ticks_usec()
	_why_button.set_pressed_no_signal(bool(get_node("/root/Main/ColonyView/OrderTool").get("why_selecting")))
	var facts: Dictionary = LearningFacts.collect(_world, state)
	if _mux2_profile:
		_mux2_record("facts", Time.get_ticks_usec() - started)
		started = Time.get_ticks_usec()
	helper.update_facts(facts)
	goals.update_facts(facts)
	if _mux2_profile: _mux2_record("learning", Time.get_ticks_usec() - started)
	var drafted: Dictionary = SimReads.read(_world, "get_drafted")
	if not drafted.get("ids", PackedInt64Array()).is_empty():
		helper.observe("drafted")
	_layout()
	_sync_foreground()

func _sync_foreground() -> void:
	if not is_instance_valid(helper) or not is_instance_valid(_highlight):
		return
	var menu_open: bool = false
	var bar: Node = get_parent().get_node_or_null("OrderBar")
	if bar != null:
		for child: Node in bar.get_children():
			if child is PanelContainer and child.name in [&"BuildMenu", &"DigMenu"] and child.is_visible_in_tree():
				menu_open = true
	helper.set_overlay_obscured(menu_open)
	_highlight.visible = not menu_open and not help.visible and not goals.visible and not settings.visible

func _learning_panel_entered(node: Node) -> void:
	if not node is Control or not LEARNING_PANELS.has(str(node.name)):
		return
	var panel: Control = node as Control
	# Deferred legacy panels hide during _ready; do not record their initial default visibility.
	var connect_panel: Callable = _watch_learning_panel.bind(panel)
	if panel.is_node_ready():
		_watch_learning_panel(panel)
	elif not panel.ready.is_connected(connect_panel):
		panel.ready.connect(connect_panel, CONNECT_ONE_SHOT)

func _watch_learning_panel(panel: Control) -> void:
	var observe_panel: Callable = _learning_panel_visibility_changed.bind(panel)
	if not panel.visibility_changed.is_connected(observe_panel):
		panel.visibility_changed.connect(observe_panel)
	_learning_panel_visibility_changed(panel)

func _learning_panel_visibility_changed(panel: Control) -> void:
	if panel.is_visible_in_tree():
		helper.observe(str(LEARNING_PANELS[str(panel.name)]))

func jump_to(target: Dictionary) -> void:
	_camera.call("center_on_tile", Vector2i(int(target.get("x", 0)), int(target.get("y", 0))))
	var kind: String = str(target.get("kind", ""))
	if kind in ["person", "household"]:
		var inspector: Node = get_parent().get_node_or_null("AgentInspectorPanel")
		if inspector != null:
			inspector.call("show_person", int(target.get("id", -1)))
	elif kind in ["technique", "knowledge"]:
		var panel: Node = get_parent().get_node_or_null("TechPanel")
		if panel != null:
			panel.call("open")
			panel.call("select_node", int(target.get("node_id", target.get("id", -1))))
	elif kind == "room":
		var panel: Node = get_parent().get_node_or_null("RoomPanel")
		if panel != null:
			panel.call("display_room", int(target.get("id", -1)))
	elif kind in ["migrant", "trader"]:
		var panel: Node = get_parent().get_node_or_null("MigrantPanel" if kind == "migrant" else "TradePanel")
		if panel != null:
			panel.call("open_wave" if kind == "migrant" else "open_caravan", int(target.get("id", -1)))
	else:
		pass # Tile alerts navigate; opening an explanation requires an explicit why action.

func _loc(key: String) -> String:
	return KoreanText.words(str(get_node("/root/Locale").call("ltr", key)))

func _poll_steward_notifications(state: Dictionary) -> void:
	var sequence: int = int(state.get("sequence", 0))
	# Defence appends chronicle rows independently of the basic-order sequence.
	# The paged cursor, not sequence equality, determines which notifications are new.
	var page: Dictionary = _world.get_chronicle_since(maxi(0, _chronicle_cursor), 1 if _chronicle_cursor < 0 else 512)
	if _chronicle_cursor < 0 or sequence < _sequence:
		# Attach/load baseline: old save history is already available in ChroniclePanel.
		_baseline_notifications()
		return
	var bar: Node = get_parent().get_node_or_null("OrderBar")
	if bar == null:
		return # Retain the cursor until a destination exists.
	var labels: Dictionary = state.get("rules", {})
	for row: Dictionary in ChronicleText.rows(page):
		if int(row.get("kind", -1)) == 46:
			bar.call("queue_background_toast", _loc("MUX_STEWARD_DEFENCE_ACTION").format({"count": int(row.get("value", 0))}), int(row.get("index", 0)))
			continue
		if int(row.get("kind", -1)) != 45 or int(row.get("value", -1)) <= _sequence:
			continue
		_sequence = int(row["value"])
		var key: String = str(labels.get(int(row["cause"]), "UI_UNKNOWN"))
		if key.strip_edges().is_empty():
			key = "UI_UNKNOWN"
		bar.call("queue_background_toast", _loc("MUX_STEWARD_ACTION") + " " + _loc(key), _sequence)
	_chronicle_cursor = int(page.get("next", _chronicle_cursor))
	if _chronicle_cursor >= int(page.get("total", _chronicle_cursor)):
		_sequence = sequence
