# why_panel.gd (UI/WhyPanel, created by chronicle_panel.gd beside itself): the M5 "→ 왜?" chain of one chronicle entry
#   (goal 3.4–3.5). Opened by a click on an entry's "→ 왜?" link in the chronicle (C) or in the inspector's 이야기 tab.
# Shows the entry's sentence, then each cause one line further down with an arrow: a chronicle entry as its text with
#   its day, a mood modifier by name ("→ 배고픔"), the relation itself (code 100, "→ 사이가 나빠서"), a pressure by name
#   ("→ 한파"); "더 거슬러 올라갈 원인이 없음" ends a chain of one line. Person links go to the inspector. Esc, 닫기 or a
#   world reset closes it; it sits in the screen's middle column, below the story banner.
# Bridge methods: get_chronicle_why (direct, once per open); via SimReads get_world_info; signal world_reset.
extends Control

const PanelTooltips = preload("res://scripts/ui/panels/panel_tooltips.gd")

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const BuildingCauseText = preload("res://scripts/ui/building_cause_text.gd")
const PANEL_ORIGIN: Vector2 = Vector2(648, 300)
const MIN_WIDTH: float = 320.0
const MAX_WIDTH: float = 640.0
const MAX_HEIGHT: float = 480.0

var _world: Node = null
var _locale: Node = null
var _title: Label
var _body: RichTextLabel
var _close: Button
var _index: int = -1
var _lines: Array[String] = []
var _panel: PanelContainer
var _column: VBoxContainer
var _available_height: float = MAX_HEIGHT
var _fit_queued: bool = false


func _ready() -> void:
	_world = get_node_or_null("/root/Main/WorldSim")
	_locale = get_node_or_null("/root/Locale")
	if _world != null:
		_world.connect("world_reset", close)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	position = PANEL_ORIGIN
	_panel = ArtUI.panel(self, Rect2())
	_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	_column = VBoxContainer.new()
	_column.add_theme_constant_override("separation", 8)
	_panel.add_child(_column)
	var head: HBoxContainer = HBoxContainer.new()
	_column.add_child(head)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(_title)
	_close = Button.new()
	PanelTooltips.attach(_close, "close")
	_close.name = "Close"
	_close.focus_mode = Control.FOCUS_NONE
	_close.pressed.connect(close)
	head.add_child(_close)
	_body = RichTextLabel.new()
	_body.name = "Chain"
	_body.bbcode_enabled = true
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_body.meta_clicked.connect(_follow_link)
	_body.resized.connect(_queue_fit)
	Names.prepare_rich_text(_body)
	_column.add_child(_body)


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
	_body.bbcode_enabled = true
	_body.text = "\n\n".join(shown)
	_body.get_v_scroll_bar().value = 0
	visible = true
	_queue_fit()
	return true


## Exact persisted source record; record is NOT a chronicle-entry index.
func open_building(site: int, record: int) -> bool:
	if _world == null:
		return false
	var raw: Variant = _world.call("get_building_cause", site, record)
	var read: Dictionary = raw if raw is Dictionary else {}
	_index = -1
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	_lines = BuildingCauseText.lines(read, _locale, info)
	_title.text = Chronicle.loc(_locale, "BUILD_SOURCE")
	_close.text = Chronicle.loc(_locale, "UI_CLOSE")
	_body.bbcode_enabled = false
	_body.text = "\n\n".join(_lines)
	_body.get_v_scroll_bar().value = 0
	visible = true
	_queue_fit()
	return bool(read.get("valid", false))


## LegacyLayout owns position and scale; only the content's natural extent changes here.
func set_available_height(height: float) -> void:
	var bounded: float = minf(height, MAX_HEIGHT)
	if not is_equal_approx(_available_height, bounded):
		_available_height = bounded
		_queue_fit()


func _queue_fit() -> void:
	if not visible or _fit_queued:
		return
	_fit_queued = true
	_fit.call_deferred()


func _fit() -> void:
	_fit_queued = false
	if not visible:
		return
	var margins: Vector2 = _panel.get_theme_stylebox("panel").get_minimum_size()
	var heading: Vector2 = (_title.get_parent() as Control).get_combined_minimum_size()
	var text_width: float = 0.0
	var font_size: int = _body.get_theme_font_size("normal_font_size")
	for text_line: String in _body.get_parsed_text().split("\n"):
		for font_name: String in ["normal_font", "bold_font"]:
			text_width = maxf(text_width, _body.get_theme_font(font_name).get_string_size(text_line, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size).x)
	var width: float = clampf(maxf(text_width + 20.0, heading.x) + margins.x, MIN_WIDTH, MAX_WIDTH)
	_body.custom_minimum_size.x = width - margins.x
	_body.size.x = _body.custom_minimum_size.x
	var chrome: float = margins.y + heading.y + _column.get_theme_constant("separation")
	_body.custom_minimum_size.y = minf(float(_body.get_content_height()) + 4.0, maxf(40.0, _available_height - chrome))
	_panel.size = Vector2(width, 0)
	size = _panel.size


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
	var parts: PackedStringArray = str(meta).split(":")
	if parts.size() == 3 and parts[0] == "building":
		if parts[1].is_valid_int() and parts[2].is_valid_int():
			open_building(int(parts[1]), int(parts[2]))
		return
	var inspector: Node = get_node_or_null("../AgentInspectorPanel")
	if inspector != null:
		inspector.call("_follow_link", meta)
