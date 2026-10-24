# ground_baker.gd — bakes one ground chunk (16 × 16 tiles = 1,024 × 1,024 world px, RGBA8) on the CPU (ADR 0034).
# Purpose: the top-down layer of the one projection (ART-STRUCT): per tile the ground texture sampled with world
#   offsets (textures pre-tiled once so a run of equal tiles in a row is one native blit), the noise-dithered 12-px
#   blend between different grounds (edges.py `render_tile`: fixed-seed value noise, the maximum of the edge weights at
#   corners, half strength so two grounds meet halfway; MART2_DECISIONS D5: a pixel takes the neighbour's texture when
#   weight × strength > a hashed threshold, because Godot's mask blend is binary). Crisp tiles (walls, undug hill)
#   neither blend nor are blended into, so floors stay crisp against walls. The noise is periodic in 256 px, so masks
#   are cached per (side set, 4 × 4 tile phase). Only native Image ops run per chunk (MART2_DECISIONS D4).
# Shores (M-ART2.1 §2.1, MART21_DECISIONS D2): the water outline is the zero level of a signed distance field of the
#   water tiles with every corner rounded — the opening and closing of the tile union by a disc of radius
#   `water.shore_radius_px` (32): a straight shore stays on the tile edge, a 90° step becomes a quarter circle. As the
#   radius is under half a tile, a pixel's field depends only on the 2 × 2 tiles around its tile quadrant's corner, so
#   it is evaluated in closed form (straight lines and rounded quadrants) per (quadrant, 2 × 2 pattern, noise phase)
#   and cached as 32 × 32 masks; the chunk applies them with native blits (a two-tile apron is implicit: every tile
#   reads its eight neighbours from the whole map). Land coverage is smoothstep(−soft, soft, s) dithered against the
#   tile's hashed threshold (craft rule 2: no anti-aliasing), the 10-px bank dithers land into the water side, and a
#   2-px ink stroke on the land side, displaced 0–4 px by the noise at the nearest contour point, follows the same
#   contour. Water tiles are transparent where the field says water (the chunk shader draws the water beneath) and
#   take the neighbouring land's texture where a rounded land corner reaches in. Two water tiles that touch only at a
#   corner stay joined. Undug hill tops get the same rounding against open ground (`hill.edge_radius_px`, crisp, no
#   dither, no ink), except at a corner whose upper row hangs a face (faces stay straight where they hang).
# Canopy (M-ART2.1 §2.2, ¼× only, `canopy_on`): trees are baked into the chunk as the canopy-mass texture (sampled at
#   world offsets, tinted per canopy style — a species in its season — by ground_layer.gd) over the union of tree tiles
#   rounded by `trees.canopy_radius_px`, darkened inside `trees.canopy_edge_px` of the edge; uniform quadrants are one
#   blit. It runs before the shores, so water still carves through it.
#   Inputs are plain arrays (`GroundMap`), so the same bake runs headless in tests; the output is deterministic.
# Bridge methods called: none.
extends RefCounted

const TILE: int = 64
## Ground index of a water tile in GroundMap.ground.
const WATER: int = -1
## Ground index of a tile outside the map.
const VOID: int = -2
const SIDES: Array[Vector2i] = [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 0)]


## The tile data a bake reads: `ground` (index into the baker's grounds, WATER or VOID) and `crisp` (1 = no blending)
## per tile, row-major over `width` × `height`.
class GroundMap:
	extends RefCounted
	var width: int = 0
	var height: int = 0
	var ground: PackedInt32Array = PackedInt32Array()
	var crisp: PackedByteArray = PackedByteArray()
	## 1 = an undug hill top that rounds against open ground (crisp too).
	var rock: PackedByteArray = PackedByteArray()
	## 1 = the tile hangs a face from its south edge.
	var face: PackedByteArray = PackedByteArray()
	## Canopy style + 1 of a tree standing on the tile (0 = no tree); read only by a canopy bake.
	var tree: PackedInt32Array = PackedInt32Array()

	func setup(w: int, h: int) -> void:
		width = w
		height = h
		ground.resize(w * h)
		ground.fill(VOID)
		crisp.resize(w * h)
		crisp.fill(0)
		rock.resize(w * h)
		rock.fill(0)
		face.resize(w * h)
		face.fill(0)
		tree.resize(w * h)
		tree.fill(0)

	func at(x: int, y: int) -> int:
		if x < 0 or y < 0 or x >= width or y >= height:
			return VOID
		return ground[y * width + x]

	func is_crisp(x: int, y: int) -> bool:
		return x >= 0 and y >= 0 and x < width and y < height and crisp[y * width + x] == 1

	func is_rock(x: int, y: int) -> bool:
		return x >= 0 and y >= 0 and x < width and y < height and rock[y * width + x] == 1

	func has_face(x: int, y: int) -> bool:
		return x >= 0 and y >= 0 and x < width and y < height and face[y * width + x] == 1

	func tree_at(x: int, y: int) -> int:
		return tree[y * width + x] if x >= 0 and y >= 0 and x < width and y < height else 0


