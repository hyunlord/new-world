# room_index.gd (Main/RoomIndex in scenes/main.tscn): the M3 rooms as data, shared by every view (M3D-SWITCH E1).
# Purpose: the index tile -> room and room -> row of get_rooms (re-read only when `rooms_version` moves) and the room
#   overlay's shown flag: R (and the order bar's "방 보기 R" button) toggles it, `toggled` tells the order bar, the 2D
#   RoomLayer and the 3D view. The order tool asks `room_at` for a click (the room panel), the inspector `room` for
#   the bed line; the role tints (ROLE_COLORS) and the label / roof words are static so panels use them without a node.
# Bridge methods: via SimReads get_colony_versions, get_rooms; signal world_reset.
extends Node

const EconomyText = preload("res://scripts/ui/economy_text.gd")
## Tint per `RoomRole::code` (0 unassigned grey, 1 bedroom blue, 2 storehouse yellow, 3 workshop teal, 4 hall orange).
const ROLE_COLORS: Array[Color] = [
	Color(0.62, 0.62, 0.62, 0.30), Color(0.35, 0.55, 1.0, 0.32), Color(1.0, 0.85, 0.25, 0.30),
	Color(0.20, 0.80, 0.75, 0.30), Color(1.0, 0.55, 0.20, 0.30),
]
const TOGGLE_KEY: Key = KEY_R

signal toggled(shown: bool)

var _world_sim: Node = null
var _version: int = -1
var _shown: bool = false
## Room id -> {id, role, overridden, tiles, beds, roofed, dug, doors, centre}.
var _rooms: Dictionary = {}
## Tile -> room id.
var _tile_room: Dictionary = {}


func _ready() -> void:
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	_version = -1
	_rooms.clear()
	_tile_room.clear()


func _process(_delta: float) -> void:
	_refresh()


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and (event as InputEventKey).keycode == TOGGLE_KEY:
		toggle()
		get_viewport().set_input_as_handled()


## Re-index when the rooms version moved (also on demand, so a lookup in the frame of a change is current).
func _refresh() -> void:
	if _world_sim == null:
		return
	var version: int = int(SimReads.read(_world_sim, "get_colony_versions").get("rooms_version", 0))
	if version != _version:
		_version = version
		_index(SimReads.read(_world_sim, "get_rooms"))


## Show or hide the room overlay.
func toggle() -> void:
	set_shown(not _shown)


## Show (true) or hide the room overlay.
func set_shown(on: bool) -> void:
	if on == _shown:
		return
	_shown = on
	toggled.emit(on)


## Whether the room overlay is shown.
func is_shown() -> bool:
	return _shown


## The room id of `tile`, or -1.
func room_at(tile: Vector2i) -> int:
	_refresh()
	return int(_tile_room.get(tile, -1))


## Room `id`'s row ({id, role, overridden, tiles, beds, roofed, dug, doors, centre}), or {} when unknown.
func room(id: int) -> Dictionary:
	_refresh()
	return _rooms.get(id, {})


## Number of rooms known (for tests and tools).
func room_count() -> int:
	_refresh()
	return _rooms.size()


## Every room id -> row and tile -> room id (the 2D RoomLayer draws from these).
func rooms() -> Dictionary:
	return _rooms


func tiles() -> Dictionary:
	return _tile_room


func _index(snap: Dictionary) -> void:
	_rooms.clear()
	_tile_room.clear()
	var ids: PackedInt32Array = snap.get("ids", PackedInt32Array())
	var roles: PackedInt32Array = snap.get("roles", PackedInt32Array())
	var overridden: PackedByteArray = snap.get("overridden", PackedByteArray())
	var tiles_: PackedInt32Array = snap.get("tiles", PackedInt32Array())
	var beds: PackedInt32Array = snap.get("beds", PackedInt32Array())
	var roofed: PackedByteArray = snap.get("roofed", PackedByteArray())
	var dug: PackedByteArray = snap.get("dug", PackedByteArray())
	var doors: PackedInt32Array = snap.get("doors", PackedInt32Array())
	var cxs: PackedInt32Array = snap.get("centre_xs", PackedInt32Array())
	var cys: PackedInt32Array = snap.get("centre_ys", PackedInt32Array())
	for i: int in mini(ids.size(), mini(roles.size(), tiles_.size())):
		_rooms[ids[i]] = {
			"id": ids[i],
			"role": clampi(roles[i], 0, ROLE_COLORS.size() - 1),
			"overridden": i < overridden.size() and overridden[i] != 0,
			"tiles": tiles_[i],
			"beds": beds[i] if i < beds.size() else 0,
			"roofed": i < roofed.size() and roofed[i] != 0,
			"dug": i < dug.size() and dug[i] != 0,
			"doors": doors[i] if i < doors.size() else 0,
			"centre": Vector2i(cxs[i], cys[i]) if i < cxs.size() and i < cys.size() else Vector2i(-1, -1),
		}
	var txs: PackedInt32Array = snap.get("tile_xs", PackedInt32Array())
	var tys: PackedInt32Array = snap.get("tile_ys", PackedInt32Array())
	var trooms: PackedInt32Array = snap.get("tile_rooms", PackedInt32Array())
	for i: int in mini(txs.size(), mini(tys.size(), trooms.size())):
		_tile_room[Vector2i(txs[i], tys[i])] = trooms[i]


## The overlay label of room row `info`: "침실 · 35칸 · 침대 4" (no beds: "창고 · 36칸"), "(역할 고정)" appended
## when the player set the role.
static func label_text(info: Dictionary, locale: Node) -> String:
	var role_text: String = EconomyText.role(locale, int(info.get("role", 0)))
	var beds: int = int(info.get("beds", 0))
	var params: Dictionary = {"role": role_text, "tiles": int(info.get("tiles", 0)), "beds": beds}
	var text: String = EconomyText.trf(locale, "UI_ROOM_LABEL_FMT" if beds > 0 else "UI_ROOM_LABEL_NOBED_FMT", params)
	if bool(info.get("overridden", false)):
		text += " " + EconomyText.ltr(locale, "UI_ROOM_OVERRIDE")
	return text


## The roof word of room row `info`: 판 굴 (dug), 지붕 있음 or 지붕 없음.
static func roof_text(info: Dictionary, locale: Node) -> String:
	if bool(info.get("dug", false)):
		return EconomyText.ltr(locale, "UI_ROOM_DUG")
	return EconomyText.ltr(locale, "UI_ROOM_ROOFED" if bool(info.get("roofed", false)) else "UI_ROOM_OPEN")
