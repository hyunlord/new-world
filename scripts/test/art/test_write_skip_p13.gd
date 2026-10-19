extends SceneTree

var failures: int = 0
var renderer: Node
var main: Node
var reads: Node
var world: Node
var snap: Dictionary
var agent: int
var bits: int
var porter: Dictionary

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)

func _initialize() -> void:
	call_deferred("run")

func counter(key: String) -> int:
	return int(renderer.get_perf_candidate_p13()[key])

func body() -> Sprite2D:
	return renderer.get("_sprites")[agent] as Sprite2D

func snapshot(sprite: Sprite2D) -> Dictionary:
	var metadata: Dictionary = {}
	for key: StringName in sprite.get_meta_list():
		metadata[key] = sprite.get_meta(key)
	var children: Array = []
	for child: Node in sprite.get_children():
		if child is Sprite2D:
			children.append(snapshot(child as Sprite2D))
	return {"texture": sprite.texture, "material": sprite.material, "position": sprite.position,
		"offset": sprite.offset, "region": sprite.region_rect, "region_enabled": sprite.region_enabled,
		"centered": sprite.centered, "flip_h": sprite.flip_h, "visible": sprite.visible,
		"scale": sprite.scale, "modulate": sprite.modulate, "self_modulate": sprite.self_modulate,
		"z": sprite.z_index, "metadata": metadata.duplicate(true), "children": children}

func parity(label: String) -> void:
	# Selection raises z after property writes; compare once that original ordering settles.
	renderer._process(0.0)
	var cached: Dictionary = snapshot(body())
	renderer.set_perf_candidate_p13("write-skip", false)
	renderer._process(0.0)
	check(snapshot(body()) == cached, label + ": cached output equals original full-write path")
	renderer.set_perf_candidate_p13("write-skip", true)
	renderer._process(0.0)

func expect_miss(label: String) -> void:
	var before: int = counter("misses")
	renderer._process(0.0)
	check(counter("misses") == before + 1, label + ": full property write")
	parity(label)

func expect_hit(label: String) -> void:
	renderer._process(0.0)
	var before: int = counter("hits")
	renderer._process(0.0)
	check(counter("hits") == before + 1, label + ": equal-input hit")
	check(renderer.get("_selection_index").find(renderer.get("_sprites"), bits) == body(), label + ": selection observation stays live")
	parity(label)

