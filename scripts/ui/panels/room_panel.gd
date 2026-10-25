# room_panel.gd (UI/RoomPanel, created in code by order_bar.gd): the M3 room panel (goal 9.3). Opened by a click on a
#   room tile while the room overlay (R) is shown (order_tool.gd `click_world`); Esc or 닫기 closes it.
# Shows the room's name ("침실 3", "(역할 고정)" when the player set the role), a role dropdown (자동 = the colony
#   decides, then 침실 / 창고 / 작업장 / 홀 / 미정) that sends `cmd_set_room_role`, the tile and bed counts, the roof
#   (지붕 있음 / 지붕 없음 / 판 굴), the doors (or "문이 없어 들어갈 수 없음"), the items under its roof, the people
#   asleep in it now and the "why?" line "침실 · 지붕 있음 · 침대 4 · 추위 보호 · 잠이 빨리 회복".
# Re-read once per tick while open (get_room_detail takes an argument, so it is called directly, never per frame).
# Bridge methods: get_room_detail (direct); via SimReads get_time_snapshot, get_rooms; via SimReads.mutate cmd_set_room_role;
#   signal world_reset.
extends Control

const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")

const PanelTooltips = preload("res://scripts/ui/panels/panel_tooltips.gd")

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const RoomLayer = preload("res://scripts/world/colony/room_layer.gd")
const PANEL_WIDTH: float = 520.0
## Dropdown entries: the bridge role names `cmd_set_room_role` takes, and their text.
const ROLE_CHOICES: Array[String] = ["auto", "bedroom", "storehouse", "workshop", "hall", "unassigned"]
const CHOICE_KEYS: Array[String] = ["ROOM_ROLE_AUTO", "ROOM_ROLE_BEDROOM", "ROOM_ROLE_STOREHOUSE", "ROOM_ROLE_WORKSHOP", "ROOM_ROLE_HALL", "ROOM_ROLE_UNASSIGNED"]
const MUTED: Color = Color("c8b89a")

var _world_sim: Node = null
var _locale: Node = null
var _room: int = -1
var _detail: Dictionary = {}
var _tick: int = -1
var _title: Label
var _role_label: Label
var _role: OptionButton
var _lines: Label
var _why: Label
var _close: Button


func _ready() -> void:
	_locale = get_node_or_null("/root/Locale")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", close)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	position = Vector2(12, 124)
	var panel: PanelContainer = ArtUI.panel(self, Rect2(Vector2.ZERO, Vector2(PANEL_WIDTH, 0)))
	panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var column: VBoxContainer = VBoxContainer.new()
	column.add_theme_constant_override("separation", 8)
	panel.add_child(column)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	column.add_child(_title)
	var row: HBoxContainer = HBoxContainer.new()
	row.add_theme_constant_override("separation", 12)
	column.add_child(row)
	_role_label = Label.new()
	row.add_child(_role_label)
	_role = OptionButton.new()
	PanelTooltips.attach(_role, "rooms", "MUX_TIP_ROOM_ROLE")
	_role.name = "RoleChoice"
	_role.focus_mode = Control.FOCUS_NONE
	for key: String in CHOICE_KEYS:
		_role.add_item(_loc(key))
	_role.item_selected.connect(choose_role)
	row.add_child(_role)
	_lines = Label.new()
	_lines.autowrap_mode = TextServer.AUTOWRAP_WORD
	_lines.custom_minimum_size.x = PANEL_WIDTH - 32
	column.add_child(_lines)
	_why = Label.new()
	_why.autowrap_mode = TextServer.AUTOWRAP_WORD
	_why.custom_minimum_size.x = PANEL_WIDTH - 32
	_why.add_theme_color_override("font_color", MUTED)
	column.add_child(_why)
	var why_button := Button.new()
	why_button.name = "Why"
	why_button.text = _loc("MUX_WHY")
	why_button.pressed.connect(_open_tile_why)
	column.add_child(why_button)
	_close = Button.new()
	PanelTooltips.attach(_close, "close")
	_close.focus_mode = Control.FOCUS_NONE
	_close.pressed.connect(close)
	column.add_child(_close)


func _loc(key: String) -> String:
	return KoreanText.words(EconomyText.ltr(_locale, key))


## Open the panel on room `id`.
func display_room(id: int) -> void:
	_room = id
	_tick = -1
	visible = true
	refresh()


## The room shown (-1 when closed).
func current_room() -> int:
	return _room if visible else -1


## The last room detail read ({found, id, role, …}; for tests and tools).
func detail() -> Dictionary:
	return _detail


func close() -> void:
	_room = -1
	visible = false


func _process(_delta: float) -> void:
	if not visible or _world_sim == null:
		return
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", -1))
	if tick != _tick:
		refresh()


func _unhandled_input(event: InputEvent) -> void:
	if visible and event is InputEventKey and event.pressed and not event.echo and (event as InputEventKey).keycode == KEY_ESCAPE:
		close()
		get_viewport().set_input_as_handled()


