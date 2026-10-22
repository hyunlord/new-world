# building_layer.gd (ColonyView/BuildingLayer in scenes/main.tscn): the hearth, the V7 shelters (until M3
#   replaces them), finished huts and (M2) finished workshops of the colony view.
# Huts are 48×48 around their centre tile (log walls, bedrolls, hearth stone, no roof); shelters are 16×20 lean-tos
#   drawn 4 px above their tile; the hearth's fire alternates two frames every FIRE_SECONDS (frame delta, not ticks).
# M2 workshops (get_workshops: knapping spot 1×1, wood bench and drying rack 2×1) are drawn at their footprint's
#   top-left with the sheet's temporary sprites, with a faint frame on the work tile south of them. A building
#   snapshot row inside a workshop footprint is skipped: the snapshot reports finished workshops as kind 0 (hearth),
#   because they sit in the building registry without a `building_kinds` entry.
# M3 built space (get_space, rebuilt when its `version` moves): walls on this node (z 3, with the buildings, so
#   people at z 5 are seen passing through doors), each log/stone wall picking `<log|stone>_wall_<mask>` from its
#   wall-or-door neighbours (N=1, E=2, S=4, W=8), a door `door_h` in an east–west wall else `door_v`; the child
#   node "Floors" (z 1, with the ground overlay; drawn before OverlayLayer's ground so stacks lie on top) draws dug
#   hill tiles and dug-rock floors as `floor_rock`, packed floors as `floor_packed` and bedrolls.
# Redrawn when the building snapshot or the workshop list changes, the space version moves, or the fire frame flips.
# Bridge methods: via SimReads get_building_snapshot, get_workshops, get_space; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
# BuildingKind codes (sim-core building_kind.rs): 0 hearth, 1 shelter, 2 stockpile marker, 3 hut.
const NAMES: Dictionary = {0: "hearth_0", 1: "shelter", 3: "hut"}
const FIRE_SECONDS: float = 0.4
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const WORK_TILE: Color = Color(0.9, 0.8, 0.55, 0.35)
## Wall sprite prefixes by `get_space` wall kind (0 log, 1 stone; 2 is a door).
const WALL_PREFIX: Array[String] = ["log_wall_", "stone_wall_"]
const DOOR_KIND: int = 2
## Neighbour offsets and their mask bits (N=1, E=2, S=4, W=8; as the sheet's `bank_*` and wall tiles).
const MASK_STEPS: Array[Vector2i] = [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 0)]

var _world_sim: Node = null
var _key: int = -1
var _rows: Array[Array] = []
## [kind code, footprint top-left, work tile] per finished workshop.
var _shops: Array[Array] = []
var _fire_clock: float = 0.0
var _fire_frame: int = 0
var _has_hearth: bool = false
var _space_version: int = -1
## Tile -> wall kind (0 log, 1 stone, 2 door) of the built space.
var _walls: Dictionary = {}
## [sprite name, tile] per floor-level piece (dug rock, floors, bedrolls), in draw order.
var _floor_pieces: Array[Array] = []
var _floors: Node2D = null


func _ready() -> void:
	z_index = 3
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_floors = Node2D.new()
	_floors.name = "Floors"
	_floors.z_index = -2
	_floors.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_floors.draw.connect(_draw_floors)
	add_child(_floors)
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	_key = -1
	_space_version = -1


func _process(delta: float) -> void:
	if _world_sim == null:
		return
	if _has_hearth:
		_fire_clock += delta
		if _fire_clock >= FIRE_SECONDS:
			_fire_clock = fmod(_fire_clock, FIRE_SECONDS)
			_fire_frame = 1 - _fire_frame
			queue_redraw()
	var snap: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var kinds: PackedByteArray = snap.get("kinds", PackedByteArray())
	var shops: Dictionary = SimReads.read(_world_sim, "get_workshops")
	var shop_kinds: PackedInt32Array = shops.get("kinds", PackedInt32Array())
	var sxs: PackedInt32Array = shops.get("xs", PackedInt32Array())
	var sys: PackedInt32Array = shops.get("ys", PackedInt32Array())
	var wxs: PackedInt32Array = shops.get("work_xs", PackedInt32Array())
	var wys: PackedInt32Array = shops.get("work_ys", PackedInt32Array())
	_refresh_space()
	var key: int = hash([xs, ys, kinds, shop_kinds, sxs, sys])
	if key == _key:
		return
	_key = key
	_shops.clear()
	var footprints: Array[Rect2i] = []
	var sws: PackedInt32Array = shops.get("ws", PackedInt32Array())
	var shs: PackedInt32Array = shops.get("hs", PackedInt32Array())
	for i: int in mini(mini(shop_kinds.size(), sxs.size()), mini(sys.size(), mini(wxs.size(), wys.size()))):
		if shop_kinds[i] > 0 and shop_kinds[i] < EconomyText.BUILD_KINDS.size():
			_shops.append([shop_kinds[i], Vector2i(sxs[i], sys[i]), Vector2i(wxs[i], wys[i])])
		if i < sws.size() and i < shs.size():
			footprints.append(Rect2i(sxs[i], sys[i], sws[i], shs[i]))
	_rows.clear()
	_has_hearth = false
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		# A finished workshop is also in the building registry without a kind, so the building snapshot reports it
		# as kind 0 (hearth); the workshop list draws it instead.
		if _inside_any(Vector2i(xs[i], ys[i]), footprints):
			continue
		if NAMES.has(int(kinds[i])):
			_rows.append([int(kinds[i]), Vector2i(xs[i], ys[i])])
			_has_hearth = _has_hearth or kinds[i] == 0
	queue_redraw()


