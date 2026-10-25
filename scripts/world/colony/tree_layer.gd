# tree_layer.gd (ColonyView/TreeLayer in scenes/main.tscn): the trees, stumps, berry bushes and deadwood of the
#   colony view at 64-px tiles (M-ART2 §3.5 "things").
# Every tree of get_trees is a standing Sprite2D in the y-sorted ColonyView/World (position = the tile's base point
#   (tile_left + 32, tile_top + 43) plus a ±2 px jitter, offset = −anchor, so trunks sort with people, walls and
#   structures by base y). Trees resolve through the flora catalogue (flora_resolver.gd, assets/v8art/flora/,
#   M-ART2.1 §2.2): (species, size, season, variant) → frame, canopy mask, anchor and season tint. Species is the
#   persisted material name in get_trees; keyed hashes still choose size and variant independently.
# Seasons: the calendar's season (get_time_snapshot `season`, named by the catalogue's season order) drives
#   shaders/tree_season.gdshader per frame: the canopy's Lab shift, a deciduous tree's bare winter frame, an
#   evergreen's snow cap (a band below each column's top canopy pixel, as seasons.py). During the first day of a season
#   the previous season's look blends into the new one (`blend` = the day's fraction; the world's first season starts
#   blended). Materials are shared per (frames, season pair) and only `blend` changes per frame.
# At ¼× the ground layer bakes trees into its chunks as canopy mass (ground_baker.gd); tree sprites standing in a chunk
#   whose texture carries the canopy are hidden (re-checked when the ground's `canopy_version` moves).
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
# Berry bushes take their species' season tint (bindings `resources` `flora_species`; the frame's mask from the
#   catalogue entry with that frame's path).
# Bridge methods: via SimReads get_colony_versions, get_trees, get_resource_snapshot, get_time_snapshot,
#   get_world_info; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const FloraResolver = preload("res://scripts/world/art/flora_resolver.gd")
const SEASON_SHADER: Shader = preload("res://shaders/tree_season.gdshader")
## Crown reach for `under_canopy`: rows south of the person a tree can stand and still cover them.
const CROWN_ROWS: int = 2
const SALT_SIZE: int = 8
const SALT_JITTER: int = 9
const SALT_VARIANT: int = 10
## Size names of the catalogue's two tree sizes, picked by `large_permille` (schema names, not content).
const SIZE_LARGE: String = "large"
const SIZE_SMALL: String = "small"
## Prefix of a stump's key in the nature pool (a key, not content).
const STUMP_KEY: String = "stump:"
## Culling chunk size in tiles.
const CHUNK: int = 16

var _world_sim: Node = null
var _world: Node2D = null
var _version: int = -1
var _seed: int = 0
## Tile → authoritative species for every standing tree.
var _trees: Dictionary = {}
## Tile → Sprite2D of a standing tree.
var _tree_sprites: Dictionary = {}
## Tile → {tick, species} remembered when the tree was seen felled.
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
var _flora: FloraResolver = null
var _ground: Node = null
## The season pair drawn now ("from|to") and the blend; materials by key; snow bands by (mask, frame, depth).
var _season_pair: String = ""
var _blend: float = 1.0
var _materials: Dictionary = {}
var _snow: Dictionary = {}
var _canopy_version: int = -1
## A season pair and blend forced by a test or a capture ("from|to", blend); "" follows the calendar.
var _forced_pair: String = ""
var _forced_blend: float = 1.0


func _ready() -> void:
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world = get_node_or_null("../World")
	_ground = get_node_or_null("../GroundLayer")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	_flora = FloraResolver.new()
	_flora.load_imported()
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
	if _update_season():
		# A new season pair changes frames (bare winter) and materials: re-make every sprite.
		_version = -1
		_nature_key = -1
		for n: Node in _nature_sprites.values():
			n.queue_free()
		_nature_sprites.clear()
	if version != _version:
		if _rebuild_trees(_version >= 0):
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
	if _ground != null and int(_ground.get("canopy_version")) != _canopy_version:
		_canopy_version = int(_ground.get("canopy_version"))
		_apply_canopy_hiding()


