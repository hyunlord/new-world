# room_panel.gd (UI/RoomPanel, created in code by order_bar.gd): the M3 room panel (goal 9.3). Opened by a click on a
#   room tile while the room overlay (R) is shown (order_tool.gd `click_world`); Esc or 닫기 closes it.
# Shows the room's name ("침실 3", "(역할 고정)" when the player set the role), a role dropdown (자동 = the colony
#   decides, then 침실 / 창고 / 작업장 / 홀 / 미정) that sends `cmd_set_room_role`, the tile and bed counts, the roof
#   (지붕 있음 / 지붕 없음 / 판 굴), the doors (or "문이 없어 들어갈 수 없음"), the items under its roof, the people
#   asleep in it now and the "why?" line "침실 · 지붕 있음 · 침대 4 · 추위 보호 · 잠이 빨리 회복".
# Re-read once per tick while open (get_room_detail takes an argument, so it is called directly, never per frame).
# INTEG1: when the room lies in a HOUSE1 house, a house section on top ("카토네 집": first completed day, template and
#   era, rooms/beds/hearths, family, lodgers, free beds; house_text.gd). "The selected house" is this panel open on one
#   of its rooms: `selected_house()` / `select_house(id)`.
# Bridge methods: get_room_detail (direct); via SimReads get_time_snapshot, get_rooms, get_house_snapshot,
#   get_household_snapshot, get_world_info; via SimReads.mutate cmd_set_room_role; signal world_reset.
extends Control

const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")

const PanelTooltips = preload("res://scripts/ui/panels/panel_tooltips.gd")

