# migrant_panel.gd (UI/MigrantPanel, created by hud_topbar.gd beside itself): the M5.5 migrant dialog (goal §9.1).
# Opens by itself when a wave starts waiting at the hearth (get_neighbours `wave_states` 1, unanswered), once per wave;
#   then, at 1× speed and not already paused, it pauses the clock (Camera2D set_paused; at 2×–4× it does not). Also
#   opened by the HUD banner (UI/StoryBanner) and by a click on the wave on the map (order_tool.gd). Shows "붉은바위에서
#   이주민 3명이 왔다", the countdown to `wave_deadline` and, per person of get_wave: name (PersonNames of the AgentId they
#   will have), sex, age, the top 3 knowledge nodes with ‰ and "숙련" (know_practice), the apparel (가죽 랩 when `wrap`,
#   else 허리 두르개). 받아들이기 / 돌려보내기 send cmd_accept_migrants / cmd_decline_migrants and toast the result on the
#   order bar; 나중에 (or Esc) closes it. It closes itself when the wave is no longer waiting, and on a world reset.
# Bridge methods: get_wave (direct, when a wave is shown or its read changes); via SimReads get_neighbours,
#   get_world_info; via SimReads.mutate cmd_accept_migrants, cmd_decline_migrants; signal world_reset.
extends Control

const PanelTooltips = preload("res://scripts/ui/panels/panel_tooltips.gd")

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const NeighbourText = preload("res://scripts/ui/neighbour_text.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")
## Below the HUD's visitor banner (UI/StoryBanner), so its countdown stays visible.
const PANEL_RECT: Rect2 = Rect2(480, 200, 960, 0)
## The people list scrolls past this height (a wave has up to 6 people); about this much per person.
const LIST_MAX: int = 600
const PERSON_HEIGHT: int = 100
const MUTED: Color = Color("c8b89a")
const TEXT: Color = Color("f2e6c8")
const ACCENT: Color = Color("e8c46e")
const SKILLED: Color = Color("8ee07a")

var _world_sim: Node = null
var _locale: Node = null
var _panel: PanelContainer
var _title: Label
var _ask: Label
var _countdown: Label
var _people: VBoxContainer
var _scroll: ScrollContainer
var _accept: Button
var _decline: Button
var _later: Button
var _wave: int = -1
var _wave_read: Dictionary = {}
var _key: int = 0
## Waves already announced (opened by themselves once).
var _announced: Dictionary = {}
var _person_texts: Array[String] = []


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	_locale = get_node_or_null("/root/Locale")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	position = PANEL_RECT.position
	_panel = ArtUI.panel(self, Rect2(Vector2.ZERO, PANEL_RECT.size))
	_panel.name = "Dialog"
	_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 8)
	_panel.add_child(column)
	_title = _label(column, ACCENT)
	_title.theme_type_variation = &"Heading"
	_ask = _label(column, MUTED)
	_countdown = _label(column, Color("f07060"))
	_scroll = ScrollContainer.new()
	_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	column.add_child(_scroll)
	_people = VBoxContainer.new()
	_people.name = "People"
	_people.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_people.add_theme_constant_override("separation", 10)
	_scroll.add_child(_people)
	var buttons: HBoxContainer = HBoxContainer.new()
	buttons.add_theme_constant_override("separation", 16)
	buttons.alignment = BoxContainer.ALIGNMENT_END
	column.add_child(buttons)
	_accept = _button(buttons, "Accept", accept)
	_decline = _button(buttons, "Decline", decline)
	_later = _button(buttons, "Later", close)


func _label(parent: Control, colour: Color) -> Label:
	var label: Label = Label.new()
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.custom_minimum_size.x = PANEL_RECT.size.x - 48
	label.add_theme_color_override("font_color", colour)
	parent.add_child(label)
	return label


func _button(parent: Control, node_name: String, action: Callable) -> Button:
	var button: Button = Button.new()
	PanelTooltips.attach(button, "neighbours")
	button.name = node_name
	button.focus_mode = Control.FOCUS_NONE
	button.pressed.connect(func() -> void: action.call())
	parent.add_child(button)
	return button


func _on_world_reset() -> void:
	_announced.clear()
	close()


func _unhandled_input(event: InputEvent) -> void:
	if visible and event is InputEventKey and event.pressed and not event.echo and (event as InputEventKey).keycode == KEY_ESCAPE:
		close()
		get_viewport().set_input_as_handled()


func _process(_delta: float) -> void:
	if _world_sim == null or not _world_sim.has_method("get_neighbours"):
		return
	var read: Dictionary = SimReads.read(_world_sim, "get_neighbours")
	if read.is_empty():
		return
	NeighbourText.observe(read)
	for wave: Dictionary in NeighbourText.waves(read):
		var id: int = int(wave["id"])
		if int(wave["state"]) == NeighbourText.WAITING and int(wave["answer"]) == NeighbourText.ANSWER_NONE and not _announced.has(id):
			_announced[id] = true
			open_wave(id)
			_pause_at_normal_speed()
	if visible:
		_refresh(read)


## At 1× and running, pause the clock for the player's answer (at higher speeds the game keeps running).
func _pause_at_normal_speed() -> void:
	var camera: Node = get_node_or_null("/root/Main/Camera2D")
	if camera == null or not camera.has_method("set_paused"):
		return
	if not bool(camera.call("is_paused")) and float(camera.call("get_speed")) <= 1.0:
		camera.call("set_paused", true)


