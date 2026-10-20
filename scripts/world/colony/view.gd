# view.gd — shared constants of the M1 colony view (scripts/world/colony/).
# Purpose: the tile size, tile <-> world conversions, the temporary tile atlas
#   (assets/v8/temp/tiles.png + tiles.json, drawn by tools/v8/gen_temp_tiles.py)
#   as named rects, and TileSets built from it. Tile (x, y) covers world pixels
#   [x*16, x*16+16) × [y*16, y*16+16); the map's top-left is the world origin.
# Bridge methods called: none.
extends RefCounted

const TILE: int = 16
const ATLAS_PNG: String = "res://assets/v8/temp/tiles.png"
const ATLAS_JSON: String = "res://assets/v8/temp/tiles.json"
# Household tunic colours (household id mod 6) and hair colours (agent id mod 4). Tunics are light or saturated
#   (cream, sky blue, brick red, teal, violet, rose) so people stand out from the dark earth and grass; no
#   dirt-brown or grass-olive (player review round 1, #3) and no yellow (the order marks and the stockpile edge
#   are yellow). The tunic layer's base grey is 212, so the drawn colour is about 0.83 × these.
const TUNICS: Array[Color] = [Color("efe2c0"), Color("6fb0e8"), Color("e0604c"), Color("50c0a8"), Color("b088e0"), Color("e888b0")]
const HAIRS: Array[Color] = [Color("2a1e16"), Color("5a3a22"), Color("8a6a3a"), Color("3a2a1e")]
# Elders: near-white, clearly lighter than the skin (the old c8c4bc was as light as the skin and did not show).
const ELDER_HAIR: Color = Color("ffffff")
const SKIN: Color = Color("c89a78")

static var _rects: Dictionary = {}
static var _texture: Texture2D = null


static func _load() -> void:
	if not _rects.is_empty():
		return
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(ATLAS_JSON))
	if parsed is Dictionary:
		var tiles: Variant = (parsed as Dictionary).get("tiles", parsed)
		if tiles is Dictionary:
			for name: String in (tiles as Dictionary).keys():
				var r: Variant = tiles[name]
				if r is Array and (r as Array).size() == 4:
					_rects[name] = Rect2i(int(r[0]), int(r[1]), int(r[2]), int(r[3]))
	_texture = load(ATLAS_PNG) as Texture2D


## The atlas texture.
static func texture() -> Texture2D:
	_load()
	return _texture


## The atlas rect of tile `name` (empty when unknown).
static func rect(name: String) -> Rect2:
	_load()
	return Rect2(_rects.get(name, Rect2i()))


## Whether the atlas has a tile called `name`.
static func has(name: String) -> bool:
	_load()
	return _rects.has(name)


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
