extends RefCounted

const Library = preload("res://scripts/ui/art_sprite_library.gd")
static var _manifest: Dictionary = {}
static var _body: ImageTexture
static var _props: ImageTexture

static func _load() -> void:
	if not _manifest.is_empty():
		return
	_manifest = JSON.parse_string(FileAccess.get_file_as_string("res://assets/art/atlas/carry_p6.json"))
	_body = ImageTexture.create_from_image(Image.load_from_file(_manifest["body_page"]))
	_props = ImageTexture.create_from_image(Image.load_from_file(_manifest["prop_page"]))
	_body.take_over_path(str(_manifest["body_page"]))
	_props.take_over_path(str(_manifest["prop_page"]))

static func apply(sprite: Sprite2D, activity: String) -> void:
	_load()
	var frame_id: String = str(sprite.get_meta("sprite_id", ""))
	var entry: Dictionary = _manifest["sprites"].get(frame_id, {})
	var enabled: bool = activity == "carry" and not entry.is_empty() and sprite.visible
	var layer: Sprite2D = sprite.get_node_or_null("CarryLoad") as Sprite2D
	if layer != null:
		layer.visible = false
	sprite.set_meta("load_visible", enabled)
	sprite.set_meta("load_kind", "food" if enabled else "")
	sprite.set_meta("load_frame_id", frame_id if enabled else "")
	if not enabled:
		if sprite.texture == _body:
			sprite.texture = load(str(Library.frame(frame_id).get("page", ""))) as Texture2D
		return
	sprite.texture = _body
	if layer == null:
		layer = Sprite2D.new()
		layer.name = "CarryLoad"
		layer.set_meta("art_kind", "carry_load")
		layer.set_meta("art_id", sprite.get_meta("art_id", 0))
		layer.centered = false
		layer.region_enabled = true
		layer.region_filter_clip_enabled = true
		layer.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		layer.z_as_relative = true
		layer.z_index = 0
		sprite.add_child(layer)
	var rect: Array = entry["rect"]
	var anchor: Array = entry["anchor"]
	layer.texture = _props
	layer.region_rect = Rect2(rect[0], rect[1], rect[2], rect[3])
	layer.position = sprite.offset + Vector2(anchor[0], anchor[1])
	layer.flip_h = sprite.flip_h
	if sprite.flip_h:
		layer.position.x = -layer.position.x
	layer.visible = true
	sprite.set_meta("load_flip_h", layer.flip_h)
	sprite.set_meta("load_z_index", sprite.z_index + layer.z_index)
	sprite.set_meta("load_anchor", layer.position)
	sprite.set_meta("load_size", layer.region_rect.size)