const ArtUI = preload("res://scripts/ui/panels/art_ui.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const RoomIndex = preload("res://scripts/world/room_index.gd")
const DiagnosisText = preload("res://scripts/ui/panels/diagnosis_text.gd")
const HouseText = preload("res://scripts/ui/panels/house_text.gd")
const PANEL_WIDTH: float = 520.0
## Dropdown entries: the bridge role names `cmd_set_room_role` takes, and their text.
const ROLE_CHOICES: Array[String] = ["auto", "bedroom", "storehouse", "workshop", "hall", "unassigned"]
const CHOICE_KEYS: Array[String] = ["ROOM_ROLE_AUTO", "ROOM_ROLE_BEDROOM", "ROOM_ROLE_STOREHOUSE", "ROOM_ROLE_WORKSHOP", "ROOM_ROLE_HALL", "ROOM_ROLE_UNASSIGNED"]
const MUTED: Color = Color("c8b89a")

var _world_sim: Node = null
var _locale: Node = null
var _room: int = -1
var _food_tile: Vector2i = Vector2i(-1, -1)
var _food_button: Button
var _detail: Dictionary = {}
var _tick: int = -1
var _title: Label
var _role_label: Label
var _role: OptionButton
var _lines: Label
var _why: Label
var _diag_label: Label
var _close: Button
var _house_box: VBoxContainer
var _house_title: Label
var _house_lines: Label
var _house: int = -1
var _house_rule: HSeparator
## The room-only controls; hidden when the panel shows a house without rooms (a hut).
var _room_box: VBoxContainer


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
	_house_box = VBoxContainer.new()
	_house_box.name = "House"
	_house_box.add_theme_constant_override("separation", 8)
	_house_box.hide()
	column.add_child(_house_box)
	_house_title = Label.new()
	_house_title.theme_type_variation = &"Heading"
	_house_box.add_child(_house_title)
	_house_lines = Label.new()
	_house_lines.autowrap_mode = TextServer.AUTOWRAP_WORD
	_house_lines.custom_minimum_size.x = PANEL_WIDTH - 32
	_house_box.add_child(_house_lines)
	_house_rule = HSeparator.new()
	_house_box.add_child(_house_rule)
	_room_box = VBoxContainer.new()
	_room_box.name = "Room"
	_room_box.add_theme_constant_override("separation", 8)
	column.add_child(_room_box)
	_title = Label.new()
	_title.theme_type_variation = &"Heading"
	_room_box.add_child(_title)
	var row: HBoxContainer = HBoxContainer.new()
	row.add_theme_constant_override("separation", 12)
	_room_box.add_child(row)
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
	_room_box.add_child(_lines)
	_why = Label.new()
	_why.autowrap_mode = TextServer.AUTOWRAP_WORD
	_why.custom_minimum_size.x = PANEL_WIDTH - 32
	_why.add_theme_color_override("font_color", MUTED)
	_room_box.add_child(_why)
	_diag_label = Label.new()
	_diag_label.name = "RoomDiagnosis"
	_diag_label.autowrap_mode = TextServer.AUTOWRAP_WORD
	_diag_label.custom_minimum_size.x = PANEL_WIDTH - 32
	_diag_label.add_theme_color_override("font_color", Color("a8d888"))
	_diag_label.hide()
	_room_box.add_child(_diag_label)
	var why_button := Button.new()
	why_button.name = "Why"
	why_button.text = _loc("MUX_WHY")
	PanelTooltips.attach(why_button, "", "MUX_WHY")
	why_button.pressed.connect(_open_tile_why)
	_room_box.add_child(why_button)
	_food_button = Button.new()
	_food_button.name = "FoodWhy"
	_food_button.disabled = true
	_food_button.text = _loc("M6A_FOOD_WHY")
	PanelTooltips.attach(_food_button, "", "M6A_FOOD_WHY")
	_food_button.pressed.connect(_open_food)
	_room_box.add_child(_food_button)
	_close = Button.new()
	PanelTooltips.attach(_close, "close")
	_close.focus_mode = Control.FOCUS_NONE
	_close.pressed.connect(close)
	column.add_child(_close)


func _loc(key: String) -> String:
	return KoreanText.words(EconomyText.ltr(_locale, key))


## Open the panel on room `id`.
func display_room(id: int) -> void:
	_food_tile = Vector2i(-1, -1)
	_food_button.disabled = true
	_room = id
	_house = -1
	_tick = -1
	visible = true
	refresh()


## The room shown (-1 when closed).
func current_room() -> int:
	return _room if visible else -1


## The last room detail read ({found, id, role, …}; for tests and tools).
func detail() -> Dictionary:
	return _detail


## The house id of the shown room (-1 when closed or the room is in no house).
func selected_house() -> int:
	return _house if visible else -1


## Open the panel on house `house_id`: on its first room, or the house section alone when it has none (a hut).
func select_house(house_id: int) -> void:
	var house: Dictionary = HouseText.by_id(_world_sim, house_id)
	if house.is_empty():
		return
	var rooms: PackedInt64Array = house.get("room_ids", PackedInt64Array())
	if not rooms.is_empty():
		display_room(rooms[0])
		return
	_food_tile = Vector2i(-1, -1)
	_food_button.disabled = true
	_room = -1
	_house = house_id
	_tick = -1
	visible = true
	refresh()


func close() -> void:
	_room = -1
	_house = -1
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


## Read the room (or the roomless house) again and fill the panel (closes when it is gone).
func refresh() -> void:
	if _world_sim == null or (_room < 0 and _house < 0):
		return
	_tick = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", -1))
	_room_box.visible = _room >= 0
	_house_rule.visible = _room >= 0
	_close.text = _loc("UI_CLOSE")
	if _room < 0:
		var hut: Dictionary = HouseText.by_id(_world_sim, _house)
		if hut.is_empty():
			close()
			return
		_show_house(hut)
		return
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
			EconomyText.trf(_locale, "UI_ROOM_BEDS_FMT", {"beds": beds}), RoomIndex.roof_text(info, _locale)],
		EconomyText.trf(_locale, "UI_ROOM_DOORS_FMT", {"doors": doors}) if doors > 0 else _loc("UI_ROOM_NO_DOOR"),
		EconomyText.trf(_locale, "UI_ROOM_COVERED_FMT", {"count": int(_detail.get("covered", 0))}),
		EconomyText.trf(_locale, "UI_ROOM_SLEEPERS_FMT", {"count": int(_detail.get("sleepers", 0))}),
	]
	_lines.text = KoreanText.words("\n".join(lines))
	_why.text = KoreanText.words(why_line(role_code, info, beds, _locale))
	# M-READ: show room diagnosis from get_subject_diagnosis("room", room_id).
	_diag_label.hide()
	if _world_sim != null and _world_sim.has_method("get_subject_diagnosis"):
		var diag: Dictionary = _world_sim.get_subject_diagnosis("room", _room)
		var room_diag: String = DiagnosisText.headline_and_next(diag, _locale)
		if not room_diag.is_empty():
			_diag_label.text = KoreanText.words(room_diag)
			_diag_label.show()
	_show_house(HouseText.of_room(_world_sim, _room))


## Fill the house section from house row `house` (hidden for {}).
func _show_house(house: Dictionary) -> void:
	_house = int(house.get("house_id", -1))
	_house_box.visible = _house >= 0
	if _house >= 0:
		_house_title.text = HouseText.title(_world_sim, house, _locale)
		_house_lines.text = "\n".join(HouseText.lines(_world_sim, house, _locale))


## The "why?" line: "침실 · 지붕 있음 · 침대 4 · 추위 보호 · 잠이 빨리 회복" (`UI_ROOM_WHY_FMT`).
static func why_line(role_code: int, info: Dictionary, beds: int, locale: Node) -> String:
	var parts: Array[String] = []
	for key: String in effect_keys(role_code, bool(info.get("roofed", false)) or bool(info.get("dug", false))):
		parts.append(EconomyText.ltr(locale, key))
	return EconomyText.trf(locale, "UI_ROOM_WHY_FMT", {"role": EconomyText.role(locale, role_code),
		"roof": RoomIndex.roof_text(info, locale), "beds": beds, "effect": " · ".join(parts)})


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


func set_food_tile(tile: Vector2i) -> void:
	_food_tile = tile
	_food_button.disabled = false

func _open_food() -> void:
	if _food_tile.x >= 0:
		get_node("../CausalPanel").call("open_food", _food_tile)
