# room_layer.gd (ColonyView/OverlayLayer/Rooms, made in code by overlay_layer.gd): the M3 room overlay (goal 9.3), drawn
#   in the 2D view. Main/RoomIndex (room_index.gd) owns the rooms, the shown flag and the R key; this node follows its
#   `toggled` and draws: every room tile tinted by its role (침실 blue, 창고 yellow,
#   작업장 teal, 홀 orange, 미정 grey; translucent, z −5: above the ground chunks and floors, below the y-sorted world
#   layer with its walls, things and people) with a 2-px line where the room ends (64-px tiles, M-ART2), and at each
#   room's centre tile a label on a dark backing box (child "Labels", z 6, above everything; the text keeps
#   `labels.font_px` screen pixels at every zoom and is redrawn when the zoom changes) "침실 · 35칸 · 침대 4" (`UI_ROOM_LABEL_FMT`, without beds `UI_ROOM_LABEL_NOBED_FMT`, "(역할 고정)" when
#   the player set the role). `room_at` / `room` / `is_shown` answer from RoomIndex (the 2D overlay hover and tools).
# Redrawn only when `rooms_version` moves (get_colony_versions is read every frame through SimReads).
# Bridge methods: via SimReads get_colony_versions.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const RoomIndex = preload("res://scripts/world/room_index.gd")
const FONT_PATH: String = "res://assets/fonts/galmuri/Galmuri11.ttf"
const BACK: Color = Color(0.094, 0.082, 0.075, 0.92)
const TEXT: Color = Color(0.96, 0.92, 0.84)
const ROLE_COLORS: Array[Color] = RoomIndex.ROLE_COLORS
const STEPS: Array[Vector2i] = [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 0)]
## Width of the line where a room ends (world pixels).
const EDGE: int = 2

var _world_sim: Node = null
var _index_node: Node = null
var _version: int = -1
var _font: Font = null
var _labels: Node2D = null
## RoomIndex's room id -> row and tile -> room id (shared dictionaries).
var _rooms: Dictionary = {}
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
	_index_node = get_node_or_null("/root/Main/RoomIndex")
	if _index_node != null:
		_index_node.connect("toggled", func(on: bool) -> void: visible = on)
		_rooms = _index_node.call("rooms")
		_tile_room = _index_node.call("tiles")


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	var zoom: float = get_viewport().get_canvas_transform().x.x
	if visible and not is_equal_approx(zoom, _zoom):
		_zoom = zoom
		_labels.queue_redraw()
	var versions: Dictionary = SimReads.read(_world_sim, "get_colony_versions")
	var version: int = int(versions.get("rooms_version", 0))
	if version == _version or _index_node == null:
		return
	_version = version
	_index_node.call("room_count")
	queue_redraw()
	_labels.queue_redraw()


## Show or hide the overlay (RoomIndex).
func toggle() -> void:
	if _index_node != null:
		_index_node.call("toggle")


func set_shown(on: bool) -> void:
	if _index_node != null:
		_index_node.call("set_shown", on)


func is_shown() -> bool:
	return _index_node != null and bool(_index_node.call("is_shown"))


## The room id of `tile`, or -1.
func room_at(tile: Vector2i) -> int:
	return int(_index_node.call("room_at", tile)) if _index_node != null else -1


## Room `id`'s row, or {} when unknown.
func room(id: int) -> Dictionary:
	return _index_node.call("room", id) if _index_node != null else {}


## Number of rooms known (for tests and tools).
func room_count() -> int:
	return int(_index_node.call("room_count")) if _index_node != null else 0


## The overlay label of room row `info` (RoomIndex.label_text; kept for tools).
static func label_text(info: Dictionary, locale: Node) -> String:
	return RoomIndex.label_text(info, locale)


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
		var text: String = RoomIndex.label_text(info, locale)
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
