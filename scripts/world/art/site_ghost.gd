# site_ghost.gd — what a blueprint looks like when finished, and its pale "ghost" (M-ART2 §3.4 zones/sites, §3.6).
# Purpose: one place that turns a BlueprintKind code (bindings.json `blueprints`) into the finished sprite — a
#   workshop's structure, the bedroll, a wall / door / floor resolved through the material system (material_resolver.gd
#   and wall_baker.gd, the same assembly the ground layer bakes), a hut's ring of walls (bindings `buildings` row
#   `ring`) — as an Image with its base point (the world point of the footprint's bottom centre: x = left·64 + w·32,
#   y = (top + h − 1)·64 + 43), and the ghost of any image: a copy with alpha × `sites.wash_alpha` plus a 1-px
#   outline in `sites.outline_rgb` on the transparent pixels next to the silhouette (render_params.json). Ghosts are
#   cached per key (a 64-px tile crop is keyed by its pixel data, so a run of wall sites reuses a handful of
#   textures). Used by the overlay (blueprint sites), the order tool (placement previews) and the order bar (build
#   menu icons). No content is named here; every id comes from bindings.json and the material documents.
# Bridge methods called: none.
extends RefCounted

const ArtData = preload("res://scripts/world/art/art_data.gd")
const MaterialResolver = preload("res://scripts/world/art/material_resolver.gd")
const WallBaker = preload("res://scripts/world/art/wall_baker.gd")
const Tone = preload("res://scripts/world/art/tone.gd")
const TILE: int = 64
const BASE_Y: int = 43
## Wall strips and doors stand with their image top CAP px above the tile (wall_baker.gd).
const CAP: int = WallBaker.CAP
## Alpha at or above which a pixel belongs to the silhouette.
const SOLID: int = 96

static var _resolver: MaterialResolver = null
static var _params_version: int = -1
static var _ghosts: Dictionary = {}
static var _finished: Dictionary = {}
static var _sets: Dictionary = {}
static var _doors: Dictionary = {}
static var _floors: Dictionary = {}


static func _fresh() -> void:
	if _params_version != ArtData.params_version():
		_params_version = ArtData.params_version()
		_ghosts.clear()
		_finished.clear()
		_sets.clear()
		_doors.clear()
		_floors.clear()


## The shared material resolver (loaded once from the imported documents).
static func resolver() -> MaterialResolver:
	if _resolver == null:
		_resolver = MaterialResolver.new()
		_resolver.load_imported()
	return _resolver


## The pale ghost of `src`: alpha × wash, a 1-px outline where the silhouette ends (transparent side).
static func ghost_image(src: Image) -> Image:
	var img: Image = src.duplicate()
	if img.is_compressed():
		img.decompress()
	img.convert(Image.FORMAT_RGBA8)
	var w: int = img.get_width()
	var h: int = img.get_height()
	var wash: float = clampf(float(ArtData.param("sites.wash_alpha", 0.25)), 0.0, 1.0)
	var rgb: Variant = ArtData.param("sites.outline_rgb", [218, 222, 205])
	var outline_a: int = clampi(roundi(float(ArtData.param("sites.outline_alpha", 0.9)) * 255.0), 0, 255)
	var orr: int = int(rgb[0]) if rgb is Array and (rgb as Array).size() == 3 else 218
	var og: int = int(rgb[1]) if rgb is Array and (rgb as Array).size() == 3 else 222
	var ob: int = int(rgb[2]) if rgb is Array and (rgb as Array).size() == 3 else 205
	var data: PackedByteArray = img.get_data()
	var out: PackedByteArray = data.duplicate()
	for y: int in h:
		var row: int = y * w
		for x: int in w:
			var i: int = (row + x) * 4
			var a: int = data[i + 3]
			if a >= SOLID:
				out[i + 3] = roundi(a * wash)
				continue
			var edge: bool = (x > 0 and data[i - 1] >= SOLID) or (x + 1 < w and data[i + 7] >= SOLID) \
				or (y > 0 and data[i - w * 4 + 3] >= SOLID) or (y + 1 < h and data[i + w * 4 + 3] >= SOLID)
			if edge:
				out[i] = orr
				out[i + 1] = og
				out[i + 2] = ob
				out[i + 3] = outline_a
			else:
				out[i + 3] = roundi(a * wash)
	return Image.create_from_data(w, h, false, Image.FORMAT_RGBA8, out)


