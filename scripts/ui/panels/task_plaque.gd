# task_plaque.gd (UI/TaskPlaque in scenes/main.tscn): the "지금 할 일" plaque under the topbar.
#   M-LOOP §2.1: always titled "지금 할 일". Content priority:
#     1. the top open card's title (from CardPanel);
#     2. the chapter's next unmet condition plus a tool hint (from GoalsPanel);
#     3. "할 일 없음 — 집사가 마을을 돌보고 있어요".
#   When the steward is handling it, show the player's next step.
# Bridge methods: via SimReads get_current_task, get_requests, get_settlement_diagnosis;
#   signal world_reset.
extends Control

const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")
const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const PanelTooltips = preload("res://scripts/ui/panels/panel_tooltips.gd")
const DiagnosisText = preload("res://scripts/ui/panels/diagnosis_text.gd")

const TILE: float = 64.0

var _locale: Node = null
var _world_sim: Node = null
var _panel: PanelContainer
var _text_label: Label
var _title_label: Label
var _chapter_label: Label
var _view_button: Button
var _task_data: Dictionary = {}
var _task_tick: int = -1
var _marker_tile: Vector2i = Vector2i(-1, -1)
var _current_text: String = ""


func _ready() -> void:
	_locale = get_node_or_null("/root/Locale")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _world_reset)
	# Position at the top centre of the screen.
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	anchors_preset = PRESET_TOP_WIDE
	var row: HBoxContainer = HBoxContainer.new()
	row.alignment = BoxContainer.ALIGNMENT_CENTER
	row.add_theme_constant_override("separation", 12)
	row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(row)
	# A centring wrapper so the panel floats top-centre.
	_panel = PanelContainer.new()
	_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var style: StyleBoxFlat = StyleBoxFlat.new()
	style.bg_color = Color(0.08, 0.07, 0.06, 0.85)
	style.border_color = Color(0.45, 0.38, 0.28, 0.6)
	style.set_border_width_all(1)
	style.set_corner_radius_all(4)
	style.set_content_margin_all(8)
	_panel.add_theme_stylebox_override("panel", style)
	row.add_child(_panel)
	var inner: HBoxContainer = HBoxContainer.new()
	inner.add_theme_constant_override("separation", 12)
	_panel.add_child(inner)
	# M-LOOP §2.1: the plaque is always titled "지금 할 일" (owner playtest verdict 2).
	_title_label = Label.new()
	_title_label.name = "Title"
	_title_label.add_theme_color_override("font_color", Color("e8c46e"))
	inner.add_child(_title_label)
	# The current chapter, always visible next to the title (owner check 1: "1장: 겨울을 난다 — 조건 0/4").
	_chapter_label = Label.new()
	_chapter_label.name = "Chapter"
	_chapter_label.add_theme_color_override("font_color", Color("b8a888"))
	inner.add_child(_chapter_label)
	_text_label = Label.new()
	_text_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_text_label.custom_minimum_size.x = 200
	_text_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	inner.add_child(_text_label)
	_view_button = Button.new()
	_view_button.name = "ViewButton"
	PanelTooltips.attach(_view_button, "plaque")
	_view_button.pressed.connect(_view_pressed)
	_view_button.focus_mode = Control.FOCUS_NONE
	inner.add_child(_view_button)
	position = Vector2(0, 8)
	visible = false
	# Ambient guidance: draw (and hit-test) behind every other UI panel and menu, so an opened
	# build menu or inspector over the plaque's slot still gets its clicks.
	get_parent().move_child.call_deferred(self, 0)


func _process(_delta: float) -> void:
	if _world_sim == null:
		visible = false
		return
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", -1))
	if tick == _task_tick:
		return
	_task_tick = tick
	_task_data = SimReads.read(_world_sim, "get_current_task")
	_refresh()
	_place()


## M-READ: sit in MuxHud's plaque slot (left of the action row, under the topbar) at the UI scale,
## wide enough that the line wraps to at most a few rows instead of a narrow column.
func _place() -> void:
	var hud: Node = get_node_or_null("../MuxHud")
	var slot: Dictionary = hud.call("plaque_slot") if hud != null and hud.has_method("plaque_slot") else {}
	if slot.is_empty():
		return
	position = slot["position"]
	scale = Vector2.ONE * float(slot["factor"])
	_text_label.custom_minimum_size.x = float(slot["width"]) - 120.0


