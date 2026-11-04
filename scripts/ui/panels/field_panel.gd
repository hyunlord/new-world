# One field inspector, issuing ordinary work-mode commands only on player clicks.
# Bridge reads via FieldFacts/SimReads: get_field_tiles, get_crop_defs; player mutation via SimReads: cmd_set_field_work.
extends PanelContainer

const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")
const Facts = preload("res://scripts/ui/field_facts.gd")
var _world: Node
var _locale: Node
var _tile := Vector2i(-1, -1)
var _text: Label
var _result: Label
var _close: Button
var _buttons: Array[Button] = []
var _elapsed: float = 0.0

func _ready() -> void:
	_world = get_node("/root/Main/WorldSim")
	_locale = get_node_or_null("/root/Locale")
	theme = preload("res://assets/ui/worldsim_theme.tres")
	mouse_filter = Control.MOUSE_FILTER_STOP
	var column := VBoxContainer.new()
	column.add_theme_constant_override("separation", 8)
	add_child(column)
	_text = Label.new()
	_text.autowrap_mode = TextServer.AUTOWRAP_WORD
	_text.custom_minimum_size.x = 320
	column.add_child(_text)
	var actions := HBoxContainer.new()
	column.add_child(actions)
	for mode: String in ["now", "paused", "auto"]:
		var button := Button.new()
		button.text = _tr("S1_WORK_" + mode.to_upper())
		button.pressed.connect(_set_work.bind(mode))
		actions.add_child(button)
		_buttons.append(button)
	_result = Label.new()
	_result.autowrap_mode = TextServer.AUTOWRAP_WORD
	column.add_child(_result)
	_close = Button.new()
	_close.text = _tr("UI_CLOSE")
	_close.pressed.connect(hide)
	column.add_child(_close)
	_world.connect("world_reset", hide)
	hide()

func _tr(key: String) -> String:
	return str(_locale.call("ltr", key)) if _locale != null else key

func open_tile(tile: Vector2i) -> void:
	if Facts.at(_world, tile).is_empty():
		hide()
		return
	_tile = tile
	_result.text = ""
	_refresh()
	position = (get_viewport_rect().size - Vector2(360, 300)) * 0.5
	show()

func _process(delta: float) -> void:
	if not visible:
		return
	_elapsed += delta
	if _elapsed >= 0.5:
		_elapsed = 0.0
		_refresh()

func _refresh() -> void:
	var row: Dictionary = Facts.at(_world, _tile)
	if row.is_empty():
		hide()
		return
	_close.text = _tr("UI_CLOSE")
	var phase: int = int(row["phase"])
	var phase_text: String = _tr(Facts.PHASE_KEYS[phase]) if phase >= 0 and phase < Facts.PHASE_KEYS.size() else _tr("UI_UNKNOWN")
	_text.text = _tr("S1_FIELD_DETAIL").format({"crop": _tr(Facts.name_key(_world, int(row["crop"]))), "phase": phase_text, "soil": row["soil"], "x": _tile.x, "y": _tile.y, "work": _tr("S1_WORK_" + str(row["mode"]).to_upper())})
	var definition: Dictionary = Facts.definition(_world, int(row["crop"]))
	var seed: String = preload("res://scripts/ui/economy_text.gd").item(_locale, int(definition.get("seed_item", -1)))
	_text.text += "\n" + _tr("S1_FIELD_REQUIREMENTS").format({"seed": seed, "count": definition.get("seeds_per_tile", 0), "days": definition.get("growth_days", 0), "soil": definition.get("min_fertility", 0)})
	if phase in [0, 1, 5, 6]:
		_text.text += "\n" + _tr("S1_WINDOW_OPEN" if bool(row["sowing_open"]) else "S1_WINDOW_CLOSED")
	if not bool(row["workable"]):
		_text.text += "\n" + _tr("S1_FIELD_COVERED")
	var knowledge: int = int(definition.get("knowledge", -1))
	if knowledge >= 0:
		_text.text += "\n" + _tr("S1_FIELD_KNOWLEDGE").format({"knowledge": preload("res://scripts/ui/knowledge_text.gd").node_name(_locale, knowledge)})
	_text.text = KoreanText.words(_text.text)
	for i: int in _buttons.size():
		_buttons[i].text = _tr("S1_WORK_" + str(["now", "paused", "auto"][i]).to_upper())
		_buttons[i].disabled = not bool(row["active"]) or not bool(row["workable"]) or str(row["mode"]) == ["now", "paused", "auto"][i]

func _set_work(mode: String) -> void:
	var row: Dictionary = Facts.at(_world, _tile)
	if row.is_empty() or not bool(row["active"]) or not bool(row["workable"]) or str(row["mode"]) == mode:
		return
	var result: Variant = SimReads.mutate(_world, "cmd_set_field_work", [_tile.x, _tile.y, mode])
	if result is Dictionary:
		_result.text = _tr("S1_WORK_ACCEPTED") if bool(result.get("accepted", false)) else str(result.get("reason", ""))
	_refresh()

func _unhandled_key_input(event: InputEvent) -> void:
	if visible and event.is_action_pressed("ui_cancel"):
		hide()
		get_viewport().set_input_as_handled()
