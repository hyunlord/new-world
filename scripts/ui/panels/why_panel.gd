# why_panel.gd (UI/WhyPanel, created by chronicle_panel.gd beside itself): the M5 "→ 왜?" chain of one chronicle entry
#   (goal 3.4–3.5). Opened by a click on an entry's "→ 왜?" link in the chronicle (C) or in the inspector's 이야기 tab.
# Shows the entry's sentence, then each cause one line further down with an arrow: a chronicle entry as its text with
#   its day, a mood modifier by name ("→ 배고픔"), the relation itself (code 100, "→ 사이가 나빠서"), a pressure by name
#   ("→ 한파"); "더 거슬러 올라갈 원인이 없음" ends a chain of one line. Person links go to the inspector. Esc, 닫기 or a
#   world reset closes it; it sits in the screen's middle column, below the story banner.
# Bridge methods: get_chronicle_why and get_building_cause when opened; via SimReads get_world_info and get_building_snapshot; signal world_reset.
extends Control

const PanelTooltips = preload("res://scripts/ui/panels/panel_tooltips.gd")

const CauseTooltips = preload("res://scripts/ui/panels/cause_tooltips.gd")
const View = preload("res://scripts/world/colony/view.gd")
const ViewProjection = preload("res://scripts/world/view_projection.gd")
const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const Icons = preload("res://scripts/ui/hud/ui1_icons.gd")
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const BuildingCauseText = preload("res://scripts/ui/building_cause_text.gd")
const PANEL_ORIGIN: Vector2 = Vector2(758, 156)
const MIN_WIDTH: float = 320.0
const MAX_WIDTH: float = 640.0
const MAX_HEIGHT: float = 480.0

var _world: Node = null
var _locale: Node = null
var _title: Label
var _title_icon: TextureRect
var _body: RichTextLabel
var _close: Button
var _index: int = -1
var _lines: Array[String] = []
var _panel: PanelContainer
var _column: VBoxContainer
var _available_height: float = MAX_HEIGHT
var _fit_queued: bool = false
var _tooltips := CauseTooltips.new()
var _chain: Array[String] = []
var _building_preview: Array[String] = []
var _building_point: Vector2 = Vector2(-1.0, -1.0)
var _person_bits: int = -1


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
	_title_icon = TextureRect.new()
	_title_icon.custom_minimum_size = Vector2(24.0, 24.0)
	_title_icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	_title_icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	_title_icon.mouse_filter = Control.MOUSE_FILTER_IGNORE
	head.add_child(_title_icon)
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
	_body.meta_underlined = false
	_body.meta_hover_started.connect(_hover_cause)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_body.meta_clicked.connect(_follow_link)
	_body.resized.connect(_queue_fit)
	Names.prepare_rich_text(_body)
	_column.add_child(_body)
	add_child(_tooltips)


