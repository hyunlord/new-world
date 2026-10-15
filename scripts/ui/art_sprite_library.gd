class_name ArtSpriteLibrary
extends RefCounted

const ATLAS_PATH: String = "res://assets/art/atlas/world_atlas.json"
static var _entries: Dictionary = {}
static var _suppressed: Dictionary = {}
static var _pages: Dictionary = {}
static var _loaded: bool = false

static func _ensure_loaded() -> void:
	if _loaded:
		return
	_loaded = true
	if not FileAccess.file_exists(ATLAS_PATH):
		push_error("Art atlas manifest missing: " + ATLAS_PATH)
		return
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(ATLAS_PATH))
	if parsed is Dictionary:
		_entries = parsed.get("sprites", {})
		_suppressed = parsed.get("suppressed", {})
		for sprite_id: String in _suppressed:
			_entries.erase(sprite_id)

static func frame(sprite_id: String) -> Dictionary:
	_ensure_loaded()
	return _entries.get(sprite_id, {})

## Includes intentionally suppressed slots so callers can avoid resurrecting reference art.
static func metadata(sprite_id: String) -> Dictionary:
	_ensure_loaded()
	if _suppressed.has(sprite_id):
		return _suppressed[sprite_id]
	return _entries.get(sprite_id, {})

static func is_suppressed(sprite_id: String) -> bool:
	_ensure_loaded()
	return _suppressed.has(sprite_id)

static func entries() -> Dictionary:
	_ensure_loaded()
	return _entries.duplicate(true)

static func apply(sprite: Sprite2D, sprite_id: String) -> bool:
	if is_suppressed(sprite_id):
		sprite.texture = null
		sprite.material = null
		sprite.set_meta("sprite_id", sprite_id)
		sprite.set_meta("frame_size", Vector2i.ZERO)
		sprite.set_meta("drawable", false)
		return false
	var entry: Dictionary = frame(sprite_id)
	if entry.is_empty():
		return false
	var page: String = str(entry["page"])
	if not _pages.has(page):
		_pages[page] = load(page) as Texture2D
	var rect: Array = entry["rect"]
	var anchor: Array = entry["anchor"]
	sprite.texture = _pages[page] as Texture2D
	sprite.region_enabled = true
	sprite.region_rect = Rect2(float(rect[0]), float(rect[1]), float(rect[2]), float(rect[3]))
	sprite.region_filter_clip_enabled = true
	sprite.centered = false
	sprite.offset = -Vector2(float(anchor[0]), float(anchor[1]))
	sprite.scale = Vector2.ONE
	sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	sprite.set_meta("sprite_id", sprite_id)
	sprite.set_meta("drawable", true)
	sprite.set_meta("frame_id", sprite_id)
	sprite.set_meta("frame_size", Vector2i(int(rect[2]), int(rect[3])))
	sprite.set_meta("anchor", Vector2(float(anchor[0]), float(anchor[1])))
	return sprite.texture != null

static func shared_layer(owner: Node, layer_name: String, z: int, sorted: bool = false) -> Node2D:
	var main: Node = owner.get_parent()
	var layer: Node2D = main.get_node_or_null(NodePath(layer_name)) as Node2D
	if layer == null:
		layer = Node2D.new()
		layer.name = layer_name
		main.add_child(layer)
	layer.z_index = z
	layer.y_sort_enabled = sorted
	return layer

static var _variant_materials: Dictionary = {}
static var _variant_regions: Dictionary = {}
static var _variant_regions_cache_enabled: bool = true

static func set_variant_regions_cache_enabled(enabled: bool) -> void:
	_variant_regions_cache_enabled = enabled
	_variant_regions.clear()

static func variant_regions_cache_enabled() -> bool:
	return _variant_regions_cache_enabled

static func _regions_for_page(page: String) -> String:
	if _variant_regions_cache_enabled and _variant_regions.has(page):
		return str(_variant_regions[page])
	var regions: String = page.get_basename() + "_regions.png"
	var result: String = regions if ResourceLoader.exists(regions) else ""
	if _variant_regions_cache_enabled:
		_variant_regions[page] = result
	return result


static func apply_variant(sprite: Sprite2D, look: Dictionary) -> bool:
	var entry: Dictionary = frame(str(sprite.get_meta("sprite_id", "")))
	if entry.is_empty():
		return false
	var page: String = str(entry["page"])
	var regions: String = _regions_for_page(page)
	if regions.is_empty():
		sprite.material = null
		sprite.self_modulate = Color.WHITE
		return false
	if not _variant_materials.has(page):
		var material: ShaderMaterial = ShaderMaterial.new()
		material.shader = load("res://shaders/villager_ramp.gdshader") as Shader
		material.set_shader_parameter("region_texture", load(regions))
		material.set_shader_parameter("ramp_texture", load("res://assets/art/villager_ramps.png"))
		_variant_materials[page] = material
	sprite.material = _variant_materials[page] as ShaderMaterial
	sprite.self_modulate = Color(float(look.get("tone", 1)) / 15.0, float(look.get("hair", 0)) / 15.0, 0.0, 1.0)
	return true
