# terrain_layer.gd (ColonyView/TerrainLayer in scenes/main.tscn): the ground of the M1 colony view.
# A TileMapLayer built once per world (rebuilt only on `world_reset`, and once more when the first tick registers
#   the hearth — checked each frame by one cached read, never by a full rebuild) from the terrain snapshot and the trees standing at that moment:
#   - grass in two families clustered by value noise on the world seed (`View.cluster_noise`), never per-tile
#     random colour;
#   - forest floor under every tree and its 8 neighbours;
#   - worn earth: the camp clearing eroded by the same noise, with grass-dither edge tiles turned toward the grass
#     by the tile transform flags;
#   - water: a 4-bit land-neighbour mask picks a bank tile (sand lip on each land side), a land diagonal an inner
#     corner, then one ring of shallow water, then deep water;
#   - hills; rock with a lit top edge under open ground and a cliff face above it.
#   The calmest layer: low contrast, no animation.
# Bridge methods: via SimReads get_terrain_snapshot, get_building_snapshot, get_trees, get_world_info; signal
#   world_reset.
extends TileMapLayer

const View = preload("res://scripts/world/colony/view.gd")
# TerrainType codes (sim-core material/terrain.rs): 0 Plain, 1 Forest, 2 Mountain, 3 Hill, 4 River.
const FOREST: int = 1
const MOUNTAIN: int = 2
const HILL: int = 3
const RIVER: int = 4
# The camp clearing: always worn within CORE tiles of the hearth (Chebyshev), eroded by noise out to REACH.
const CORE: int = 4
const REACH: int = 8
const DARK_GRASS: float = 0.57
# Within this distance of the threshold the family is picked by the tile hash, so cluster edges are ragged.
const RAGGED: float = 0.035
const N4: Array[Vector2i] = [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 0)]
const DIAG: Array[Vector2i] = [Vector2i(1, -1), Vector2i(-1, -1), Vector2i(1, 1), Vector2i(-1, 1)]
const DIAG_NAMES: Array[String] = ["bank_in_ne", "bank_in_nw", "bank_in_se", "bank_in_sw"]
# The dither edge tile has its grass on the north side; these turn it to face N, E, S, W.
const EDGE_TRANSFORMS: Array[int] = [
	0,
	TileSetAtlasSource.TRANSFORM_TRANSPOSE | TileSetAtlasSource.TRANSFORM_FLIP_H,
	TileSetAtlasSource.TRANSFORM_FLIP_V,
	TileSetAtlasSource.TRANSFORM_TRANSPOSE,
]

var _world_sim: Node = null
var _coords: Dictionary = {}
var _pending: bool = true
var _drawn: bool = false


func _ready() -> void:
	z_index = 0
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	var names: Array[String] = ["mountain_cliff", "stock_floor", "dirt_edge_0", "dirt_edge_1"]
	for i: int in 3:
		names.append_array(["grass_%d" % i, "grass_dark_%d" % i, "hill_%d" % i, "mountain_%d" % i, "mountain_top_%d" % i])
	for i: int in 4:
		names.append_array(["forest_floor_%d" % i, "dirt_%d" % i, "water_%d" % i, "shallow_%d" % i])
	for mask: int in range(1, 16):
		names.append("bank_%d" % mask)
	names.append_array(DIAG_NAMES)
	var built: Dictionary = View.tileset(names)
	tile_set = built["tileset"]
	_coords = built["coords"]
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	_pending = true
	_drawn = false


func _process(_delta: float) -> void:
	# Draw once at once; while the hearth is unknown, rebuild only when it appears (not every frame).
	if _pending and _world_sim != null and (not _drawn or _hearth_known()):
		refresh()


func _hearth_known() -> bool:
	var kinds: PackedByteArray = (SimReads.read(_world_sim, "get_building_snapshot") as Dictionary).get("kinds", PackedByteArray())
	return kinds.has(0)