## M3: re-read get_space when its version moved; index walls and the floor-level pieces.
func _refresh_space() -> void:
	var space: Dictionary = SimReads.read(_world_sim, "get_space")
	var version: int = int(space.get("version", 0))
	if version == _space_version:
		return
	_space_version = version
	_walls.clear()
	var wxs: PackedInt32Array = space.get("wall_xs", PackedInt32Array())
	var wys: PackedInt32Array = space.get("wall_ys", PackedInt32Array())
	var wkinds: PackedInt32Array = space.get("wall_kinds", PackedInt32Array())
	for i: int in mini(wxs.size(), mini(wys.size(), wkinds.size())):
		_walls[Vector2i(wxs[i], wys[i])] = clampi(wkinds[i], 0, DOOR_KIND)
	_floor_pieces.clear()
	var dxs: PackedInt32Array = space.get("dug_xs", PackedInt32Array())
	var dys: PackedInt32Array = space.get("dug_ys", PackedInt32Array())
	for i: int in mini(dxs.size(), dys.size()):
		_floor_pieces.append(["floor_rock", Vector2i(dxs[i], dys[i])])
	var fxs: PackedInt32Array = space.get("floor_xs", PackedInt32Array())
	var fys: PackedInt32Array = space.get("floor_ys", PackedInt32Array())
	var fkinds: PackedInt32Array = space.get("floor_kinds", PackedInt32Array())
	for i: int in mini(fxs.size(), fys.size()):
		var rock: bool = i < fkinds.size() and fkinds[i] == 1
		_floor_pieces.append(["floor_rock" if rock else "floor_packed", Vector2i(fxs[i], fys[i])])
	var bxs: PackedInt32Array = space.get("bed_xs", PackedInt32Array())
	var bys: PackedInt32Array = space.get("bed_ys", PackedInt32Array())
	for i: int in mini(bxs.size(), bys.size()):
		_floor_pieces.append(["bedroll", Vector2i(bxs[i], bys[i])])
	_floors.queue_redraw()
	queue_redraw()


## The wall-or-door neighbour mask of `tile` in `walls` (tile -> kind): N=1, E=2, S=4, W=8.
static func wall_mask(walls: Dictionary, tile: Vector2i) -> int:
	var mask: int = 0
	for bit: int in MASK_STEPS.size():
		if walls.has(tile + MASK_STEPS[bit]):
			mask |= 1 << bit
	return mask


## The sheet sprite of the wall piece of `kind` (0 log, 1 stone, 2 door) on `tile` among `walls`.
static func wall_sprite(walls: Dictionary, tile: Vector2i, kind: int) -> String:
	var mask: int = wall_mask(walls, tile)
	if kind == DOOR_KIND:
		# A door hangs in the wall it joins: east–west when a wall is east or west of it (or none north/south).
		return "door_h" if (mask & 10) != 0 or (mask & 5) == 0 else "door_v"
	return WALL_PREFIX[clampi(kind, 0, WALL_PREFIX.size() - 1)] + str(mask)


## Number of wall and door tiles drawn (for tests and tools).
func wall_count() -> int:
	return _walls.size()


## Number of floor-level pieces drawn (dug rock, floors, bedrolls; for tests and tools).
func floor_piece_count() -> int:
	return _floor_pieces.size()


func _draw_floors() -> void:
	var texture: Texture2D = View.texture()
	if texture == null:
		return
	var tile_size: Vector2 = Vector2(View.TILE, View.TILE)
	for piece: Array in _floor_pieces:
		_floors.draw_texture_rect_region(texture, Rect2(View.tile_to_world(piece[1]), tile_size), View.rect(str(piece[0])))


## Number of huts drawn (for tests and tools).
func hut_count() -> int:
	var n: int = 0
	for row: Array in _rows:
		if int(row[0]) == 3:
			n += 1
	return n


static func _inside_any(tile: Vector2i, rects: Array[Rect2i]) -> bool:
	for rect: Rect2i in rects:
		if rect.has_point(tile):
			return true
	return false


## Number of hearths drawn (for tests and tools).
func hearth_count() -> int:
	var n: int = 0
	for row: Array in _rows:
		if int(row[0]) == 0:
			n += 1
	return n


## Number of workshops drawn (for tests and tools).
func workshop_count() -> int:
	return _shops.size()


## The fire frame shown now (0 or 1; for tests and tools).
func fire_frame() -> int:
	return _fire_frame


func _draw() -> void:
	var texture: Texture2D = View.texture()
	if texture == null:
		return
	var tile_size: Vector2 = Vector2(View.TILE, View.TILE)
	for cell: Vector2i in _walls.keys():
		var sprite_name: String = wall_sprite(_walls, cell, int(_walls[cell]))
		draw_texture_rect_region(texture, Rect2(View.tile_to_world(cell), tile_size), View.rect(sprite_name))
	for shop: Array in _shops:
		var work: Rect2 = Rect2(View.tile_to_world(shop[2]), Vector2(View.TILE, View.TILE))
		draw_rect(work.grow(-1), WORK_TILE, false, 1.0)
		var sprite: Rect2 = View.rect(str(EconomyText.BUILD_KINDS[int(shop[0])]["sprite"]))
		draw_texture_rect_region(texture, Rect2(View.tile_to_world(shop[1]), sprite.size), sprite)
	for row: Array in _rows:
		var kind: int = row[0]
		var tile: Vector2i = row[1]
		var name: String = "hearth_%d" % _fire_frame if kind == 0 else str(NAMES[kind])
		var src: Rect2 = View.rect(name)
		var origin: Vector2 = View.tile_to_world(tile)
		if kind == 3:
			origin -= Vector2(View.TILE, View.TILE)
		elif kind == 1:
			origin.y -= src.size.y - View.TILE
		draw_texture_rect_region(texture, Rect2(origin, src.size), src)
