# chronicle_panel.gd (UI/ChroniclePanel in scenes/main.tscn): C-toggled world chronicle feed; pages new entries once
#   per tick (even while hidden), merges porter groups, keeps the last 60 notable rows; links go to the inspector.
#   NOTABLE lists every kind the feed keeps (M3 adds 23 RoomFormed; M4 adds 24 ToolBroke, 25 KnowledgeLearned,
#   26 KnowledgeDiscovered, 27 KnowledgeAtRisk, 28 KnowledgeLost). M4: a KnowledgeLost row from the last day shows the
#   toast "기술 소실: 문 만들기 (마지막: 카토)" on the order bar (UI/OrderBar show_toast).
# M5 (goal 3.4–3.5): NOTABLE adds 29–37 (pressures, hunts, fights, breakdowns, drafts, wolves, illness); an entry
#   with a cause link ends in "→ 왜?", whose click opens the chain (UI/WhyPanel, why_panel.gd, created here beside
#   this node); a Drafted row with cause 2 from the last day toasts "카토: 너무 지쳐 소집에서 빠짐". Filters under the
#   title: the kind group (전체 / 압력 / 사냥·늑대 / 다툼·기분 / 지식 / 삶) and "고른 사람만" (the inspector's person).
#   Once a year is over, a header line sums it up (get_year_summary of the last complete year: births, deaths,
#   pressures, discoveries, quarrels, deer and wolves).
# M5.5 (goal §9): NOTABLE adds 38–44 (migrants arrived, joined, declined; traders arrived, traded, left; eureka; the
#   eureka in the 지식 filter); neighbours are named from get_neighbours (NeighbourText.observe before each refresh);
#   the year line adds migrants joined, trades and eurekas.
# Bridge methods: get_chronicle_since, get_year_summary (direct); via SimReads get_time_snapshot, get_world_info,
#   get_neighbours; signal world_reset.
extends Control