## Show wave `id` (also used by the banner, the map click and tools).
func open_wave(id: int) -> void:
	_wave = id
	_key = 0
	visible = true
	if _world_sim != null:
		_refresh(SimReads.read(_world_sim, "get_neighbours"))


func close() -> void:
	visible = false
	_wave = -1


## The wave shown (-1 none).
func shown_wave() -> int:
	return _wave if visible else -1


func _refresh(read: Dictionary) -> void:
	var row: Dictionary = {}
	for wave: Dictionary in NeighbourText.waves(read):
		if int(wave["id"]) == _wave:
			row = wave
	if row.is_empty() or int(row["state"]) != NeighbourText.WAITING or int(row["answer"]) != NeighbourText.ANSWER_NONE:
		close()
		return
	var info: Dictionary = SimReads.read(_world_sim, "get_world_info")
	var hours: int = NeighbourText.hours_left(int(row["deadline"]), int(read.get("now", 0)), info)
	var language: String = NeighbourText.language(_locale)
	var key: int = hash([_wave, read.get("version", 0), hours, language])
	if key == _key:
		return
	_key = key
	var raw: Variant = _world_sim.call("get_wave", _wave)
	_wave_read = raw if raw is Dictionary else {}
	var neighbour: String = NeighbourText.name_in(_locale, read, int(row["neighbour"]))
	var people: Array[Dictionary] = NeighbourText.wave_people(_wave_read)
	_title.text = NeighbourText.trf(_locale, "M55_MIGRANT_TITLE_FMT", {"neighbour": neighbour, "count": people.size()})
	_ask.text = NeighbourText.ltr(_locale, "M55_MIGRANT_ASK")
	_countdown.text = NeighbourText.trf(_locale, "M55_MIGRANT_LEFT_FMT", {"hours": maxi(0, hours)})
	_accept.text = NeighbourText.ltr(_locale, "M55_MIGRANT_ACCEPT")
	_decline.text = NeighbourText.ltr(_locale, "M55_MIGRANT_DECLINE")
	_later.text = NeighbourText.ltr(_locale, "M55_MIGRANT_LATER")
	for child: Node in _people.get_children():
		_people.remove_child(child)
		child.queue_free()
	_person_texts.clear()
	for person: Dictionary in people:
		_add_person(person, language)
	_scroll.custom_minimum_size = Vector2(PANEL_RECT.size.x - 32, mini(LIST_MAX, maxi(1, people.size()) * PERSON_HEIGHT))


func _add_person(person: Dictionary, language: String) -> void:
	var box: VBoxContainer = VBoxContainer.new()
	box.name = "Person_%d" % int(person["agent"])
	box.add_theme_constant_override("separation", 2)
	_people.add_child(box)
	var head: Label = _label(box, TEXT)
	head.text = NeighbourText.trf(_locale, "M55_MIGRANT_PERSON_FMT", {"name": Names.display_name(int(person["agent"]), language),
		"sex": NeighbourText.sex(_locale, bool(person["female"])), "age": int(person["age"])})
	var know: Array[Dictionary] = []
	know.assign(person["know"])
	var skilled: bool = false
	for k: Dictionary in know:
		skilled = skilled or bool(k["practice"])
	var knowledge: Label = _label(box, SKILLED if skilled else MUTED)
	knowledge.text = NeighbourText.know_text(_locale, know)
	var apparel: Label = _label(box, MUTED)
	apparel.text = NeighbourText.trf(_locale, "M55_MIGRANT_APPAREL_FMT", {"top": StoryText.top(_locale, int(person["top"]))})
	_person_texts.append(" / ".join([head.text, knowledge.text, apparel.text]))


## 받아들이기: cmd_accept_migrants on the wave shown; returns the result.
func accept() -> Dictionary:
	return _answer("cmd_accept_migrants")


## 돌려보내기: cmd_decline_migrants on the wave shown; returns the result.
func decline() -> Dictionary:
	return _answer("cmd_decline_migrants")


func _answer(command: String) -> Dictionary:
	if _world_sim == null or _wave < 0:
		return {}
	var result: Variant = SimReads.mutate(_world_sim, command, [_wave])
	var data: Dictionary = result if result is Dictionary else {}
	var text: String
	if bool(data.get("accepted", false)):
		text = NeighbourText.trf(_locale, "M55_TOAST_ACCEPTED_FMT", {"count": int(data.get("count", 0))}) if command == "cmd_accept_migrants" \
			else NeighbourText.ltr(_locale, "M55_TOAST_DECLINED")
		close()
	else:
		text = EconomyText.reason(_locale, str(data.get("reason", "nothing")))
	var bar: Node = get_node_or_null("../OrderBar")
	if bar != null and bar.has_method("show_toast"):
		bar.call("show_toast", text)
	return data


## The dialog's lines for tests and tools: {title, countdown, people: [one line per person], buttons: [3 texts]}.
func texts() -> Dictionary:
	return {"title": _title.text, "ask": _ask.text, "countdown": _countdown.text, "people": _person_texts.duplicate(),
		"buttons": [_accept.text, _decline.text, _later.text]}
