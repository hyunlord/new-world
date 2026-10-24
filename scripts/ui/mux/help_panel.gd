# help_panel.gd: scrollable controls overlay generated from the shared controls registry.
# Bridge methods: none. The same catalog renders README's generated control table.
extends PanelContainer

signal opened
const Catalog = preload("res://scripts/ui/mux/controls_catalog.gd")
const THEME = preload("res://assets/ui/worldsim_theme.tres")
var catalog: Catalog = Catalog.new()
var _title: Label
var _note: Label
var _rows: GridContainer
var _close: Button
var _scroll: ScrollContainer
var _fitting: bool = false

func _ready() -> void:
	theme = THEME
	var surface: StyleBoxFlat = get_theme_stylebox("panel").duplicate() as StyleBoxFlat
	surface.bg_color.a = 1.0
	add_theme_stylebox_override("panel", surface)
	mouse_filter = Control.MOUSE_FILTER_STOP
	clip_contents = true
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 12)
	add_child(column)
	var heading: HBoxContainer = HBoxContainer.new()
	column.add_child(heading)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	heading.add_child(_title)
	_close = Button.new()
	_close.name = "Close"
	_close.pressed.connect(hide)
	heading.add_child(_close)
	_note = Label.new()
	_note.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	column.add_child(_note)
	_scroll = ScrollContainer.new()
	_scroll.name = "ControlsScroll"
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	_scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_SHOW_ALWAYS
	_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	column.add_child(_scroll)
	_rows = GridContainer.new()
	_rows.name = "ControlRows"
	_rows.columns = 2
	_rows.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_rows.add_theme_constant_override("h_separation", 24)
	_rows.add_theme_constant_override("v_separation", 12)
	_scroll.add_child(_rows)
	var locale: Node = get_node_or_null("/root/Locale")
	if locale != null:
		locale.connect("language_changed", _refresh.unbind(1))
	_refresh()
	minimum_size_changed.connect(_on_minimum_size_changed)
	get_viewport().size_changed.connect(center_in_viewport)
	hide()

func toggle_panel() -> void:
	visible = not visible
	if visible:
		center_in_viewport()
		_close.grab_focus()
		opened.emit()

func center_in_viewport() -> void:
	if _rows == null or _fitting:
		return
	_fitting = true
	var available: Vector2 = (get_viewport_rect().size - Vector2(24, 24)) / scale
	var desired: Vector2 = Vector2(minf(1000, available.x), minf(680, available.y))
	var content_width: float = maxf(200, desired.x - 68)
	for index: int in _rows.get_child_count():
		var label: Label = _rows.get_child(index) as Label
		label.custom_minimum_size.x = content_width * (0.42 if index % 2 == 0 else 0.58)
	custom_minimum_size = desired
	reset_size()
	size = desired
	position = ((get_viewport_rect().size - size * scale) * 0.5).max(Vector2(8, 8))
	_fitting = false

func _on_minimum_size_changed() -> void:
	# Containers can release their initial wrapped-text minimum after opening.
	# Shrink now, rather than waiting for the HUD's half-second layout poll.
	center_in_viewport()

func _refresh() -> void:
	_title.text = Catalog._text("MUX_HELP_TITLE", Callable())
	_note.text = Catalog._text("MUX_HELP_MAC", Callable())
	_close.text = Catalog._text("MUX_CLOSE", Callable())
	_close.tooltip_text = "Esc"
	for child: Node in _rows.get_children():
		_rows.remove_child(child)
		child.queue_free()
	for entry: Dictionary in catalog.entries:
		var input: Label = Label.new()
		input.text = Catalog.input_text(entry)
		input.custom_minimum_size.x = 360
		input.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		_rows.add_child(input)
		var action: Label = Label.new()
		action.text = Catalog._text(str(entry["text_key"]), Callable())
		action.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		action.custom_minimum_size.x = 540
		action.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		_rows.add_child(action)
	center_in_viewport.call_deferred()

func _unhandled_key_input(event: InputEvent) -> void:
	if not event is InputEventKey or not event.pressed or event.is_echo():
		return
	var key: InputEventKey = event
	if key.keycode == KEY_SLASH:
		toggle_panel()
		get_viewport().set_input_as_handled()
	elif key.keycode == KEY_ESCAPE and visible:
		hide()
		get_viewport().set_input_as_handled()
