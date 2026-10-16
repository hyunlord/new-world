extends SceneTree

var _failures: int = 0

func _initialize() -> void:
	call_deferred("_run")

func _check(condition: bool, message: String) -> void:
	if not condition:
		_failures += 1
		push_error(message)

func _run() -> void:
	var scene: PackedScene = load("res://scenes/main.tscn") as PackedScene
	var main: Node = scene.instantiate()
	root.add_child(main)
	await process_frame
	await process_frame
	main.process_mode = Node.PROCESS_MODE_DISABLED
	var world: WorldSimNode = main.get_node("WorldSim") as WorldSimNode
	var agents: Node = main.get_node("AgentRenderer")
	var renderer: Node = main.get_node("WorldRenderer")
	var inspector: Node = main.get_node("UI/AgentInspectorPanel")
	var snapshot: Dictionary = SimReads.read(world, "get_agent_snapshot")
	var ids: PackedInt64Array = snapshot["ids"]
	var agent_ids: PackedInt64Array = snapshot["agent_ids"]
	_check(ids.size() >= 2, "real world supplies two selectable villagers")
	if ids.size() < 2:
		main.free()
		quit(1)
		return
	var sprites: Dictionary = agents.get("_sprites")
	for sprite: Sprite2D in sprites.values():
		sprite.visible = false
	var first: Sprite2D = agents.call("get_agent_sprite", agent_ids[0]) as Sprite2D
	var second: Sprite2D = agents.call("get_agent_sprite", agent_ids[1]) as Sprite2D
	var xs: PackedInt32Array = snapshot["xs"]
	var ys: PackedInt32Array = snapshot["ys"]
	var click: Vector2 = Vector2(456 + 16 * xs[1], 36 + 16 * ys[1])
	# A small asymmetric native frame also exercises feet anchors and flips.
	var pixels: Image = Image.create(12, 16, false, Image.FORMAT_RGBA8)
	pixels.fill_rect(Rect2i(2, 3, 5, 12), Color.WHITE)
	first.texture = ImageTexture.create_from_image(pixels)
	first.region_enabled = false
	first.centered = false
	first.offset = Vector2(-6, -15)
	first.set_meta("sprite_id", "click_test_first")
	first.visible = true
	first.position = click + Vector2(2, 5)
	var drawn: Dictionary = agents.get("_drawn_positions")
	drawn[agent_ids[0]] = first.position
	_check(bool(renderer.call("_try_agent_click", click)), "drawn villager click accepted")
	_check(int(inspector.call("get_selected_entity_bits")) == ids[0], "drawn resident beats another villager's exact tile centre")
	first.flip_h = true
	var flipped_click: Vector2 = first.position + Vector2(1, -5)
	_check(int(agents.call("drawn_entity_at", flipped_click)) == ids[0], "flipped native opaque bounds selectable")
	first.flip_h = false
	second.texture = first.texture
	second.region_enabled = false
	second.centered = false
	second.offset = first.offset
	second.set_meta("sprite_id", "click_test_second")
	second.position = first.position + Vector2(0, 1)
	second.visible = true
	drawn[agent_ids[1]] = second.position
	_check(int(agents.call("drawn_entity_at", click)) == ids[1], "overlap selects larger drawn y")
	second.visible = false
	_check(int(agents.call("drawn_entity_at", click)) == ids[0], "hidden sprite cannot take drawn click")
	first.visible = false
	_check(bool(renderer.call("_try_agent_click", click)), "empty drawn hit falls back to tile-centre selection")
	var expected_fallback: int = -1
	for index: int in ids.size():
		if xs[index] == xs[1] and ys[index] == ys[1]:
			expected_fallback = ids[index]
			break
	_check(int(inspector.call("get_selected_entity_bits")) == expected_fallback, "legacy nearest-centre result and first-index tie preserved")
	var seek: Node = main.get_node("SeekVizRenderer")
	seek.call("_process", 0.0)
	_check(seek.call("_agent_origin", 0) == first.position, "seek origin uses agent id rather than entity bits")
	var trail: Node = main.get_node("ActivityTrailRenderer")
	trail.call("_process", 0.0)
	var history: Dictionary = trail.get("_history")
	_check(history[agent_ids[0]].back() == first.position, "trail latest point follows drawn position")
	drawn.erase(agent_ids[0])
	_check(seek.call("_agent_origin", 0) == Vector2(456 + 16 * xs[0], 36 + 16 * ys[0]), "seek missing drawn position retains tile centre")
	trail.call("_process", 0.0)
	_check(history[agent_ids[0]].back() == Vector2(456 + 16 * xs[0], 36 + 16 * ys[0]), "trail missing drawn position retains tile centre")
	main.free()
	print("AGENT DRAWN CLICK failures=", _failures)
	quit(1 if _failures else 0)
