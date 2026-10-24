# neighbours_panel.gd (UI/NeighboursPanel, created by hud_topbar.gd beside itself): the M5.5 neighbours panel, toggled
#   with N (goal §9.3). One card per neighbour of get_neighbours: its name, "남쪽 · 2일 거리" (edge, distance_days),
#   "인구 51명", "태도 58 · 무심" over an attitude bar (0–100, coloured by band), "마지막 만남: 10일째" (last_contact) or
#   "아직 만난 적 없음", a line while one of its waves waits or its caravan trades, and "소문으로 아는 신석기 기술" with
#   the node names of its `rumoured` bits. Under the cards the tallies (migrants joined, trades, gifts, eurekas).
#   Rebuilt only when the read's version or the language changes. N, Esc, 닫기 or a world reset closes it.
# Bridge methods: via SimReads get_neighbours; signal world_reset.
extends Control

const PanelTooltips = preload("res://scripts/ui/panels/panel_tooltips.gd")

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const NeighbourText = preload("res://scripts/ui/neighbour_text.gd")
const KnowledgeText = preload("res://scripts/ui/knowledge_text.gd")
const TOGGLE_KEY: Key = KEY_N
const PANEL_RECT: Rect2 = Rect2(472, 140, 976, 0)
const CARD_WIDTH: int = 456
const MUTED: Color = Color("c8b89a")
const TEXT: Color = Color("f2e6c8")
const ACCENT: Color = Color("e8c46e")
const VISIT: Color = Color("f0c040")

var _world_sim: Node = null
var _locale: Node = null
var _title: Label
var _hint: Label
var _close: Button
var _cards: HBoxContainer
var _tally: Label
var _key: int = 0
var _card_texts: Array[String] = []


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	_locale = get_node_or_null("/root/Locale")
	if _world_sim != null:
		_world_sim.connect("world_reset", close)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	position = PANEL_RECT.position
	var panel: PanelContainer = ArtUI.panel(self, Rect2(Vector2.ZERO, PANEL_RECT.size))
	panel.name = "Panel"
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 10)
	panel.add_child(column)
	var head: HBoxContainer = HBoxContainer.new()
	head.add_theme_constant_override("separation", 16)
	column.add_child(head)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	head.add_child(_title)
	_hint = Label.new()
	_hint.add_theme_color_override("font_color", MUTED)
	_hint.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(_hint)
	_close = Button.new()
	PanelTooltips.attach(_close, "close")
	_close.name = "Close"
	_close.focus_mode = Control.FOCUS_NONE
	_close.pressed.connect(close)
	head.add_child(_close)
	_cards = HBoxContainer.new()
	_cards.name = "Cards"
	_cards.add_theme_constant_override("separation", 16)
	column.add_child(_cards)
	_tally = Label.new()
	_tally.name = "Tally"
	_tally.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_tally.custom_minimum_size.x = PANEL_RECT.size.x - 32
	_tally.add_theme_color_override("font_color", MUTED)
	column.add_child(_tally)


func open() -> void:
	visible = true
	_key = 0
	_refresh()


func close() -> void:
	visible = false


## Show or hide the panel (N).
func toggle_visible() -> void:
	if visible:
		close()
	else:
		open()


func _unhandled_input(event: InputEvent) -> void:
	if not (event is InputEventKey and event.pressed and not event.echo):
		return
	var key: InputEventKey = event
	if key.keycode == TOGGLE_KEY:
		toggle_visible()
		get_viewport().set_input_as_handled()
	elif key.keycode == KEY_ESCAPE and visible:
		close()
		get_viewport().set_input_as_handled()


func _process(_delta: float) -> void:
	if visible:
		_refresh()


func _refresh() -> void:
	if _world_sim == null:
		return
	var read: Dictionary = SimReads.read(_world_sim, "get_neighbours") if _world_sim.has_method("get_neighbours") else {}
	var language: String = NeighbourText.language(_locale)
	var key: int = hash([read.get("version", -1), language])
	if key == _key:
		return
	_key = key
	NeighbourText.observe(read)
	_title.text = NeighbourText.ltr(_locale, "M55_NB_TITLE")
	_hint.text = NeighbourText.ltr(_locale, "M55_NB_HINT")
	_close.text = NeighbourText.ltr(_locale, "UI_CLOSE")
	for child: Node in _cards.get_children():
		_cards.remove_child(child)
		child.queue_free()
	_card_texts.clear()
	var rows: Array[Dictionary] = NeighbourText.neighbours(read)
	if rows.is_empty():
		_tally.text = NeighbourText.ltr(_locale, "M55_NB_NONE")
		return
	var waves: Array[Dictionary] = NeighbourText.waves(read)
	var caravans: Array[Dictionary] = NeighbourText.caravans(read)
	for row: Dictionary in rows:
		_cards.add_child(_card(row, waves, caravans))
	_tally.text = NeighbourText.trf(_locale, "M55_NB_TALLY_FMT", {"migrants": int(read.get("migrants_joined", 0)), "trades": int(read.get("trades", 0)),
		"gifts": int(read.get("gifts", 0)), "eurekas": int(read.get("eurekas", 0))})