## The season pair and blend from the calendar; true when the pair changed. Materials get the new blend.
func _update_season() -> bool:
	if _forced_pair != "":
		return _set_season(_forced_pair, _forced_blend)
	var state: Array = season_state(SimReads.read(_world_sim, "get_time_snapshot"), season_names())
	if state.is_empty():
		return false
	return _set_season(str(state[0]), float(state[1]))


## The catalogue's season names by the calendar's season code (bindings `trees.season_by_code`, the bridge's
## `season` contract 0 spring … 3 winter; the catalogue's own order when the binding is missing).
func season_names() -> Array[String]:
	var out: Array[String] = []
	for n: Variant in (ArtData.bindings().get("trees", {}) as Dictionary).get("season_by_code", []):
		if _flora.seasons().has(str(n)):
			out.append(str(n))
	return out if out.size() == _flora.seasons().size() else _flora.seasons()


## ["from|to", blend] for a time snapshot: the season by code and the one before it; blend = the fraction of the
## first day of a season (0 at its midnight, 1 from the second day), except in the world's first season, which has
## no previous season on screen (the calendar starts on day 1 of a season). [] without seasons. Pure (tests).
static func season_state(t: Dictionary, names: Array[String]) -> Array:
	if names.is_empty() or int(t.get("days_per_season", 0)) <= 0:
		return [] if names.is_empty() else ["%s|%s" % [names[0], names[0]], 1.0]
	var code: int = clampi(int(t.get("season", 0)), 0, names.size() - 1)
	var blend: float = 1.0
	if int(t.get("day_of_season", 0)) == 1 and int(t.get("day_index", 0)) >= int(t.get("days_per_season", 0)):
		blend = clampf(float(t.get("time_of_day", 24.0)) / 24.0, 0.0, 1.0)
	return ["%s|%s" % [names[posmod(code - 1, names.size())], names[code]], blend]


## Force the drawn season pair ("" to follow the calendar again); tests and captures. Applied on the next frame.
func force_season(from: String, to: String, blend: float = 1.0) -> void:
	_forced_pair = "%s|%s" % [from, to] if to != "" else ""
	_forced_blend = clampf(blend, 0.0, 1.0)


func _set_season(pair: String, blend: float) -> bool:
	blend = roundf(blend * 96.0) / 96.0
	var changed: bool = pair != _season_pair
	_season_pair = pair
	if changed:
		_materials.clear()
	if changed or blend != _blend:
		_blend = blend
		for m: ShaderMaterial in _materials.values():
			m.set_shader_parameter("blend", _blend)
	return changed


## The season names drawn now: [from, to] (tests and the ground layer's canopy, which switches at half blend).
func season_pair() -> PackedStringArray:
	return _season_pair.split("|")


## The blend between the season pair (1 = fully the current season).
func season_blend() -> float:
	return _blend

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


## Number of tree sprites drawn now (held and not hidden under a ¼× canopy bake; tests and tools).
func tree_sprites_visible() -> int:
	var n: int = 0
	for s: Sprite2D in _tree_sprites.values():
		if s.visible:
			n += 1
	return n


## The tree sprite of tile `cell` (null when none is held; tests).
func tree_sprite(cell: Vector2i) -> Sprite2D:
	return _tree_sprites.get(cell, null)


## The flora resolver in use (tests: the data-only species).
func flora() -> FloraResolver:
	return _flora


## Hide the tree sprites of chunks whose ground texture carries the baked canopy (¼×); show the others.
func _apply_canopy_hiding() -> void:
	if _ground == null:
		return
	for cell: Vector2i in _tree_sprites.keys():
		(_tree_sprites[cell] as Sprite2D).visible = not bool(_ground.call("canopy_baked_at", cell))


## The stump tiles currently drawn (for tests and tools).
func stump_tiles() -> Array:
	return _stumps.keys()


