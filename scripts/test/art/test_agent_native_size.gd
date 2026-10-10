extends SceneTree
const Library = preload("res://scripts/ui/art_sprite_library.gd")
func _initialize() -> void:
	var sprite: Sprite2D = Sprite2D.new()
	assert(Library.apply(sprite, "direction_09_villager_gatherer"))
	var entry: Dictionary = Library.frame("direction_09_villager_gatherer")
	var rect: Array = entry["rect"]
	var anchor: Array = entry["anchor"]
	assert(sprite.scale == Vector2.ONE)
	assert(sprite.region_rect.size == Vector2(float(rect[2]), float(rect[3])))
	assert(sprite.offset == -Vector2(float(anchor[0]), float(anchor[1])))
	assert(not sprite.centered)
	assert(sprite.texture_filter == CanvasItem.TEXTURE_FILTER_NEAREST)
	assert(sprite.modulate == Color.WHITE)
	assert(Library.metadata("direction_09_villager_gatherer") == entry)
	assert(not Library.is_suppressed("direction_09_villager_gatherer"))
	assert(Library.metadata("unknown_test_sprite").is_empty())
	assert(not Library.is_suppressed("unknown_test_sprite"))
	var saved_suppressed: Dictionary = Library._suppressed.duplicate(true)
	Library._suppressed["test_suppressed"] = {"drawable": false, "qa_status": "suppressed", "reason": "no accepted decorative art"}
	assert(Library.is_suppressed("test_suppressed"))
	assert(not bool(Library.metadata("test_suppressed")["drawable"]))
	assert(Library.frame("test_suppressed").is_empty())
	assert(not Library.entries().has("test_suppressed"))
	assert(not Library.apply(sprite, "test_suppressed"))
	assert(sprite.texture == null)
	assert(not bool(sprite.get_meta("drawable")))
	assert(Library.apply(sprite, "direction_09_villager_gatherer"))
	assert(sprite.texture != null and bool(sprite.get_meta("drawable")))
	Library._suppressed = saved_suppressed
	sprite.free()
	print("ART NATIVE PASS: atlas frame, unit scale, feet anchor, nearest, no tint")
	quit(0)
