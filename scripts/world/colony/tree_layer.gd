# tree_layer.gd (ColonyView/TreeLayer in scenes/main.tscn): the canopy of the M1 colony, stumps of felled trees,
#   berry bushes and deadwood.
# The scene node is a TileMapLayer, but the canopy does not fit a tile map (24×24 crowns overlapping their
#   neighbours), so its cells stay empty and the trees are drawn by child Node2D chunks of CHUNK×CHUNK tiles, created
#   in row order so a southern chunk draws over a northern one. Inside a chunk: every tree's shadow,
#   then the crowns sorted by (y, x), so southern crowns overlap northern ones. Per tree, a keyed hash of (x, y) on
#   the world seed picks the species (oak 50 %, birch 30 %, pine 20 %), the size (3) and a ±2 px jitter. At 1× zoom
#   every chunk redraws once without the shadows (half the quads; a 16×16 impostor turned the forest back into
#   dots — M1-ART DECISIONS D9).
# The tree set is re-read only when `trees_version` changes; a felled tree (in the old set, not in the new one)
#   leaves a stump for one game day (STUMP_TICKS), kept locally from these version diffs.
# Stumps, bushes (full / bare by their amount) and deadwood are drawn by the child node "Nature" (z 1, under the
#   log piles of the overlay) when the resource snapshot or the stump set changes. Canopy stays below the people
#   (z 2 < 5); `under_canopy` tells the agent layer when a person stands where a crown can cover it.
# Bridge methods: via SimReads get_colony_versions, get_trees, get_resource_snapshot, get_time_snapshot,
#   get_world_info; signal world_reset.
extends TileMapLayer

const View = preload("res://scripts/world/colony/view.gd")
const CHUNK: int = 16
const SPECIES: Array[String] = ["oak", "oak", "oak", "oak", "oak", "birch", "birch", "birch", "pine", "pine"]
const STUMP_TICKS: int = 1440
const LOW_ZOOM: float = 1.5

var _world_sim: Node = null
var _version: int = -1
var _seed: int = 0
var _trees: Dictionary = {}
var _stumps: Dictionary = {}
var _chunks: Dictionary = {}
var _chunk_trees: Dictionary = {}
var _low: bool = false
var _texture: Texture2D = null
var _shadow_rect: Rect2 = Rect2()
var _stump_rect: Rect2 = Rect2()
var _crowns: Dictionary = {}
var _nature: Node2D = null
var _nature_key: int = -1
var _bushes: Array[Array] = []
var _stump_check: int = 0


func _ready() -> void:
	z_index = 2
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_texture = View.texture()
	_shadow_rect = View.rect("tree_shadow")
	_stump_rect = View.rect("stump")
	for species: String in ["oak", "birch", "pine"]:
		for size: int in 3:
			_crowns["%s_%d" % [species, size]] = View.rect("tree_%s_%d" % [species, size])
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
	_trees.clear()
	_stumps.clear()
	for chunk: Node2D in _chunks.values():
		chunk.queue_free()
	_chunks.clear()
	_chunk_trees.clear()


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	var low: bool = get_viewport().get_canvas_transform().x.x < LOW_ZOOM
	if low != _low:
		_low = low
		for chunk: Node2D in _chunks.values():
			chunk.queue_redraw()
	var versions: Dictionary = SimReads.read(_world_sim, "get_colony_versions")
	var version: int = int(versions.get("trees_version", -1))
	if version != _version:
		_rebuild_trees(_version >= 0)
		_version = version
	_stump_check -= 1
	if not _stumps.is_empty() and _stump_check <= 0:
		_stump_check = 30
		_expire_stumps()
	var resources: Dictionary = SimReads.read(_world_sim, "get_resource_snapshot")
	var amounts: PackedInt32Array = resources.get("amounts", PackedInt32Array())
	var key: int = hash(amounts)
	if key != _nature_key:
		_nature_key = key
		_collect_nature(resources)
		_nature.queue_redraw()


## Whether a crown can cover tile `t`: a tree on it or on one of its 8 neighbours.
func under_canopy(t: Vector2i) -> bool:
	for dy: int in range(-1, 2):
		for dx: int in range(-1, 2):
			if _trees.has(t + Vector2i(dx, dy)):
				return true
	return false


## Number of standing trees drawn (for tests and tools).
func tree_count() -> int:
	return _trees.size()


## The stump tiles currently drawn (for tests and tools).
func stump_tiles() -> Array:
	return _stumps.keys()


func _now() -> int:
	return int((SimReads.read(_world_sim, "get_time_snapshot") as Dictionary).get("tick", 0))


