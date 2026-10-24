# goals_panel.gd: data-driven first-year goals and registry-supplied era marker progress.
# Bridge methods: none; the coordinator supplies authoritative world facts and era rows.
extends PanelContainer

signal opened
const Catalog = preload("res://scripts/ui/mux/controls_catalog.gd")
const THEME = preload("res://assets/ui/worldsim_theme.tres")
var goals: Array[Dictionary] = []
var _facts: Dictionary = {}
var _labels: Array[Label] = []
var _title: Label
var _era: Label
var _progress: ProgressBar
var _markers: Label
var _close: Button

func _ready() -> void:
	theme = THEME
	mouse_filter = Control.MOUSE_FILTER_STOP
	custom_minimum_size = Vector2(560, 0)
	var result: Dictionary = Catalog.read_registry("res://assets/ui/goals.json", "goals", ["id", "text_key", "completion", "group"])
	goals.assign(result["rows"])
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
	for _goal: Dictionary in goals:
		var label: Label = Label.new()
		label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		label.custom_minimum_size.x = 520
		column.add_child(label)
		_labels.append(label)
	_era = Label.new()
	column.add_child(_era)
	_progress = ProgressBar.new()
	_progress.max_value = 1000.0
	_progress.show_percentage = false
	_progress.custom_minimum_size.y = 20
	column.add_child(_progress)
	_markers = Label.new()
	_markers.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	column.add_child(_markers)
	var locale: Node = get_node_or_null("/root/Locale")
	if locale != null:
		locale.connect("language_changed", _refresh.unbind(1))
	_refresh()
	hide()

func update_facts(facts: Dictionary) -> void:
	_facts = facts.duplicate(true)
	_refresh()

func toggle_panel() -> void:
	visible = not visible
	if visible:
		center_in_viewport()
		opened.emit()

func center_in_viewport() -> void:
	position = ((get_viewport_rect().size - size * scale) * 0.5).max(Vector2(8, 8))

func goal_complete(id: String) -> bool:
	for goal: Dictionary in goals:
		if str(goal["id"]) == id:
			return Catalog.condition_met(goal["completion"], _facts, {})
	return false

func _refresh() -> void:
	if _title == null:
		return
	_title.text = _loc("MUX_GOALS_TITLE")
	_close.text = _loc("MUX_CLOSE")
	_close.tooltip_text = "Esc"
	for index: int in goals.size():
		var goal: Dictionary = goals[index]
		_labels[index].text = "%s %s" % ["✓" if goal_complete(str(goal["id"])) else "□", _loc(str(goal["text_key"]))]
	var progress: int = clampi(int(_facts.get("era_progress_permille", 0)), 0, 1000)
	_era.text = "%s · %s/1000" % [_loc("MUX_GOALS_ERA"), progress]
	_progress.value = progress
	var lines: PackedStringArray = [_loc("MUX_GOALS_MARKERS")]
	var markers: Variant = _facts.get("era_markers", [])
	if markers is Array:
		for candidate: Variant in markers:
			if candidate is Dictionary:
				var marker: Dictionary = candidate
				var name_text: String = str(marker.get("name", ""))
				if marker.has("name_key"):
					name_text = _loc(str(marker["name_key"]))
				lines.append("%s %s" % ["✓" if bool(marker.get("done", false)) else "□", name_text])
	if lines.size() == 1:
		lines.append(_loc("MUX_GOALS_NO_MARKERS"))
	_markers.text = "\n".join(lines)

func _unhandled_key_input(event: InputEvent) -> void:
	if not event is InputEventKey or not event.pressed or event.is_echo():
		return
	var key: InputEventKey = event
	if key.keycode == KEY_G:
		toggle_panel()
		get_viewport().set_input_as_handled()
	elif key.keycode == KEY_ESCAPE and visible:
		hide()
		get_viewport().set_input_as_handled()

static func _loc(key: String) -> String:
	return Catalog._text(key, Callable())
