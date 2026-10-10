class_name ArtSpriteLibrary
extends RefCounted

const ATLAS_PATH: String = "res://assets/art/atlas/world_atlas.json"
static var _entries: Dictionary = {}
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

static func frame(sprite_id: String) -> Dictionary:
	_ensure_loaded()
	return _entries.get(sprite_id, {})

static func entries() -> Dictionary:
	_ensure_loaded()
	return _entries.duplicate(true)

static func apply(sprite: Sprite2D, sprite_id: String) -> bool:
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
