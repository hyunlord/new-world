# view.gd — the colony view's world scale and integer noise (scripts/world/colony/).
# Purpose: the one world scale (TILE = 64 world pixels per tile, ADR 0034; the M-ART2 projection: the ground from
#   above, everything standing in the same front 3/4), tile <-> world conversions, the standing base point of a tile
#   (`base_point`: where a person's feet or a structure's anchor sits, (tile_left + 32, tile_top + 43)), a stable tile
#   hash and the value noise that clusters the camp clearing (keyed on the world seed, never randf()). Tile (x, y)
#   covers world pixels [x*64, x*64+64) × [y*64, y*64+64); the map's top-left is the world origin. What is drawn where
#   lives in data (assets/v8art/), read by scripts/world/art/art_data.gd.
# Bridge methods called: none.
extends RefCounted

const TILE: int = 64
## The standing base inside a tile: feet and anchors sit in the tile's lower third (STRUCT PROJECTION.md).
const BASE: Vector2i = Vector2i(32, 43)


## Top-left world position of tile `t`.
static func tile_to_world(t: Vector2i) -> Vector2:
	return Vector2(t.x * TILE, t.y * TILE)


## The tile under world position `p`.
static func world_to_tile(p: Vector2) -> Vector2i:
	return Vector2i(floori(p.x / float(TILE)), floori(p.y / float(TILE)))


## The world point where a standing thing on tile `t` touches the ground.
static func base_point(t: Vector2i) -> Vector2:
	return Vector2(t.x * TILE + BASE.x, t.y * TILE + BASE.y)


## The centre of tile `t` in world pixels.
static func tile_centre(t: Vector2i) -> Vector2:
	return Vector2(t.x * TILE + TILE * 0.5, t.y * TILE + TILE * 0.5)


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


## Smooth value noise in [0, 1): a lattice of hashed values every `scale` tiles, interpolated with smoothstep.
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


## The clearing's cluster noise: 0.65 × coarse (9 tiles) + 0.35 × fine (4 tiles).
static func cluster_noise(seed: int, x: int, y: int) -> float:
	return 0.65 * value_noise(seed, x, y, 9, 1) + 0.35 * value_noise(seed, x, y, 4, 2)