func _refresh() -> void:
	# M-LOOP §2.1: content priority — card → chapter condition → idle message.
	# Priority 1: the top open card's title.
	var card_panel: Node = get_node_or_null("../MuxHud/CardPanel") if get_node_or_null("../MuxHud") != null else null
	if card_panel == null:
		card_panel = get_node_or_null("../CardPanel")
	if card_panel != null and card_panel.has_method("has_active_cards") and card_panel.call("has_active_cards"):
		var card_title: String = str(card_panel.call("top_card_title"))
		if not card_title.is_empty():
			_show_line(_loc("LOOP_PLAQUE_CARD_FMT").format({"title": card_title}), "")
			return
	# Priority 2: the chapter's next unmet condition.
	var goals: Node = get_node_or_null("../MuxHud/Goals") if get_node_or_null("../MuxHud") != null else null
	if goals == null:
		goals = get_node_or_null("../Goals")
	if goals != null and goals.has_method("next_unmet_condition"):
		var unmet: Dictionary = goals.call("next_unmet_condition")
		if not unmet.is_empty():
			var hint: String = str(unmet.get("text", ""))
			_show_line(_loc("LOOP_PLAQUE_CHAPTER_FMT").format({"condition": hint, "hint": ""}), "")
			return
	# Priority 3: fall back to request-based plaque (M-READ behaviour).
	var source: String = str(_task_data.get("source", "none"))
	if source != "none" and not _task_data.is_empty():
		var voice_name: String = str(_task_data.get("voice_name", ""))
		if voice_name.is_empty():
			var voice_id: int = int(_task_data.get("voice", -1))
			if voice_id >= 0:
				var language: String = "ko"
				if _locale != null and _locale.has_method("current_language"):
					language = str(_locale.call("current_language"))
				voice_name = Names.present(voice_id, language)
		var text_key: String = str(_task_data.get("text_key", ""))
		var action_key: String = str(_task_data.get("action_key", ""))
		var is_steward: bool = bool(_task_data.get("steward_active", false))
		var text: String = _loc(text_key)
		var voice: String = Names.unbroken(voice_name) if not voice_name.is_empty() else ""
		var action: String = _loc(action_key) if not action_key.is_empty() else ""
		var status: String = _loc("READ_STEWARD_DOING") if is_steward else _loc("READ_STEWARD_TODO")
		var diag: Dictionary = SimReads.read(_world_sim, "get_settlement_diagnosis")
		var request_tags: PackedStringArray = _task_data.get("match_tags", PackedStringArray())
		var reason: String = DiagnosisText.request_reason(diag, request_tags, _locale)
		var shown: String = text + _loc("READ_DIAG_CHAIN_SEP") + reason if not reason.is_empty() else text
		var line: String = _loc("READ_PLAQUE_FMT").format({"voice": voice, "text": shown, "status": status, "action": action})
		if voice.is_empty():
			line = line.trim_prefix(": ")
		var check: int = DiagnosisText.request_check(diag, request_tags)
		var chain: String = DiagnosisText.chain_text(diag, check, _locale) if check >= 0 else DiagnosisText.headline_and_next(diag, _locale)
		_show_line(line, chain)
		var has_marker: bool = bool(_task_data.get("has_marker", false))
		_marker_tile = Vector2i(int(_task_data.get("marker_x", -1)), int(_task_data.get("marker_y", -1))) if has_marker else Vector2i(-1, -1)
		_view_button.visible = has_marker
		return
	# Priority 4: idle message.
	_show_line(_loc("LOOP_PLAQUE_IDLE"), "")
	_view_button.visible = false


## M-LOOP §2.1: common display logic for the plaque content.
func _show_line(line: String, tooltip: String) -> void:
	_current_text = line
	_title_label.text = _loc("LOOP_PLAQUE_TITLE")
	var goals: Node = get_node_or_null("../Goals")
	if goals == null:
		goals = get_tree().root.find_child("Goals", true, false) if is_inside_tree() else null
	_chapter_label.text = KoreanText.words(str(goals.call("chapter_line"))) if goals != null and goals.has_method("chapter_line") else ""
	_chapter_label.visible = not _chapter_label.text.is_empty()
	_text_label.text = KoreanText.words(line)
	_view_button.text = _loc("READ_PLAQUE_VIEW")
	_panel.tooltip_text = KoreanText.raw(tooltip) if not tooltip.is_empty() else ""
	visible = true


func _view_pressed() -> void:
	if _marker_tile.x < 0:
		return
	var camera: Camera2D = get_node_or_null("/root/Main/Camera2D") as Camera2D
	if camera == null:
		return
	if camera.has_method("set_auto_track"):
		camera.call("set_auto_track", false)
	camera.position = Vector2(_marker_tile) * TILE + Vector2(32, 32)


func _loc(key: String) -> String:
	if _locale != null and _locale.has_method("ltr"):
		return KoreanText.words(str(_locale.call("ltr", key)))
	return key


func _world_reset() -> void:
	_task_tick = -1
	_task_data = {}
	_marker_tile = Vector2i(-1, -1)
	_current_text = ""
	visible = false


## The plaque's current text (for tests).
func plaque_text() -> String:
	return _current_text


## The plaque's hover text: the full reason chain (for tests and captures).
func plaque_tooltip() -> String:
	return _panel.tooltip_text


## The plaque's marker tile (for tests).
func marker_tile() -> Vector2i:
	return _marker_tile
