extends SceneTree

const Metadata = preload("res://scripts/test/art/capture_metadata.gd")
const Plans = preload("res://scripts/test/art/capture_plans.gd")


class CaptureRenderer:
	extends Node
	var calls: int = 0
	var managed: Array[CanvasItem] = []

	func set_capture_visible(shown: bool) -> void:
		calls += 1
		for item: CanvasItem in managed:
			item.visible = shown and str(item.get_meta("capture_state", "")) == "live"


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
	agent.set_meta("pose", "sit")
	agent.set_meta("drawn_position", Vector2(25, 31))
	agent.set_meta("seated", true)
	objects.add_child(agent)
	agent.position = Vector2(25, 31)
	agent.modulate.a = 0.45
	agent.self_modulate.a = 0.8
	agent.modulate.r = 0.25
	agent.self_modulate.b = 0.5
	agent.texture = ImageTexture.create_from_image(Image.create(8, 8, false, Image.FORMAT_RGBA8))
	var icon: Sprite2D = Sprite2D.new()
	icon.texture = ImageTexture.create_from_image(Image.create(4, 4, false, Image.FORMAT_RGBA8))
	icon.position = Vector2(0, -10)
	agent.add_child(icon)
	var building: Sprite2D = Sprite2D.new()
	building.set_meta("art_kind", "building")
	building.set_meta("fire_lit", false)
	building.set_meta("woodpile_stage", 2)
	building.set_meta("glow_alpha", 0.0)
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
	assert(rows[0]["pose"] == "sit")
	assert(rows[0]["drawn_position"] == [25.0, 31.0] and rows[0]["seated"] == true)
	assert(rows[1]["fire_lit"] == false and rows[1]["woodpile_stage"] == 2)
	assert(rows[1]["glow_alpha"] == 0.0)
	assert(is_equal_approx(float(rows[0]["modulate"][0]), 0.25))
	assert(is_equal_approx(float(rows[0]["self_modulate"][2]), 0.5))
	assert(Metadata.life_stage_counts({"life_stages": PackedByteArray([0, 1, 2, 0, 7])}) == {"adult": 2, "child": 1, "elder": 1, "unknown": 1})
	assert(Metadata.life_stage_counts({}) == {"adult": 0, "child": 0, "elder": 0, "unknown": 0})
	assert(Plans.ART.size() == 10 and Plans.ART_LONG.size() == 9)
	assert(Plans.ART_LONG[1] == [10, "15:00", "F2"] and Plans.ART_LONG[2] == [10, "15:00", "F5"])
	assert(Metadata.deadwood_tile({}, Vector2.ZERO) == Vector2(-1, -1))
	assert(Metadata.deadwood_tile({
		"kinds": PackedInt32Array([0, 3, 3, 3, 3]),
		"xs": PackedInt32Array([10, 8, 12, 11, 9]),
		"ys": PackedInt32Array([10, 10, 8, 8, 8]),
	}, Vector2(10, 10)) == Vector2(9, 8))
	var previous: int = -1
	for shot: Array in Plans.ART_LONG:
		var clock: PackedStringArray = str(shot[1]).split(":")
		var minute: int = (int(shot[0]) - 1) * 1440 + int(clock[0]) * 60 + int(clock[1])
		assert(minute >= previous)
		previous = minute
	assert(rows[0]["world_anchor"] == [25.0, 31.0])
	assert(rows[0]["cue_rect"][3] > rows[0]["rect"][3])
	assert(rows[0]["z_index"] == 4 and rows[0]["sibling_index"] == 0)
	var rect: Rect2 = Metadata.screen_rect(Rect2(1, 2, 18, 28), Transform2D(0.0, Vector2(4, 6)), Vector2(2, 2))
	assert(rect == Rect2(10, 16, 36, 56))
	var renderer: CaptureRenderer = CaptureRenderer.new()
	renderer.name = "AgentRenderer"
	main.add_child(renderer)
	for state: String in ["live", "retired", "sheltered"]:
		var managed_agent: Sprite2D = Sprite2D.new()
		managed_agent.set_meta("art_kind", "agent")
		managed_agent.set_meta("capture_state", state)
		managed_agent.visible = state == "live"
		objects.add_child(managed_agent)
		renderer.managed.append(managed_agent)
		var managed_shadow: Sprite2D = Sprite2D.new()
		managed_shadow.set_meta("agent_shadow", true)
		managed_shadow.set_meta("capture_state", state)
		managed_shadow.visible = state == "live"
		shadows.add_child(managed_shadow)
		renderer.managed.append(managed_shadow)
	var failures: int = 0
	for shown: bool in [false, true]:
		Metadata.show_agents(main, shown)
		for managed: CanvasItem in renderer.managed:
			var expected: bool = shown and str(managed.get_meta("capture_state")) == "live"
			if managed.visible != expected:
				push_error("Capture overrode renderer visibility for " + str(managed.get_meta("capture_state")))
				failures += 1
	if renderer.calls != 2:
		push_error("Capture must delegate visibility exactly once per change")
		failures += 1
	main.queue_free()
	print("capture metadata failures=", failures)
	quit(1 if failures > 0 else 0)