var chunk_tiles: int = 16
var blend_px: int = 12
var blend_strength: float = 0.5
var bank_px: int = 10
var ink_px: int = 2
var ink_dark: Vector2 = Vector2(0.20, 0.35)
var ink_displace: int = 4
var noise_seed: int = 5005
var noise_period: int = 256
## Shore and hill-edge rounding (render_params `water.shore_radius_px`, `water.contour_soft_px`, `hill.edge_radius_px`).
var shore_radius: float = 32.0
var contour_soft: float = 6.0
var hill_radius: float = TILE * 0.25
## Quadrant masks: key → [land mask, water mask, ink source, ink mask, has ink, any land, any water].
var _shore: Dictionary = {}
## The canopy bake (¼×): on/off for the next bake, rounding radius, darkened edge, and per style the pre-tiled
## canopy image and its darkened copy (`set_canopies`).
var canopy_on: bool = false
var canopy_radius: float = 20.0
var canopy_edge: float = 6.0
var _canopy: Array[Image] = []
var _canopy_edge: Array[Image] = []
var _canopy_period: Array[int] = []
var _clear: Image = null
var _tmp_q: Image = null
## Pre-tiled ground images (period × period source repeated to cover a chunk plus one period), by ground index.
var _tiled: Array[Image] = []
var _period: Array[int] = []
var _masks: Dictionary = {}
var _fields: Dictionary = {}
var _tmp: Image = null


## Configure from render_params.json's `ground`, `water` and `hill` sections (missing keys keep the defaults).
func configure(ground: Dictionary, water: Dictionary, hill: Dictionary = {}, trees: Dictionary = {}) -> void:
	chunk_tiles = int(ground.get("chunk_tiles", chunk_tiles))
	blend_px = int(ground.get("blend_px", blend_px))
	blend_strength = float(ground.get("blend_strength", blend_strength))
	noise_seed = int(ground.get("noise_seed", noise_seed))
	noise_period = maxi(TILE, floori(float(ground.get("noise_period_px", noise_period)) / TILE) * TILE)
	bank_px = int(water.get("bank_px", bank_px))
	ink_px = int(water.get("bank_ink_px", ink_px))
	ink_displace = int(water.get("ink_displace_px", ink_displace))
	var dark: Variant = water.get("bank_ink_dark", null)
	if dark is Array and (dark as Array).size() == 2:
		ink_dark = Vector2(float(dark[0]), float(dark[1]))
	# The rounding stays inside a tile quadrant (the closed form below assumes it), so radii are clamped to it.
	var half: float = float(TILE) * 0.5
	shore_radius = clampf(float(water.get("shore_radius_px", shore_radius)), 0.0, half)
	contour_soft = clampf(float(water.get("contour_soft_px", contour_soft)), 0.0, half * 0.5)
	hill_radius = clampf(float(hill.get("edge_radius_px", hill_radius)), 0.0, half - 1.0)
	canopy_radius = clampf(float(trees.get("canopy_radius_px", canopy_radius)), 0.0, half - 1.0)
	canopy_edge = clampf(float(trees.get("canopy_edge_px", canopy_edge)), 0.0, half - 1.0)
	_masks.clear()
	_fields.clear()
	_shore.clear()


## Set the ground textures, in ground-index order (each an RGBA8 image whose size is its period).
func set_grounds(images: Array[Image]) -> void:
	_tiled.clear()
	_period.clear()
	var span: int = chunk_tiles * TILE
	for src: Image in images:
		var p: int = maxi(1, src.get_width())
		var reps: int = ceili(float(span + p) / float(p))
		var tiled: Image = Image.create(p * reps, p * reps, false, Image.FORMAT_RGBA8)
		for ry: int in reps:
			for rx: int in reps:
				tiled.blit_rect(src, Rect2i(0, 0, p, p), Vector2i(rx * p, ry * p))
		_tiled.append(tiled)
		_period.append(p)
	_tmp = Image.create(TILE, TILE, false, Image.FORMAT_RGBA8)
	_tmp_q = Image.create(TILE >> 1, TILE >> 1, false, Image.FORMAT_RGBA8)
	_clear = Image.create(TILE >> 1, TILE >> 1, false, Image.FORMAT_RGBA8)


