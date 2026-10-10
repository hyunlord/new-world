extends SceneTree

const Hash = preload("res://scripts/ui/art_hash.gd")

func _initialize() -> void:
	var shader: Shader = load("res://shaders/villager_ramp.gdshader") as Shader
	assert(shader != null)
	for agent_id: int in range(1000):
		var look: Dictionary = Hash.look(agent_id)
		var sprite := Sprite2D.new()
		sprite.self_modulate = Color(float(look["skin"]) / 15.0, float(look["hair"]) / 15.0, float(look["clothing"]) / 15.0, 1.0)
		assert(roundi(sprite.self_modulate.r * 15.0) == int(look["skin"]))
		assert(roundi(sprite.self_modulate.g * 15.0) == int(look["hair"]))
		assert(roundi(sprite.self_modulate.b * 15.0) == int(look["clothing"]))
		sprite.free()
	print("ART VARIANT CODES PASS: 1000 self_modulate round trips; pixel shader proof requires windowed calibration")
	quit(0)
