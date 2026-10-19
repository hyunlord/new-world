extends SceneTree

var failures: int = 0

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)

func _initialize() -> void:
	_run.call_deferred()

func _run() -> void:
	var scene: PackedScene = load("res://scenes/main.tscn") as PackedScene
	var main: Node = scene.instantiate()
	root.add_child(main)
	await process_frame
	await process_frame
	main.process_mode = Node.PROCESS_MODE_DISABLED
	var agents: Node = main.get_node("AgentRenderer")
	if not agents.has_method("drawn_entities_at"):
		check(false, "crowd selection requires every opaque hit")
		main.free()
		quit(1)
		return
	var world: Node = main.get_node("WorldSim")
	var clicks: Node = main.get_node("WorldRenderer")
	var inspector: Node = main.get_node("UI/AgentInspectorPanel")
	var camera: Camera2D = main.get_node("Camera2D") as Camera2D
	var snapshot: Dictionary = SimReads.read(world, "get_agent_snapshot")
	var ids: PackedInt64Array = snapshot["ids"]
	var agent_ids: PackedInt64Array = snapshot["agent_ids"]
	check(ids.size() >= 3, "fixed world provides three real people")
	var sprites: Dictionary = agents.get("_sprites")
	var drawn: Dictionary = agents.get("_drawn_positions")
	var icons: Dictionary = agents.get("_activity_icons")
	for sprite: Sprite2D in sprites.values():
		sprite.visible = false
	var pixels := Image.create(12, 16, false, Image.FORMAT_RGBA8)
	pixels.fill(Color.WHITE)
	var texture := ImageTexture.create_from_image(pixels)
	var position := Vector2(960, 540)
	for index: int in 3:
		var sprite: Sprite2D = sprites[agent_ids[index]]
		sprite.texture = texture
		sprite.region_enabled = false
		sprite.centered = false
		sprite.offset = Vector2(-6, -15)
		sprite.position = position
		sprite.visible = true
		sprite.set_meta("sprite_id", "p12_crowd_fixture")
		drawn[agent_ids[index]] = position
		if not icons.has(agent_ids[index]):
			var icon := Sprite2D.new()
			sprite.add_child(icon)
			icons[agent_ids[index]] = icon
	var point: Vector2 = position + Vector2(0, -5)
	var order: Array[int] = [0, 1, 2]
	order.sort_custom(func(a: int, b: int) -> bool: return agent_ids[a] > agent_ids[b])
	var expected := PackedInt64Array()
	for index: int in order:
		expected.append(ids[index])
	check(agents.call("drawn_entities_at", point) == expected, "equal y ties use descending stable AgentId")
	camera.zoom = Vector2(4, 4)
	camera.force_update_scroll()
	for turn: int in 4:
		check(bool(clicks.call("_try_agent_click", point)), "crowd click accepted")
		check(int(inspector.call("get_selected_entity_bits")) == expected[turn % 3], "repeated clicks cycle and wrap")
		camera.zoom = Vector2.ONE * (1.0 if turn % 2 == 0 else 4.0)
		agents.call("_update_selection_ring")
		var selected_index: int = order[turn % 3]
		for index: int in 3:
			var selected: bool = index == selected_index
			check((sprites[agent_ids[index]] as Sprite2D).z_index == (1 if selected else 0), "selected body raises and previous body restores")
			check((icons[agent_ids[index]] as Sprite2D).z_index == (1 if selected else 0), "selected icon raises and previous icon restores")
		var ring: Line2D = agents.get("_selection_ring") as Line2D
		check(is_equal_approx(ring.width, 2.0 if turn % 2 == 0 else 1.5), "ring width follows zoom band")
		camera.zoom = Vector2(4, 4)
		camera.force_update_scroll()
	clicks.set("_last_agent_click_usec_p12", Time.get_ticks_usec() - 1500001)
	clicks.call("_try_agent_click", point)
	check(int(inspector.call("get_selected_entity_bits")) == expected[0], "expired click starts at front")
	(sprites[agent_ids[order[0]]] as Sprite2D).visible = false
	check(agents.call("drawn_entities_at", point) == expected.slice(1), "hidden person excluded from complete hit list")
	main.free()
	print("CROWDS_P12 failures=", failures)
	quit(1 if failures else 0)
