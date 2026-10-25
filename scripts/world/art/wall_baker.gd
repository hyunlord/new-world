# wall_baker.gd — assembles walls and hill faces of one chunk into per-row strips (ADR 0034; port of walls.py).
# Purpose: the material system's wall assembly with the owner's top option (b) (art/compositor/walls.py at
#   v8/art-polish2 c8ed8608, `_material_tiles`): per wall tile the top runs along the wall — the E–W strip sampled at
#   world x for an east–west run, the independently drawn N–S strip at world y for a north–south run, the junction
#   piece for corners, T, cross and isolated tiles — full width (width 64), 16 px tall, or 64 px when the south
#   neighbour is a wall so north–south tops join; ink (2 px) on every top edge whose neighbour is not a wall; the
#   44-px face only where the south neighbour is not a wall and the tile is not a door; end pieces only at face ends.
#   Hill faces by the same rule (STRUCT `hill_tiles`): an undug blocking tile whose south neighbour is passable draws
#   the hill ground as its 16-px cap and the rock face beneath, sampled at world x. Doors are not baked (they stand
#   as their own sprites). Connectivity reads the whole occupancy (walls and doors of every material), never one
#   material's subset. Texture sets come from `prepare_set` (resolved by material_resolver.gd, tinted by the
#   material's Lab shift, the top toned by value / saturation / blur); no style or material is named here.
#   A strip covers world rows [row_top − 16, row_top + 48) and stands at base y = row_top + 43 in the y-sorted layer.
# Bridge methods called: none.
extends RefCounted

