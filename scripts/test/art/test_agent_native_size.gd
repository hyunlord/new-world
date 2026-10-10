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
	sprite.free()
	print("ART NATIVE PASS: atlas frame, unit scale, feet anchor, nearest, no tint")
	quit(0)
