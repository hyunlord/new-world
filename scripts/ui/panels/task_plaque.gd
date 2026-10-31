# task_plaque.gd (UI/TaskPlaque in scenes/main.tscn): the single "지금 할 일" plaque at the top of
#   the screen; one line showing the top active request or nothing; reason chain on hover; a "보기"
#   button that centres the camera on the marker tile. Must not duplicate an alert line word for word.
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
	var icon: Label = Label.new()
	icon.text = "▶"
	icon.add_theme_color_override("font_color", Color("e8c46e"))
	inner.add_child(icon)
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


func _refresh() -> void:
	var source: String = str(_task_data.get("source", "none"))
	if source == "none" or _task_data.is_empty():
		visible = false
		_current_text = ""
		return
	var voice_name: String = str(_task_data.get("voice_name", ""))
	# M-READ: the bridge may not fill voice_name; look it up from the AgentId.
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
	var line: String = _loc("READ_PLAQUE_FMT").format({"voice": voice, "text": text, "status": status, "action": action})
	# Deduplicate against active alerts.
	if _is_alert_duplicate(text):
		# Use the request line with voice to differentiate.
		line = voice + ": " + text + " — " + status + ": " + action
	_current_text = line
	_text_label.text = KoreanText.words(line)
	_view_button.text = _loc("READ_PLAQUE_VIEW")
	# Set the hover tooltip to the full reason chain from settlement diagnosis.
	var diag: Dictionary = SimReads.read(_world_sim, "get_settlement_diagnosis")
	var chain: String = DiagnosisText.headline_and_next(diag, _locale)
	if not chain.is_empty():
		_panel.tooltip_text = chain
	else:
		_panel.tooltip_text = ""
	# Track marker tile.
	var has_marker: bool = bool(_task_data.get("has_marker", false))
	_marker_tile = Vector2i(int(_task_data.get("marker_x", -1)), int(_task_data.get("marker_y", -1))) if has_marker else Vector2i(-1, -1)
	_view_button.visible = has_marker
	visible = true


## Whether `text` duplicates an active alert line word for word.
func _is_alert_duplicate(text: String) -> bool:
	var alerts: Node = get_node_or_null("../MuxHud")
	if alerts == null or not alerts.has_method("active_alert_texts"):
		return false
	var alert_texts: PackedStringArray = alerts.call("active_alert_texts")
	for alert_text: String in alert_texts:
		if alert_text == text:
			return true
	return false


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


## The plaque's marker tile (for tests).
func marker_tile() -> Vector2i:
	return _marker_tile
