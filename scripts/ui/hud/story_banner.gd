# story_banner.gd (UI/StoryBanner, created by hud_topbar.gd beside itself): the M5 HUD lines under the top bar, in
#   the screen's middle column (goal 3.5).
# Banners from get_story (StoryText.banners): "늑대 무리가 다가온다 (3)" while wolves are on the map, "한파 2일째 · 장작 12
#   (하루 18 필요)" (`days_of`, `firewood`, `firewood_day_need`), "가뭄 3일째", "병 2일째 · 아픈 사람 1"; each on its own
#   dark panel in the pressure's colour. While wolves are on the map their mean tile (`wolves_x`, `wolves_y`) pulses:
#   a ring around it when it is on screen, else a chevron on the screen edge pointing at it. Under the banners the
#   mood overlay's legend while M shows it ("기분: 파랑 우울 · 노랑 흡족 · 빨강 무너짐"), then the drafted list
#   (get_drafted: "카토 · 몸 92 · 늑대 공격 · 창", the order named from get_agent_m5) with "모두 해제"
#   (cmd_undraft_all). Rebuilt only when the read or the language changes.
# Bridge methods: get_agent_m5 (direct, per drafted person when the list changes); via SimReads get_story, get_drafted,
#   get_animals; via SimReads.mutate cmd_undraft_all; signal world_reset.
extends Control

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const View = preload("res://scripts/world/colony/view.gd")
const COLUMN: Rect2 = Rect2(648, 124, 880, 0)
const EDGE: float = 40.0
const PULSE_SPEED: float = 4.0
const WOLF_RED: Color = Color("f07060")

var _world_sim: Node = null
var _locale: Node = null
var _column: VBoxContainer
var _banners: VBoxContainer
var _legend: Label
var _draft_panel: PanelContainer
var _draft_title: Label
var _draft_rows: VBoxContainer
var _undraft: Button
var _banner_key: int = 0
var _draft_key: int = 0
var _texts: Array[String] = []
var _draft_texts: Array[String] = []
var _time: float = 0.0
var _wolves: Vector2i = Vector2i(-1, -1)


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	_locale = get_node_or_null("/root/Locale")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	set_anchors_preset(Control.PRESET_FULL_RECT)
	theme = ArtUI.THEME
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_column = VBoxContainer.new()
	_column.name = "Column"
	_column.position = COLUMN.position
	_column.custom_minimum_size.x = COLUMN.size.x
	_column.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_column.add_theme_constant_override("separation", 6)
	add_child(_column)
	_banners = VBoxContainer.new()
	_banners.name = "Banners"
	_banners.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_banners.add_theme_constant_override("separation", 6)
	_column.add_child(_banners)
	_legend = Label.new()
	_legend.name = "MoodLegend"
	_legend.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_legend.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	var legend_panel: PanelContainer = PanelContainer.new()
	legend_panel.name = "LegendPanel"
	legend_panel.mouse_filter = Control.MOUSE_FILTER_IGNORE
	legend_panel.add_child(_legend)
	legend_panel.hide()
	_column.add_child(legend_panel)
	_draft_panel = PanelContainer.new()
	_draft_panel.name = "Drafted"
	_draft_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	_draft_panel.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	_column.add_child(_draft_panel)
	var box: VBoxContainer = VBoxContainer.new()
	box.add_theme_constant_override("separation", 4)
	_draft_panel.add_child(box)
	var head: HBoxContainer = HBoxContainer.new()
	head.add_theme_constant_override("separation", 16)
	box.add_child(head)
	_draft_title = Label.new()
	_draft_title.theme_type_variation = &"Heading"
	_draft_title.add_theme_color_override("font_color", WOLF_RED)
	_draft_title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(_draft_title)
	_undraft = Button.new()
	_undraft.name = "UndraftAll"
	_undraft.focus_mode = Control.FOCUS_NONE
	_undraft.pressed.connect(undraft_all)
	head.add_child(_undraft)
	_draft_rows = VBoxContainer.new()
	_draft_rows.add_theme_constant_override("separation", 2)
	box.add_child(_draft_rows)
	_draft_panel.hide()


func _on_world_reset() -> void:
	_banner_key = 0
	_draft_key = 0


func _process(delta: float) -> void:
	if _world_sim == null:
		return
	_time += delta
	var story: Dictionary = SimReads.read(_world_sim, "get_story")
	var language: String = StoryText.language(_locale)
	var key: int = hash([story, language])
	if key != _banner_key:
		_banner_key = key
		_fill_banners(story)
	_wolves = Vector2i(int(story.get("wolves_x", -1)), int(story.get("wolves_y", -1))) if int(story.get("wolves", 0)) > 0 else Vector2i(-1, -1)
	_refresh_legend(language)
	var drafted: Dictionary = SimReads.read(_world_sim, "get_drafted")
	var draft_key: int = hash([drafted.get("ids", []), drafted.get("hps", []), drafted.get("orders", []), drafted.get("weapons", []), language])
	if draft_key != _draft_key:
		_draft_key = draft_key
		_fill_drafted(drafted)
	queue_redraw()


func _fill_banners(story: Dictionary) -> void:
	for child: Node in _banners.get_children():
		_banners.remove_child(child)
		child.queue_free()
	_texts.clear()
	for banner: Dictionary in StoryText.banners(_locale, story):
		var panel: PanelContainer = PanelContainer.new()
		panel.mouse_filter = Control.MOUSE_FILTER_IGNORE
		panel.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
		var label: Label = Label.new()
		label.text = str(banner["text"])
		label.add_theme_color_override("font_color", banner["color"] as Color)
		label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		panel.add_child(label)
		_banners.add_child(panel)
		_texts.append(label.text)


