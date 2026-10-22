# priority_panel.gd (UI/PriorityPanel in scenes/main.tscn): the M2 work-priority table, toggled with P.
# Rows = the adults (get_work_priorities, AgentId order; name as the inspector shows it, "노인" for elders),
#   columns = the six work types in the bridge's order (제작, 건축, 채석, 벌목, 장작, 운반). A cell shows the level
#   (1 first … 4 last, "—" = 0 never) in a colour per level; a click cycles 0→1→2→3→4→0 through
#   cmd_set_work_priority. "모두 기본" sets every cell that differs from the default (3; elders: quarry 0) back.
#   Children have no row (the simulation refuses them). Rebuilt only when the priorities change; Esc closes it;
#   a world reset hides it.
# Bridge methods: via SimReads get_work_priorities; via SimReads.mutate cmd_set_work_priority; signal world_reset.
extends Control

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const TOGGLE_KEY: Key = KEY_P
const PANEL_SIZE: Vector2 = Vector2(1000, 760)
const DEFAULT_LEVEL: int = 3
## Cell colours by level 0–4 (0 = never: dim; 1 = first: bright green … 4 = last: muted).
const LEVEL_COLORS: Array[Color] = [Color("6a6058"), Color("8ee07a"), Color("e8d26e"), Color("e0a060"), Color("b08878")]
const MUTED: Color = Color("c8b89a")

var _world_sim: Node = null
var _locale: Node = null
var _key: int = 0
var _title: Label
var _hint: Label
var _reset: Button
var _close: Button
var _grid: GridContainer


func _ready() -> void:
	_locale = get_node_or_null("/root/Locale")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", func() -> void: visible = false)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	position = Vector2(460, 140)
	size = PANEL_SIZE
	var panel: PanelContainer = ArtUI.panel(self, Rect2(Vector2.ZERO, PANEL_SIZE))
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 8)
	panel.add_child(column)
	var head: HBoxContainer = HBoxContainer.new()
	head.add_theme_constant_override("separation", 8)
	column.add_child(head)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(_title)
	_reset = _button("Reset", reset_all)
	head.add_child(_reset)
	_close = _button("Close", func() -> void: visible = false)
	head.add_child(_close)
	_hint = Label.new()
	_hint.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_hint.add_theme_color_override("font_color", MUTED)
	column.add_child(_hint)
	var scroll: ScrollContainer = ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	column.add_child(scroll)
	_grid = GridContainer.new()
	_grid.add_theme_constant_override("h_separation", 6)
	_grid.add_theme_constant_override("v_separation", 4)
	scroll.add_child(_grid)


func _button(node_name: String, action: Callable) -> Button:
	var button: Button = Button.new()
	button.name = node_name
	button.focus_mode = Control.FOCUS_NONE
	button.pressed.connect(action)
	return button


## Show or hide the table.
func toggle_visible() -> void:
	visible = not visible
	_key = 0
	if visible:
		_refresh()


func _unhandled_input(event: InputEvent) -> void:
	if not (event is InputEventKey and event.pressed and not event.echo):
		return
	var key: InputEventKey = event
	if key.keycode == TOGGLE_KEY:
		toggle_visible()
		get_viewport().set_input_as_handled()
	elif key.keycode == KEY_ESCAPE and visible:
		visible = false
		get_viewport().set_input_as_handled()


func _process(_delta: float) -> void:
	if visible:
		_refresh()


## The table as rows {agent, elder, levels: Array[int]} plus the work-type names.
static func table(snap: Dictionary) -> Dictionary:
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var elders: PackedInt32Array = snap.get("elders", PackedInt32Array())
	var levels: PackedByteArray = snap.get("levels", PackedByteArray())
	var works: PackedStringArray = snap.get("work_types", PackedStringArray())
	var width: int = works.size()
	var rows: Array[Dictionary] = []
	for i: int in ids.size():
		if width == 0 or (i + 1) * width > levels.size():
			break
		var row: Array[int] = []
		for w: int in width:
			row.append(int(levels[i * width + w]))
		rows.append({"agent": ids[i], "elder": i < elders.size() and elders[i] != 0, "levels": row})
	return {"rows": rows, "works": works}


## The default level of work type `work` for a person (elders never quarry).
static func default_level(work_name: String, elder: bool) -> int:
	return 0 if elder and work_name == "quarry" else DEFAULT_LEVEL


