extends SceneTree

const Hash = preload("res://scripts/ui/art_hash.gd")
const SpriteLibrary = preload("res://scripts/ui/art_sprite_library.gd")

func _initialize() -> void:
	var sprite := Sprite2D.new()
	var installed_id: String = ""
	for sprite_id: String in SpriteLibrary.entries():
		var entry: Dictionary = SpriteLibrary.frame(sprite_id)
		if str(entry.get("body", "")) == "adult_a":
			installed_id = sprite_id
			break
	assert(not installed_id.is_empty(), "An installed adult frame exercises the actual material path")
	assert(SpriteLibrary.apply(sprite, installed_id))
	for agent_id: int in range(3000):
		var look: Dictionary = Hash.look(agent_id)
		assert(SpriteLibrary.apply_variant(sprite, look))
		assert(roundi(sprite.self_modulate.r * 15.0) == int(look["tone"]))
		assert(roundi(sprite.self_modulate.g * 15.0) == int(look["hair"]))
		assert(sprite.self_modulate.b == 0.0)
		assert(sprite.self_modulate.a == 1.0)
	assert(sprite.material is ShaderMaterial)
	var material: ShaderMaterial = sprite.material as ShaderMaterial
	assert(material.shader != null)
	assert(material.get_shader_parameter("region_texture") is Texture2D)
	assert(material.get_shader_parameter("ramp_texture") is Texture2D)
	for tone: int in range(3):
		for hair: int in range(5):
			assert(SpriteLibrary.apply_variant(sprite, {"tone": tone, "hair": hair}))
			assert(roundi(sprite.self_modulate.r * 15.0) == tone)
			assert(roundi(sprite.self_modulate.g * 15.0) == hair)
			assert(sprite.self_modulate.b == 0.0)
	sprite.free()
	print("ART VARIANT CODES PASS: actual apply_variant, 3000 identities and all 15 tone/hair pairs; pixel proof requires calibration")
	quit(0)