const PanelTooltips = preload("res://scripts/ui/panels/panel_tooltips.gd")
const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const WhyPanel = preload("res://scripts/ui/panels/why_panel.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
const NeighbourText = preload("res://scripts/ui/neighbour_text.gd")
const NOTABLE: Array[int] = [0, 2, 11, 13, 14, 15, 16, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37,
	38, 39, 40, 41, 42, 43, 44, 45, 46, 47]
## M5: the kind groups of the filter (0 = all): 압력, 사냥·늑대, 다툼·기분, 지식, 삶 (every other notable kind).
const FILTER_KEYS: Array[String] = ["M5_FILTER_ALL", "M5_FILTER_PRESSURE", "M5_FILTER_HUNT", "M5_FILTER_MOOD", "M5_FILTER_KNOWLEDGE", "M5_FILTER_LIFE"]
const FILTER_KINDS: Array = [[], [29, 30, 36, 37], [31, 34, 35], [19, 32, 33], [24, 25, 26, 27, 28, 44], []]
## M5: the Drafted kind and its "left the draft at a critical need" cause.
const DRAFTED_KIND: int = 34
const SELF_UNDRAFT: int = 2
## M4: a KnowledgeLost row toasts only when it is at most this many ticks old (a reload pages old rows too).
const LOSS_TOAST_TICKS: int = 1440
var _world: Node
var _locale: Node
var _title: Label
var _body: RichTextLabel
var _body_padding: StyleBoxEmpty
var _next: int = 0
var _total: int = 0
var _tick: int = -1
var _language: String = ""
var _entries: Array[Dictionary] = []
var _pending_porter: Dictionary = {}
var _unknown_kinds: int = 0
## M5: the filter (kind group index, the inspector's person only) and the year-summary header.
var _filter: int = 0
var _person_only: bool = false
var _filter_box: OptionButton
var _person_button: Button
var _year_label: Label
var _year_shown: int = -1
var _year_key: int = -2

func _ready() -> void:
	_world = get_node_or_null("/root/Main/WorldSim")
	_locale = get_node_or_null("/root/Locale")
	if _world != null:
		_world.connect("world_reset", _reset)
	position = Vector2(12, 124)
	size = Vector2(620, 480)
	mouse_filter = Control.MOUSE_FILTER_STOP
	var panel: PanelContainer = ArtUI.panel(self, Rect2(Vector2.ZERO, size))
	var box := VBoxContainer.new()
	panel.add_child(box)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	box.add_child(_title)
	var filters: HBoxContainer = HBoxContainer.new()
	filters.add_theme_constant_override("separation", 8)
	box.add_child(filters)
	_filter_box = OptionButton.new()
	PanelTooltips.attach(_filter_box, "chronicle", "MUX_TIP_CHRONICLE_FILTER")
	_filter_box.name = "KindFilter"
	_filter_box.focus_mode = Control.FOCUS_NONE
	_filter_box.item_selected.connect(set_filter)
	filters.add_child(_filter_box)
	_person_button = Button.new()
	PanelTooltips.attach(_person_button, "chronicle")
	_person_button.name = "PersonFilter"
	_person_button.toggle_mode = true
	_person_button.focus_mode = Control.FOCUS_NONE
	_person_button.toggled.connect(set_person_only)
	filters.add_child(_person_button)
	_year_label = Label.new()
	_year_label.name = "YearSummary"
	_year_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_year_label.add_theme_color_override("font_color", Color("e8c46e"))
	_year_label.hide()
	box.add_child(_year_label)
	_body = RichTextLabel.new()
	_body.autowrap_mode = TextServer.AUTOWRAP_WORD
	_body.bbcode_enabled = true
	_body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body.meta_clicked.connect(_follow_link)
	box.add_child(_body)
	Names.prepare_rich_text(_body)
	_body_padding = StyleBoxEmpty.new()
	_body_padding.content_margin_top = 4.0
	_body.add_theme_stylebox_override("normal", _body_padding)
	_body.resized.connect(_fit_body_lines)
	_refresh()
	hide()
	if get_parent() != null and get_parent().get_node_or_null("WhyPanel") == null:
		var why: Control = WhyPanel.new()
		why.name = "WhyPanel"
		get_parent().add_child.call_deferred(why)

func _process(_delta: float) -> void:
	if _world == null or not _world.has_method("get_chronicle_since"):
		return
	var clock: Dictionary = SimReads.read(_world, "get_time_snapshot")
	var now: int = int(clock.get("tick", 0))
	var changed: bool = now != _tick
	if changed:
		_tick = now
		# One page per tick, even when hidden. Preserve the delivered cursor.
		var page: Dictionary = _world.call("get_chronicle_since", _next, 512)
		_next = int(page.get("next", _next))
		_total = int(page.get("total", _next))
		var unknown_before: int = _unknown_kinds
		for row: Dictionary in Chronicle.rows(page):
			_ingest_row(row)
		if _unknown_kinds != unknown_before:
			print_debug("Chronicle skipped unknown kinds: ", _unknown_kinds)
		while _entries.size() > 60:
			_entries.pop_front()
	if visible and (changed or Chronicle.language(_locale) != _language):
		_refresh()

func _ingest_row(row: Dictionary) -> void:
	var kind: int = int(row["kind"])
	if kind != 18:
		_pending_porter = {}
		if kind > Chronicle.LAST_KIND:
			_unknown_kinds += 1
		elif kind in NOTABLE:
			_entries.append(row)
			if kind == 28 and int(row.get("tick", 0)) >= _tick - LOSS_TOAST_TICKS:
				_toast_loss(row)
			elif kind == DRAFTED_KIND and int(row.get("cause", -1)) == SELF_UNDRAFT and int(row.get("tick", 0)) >= _tick - LOSS_TOAST_TICKS:
				_toast_text(Chronicle.self_undraft_plain(row, _locale))
		return
	var same_trip: bool = not _pending_porter.is_empty()
	for key: String in ["tick", "settlement", "settlement2", "cause"]:
		if row[key] != _pending_porter.get(key, -1):
			same_trip = false
	if same_trip:
		_pending_porter["porter_count"] = int(_pending_porter["porter_count"]) + 1
		_pending_porter["value"] = int(_pending_porter["value"]) + int(row["value"])
	else:
		_pending_porter = row.duplicate()
		_pending_porter["porter_count"] = 1
		_entries.append(_pending_porter)

## M4: the loss toast on the order bar.
func _toast_loss(row: Dictionary) -> void:
	_toast_text(Chronicle.lost_plain(row, _locale))

## The order bar's toast (M4 loss, M5 self-undraft).
func _toast_text(text: String) -> void:
	var bar: Node = get_node_or_null("../OrderBar")
	if bar != null and bar.has_method("show_toast"):
		bar.call("show_toast", text)

func _refresh() -> void:
	_language = Chronicle.language(_locale)
	if _world != null and _world.has_method("get_neighbours"):
		NeighbourText.observe(SimReads.read(_world, "get_neighbours"))
	_title.text = Chronicle.loc(_locale, "UI_CHRONICLE_TITLE")
	_refresh_filters()
	_refresh_year()
	var info: Dictionary = SimReads.read(_world, "get_world_info") if _world != null and _world.has_method("get_world_info") else {}
	var person: int = _selected_person()
	var lines: Array[String] = []
	for index: int in range(_entries.size() - 1, -1, -1):
		var row: Dictionary = _entries[index]
		if not passes(row, _filter, person if _person_only else -1):
			continue
		lines.append(Chronicle.line(row, -1, _locale, info, true, true))
	_body.text = "\n\n".join(lines)
	_fit_body_lines.call_deferred()

## M5: the filter controls' texts (rebuilt on a language change).
func _refresh_filters() -> void:
	if _filter_box.item_count != FILTER_KEYS.size() or str(_filter_box.get_meta("language", "")) != _language:
		_filter_box.clear()
		for i: int in FILTER_KEYS.size():
			_filter_box.add_item(Chronicle.loc(_locale, FILTER_KEYS[i]), i)
		_filter_box.set_meta("language", _language)
	_filter_box.select(_filter)
	_person_button.text = Chronicle.loc(_locale, "M5_FILTER_PERSON")
	_person_button.set_pressed_no_signal(_person_only)

## M5: whether `row` shows under kind group `group` (0 all; 5 삶 = no other group) and, when `person` ≥ 0, names them.
static func passes(row: Dictionary, group: int, person: int) -> bool:
	var kind: int = int(row.get("kind", -1))
	if group > 0 and group < FILTER_KINDS.size():
		var kinds: Array = FILTER_KINDS[group]
		if group == FILTER_KINDS.size() - 1:
			for g: int in range(1, FILTER_KINDS.size() - 1):
				if (FILTER_KINDS[g] as Array).has(kind):
					return false
		elif not kinds.has(kind):
			return false
	if person >= 0:
		return int(row.get("subject", -1)) == person or int(row.get("other", -1)) == person or int(row.get("third", -1)) == person
	return true

## M5: choose kind group `group` (0 all … 5 삶; also used by tools).
func set_filter(group: int) -> void:
	_filter = clampi(group, 0, FILTER_KEYS.size() - 1)
	_refresh()

## M5: show only the inspector's person (also used by tools).
func set_person_only(on: bool) -> void:
	_person_only = on
	_refresh()

## M5: the inspector's selected AgentId (alive or a record), or -1.
func _selected_person() -> int:
	var inspector: Node = get_node_or_null("../AgentInspectorPanel")
	if inspector == null or not (inspector as Control).visible:
		return -1
	return int(inspector.get("_selected_person"))

## M5: the header line for the last complete year (read again when the year changes).
func _refresh_year() -> void:
	if _world == null or not _world.has_method("get_year_summary"):
		_year_label.hide()
		return
	var info: Dictionary = SimReads.read(_world, "get_world_info")
	var per_year: int = maxi(1, int(info.get("ticks_per_year", 0)))
	@warning_ignore("integer_division")
	var last: int = maxi(_tick, 0) / per_year - 1
	var key: int = hash([last, _language])
	if key == _year_key:
		return
	_year_key = key
	_year_shown = -1
	_year_label.hide()
	if last < 0 or int(info.get("ticks_per_year", 0)) <= 0:
		return
	var read: Variant = _world.call("get_year_summary", last)
	var summary: Dictionary = read if read is Dictionary else {}
	if not bool(summary.get("complete", false)):
		return
	_year_shown = last
	_year_label.text = StoryText.year_line(_locale, summary)
	_year_label.show()

## M5: the year the header sums up (-1 none) and its text (for tests).
func year_summary() -> Dictionary:
	return {"year": _year_shown, "text": _year_label.text if _year_label.visible else ""}

## The feed's text without BBCode (for tests).
func plain_text() -> String:
	return _body.get_parsed_text()

func _fit_body_lines() -> void:
	if _body_padding == null:
		return
	var line_height: float = ceilf(_body.get_theme_font("normal_font").get_height(_body.get_theme_font_size("normal_font_size")))
	if _body.get_line_count() > 0:
		line_height = maxf(line_height, _body.get_line_height(0))
	var available: float = maxf(0.0, _body.size.y - _body_padding.content_margin_top - line_height)
	# Keep the scroll viewport on whole text lines at both ends of the feed.
	_body_padding.content_margin_bottom = line_height + fmod(available, line_height)

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and event.keycode == KEY_C:
		toggle_visible()
		get_viewport().set_input_as_handled()

func toggle_visible() -> void:
	visible = not visible
	if visible:
		var causal: Control = get_node_or_null("../CausalPanel") as Control
		if causal != null:
			causal.hide()
		_refresh()

func _follow_link(meta: Variant) -> void:
	if str(meta).begins_with("why:"):
		open_why(int(str(meta).split(":")[1]))
		return
	var inspector: Node = get_node_or_null("../AgentInspectorPanel")
	if inspector != null:
		inspector.call("_follow_link", meta)

## M5: open the "→ 왜?" chain of chronicle entry `index` (the link's action; also used by tools and the inspector).
func open_why(index: int) -> bool:
	var why: Node = get_node_or_null("../WhyPanel")
	return why != null and bool(why.call("open", index))

func _reset() -> void:
	_next = 0
	_total = 0
	_tick = -1
	_entries.clear()
	_pending_porter = {}
	_unknown_kinds = 0
	_year_key = -2
	_refresh()
	hide()
