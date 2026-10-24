# learning_helper.gd: data-driven ordered lesson card with persistent completion and explicit skips.
# Bridge methods: none; the coordinator supplies real read-only world facts and observed UI events.
extends PanelContainer

signal highlight_requested(target: String)
signal lesson_changed(id: String)
const Catalog = preload("res://scripts/ui/mux/controls_catalog.gd")
const Tooltips = preload("res://scripts/ui/panels/panel_tooltips.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
const THEME = preload("res://assets/ui/worldsim_theme.tres")
const FIELDS: Array[String] = ["id", "title_key", "body_key", "trigger", "completion", "highlight", "steward_key", "steward_rule"]
var lessons: Array[Dictionary] = []
var registry_error: String = ""
var _store: Store
var _facts: Dictionary = {}
var _events: Dictionary = {}
var _completed: PackedStringArray = []
var _skipped: PackedStringArray = []
var _observed: PackedStringArray = []
var _steward_observed: PackedStringArray = []
var _current: Dictionary = {}
var _title: Label
var _body: Label
var _status: Label
var _next: Button
var _skip: Button
var _off: Button
var _overlay_obscured: bool = false

func set_overlay_obscured(obscured: bool) -> void:
	if _overlay_obscured != obscured:
		_overlay_obscured = obscured
		_refresh()

func _ready() -> void:
	theme = THEME
	mouse_filter = Control.MOUSE_FILTER_STOP
	custom_minimum_size = Vector2(440, 0)
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 8)
	add_child(column)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	column.add_child(_title)
	_body = Label.new()
	_body.custom_minimum_size.x = 416
	_body.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	column.add_child(_body)
	_status = Label.new()
	_status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	column.add_child(_status)
	var buttons: HBoxContainer = HBoxContainer.new()
	buttons.add_theme_constant_override("separation", 8)
	column.add_child(buttons)
	_next = _button(buttons, advance)
	_next.name = "Next"
	_skip = _button(buttons, skip_current)
	_skip.name = "Skip"
	_off = _button(buttons, turn_off)
	_off.name = "Off"
	var locale: Node = get_node_or_null("/root/Locale")
	if locale != null:
		locale.connect("language_changed", _refresh.unbind(1))
	if lessons.is_empty():
		load_lessons("res://assets/ui/lessons.json")
	if _store == null:
		setup(Store.new())
	_refresh()

func setup(store: Store) -> void:
	if _store != null and _store.is_connected("changed", _setting_changed):
		_store.disconnect("changed", _setting_changed)
	_store = store
	var saved: Dictionary = _store.progress()
	_completed = saved.get("completed", PackedStringArray())
	_skipped = saved.get("skipped", PackedStringArray())
	_events = saved.get("events", {})
	_observed = saved.get("observed", PackedStringArray())
	_steward_observed = saved.get("steward_observed", PackedStringArray())
	_store.connect("changed", _setting_changed)
	_refresh()

func load_lessons(path: String) -> bool:
	var result: Dictionary = Catalog.read_registry(path, "lessons", FIELDS)
	registry_error = str(result["error"])
	lessons.assign(result["rows"])
	_refresh()
	return registry_error.is_empty()

func update_facts(facts: Dictionary) -> void:
	_facts = facts.duplicate(true)
	_capture_observed_completion()
	_refresh()

func observe(event_id: String) -> void:
	if event_id.is_empty() or _events.has(event_id):
		return
	_events[event_id] = true
	_capture_observed_completion()
	_save()
	_refresh()

func current_id() -> String:
	return str(_current.get("id", ""))

func can_advance() -> bool:
	return not _current.is_empty() and (_observed.has(current_id()) or Catalog.condition_met(_current["completion"], _facts, _events))

func advance() -> void:
	if not can_advance():
		return
	_completed.append(current_id())
	_save()
	_refresh()

func skip_current() -> void:
	if _current.is_empty():
		return
	_skipped.append(current_id())
	_save()
	_refresh()

func turn_off() -> void:
	if _store != null:
		_store.set_value("helper_on", false)

func _setting_changed(key: String, _value: Variant) -> void:
	if key == "helper_on":
		_refresh()

func _refresh() -> void:
	var previous: String = current_id()
	_current = {}
	for lesson: Dictionary in lessons:
		var id: String = str(lesson["id"])
		if not _completed.has(id) and not _skipped.has(id) and Catalog.condition_met(lesson["trigger"], _facts, _events):
			_current = lesson
			break
	if _title == null:
		return
	visible = not _overlay_obscured and _store != null and bool(_store.value("helper_on")) and not _current.is_empty()
	if not visible:
		highlight_requested.emit("")
		return
	_title.text = "%s · %s/%s" % [_loc(str(_current["title_key"])), _completed.size() + _skipped.size() + 1, lessons.size()]
	var by_steward: bool = _steward_observed.has(current_id())
	_body.text = _loc(str(_current["steward_key"] if by_steward else _current["body_key"]))
	_next.text = _loc("MUX_NEXT")
	_next.disabled = not can_advance()
	_skip.text = _loc("MUX_SKIP")
	_off.text = _loc("MUX_TURN_OFF")
	_status.text = _loc("MUX_LESSON_DONE" if can_advance() else "MUX_LESSON_WAIT")
	if previous != current_id():
		lesson_changed.emit(current_id())
	highlight_requested.emit(str(_current["highlight"]))

func _capture_observed_completion() -> void:
	# Training history survives world loads; goals still use current world facts.
	var dirty: bool = false
	var accepted: Variant = _facts.get("steward_accepted_rules", [])
	for lesson: Dictionary in lessons:
		if not Catalog.condition_met(lesson["trigger"], _facts, _events) or not Catalog.condition_met(lesson["completion"], _facts, _events):
			continue
		var id: String = str(lesson["id"])
		if not _observed.has(id):
			_observed.append(id)
			dirty = true
		var rule: String = str(lesson["steward_rule"])
		if rule.is_valid_int() and accepted.has(int(rule)) and not _steward_observed.has(id):
			_steward_observed.append(id)
			dirty = true
	if dirty:
		_save()

func _save() -> void:
	if _store != null:
		_store.save_progress(_completed, _skipped, _events, _observed, _steward_observed)

static func _button(row: HBoxContainer, callback: Callable) -> Button:
	var button: Button = Button.new()
	button.pressed.connect(callback)
	row.add_child(button)
	Tooltips.attach(button)
	return button

static func _loc(key: String) -> String:
	return Catalog._text(key, Callable())