## The level after `level` in the click cycle 0→1→2→3→4→0.
static func next_level(level: int) -> int:
	return (level + 1) % 5


func _refresh() -> void:
	if _world_sim == null:
		return
	var data: Dictionary = table(SimReads.read(_world_sim, "get_work_priorities"))
	var key: int = hash([data, Chronicle.language(_locale)])
	if key == _key:
		return
	_key = key
	_title.text = EconomyText.ltr(_locale, "UI_PRIORITY_TITLE")
	_hint.text = EconomyText.ltr(_locale, "UI_PRIORITY_HINT")
	_reset.text = EconomyText.ltr(_locale, "UI_PRIORITY_RESET")
	_close.text = EconomyText.ltr(_locale, "UI_CLOSE")
	for child: Node in _grid.get_children():
		_grid.remove_child(child)
		child.queue_free()
	var works: PackedStringArray = data["works"]
	var rows: Array[Dictionary] = data["rows"]
	_grid.columns = works.size() + 1
	var corner: Label = Label.new()
	corner.text = EconomyText.ltr(_locale, "UI_PRIORITY_NAME")
	corner.custom_minimum_size.x = 300
	_grid.add_child(corner)
	for w: String in works:
		var title: Label = Label.new()
		title.text = EconomyText.work(_locale, w)
		title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		title.custom_minimum_size.x = 96
		_grid.add_child(title)
	if rows.is_empty():
		var none: Label = Label.new()
		none.text = EconomyText.ltr(_locale, "UI_PRIORITY_EMPTY")
		none.add_theme_color_override("font_color", MUTED)
		_grid.add_child(none)
		return
	var language: String = Chronicle.language(_locale)
	for r: int in rows.size():
		var row: Dictionary = rows[r]
		var agent: int = int(row["agent"])
		var name_label: Label = Label.new()
		name_label.text = Names.present(agent, language)
		if bool(row["elder"]):
			name_label.text += " · " + EconomyText.ltr(_locale, "UI_PRIORITY_ELDER")
		name_label.clip_text = true
		name_label.custom_minimum_size.x = 300
		_grid.add_child(name_label)
		var levels: Array[int] = row["levels"]
		for w: int in works.size():
			var level: int = levels[w]
			var cell: Button = _button("Cell_%d_%d" % [r, w], set_level.bind(agent, works[w], next_level(level)))
			cell.text = str(level) if level > 0 else "—"
			cell.custom_minimum_size = Vector2(96, 0)
			var colour: Color = LEVEL_COLORS[clampi(level, 0, LEVEL_COLORS.size() - 1)]
			for state: String in ["font_color", "font_hover_color", "font_pressed_color", "font_focus_color"]:
				cell.add_theme_color_override(state, colour)
			_grid.add_child(cell)


## Set person `agent`'s level for `work_name`; a rejection shows its reason as the order bar's toast.
func set_level(agent: int, work_name: String, level: int) -> Dictionary:
	if _world_sim == null:
		return {}
	var result: Variant = SimReads.mutate(_world_sim, "cmd_set_work_priority", [agent, work_name, level])
	var data: Dictionary = result if result is Dictionary else {}
	if not bool(data.get("accepted", false)):
		_toast(EconomyText.reason(_locale, str(data.get("reason", "unknown"))))
	# Rebuilt on the next frame: the pressed cell belongs to the grid being replaced.
	_key = 0
	return data


## "모두 기본": every cell that differs from its default goes back (3; elders quarry 0). Returns commands sent.
func reset_all() -> int:
	if _world_sim == null:
		return 0
	var data: Dictionary = table(SimReads.read(_world_sim, "get_work_priorities"))
	var works: PackedStringArray = data["works"]
	var sent: int = 0
	for row: Dictionary in data["rows"]:
		var levels: Array[int] = row["levels"]
		for w: int in works.size():
			var want: int = default_level(works[w], bool(row["elder"]))
			if levels[w] != want:
				set_level(int(row["agent"]), works[w], want)
				sent += 1
	_toast(EconomyText.ltr(_locale, "UI_ORDER_TOAST_PRIORITY"))
	return sent


func _toast(text: String) -> void:
	var bar: Node = get_node_or_null("/root/Main/UI/OrderBar")
	if bar != null and bar.has_method("show_toast"):
		bar.call("show_toast", text)