## Set the canopy styles (style index = GroundMap.tree − 1): each an RGBA8 image whose size is its period, and its
## darkened copy for the canopy's edge.
func set_canopies(images: Array[Image], edges: Array[Image]) -> void:
	_canopy.clear()
	_canopy_edge.clear()
	_canopy_period.clear()
	var span: int = chunk_tiles * TILE
	for k: int in mini(images.size(), edges.size()):
		var p: int = maxi(1, images[k].get_width())
		var reps: int = ceili(float(span + p) / float(p))
		for pair: Array in [[images[k], _canopy], [edges[k], _canopy_edge]]:
			var tiled: Image = Image.create(p * reps, p * reps, false, Image.FORMAT_RGBA8)
			for ry: int in reps:
				for rx: int in reps:
					tiled.blit_rect(pair[0], Rect2i(0, 0, p, p), Vector2i(rx * p, ry * p))
			(pair[1] as Array).append(tiled)
		_canopy_period.append(p)


## Number of canopy styles set.
func canopy_count() -> int:
	return _canopy.size()


## The pre-tiled image of ground `index` (null when out of range).
func tiled(index: int) -> Image:
	return _tiled[index] if index >= 0 and index < _tiled.size() else null


## The period (source size in px) of ground `index` (0 when out of range).
func period(index: int) -> int:
	return _period[index] if index >= 0 and index < _period.size() else 0


## Number of ground textures set.
func ground_count() -> int:
	return _tiled.size()


## Bake chunk (cx, cy) of `map`.
func bake(map: GroundMap, cx: int, cy: int) -> Image:
	var span: int = chunk_tiles * TILE
	var img: Image = Image.create(span, span, false, Image.FORMAT_RGBA8)
	var x0: int = cx * chunk_tiles
	var y0: int = cy * chunk_tiles
	# 1. Runs of equal ground in each row: one blit per run.
	for ty: int in range(y0, y0 + chunk_tiles):
		var tx: int = x0
		while tx < x0 + chunk_tiles:
			var g: int = map.at(tx, ty)
			var end: int = tx + 1
			while end < x0 + chunk_tiles and map.at(end, ty) == g:
				end += 1
			if g >= 0 and g < _tiled.size():
				var p: int = _period[g]
				var src: Rect2i = Rect2i(posmod(tx * TILE, p), posmod(ty * TILE, p), (end - tx) * TILE, TILE)
				img.blit_rect(_tiled[g], src, Vector2i((tx - x0) * TILE, (ty - y0) * TILE))
			tx = end
	# 2. Edges between land grounds: dithered blends (water and crisp neighbours take no part).
	var per: int = floori(float(noise_period) / TILE)
	for ty: int in range(y0, y0 + chunk_tiles):
		for tx: int in range(x0, x0 + chunk_tiles):
			var g: int = map.at(tx, ty)
			if g < 0 or map.is_crisp(tx, ty):
				continue
			var dst: Vector2i = Vector2i((tx - x0) * TILE, (ty - y0) * TILE)
			var phase: int = posmod(tx, per) + posmod(ty, per) * per
			var groups: Dictionary = {}
			for s: int in 4:
				var nx: int = tx + SIDES[s].x
				var ny: int = ty + SIDES[s].y
				var n: int = map.at(nx, ny)
				if n < 0 or n == g or map.is_crisp(nx, ny):
					continue
				groups[n] = int(groups.get(n, 0)) | (1 << s)
			for n: int in groups.keys():
				if n >= _tiled.size():
					continue
				var mask: Image = _mask(int(groups[n]), phase, false)
				var p: int = _period[n]
				_tmp.blit_rect(_tiled[n], Rect2i(posmod(tx * TILE, p), posmod(ty * TILE, p), TILE, TILE), Vector2i.ZERO)
				img.blend_rect_mask(_tmp, mask, Rect2i(0, 0, TILE, TILE), dst)
	# 3. The canopy (¼× only), then rounded shores and hill edges, per tile quadrant.
	if canopy_on and not _canopy.is_empty():
		_canopy_pass(map, img, x0, y0)
	_round_edges(map, img, x0, y0, per)
	return img


func _hash(hx: int, hy: int) -> float:
	var v: int = (hx * 73856093) ^ (hy * 19349663) ^ noise_seed
	v = ((v ^ (v >> 13)) * 1274126177) & 0xFFFFFFFF
	return float(v & 65535) / 65535.0


