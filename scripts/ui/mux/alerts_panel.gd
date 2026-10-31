# Right-edge alert list; clicks cycle through causes and request selection from the HUD.
# Bridge methods: get_ui_facts via SimReads at most twice per second; world_reset signal.
extends PanelContainer

const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")

signal jump_requested(target: Dictionary)
signal alert_activated(id: String)

const FactsAdapter = preload("res://scripts/ui/mux/ui_facts_adapter.gd")
const Evaluator = preload("res://scripts/ui/mux/alert_evaluator.gd")
const THEME = preload("res://assets/ui/worldsim_theme.tres")

var evaluator: Evaluator = Evaluator.new()
var _world: Node
var _locale: Node
var _list: VBoxContainer
var _status: Label
var _title: Label
var _scroll: ScrollContainer
var _last_rows: Array[Dictionary] = []
var _last_error: String = ""
var _available_height: float = 400.0
var _mux2_profile: bool = OS.get_environment("WORLDSIM_MUX2_PROFILE") == "1"
var _mux2_frame: int = -1
var _mux2_usec: Dictionary = {}


func set_available_height(height: float) -> void:
	_available_height = maxf(120, height)
	_fit_rows()


func _fit_rows() -> void:
	if not is_instance_valid(_scroll):
		return
	var available: float = maxf(40, _available_height - 64)
	var rows_height: float = 0.0
	for child: Node in _list.get_children():
		var row_height: float = (child as Control).get_combined_minimum_size().y
		var next_height: float = rows_height + (4 if rows_height > 0 else 0) + row_height
		if next_height > available:
			break
		rows_height = next_height
	_scroll.custom_minimum_size.y = maxf(40, rows_height)
	size = Vector2.ZERO


func setup(world: Node) -> void:
	if is_instance_valid(_world) and _world.has_signal("world_reset") and _world.is_connected("world_reset", _reset):
		_world.disconnect("world_reset", _reset)
	_world = world
	if is_instance_valid(_world) and _world.has_signal("world_reset"):
		_world.connect("world_reset", _reset)
	evaluator.reset()


func _ready() -> void:
	theme = THEME
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	custom_minimum_size.x = 360.0
	mouse_filter = Control.MOUSE_FILTER_STOP
	_locale = get_node_or_null("/root/Locale")
	if _locale != null and _locale.has_signal("language_changed"):
		_locale.connect("language_changed", _language_changed)
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 8)
	add_child(column)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	column.add_child(_title)
	_status = Label.new()
	_status.autowrap_mode = TextServer.AUTOWRAP_WORD
	column.add_child(_status)
	_scroll = ScrollContainer.new()
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_scroll.custom_minimum_size = Vector2(360, 320)
	column.add_child(_scroll)
	_list = VBoxContainer.new()
	_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_list.add_theme_constant_override("separation", 4)
	_scroll.add_child(_list)
	if not evaluator.load_registry():
		push_error(evaluator.last_error)
	if _world == null:
		setup(get_node_or_null("/root/Main/WorldSim"))
	_rebuild()


func _process(delta: float) -> void:
	if not _mux2_profile:
		_process_alerts(delta)
		return
	var started: int = Time.get_ticks_usec()
	_process_alerts(delta)
	_mux2_record("process", Time.get_ticks_usec() - started)


## Read-only frame-local inclusive timings; poll includes fact adaptation and evaluation.
func mux2_frame_timings() -> Dictionary:
	if not _mux2_profile: return {"status": "DISABLED"}
	_mux2_sync_frame()
	return {"status": "MEASURED", "frame": _mux2_frame, "usec": _mux2_usec.duplicate()}


func _mux2_sync_frame() -> void:
	var frame: int = Engine.get_process_frames()
	if frame != _mux2_frame:
		_mux2_frame = frame
		_mux2_usec = {"process": 0, "poll": 0, "rebuild": 0}


func _mux2_record(metric: String, elapsed: int) -> void:
	_mux2_sync_frame()
	_mux2_usec[metric] = int(_mux2_usec.get(metric, 0)) + elapsed