func _refresh_legend(language: String) -> void:
	var agents: Node = get_node_or_null("/root/Main/ColonyView/AgentLayer")
	var shown: bool = agents != null and bool(agents.call("mood_shown"))
	var panel: Control = _legend.get_parent() as Control
	panel.visible = shown
	if shown and str(_legend.get_meta("language", "")) != language:
		_legend.text = StoryText.ltr(_locale, "M5_MOOD_LEGEND")
		_legend.set_meta("language", language)


func _fill_drafted(drafted: Dictionary) -> void:
	for child: Node in _draft_rows.get_children():
		_draft_rows.remove_child(child)
		child.queue_free()
	_draft_texts.clear()
	var ids: PackedInt64Array = drafted.get("ids", PackedInt64Array())
	var hps: PackedInt32Array = drafted.get("hps", PackedInt32Array())
	var weapons: PackedInt32Array = drafted.get("weapons", PackedInt32Array())
	_draft_panel.visible = not ids.is_empty()
	if ids.is_empty():
		return
	var language: String = StoryText.language(_locale)
	var animals: Dictionary = SimReads.read(_world_sim, "get_animals")
	_draft_title.text = StoryText.trf(_locale, "M5_DRAFT_TITLE_FMT", {"count": ids.size()})
	_undraft.text = StoryText.ltr(_locale, "M5_DRAFT_UNDRAFT_ALL")
	for i: int in ids.size():
		var m5: Variant = _world_sim.call("get_agent_m5", ids[i])
		var read: Dictionary = m5 if m5 is Dictionary else {}
		var order: String = StoryText.order_text(_locale, int(read.get("draft_order", 0)), int(read.get("draft_x", -1)), int(read.get("draft_y", -1)), animals)
		var row: Label = Label.new()
		row.text = StoryText.trf(_locale, "M5_DRAFT_ROW_FMT", {"name": Names.present(ids[i], language), "hp": hps[i] if i < hps.size() else 0,
			"order": order, "weapon": StoryText.weapon(_locale, weapons[i] if i < weapons.size() else -1)})
		_draft_rows.add_child(row)
		_draft_texts.append(row.text)


## 모두 해제: release every drafted person; the toast counts them. Returns the result.
func undraft_all() -> Dictionary:
	if _world_sim == null:
		return {}
	var result: Variant = SimReads.mutate(_world_sim, "cmd_undraft_all")
	var data: Dictionary = result if result is Dictionary else {}
	var bar: Node = get_node_or_null("../OrderBar")
	if bar != null and bar.has_method("show_toast"):
		var text: String = StoryText.trf(_locale, "M5_TOAST_UNDRAFT_ALL_FMT", {"count": int(data.get("count", 0))}) if bool(data.get("accepted", false)) \
			else EconomyText.reason(_locale, str(data.get("reason", "nothing")))
		bar.call("show_toast", text)
	_draft_key = 0
	return data


## The banner texts shown (for tests and tools).
func banner_texts() -> Array[String]:
	return _texts


## The drafted list's rows (for tests and tools).
func drafted_texts() -> Array[String]:
	return _draft_texts


## The wolves' pulse: {shown, on_screen, at} in UI pixels (for tests and tools).
func pulse() -> Dictionary:
	if _wolves.x < 0:
		return {"shown": false}
	var at: Vector2 = _screen_of(_wolves)
	var view: Rect2 = Rect2(Vector2.ZERO, Vector2(1920, 1080))
	return {"shown": true, "on_screen": view.has_point(at), "at": at, "edge": _edge_point(at)}


func _screen_of(tile: Vector2i) -> Vector2:
	return get_viewport().get_canvas_transform() * (View.tile_to_world(tile) + Vector2(View.TILE * 0.5, View.TILE * 0.5))


## The point on the screen's edge (inset by EDGE) toward `at` from the screen's centre.
static func _edge_point(at: Vector2) -> Vector2:
	var centre: Vector2 = Vector2(960, 540)
	var d: Vector2 = at - centre
	if d.length() < 1.0:
		return centre
	var sx: float = (960.0 - EDGE) / maxf(absf(d.x), 0.001)
	var sy: float = (540.0 - EDGE) / maxf(absf(d.y), 0.001)
	return centre + d * minf(sx, sy)


func _draw() -> void:
	if _wolves.x < 0:
		return
	var at: Vector2 = _screen_of(_wolves)
	var pulse_k: float = 0.5 + 0.5 * sin(_time * PULSE_SPEED)
	var colour: Color = Color(WOLF_RED, 0.35 + 0.55 * pulse_k)
	if Rect2(Vector2.ZERO, Vector2(1920, 1080)).has_point(at):
		draw_arc(at, 20.0 + 10.0 * pulse_k, 0.0, TAU, 32, colour, 3.0)
		return
	var edge: Vector2 = _edge_point(at)
	var dir: Vector2 = (at - Vector2(960, 540)).normalized()
	var side: Vector2 = Vector2(-dir.y, dir.x)
	var tip: Vector2 = edge + dir * (12.0 + 6.0 * pulse_k)
	draw_colored_polygon(PackedVector2Array([tip, edge - dir * 10.0 + side * 14.0, edge - dir * 10.0 - side * 14.0]), colour)
	draw_arc(edge, 22.0 + 8.0 * pulse_k, 0.0, TAU, 32, colour, 2.0)