func _now() -> int:
	return int((SimReads.read(_world_sim, "get_time_snapshot") as Dictionary).get("tick", 0))


func _rebuild_trees(diff: bool) -> bool:
	_seed = int((SimReads.read(_world_sim, "get_world_info") as Dictionary).get("seed", 0))
	var trees: Dictionary = SimReads.read(_world_sim, "get_trees")
	var xs: PackedInt32Array = trees.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = trees.get("ys", PackedInt32Array())
	var species: PackedStringArray = trees.get("species", PackedStringArray())
	if xs.size() != ys.size() or xs.size() != species.size() or species.has(""):
		push_error("get_trees requires aligned nonempty species identities")
		return false
	var now: Dictionary = {}
	for i: int in xs.size():
		now[Vector2i(xs[i], ys[i])] = species[i]
	var tick: int = _now()
	for cell: Vector2i in _trees.keys():
		if not now.has(cell) and diff:
			_stumps[cell] = {"tick": tick, "species": str(_trees[cell])}
	for cell: Vector2i in now.keys():
		_stumps.erase(cell)
		if _tree_sprites.has(cell) and str(_trees.get(cell, "")) != str(now[cell]):
			(_tree_sprites[cell] as Node).queue_free()
			_tree_sprites.erase(cell)
	_trees = now
	if not diff:
		for cell: Vector2i in _tree_sprites.keys():
			(_tree_sprites[cell] as Node).queue_free()
		_tree_sprites.clear()
	_sync_trees()
	_nature_key = -1
	return true


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
			var s: Sprite2D = _tree(cell)
			if s != null:
				_tree_sprites[cell] = s
				if _ground != null and bool(_ground.call("canopy_baked_at", cell)):
					s.visible = false


## The size name of tile `cell` on world `seed` (large below `trees.large_permille`).
static func tree_size(seed: int, cell: Vector2i) -> String:
	var cfg: Dictionary = ArtData.bindings().get("trees", {})
	return SIZE_LARGE if View.key_hash(seed, cell.x, cell.y, SALT_SIZE) % 1000 < int(cfg.get("large_permille", 500)) else SIZE_SMALL


## The variant hash of tile `cell` on world `seed`.
static func tree_variant(seed: int, cell: Vector2i) -> int:
	return View.key_hash(seed, cell.x, cell.y, SALT_VARIANT)


## The standing sprite of the tree on `cell`, resolved for the season pair drawn now.
func _tree(cell: Vector2i) -> Sprite2D:
	var habit: String = str((ArtData.bindings().get("trees", {}) as Dictionary).get("habit", ""))
	return make_plant(str(_trees[cell]), tree_size(_seed, cell), tree_variant(_seed, cell), habit, View.base_point(cell) + _jitter(cell))


## A standing plant of `species` (resolved through the catalogue for the season pair) at world point `base`; null when
## the catalogue or the texture cannot answer. Public so a test can draw a data-only species through this code path.
func make_plant(species: String, size: String, variant: int, habit: String, base: Vector2) -> Sprite2D:
	var pair: PackedStringArray = _season_pair.split("|")
	if pair.size() != 2:
		return null
	var to: Dictionary = _flora.resolve(species, size, pair[1], variant, habit)
	var from: Dictionary = _flora.resolve(species, size, pair[0], variant, habit)
	if to.is_empty():
		return null
	var tex: Texture2D = ArtData.texture_at(FloraResolver.res_path(str(to["path"])))
	if tex == null:
		return null
	var s: Sprite2D = Sprite2D.new()
	s.texture = tex
	s.centered = false
	s.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	s.position = base.round()
	s.offset = -Vector2(int(to["anchor"][0]), int(to["anchor"][1]))
	s.material = _season_material(to, from if not from.is_empty() else to)
	_world.add_child(s)
	return s


