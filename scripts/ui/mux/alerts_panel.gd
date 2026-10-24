# Right-edge alert list; clicks cycle through causes and request selection from the HUD.
# Bridge methods: get_ui_facts via SimReads at most twice per second; world_reset signal.
extends PanelContainer

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
	_status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
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
	if is_instance_valid(_world) and evaluator.poll(delta, _read_facts):
		var rows: Array[Dictionary] = evaluator.active_alerts()
		if rows != _last_rows or evaluator.data_error != _last_error:
			_rebuild()


func _read_facts() -> Dictionary:
	return FactsAdapter.adapt(SimReads.read(_world, "get_ui_facts"))


func _tr(key: String) -> String:
	return str(_locale.call("ltr", key)) if _locale != null else key


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
	button.clip_text = true
	button.custom_minimum_size.y = 40.0
	button.text = "%s %s" % [alert["icon"], _tr("MUX_ALERTS_COUNT").format({"title": _tr(alert["text_key"]), "count": alert["count"]})]
	button.add_theme_color_override("font_color", Color(alert["color"]))
	button.tooltip_text = "%s\n%s\n%s" % [_tr(alert["text_key"]), _tr(alert["explanation_key"]), _tr(alert["fix_key"])]
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
