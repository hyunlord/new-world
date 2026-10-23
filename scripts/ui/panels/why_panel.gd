# why_panel.gd (UI/WhyPanel, created by chronicle_panel.gd beside itself): the M5 "→ 왜?" chain of one chronicle entry
#   (goal 3.4–3.5). Opened by a click on an entry's "→ 왜?" link in the chronicle (C) or in the inspector's 이야기 tab.
# Shows the entry's sentence, then each cause one line further down with an arrow: a chronicle entry as its text with
#   its day, a mood modifier by name ("→ 배고픔"), the relation itself (code 100, "→ 사이가 나빠서"), a pressure by name
#   ("→ 한파"); "더 거슬러 올라갈 원인이 없음" ends a chain of one line. Person links go to the inspector. Esc, 닫기 or a
#   world reset closes it; it sits in the screen's middle column, below the story banner.
# Bridge methods: get_chronicle_why (direct, once per open); via SimReads get_world_info; signal world_reset.
extends Control

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const PANEL_RECT: Rect2 = Rect2(648, 300, 880, 360)

var _world: Node = null
var _locale: Node = null
var _title: Label
var _body: RichTextLabel
var _close: Button
var _index: int = -1
var _lines: Array[String] = []


func _ready() -> void:
	_world = get_node_or_null("/root/Main/WorldSim")
	_locale = get_node_or_null("/root/Locale")
	if _world != null:
		_world.connect("world_reset", close)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	position = PANEL_RECT.position
	size = PANEL_RECT.size
	var panel: PanelContainer = ArtUI.panel(self, Rect2(Vector2.ZERO, PANEL_RECT.size))
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 8)
	panel.add_child(column)
	var head: HBoxContainer = HBoxContainer.new()
	column.add_child(head)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(_title)
	_close = Button.new()
	_close.name = "Close"
	_close.focus_mode = Control.FOCUS_NONE
	_close.pressed.connect(close)
	head.add_child(_close)
	_body = RichTextLabel.new()
	_body.name = "Chain"
	_body.bbcode_enabled = true
	_body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body.meta_clicked.connect(_follow_link)
	Names.prepare_rich_text(_body)
	column.add_child(_body)


## Open the chain of chronicle entry `index`; returns whether the entry exists.
func open(index: int) -> bool:
	if _world == null:
		return false
	var read: Variant = _world.call("get_chronicle_why", index)
	var why: Dictionary = read if read is Dictionary else {}
	if not bool(why.get("found", false)):
		return false
	_index = index
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	_lines = Chronicle.why_lines(why, _locale, info)
	_title.text = Chronicle.loc(_locale, "M5_WHY_TITLE")
	_close.text = Chronicle.loc(_locale, "UI_CLOSE")
	var shown: Array[String] = _lines.duplicate()
	if shown.size() <= 1:
		shown.append(Chronicle.loc(_locale, "M5_WHY_NONE"))
	_body.text = "\n\n".join(shown)
	visible = true
	return true


func close() -> void:
	visible = false
	_index = -1


## The entry shown (-1 when closed).
func current_index() -> int:
	return _index if visible else -1


## The chain's lines as BBCode (for tests): the entry first, then each cause.
func lines() -> Array[String]:
	return _lines


## The chain as plain text (for tests).
func plain_text() -> String:
	return _body.get_parsed_text()


func _unhandled_input(event: InputEvent) -> void:
	if visible and event is InputEventKey and event.pressed and not event.echo and (event as InputEventKey).keycode == KEY_ESCAPE:
		close()
		get_viewport().set_input_as_handled()


func _follow_link(meta: Variant) -> void:
	var inspector: Node = get_node_or_null("../AgentInspectorPanel")
	if inspector != null:
		inspector.call("_follow_link", meta)