func _card(row: Dictionary, waves: Array[Dictionary], caravans: Array[Dictionary]) -> PanelContainer:
	var card: PanelContainer = PanelContainer.new()
	card.name = "Neighbour_%d" % int(row["id"])
	card.custom_minimum_size.x = CARD_WIDTH
	var box: VBoxContainer = VBoxContainer.new()
	box.add_theme_constant_override("separation", 4)
	card.add_child(box)
	var lines: Array[String] = []
	var heading: Label = _line(box, NeighbourText.name_of_key(_locale, String(row["name_key"])), ACCENT, lines)
	heading.theme_type_variation = &"Heading"
	_line(box, NeighbourText.trf(_locale, "M55_NB_DIR_FMT", {"dir": NeighbourText.direction(_locale, int(row["edge"])), "days": int(row["days"])}), TEXT, lines)
	_line(box, NeighbourText.trf(_locale, "M55_NB_POP_FMT", {"count": int(row["population"])}), TEXT, lines)
	var attitude: int = int(row["attitude"])
	_line(box, NeighbourText.trf(_locale, "M55_NB_ATTITUDE_FMT", {"value": attitude, "word": NeighbourText.attitude_word(_locale, attitude)}),
		NeighbourText.attitude_color(attitude), lines)
	box.add_child(_attitude_bar(attitude))
	var contact: int = int(row["last_contact"])
	_line(box, NeighbourText.trf(_locale, "M55_NB_CONTACT_FMT", {"day": contact}) if contact >= 0 else NeighbourText.ltr(_locale, "M55_NB_CONTACT_NONE"),
		MUTED, lines)
	for wave: Dictionary in waves:
		if int(wave["neighbour"]) == int(row["id"]) and int(wave["state"]) == NeighbourText.WAITING:
			_line(box, NeighbourText.trf(_locale, "M55_NB_VISIT_WAVE_FMT", {"count": int(wave["size"])}), VISIT, lines)
	for caravan: Dictionary in caravans:
		if int(caravan["neighbour"]) == int(row["id"]) and int(caravan["state"]) == NeighbourText.WAITING:
			_line(box, NeighbourText.ltr(_locale, "M55_NB_VISIT_CARAVAN"), VISIT, lines)
	var rumour_head: Label = _line(box, NeighbourText.ltr(_locale, "M55_NB_RUMOUR_HEAD"), TEXT, lines)
	rumour_head.theme_type_variation = &"Heading"
	var names: Array[String] = []
	for node: int in NeighbourText.rumoured_nodes(int(row["rumoured"])):
		names.append(KnowledgeText.node_name(_locale, node))
	_line(box, ", ".join(names) if not names.is_empty() else NeighbourText.ltr(_locale, "M55_NB_RUMOUR_NONE"), MUTED, lines)
	_card_texts.append(" | ".join(lines))
	return card


func _line(parent: Control, text: String, colour: Color, lines: Array[String]) -> Label:
	var label: Label = Label.new()
	label.text = text
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.custom_minimum_size.x = CARD_WIDTH - 24
	label.add_theme_color_override("font_color", colour)
	parent.add_child(label)
	lines.append(text)
	return label


func _attitude_bar(value: int) -> ProgressBar:
	var bar: ProgressBar = ProgressBar.new()
	bar.name = "Attitude"
	bar.min_value = 0.0
	bar.max_value = 100.0
	bar.value = clampi(value, 0, 100)
	bar.show_percentage = false
	bar.custom_minimum_size = Vector2(CARD_WIDTH - 24, 12)
	bar.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var fill: StyleBoxFlat = StyleBoxFlat.new()
	fill.bg_color = NeighbourText.attitude_color(value)
	bar.add_theme_stylebox_override("fill", fill)
	var background: StyleBoxFlat = StyleBoxFlat.new()
	background.bg_color = Color8(41, 38, 33)
	bar.add_theme_stylebox_override("background", background)
	return bar


## One line per card (its texts joined by " | ") and the tally (for tests and tools).
func texts() -> Dictionary:
	return {"title": _title.text, "cards": _card_texts.duplicate(), "tally": _tally.text}