const Tone = preload("res://scripts/world/art/tone.gd")
const MaterialResolver = preload("res://scripts/world/art/material_resolver.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const TILE: int = 64
const CAP: int = 16
const FACE: int = 44
## Strip height: a joined top (64) is the tallest piece.
const STRIP: int = 64
const SIZES: Dictionary = {"face": Vector2i(256, 44), "top_ew": Vector2i(256, 64), "top_ns": Vector2i(64, 256),
	"end": Vector2i(24, 44), "junction": Vector2i(64, 64), "door_closed": Vector2i(64, 60), "door_open": Vector2i(64, 60)}

var chunk_tiles: int = 16
var ink: Color = Color8(20, 18, 16, 204)
var ink_px: int = 2
## Tile (Vector2i) → true for every wall and door tile (connectivity).
var occupancy: Dictionary = {}
## Door tiles (apertures: no face, no top).
var openings: Dictionary = {}
## Wall tile → texture-set key; `sets[key]` holds its prepared images.
var cell_set: Dictionary = {}
var sets: Dictionary = {}
## Hill tiles that draw a face (undug, blocking, passable south neighbour).
var faces: Dictionary = {}
## The hill cap source (the hill ground texture, any size, sampled with world offsets) and the rock face (256 × 44).
var hill_cap: Image = null
var rock_face: Image = null
var _ink_img: Image = null


## Prepare one texture set: the resolved textures (material_resolver.gd `resolve` result) loaded from their imports,
## Lab-tinted, the tops toned. `top_override` (render_params.json `walls`) replaces the style's value / saturation /
## blur. Returns {} and lists problems in `out_problems` when a texture is missing or of the wrong size.
static func prepare_set(resolved: Dictionary, top_override: Dictionary, out_problems: Array[String]) -> Dictionary:
	var tex: Dictionary = resolved.get("textures", {})
	var tint: Dictionary = resolved.get("tint", {})
	var top: Dictionary = resolved.get("top", {})
	var value: float = float(top_override.get("top_value", top.get("value", 0.75)))
	var saturation: float = float(top_override.get("top_saturation", top.get("saturation", 0.85)))
	var blur: float = float(top_override.get("top_blur_px", top.get("blur_px", 2)))
	var out: Dictionary = {}
	for role: String in SIZES.keys():
		var art_path: String = str(tex.get(role, ""))
		var img: Image = ArtData.image_at(MaterialResolver.res_path(art_path))
		if img == null:
			out_problems.append("missing %s texture %s" % [role, art_path])
			return {}
		if img.get_size() != SIZES[role]:
			out_problems.append("%s is %s, expected %s" % [art_path, img.get_size(), SIZES[role]])
			return {}
		var shifted: Image = Tone.lab_shift(img, float(tint.get("dL", 0)), float(tint.get("da", 0)), float(tint.get("db", 0)))
		if role.begins_with("top_") or role == "junction":
			shifted = Tone.top_tone(shifted, value, saturation, blur)
		out[role] = shifted
	return out


## Exact prepared images shared only inside this baker; callers draw but do not mutate these images.
var _prepared_sets: Dictionary = {}
var prepared_misses: int = 0

func clear_prepared_sets() -> void:
	_prepared_sets.clear()
	prepared_misses = 0

func prepared_set_count() -> int:
	return _prepared_sets.size()

## The cache key includes effective pixel inputs, not material names. Image identity changes on ArtData.clear().
static func _preparation_key(resolved: Dictionary, top_override: Dictionary) -> String:
	var tex: Dictionary = resolved.get("textures", {})
	var tint: Dictionary = resolved.get("tint", {})
	var top: Dictionary = resolved.get("top", {})
	var inputs: Array = [float(tint.get("dL", 0)), float(tint.get("da", 0)), float(tint.get("db", 0)),
		float(top_override.get("top_value", top.get("value", 0.75))),
		float(top_override.get("top_saturation", top.get("saturation", 0.85))),
		float(top_override.get("top_blur_px", top.get("blur_px", 2)))]
	for role: String in SIZES:
		var path: String = MaterialResolver.res_path(str(tex.get(role, "")))
		var img: Image = ArtData.image_at(path)
		inputs.append([role, path, img.get_instance_id() if img != null else 0])
	return JSON.stringify(inputs)

func prepare_cached(resolved: Dictionary, top_override: Dictionary, out_problems: Array[String]) -> Dictionary:
	var key: String = _preparation_key(resolved, top_override)
	if _prepared_sets.has(key): return _prepared_sets[key]
	var prepared: Dictionary = prepare_set(resolved, top_override, out_problems)
	if not prepared.is_empty():
		_prepared_sets[key] = prepared
		prepared_misses += 1
	return prepared

## Warm the current registry's distinct appearances, with explicit work/time bounds for larger future catalogs.
## Missing or over-budget appearances retain the exact synchronous fallback; the report never calls that complete.
func warm_catalog(resolver: MaterialResolver, top_override: Dictionary, out_problems: Array[String]) -> Dictionary:
	var started: int = Time.get_ticks_usec()
	var candidates: int = 0
	var complete: bool = true
	var reason: String = "complete"
	var materials: Array = [""]
	for material: Dictionary in resolver.materials: materials.append(material.get("id", ""))
	var categories: Array[String] = [""]
	for category: Dictionary in resolver.categories: categories.append(str(category.get("category", category.get("id", ""))))
	for style: Dictionary in resolver.styles:
		if str(style.get("kind", "")) != "wall": continue
		for index: int in materials.size() + categories.size():
			if candidates >= 4096 or _prepared_sets.size() >= 128 or Time.get_ticks_usec() - started >= 5000000:
				complete = false
				reason = "candidate, appearance or five-second warmup budget reached"
				break
			var resolved: Dictionary
			if index < materials.size():
				resolved = resolver.resolve(str(style.get("id", "")), materials[index], "", "wall")
			else:
				resolved = resolver.resolve(str(style.get("id", "")), "", categories[index - materials.size()], "wall")
			candidates += 1
			if prepare_cached(resolved, top_override, out_problems).is_empty():
				complete = false
				reason = "invalid texture input"
				break
		if not complete: break
	return {"complete": complete, "reason": reason, "candidates": candidates, "unique_appearances": _prepared_sets.size(),
		"elapsed_usec": Time.get_ticks_usec() - started, "max_candidates": 4096, "max_appearances": 128, "budget_usec": 5000000}


func _ink() -> Image:
	if _ink_img == null:
		_ink_img = Image.create(TILE, TILE, false, Image.FORMAT_RGBA8)
		_ink_img.fill(ink)
	return _ink_img


## Bake chunk (cx, cy): {row (tile y): Image (chunk_tiles·64 × 64)} for every row with a wall or a hill face.
func bake(cx: int, cy: int) -> Dictionary:
	var rows: Dictionary = {}
	var x0: int = cx * chunk_tiles
	var y0: int = cy * chunk_tiles
	for ty: int in range(y0, y0 + chunk_tiles):
		var strip: Image = null
		for tx: int in range(x0, x0 + chunk_tiles):
			var t: Vector2i = Vector2i(tx, ty)
			var lx: int = (tx - x0) * TILE
			if faces.has(t) and hill_cap != null and rock_face != null:
				if strip == null:
					strip = Image.create(chunk_tiles * TILE, STRIP, false, Image.FORMAT_RGBA8)
				_hill_tile(strip, t, lx)
			if not occupancy.has(t) or openings.has(t) or not cell_set.has(t):
				continue
			var s: Dictionary = sets.get(cell_set[t], {})
			if s.is_empty():
				continue
			if strip == null:
				strip = Image.create(chunk_tiles * TILE, STRIP, false, Image.FORMAT_RGBA8)
			_wall_tile(strip, t, lx, s)
		if strip != null:
			rows[ty] = strip
	return rows


func _hill_tile(strip: Image, t: Vector2i, lx: int) -> void:
	var cw: int = hill_cap.get_width()
	var ch: int = hill_cap.get_height()
	_blit_wrapped(strip, hill_cap, posmod(t.x * TILE, cw), posmod(t.y * TILE - CAP, ch), TILE, CAP, Vector2i(lx, 0))
	_blit_wrapped(strip, rock_face, posmod(t.x * TILE, rock_face.get_width()), 0, TILE, FACE, Vector2i(lx, CAP))


## Copy a w × h region starting at (sx, sy) of a repeating source, wrapping at its edges.
func _blit_wrapped(dst: Image, src: Image, sx: int, sy: int, w: int, h: int, at: Vector2i) -> void:
	var sw: int = src.get_width()
	var sh: int = src.get_height()
	var y: int = 0
	while y < h:
		var py: int = (sy + y) % sh
		var hh: int = mini(h - y, sh - py)
		var x: int = 0
		while x < w:
			var px: int = (sx + x) % sw
			var ww: int = mini(w - x, sw - px)
			dst.blend_rect(src, Rect2i(px, py, ww, hh), at + Vector2i(x, y))
			x += ww
		y += hh


func _wall_tile(strip: Image, t: Vector2i, lx: int, s: Dictionary) -> void:
	var n: bool = occupancy.has(t + Vector2i(0, -1))
	var e: bool = occupancy.has(t + Vector2i(1, 0))
	var so: bool = occupancy.has(t + Vector2i(0, 1))
	var w: bool = occupancy.has(t + Vector2i(-1, 0))
	var height: int = TILE if so else CAP
	var horizontal: bool = e or w
	var vertical: bool = n or so
	var top: Image
	if horizontal and not vertical:
		top = s["top_ew"]
		_blit_wrapped(strip, top, posmod(t.x * TILE, top.get_width()), 0, TILE, height, Vector2i(lx, 0))
	elif vertical and not horizontal:
		top = s["top_ns"]
		_blit_wrapped(strip, top, 0, posmod(t.y * TILE, top.get_height()), TILE, height, Vector2i(lx, 0))
	else:
		strip.blend_rect(s["junction"], Rect2i(0, 0, TILE, height), Vector2i(lx, 0))
	# Ink on the top's edges that face a tile without a wall (walls.py `_finish_top`, inset by ink_px).
	var k: Image = _ink()
	if not n:
		strip.blend_rect(k, Rect2i(0, 0, TILE, ink_px), Vector2i(lx, 0))
	if not so:
		strip.blend_rect(k, Rect2i(0, 0, TILE, ink_px), Vector2i(lx, height - ink_px))
	if not e:
		strip.blend_rect(k, Rect2i(0, 0, ink_px, height), Vector2i(lx + TILE - ink_px, 0))
	if not w:
		strip.blend_rect(k, Rect2i(0, 0, ink_px, height), Vector2i(lx, 0))
	if _face_visible(t):
		var face: Image = s["face"]
		_blit_wrapped(strip, face, posmod(t.x * TILE, face.get_width()), 0, TILE, FACE, Vector2i(lx, CAP))
		var end: Image = s["end"]
		if not _face_visible(t + Vector2i(-1, 0)):
			strip.blend_rect(end, Rect2i(0, 0, end.get_width(), FACE), Vector2i(lx, CAP))
		if not _face_visible(t + Vector2i(1, 0)):
			strip.blend_rect(end, Rect2i(0, 0, end.get_width(), FACE), Vector2i(lx + TILE - end.get_width(), CAP))


func _face_visible(t: Vector2i) -> bool:
	return occupancy.has(t) and not openings.has(t) and not occupancy.has(t + Vector2i(0, 1))
