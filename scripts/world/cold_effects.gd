# cold_effects.gd: cold presentation for villagers: crossed-arms pose, shiver offset, breath puff, and the
#   cold_audit.json frame rejection check; used by agent_renderer.gd, agent_activity.gd and morning_chill.gd.
# Bridge methods: none.
extends RefCounted

static var _texture: ImageTexture
static var _audit: Dictionary = {}

static func snapshot_cold(snapshot: Dictionary, index: int) -> bool:
	var values: Variant = snapshot.get("colds")
	return values is PackedByteArray and index >= 0 and index < values.size() and values[index] == 1

static func pose(base: String, cold: bool, moving: bool) -> String:
	return "crossed" if cold and not moving and base in ["idle", "sit", "hop"] else base

static func shiver(cold: bool, moving: bool, agent_id: int, seconds: float) -> float:
	return float(posmod(floori((seconds + float(posmod(agent_id, 6)) / 36.0) * 6.0), 2) == 0) if cold and not moving else 0.0

static func breath(cold: bool, agent_id: int, seconds: float) -> Dictionary:
	var age: float = fposmod(seconds + float(posmod(agent_id, 24)) * 0.1, 2.4)
	return {"alpha": 0.55 * (1.0 - age) if cold and age < 1.0 else 0.0, "rise": 4.0 * minf(age, 1.0)}

static func audit_rejected(sprite_id: String, rgba_sha256: String) -> bool:
	if _audit.is_empty():
		var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string("res://scripts/world/cold_audit.json"))
		if parsed is Dictionary:
			_audit = parsed
	return _audit.has(sprite_id) and str(_audit[sprite_id]) == rgba_sha256

static func mark_audit(entry: Dictionary, pages: Dictionary) -> bool:
	var sprite_id: String = str(entry.get("sprite_id", ""))
	audit_rejected(sprite_id, "")
	if not _audit.has(sprite_id):
		return false
	var page: String = str(entry.get("page", ""))
	var rect: Array = entry.get("rect", [])
	if page.is_empty() or rect.size() != 4:
		return false
	if not pages.has(page):
		var texture: CompressedTexture2D = load(page) as CompressedTexture2D
		pages[page] = texture.get_image() if texture != null else null
	var source: Image = pages[page]
	if source == null or source.is_empty():
		return true
	if source.is_compressed():
		source.decompress()
	var crop: Image = source.get_region(Rect2i(int(rect[0]), int(rect[1]), int(rect[2]), int(rect[3])))
	crop.convert(Image.FORMAT_RGBA8)
	var hash: HashingContext = HashingContext.new()
	hash.start(HashingContext.HASH_SHA256)
	hash.update(crop.get_data())
	return audit_rejected(sprite_id, hash.finish().hex_encode())

static func apply(sprite: Sprite2D, cold: bool, moving: bool, agent_id: int, seconds: float) -> void:
	var offset: float = shiver(cold, moving, agent_id, seconds)
	sprite.position.x += offset
	sprite.set_meta("cold", cold)
	sprite.set_meta("shiver_offset", offset)
	var puff: Dictionary = breath(cold and sprite.visible, agent_id, seconds)
	sprite.set_meta("puff_alpha", puff["alpha"])
	var child: Sprite2D = sprite.get_node_or_null("ColdBreath") as Sprite2D
	if child == null and cold:
		child = Sprite2D.new()
		child.name = "ColdBreath"
		if _texture == null:
			var pixels: Image = Image.create(3, 2, false, Image.FORMAT_RGBA8)
			pixels.fill(Color.WHITE)
			_texture = ImageTexture.create_from_image(pixels)
		child.texture = _texture
		child.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		sprite.add_child(child)
	if child != null:
		child.visible = float(puff["alpha"]) > 0.0
		child.modulate = Color(1.0, 1.0, 1.0, float(puff["alpha"]))
		child.position = Vector2(0.0, sprite.offset.y - 3.0 - float(puff["rise"]))
