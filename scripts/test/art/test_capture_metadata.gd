extends SceneTree

const Metadata = preload("res://scripts/test/art/capture_metadata.gd")


func _init() -> void:
	call_deferred("_run")


func _run() -> void:
	var main: Node2D = Node2D.new()
	root.add_child(main)
	var objects: Node2D = Node2D.new()
	objects.name = "WorldObjects"
	main.add_child(objects)
	var shadows: Node2D = Node2D.new()
	shadows.name = "Shadows"
	main.add_child(shadows)
	var agent: Sprite2D = Sprite2D.new()
	agent.set_meta("art_kind", "agent")
	agent.set_meta("art_id", 17)
	objects.add_child(agent)
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
	var rect: Rect2 = Metadata.screen_rect(Rect2(1, 2, 18, 28), Transform2D(0.0, Vector2(4, 6)), Vector2(2, 2))
	assert(rect == Rect2(10, 16, 36, 56))
	main.queue_free()
	print("capture metadata properties passed")
	quit(0)