## edges.py `_noise` made periodic in noise_period: 4-px bilinear lattice + 0.2 of a per-pixel hash.
func _noise(px: int, py: int) -> float:
	var x: int = posmod(px, noise_period)
	var y: int = posmod(py, noise_period)
	var cells: int = noise_period >> 2
	var gx: int = x >> 2
	var gy: int = y >> 2
	var fx: float = float(x & 3) / 4.0
	var fy: float = float(y & 3) / 4.0
	var a: float = _hash(gx, gy)
	var b: float = _hash((gx + 1) % cells, gy)
	var c: float = _hash(gx, (gy + 1) % cells)
	var d: float = _hash((gx + 1) % cells, (gy + 1) % cells)
	var smooth: float = (1.0 - fy) * ((1.0 - fx) * a + fx * b) + fy * ((1.0 - fx) * c + fx * d)
	return 0.8 * smooth + 0.2 * _hash(x + 7919, y + 104729)


func _threshold(px: int, py: int) -> float:
	return _hash(posmod(px, noise_period) + 31337, posmod(py, noise_period) + 271828)


## The noise and the dither threshold of one tile phase (TILE × TILE values), computed once per phase.
func _phase_fields(phase: int) -> Array:
	if _fields.has(phase):
		return _fields[phase]
	var per_row: int = floori(float(noise_period) / TILE)
	var ox: int = (phase % per_row) * TILE
	var oy: int = floori(float(phase) / per_row) * TILE
	var noise: PackedFloat32Array = PackedFloat32Array()
	var thresh: PackedFloat32Array = PackedFloat32Array()
	noise.resize(TILE * TILE)
	thresh.resize(TILE * TILE)
	for y: int in TILE:
		for x: int in TILE:
			noise[y * TILE + x] = _noise(ox + x, oy + y)
			thresh[y * TILE + x] = _threshold(ox + x, oy + y)
	_fields[phase] = [noise, thresh]
	return _fields[phase]


## The dithered blend mask of the sides `bits` (N=1, E=2, S=4, W=8) for a tile of `phase`. Each side's mask is
## computed once from the phase's fields; a several-sided mask is their union, which equals dithering the maximum of
## the side weights (the threshold is shared: max(w_a, w_b) · s > t ⇔ w_a · s > t or w_b · s > t).
func _mask(bits: int, phase: int, water: bool) -> Image:
	var key: int = bits | (phase << 4) | ((1 if water else 0) << 12)
	if _masks.has(key):
		return _masks[key]
	var mask: Image = null
	for s: int in 4:
		if bits & (1 << s) == 0:
			continue
		var side: Image = _side_mask(s, phase, water)
		if mask == null:
			mask = side.duplicate()
		else:
			mask.blend_rect(side, Rect2i(0, 0, TILE, TILE), Vector2i.ZERO)
	if mask == null:
		mask = Image.create(TILE, TILE, false, Image.FORMAT_RGBA8)
	_masks[key] = mask
	return mask


func _side_mask(side: int, phase: int, water: bool) -> Image:
	var key: int = (1 << side) | (phase << 4) | ((1 if water else 0) << 12) | (1 << 13)
	if _masks.has(key):
		return _masks[key]
	var fields: Array = _phase_fields(phase)
	var noise: PackedFloat32Array = fields[0]
	var thresh: PackedFloat32Array = fields[1]
	var width: int = bank_px if water else blend_px
	var strength: float = 1.0 if water else blend_strength
	var data: PackedByteArray = PackedByteArray()
	data.resize(TILE * TILE * 4)
	for y: int in TILE:
		for x: int in TILE:
			var d: int = y if side == 0 else (TILE - 1 - x if side == 1 else (TILE - 1 - y if side == 2 else x))
			if d >= width:
				continue
			var p: int = y * TILE + x
			var w: float = clampf((float(width - d) - 0.5) / float(width) + (noise[p] - 0.5) * 0.6, 0.0, 1.0)
			if w * strength > thresh[p]:
				var i: int = p * 4
				data[i] = 255
				data[i + 1] = 255
				data[i + 2] = 255
				data[i + 3] = 255
	var mask: Image = Image.create_from_data(TILE, TILE, false, Image.FORMAT_RGBA8, data)
	_masks[key] = mask
	return mask


## Compute every phase's fields and every single-side mask now (a world load), so no bake pays for them later.
## Shore and hill-edge quadrant masks are made on first use (most of the 900 possible keys never occur).
func warm() -> void:
	var phases: int = floori(float(noise_period) / TILE) * floori(float(noise_period) / TILE)
	for phase: int in phases:
		for side: int in 4:
			_side_mask(side, phase, false)


## Number of cached shore and hill-edge quadrant masks (tests, timing).
func shore_mask_count() -> int:
	return _shore.size()


## Shore class: 1 water, 0 natural land including rock, −1 constructed crisp land/outside.
func _water_class(map: GroundMap, x: int, y: int) -> int:
	var g: int = map.at(x, y)
	if g == WATER:
		return 1
	if g < 0 or (map.is_crisp(x, y) and not map.is_rock(x, y)):
		return -1
	return 0