## The ghost texture of `src`, cached under `key` (an id, or "" to key it by the image's pixels).
static func ghost(key: String, src: Image) -> Texture2D:
	_fresh()
	if src == null:
		return null
	var k: String = key if key != "" else "px:%d:%d:%d" % [src.get_width(), src.get_height(), hash(src.get_data())]
	if not _ghosts.has(k):
		_ghosts[k] = ImageTexture.create_from_image(ghost_image(src))
	return _ghosts[k]


## The bindings row of blueprint `code` (empty when unknown).
static func blueprint(code: int) -> Dictionary:
	return ArtData.binding("blueprints", code)


## The finished look of blueprint `code`: {image, base (Vector2i in the image), footprint (Vector2i), flat (bool)};
## {} when the code has nothing to draw.
static func finished(code: int) -> Dictionary:
	_fresh()
	if _finished.has(code):
		return _finished[code]
	var row: Dictionary = blueprint(code)
	var out: Dictionary = {}
	if row.has("workshop"):
		var id: Variant = ArtData.binding("workshops", int(row["workshop"])).get("structure", null)
		out = _structure(str(id)) if id is String else {}
	elif bool(row.get("bed", false)):
		var bed: Dictionary = (ArtData.bindings().get("space", {}) as Dictionary).get("bed", {})
		out = _structure(str(bed.get("structure", "")))
		var fp: Variant = bed.get("footprint", null)
		if not out.is_empty() and fp is Array and (fp as Array).size() == 2:
			out["footprint"] = Vector2i(int(fp[0]), int(fp[1]))
		out["flat"] = true
	elif row.has("style"):
		var style: String = str(row["style"])
		match str(row.get("kind", "")):
			"wall":
				var wall: Image = wall_tile(wall_rows({Vector2i.ZERO: style}, {}), Vector2i.ZERO)
				if wall != null:
					out = {"image": wall, "base": Vector2i(TILE >> 1, CAP + BASE_Y), "footprint": Vector2i.ONE, "flat": false}
			"door":
				var door: Image = door_image(style, false)
				if door != null:
					out = {"image": door, "base": Vector2i(TILE >> 1, CAP + BASE_Y), "footprint": Vector2i.ONE, "flat": false}
			_:
				var fl: Image = floor_tile(style, Vector2i.ZERO)
				if fl != null:
					out = {"image": fl, "base": Vector2i(TILE >> 1, BASE_Y), "footprint": Vector2i.ONE, "flat": true}
	elif row.has("building"):
		var ring: Dictionary = ArtData.binding("buildings", int(row["building"])).get("ring", {})
		if not ring.is_empty():
			out = ring_image(ring)
	_finished[code] = out
	return out


static func _structure(id: String) -> Dictionary:
	var img: Image = ArtData.image(id)
	if img == null:
		return {}
	return {"image": img, "base": ArtData.anchor(id), "footprint": ArtData.footprint(id), "flat": false}


## A prepared wall texture set for blueprint `style` (no material: the style's own look).
static func _wall_set(style: String) -> Dictionary:
	if _sets.has(style):
		return _sets[style]
	var r: Dictionary = resolver().resolve(style, "", "", "wall")
	var probs: Array[String] = []
	var s: Dictionary = WallBaker.prepare_set(r, ArtData.param("walls", {}), probs)
	_sets[style] = s
	return s


