class_name ArtHash
extends RefCounted

## One overflow-safe visual identity hash, independent of simulation storage ids.
static func mix(x: int) -> int:
	x = (x ^ 0x9E3779B9) & 0xFFFFFFFF
	x = ((x ^ (x >> 16)) * 0x45D9F3B) & 0xFFFFFFFF
	x = ((x ^ (x >> 16)) * 0x45D9F3B) & 0xFFFFFFFF
	return x ^ (x >> 16)

static func look(agent_id: int) -> Dictionary:
	var h: int = mix(agent_id)
	return {
		"jx": posmod(h, 9) - 4,
		"jy": posmod(h >> 4, 5) - 2,
		"body": (h >> 8) & 1,
		"skin": posmod(h >> 9, 4),
		"tone": posmod(h >> 9, 3),
		"hair": posmod(h >> 11, 5),
		"clothing": posmod(h >> 15, 6),
		"phase": float(posmod(h >> 20, 1000)) / 1000.0,
	}