## Hill-edge class of a tile: 1 undug hill top, 0 open land, −1 neither (water, walls, outside the map).
func _rock_class(map: GroundMap, x: int, y: int) -> int:
	var g: int = map.at(x, y)
	if g < 0:
		return -1
	if map.is_rock(x, y):
		return 1
	if map.is_crisp(x, y):
		return -1
	return 0


## The 2 × 2 pattern of a quadrant: bit 0 the tile, 1 its horizontal, 2 its vertical, 3 its diagonal neighbour
## (1 = class 1). A neighbour of neither class counts as the tile's own class, so it neither rounds nor banks.
static func _bits(t: int, h: int, v: int, d: int) -> int:
	h = t if h < 0 else h
	v = t if v < 0 else v
	d = t if d < 0 else d
	return t | (h << 1) | (v << 2) | (d << 3)


## Signed distance of the quarter plane {xq > 0, yq > 0} with its corner rounded by radius r (negative inside).
static func _rounded(xq: float, yq: float, r: float) -> float:
	var a: float = r - xq
	var b: float = r - yq
	return Vector2(maxf(a, 0.0), maxf(b, 0.0)).length() + minf(maxf(a, b), 0.0) - r


## The rounded field of class 1 at (X, Y): X, Y > 0 are the distances from the quadrant's corner lines into the tile
## (the tile is X > 0, Y > 0; its horizontal neighbour X < 0; its vertical neighbour Y < 0). Negative inside class 1.
## Straight boundaries are exact lines; an odd quadrant out is a rounded corner (convex for its class, concave for
## the other); a diagonal pair keeps class 1 joined and rounds the two class-0 corners.
static func shore_field(bits: int, x: float, y: float, r: float) -> float:
	var t: int = bits & 1
	var h: int = (bits >> 1) & 1
	var v: int = (bits >> 2) & 1
	var d: int = (bits >> 3) & 1
	if t == h and v == d and t != v:
		return -y if t == 1 else y
	if t == v and h == d and t != h:
		return -x if t == 1 else x
	if t == d and h == v:
		var s0: float
		if t == 1:
			s0 = minf(_rounded(-x, y, r), _rounded(x, -y, r))
		else:
			s0 = minf(_rounded(x, y, r), _rounded(-x, -y, r))
		return -s0
	var odd: int = 1 if t + h + v + d == 1 else 0
	var dq: float
	if t == odd:
		dq = _rounded(x, y, r)
	elif h == odd:
		dq = _rounded(-x, y, r)
	elif v == odd:
		dq = _rounded(x, -y, r)
	else:
		dq = _rounded(-x, -y, r)
	return dq if odd == 1 else -dq


## Rounded shores (pair 0) and hill edges (pair 1) of the chunk at tile (x0, y0): quadrant masks applied natively.
func _round_edges(map: GroundMap, img: Image, x0: int, y0: int, per: int) -> void:
	var n: int = chunk_tiles + 2
	var wc: PackedInt32Array = PackedInt32Array()
	var rc: PackedInt32Array = PackedInt32Array()
	wc.resize(n * n)
	rc.resize(n * n)
	wc.fill(-1)
	rc.fill(-1)
	var any_w: bool = false
	var any_r: bool = false
	# The same classes as _water_class / _rock_class, read from the packed arrays directly (no call per tile).
	var ground: PackedInt32Array = map.ground
	var crisp: PackedByteArray = map.crisp
	var rock: PackedByteArray = map.rock
	var w: int = map.width
	for ly: int in n:
		var y: int = y0 + ly - 1
		if y < 0 or y >= map.height:
			continue
		for lx: int in n:
			var x: int = x0 + lx - 1
			if x < 0 or x >= w:
				continue
			var i: int = y * w + x
			var at: int = ly * n + lx
			var g: int = ground[i]
			if g == WATER:
				wc[at] = 1
				any_w = true
				continue
			if g < 0:
				continue
			if rock[i] == 1:
				wc[at] = 0
				rc[at] = 1
				any_r = true
			elif crisp[i] == 0:
				wc[at] = 0
				rc[at] = 0
	if not any_w and not any_r:
		return
	var half: int = TILE >> 1
	for ty: int in chunk_tiles:
		for tx: int in chunk_tiles:
			var c: int = (ty + 1) * n + tx + 1
			var x: int = x0 + tx
			var y: int = y0 + ty
			var phase: int = posmod(x, per) + posmod(y, per) * per
			for q: int in 4:
				var sx: int = -1 if (q & 1) == 0 else 1
				var sy: int = -1 if q < 2 else 1
				var cells: Array[int] = [c, c + sx, c + sy * n, c + sx + sy * n]
				var dst: Vector2i = Vector2i(tx * TILE + (0 if sx < 0 else half), ty * TILE + (0 if sy < 0 else half))
				var world: Vector2i = Vector2i(x * TILE + (0 if sx < 0 else half), y * TILE + (0 if sy < 0 else half))
				if any_w and wc[c] >= 0:
					var bits: int = _bits(wc[c], wc[cells[1]], wc[cells[2]], wc[cells[3]])
					if bits != 0 and bits != 15:
						_apply_quadrant(map, img, 0, q, bits, phase, dst, world, x, y, sx, sy)
				if any_r and rc[c] >= 0:
					# Faces stay straight where they hang: skip a corner whose upper row hangs a face.
					var upper: int = y - 1 if sy < 0 else y
					if map.has_face(x, upper) or map.has_face(x + sx, upper):
						continue
					var rbits: int = _bits(rc[c], rc[cells[1]], rc[cells[2]], rc[cells[3]])
					if rbits != 0 and rbits != 15:
						_apply_quadrant(map, img, 1, q, rbits, 0, dst, world, x, y, sx, sy)


