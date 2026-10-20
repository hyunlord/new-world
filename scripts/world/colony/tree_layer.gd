# tree_layer.gd (ColonyView/TreeLayer in scenes/main.tscn): the standing trees of the M1 colony, plus berry
#   bushes and deadwood.
# Trees: a TileMapLayer rebuilt only when `trees_version` changes (felled trees are erased by diff). Bushes
#   (full / bare by their amount) and deadwood are drawn by a child node redrawn when the resource snapshot
#   changes.
# Bridge methods: via SimReads get_colony_versions, get_trees, get_resource_snapshot; signal world_reset.
extends TileMapLayer

const View = preload("res://scripts/world/colony/view.gd")
const NAMES: Array[String] = ["tree_0", "tree_1", "tree_2"]

var _world_sim: Node = null
var _coords: Dictionary = {}
var _version: int = -1
var _cells: Dictionary = {}
var _nature: Node2D = null
var _nature_key: int = -1
var _bushes: Array[Array] = []


func _ready() -> void:
	z_index = 2
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	var built: Dictionary = View.tileset(NAMES)
	tile_set = built["tileset"]
	_coords = built["coords"]
	_nature = Node2D.new()
	_nature.name = "Nature"
	_nature.z_index = -1
	_nature.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_nature.draw.connect(_draw_nature)
	add_child(_nature)
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	_version = -1
	_nature_key = -1


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	var versions: Dictionary = SimReads.read(_world_sim, "get_colony_versions")
	var version: int = int(versions.get("trees_version", -1))
	if version != _version:
		_rebuild_trees()
		_version = version
	var resources: Dictionary = SimReads.read(_world_sim, "get_resource_snapshot")
	var amounts: PackedInt32Array = resources.get("amounts", PackedInt32Array())
	var key: int = hash(amounts)
	if key != _nature_key:
		_nature_key = key
		_collect_nature(resources)
		_nature.queue_redraw()


func _rebuild_trees() -> void:
	var trees: Dictionary = SimReads.read(_world_sim, "get_trees")
	var xs: PackedInt32Array = trees.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = trees.get("ys", PackedInt32Array())
	var now: Dictionary = {}
	for i: int in mini(xs.size(), ys.size()):
		now[Vector2i(xs[i], ys[i])] = true
	for cell: Vector2i in _cells.keys():
		if not now.has(cell):
			erase_cell(cell)
	for cell: Vector2i in now.keys():
		if not _cells.has(cell):
			var name: String = "tree_%d" % (View.tile_hash(cell.x, cell.y) % 3)
			set_cell(cell, 0, _coords.get(name, Vector2i.ZERO))
	_cells = now


func _collect_nature(resources: Dictionary) -> void:
	_bushes.clear()
	var xs: PackedInt32Array = resources.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = resources.get("ys", PackedInt32Array())
	var kinds: PackedInt32Array = resources.get("kinds", PackedInt32Array())
	var amounts: PackedInt32Array = resources.get("amounts", PackedInt32Array())
	for i: int in mini(mini(xs.size(), ys.size()), mini(kinds.size(), amounts.size())):
		# Resource kinds: 0 food (berry bush), 3 deadwood; water and sleep sources are not drawn here.
		if kinds[i] == 0:
			_bushes.append([Vector2i(xs[i], ys[i]), "bush_full" if amounts[i] > 0 else "bush_bare"])
		elif kinds[i] == 3 and amounts[i] > 0:
			_bushes.append([Vector2i(xs[i], ys[i]), "deadwood"])


func _draw_nature() -> void:
	var texture: Texture2D = View.texture()
	if texture == null:
		return
	for entry: Array in _bushes:
		var tile: Vector2i = entry[0]
		_nature.draw_texture_rect_region(texture, Rect2(View.tile_to_world(tile), Vector2(View.TILE, View.TILE)), View.rect(str(entry[1])))
