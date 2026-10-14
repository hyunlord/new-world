extends SceneTree
const Chill = preload("res://scripts/ui/morning_chill.gd")
var failures: int = 0

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	var memory = Chill.new()
	var cold: Dictionary = {"agent_ids": PackedInt64Array([1]), "colds": PackedByteArray([1])}
	var warm: Dictionary = {"agent_ids": PackedInt64Array([1]), "colds": PackedByteArray([0])}
	memory.ingest(cold, 359)
	memory.ingest(warm, 360)
	var image := Image.create(16, 16, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)
	image.fill_rect(Rect2i(4, 4, 8, 8), Color.WHITE)
	var sprite := Sprite2D.new()
	sprite.texture = ImageTexture.create_from_image(image)
	sprite.region_enabled = true
	sprite.region_rect = Rect2(2, 2, 12, 12)
	sprite.flip_h = true
	sprite.set_meta("activity", "walk")
	sprite.position = Vector2(0.75, 20.0)
	root.add_child(sprite)
	for elapsed: int in [0, 59, 60, 75, 89, 90]:
		memory.apply(sprite, 1, 360 + elapsed, 0.0)
		var expected: float = 0.8 * clampf(float(90 - elapsed) / 30.0, 0.0, 1.0)
		check(is_equal_approx(float(sprite.get_meta("chill_outline_alpha")), expected), "outline alpha has sixty-minute hold and thirty-minute fade")
		check(sprite.position == Vector2(0.75, 20.0), "morning chill never shivers or changes walker offset")
		var outline: Sprite2D = sprite.get_node("ChillOutline")
		check(outline.visible == (elapsed < 90), "outline window ends at ninety minutes")
		check(outline.region_rect == sprite.region_rect.grow(1.0) and outline.flip_h, "outline is one native texel with matching atlas frame and flip")
		if elapsed < 90:
			var color: Color = (outline.material as ShaderMaterial).get_shader_parameter("outline_color")
			check(color.is_equal_approx(Color(207.0/255.0, 227.0/255.0, 245.0/255.0, expected)), "pale blue outline color and alpha")
	check((sprite.get_node("ChillBreath") as Sprite2D).scale == Vector2(1.5, 1.5), "breath enlarged by exactly one half")
	sprite.set_meta("activity", "asleep")
	memory.apply(sprite, 1, 390, 0.0)
	check(not sprite.get_node("ChillOutline").visible and not sprite.get_node("ChillBreath").visible, "sleep suppresses both cues")
	sprite.set_meta("activity", "walk")
	memory.ingest(cold, 400)
	memory.apply(sprite, 1, 400, 0.0)
	check(not sprite.get_node("ChillOutline").visible, "active cold is not morning chill")
	memory.clear()
	memory.apply(sprite, 1, 390, 0.0)
	check(not sprite.get_node("ChillOutline").visible, "reset clears outline")
	sprite.free()
	print("MORNING_CHILL_V2 failures=", failures)
	quit(1 if failures else 0)
