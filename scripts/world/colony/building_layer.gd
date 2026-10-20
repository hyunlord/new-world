# building_layer.gd (ColonyView/BuildingLayer in scenes/main.tscn): the hearth, the V7 shelters (until M3
#   replaces them) and finished huts (48×48 around their centre tile) of the M1 colony view.
# Redrawn when the building snapshot changes.
# Bridge methods: via SimReads get_building_snapshot; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
# BuildingKind codes (sim-core building_kind.rs): 0 hearth, 1 shelter, 2 stockpile marker, 3 hut.
const NAMES: Dictionary = {0: "hearth", 1: "shelter", 3: "hut"}

var _world_sim: Node = null
var _key: int = -1
var _rows: Array[Array] = []


func _ready() -> void:
	z_index = 3
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", func() -> void: _key = -1)


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	var snap: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var kinds: PackedByteArray = snap.get("kinds", PackedByteArray())
	var key: int = hash([xs, ys, kinds])
	if key == _key:
		return
	_key = key
	_rows.clear()
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if NAMES.has(int(kinds[i])):
			_rows.append([int(kinds[i]), Vector2i(xs[i], ys[i])])
	queue_redraw()


## Number of huts drawn (for tests and tools).
func hut_count() -> int:
	var n: int = 0
	for row: Array in _rows:
		if int(row[0]) == 3:
			n += 1
	return n


func _draw() -> void:
	var texture: Texture2D = View.texture()
	if texture == null:
		return
	for row: Array in _rows:
		var kind: int = row[0]
		var tile: Vector2i = row[1]
		var name: String = NAMES[kind]
		var src: Rect2 = View.rect(name)
		var origin: Vector2 = View.tile_to_world(tile)
		if kind == 3:
			origin -= Vector2(View.TILE, View.TILE)
		draw_texture_rect_region(texture, Rect2(origin, src.size), src)