## Rebuild every cell from the terrain snapshot.
func refresh() -> void:
	var snap: Dictionary = SimReads.read(_world_sim, "get_terrain_snapshot")
	var w: int = int(snap.get("width", 0))
	var h: int = int(snap.get("height", 0))
	var terrain: PackedByteArray = snap.get("terrain", PackedByteArray())
	if w <= 0 or terrain.size() != w * h:
		return
	var hearth: Vector2i = Vector2i(-1000, -1000)
	var buildings: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var kinds: PackedByteArray = buildings.get("kinds", PackedByteArray())
	var bxs: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var bys: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(bxs.size(), bys.size())):
		if kinds[i] == 0:
			hearth = Vector2i(bxs[i], bys[i])
			break
	# Buildings register on the first tick: until then draw the ground and retry for the clearing.
	_pending = hearth.x < 0
	_drawn = true
	var seed: int = int((SimReads.read(_world_sim, "get_world_info") as Dictionary).get("seed", 0))
	var floor_mask: PackedByteArray = _forest_floor(w, h, terrain)
	var worn: PackedByteArray = PackedByteArray()
	worn.resize(w * h)
	for y: int in h:
		for x: int in w:
			worn[y * w + x] = 1 if _is_worn(terrain[y * w + x], floor_mask[y * w + x], x, y, hearth, seed) else 0
	clear()
	for y: int in h:
		for x: int in w:
			var i: int = y * w + x
			var v: int = View.tile_hash(x, y)
			var name: String
			var alt: int = 0
			match terrain[i]:
				MOUNTAIN:
					if y > 0 and terrain[i - w] != MOUNTAIN:
						name = "mountain_top_%d" % (v % 3)
					elif y + 1 < h and terrain[i + w] != MOUNTAIN:
						name = "mountain_cliff"
					else:
						name = "mountain_%d" % (v % 3)
				HILL:
					name = "hill_%d" % (v % 3)
				RIVER:
					name = _water(terrain, w, h, x, y, v)
				_:
					if worn[i] == 1:
						var side: int = _grass_side(worn, terrain, w, h, x, y)
						if side >= 0:
							name = "dirt_edge_%d" % (v % 2)
							alt = EDGE_TRANSFORMS[side]
						else:
							name = "dirt_%d" % (v % 4)
					elif floor_mask[i] == 1:
						name = "forest_floor_%d" % (v % 4)
					elif _dark_grass(View.cluster_noise(seed, x, y), v):
						name = "grass_dark_%d" % (v % 3)
					else:
						name = "grass_%d" % (v % 3)
			set_cell(Vector2i(x, y), 0, _coords.get(name, Vector2i.ZERO), alt)


## 1 where the forest floor goes: every tree tile and its 8 neighbours (the trees standing now; forest terrain
## when the tree read is empty).
func _forest_floor(w: int, h: int, terrain: PackedByteArray) -> PackedByteArray:
	var out: PackedByteArray = PackedByteArray()
	out.resize(w * h)
	var trees: Dictionary = SimReads.read(_world_sim, "get_trees")
	var xs: PackedInt32Array = trees.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = trees.get("ys", PackedInt32Array())
	var cells: Array[Vector2i] = []
	for i: int in mini(xs.size(), ys.size()):
		cells.append(Vector2i(xs[i], ys[i]))
	if cells.is_empty():
		for i: int in terrain.size():
			if terrain[i] == FOREST:
				cells.append(Vector2i(i % w, floori(float(i) / float(w))))
	for c: Vector2i in cells:
		for dy: int in range(-1, 2):
			for dx: int in range(-1, 2):
				var x: int = c.x + dx
				var y: int = c.y + dy
				if x >= 0 and y >= 0 and x < w and y < h and terrain[y * w + x] != RIVER and terrain[y * w + x] != MOUNTAIN:
					out[y * w + x] = 1
	return out


func _is_worn(code: int, under_trees: int, x: int, y: int, hearth: Vector2i, seed: int) -> bool:
	if code == RIVER or code == MOUNTAIN or code == HILL:
		return false
	var d: int = maxi(absi(x - hearth.x), absi(y - hearth.y))
	if d <= CORE:
		return true
	if d > REACH or under_trees == 1:
		return false
	var falloff: float = 1.0 - float(d - CORE) / float(REACH - CORE + 1)
	return View.cluster_noise(seed, x + 1000, y + 1000) + 0.7 * falloff > 0.8


func _dark_grass(noise: float, hash_value: int) -> bool:
	if absf(noise - DARK_GRASS) < RAGGED:
		@warning_ignore("integer_division")
		return (hash_value / 7) % 2 == 0
	return noise > DARK_GRASS


## The side (0 N, 1 E, 2 S, 3 W) of a worn tile that faces open grass (not water, rock or hill), or -1.
func _grass_side(worn: PackedByteArray, terrain: PackedByteArray, w: int, h: int, x: int, y: int) -> int:
	for side: int in 4:
		var nx: int = x + N4[side].x
		var ny: int = y + N4[side].y
		if nx < 0 or ny < 0 or nx >= w or ny >= h:
			continue
		var j: int = ny * w + nx
		if worn[j] == 0 and terrain[j] != RIVER and terrain[j] != MOUNTAIN and terrain[j] != HILL:
			return side
	return -1


func _water(terrain: PackedByteArray, w: int, h: int, x: int, y: int, v: int) -> String:
	var mask: int = 0
	for side: int in 4:
		if _land(terrain, w, h, x + N4[side].x, y + N4[side].y):
			mask |= 1 << side
	if mask > 0:
		return "bank_%d" % mask
	for k: int in 4:
		if _land(terrain, w, h, x + DIAG[k].x, y + DIAG[k].y):
			return DIAG_NAMES[k]
	for dy: int in range(-2, 3):
		for dx: int in range(-2, 3):
			if _land(terrain, w, h, x + dx, y + dy):
				return "shallow_%d" % (v % 4)
	return "water_%d" % (v % 4)


func _land(terrain: PackedByteArray, w: int, h: int, x: int, y: int) -> bool:
	return x >= 0 and y >= 0 and x < w and y < h and terrain[y * w + x] != RIVER
