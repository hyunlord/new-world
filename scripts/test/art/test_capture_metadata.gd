extends SceneTree

const Metadata = preload("res://scripts/test/art/capture_metadata.gd")


func _init() -> void:
	call_deferred("_run")


func _run() -> void:
	var main: Node2D = Node2D.new()
	root.add_child(main)
	var objects: Node2D = Node2D.new()
	objects.name = "WorldObjects"
	objects.z_index = 4
	main.add_child(objects)
	var shadows: Node2D = Node2D.new()
	shadows.name = "Shadows"
	main.add_child(shadows)
	var agent: Sprite2D = Sprite2D.new()
	agent.set_meta("art_kind", "agent")
	agent.set_meta("art_id", 17)
	objects.add_child(agent)
	agent.position = Vector2(25, 31)
	agent.modulate.a = 0.45
	agent.self_modulate.a = 0.8
	agent.texture = ImageTexture.create_from_image(Image.create(8, 8, false, Image.FORMAT_RGBA8))
	var icon: Sprite2D = Sprite2D.new()
	icon.texture = ImageTexture.create_from_image(Image.create(4, 4, false, Image.FORMAT_RGBA8))
	icon.position = Vector2(0, -10)
	agent.add_child(icon)
	var building: Sprite2D = Sprite2D.new()
	building.set_meta("art_kind", "building")
	objects.add_child(building)
	var shadow: Sprite2D = Sprite2D.new()
	shadow.set_meta("agent_shadow", true)
	shadows.add_child(shadow)
	Metadata.show_agents(main, false)
	assert(not agent.visible and not shadow.visible and building.visible)
	Metadata.show_agents(main, true)
	assert(agent.visible and shadow.visible and building.visible)
	assert(Metadata.tagged_nodes(objects).size() == 2)
	var rows: Array[Dictionary] = Metadata.entities(main, Vector2(1920, 1080), Vector2.ONE)
	assert(rows.size() == 2)
	assert(is_equal_approx(float(rows[0]["modulate_alpha"]), 0.45))
	assert(is_equal_approx(float(rows[0]["self_modulate_alpha"]), 0.8))
	assert(rows[0]["world_anchor"] == [25.0, 31.0])
	assert(rows[0]["cue_rect"][3] > rows[0]["rect"][3])
	assert(rows[0]["z_index"] == 4 and rows[0]["sibling_index"] == 0)
	var rect: Rect2 = Metadata.screen_rect(Rect2(1, 2, 18, 28), Transform2D(0.0, Vector2(4, 6)), Vector2(2, 2))
	assert(rect == Rect2(10, 16, 36, 56))
	main.queue_free()
	print("capture metadata properties passed")
	quit(0)
