extends SceneTree
const Cold = preload("res://scripts/ui/shelter_cold.gd")
const ART = preload("res://scripts/ui/art_sprite_library.gd")
var failures: int = 0

func check(value: bool, message: String) -> void:
	if not value:
		failures += 1
		push_error(message)

func _initialize() -> void:
	var buildings: Dictionary = {"xs": PackedInt32Array([1,2,3]), "ys": PackedInt32Array([4,4,4]), "kinds": PackedByteArray([1,1,0])}
	var agents: Dictionary = {"xs": PackedInt32Array([1,2,3,1]), "ys": PackedInt32Array([4,4,4,5]), "states": PackedByteArray([3,3,3,0]), "target_kinds": PackedByteArray([3,3,3,0]), "colds": PackedByteArray([1,0,1,1])}
	var counts: Dictionary = Cold.sleeper_counts(agents, buildings)
	check(counts.size() == 1 and counts.get(Vector2i(1,4)) == 1, "only exact cold shelter sleeper counts")
	check(Cold.sleeper_counts({}, buildings).is_empty(), "absent optional columns safe")
	var source: Image = Image.create(8, 10, false, Image.FORMAT_RGBA8)
	source.fill(Color.TRANSPARENT)
	for y: int in range(1, 8):
		for x: int in range(1, 7):
			source.set_pixel(x, y, Color("665544"))
	var frosted: Image = Cold.frost_image(source)
	for y: int in 10:
		for x: int in 8:
			var expected: Color = source.get_pixel(x, y)
			if y == 1 and x >= 1 and x <= 6:
				expected = Color("D8E8F0")
			elif y == 2 and x in [2,4,6]:
				expected = Color("A8C0D0")
			check(frosted.get_pixel(x,y).is_equal_approx(expected), "only roof edge and alternate opaque underside change")
	check(is_equal_approx(Cold.fade(0.0, true, 0.2), 0.5), "half frost after .2 seconds")
	check(Cold.fade(0.0, true, 0.4) == 1.0 and Cold.fade(1.0, false, 0.4) == 0.0, "fade each direction .4 seconds")
	check(is_equal_approx(Cold.tint_strength(0.0, 0.12, 0.5), 0.06), "tint smooth midpoint")
	check(is_equal_approx(Cold.tint_strength(0.0, 0.12, 1.0), 0.12), "natural tint maximum at1s")
	var sprite := Sprite2D.new()
	sprite.set_meta("art_id", 0)
	ART.apply(sprite, "S1_11")
	Cold.apply(sprite, 1, 0.0, 0.4)
	check(sprite.get_meta("cold_sleepers") == 1 and sprite.get_meta("frost_alpha") == 1.0, "runtime metadata")
	var overlay: Sprite2D = sprite.get_node("Frost") as Sprite2D
	check(overlay.texture.get_size() == sprite.region_rect.size and overlay.offset == sprite.offset, "frost aligned with native frame")
	var breath: Sprite2D = sprite.get_node("DoorwayColdBreath") as Sprite2D
	check(breath.texture.get_size() == Vector2(3,2) and breath.visible, "doorway 3x2 breath")
	Cold.apply(sprite, 1, 1.1, 0.0)
	check(not breath.visible, "doorway puff quiet phase")
	Cold.apply(sprite, 1, 2.4, 0.0)
	check(breath.visible, "doorway repeats2.4 seconds")
	Cold.apply(sprite, 0, 2.4, 0.4)
	check(not overlay.visible and not breath.visible, "warm transition clears presentation")
	sprite.free()
	print("SHELTER_COLD failures=", failures)
	quit(1 if failures else 0)