## Walls of `cells` (tile → blueprint style) with door apertures `doors` (tile → style), assembled by wall_baker.gd:
## {row (tile y): {image: strip 64 px tall whose top is 16 px above the row, x0: the tile x of its left edge}}.
## Tiles may be anywhere; the bake runs in a local frame aligned to 4 tiles so textures keep their world phase.
static func wall_rows(cells: Dictionary, doors: Dictionary) -> Dictionary:
	if cells.is_empty():
		return {}
	var all: Dictionary = cells.duplicate()
	all.merge(doors)
	var lo: Vector2i = all.keys()[0]
	var hi: Vector2i = lo
	for t: Vector2i in all.keys():
		lo = Vector2i(mini(lo.x, t.x), mini(lo.y, t.y))
		hi = Vector2i(maxi(hi.x, t.x), maxi(hi.y, t.y))
	var origin: Vector2i = Vector2i(floori(lo.x / 4.0) * 4, floori(lo.y / 4.0) * 4)
	var baker: WallBaker = WallBaker.new()
	baker.chunk_tiles = maxi(hi.x - origin.x, hi.y - origin.y) + 1
	var ink: Variant = ArtData.param("walls.ink_rgba", [20, 18, 16, 204])
	if ink is Array and (ink as Array).size() == 4:
		baker.ink = Color8(int(ink[0]), int(ink[1]), int(ink[2]), int(ink[3]))
	for t: Vector2i in all.keys():
		var lt: Vector2i = t - origin
		baker.occupancy[lt] = true
		if doors.has(t):
			baker.openings[lt] = true
			continue
		var style: String = str(cells[t])
		var s: Dictionary = _wall_set(style)
		if s.is_empty():
			continue
		baker.sets[style] = s
		baker.cell_set[lt] = style
	var local: Dictionary = baker.bake(0, 0)
	var out: Dictionary = {}
	for ty: int in local.keys():
		out[ty + origin.y] = {"image": local[ty], "x0": origin.x}
	return out


## The wall image (64 × 64, top 16 px above the tile) of tile `t` from `wall_rows` output, or null.
static func wall_tile(rows: Dictionary, t: Vector2i) -> Image:
	if not rows.has(t.y):
		return null
	var r: Dictionary = rows[t.y]
	var img: Image = r["image"]
	var lx: int = (t.x - int(r["x0"])) * TILE
	if lx < 0 or lx + TILE > img.get_width():
		return null
	return img.get_region(Rect2i(lx, 0, TILE, TILE))


## The door image (64 × 60, its top 16 px above the tile) of door blueprint `style`.
static func door_image(style: String, open: bool) -> Image:
	_fresh()
	var key: String = "%s|%s" % [style, str(open)]
	if _doors.has(key):
		return _doors[key]
	var r: Dictionary = resolver().resolve(style, "", "", "door")
	var tex: Dictionary = r.get("textures", {})
	var tint: Dictionary = r.get("tint", {})
	# Texture roles of the material schema (as ground_layer.gd reads them), not content ids.
	var img: Image = ArtData.image_at(MaterialResolver.res_path(str(tex.get("door_open" if open else "door_closed", ""))))
	if img != null:
		img = Tone.lab_shift(img, float(tint.get("dL", 0)), float(tint.get("da", 0)), float(tint.get("db", 0)))
	_doors[key] = img
	return img


## The 64 × 64 floor image of floor blueprint `style` on tile `t` (sampled at world offsets so floors continue).
static func floor_tile(style: String, t: Vector2i) -> Image:
	_fresh()
	var full: Image = _floors.get(style, null)
	if full == null and not _floors.has(style):
		var r: Dictionary = resolver().resolve(style, "", "", "floor")
		var tex: Dictionary = r.get("textures", {})
		var tint: Dictionary = r.get("tint", {})
		full = ArtData.image_at(MaterialResolver.res_path(str(tex.get("floor", ""))))
		if full != null:
			full = Tone.lab_shift(full, float(tint.get("dL", 0)), float(tint.get("da", 0)), float(tint.get("db", 0)))
		_floors[style] = full
	if full == null:
		return null
	var w: int = full.get_width()
	var h: int = full.get_height()
	var sx: int = posmod(t.x * TILE, w) if w >= 2 * TILE else 0
	var sy: int = posmod(t.y * TILE, h) if h >= 2 * TILE else 0
	return full.get_region(Rect2i(mini(sx, w - mini(TILE, w)), mini(sy, h - mini(TILE, h)), mini(TILE, w), mini(TILE, h)))


