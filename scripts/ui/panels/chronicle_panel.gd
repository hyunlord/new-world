extends Control
const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const NOTABLE: Array[int] = [0, 2, 11, 13, 14, 15, 16]
var _world: Node
var _locale: Node
var _title: Label
var _body: RichTextLabel
var _next: int = 0
var _total: int = 0
var _tick: int = -1
var _language: String = ""
var _entries: Array[Dictionary] = []

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
	_body = RichTextLabel.new()
	_body.bbcode_enabled = true
	_body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body.meta_clicked.connect(_follow_link)
	box.add_child(_body)
	_refresh()
	hide()

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
		for row: Dictionary in Chronicle.rows(page):
			if int(row["kind"]) in NOTABLE:
				_entries.append(row)
		while _entries.size() > 60:
			_entries.pop_front()
	if visible and (changed or Chronicle.language(_locale) != _language):
		_refresh()

func _refresh() -> void:
	_language = Chronicle.language(_locale)
	_title.text = Chronicle.loc(_locale, "UI_CHRONICLE_TITLE")
	var info: Dictionary = SimReads.read(_world, "get_world_info") if _world != null and _world.has_method("get_world_info") else {}
	var lines: Array[String] = []
	for index: int in range(_entries.size() - 1, -1, -1):
		lines.append(Chronicle.line(_entries[index], -1, _locale, info, true))
	_body.text = "\n\n".join(lines)

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
	var inspector: Node = get_node_or_null("../AgentInspectorPanel")
	if inspector != null:
		inspector.call("_follow_link", meta)

func _reset() -> void:
	_next = 0
	_total = 0
	_tick = -1
	_entries.clear()
	_refresh()
	hide()
