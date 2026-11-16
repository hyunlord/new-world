# view.gd — the colony's world-pixel scale and integer noise (scripts/world/colony/).
# Purpose: the one world scale (TILE = 64 world pixels per tile, ADR 0034; the 3D view keeps it: one tile = one 3D
#   unit = 64 world pixels), tile <-> world conversions, and the keyed hash and value noise (keyed on the world seed,
#   never randf()) the 3D terrain, the minimap and the people colours use. Tile (x, y) covers world pixels
#   [x*64, x*64+64) × [y*64, y*64+64); the map's top-left is the world origin.
# Bridge methods called: none.
extends RefCounted

const TILE: int = 64


## Top-left world position of tile `t`.
static func tile_to_world(t: Vector2i) -> Vector2:
	return Vector2(t.x * TILE, t.y * TILE)


## The tile under world position `p`.
static func world_to_tile(p: Vector2) -> Vector2i:
	return Vector2i(floori(p.x / float(TILE)), floori(p.y / float(TILE)))


## The centre of tile `t` in world pixels.
static func tile_centre(t: Vector2i) -> Vector2:
	return Vector2(t.x * TILE + TILE * 0.5, t.y * TILE + TILE * 0.5)


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