func _process_alerts(delta: float) -> void:
	if not is_instance_valid(_world): return
	var started: int = Time.get_ticks_usec() if _mux2_profile else 0
	var refreshed: bool = evaluator.poll(delta, _read_facts)
	if _mux2_profile: _mux2_record("poll", Time.get_ticks_usec() - started)
	if refreshed:
		var rows: Array[Dictionary] = evaluator.active_alerts()
		if rows != _last_rows or evaluator.data_error != _last_error:
			_rebuild()


func _read_facts() -> Dictionary:
	return FactsAdapter.adapt(SimReads.read(_world, "get_ui_facts"))


func _tr(key: String) -> String:
	return KoreanText.words(str(_locale.call("ltr", key))) if _locale != null else key


func _language_changed(_language: String) -> void:
	_rebuild()


func _reset() -> void:
	evaluator.reset()
	_rebuild()


func refresh_from_facts(facts: Dictionary) -> bool:
	var valid: bool = evaluator.evaluate(facts)
	_rebuild()
	return valid


func _rebuild() -> void:
	if not _mux2_profile:
		_rebuild_rows()
		return
	var started: int = Time.get_ticks_usec()
	_rebuild_rows()
	_mux2_record("rebuild", Time.get_ticks_usec() - started)


func _rebuild_rows() -> void:
	if not is_instance_valid(_list):
		return
	for child: Node in _list.get_children():
		_list.remove_child(child)
		child.queue_free()
	_title.text = _tr("MUX_ALERTS_TITLE")
	_last_rows = evaluator.active_alerts()
	_last_error = evaluator.data_error
	var invalid: bool = not evaluator.last_error.is_empty() or not evaluator.data_error.is_empty()
	_status.visible = invalid or _last_rows.is_empty()
	_status.text = _tr("MUX_ALERTS_INVALID" if invalid else "MUX_ALERTS_EMPTY")
	_scroll.visible = not _last_rows.is_empty()
	for alert: Dictionary in _last_rows:
		_add_alert(alert)
	_fit_rows.call_deferred()


func _add_alert(alert: Dictionary) -> void:
	var row: HBoxContainer = HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)
	_list.add_child(row)
	var button: Button = Button.new()
	button.name = str(alert["id"])
	button.alignment = HORIZONTAL_ALIGNMENT_LEFT
	button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	button.clip_text = false
	button.autowrap_mode = TextServer.AUTOWRAP_WORD
	button.custom_minimum_size.x = 280.0
	button.custom_minimum_size.y = 40.0
	button.text = "%s %s" % [alert["icon"], _tr("MUX_ALERTS_COUNT").format({"title": _tr(alert["text_key"]).format(alert.get("facts", {})), "count": alert["count"]})]
	button.add_theme_color_override("font_color", Color(alert["color"]))
	button.tooltip_text = "%s\n%s\n%s" % [_tr(alert["text_key"]).format(alert.get("facts", {})), _tr(alert["explanation_key"]).format(alert.get("facts", {})), _tr(alert["fix_key"]).format(alert.get("facts", {}))]
	button.pressed.connect(activate.bind(str(alert["id"])))
	row.add_child(button)
	var dismiss_button: Button = Button.new()
	dismiss_button.text = "×"
	dismiss_button.custom_minimum_size = Vector2(40, 40)
	dismiss_button.tooltip_text = _tr("MUX_ALERTS_DISMISS_TIP")
	dismiss_button.pressed.connect(dismiss.bind(str(alert["id"])))
	row.add_child(dismiss_button)


func activate(id: String) -> void:
	var target: Dictionary = evaluator.next_target(id)
	if not target.is_empty():
		jump_requested.emit(target)
		alert_activated.emit(id)


func dismiss(id: String) -> void:
	evaluator.dismiss(id)
	_rebuild()


## M-READ: the formatted text of every currently active alert (for plaque deduplication).
func active_alert_texts() -> PackedStringArray:
	var texts: PackedStringArray = PackedStringArray()
	for alert: Dictionary in _last_rows:
		var key: String = str(alert.get("text_key", ""))
		var facts: Dictionary = alert.get("facts", {})
		if not key.is_empty():
			texts.append(_tr(key).format(facts))
	return texts