## The shared season material of a frame pair (current season `to`, previous `from`).
func _season_material(to: Dictionary, from: Dictionary) -> ShaderMaterial:
	# A dissolve between two frames samples both at the same UV: only frames of one canvas and anchor dissolve; any
	# other pair draws the current season's frame (it was resolved for `to`).
	var alt: bool = str(from["path"]) != str(to["path"]) and str(from.get("canvas", [])) == str(to.get("canvas", [])) and str(from.get("anchor", [])) == str(to.get("anchor", []))
	var key: String = "%s|%s|%s|%s" % [to["path"], from["path"] if alt else "", str(to["tint"]), str(from["tint"])]
	if _materials.has(key):
		return _materials[key]
	var m: ShaderMaterial = ShaderMaterial.new()
	m.shader = SEASON_SHADER
	var tt: Dictionary = to["tint"]
	var ft: Dictionary = from["tint"]
	m.set_shader_parameter("delta_to", Vector3(float(tt["delta_lab"][0]), float(tt["delta_lab"][1]), float(tt["delta_lab"][2])))
	m.set_shader_parameter("delta_from", Vector3(float(ft["delta_lab"][0]), float(ft["delta_lab"][1]), float(ft["delta_lab"][2])))
	m.set_shader_parameter("snow_to", _snow_alpha(tt))
	m.set_shader_parameter("snow_from", _snow_alpha(ft))
	_bind_frame(m, to, "canopy_mask", "snow_mask", "has_mask", "has_snow", maxi(int((tt["snow_cap"] as Dictionary)["depth_px"]), int((ft["snow_cap"] as Dictionary)["depth_px"])))
	m.set_shader_parameter("has_alt", alt)
	if alt:
		m.set_shader_parameter("alt_tex", ArtData.texture_at(FloraResolver.res_path(str(from["path"]))))
		_bind_frame(m, from, "alt_mask", "alt_snow", "has_alt_mask", "has_alt_snow", int((ft["snow_cap"] as Dictionary)["depth_px"]))
	m.set_shader_parameter("blend", _blend)
	_materials[key] = m
	return m


## seasons.py floors the snow alpha to 8 bits (0.7 → 178/255).
static func _snow_alpha(tint: Dictionary) -> float:
	var snow: Dictionary = tint["snow_cap"]
	return floorf(float(snow["alpha"]) * 255.0) / 255.0 if int(snow["depth_px"]) > 0 else 0.0


func _bind_frame(m: ShaderMaterial, r: Dictionary, mask_name: String, snow_name: String, has_mask: String, has_snow: String, depth: int) -> void:
	var mask: Texture2D = ArtData.texture_at(FloraResolver.res_path(str(r["canopy_mask"])))
	m.set_shader_parameter(has_mask, mask != null)
	if mask != null:
		m.set_shader_parameter(mask_name, mask)
	var band: Texture2D = _snow_band(str(r["path"]), str(r["canopy_mask"]), depth) if depth > 0 else null
	m.set_shader_parameter(has_snow, band != null)
	if band != null:
		m.set_shader_parameter(snow_name, band)


## seasons.py `snow_layer`: support = mask ∧ alpha > 0; per column the band from its top support pixel down `depth`
## rows, within the support. Cached per (frame, depth).
func _snow_band(frame_path: String, mask_path: String, depth: int) -> Texture2D:
	var key: String = "%s|%d" % [frame_path, depth]
	if _snow.has(key):
		return _snow[key]
	var frame: Image = ArtData.image_at(FloraResolver.res_path(frame_path))
	var mask: Image = ArtData.image_at(FloraResolver.res_path(mask_path))
	if frame == null or mask == null or frame.get_size() != mask.get_size():
		_snow[key] = null
		return null
	var w: int = frame.get_width()
	var h: int = frame.get_height()
	var fd: PackedByteArray = frame.get_data()
	var md: PackedByteArray = mask.get_data()
	var out: PackedByteArray = PackedByteArray()
	out.resize(w * h * 4)
	for x: int in w:
		var top: int = -1
		for y: int in h:
			var i: int = (y * w + x) * 4
			if md[i] > 0 and fd[i + 3] > 0:
				if top < 0:
					top = y
				if y < top + depth:
					out[i] = 255
					out[i + 1] = 255
					out[i + 2] = 255
					out[i + 3] = 255
	var tex: ImageTexture = ImageTexture.create_from_image(Image.create_from_data(w, h, false, Image.FORMAT_RGBA8, out))
	_snow[key] = tex
	return tex


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
		if tick - int((_stumps[cell] as Dictionary)["tick"]) >= ttl or tick < int((_stumps[cell] as Dictionary)["tick"]):
			_stumps.erase(cell)