func run() -> void:
	main = load("res://scenes/main.tscn").instantiate()
	world = main.get_node("WorldSim")
	world.set_sim_speed(0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	main.set_process(false)
	reads = main.get_node("SimReads")
	SimReads.mutate(world, "reset_world_stress", [15100, 200])
	renderer = main.get_node("AgentRenderer")
	renderer.set_process(false)
	renderer.set("_activity_oracle_enabled", true)
	var full: Dictionary = SimReads.read(world, "get_agent_snapshot")
	snap = full.duplicate(true)
	for key: Variant in snap:
		if typeof(snap[key]) in [TYPE_PACKED_BYTE_ARRAY, TYPE_PACKED_INT32_ARRAY, TYPE_PACKED_INT64_ARRAY, TYPE_PACKED_FLOAT32_ARRAY, TYPE_PACKED_FLOAT64_ARRAY, TYPE_ARRAY]:
			snap[key] = snap[key].slice(0, 1)
	agent = int(snap["agent_ids"][0])
	bits = int(snap["ids"][0])
	snap["states"][0] = 4
	snap["carried_foods"][0] = 4.0
	snap["carried_woods"][0] = 0
	porter = {"ids": PackedInt64Array([bits]), "from": PackedInt64Array([0]), "to": PackedInt64Array([0]), "kind": PackedByteArray([0]), "amount": PackedInt32Array([99]), "stage": PackedByteArray([0])}
	reads.set("_tick", int(SimReads.read(world, "get_time_snapshot")["tick"]))
	reads.get("_cache")["get_agent_snapshot"] = snap
	reads.get("_cache")["get_settlement_snapshot"] = {"settlement_ids": PackedInt32Array([0]), "formation_xs": PackedInt32Array([20]), "formation_ys": PackedInt32Array([24])}
	reads.get("_cache")["get_society_snapshot"] = {"founder_agent_ids": PackedInt64Array([agent])}
	reads.get("_cache")["get_activity_snapshot"] = {"ids": PackedInt64Array([bits]), "kinds": PackedByteArray([1]), "place_x": PackedInt32Array([1]), "place_y": PackedInt32Array([1]), "until_tick": PackedInt64Array([999])}
	reads.get("_tick_cache")["get_porter_snapshot"] = porter
	var camera: Camera2D = main.get_node("Camera2D")
	camera.zoom = Vector2(4, 4)
	var kernel: Dictionary = renderer.get_perf_candidate_p12()
	check(renderer.set_perf_candidate_p13("write-skip", true), "write-skip setter supported")
	check(not renderer.set_perf_candidate_p13("declump-kernel", true), "P13 setter cannot toggle kernel")
	check(renderer.get_perf_candidate_p12() == kernel, "P12 kernel state unchanged")
	check(renderer.get_perf_candidate_p13()["kernel_enabled"] == kernel["candidate_enabled"], "kernel readback preserved")
	expect_miss("first enabled frame")
	expect_hit("stable carry")
	var saved_key: Array = renderer.get("_write_keys_p13")[agent].duplicate(true)
	renderer.get("_pose_output")["pickup"] = true
	body().get_meta("activity_overrides").append("alias_probe")
	check(renderer.get("_write_keys_p13")[agent] == saved_key, "owned key does not alias reused pose or metadata overrides")
	body().get_meta("activity_overrides").erase("alias_probe")
	var before_hits: int = counter("hits")
	var before_time: float = renderer.get("_animation_time")
	var before_stationary: float = renderer.get("_stationary_seconds")[agent]
	renderer._process(0.000001)
	check(counter("hits") == before_hits + 1, "nonzero delta retains equal resolved-input hit")
	check(float(renderer.get("_animation_time")) > before_time, "animation clock runs on hit")
	check(float(renderer.get("_stationary_seconds")[agent]) > before_stationary, "stationary timer runs on hit")
	check(float(body().get_meta("activity_oracle_inputs")["animation_seconds"]) == float(renderer.get("_animation_time")), "oracle refreshed on hit")
	check(body().get_meta("drawn_position") == body().position, "post-loop geometry remains live")
	check(renderer.get("_porter_lines").routes.size() == 1, "route rebuilt on hit")
	var inspector: Node = main.get_node("UI/AgentInspectorPanel")
	inspector.set("_selected_entity_bits", bits)
	expect_miss("selected agent")
	expect_miss("selected agent repeat")
	inspector.set("_selected_entity_bits", -1)
	renderer._process(0.0)
	expect_miss("deselection restores carry z input")
	var forced: int = counter("forced")
	renderer.refresh_snapshot()
	check(counter("forced") == forced + 1, "refresh invalidates before synchronous process")
	parity("refresh")
	root.get_node("Locale").emit_signal("language_changed", "en")
	expect_miss("locale signal")
	for shown: bool in [false, true]:
		forced = counter("forced")
		renderer.set_capture_visible(shown)
		check(counter("forced") == forced + 1, "capture transition forces synchronous write")
		check(body().visible == shown, "capture visibility current")
		parity("capture " + str(shown))
	var objects: CanvasItem = body().get_parent() as CanvasItem
	objects.hide()
	expect_miss("parent hidden")
	objects.show()
	expect_miss("parent visible")
	renderer.set_perf_candidate_p10("allocation-cache", true)
	expect_miss("allocation reset")
	for fixture: Dictionary in [{"food": 0.0, "wood": 0, "stage": 1, "kind": 0}, {"food": 0.0, "wood": 3, "stage": 1, "kind": 1}, {"food": 4.0, "wood": 0, "stage": 0, "kind": 0}]:
		snap["carried_foods"][0] = fixture["food"]
		snap["carried_woods"][0] = fixture["wood"]
		porter["stage"][0] = fixture["stage"]
		porter["kind"][0] = fixture["kind"]
		expect_miss("carry and founder inputs " + str(fixture))
		check(str(body().get_meta("founder_bundle_suppressed")) == ("real_load" if float(fixture["food"]) > 0 or int(fixture["wood"]) > 0 else ""), "founder suppression uses current load")
	camera.zoom = Vector2.ONE
	expect_miss("porter zoom")
	camera.zoom = Vector2(4, 4)
	expect_miss("porter zoom restored")
	reads.get("_tick_cache")["get_porter_snapshot"] = {}
	expect_miss("porter to daily")
	reads.get("_tick_cache")["get_porter_snapshot"] = porter
	expect_miss("daily to porter")
	renderer.get("_pose_moments")._meal[agent] = 3.0
	expect_miss("meal override")
	check(body().get_meta("pose") == "eat" and not body().get_meta("load_visible"), "meal hides real load")
	renderer.get("_pose_moments").clear()
	expect_miss("meal ends")
	for child_name: String in ["CarryLoad", "FounderBundle"]:
		var child: Node = body().get_node_or_null(child_name)
		if child != null:
			child.free()
			expect_miss("child lifetime " + child_name)
	var icon: Node = renderer.get("_activity_icons")[agent]
	icon.free()
	expect_miss("icon recreation")
	var changed_look: Dictionary = renderer.get("_base_looks")[agent].duplicate(true)
	renderer.get("_base_looks")[agent] = changed_look
	var old_tone: int = int(changed_look["tone"])
	renderer.get("_base_looks")[agent]["tone"] = (old_tone + 1) % 16
	expect_miss("variant tone")
	var old_hair: int = int(renderer.get("_base_looks")[agent]["hair"])
	renderer.get("_base_looks")[agent]["hair"] = (old_hair + 1) % 16
	expect_miss("variant hair")
	snap["life_stages"][0] = 1
	expect_miss("child frame and geometry")
	snap["life_stages"][0] = 0
	expect_miss("adult frame and geometry")
	snap["xs"][0] += 1
	expect_miss("tile metadata")
	body().set_meta("facing", "W")
	expect_miss("facing input")
	var original: Sprite2D = body()
	renderer._retire_missing({})
	expect_miss("pool reuse same art id")
	check(body() == original, "test exercised actual pooled body reuse")
	renderer._retire_missing({})
	renderer.get("_sprite_pool").clear()
	expect_miss("new sprite with same semantic inputs")
	check(body() != original, "test exercised newly allocated body")
	forced = counter("forced")
	renderer._on_world_reset()
	check(counter("forced") == forced + 1, "world reset invalidates before synchronous process")
	parity("world reset")
	renderer.set_perf_candidate_p13("write-skip", false)
	renderer._process(0.0)
	renderer.set_perf_candidate_p13("write-skip", true)
	expect_miss("switch off then on")
	main.free()
	print("WRITE_SKIP_P13 failures=", failures)
	quit(1 if failures else 0)
