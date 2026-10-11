extends RefCounted

const Cold = preload("res://scripts/ui/cold_effects.gd")
const FROST: Color = Color("D8E8F0")
const FROST_LOWER: Color = Color("A8C0D0")
const FADE_SECONDS: float = 0.4
static var _textures: Dictionary = {}
static var _breath_texture: ImageTexture

static func sleeper_counts(agents: Dictionary, buildings: Dictionary) -> Dictionary:
	var counts: Dictionary = {}
	for key: String in ["xs", "ys"]:
		if not agents.get(key) is PackedInt32Array or not buildings.get(key) is PackedInt32Array:
			return counts
	for key: String in ["states", "target_kinds", "colds"]:
		if not agents.get(key) is PackedByteArray:
			return counts
	if not buildings.get("kinds") is PackedByteArray:
		return counts
	var shelters: Dictionary = {}
	for i: int in mini(buildings.xs.size(), mini(buildings.ys.size(), buildings.kinds.size())):
		if buildings.kinds[i] == 1:
			shelters[Vector2i(buildings.xs[i], buildings.ys[i])] = true
	var size: int = mini(agents.xs.size(), mini(agents.ys.size(), mini(agents.states.size(), mini(agents.target_kinds.size(), agents.colds.size()))))
	for i: int in size:
		var tile := Vector2i(agents.xs[i], agents.ys[i])
		if agents.states[i] == 3 and agents.target_kinds[i] == 3 and agents.colds[i] == 1 and shelters.has(tile):
			counts[tile] = int(counts.get(tile, 0)) + 1
	return counts

static func frost_image(source: Image) -> Image:
	var result: Image = source.duplicate()
	var edges: Array[Vector2i] = []
	for y: int in source.get_height():
		if float(y) >= float(source.get_height()) * 0.4:
			break
		for x: int in source.get_width():
			if source.get_pixel(x, y).a > 0.0 and (y == 0 or source.get_pixel(x, y - 1).a == 0.0):
				edges.append(Vector2i(x, y))
	for i: int in edges.size():
		var edge: Vector2i = edges[i]
		if i % 2 == 1 and edge.y + 1 < source.get_height() and source.get_pixel(edge.x, edge.y + 1).a > 0.0:
			result.set_pixel(edge.x, edge.y + 1, Color(FROST_LOWER, source.get_pixel(edge.x, edge.y + 1).a))
	for edge: Vector2i in edges:
		result.set_pixelv(edge, Color(FROST, source.get_pixelv(edge).a))
	return result

static func fade(alpha: float, cold: bool, delta: float) -> float:
	return move_toward(alpha, 1.0 if cold else 0.0, maxf(delta, 0.0) / FADE_SECONDS)

static func tint_strength(start: float, target: float, elapsed: float) -> float:
	return lerpf(start, target, smoothstep(0.0, 1.0, clampf(elapsed, 0.0, 1.0)))

static func apply(sprite: Sprite2D, sleepers: int, seconds: float, delta: float) -> void:
	var alpha: float = fade(float(sprite.get_meta("frost_alpha", 0.0)), sleepers > 0, delta)
	sprite.set_meta("cold_sleepers", sleepers)
	sprite.set_meta("frost_alpha", alpha)
	var overlay: Sprite2D = sprite.get_node_or_null("Frost") as Sprite2D
	if overlay == null and alpha > 0.0 and sprite.texture != null:
		overlay = Sprite2D.new()
		overlay.name = "Frost"
		overlay.centered = false
		overlay.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		overlay.set_meta("art_kind", "shelter_frost")
		overlay.set_meta("art_id", sprite.get_meta("art_id", 0))
		sprite.add_child(overlay)
	if overlay != null:
		var key: String = str(sprite.get_meta("sprite_id", ""))
		if not _textures.has(key):
			var source: Image = sprite.texture.get_image()
			if source.is_compressed():
				source.decompress()
			if sprite.region_enabled:
				source = source.get_region(Rect2i(sprite.region_rect))
			_textures[key] = ImageTexture.create_from_image(frost_image(source))
		overlay.texture = _textures[key] as Texture2D
		overlay.offset = sprite.offset
		overlay.modulate.a = alpha
		overlay.visible = alpha > 0.0
	var puff: Dictionary = Cold.breath(sleepers > 0, int(sprite.get_meta("art_id", 0)), seconds)
	sprite.set_meta("doorway_puff_alpha", puff.alpha)
	var breath: Sprite2D = sprite.get_node_or_null("DoorwayColdBreath") as Sprite2D
	if breath == null and sleepers > 0:
		breath = Sprite2D.new()
		breath.name = "DoorwayColdBreath"
		if _breath_texture == null:
			var pixels: Image = Image.create(3, 2, false, Image.FORMAT_RGBA8)
			pixels.fill(Color.WHITE)
			_breath_texture = ImageTexture.create_from_image(pixels)
		breath.texture = _breath_texture
		breath.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		breath.set_meta("art_kind", "shelter_cold_breath")
		breath.set_meta("art_id", sprite.get_meta("art_id", 0))
		sprite.add_child(breath)
	if breath != null:
		breath.visible = float(puff.alpha) > 0.0
		breath.modulate = Color(FROST, float(puff.alpha))
		breath.position = Vector2(0.0, -5.0 - float(puff.rise))
