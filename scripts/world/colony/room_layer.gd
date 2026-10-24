# room_layer.gd (ColonyView/OverlayLayer/Rooms, made in code by overlay_layer.gd): the M3 room overlay (goal 9.3).
# R toggles it (also the order bar's "방 보기 R" button): every room tile tinted by its role (침실 blue, 창고 yellow,
#   작업장 teal, 홀 orange, 미정 grey; translucent, z −5: above the ground chunks and floors, below the y-sorted world
#   layer with its walls, things and people) with a 2-px line where the room ends (64-px tiles, M-ART2), and at each
#   room's centre tile a label on a dark backing box (child "Labels", z 6, above everything; the text keeps
#   `labels.font_px` screen pixels at every zoom and is redrawn when the zoom changes) "침실 · 35칸 · 침대 4" (`UI_ROOM_LABEL_FMT`, without beds `UI_ROOM_LABEL_NOBED_FMT`, "(역할 고정)" when
#   the player set the role). The index (tile -> room, room -> row) is kept while the overlay is hidden too: the
#   order tool asks `room_at` for a click (the room panel), the overlay hover and the inspector ask `room` for
#   "방: 창고 2" and the bed line.
# Re-read only when `rooms_version` moves (get_colony_versions is read every frame through SimReads).
# Bridge methods: via SimReads get_colony_versions, get_rooms; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const FONT_PATH: String = "res://assets/fonts/galmuri/Galmuri11.ttf"
const BACK: Color = Color(0.094, 0.082, 0.075, 0.92)
const TEXT: Color = Color(0.96, 0.92, 0.84)
## Tint per `RoomRole::code` (0 unassigned grey, 1 bedroom blue, 2 storehouse yellow, 3 workshop teal, 4 hall orange).
const ROLE_COLORS: Array[Color] = [
	Color(0.62, 0.62, 0.62, 0.30), Color(0.35, 0.55, 1.0, 0.32), Color(1.0, 0.85, 0.25, 0.30),
	Color(0.20, 0.80, 0.75, 0.30), Color(1.0, 0.55, 0.20, 0.30),
]
const TOGGLE_KEY: Key = KEY_R
const STEPS: Array[Vector2i] = [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 0)]
## Width of the line where a room ends (world pixels).
const EDGE: int = 2

signal toggled(shown: bool)

var _world_sim: Node = null
var _version: int = -1
var _font: Font = null
var _labels: Node2D = null
## Room id -> {id, role, overridden, tiles, beds, roofed, dug, doors, centre}.
var _rooms: Dictionary = {}
## Tile -> room id.
var _tile_room: Dictionary = {}
var _zoom: float = 1.0


func _ready() -> void:
	z_as_relative = false
	z_index = -5
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	visible = false
	_font = load(FONT_PATH) as Font
	_labels = Node2D.new()
	_labels.name = "Labels"
	_labels.z_as_relative = false
	_labels.z_index = 6
	_labels.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_labels.draw.connect(_draw_labels)
	add_child(_labels)
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	_version = -1
	_rooms.clear()
	_tile_room.clear()


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	var zoom: float = get_viewport().get_canvas_transform().x.x
	if visible and not is_equal_approx(zoom, _zoom):
		_zoom = zoom
		_labels.queue_redraw()
	var versions: Dictionary = SimReads.read(_world_sim, "get_colony_versions")
	var version: int = int(versions.get("rooms_version", 0))
	if version == _version:
		return
	_version = version
	_index(SimReads.read(_world_sim, "get_rooms"))
	queue_redraw()
	_labels.queue_redraw()


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and (event as InputEventKey).keycode == TOGGLE_KEY:
		toggle()
		get_viewport().set_input_as_handled()


## Show or hide the overlay.
func toggle() -> void:
	set_shown(not visible)


## Show (true) or hide the overlay.
func set_shown(on: bool) -> void:
	if on == visible:
		return
	visible = on
	toggled.emit(on)


## Whether the overlay is shown.
func is_shown() -> bool:
	return visible


