extends RefCounted
const HASH = preload("res://scripts/ui/art_hash.gd")
const GROVE_BASE: float = 0.05
const GROVE_GAIN: float = 0.65

static func roll(x: int, y: int, salt: int) -> float:
	return float(HASH.mix(((x*73856093) ^ (y*19349663)) ^ salt)) / 4294967295.0

static func grove(x: int, y: int) -> float:
	var lx: int = floori(float(x)/6.0)
	var ly: int = floori(float(y)/6.0)
	var tx: float = float(posmod(x,6))/6.0
	var ty: float = float(posmod(y,6))/6.0
	return lerpf(lerpf(roll(lx,ly,0),roll(lx+1,ly,0),tx),lerpf(roll(lx,ly+1,0),roll(lx+1,ly+1,0),tx),ty)

static func forest_tree(x: int, y: int) -> bool:
	return roll(x,y,0x713B) < GROVE_BASE + GROVE_GAIN * smoothstep(0.35,0.75,grove(x,y))

static func clearings(buildings: Dictionary, sites: Dictionary, food: Dictionary, settlements: Dictionary) -> Dictionary:
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
		for dy: int in range(-4,5):
			for dx: int in range(-4,5):
				cleared[center+Vector2i(dx,dy)] = true
	return cleared