## Open the chain of chronicle entry `index`; returns whether the entry exists.
func open(index: int) -> bool:
	if _world == null:
		return false
	var why: Dictionary = _world.get_chronicle_why(index)
	if not bool(why.get("found", false)):
		return false
	_index = index
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	_lines = Chronicle.why_lines(why, _locale, info)
	_title_icon.texture = Icons.texture("why")
	_title.text = Chronicle.loc(_locale, "M5_WHY_TITLE")
	_close.text = Chronicle.loc(_locale, "UI_CLOSE")
	_reset_focus()
	var subjects: PackedInt64Array = why.get("subjects", PackedInt64Array())
	var others: PackedInt64Array = why.get("others", PackedInt64Array())
	if not subjects.is_empty() and subjects[0] >= 0:
		_track_person(subjects[0])
	elif not others.is_empty() and others[0] >= 0:
		_track_person(others[0])
	var sites: PackedInt64Array = why.get("building_sites", PackedInt64Array())
	var records: PackedInt64Array = why.get("building_records", PackedInt64Array())
	for index_in_chain: int in mini(sites.size(), records.size()):
		if sites[index_in_chain] >= 0 and records[index_in_chain] >= 0:
			var building: Dictionary = _world.get_building_cause(sites[index_in_chain], records[index_in_chain])
			if bool(building.get("valid", false)):
				_track_building(building)
				_building_preview.append("\n".join(BuildingCauseText.lines(building, _locale, info)))
				break
	var shown: Array[String] = _lines.duplicate()
	if shown.size() <= 1:
		shown.append(Chronicle.loc(_locale, "M5_WHY_NONE"))
	elif shown.size() > 2 and not shown[1].contains("[url=building:"):
		# RichTextLabel cannot nest person links inside a hoverable ancestor phrase.
		_body.bbcode_enabled = true
		for cause: String in shown.slice(2, 2 + CauseTooltips.MAX_DEPTH):
			_body.text = cause
			_chain.append(_body.get_parsed_text())
		_body.text = shown[1]
		shown[1] = CauseTooltips.link(_body.get_parsed_text(), 0)
	if not _building_preview.is_empty():
		for i: int in shown.size():
			if shown[i].contains("[url=building:"):
				shown[i] = shown[i].replace("[color=#e8c46e]", "[color=#B5562F][u]").replace("[/color]", "[/u][/color]")
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
	var read: Dictionary = _world.get_building_cause(site, record)
	_index = -1
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	_lines = BuildingCauseText.lines(read, _locale, info)
	_reset_focus()
	if bool(read.get("valid", false)):
		_track_building(read)
	var shown: Array[String] = _lines.duplicate()
	if str(read.get("zone_trigger", "")) == "beds_short" and read.has("zone_people") and read.has("zone_slots") and shown.size() > 3:
		var people: int = int(read["zone_people"])
		var beds: int = int(read["zone_slots"])
		var phrase: String = Chronicle.loc(_locale, "UI1_BEDS_SHORT")
		var details: String = Chronicle.loc(_locale, "UI1_HOUSING_COUNTS").format({"people": people, "beds": beds})
		_chain.append(details)
		shown[2] = Chronicle.loc(_locale, "BUILD_CAUSE_LAYOUT_FMT").format({
			"zone": Chronicle.loc(_locale, str(read.get("zone_name_key", "UI_UNKNOWN"))),
			"cause": CauseTooltips.link(phrase, 0)})
	_title_icon.texture = Icons.texture("house")
	_title.text = Chronicle.loc(_locale, "BUILD_SOURCE")
	_close.text = Chronicle.loc(_locale, "UI_CLOSE")
	_body.bbcode_enabled = not _chain.is_empty()
	_body.text = "\n\n".join(shown)
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
	_reset_focus()


func _reset_focus() -> void:
	_tooltips.clear()
	_chain.clear()
	_building_preview.clear()
	_building_point = Vector2(-1.0, -1.0)
	_person_bits = -1
	queue_redraw()


func _track_person(person: int) -> void:
	var people: Dictionary = SimReads.read(_world, "get_agent_snapshot")
	var ids: PackedInt64Array = people.get("agent_ids", PackedInt64Array())
	var entities: PackedInt64Array = people.get("ids", PackedInt64Array())
	var row: int = ids.find(person)
	if row >= 0 and row < entities.size():
		_person_bits = entities[row]


func _track_building(building: Dictionary) -> void:
	var tile := Vector2i(int(building["x"]), int(building["y"]))
	var snapshot: Dictionary = SimReads.read(_world, "get_building_snapshot")
	for site: Dictionary in snapshot.get("template_sites", []):
		if int(site.get("id", -1)) == int(building["site"]) and str(site.get("status", "")) != "cancelled":
			_building_point = View.tile_to_world(tile) + Vector2(float(site["width"]), float(site["height"])) * View.TILE * 0.5
			break


func _process(_delta: float) -> void:
	if visible:
		queue_redraw()


func _draw() -> void:
	if _building_point.x >= 0.0:
		CauseTooltips.draw_connector(self, ViewProjection.world_to_screen(_building_point))
	elif _person_bits >= 0:
		var person: Rect2 = ViewProjection.person_rect(_person_bits)
		if person.has_area():
			CauseTooltips.draw_connector(self, person.get_center())


func _hover_cause(meta: Variant) -> void:
	if str(meta) == "cause:0":
		_tooltips.open_chain(_body, _chain)
	elif str(meta).begins_with("building:"):
		_tooltips.open_chain(_body, _building_preview)


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
	if str(meta) == "cause:0":
		_hover_cause(meta)
		return
	var parts: PackedStringArray = str(meta).split(":")
	if parts.size() == 3 and parts[0] == "building":
		if parts[1].is_valid_int() and parts[2].is_valid_int():
			open_building(int(parts[1]), int(parts[2]))
		return
	var inspector: Node = get_node_or_null("../AgentInspectorPanel")
	if inspector != null:
		inspector.call("_follow_link", meta)
