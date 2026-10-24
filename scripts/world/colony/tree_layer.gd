# tree_layer.gd (ColonyView/TreeLayer in scenes/main.tscn): the trees, stumps, berry bushes and deadwood of the
#   colony view at 64-px tiles (M-ART2 §3.5 "things").
# Every tree of get_trees is a standing Sprite2D in the y-sorted ColonyView/World (position = the tile's base point
#   (tile_left + 32, tile_top + 43) plus a ±2 px jitter, offset = −anchor, so trunks sort with people, walls and
#   structures by base y). The simulation has no species: a keyed hash of (x, y) on the world seed picks one of
#   bindings.json `trees.species` by weight and a second hash picks the large or small sprite (`large_permille`).
# Only the chunks (CHUNK × CHUNK tiles) in view plus a one-chunk margin hold sprites (a seed-0 map has ~10,000 trees
#   and ~600 bushes and deadwood; creating them all made 13,000 y-sorted nodes); a chunk that leaves the view by more
#   than one further chunk frees its sprites, so panning does not rebuild at every step.
# The tree set is re-read only when `trees_version` changes and synced tile by tile (sprites are pooled per tile); a
#   felled tree (in the old set, not in the new one) leaves the `trees.stump` sprite for `trees.stump_ticks`, kept
#   locally from these version diffs. Berry bushes (get_resource_snapshot kind 0: `resources` row structure, its
#   `depleted` sprite below `depleted_below_permille` of the source's max) and deadwood (kind 3: its `item` sprite)
#   stand the same way; they are re-synced when the resource amounts change.
# `under_canopy(t)` tells the agent layer when a crown can hide a person on tile t: a tree on t, or on the tile left of,
#   at or right of t's column up to two rows south of it (the tree sorts later and its crown rises ~3 tiles).
# Bridge methods: via SimReads get_colony_versions, get_trees, get_resource_snapshot, get_time_snapshot,
#   get_world_info; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
## Crown reach for `under_canopy`: rows south of the person a tree can stand and still cover them.
const CROWN_ROWS: int = 2
const SALT_SPECIES: int = 7
const SALT_SIZE: int = 8
const SALT_JITTER: int = 9
## Culling chunk size in tiles.
const CHUNK: int = 16

var _world_sim: Node = null
var _world: Node2D = null
var _version: int = -1
var _seed: int = 0
## Tile → true for every standing tree.
var _trees: Dictionary = {}
## Tile → Sprite2D of a standing tree.
var _tree_sprites: Dictionary = {}
## Tile → tick the tree was seen felled.
var _stumps: Dictionary = {}
## Tile → Sprite2D of a stump / bush / deadwood ("nature").
var _nature_sprites: Dictionary = {}
var _nature_key: int = -1
var _resources: Dictionary = {}
var _stump_check: int = 0
var _params_version: int = -1
## Chunks that hold sprites now (Vector2i → true) and the culling window they were computed for.
var _live: Dictionary = {}
var _window: Rect2i = Rect2i()


func _ready() -> void:
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world = get_node_or_null("../World")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	_version = -1
	_nature_key = -1
	_trees.clear()
	_stumps.clear()
	for s: Node in _tree_sprites.values():
		s.queue_free()
	_tree_sprites.clear()
	for s: Node in _nature_sprites.values():
		s.queue_free()
	_nature_sprites.clear()
	_live.clear()
	_window = Rect2i()


func _process(_delta: float) -> void:
	if _world_sim == null or _world == null:
		return
	if _params_version != ArtData.params_version():
		_params_version = ArtData.params_version()
		_version = -1
		_nature_key = -1
	var versions: Dictionary = SimReads.read(_world_sim, "get_colony_versions")
	var version: int = int(versions.get("trees_version", -1))
	var window_moved: bool = _update_window()
	if version != _version:
		_rebuild_trees(_version >= 0)
		_version = version
	elif window_moved:
		_sync_trees()
		_nature_key = -1
	_stump_check -= 1
	if not _stumps.is_empty() and _stump_check <= 0:
		_stump_check = 30
		_expire_stumps()
	var resources: Dictionary = SimReads.read(_world_sim, "get_resource_snapshot")
	var key: int = hash([resources.get("amounts", PackedInt32Array()), _stumps.size()])
	if key != _nature_key:
		_nature_key = key
		_resources = resources
		_sync_nature()


