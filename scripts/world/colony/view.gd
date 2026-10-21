# view.gd — shared constants of the M1 colony view (scripts/world/colony/).
# Purpose: the tile size, tile <-> world conversions, the M1-ART sheet (assets/v8/tiles.png + tiles.json, drawn by
#   tools/v8/gen_tiles.py) as named rects plus its palettes (tunics, hairs, elder hair, skin, halo), TileSets built
#   from it, a stable tile hash and the value noise that clusters the ground (the mock-up's `value_noise`, keyed
#   on the world seed instead of a random generator). Tile (x, y) covers world pixels [x*16, x*16+16) ×
#   [y*16, y*16+16); the map's top-left is the world origin.
# Bridge methods called: none.
extends RefCounted

const TILE: int = 16
const ATLAS_PNG: String = "res://assets/v8/tiles.png"
const ATLAS_JSON: String = "res://assets/v8/tiles.json"

static var _rects: Dictionary = {}
static var _palettes: Dictionary = {}
static var _texture: Texture2D = null


static func _load() -> void:
	if not _rects.is_empty():
		return
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(ATLAS_JSON))
	if parsed is Dictionary:
		var tiles: Variant = (parsed as Dictionary).get("tiles", {})
		if tiles is Dictionary:
			for name: String in (tiles as Dictionary).keys():
				var r: Variant = tiles[name]
				if r is Array and (r as Array).size() == 4:
					_rects[name] = Rect2i(int(r[0]), int(r[1]), int(r[2]), int(r[3]))
		var palettes: Variant = (parsed as Dictionary).get("palettes", {})
		if palettes is Dictionary:
			_palettes = palettes
	_texture = load(ATLAS_PNG) as Texture2D


## The atlas texture.
static func texture() -> Texture2D:
	_load()
	return _texture


## The atlas rect of sprite `name` (empty when unknown).
static func rect(name: String) -> Rect2:
	_load()
	return Rect2(_rects.get(name, Rect2i()))


## Whether the atlas has a sprite called `name`.
static func has(name: String) -> bool:
	_load()
	return _rects.has(name)


## A palette colour list from the sheet (`tunics`, `hairs`); white when missing.
static func palette(key: String) -> Array[Color]:
	_load()
	var out: Array[Color] = []
	var raw: Variant = _palettes.get(key, [])
	if raw is Array:
		for h: Variant in raw:
			out.append(Color(str(h)))
	if out.is_empty():
		out.append(Color.WHITE)
	return out


## A single palette colour from the sheet (`skin`, `elder_hair`, `halo`); white when missing.
static func colour(key: String) -> Color:
	_load()
	return Color(str(_palettes.get(key, "ffffff")))


## Top-left world position of tile `t`.
static func tile_to_world(t: Vector2i) -> Vector2:
	return Vector2(t.x * TILE, t.y * TILE)


## The tile under world position `p`.
static func world_to_tile(p: Vector2) -> Vector2i:
	return Vector2i(floori(p.x / float(TILE)), floori(p.y / float(TILE)))


## A TileSet with one atlas source holding the 16×16 tiles `names`; returns
## {"tileset": TileSet, "coords": {name: Vector2i atlas coords}}.
static func tileset(names: Array) -> Dictionary:
	_load()
	var atlas: TileSetAtlasSource = TileSetAtlasSource.new()
	atlas.texture = _texture
	atlas.texture_region_size = Vector2i(TILE, TILE)
	var coords: Dictionary = {}
	for name: String in names:
		if not _rects.has(name):
			continue
		var r: Rect2i = _rects[name]
		@warning_ignore("integer_division")
		var cell: Vector2i = Vector2i(r.position.x / TILE, r.position.y / TILE)
		if not atlas.has_tile(cell):
			atlas.create_tile(cell)
		coords[name] = cell
	var tiles: TileSet = TileSet.new()
	tiles.tile_size = Vector2i(TILE, TILE)
	tiles.add_source(atlas, 0)
	return {"tileset": tiles, "coords": coords}


## A stable small hash of a tile (for variants; never randf()).
static func tile_hash(x: int, y: int) -> int:
	var h: int = (x * 73856093) ^ (y * 19349663)
	return absi(h) % 1000003


## A stable hash of (seed, x, y, salt) in [0, 2^31) (integer mixing; no randf()).
static func key_hash(seed: int, x: int, y: int, salt: int) -> int:
	var h: int = seed * 0x27d4eb2d + x * 0x165667b1 + y * 0x61c88647 + salt * 0x9e3779b1
	h = (h ^ (h >> 15)) * 0x2c1b3c6d
	h = (h ^ (h >> 12)) * 0x297a2d39
	h = h ^ (h >> 15)
	return h & 0x7fffffff


## Smooth value noise in [0, 1): a lattice of hashed values every `scale` tiles, interpolated with smoothstep
## (the mock-up's value_noise).
static func value_noise(seed: int, x: int, y: int, scale: int, salt: int) -> float:
	@warning_ignore("integer_division")
	var gx: int = x / scale
	@warning_ignore("integer_division")
	var gy: int = y / scale
	var tx: float = float(x - gx * scale) / float(scale)
	var ty: float = float(y - gy * scale) / float(scale)
	tx = tx * tx * (3.0 - 2.0 * tx)
	ty = ty * ty * (3.0 - 2.0 * ty)
	var a: float = float(key_hash(seed, gx, gy, salt) % 10000) / 10000.0
	var b: float = float(key_hash(seed, gx + 1, gy, salt) % 10000) / 10000.0
	var c: float = float(key_hash(seed, gx, gy + 1, salt) % 10000) / 10000.0
	var d: float = float(key_hash(seed, gx + 1, gy + 1, salt) % 10000) / 10000.0
	return lerpf(lerpf(a, b, tx), lerpf(c, d, tx), ty)


## The ground's cluster noise: 0.65 × coarse (9 tiles) + 0.35 × fine (4 tiles), as the mock-up mixes them.
static func cluster_noise(seed: int, x: int, y: int) -> float:
	return 0.65 * value_noise(seed, x, y, 9, 1) + 0.35 * value_noise(seed, x, y, 4, 2)
