# settings_panel.gd: interface preferences backed by settings.cfg, not simulation saves.
# Bridge methods: none. The coordinator applies store.changed to camera and UI scale.
extends PanelContainer

signal defence_toggled(on: bool)
var _defence: CheckButton

const Catalog = preload("res://scripts/ui/mux/controls_catalog.gd")
const Tooltips = preload("res://scripts/ui/panels/panel_tooltips.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
const THEME = preload("res://assets/ui/worldsim_theme.tres")
const BOOLEAN_KEYS: Dictionary = {"edge_pan": "MUX_SETTING_EDGE_PAN", "invert_pan": "MUX_SETTING_INVERT_PAN", "helper_on": "MUX_SETTING_HELPER", "steward_default": "MUX_SETTING_STEWARD"}
var _store: Store
var _checks: Dictionary = {}
var _title: Label
var _scale_label: Label
var _scale: OptionButton
var _note: Label
var _error: Label
var _close: Button

func _ready() -> void:
	theme = THEME
	mouse_filter = Control.MOUSE_FILTER_STOP
	custom_minimum_size = Vector2(560, 0)
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 12)
	add_child(column)
	var row: HBoxContainer = HBoxContainer.new()
	column.add_child(row)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(_title)
	_close = Button.new()
	_close.pressed.connect(hide)
	row.add_child(_close)
	for key: String in BOOLEAN_KEYS:
		var check: CheckButton = CheckButton.new()
		check.name = key
		check.toggled.connect(_toggle_setting.bind(key))
		column.add_child(check)
		_checks[key] = check
		Tooltips.attach(check, "", str(BOOLEAN_KEYS[key]))
	_defence = CheckButton.new()
	_defence.name = "StewardDefence"
	_defence.toggled.connect(func(on: bool) -> void: defence_toggled.emit(on))
	column.add_child(_defence)
	var scale_row: HBoxContainer = HBoxContainer.new()
	column.add_child(scale_row)
	_scale_label = Label.new()
	_scale_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scale_row.add_child(_scale_label)
	_scale = OptionButton.new()
	_scale.name = "UIScale"
	for factor: float in [1.0, 1.25, 1.5]:
		_scale.add_item("%s×" % factor)
	_scale.item_selected.connect(_scale_selected)
	scale_row.add_child(_scale)
	Tooltips.attach(_scale, "", "MUX_SETTING_UI_SCALE")
	_note = Label.new()
	_note.custom_minimum_size.x = 520
	_note.autowrap_mode = TextServer.AUTOWRAP_WORD
	column.add_child(_note)
	_error = Label.new()
	_error.autowrap_mode = TextServer.AUTOWRAP_WORD
	column.add_child(_error)
	var locale: Node = get_node_or_null("/root/Locale")
	if locale != null:
		locale.connect("language_changed", _refresh.unbind(1))
	if _store == null:
		setup(Store.new())
	_refresh()
	hide()

func setup(store: Store) -> void:
	if _store != null and _store.is_connected("changed", _store_changed):
		_store.disconnect("changed", _store_changed)
	_store = store
	_store.changed.connect(_store_changed)
	_refresh()

func toggle_panel() -> void:
	visible = not visible
	if visible:
		center_in_viewport()

func center_in_viewport() -> void:
	position = ((get_viewport_rect().size - size * scale) * 0.5).max(Vector2(8, 8))

func _toggle_setting(enabled: bool, key: String) -> void:
	_store.set_value(key, enabled)

func _scale_selected(index: int) -> void:
	var factors: Array[float] = [1.0, 1.25, 1.5]
	_store.set_value("ui_scale", factors[index])

func _store_changed(_key: String, _value: Variant) -> void:
	_refresh()

func _refresh() -> void:
	if _title == null or _store == null:
		return
	_title.text = _loc("MUX_SETTINGS_TITLE")
	_close.text = _loc("MUX_CLOSE")
	_close.tooltip_text = "Esc"
	_scale_label.text = _loc("MUX_SETTING_UI_SCALE")
	_defence.text = _loc("MUX_STEWARD_DEFENCE")
	_defence.tooltip_text = _loc("MUX_STEWARD_DEFENCE_TIP")
	_note.text = _loc("MUX_SETTING_STEWARD_NOTE")
	for key: String in BOOLEAN_KEYS:
		var check: CheckButton = _checks[key]
		check.text = _loc(str(BOOLEAN_KEYS[key]))
		check.set_pressed_no_signal(bool(_store.value(key)))
	var factors: Array[float] = [1.0, 1.25, 1.5]
	_scale.select(factors.find(float(_store.value("ui_scale"))))
	_error.text = _loc("MUX_SETTINGS_SAVE_FAILED") if _store.last_error != OK else ""

func _unhandled_key_input(event: InputEvent) -> void:
	if visible and event is InputEventKey and event.pressed and not event.is_echo() and event.keycode == KEY_ESCAPE:
		hide()
		get_viewport().set_input_as_handled()

static func _loc(key: String) -> String:
	return Catalog._text(key, Callable())

func set_defence_state(on: bool) -> void:
	if _defence != null:
		_defence.set_pressed_no_signal(on)
