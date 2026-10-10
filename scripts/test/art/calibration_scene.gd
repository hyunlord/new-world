extends Node2D

const SpriteLibrary = preload("res://scripts/ui/art_sprite_library.gd")


func _ready() -> void:
	var objects: Node2D = get_node("WorldObjects") as Node2D
	var checker_image: Image = Image.create(8, 8, false, Image.FORMAT_RGBA8)
	for y: int in range(8):
		for x: int in range(8):
			checker_image.set_pixel(x, y, Color.WHITE if (x + y) % 2 == 0 else Color.BLACK)
	var checker: Sprite2D = Sprite2D.new()
	checker.name = "Checker"
	checker.texture = ImageTexture.create_from_image(checker_image)
	checker.centered = false
	checker.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	checker.position = Vector2(-130, -65)
	checker.set_meta("art_kind", "checker")
	checker.set_meta("art_id", 0)
	checker.set_meta("calibration_kind", "checker")
	objects.add_child(checker)
	_add(objects, "direction_09_villager_gatherer", Vector2(-30, -4), "north_agent", 1)
	_add(objects, "direction_02_lean_to", Vector2(-30, 0), "north_lean_to", 2)
	_add(objects, "direction_02_lean_to", Vector2(30, 0), "south_lean_to", 3)
	_add(objects, "direction_09_villager_gatherer", Vector2(30, 4), "south_agent", 4)
	var looks: Array[String] = ["direction_09_villager_gatherer", "direction_10_villager_builder", "direction_11_villager_elder", "direction_12_villager_child"]
	for index: int in range(12):
		var actor: Sprite2D = _add(objects, looks[index % looks.size()], Vector2(-132 + index * 24, 67), "ramp_variant", 10 + index)
		actor.set_meta("body", index % 2)
		var variant: Dictionary = {"skin": index % 4, "hair": index % 5, "clothing": index % 6}
		actor.set_meta("variant", variant)
		SpriteLibrary.apply_variant(actor, variant)


func _add(parent: Node, sprite_id: String, base: Vector2, role: String, id: int) -> Sprite2D:
	var sprite: Sprite2D = Sprite2D.new()
	sprite.name = role + "_" + str(id)
	parent.add_child(sprite)
	var selected: String = sprite_id
	if sprite_id == "direction_02_lean_to" and not SpriteLibrary.frame("S1_11").is_empty():
		selected = "S1_11"
	if "villager" in sprite_id:
		var body: String = "child" if "child" in sprite_id else "adult_a"
		for id_key: String in SpriteLibrary.entries():
			var entry: Dictionary = SpriteLibrary.frame(id_key)
			if str(entry.get("body", "")) == body and str(entry.get("activity", "")) == "idle" and str(entry.get("view", "")) == "S":
				selected = id_key
				break
	if not SpriteLibrary.apply(sprite, selected):
		push_error("Calibration sprite missing: " + sprite_id)
	sprite.position = base
	sprite.set_meta("art_kind", "villager" if "agent" in role or role == "ramp_variant" else "lean_to")
	sprite.set_meta("calibration_kind", role)
	sprite.set_meta("art_id", id)
	return sprite