## Canopy class of a tile: 1 a tree, 0 open land, −1 neither (water, crisp tiles, outside the map).
func _tree_class(map: GroundMap, x: int, y: int) -> int:
	var g: int = map.at(x, y)
	if g < 0 or map.is_crisp(x, y):
		return -1
	return 1 if map.tree_at(x, y) > 0 else 0


## The canopy over the chunk at tile (x0, y0). Classes and styles are read once with a one-tile apron; a row's run of
## interior tree tiles (all eight neighbours trees) of one style is one blit; the rest go quadrant by quadrant —
## uniform tree quadrants one blit of the tile's style, boundary quadrants the interior and edge masks (pair 2) with
## the tree's style (the tile's, else a neighbour's).
func _canopy_pass(map: GroundMap, img: Image, x0: int, y0: int) -> void:
	var half: int = TILE >> 1
	var rect: Rect2i = Rect2i(0, 0, half, half)
	var n: int = chunk_tiles + 2
	var tc: PackedInt32Array = PackedInt32Array()
	var ts: PackedInt32Array = PackedInt32Array()
	tc.resize(n * n)
	ts.resize(n * n)
	tc.fill(-1)
	var any: bool = false
	# The map's packed arrays read directly (a function call per tile costs as much as the rest of this loop).
	var ground: PackedInt32Array = map.ground
	var crisp: PackedByteArray = map.crisp
	var trees: PackedInt32Array = map.tree
	var w: int = map.width
	for ly: int in n:
		var y: int = y0 + ly - 1
		if y < 0 or y >= map.height:
			continue
		for lx: int in n:
			var x: int = x0 + lx - 1
			if x < 0 or x >= w:
				continue
			var i: int = y * w + x
			var at: int = ly * n + lx
			ts[at] = trees[i]
			if ground[i] < 0 or crisp[i] == 1:
				continue
			tc[at] = 1 if trees[i] > 0 else 0
			any = any or trees[i] > 0
	if not any:
		return
	var nb: Array[int] = [-n - 1, -n, -n + 1, -1, 1, n - 1, n, n + 1]
	for ty: int in chunk_tiles:
		var tx: int = 0
		while tx < chunk_tiles:
			var c: int = (ty + 1) * n + tx + 1
			var t: int = tc[c]
			if t < 0:
				tx += 1
				continue
			# A run of interior tree tiles of one style: one blit.
			var end: int = tx
			while end < chunk_tiles:
				var e: int = (ty + 1) * n + end + 1
				if tc[e] != 1 or ts[e] != ts[c]:
					break
				var inner: bool = true
				for d: int in nb:
					inner = inner and tc[e + d] == 1
				if not inner:
					break
				end += 1
			var style: int = ts[c] - 1
			if end > tx and style >= 0 and style < _canopy.size():
				var p: int = _canopy_period[style]
				img.blit_rect(_canopy[style], Rect2i(posmod((x0 + tx) * TILE, p), posmod((y0 + ty) * TILE, p), (end - tx) * TILE, TILE), Vector2i(tx * TILE, ty * TILE))
				tx = end
				continue
			var x: int = x0 + tx
			var y: int = y0 + ty
			for q: int in 4:
				var sx: int = -1 if (q & 1) == 0 else 1
				var sy: int = -1 if q < 2 else 1
				var ch: int = c + sx
				var cv: int = c + sy * n
				var cd: int = c + sx + sy * n
				var bits: int = _bits(t, tc[ch], tc[cv], tc[cd])
				if bits == 0:
					continue
				var st: int = ts[c]
				for o: int in [ch, cv, cd]:
					if st <= 0:
						st = ts[o]
				st -= 1
				if st < 0 or st >= _canopy.size():
					continue
				var p: int = _canopy_period[st]
				var world: Vector2i = Vector2i(x * TILE + (0 if sx < 0 else half), y * TILE + (0 if sy < 0 else half))
				var dst: Vector2i = Vector2i(tx * TILE + (0 if sx < 0 else half), ty * TILE + (0 if sy < 0 else half))
				var src: Rect2i = Rect2i(posmod(world.x, p), posmod(world.y, p), half, half)
				if bits == 15:
					img.blit_rect(_canopy[st], src, dst)
					continue
				var m: Array = _quadrant_masks(2, q, bits, 0)
				if bool(m[6]):
					_tmp_q.blit_rect(_canopy[st], src, Vector2i.ZERO)
					img.blend_rect_mask(_tmp_q, m[1], rect, dst)
				if bool(m[5]):
					_tmp_q.blit_rect(_canopy_edge[st], src, Vector2i.ZERO)
					img.blend_rect_mask(_tmp_q, m[0], rect, dst)
			tx += 1