## Whether a crown can cover tile `t`: a tree on it, or up to CROWN_ROWS rows south of it in its column or the two
## beside it (the tree stands later in the y-sort and its crown rises ~3 tiles above its base).
func under_canopy(t: Vector2i) -> bool:
	for dy: int in range(0, CROWN_ROWS + 1):
		for dx: int in range(-1, 2):
			if _trees.has(t + Vector2i(dx, dy)):
				return true
	return false


## Number of standing trees (for tests and tools; only those in view hold a sprite).
func tree_count() -> int:
	return _trees.size()


## Number of tree sprites held now (the chunks in view; for tests and tools).
func tree_sprite_count() -> int:
	return _tree_sprites.size()


## The stump tiles currently drawn (for tests and tools).
func stump_tiles() -> Array:
	return _stumps.keys()


func _now() -> int:
	return int((SimReads.read(_world_sim, "get_time_snapshot") as Dictionary).get("tick", 0))


func _rebuild_trees(diff: bool) -> void:
	_seed = int((SimReads.read(_world_sim, "get_world_info") as Dictionary).get("seed", 0))
	var trees: Dictionary = SimReads.read(_world_sim, "get_trees")
	var xs: PackedInt32Array = trees.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = trees.get("ys", PackedInt32Array())
	var now: Dictionary = {}
	for i: int in mini(xs.size(), ys.size()):
		now[Vector2i(xs[i], ys[i])] = true
	var tick: int = _now()
	for cell: Vector2i in _trees.keys():
		if not now.has(cell) and diff:
			_stumps[cell] = tick
	for cell: Vector2i in now.keys():
		_stumps.erase(cell)
	_trees = now
	if not diff:
		for cell: Vector2i in _tree_sprites.keys():
			(_tree_sprites[cell] as Node).queue_free()
		_tree_sprites.clear()
	_sync_trees()


## Visible chunk window (view + one chunk); recomputes `_live` (keeping chunks within one more chunk). True if moved.
func _update_window() -> bool:
	var size: float = float(CHUNK * View.TILE)
	var view: Rect2 = get_viewport().get_canvas_transform().affine_inverse() * get_viewport_rect()
	var c0: Vector2i = Vector2i(floori(view.position.x / size) - 1, floori(view.position.y / size) - 1)
	var c1: Vector2i = Vector2i(floori(view.end.x / size) + 1, floori(view.end.y / size) + 1)
	var window: Rect2i = Rect2i(c0, c1 - c0 + Vector2i.ONE)
	if window == _window:
		return false
	_window = window
	var keep: Rect2i = window.grow(1)
	for c: Vector2i in _live.keys():
		if not keep.has_point(c):
			_live.erase(c)
	for cy: int in range(window.position.y, window.end.y):
		for cx: int in range(window.position.x, window.end.x):
			_live[Vector2i(cx, cy)] = true
	return true


func _chunk_of(cell: Vector2i) -> Vector2i:
	return Vector2i(floori(float(cell.x) / CHUNK), floori(float(cell.y) / CHUNK))


## Trees in live chunks have sprites; others not.
func _sync_trees() -> void:
	for cell: Vector2i in _tree_sprites.keys():
		if not _trees.has(cell) or not _live.has(_chunk_of(cell)):
			(_tree_sprites[cell] as Node).queue_free()
			_tree_sprites.erase(cell)
	for cell: Vector2i in _trees.keys():
		if not _tree_sprites.has(cell) and _live.has(_chunk_of(cell)):
			var s: Sprite2D = _standing(tree_id(_seed, cell), View.base_point(cell) + _jitter(cell))
			if s != null:
				_tree_sprites[cell] = s


## The tree sprite id of tile `cell` on world `seed` (bindings `trees`: species by weight, large by permille).
static func tree_id(seed: int, cell: Vector2i) -> String:
	var cfg: Dictionary = ArtData.bindings().get("trees", {})
	var species: Array = cfg.get("species", [])
	var total: int = 0
	for row: Variant in species:
		if row is Dictionary:
			total += maxi(0, int((row as Dictionary).get("weight", 1)))
	if total <= 0:
		return ""
	var pick: int = View.key_hash(seed, cell.x, cell.y, SALT_SPECIES) % total
	var chosen: Dictionary = {}
	for row: Variant in species:
		if row is Dictionary:
			pick -= maxi(0, int((row as Dictionary).get("weight", 1)))
			if pick < 0:
				chosen = row
				break
	var large: bool = View.key_hash(seed, cell.x, cell.y, SALT_SIZE) % 1000 < int(cfg.get("large_permille", 500))
	return str(chosen.get("large" if large else "small", ""))