## The room id of `tile`, or -1.
func room_at(tile: Vector2i) -> int:
	return int(_tile_room.get(tile, -1))


## Room `id`'s row ({id, role, overridden, tiles, beds, roofed, dug, doors, centre}), or {} when unknown.
func room(id: int) -> Dictionary:
	return _rooms.get(id, {})


## Number of rooms known (for tests and tools).
func room_count() -> int:
	return _rooms.size()


func _index(snap: Dictionary) -> void:
	_rooms.clear()
	_tile_room.clear()
	var ids: PackedInt32Array = snap.get("ids", PackedInt32Array())
	var roles: PackedInt32Array = snap.get("roles", PackedInt32Array())
	var overridden: PackedByteArray = snap.get("overridden", PackedByteArray())
	var tiles: PackedInt32Array = snap.get("tiles", PackedInt32Array())
	var beds: PackedInt32Array = snap.get("beds", PackedInt32Array())
	var roofed: PackedByteArray = snap.get("roofed", PackedByteArray())
	var dug: PackedByteArray = snap.get("dug", PackedByteArray())
	var doors: PackedInt32Array = snap.get("doors", PackedInt32Array())
	var cxs: PackedInt32Array = snap.get("centre_xs", PackedInt32Array())
	var cys: PackedInt32Array = snap.get("centre_ys", PackedInt32Array())
	for i: int in mini(ids.size(), mini(roles.size(), tiles.size())):
		_rooms[ids[i]] = {
			"id": ids[i],
			"role": clampi(roles[i], 0, ROLE_COLORS.size() - 1),
			"overridden": i < overridden.size() and overridden[i] != 0,
			"tiles": tiles[i],
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


func _draw() -> void:
	var tile_size: Vector2 = Vector2(View.TILE, View.TILE)
	for cell: Vector2i in _tile_room.keys():
		var id: int = int(_tile_room[cell])
		var info: Dictionary = _rooms.get(id, {})
		var colour: Color = ROLE_COLORS[int(info.get("role", 0))]
		var origin: Vector2 = View.tile_to_world(cell)
		draw_rect(Rect2(origin, tile_size), colour)
		var edge: Color = Color(colour, 0.85)
		for bit: int in STEPS.size():
			if int(_tile_room.get(cell + STEPS[bit], -1)) == id:
				continue
			match bit:
				0: draw_rect(Rect2(origin, Vector2(View.TILE, EDGE)), edge)
				1: draw_rect(Rect2(origin + Vector2(View.TILE - EDGE, 0), Vector2(EDGE, View.TILE)), edge)
				2: draw_rect(Rect2(origin + Vector2(0, View.TILE - EDGE), Vector2(View.TILE, EDGE)), edge)
				_: draw_rect(Rect2(origin, Vector2(EDGE, View.TILE)), edge)


func _draw_labels() -> void:
	if _font == null:
		return
	var locale: Node = get_node_or_null("/root/Locale")
	for id: int in _rooms.keys():
		var info: Dictionary = _rooms[id]
		var centre: Vector2i = info["centre"]
		if centre.x < 0:
			continue
		var text: String = label_text(info, locale)
		var px: int = int(ArtData.param("labels.font_px", 24))
		var inv: float = 1.0 / maxf(0.05, _zoom) if bool(ArtData.param("labels.follow_camera", true)) else 1.0
		var pad: float = floorf(px / 4.0)
		var size: Vector2 = _font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, px)
		_labels.draw_set_transform(View.tile_centre(centre).round(), 0.0, Vector2(inv, inv))
		var box: Rect2 = Rect2(-Vector2(size.x * 0.5 + pad, (px + pad) * 0.5), Vector2(size.x + pad * 2.0, px + pad))
		box.position = box.position.round()
		_labels.draw_rect(box, BACK)
		_labels.draw_rect(box, Color(ROLE_COLORS[int(info["role"])], 1.0), false, 2.0)
		_labels.draw_string(_font, box.position + Vector2(pad, px - 2.0), text, HORIZONTAL_ALIGNMENT_LEFT, -1, px, TEXT)
	_labels.draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