## A hut ring (bindings `buildings` row `ring`: size, wall_style, floor_style, door, door_style) in local tiles:
## {walls: {tile: style}, doors: {tile: style}, floors: Array[Vector2i], size: Vector2i}.
static func ring_tiles(ring: Dictionary) -> Dictionary:
	var size_v: Variant = ring.get("size", [3, 3])
	var size: Vector2i = Vector2i(int(size_v[0]), int(size_v[1])) if size_v is Array and (size_v as Array).size() == 2 else Vector2i(3, 3)
	var door_v: Variant = ring.get("door", [])
	var door: Vector2i = Vector2i(int(door_v[0]), int(door_v[1])) if door_v is Array and (door_v as Array).size() == 2 else Vector2i(-1, -1)
	var door_styles: Array = (ArtData.bindings().get("space", {}) as Dictionary).get("door_styles", [])
	var door_style: String = str(ring.get("door_style", door_styles[0] if not door_styles.is_empty() else ""))
	var walls: Dictionary = {}
	var doors: Dictionary = {}
	var floors: Array[Vector2i] = []
	for y: int in size.y:
		for x: int in size.x:
			var t: Vector2i = Vector2i(x, y)
			if x == 0 or y == 0 or x == size.x - 1 or y == size.y - 1:
				if t == door:
					doors[t] = door_style
				else:
					walls[t] = str(ring.get("wall_style", ""))
			else:
				floors.append(t)
	return {"walls": walls, "doors": doors, "floors": floors, "size": size}


## The whole finished ring as one image (size.x·64 × size.y·64 + 16; walls, the door, the floor) with its base.
static func ring_image(ring: Dictionary) -> Dictionary:
	var rt: Dictionary = ring_tiles(ring)
	var size: Vector2i = rt["size"]
	var img: Image = Image.create(size.x * TILE, size.y * TILE + CAP, false, Image.FORMAT_RGBA8)
	for t: Vector2i in rt["floors"]:
		var fl: Image = floor_tile(str(ring.get("floor_style", "")), t)
		if fl != null:
			img.blend_rect(fl, Rect2i(Vector2i.ZERO, fl.get_size()), Vector2i(t.x * TILE, t.y * TILE + CAP))
	var rows: Dictionary = wall_rows(rt["walls"], rt["doors"])
	var ys: Array = rows.keys()
	ys.sort()
	for ty: int in ys:
		var r: Dictionary = rows[ty]
		var strip: Image = r["image"]
		img.blend_rect(strip, Rect2i(Vector2i.ZERO, strip.get_size()), Vector2i(int(r["x0"]) * TILE, ty * TILE))
	for t: Vector2i in (rt["doors"] as Dictionary).keys():
		var d: Image = door_image(str(rt["doors"][t]), false)
		if d != null:
			img.blend_rect(d, Rect2i(Vector2i.ZERO, d.get_size()), Vector2i(t.x * TILE, t.y * TILE))
	return {"image": img, "base": Vector2i(size.x * (TILE >> 1), (size.y - 1) * TILE + BASE_Y + CAP), "footprint": size, "flat": false}


## The world base point of a footprint (top-left `left_top`, size `size` tiles): its bottom centre on the base line.
static func footprint_base(left_top: Vector2i, size: Vector2i) -> Vector2:
	return Vector2(left_top.x * TILE + size.x * (TILE >> 1), (left_top.y + size.y - 1) * TILE + BASE_Y)
