# ground_baker.gd — bakes one ground chunk (16 × 16 tiles = 1,024 × 1,024 world px, RGBA8) on the CPU (ADR 0034).
# Purpose: the top-down layer of the one projection (ART-STRUCT): per tile the ground texture sampled with world
#   offsets (textures pre-tiled once so a run of equal tiles in a row is one native blit), the noise-dithered 12-px
#   blend between different grounds (edges.py `render_tile`: fixed-seed value noise, the maximum of the edge weights at
#   corners, half strength so two grounds meet halfway; MART2_DECISIONS D5: a pixel takes the neighbour's texture when
#   weight × strength > a hashed threshold, because Godot's mask blend is binary), water tiles left transparent except
#   the 10-px dithered bank where land erodes into the water (the chunk's shader draws the water texture beneath), a
#   2-px ink stroke on the land side of a bank, displaced 0–4 px by the noise (edges.py `_bank_ink`). Crisp tiles (walls,
#   undug hill) neither blend nor are blended into, so floors stay crisp against walls. The noise is periodic in 256 px,
#   so masks are cached per (side set, 4 × 4 tile phase). Only native Image ops run per chunk (MART2_DECISIONS D4).
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

	func setup(w: int, h: int) -> void:
		width = w
		height = h
		ground.resize(w * h)
		ground.fill(VOID)
		crisp.resize(w * h)
		crisp.fill(0)

	func at(x: int, y: int) -> int:
		if x < 0 or y < 0 or x >= width or y >= height:
			return VOID
		return ground[y * width + x]

	func is_crisp(x: int, y: int) -> bool:
		return x >= 0 and y >= 0 and x < width and y < height and crisp[y * width + x] == 1


var chunk_tiles: int = 16
var blend_px: int = 12
var blend_strength: float = 0.5
var bank_px: int = 10
var ink_px: int = 2
var ink_dark: Vector2 = Vector2(0.20, 0.35)
var ink_displace: int = 4
var noise_seed: int = 5005
var noise_period: int = 256
## Pre-tiled ground images (period × period source repeated to cover a chunk plus one period), by ground index.
var _tiled: Array[Image] = []
var _period: Array[int] = []
var _masks: Dictionary = {}
var _ink: Dictionary = {}
var _fields: Dictionary = {}
var _tmp: Image = null


## Configure from render_params.json's `ground` and `water` sections (missing keys keep the defaults).
func configure(ground: Dictionary, water: Dictionary) -> void:
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
	_masks.clear()
	_ink.clear()
	_fields.clear()


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
	# 2. Edges: dithered blends between grounds, banks into water, ink on the land side.
	for ty: int in range(y0, y0 + chunk_tiles):
		for tx: int in range(x0, x0 + chunk_tiles):
			var g: int = map.at(tx, ty)
			if g == VOID or map.is_crisp(tx, ty):
				continue
			var dst: Vector2i = Vector2i((tx - x0) * TILE, (ty - y0) * TILE)
			var per: int = floori(float(noise_period) / TILE)
			var phase: int = posmod(tx, per) + posmod(ty, per) * per
			var groups: Dictionary = {}
			var ink_bits: int = 0
			for s: int in 4:
				var nx: int = tx + SIDES[s].x
				var ny: int = ty + SIDES[s].y
				var n: int = map.at(nx, ny)
				if n == VOID or n == g:
					continue
				if n == WATER:
					if not map.is_crisp(nx, ny):
						ink_bits |= 1 << s
					continue
				if map.is_crisp(nx, ny):
					continue
				groups[n] = int(groups.get(n, 0)) | (1 << s)
			for n: int in groups.keys():
				if n < 0 or n >= _tiled.size():
					continue
				var water: bool = g == WATER
				var mask: Image = _mask(int(groups[n]), phase, water)
				var p: int = _period[n]
				_tmp.blit_rect(_tiled[n], Rect2i(posmod(tx * TILE, p), posmod(ty * TILE, p), TILE, TILE), Vector2i.ZERO)
				img.blend_rect_mask(_tmp, mask, Rect2i(0, 0, TILE, TILE), dst)
			if ink_bits != 0 and g >= 0:
				var pair: Array = _ink_pair(ink_bits, phase)
				img.blend_rect_mask(pair[0], pair[1], Rect2i(0, 0, TILE, TILE), dst)
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
func warm() -> void:
	var phases: int = floori(float(noise_period) / TILE) * floori(float(noise_period) / TILE)
	for phase: int in phases:
		for side: int in 4:
			_side_mask(side, phase, false)
			_side_mask(side, phase, true)


## [ink source (black with the stroke's alpha), stroke mask] for the water sides `bits` of a land tile.
func _ink_pair(bits: int, phase: int) -> Array:
	var key: int = bits | (phase << 4)
	if _ink.has(key):
		return _ink[key]
	var noise: PackedFloat32Array = _phase_fields(phase)[0]
	var src: PackedByteArray = PackedByteArray()
	src.resize(TILE * TILE * 4)
	var mask: PackedByteArray = PackedByteArray()
	mask.resize(TILE * TILE * 4)
	for y: int in TILE:
		for x: int in TILE:
			var strength: float = 0.0
			for s: int in 4:
				if bits & (1 << s) == 0:
					continue
				var d: int = y if s == 0 else (TILE - 1 - x if s == 1 else (TILE - 1 - y if s == 2 else x))
				if d > ink_displace + ink_px:
					continue
				# The displacement follows the noise along the edge (edges.py: the noise row/column at that edge).
				var along: float = noise[x] if s == 0 else (noise[y * TILE + TILE - 1] if s == 1 else (noise[(TILE - 1) * TILE + x] if s == 2 else noise[y * TILE]))
				var disp: int = int(roundf(along * ink_displace))
				if d >= disp and d < disp + ink_px:
					strength = maxf(strength, ink_dark.x + (ink_dark.y - ink_dark.x) * noise[y * TILE + x])
			var i: int = (y * TILE + x) * 4
			if strength > 0.0:
				src[i + 3] = int(roundf(strength * 255.0))
				mask[i + 3] = 255
				mask[i] = 255
	var pair: Array = [Image.create_from_data(TILE, TILE, false, Image.FORMAT_RGBA8, src), Image.create_from_data(TILE, TILE, false, Image.FORMAT_RGBA8, mask)]
	_ink[key] = pair
	return pair
