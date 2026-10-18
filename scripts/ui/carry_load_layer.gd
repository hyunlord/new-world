extends RefCounted

const Library = preload("res://scripts/ui/art_sprite_library.gd")
static var _manifest: Dictionary = {}
static var _body: Texture2D
static var _props: Texture2D

static func _load() -> void:
	if not _manifest.is_empty():
		return
	_manifest = JSON.parse_string(FileAccess.get_file_as_string("res://assets/art/atlas/carry_p6.json"))
	_body = load(str(_manifest["body_page"])) as Texture2D
	_props = load(str(_manifest["prop_page"])) as Texture2D
	_body.take_over_path(str(_manifest["body_page"]))
	_props.take_over_path(str(_manifest["prop_page"]))

static func apply(sprite: Sprite2D, activity: String) -> void:
	_load()
	var frame_id: String = str(sprite.get_meta("sprite_id", ""))
	var entry: Dictionary = _manifest["sprites"].get(frame_id, {})
	var enabled: bool = activity == "carry" and not entry.is_empty() and sprite.visible
	var source: Dictionary = Library.frame(frame_id)
	var approved: bool = str(source.get("qa_status", "")) == "accepted" or (str(source.get("qa_status", "")) == "derived" and str(source.get("source", "")) == "derived:same_cycle_phase_copy")
	var baked_wood: bool = activity == "carry_wood" and sprite.visible and str(source.get("activity", "")) == "carry_wood" and approved and bool(source.get("drawable", true))
	var layer: Sprite2D = sprite.get_node_or_null("CarryLoad") as Sprite2D
	if layer != null:
		layer.visible = false
	sprite.set_meta("load_visible", enabled or baked_wood)
	sprite.set_meta("load_kind", "food" if enabled else ("wood" if baked_wood else ""))
	sprite.set_meta("load_frame_id", frame_id if enabled or baked_wood else "")
	sprite.set_meta("load_baked", baked_wood)
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


static func apply_founder(sprite: Sprite2D, founder: bool, activity: String, carried_food: float, carried_wood: int, pose: String = "") -> void:
	var suppressed: String = ""
	if not founder:
		suppressed = "not_founder"
	elif not sprite.visible:
		suppressed = "hidden_agent"
	elif activity in ["eat", "asleep"] or pose in ["eat", "asleep"]:
		suppressed = activity if activity in ["eat", "asleep"] else pose
	elif carried_food > 0.0 or carried_wood > 0:
		suppressed = "real_load"
	var layer: Sprite2D = sprite.get_node_or_null("FounderBundle") as Sprite2D
	sprite.set_meta("founder_bundle_visible", suppressed.is_empty())
	sprite.set_meta("founder_bundle_suppressed", suppressed)
	if not suppressed.is_empty():
		if layer != null:
			layer.hide()
		return
	if layer == null:
		layer = Sprite2D.new()
		layer.name = "FounderBundle"
		layer.set_meta("art_kind", "founder_bundle")
		layer.set_meta("art_id", sprite.get_meta("art_id", 0))
		sprite.add_child(layer)
		Library.apply(layer, "W1_16")
		layer.scale = Vector2(0.5, 0.5)
		layer.z_as_relative = true
		layer.z_index = 0
	var size: Vector2 = Vector2(sprite.get_meta("frame_size", Vector2i.ZERO))
	var anchor: Vector2 = sprite.offset + Vector2(size.x * 0.5 - 3.0, size.y * 0.45)
	if sprite.flip_h:
		anchor.x = -anchor.x
	layer.position = anchor.round()
	layer.flip_h = sprite.flip_h
	layer.show()
	sprite.set_meta("founder_bundle_anchor", layer.position)
	sprite.set_meta("founder_bundle_source", "W1_16")
