# building_layer.gd (ColonyView/BuildingLayer in scenes/main.tscn): the hearth, the V7 shelters (until M3
#   replaces them) and finished huts of the M1 colony view.
# Huts are 48×48 around their centre tile (log walls, bedrolls, hearth stone, no roof); shelters are 16×20 lean-tos
#   drawn 4 px above their tile; the hearth's fire alternates two frames every FIRE_SECONDS (frame delta, not ticks).
# Redrawn when the building snapshot changes or the fire frame flips.
# Bridge methods: via SimReads get_building_snapshot; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
# BuildingKind codes (sim-core building_kind.rs): 0 hearth, 1 shelter, 2 stockpile marker, 3 hut.
const NAMES: Dictionary = {0: "hearth_0", 1: "shelter", 3: "hut"}
const FIRE_SECONDS: float = 0.4

var _world_sim: Node = null
var _key: int = -1
var _rows: Array[Array] = []
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
	var key: int = hash([xs, ys, kinds])
	if key == _key:
		return
	_key = key
	_rows.clear()
	_has_hearth = false
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
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


## The fire frame shown now (0 or 1; for tests and tools).
func fire_frame() -> int:
	return _fire_frame


func _draw() -> void:
	var texture: Texture2D = View.texture()
	if texture == null:
		return
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