func _rebuild_trees(diff: bool) -> void:
	if _chunks.is_empty():
		_seed = int((SimReads.read(_world_sim, "get_world_info") as Dictionary).get("seed", 0))
	var trees: Dictionary = SimReads.read(_world_sim, "get_trees")
	var xs: PackedInt32Array = trees.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = trees.get("ys", PackedInt32Array())
	var now: Dictionary = {}
	for i: int in mini(xs.size(), ys.size()):
		now[Vector2i(xs[i], ys[i])] = true
	var dirty: Dictionary = {}
	var tick: int = _now()
	var stumps_before: int = _stumps.size()
	for cell: Vector2i in _trees.keys():
		if not now.has(cell):
			dirty[_chunk_of(cell)] = true
			if diff:
				_stumps[cell] = tick
	for cell: Vector2i in now.keys():
		if not _trees.has(cell):
			dirty[_chunk_of(cell)] = true
			_stumps.erase(cell)
	if _stumps.size() != stumps_before:
		_nature.queue_redraw()
	_trees = now
	if dirty.is_empty():
		return
	for key: Vector2i in dirty.keys():
		_chunk_trees[key] = []
	for cell: Vector2i in _trees.keys():
		var key: Vector2i = _chunk_of(cell)
		if dirty.has(key):
			(_chunk_trees[key] as Array).append(_tree_entry(cell))
	for key: Vector2i in dirty.keys():
		(_chunk_trees[key] as Array).sort_custom(_entry_before)
		_ensure_chunk(key).queue_redraw()


func _expire_stumps() -> void:
	var tick: int = _now()
	for cell: Vector2i in _stumps.keys():
		if tick - int(_stumps[cell]) >= STUMP_TICKS or tick < int(_stumps[cell]):
			_stumps.erase(cell)
			_nature.queue_redraw()


func _chunk_of(cell: Vector2i) -> Vector2i:
	return Vector2i(floori(float(cell.x) / CHUNK), floori(float(cell.y) / CHUNK))


## [y, x, crown key, top-left Vector2]: the tree's species, size and jittered origin.
func _tree_entry(cell: Vector2i) -> Array:
	var h: int = View.key_hash(_seed, cell.x, cell.y, 7)
	var species: String = SPECIES[h % SPECIES.size()]
	@warning_ignore("integer_division")
	var size: int = (h / 10) % 3
	@warning_ignore("integer_division")
	var jx: int = (h / 30) % 5 - 2
	@warning_ignore("integer_division")
	var jy: int = (h / 150) % 5 - 2
	return [cell.y, cell.x, species, size, View.tile_to_world(cell) + Vector2(jx, jy)]


func _entry_before(a: Array, b: Array) -> bool:
	return int(a[0]) < int(b[0]) or (int(a[0]) == int(b[0]) and int(a[1]) < int(b[1]))


## The chunk node for `key`; chunks are kept as children in (row, column) order.
func _ensure_chunk(key: Vector2i) -> Node2D:
	var chunk: Node2D = _chunks.get(key)
	if chunk != null:
		return chunk
	chunk = Node2D.new()
	chunk.name = "Chunk_%d_%d" % [key.x, key.y]
	chunk.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	chunk.draw.connect(_draw_chunk.bind(chunk, key))
	add_child(chunk)
	_chunks[key] = chunk
	var index: int = 1  # child 0 is Nature
	for other: Vector2i in _chunks.keys():
		if other.y < key.y or (other.y == key.y and other.x < key.x):
			index += 1
	move_child(chunk, index)
	if not _chunk_trees.has(key):
		_chunk_trees[key] = []
	return chunk


func _draw_chunk(chunk: Node2D, key: Vector2i) -> void:
	if _texture == null:
		return
	var entries: Array = _chunk_trees.get(key, [])
	if not _low:
		for e: Array in entries:
			var origin: Vector2 = e[4]
			chunk.draw_texture_rect_region(_texture, Rect2(origin + Vector2(0, 12), _shadow_rect.size), _shadow_rect)
	for e: Array in entries:
		var src: Rect2 = _crowns["%s_%d" % [e[2], e[3]]]
		chunk.draw_texture_rect_region(_texture, Rect2(e[4] as Vector2 - Vector2(4, 4), src.size), src)


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
	if _texture == null:
		return
	var size: Vector2 = Vector2(View.TILE, View.TILE)
	for cell: Vector2i in _stumps.keys():
		_nature.draw_texture_rect_region(_texture, Rect2(View.tile_to_world(cell), size), _stump_rect)
	for entry: Array in _bushes:
		_nature.draw_texture_rect_region(_texture, Rect2(View.tile_to_world(entry[0] as Vector2i), size), View.rect(str(entry[1])))
