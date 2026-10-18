extends SceneTree
const Porter = preload("res://scripts/ui/porter_presentation_p12.gd")
var failures: int = 0
func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)
func _initialize() -> void:
	call_deferred("run")
func run() -> void:
	var main: Node = load("res://scenes/main.tscn").instantiate()
	var world: Node = main.get_node("WorldSim")
	world.set_sim_speed(0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	main.set_process(false)
	var reads: Node = main.get_node("SimReads")
	reads.reset_world_stress(15100, 200)
	var renderer: Node = main.get_node("AgentRenderer")
	renderer.set_process(false)
	var snap: Dictionary = world.get_agent_snapshot().duplicate(true)
	var index: int = 0
	var bits: int = snap["ids"][index]
	var agent: int = snap["agent_ids"][index]
	var camera: Camera2D = main.get_node("Camera2D")
	camera.zoom = Vector2(4,4)
	# Synthetic bridge-boundary rows exercise return failure and concurrent daily activity.
	var row := {"ids": PackedInt64Array([bits]), "from": PackedInt64Array([0]), "to": PackedInt64Array([0]), "kind": PackedByteArray([0]), "amount": PackedInt32Array([99]), "stage": PackedByteArray([0])}
	var settlements := {"settlement_ids": PackedInt32Array([0]), "formation_xs": PackedInt32Array([20]), "formation_ys": PackedInt32Array([24])}
	var cases: Array[Dictionary] = [{"food": 4, "wood": 0, "stage": 0, "pose": "carry"}, {"food": 0, "wood": 0, "stage": 1, "pose": "walk"}, {"food": 0, "wood": 3, "stage": 1, "pose": "carry_wood"}]
	for fixture: Dictionary in cases:
		snap["carried_foods"][index] = fixture["food"]
		snap["carried_woods"][index] = fixture["wood"]
		snap["states"][index] = 4
		row["stage"][0] = fixture["stage"]
		row["kind"][0] = 1 if int(fixture["wood"]) > 0 else 0
		reads.read_raw("get_time_snapshot")
		reads.set("_tick", int(world.get_time_snapshot()["tick"]))
		reads.get("_cache")["get_agent_snapshot"] = snap
		reads.get("_cache")["get_settlement_snapshot"] = settlements
		reads.get("_cache")["get_activity_snapshot"] = {"ids": PackedInt64Array([bits]), "kinds": PackedByteArray([1]), "place_x": PackedInt32Array([1]), "place_y": PackedInt32Array([1]), "until_tick": PackedInt64Array([999])}
		reads.get("_tick_cache")["get_porter_snapshot"] = row
		renderer._process(0.0)
		var body: Sprite2D = renderer.get("_sprites")[agent]
		check(body.get_meta("pose") == fixture["pose"], "actual renderer porter beats daily pose: " + str(fixture))
		check(body.get_meta("load_visible",false) == (fixture["pose"] != "walk"), "actual load visibility follows carried snapshot " + str(fixture) + " frame=" + str(body.get_meta("sprite_id")))
		var icon: Sprite2D = renderer.get("_activity_icons")[agent]
		check(str(icon.get_meta("sprite_id")).begins_with("p12_porter_"), "porter replaces daily icon")
		check(is_equal_approx(icon.modulate.a,0.6 if fixture["stage"] == 1 else 1.0), "return alpha")
	# Meal timer is presentation state, not fabricated simulation evidence.
	renderer.get("_pose_moments")._meal[agent] = 3.0
	renderer._process(0.0)
	var body: Sprite2D = renderer.get("_sprites")[agent]
	check(body.get_meta("pose") == "eat" and not body.get_meta("load_visible"), "meal wins and suppresses load")
	camera.zoom = Vector2.ONE
	renderer._process(0.0)
	check(renderer.get("_activity_icons")[agent].scale == Vector2(2,2), "actual low zoom icon enlarged")
	var lines: Node2D = renderer.get("_porter_lines")
	check(lines.zoom == 1.0 and lines.routes.size() == 1, "low zoom route references live porter")
	if not lines.routes.is_empty():
		check(lines.routes[0]["end"] == Vector2(448,28) + Vector2(20,24)*16 + Vector2(8,8), "route ends at current destination hearth center")
	reads.get("_tick_cache")["get_porter_snapshot"] = {}
	renderer.get("_pose_moments").clear()
	renderer._process(0.0)
	camera.zoom = Vector2(4,4)
	renderer._process(0.0)
	var icon: Sprite2D = renderer.get("_activity_icons")[agent]
	check(icon.visible and icon.modulate.a == 1.0 and icon.scale == Vector2.ONE, "daily icon recovers opacity and scale after porter disappears")
	check(lines.routes.is_empty(), "departed porter leaves no route")
	renderer._on_world_reset()
	check(lines.routes.is_empty(), "world reset clears routes")
	main.free()
	print("PORTER_RENDERER_P12 failures=",failures)
	quit(1 if failures else 0)
