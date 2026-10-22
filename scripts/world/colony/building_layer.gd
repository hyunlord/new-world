# building_layer.gd (ColonyView/BuildingLayer in scenes/main.tscn): the hearth, the V7 shelters (until M3
#   replaces them), finished huts and (M2) finished workshops of the colony view.
# Huts are 48×48 around their centre tile (log walls, bedrolls, hearth stone, no roof); shelters are 16×20 lean-tos
#   drawn 4 px above their tile; the hearth's fire alternates two frames every FIRE_SECONDS (frame delta, not ticks).
# M2 workshops (get_workshops: knapping spot 1×1, wood bench and drying rack 2×1) are drawn at their footprint's
#   top-left with the sheet's temporary sprites, with a faint frame on the work tile south of them. A building
#   snapshot row inside a workshop footprint is skipped: the snapshot reports finished workshops as kind 0 (hearth),
#   because they sit in the building registry without a `building_kinds` entry.
# Redrawn when the building snapshot or the workshop list changes, or the fire frame flips.
# Bridge methods: via SimReads get_building_snapshot, get_workshops; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
# BuildingKind codes (sim-core building_kind.rs): 0 hearth, 1 shelter, 2 stockpile marker, 3 hut.
const NAMES: Dictionary = {0: "hearth_0", 1: "shelter", 3: "hut"}
const FIRE_SECONDS: float = 0.4
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const WORK_TILE: Color = Color(0.9, 0.8, 0.55, 0.35)

var _world_sim: Node = null
var _key: int = -1
var _rows: Array[Array] = []
## [kind code, footprint top-left, work tile] per finished workshop.
var _shops: Array[Array] = []
var _fire_clock: float = 0.0
var _fire_frame: int = 0
var _has_hearth: bool = false


func _ready() -> void:
	z_index = 3
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", func() -> void: _key = -1)


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
