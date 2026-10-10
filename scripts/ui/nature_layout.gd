extends RefCounted
const HASH = preload("res://scripts/ui/art_hash.gd")
const GROVE_BASE: float = 0.05
const GROVE_GAIN: float = 0.65

static func roll(x: int, y: int, salt: int) -> float:
	return float(HASH.mix(((x*73856093) ^ (y*19349663)) ^ salt)) / 4294967295.0

static func grove(x: int, y: int, salt: int = 0) -> float:
	var lx: int = floori(float(x)/6.0)
	var ly: int = floori(float(y)/6.0)
	var tx: float = float(posmod(x,6))/6.0
	var ty: float = float(posmod(y,6))/6.0
	return lerpf(lerpf(roll(lx,ly,salt),roll(lx+1,ly,salt),tx),lerpf(roll(lx,ly+1,salt),roll(lx+1,ly+1,salt),tx),ty)

static func forest_tree(x: int, y: int) -> bool:
	return roll(x,y,0x713B) < GROVE_BASE + GROVE_GAIN * smoothstep(0.35,0.75,grove(x,y))

static func clearings(buildings: Dictionary, sites: Dictionary, food: Dictionary, settlements: Dictionary, radius: int = 4) -> Dictionary:
	var centers: Array[Vector2i] = []
	for snapshot: Dictionary in [buildings, sites]:
		var xs: PackedInt32Array = snapshot.get("xs",PackedInt32Array())
		var ys: PackedInt32Array = snapshot.get("ys",PackedInt32Array())
		for i: int in range(xs.size()):
			centers.append(Vector2i(xs[i],ys[i]))
	var sx: PackedInt32Array = settlements.get("formation_xs",PackedInt32Array())
	var sy: PackedInt32Array = settlements.get("formation_ys",PackedInt32Array())
	for i: int in range(sx.size()):
		centers.append(Vector2i(sx[i],sy[i]))
	var fx: PackedInt32Array = food.get("xs",PackedInt32Array())
	var fy: PackedInt32Array = food.get("ys",PackedInt32Array())
	var kinds: PackedInt32Array = food.get("kinds",PackedInt32Array())
	for i: int in range(fx.size()):
		if kinds[i] == 0:
			centers.append(Vector2i(fx[i],fy[i]))
	var cleared: Dictionary = {}
	for center: Vector2i in centers:
		for dy: int in range(-radius,radius+1):
			for dx: int in range(-radius,radius+1):
				cleared[center+Vector2i(dx,dy)] = true
	return cleared

## Rank eligible tiles by independent value noise, with stable local variation.
## The caller supplies a budget computed from ALL tiles of the terrain type.
static func patches(candidates: Array[Vector2i], budget: int, salt: int) -> Dictionary:
	var ranked: Array[Dictionary] = []
	for tile: Vector2i in candidates:
		ranked.append({"tile": tile, "score": grove(tile.x,tile.y,salt) * 0.85 + roll(tile.x,tile.y,salt ^ 0x713B) * 0.15})
	ranked.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		if a["score"] == b["score"]:
			var at: Vector2i = a["tile"]
			var bt: Vector2i = b["tile"]
			return at.y < bt.y or (at.y == bt.y and at.x < bt.x)
		return a["score"] > b["score"])
	var selected: Dictionary = {}
	for i: int in range(mini(budget,ranked.size())):
		selected[ranked[i]["tile"]] = true
	return selected

static func rocks(terrain: PackedByteArray, width: int, cleared: Dictionary) -> Dictionary:
	var hills: Array[Vector2i] = []
	var mountains: Array[Vector2i] = []
	for i: int in range(terrain.size()):
		var tile: Vector2i = Vector2i(i % width, floori(float(i) / float(width)))
		if cleared.has(tile):
			continue
		if terrain[i] == 3:
			hills.append(tile)
		elif terrain[i] == 2:
			mountains.append(tile)
	var selected: Dictionary = patches(hills,roundi(terrain.count(3)*0.06),0x34AC)
	selected.merge(patches(mountains,roundi(terrain.count(2)*0.45),0x34AC))
	return selected