## The rounded field the bake uses at world pixel (wx, wy) — pair 0 shores (negative in water), pair 1 hill edges
## (negative on the hill top) — or INF where the pixel's quadrant has no boundary (tests measure the contour).
func shore_distance(map: GroundMap, wx: int, wy: int, pair: int = 0) -> float:
	var x: int = floori(float(wx) / TILE)
	var y: int = floori(float(wy) / TILE)
	var px: int = wx - x * TILE
	var py: int = wy - y * TILE
	var half: int = TILE >> 1
	var sx: int = -1 if px < half else 1
	var sy: int = -1 if py < half else 1
	var cls: Callable = _water_class if pair == 0 else _rock_class
	var t: int = cls.call(map, x, y)
	if t < 0:
		return INF
	var bits: int = _bits(t, cls.call(map, x + sx, y), cls.call(map, x, y + sy), cls.call(map, x + sx, y + sy))
	if bits == 0 or bits == 15:
		return INF
	var lx: float = (float(px) + 0.5) if sx < 0 else (float(TILE - px) - 0.5)
	var ly: float = (float(py) + 0.5) if sy < 0 else (float(TILE - py) - 0.5)
	return shore_field(bits, lx, ly, shore_radius if pair == 0 else hill_radius)


## One quadrant: the other class's pixels become transparent (a land tile's water) or take the neighbour's texture
## (a water tile's land, a hill top's open ground, open ground's hill top); then the shore's ink.
func _apply_quadrant(map: GroundMap, img: Image, pair: int, q: int, bits: int, phase: int, dst: Vector2i, world: Vector2i, x: int, y: int, sx: int, sy: int) -> void:
	var m: Array = _quadrant_masks(pair, q, bits, phase)
	var own: int = bits & 1
	var other: int = 1 - own
	var rect: Rect2i = Rect2i(0, 0, TILE >> 1, TILE >> 1)
	var mask: Image = m[1] if other == 1 else m[0]
	var present: bool = bool(m[6]) if other == 1 else bool(m[5])
	if present:
		if pair == 0 and other == 1:
			img.blit_rect_mask(_clear, mask, rect, dst)
		else:
			var g: int = -1
			for nb: Vector2i in [Vector2i(x + sx, y), Vector2i(x, y + sy), Vector2i(x + sx, y + sy)]:
				var k: int = _water_class(map, nb.x, nb.y) if pair == 0 else _rock_class(map, nb.x, nb.y)
				if k == other:
					g = map.at(nb.x, nb.y)
					break
			if g >= 0 and g < _tiled.size():
				var p: int = _period[g]
				_tmp_q.blit_rect(_tiled[g], Rect2i(posmod(world.x, p), posmod(world.y, p), TILE >> 1, TILE >> 1), Vector2i.ZERO)
				img.blend_rect_mask(_tmp_q, mask, rect, dst)
	if bool(m[4]):
		img.blend_rect_mask(m[2], m[3], rect, dst)