func _jitter(cell: Vector2i) -> Vector2:
	var h: int = View.key_hash(_seed, cell.x, cell.y, SALT_JITTER)
	return Vector2(h % 5 - 2, (h >> 4) % 5 - 2)


## A standing sprite of manifest id `id` with its anchor at world point `base` (child of ColonyView/World).
func _standing(id: String, base: Vector2) -> Sprite2D:
	var tex: Texture2D = ArtData.texture(id)
	if tex == null:
		return null
	var s: Sprite2D = Sprite2D.new()
	s.texture = tex
	s.centered = false
	s.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	s.position = base.round()
	s.offset = -Vector2(ArtData.anchor(id))
	_world.add_child(s)
	return s


func _expire_stumps() -> void:
	var tick: int = _now()
	var ttl: int = int((ArtData.bindings().get("trees", {}) as Dictionary).get("stump_ticks", 1440))
	for cell: Vector2i in _stumps.keys():
		if tick - int(_stumps[cell]) >= ttl or tick < int(_stumps[cell]):
			_stumps.erase(cell)


## Stumps, bushes and deadwood: the wanted sprite per tile, synced with the pool (only changed tiles are touched).
func _sync_nature() -> void:
	var wanted: Dictionary = {}
	var stump: String = str(_tree_cfg("trees.stump"))
	for cell: Vector2i in _stumps.keys():
		if _live.has(_chunk_of(cell)):
			wanted[cell] = stump
	var xs: PackedInt32Array = _resources.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = _resources.get("ys", PackedInt32Array())
	var kinds: PackedInt32Array = _resources.get("kinds", PackedInt32Array())
	var amounts: PackedInt32Array = _resources.get("amounts", PackedInt32Array())
	var maxes: PackedInt32Array = _resources.get("maxes", PackedInt32Array())
	for i: int in mini(mini(xs.size(), ys.size()), mini(kinds.size(), amounts.size())):
		if not _live.has(_chunk_of(Vector2i(xs[i], ys[i]))):
			continue
		var row: Dictionary = _resource_row(kinds[i])
		if row.is_empty():
			continue
		var cap: int = maxi(1, maxes[i] if i < maxes.size() else 1)
		var permille: int = floori(amounts[i] * 1000.0 / cap)
		var id: String = ""
		if row.has("structure"):
			id = str(row["structure"])
			if row.has("depleted") and permille < int(row.get("depleted_below_permille", 0)):
				id = str(row["depleted"])
		elif row.has("item") and amounts[i] > 0:
			id = str(row["item"])
		if id != "":
			wanted[Vector2i(xs[i], ys[i])] = id
	for cell: Vector2i in _nature_sprites.keys():
		var s: Sprite2D = _nature_sprites[cell]
		if str(wanted.get(cell, "")) != str(s.get_meta("id", "")):
			s.queue_free()
			_nature_sprites.erase(cell)
	for cell: Vector2i in wanted.keys():
		if _nature_sprites.has(cell):
			continue
		var s: Sprite2D = _standing(str(wanted[cell]), View.base_point(cell) - Vector2(0, 1))
		if s != null:
			s.set_meta("id", str(wanted[cell]))
			_nature_sprites[cell] = s


## A value of bindings.json by dotted path ("" when missing).
static func _tree_cfg(path: String) -> Variant:
	var node: Variant = ArtData.bindings()
	for key: String in path.split("."):
		if not (node is Dictionary and (node as Dictionary).has(key)):
			return ""
		node = (node as Dictionary)[key]
	return node


## The bindings `resources` row of resource kind `kind` (only listed kinds are drawn here).
func _resource_row(kind: int) -> Dictionary:
	for r: Variant in (ArtData.bindings().get("resources", {}) as Dictionary).get("codes", []):
		if r is Dictionary and int((r as Dictionary).get("code", -1)) == kind:
			return r
	return {}