## Stumps, bushes and deadwood: the wanted sprite per tile, synced with the pool (only changed tiles are touched).
func _sync_nature() -> void:
	var wanted: Dictionary = {}
	var flora_of: Dictionary = {}
	for cell: Vector2i in _stumps.keys():
		if _live.has(_chunk_of(cell)):
			# The felled tree's own species' stump (its habit fallback's when the species has none).
			wanted[cell] = STUMP_KEY + str((_stumps[cell] as Dictionary)["species"])
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
			if row.has("flora_species"):
				flora_of[Vector2i(xs[i], ys[i])] = str(row["flora_species"])
	for cell: Vector2i in _nature_sprites.keys():
		var s: Sprite2D = _nature_sprites[cell]
		if str(wanted.get(cell, "")) != str(s.get_meta("id", "")):
			s.queue_free()
			_nature_sprites.erase(cell)
	for cell: Vector2i in wanted.keys():
		if _nature_sprites.has(cell):
			continue
		var want: String = str(wanted[cell])
		var base: Vector2 = View.base_point(cell) - Vector2(0, 1)
		var s: Sprite2D = _stump(want.substr(STUMP_KEY.length()), base) if want.begins_with(STUMP_KEY) else _standing(want, base)
		if s != null:
			s.set_meta("id", want)
			if flora_of.has(cell):
				s.material = _bush_material(want, str(flora_of[cell]))
			_nature_sprites[cell] = s


## A stump sprite of `species` at world point `base` (the catalogue's stump frame).
func _stump(species: String, base: Vector2) -> Sprite2D:
	var habit: String = str((ArtData.bindings().get("trees", {}) as Dictionary).get("habit", ""))
	var frame: Dictionary = _flora.stump(species, habit)
	if frame.is_empty():
		return null
	var tex: Texture2D = ArtData.texture_at(FloraResolver.res_path(str(frame.get("path", ""))))
	if tex == null:
		return null
	var s: Sprite2D = Sprite2D.new()
	s.texture = tex
	s.centered = false
	s.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	s.position = base.round()
	var a: Array = frame.get("anchor", [0, 0])
	s.offset = -Vector2(int(a[0]), int(a[1]))
	_world.add_child(s)
	return s


## The season material of bush frame `id` (manifest id) for `species`: its season tint, the mask imported beside the
## frame (`<id>_mask`, the import's naming).
func _bush_material(id: String, species: String) -> ShaderMaterial:
	var pair: PackedStringArray = _season_pair.split("|")
	if pair.size() != 2:
		return null
	var to: Dictionary = _flora.resolve(species, SIZE_SMALL, pair[1], 0)
	var from: Dictionary = _flora.resolve(species, SIZE_SMALL, pair[0], 0)
	if to.is_empty():
		return null
	var source: String = str(ArtData.row(id).get("source", ""))
	var mask_source: String = _flora.mask_of(source)
	to = to.duplicate()
	to["path"] = source
	to["canopy_mask"] = mask_source
	from = (from if not from.is_empty() else to).duplicate()
	from["path"] = source
	from["canopy_mask"] = mask_source
	return _season_material(to, from)


## The bindings `resources` row of resource kind `kind` (only listed kinds are drawn here).
func _resource_row(kind: int) -> Dictionary:
	for r: Variant in (ArtData.bindings().get("resources", {}) as Dictionary).get("codes", []):
		if r is Dictionary and int((r as Dictionary).get("code", -1)) == kind:
			return r
	return {}