## [class-0 mask, class-1 mask, ink source, ink mask, has ink, any class 0, any class 1] of one quadrant (32 × 32).
## Pair 2 (canopy) returns the edge band as "class 0" and the interior as "class 1"; pixels outside the canopy are in
## neither.
## Pair 0 (shores) dithers: land where smoothstep(−soft, soft, s) > the tile's threshold, or inside the 10-px bank
## (the M-ART2 bank weight, measured from the rounded contour); ink where the land-side distance lies in
## [disp, disp + ink_px), disp = the noise at the nearest contour point × ink_displace_px. Pair 1 (hills) is crisp.
func _quadrant_masks(pair: int, q: int, bits: int, phase: int) -> Array:
	# pair (0–2) takes two bits, the quadrant two, the pattern four; the phase above them (MART21 D7: a one-bit pair
	# made canopy and shore masks collide).
	var key: int = pair | (q << 2) | (bits << 4) | (phase << 8)
	if _shore.has(key):
		return _shore[key]
	var half: int = TILE >> 1
	var r: float = shore_radius if pair == 0 else (hill_radius if pair == 1 else canopy_radius)
	var sx: int = -1 if (q & 1) == 0 else 1
	var sy: int = -1 if q < 2 else 1
	var ox: int = 0 if sx < 0 else half
	var oy: int = 0 if sy < 0 else half
	var per_row: int = floori(float(noise_period) / TILE)
	var nox: int = (phase % per_row) * TILE
	var noy: int = floori(float(phase) / per_row) * TILE
	var noise: PackedFloat32Array = PackedFloat32Array()
	var thresh: PackedFloat32Array = PackedFloat32Array()
	if pair == 0:
		var fields: Array = _phase_fields(phase)
		noise = fields[0]
		thresh = fields[1]
	var m0: PackedByteArray = PackedByteArray()
	var m1: PackedByteArray = PackedByteArray()
	var isrc: PackedByteArray = PackedByteArray()
	var imask: PackedByteArray = PackedByteArray()
	m0.resize(half * half * 4)
	m1.resize(half * half * 4)
	isrc.resize(half * half * 4)
	imask.resize(half * half * 4)
	var has_ink: bool = false
	var any0: bool = false
	var any1: bool = false
	var dirx: float = 1.0 if sx < 0 else -1.0
	var diry: float = 1.0 if sy < 0 else -1.0
	for qy: int in half:
		for qx: int in half:
			var px: int = ox + qx
			var py: int = oy + qy
			var lx: float = (float(px) + 0.5) if sx < 0 else (float(TILE - px) - 0.5)
			var ly: float = (float(py) + 0.5) if sy < 0 else (float(TILE - py) - 0.5)
			var s1: float = shore_field(bits, lx, ly, r)
			var is1: bool
			var ink: float = 0.0
			if pair == 1:
				is1 = s1 < 0.0
			elif pair == 2:
				# Canopy: class 1 = the interior, class 0 = the darkened edge band; outside both, neither mask.
				if s1 >= 0.0:
					continue
				is1 = s1 < -canopy_edge
			else:
				# The field is negative inside water, so it is the land-side distance itself.
				var s: float = s1
				var p: int = py * TILE + px
				var t: float = thresh[p]
				var land: bool = (smoothstep(-contour_soft, contour_soft, s) > t) if contour_soft > 0.0 else s > 0.0
				if not land and s < 0.0 and -s < float(bank_px):
					land = clampf((float(bank_px) + s) / float(bank_px) + (noise[p] - 0.5) * 0.6, 0.0, 1.0) > t
				if s >= 0.0 and s < float(ink_displace + ink_px):
					# The stroke's displacement is the noise at the nearest contour point (s along −∇s).
					var gx: float = (shore_field(bits, lx + 0.5, ly, r) - shore_field(bits, lx - 0.5, ly, r))
					var gy: float = (shore_field(bits, lx, ly + 0.5, r) - shore_field(bits, lx, ly - 0.5, r))
					var g: Vector2 = Vector2(gx * dirx, gy * diry)
					if g.length() > 0.0:
						g = g.normalized()
					var cx: int = floori(float(px) + 0.5 - s * g.x)
					var cy: int = floori(float(py) + 0.5 - s * g.y)
					var disp: int = int(roundf(_noise(nox + cx, noy + cy) * ink_displace))
					var dpx: int = floori(s)
					if dpx >= disp and dpx < disp + ink_px:
						ink = ink_dark.x + (ink_dark.y - ink_dark.x) * noise[p]
						land = true
				is1 = not land
			var i: int = (qy * half + qx) * 4
			# Packed arrays are values in GDScript: write each buffer by its own name.
			if is1:
				m1[i] = 255
				m1[i + 3] = 255
			else:
				m0[i] = 255
				m0[i + 3] = 255
			any1 = any1 or is1
			any0 = any0 or not is1
			if ink > 0.0:
				isrc[i + 3] = int(roundf(ink * 255.0))
				imask[i] = 255
				imask[i + 3] = 255
				has_ink = true
	var out: Array = [Image.create_from_data(half, half, false, Image.FORMAT_RGBA8, m0),
		Image.create_from_data(half, half, false, Image.FORMAT_RGBA8, m1),
		Image.create_from_data(half, half, false, Image.FORMAT_RGBA8, isrc),
		Image.create_from_data(half, half, false, Image.FORMAT_RGBA8, imask), has_ink, any0, any1]
	_shore[key] = out
	return out