## Read the room again and fill the panel (closes when the room is gone).
func refresh() -> void:
	if _world_sim == null or _room < 0:
		return
	_tick = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", -1))
	var read: Variant = _world_sim.call("get_room_detail", _room)
	_detail = read if read is Dictionary else {}
	if not bool(_detail.get("found", false)):
		close()
		return
	var role_code: int = int(_detail.get("role", 0))
	var role_text: String = EconomyText.role(_locale, role_code)
	_title.text = "%s · %s" % [_loc("UI_ROOM_PANEL_TITLE"), EconomyText.trf(_locale, "UI_ROOM_NAME_FMT", {"role": role_text, "id": _room})]
	if bool(_detail.get("overridden", false)):
		_title.text += " " + _loc("UI_ROOM_OVERRIDE")
	_role_label.text = _loc("UI_ROOM_ROLE")
	var choice: int = ROLE_CHOICES.find(str(_detail.get("role_name", ""))) if bool(_detail.get("overridden", false)) else 0
	_role.select(maxi(0, choice))
	var info: Dictionary = {"roofed": bool(_detail.get("roofed", false)), "dug": bool(_detail.get("dug", false))}
	var beds: int = int(_detail.get("beds", 0))
	var doors: int = int(_detail.get("doors", 0))
	var lines: Array[String] = [
		"%s · %s · %s" % [EconomyText.trf(_locale, "UI_ROOM_TILES_FMT", {"tiles": int(_detail.get("tiles", 0))}),
			EconomyText.trf(_locale, "UI_ROOM_BEDS_FMT", {"beds": beds}), RoomLayer.roof_text(info, _locale)],
		EconomyText.trf(_locale, "UI_ROOM_DOORS_FMT", {"doors": doors}) if doors > 0 else _loc("UI_ROOM_NO_DOOR"),
		EconomyText.trf(_locale, "UI_ROOM_COVERED_FMT", {"count": int(_detail.get("covered", 0))}),
		EconomyText.trf(_locale, "UI_ROOM_SLEEPERS_FMT", {"count": int(_detail.get("sleepers", 0))}),
	]
	_lines.text = KoreanText.words("\n".join(lines))
	_why.text = KoreanText.words(why_line(role_code, info, beds, _locale))
	_close.text = _loc("UI_CLOSE")


## The "why?" line: "침실 · 지붕 있음 · 침대 4 · 추위 보호 · 잠이 빨리 회복" (`UI_ROOM_WHY_FMT`).
static func why_line(role_code: int, info: Dictionary, beds: int, locale: Node) -> String:
	var parts: Array[String] = []
	for key: String in effect_keys(role_code, bool(info.get("roofed", false)) or bool(info.get("dug", false))):
		parts.append(EconomyText.ltr(locale, key))
	return EconomyText.trf(locale, "UI_ROOM_WHY_FMT", {"role": EconomyText.role(locale, role_code),
		"roof": RoomLayer.roof_text(info, locale), "beds": beds, "effect": " · ".join(parts)})


## The effect keys of a room of role `role_code` (ADR 0023): a roofed bedroom keeps the cold off and speeds sleep,
## a storehouse keeps goods covered, a workshop room speeds crafting, the hall is the evening gathering place; an
## open room or an unassigned one has no effect.
static func effect_keys(role_code: int, roofed: bool) -> Array[String]:
	if not roofed and role_code != 4:
		return ["UI_ROOM_EFFECT_NONE"]
	match role_code:
		1: return ["UI_ROOM_EFFECT_COLD", "UI_ROOM_EFFECT_SLEEP"]
		2: return ["UI_ROOM_EFFECT_STORE"]
		3: return ["UI_ROOM_EFFECT_CRAFT"]
		4: return ["UI_ROOM_EFFECT_GATHER"]
	return ["UI_ROOM_EFFECT_NONE"]


## Set the shown room's role to dropdown entry `index` (0 자동 …); the result shows as the order bar's toast.
func choose_role(index: int) -> Dictionary:
	if _world_sim == null or _room < 0 or index < 0 or index >= ROLE_CHOICES.size():
		return {}
	var result: Variant = SimReads.mutate(_world_sim, "cmd_set_room_role", [_room, ROLE_CHOICES[index]])
	var data: Dictionary = result if result is Dictionary else {}
	var bar: Node = get_node_or_null("/root/Main/UI/OrderBar")
	if bar != null and bar.has_method("show_toast"):
		var text: String = _loc("UI_ORDER_TOAST_ROOM_ROLE") if bool(data.get("accepted", false)) else EconomyText.reason(_locale, str(data.get("reason", "unknown")))
		bar.call("show_toast", text)
	_tick = -1
	return data

func _open_tile_why() -> void:
	var rooms: Dictionary = SimReads.read(_world_sim, "get_rooms")
	var ids: PackedInt32Array = rooms.get("ids", PackedInt32Array())
	var row: int = ids.find(_room)
	var xs: PackedInt32Array = rooms.get("centre_xs", PackedInt32Array())
	var ys: PackedInt32Array = rooms.get("centre_ys", PackedInt32Array())
	if row >= 0 and row < mini(xs.size(), ys.size()):
		get_node("../CausalPanel").call("open_tile", Vector2i(xs[row], ys[row]))
