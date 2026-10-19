extends SceneTree
const Activity = preload("res://scripts/ui/agent_activity.gd")
const Library = preload("res://scripts/ui/art_sprite_library.gd")
var _failures: int = 0

func _check(condition: bool, message: String = "activity invariant") -> void:
	if not condition:
		push_error(message)
		_failures += 1

func _initialize() -> void:
	call_deferred("_run")

func _run() -> void:
	_test_copied_carry()
	_test_wood()
	_test_frame_lookup()
	_test_carry_other_views()
	var shelter: Dictionary = {"shelter": true}
	_check(Activity.resolve(3, 3, true, false, 0.0, shelter, 23.0) == "asleep")
	_check(Activity.hidden("asleep", true, shelter))
	_check(not Activity.hidden("asleep", false, shelter))
	_check(not Activity.hidden("asleep", true, {}))
	_check(Activity.resolve(3, 0, false, false, 0.0, shelter, 12.0) == "asleep")
	_check(Activity.resolve(3, 0, false, false, 0.0, {"construction": true, "food": true}, 12.0) == "build")
	_check(Activity.resolve(3, 0, false, false, 0.0, {"food": true, "water": true}, 12.0) == "gather")
	_check(Activity.resolve(3, 0, false, false, 0.0, {"water": true}, 12.0) == "drink")
	_check(Activity.resolve(3, 1, true, false, 0.0, shelter, 12.0) == "eat")
	_check(Activity.resolve(3, 6, true, false, 0.0, {}, 12.0) == "gather")
	_check(Activity.resolve(2, 0, false, true, 1.0, {}, 12.0) == "talk")
	_check(Activity.resolve(1, 0, false, false, 1.0, {}, 12.0) == "carry")
	_check(Activity.resolve(0, 0, false, true, 0.0, {}, 12.0) == "walk")
	_check(Activity.resolve(0, 0, false, false, 0.0, {"near_hearth": true}, 18.0) == "idle")
	_check(Activity.resolve(0, 0, false, false, 0.0, {"near_hearth": true}, 23.0) == "idle")
	_check(Activity.facing(Vector2.ZERO, "N") == "N")
	_check(Activity.facing(Vector2(-2, 1), "S") == "W")
	_check(Activity.facing(Vector2(0, -2), "S") == "N")
	_check(Activity.frame_index("walk", 0.125, 0.0, 4) == 1)
	_check(Activity.frame_index("idle", 0.7, 0.0, 2) == 1)
	_check(Activity.frame_index("build", 0.25, 0.0, 2) == 1)
	var sprite: Sprite2D = Sprite2D.new()
	_check(Library.apply(sprite, "direction_09_villager_gatherer"))
	_check(Library.apply_variant(sprite, {"tone": 2, "hair": 4}))
	_check(roundi(sprite.self_modulate.r * 15.0) == 2)
	_check(roundi(sprite.self_modulate.g * 15.0) == 4)
	_check(sprite.self_modulate.b == 0.0)
	_check(sprite.material is ShaderMaterial)
	sprite.free()
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var world: Node = main.get_node("WorldSim")
	world.call("set_sim_speed", 0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	await process_frame
	var renderer: Node = main.get_node("AgentRenderer")
	renderer.call("refresh_snapshot")
	var context: Dictionary = renderer.call("_activity_context")
	_check(context.has("food") and context.has("hearths") and context.has("terrain"))
	_check((context["terrain"] as Dictionary).has("terrain"))
	var snapshot: Dictionary = SimReads.read(world, "get_agent_snapshot")
	var ids: PackedInt64Array = snapshot["ids"]
	var inspector: Node = main.get_node("UI/AgentInspectorPanel")
	inspector.call("display_agent", world.call("get_agent_detail", ids[0]))
	renderer.call("_update_selection_ring")
	var ring: Line2D = main.get_node("Shadows/SelectionRing") as Line2D
	_check(ring.visible)
	_check(ring.width == 1.5, "P12 selection ring is 1.5 px at default Z4")
	_check(ring.get_parent().name == "Shadows")
	var agent_ids: PackedInt64Array = snapshot["agent_ids"]
	var selected_sprite: Sprite2D = renderer.call("get_agent_sprite", int(agent_ids[0])) as Sprite2D
	_check(ring.position == selected_sprite.position)
	var body: String = str(selected_sprite.get_meta("body"))
	var icon_choice: Dictionary = renderer.call("frame_choice", body, "carry", "S", 0.0)
	if _has_real_carry(Activity.build_frame_groups(Library.entries()),body):
		_check(icon_choice.get("fallback_kind","") != "idle_icon")
		_check(not Activity.show_activity_icon(icon_choice,4.0))
	icon_choice = {"fallback_kind":"idle_icon","activity_icon":"U1_05"}
	var icon: Sprite2D = Sprite2D.new()
	selected_sprite.add_child(icon)
	_check(not Activity.apply_activity_icon(selected_sprite, icon, icon_choice, 1.0))
	_check(not icon.visible)
	_check(Activity.apply_activity_icon(selected_sprite, icon, icon_choice, 2.0))
	_check(icon.visible and icon.texture != null)
	_check(icon.position == Vector2(0, selected_sprite.offset.y - 2.0))
	_check(str(icon.get_meta("sprite_id")) == "U1_05")
	_check(not Activity.apply_activity_icon(selected_sprite, icon, {}, 4.0))
	_check(not icon.visible)
	icon.queue_free()
	renderer.set("_animation_time", 0.0)
	renderer.call("_update_selection_ring")
	_check(int(ring.get_meta("frame_id")) == 0)
	renderer.set("_animation_time", 0.5)
	renderer.call("_update_selection_ring")
	_check(int(ring.get_meta("frame_id")) == 1)
	inspector.call("close")
	renderer.call("_update_selection_ring")
	_check(not ring.visible)
	main.free()
	print("ACTIVITY ", "PASS" if _failures == 0 else "FAIL", ": failures=", _failures)
	quit(0 if _failures == 0 else 1)

func _test_frame_lookup() -> void:
	var entries: Dictionary = {
		"a_idle": {"body": "adult_a", "activity": "idle", "view": "S", "frame": 0, "qa_status": "accepted", "contains_basket": false},
		"a_walk_basket": {"body": "adult_a", "activity": "walk", "view": "S", "frame": 0, "qa_status": "fallback", "contains_basket": true},
		"a_carry": {"body": "adult_a", "activity": "carry", "view": "S", "frame": 0, "qa_status": "accepted", "contains_basket": true},
		"b_build": {"body": "adult_b", "activity": "build", "view": "S", "frame": 0, "qa_status": "accepted", "contains_basket": false},
		"reference": {"body": "adult_a", "activity": "build", "view": "S", "frame": 0, "qa_status": "provisional", "contains_basket": false},
	}
	var groups: Dictionary = Activity.build_frame_groups(entries)
	var walk: Dictionary = Activity.choose_frame(groups, "adult_a", "walk", "S", 0.0, 0.0)
	_check(walk["sprite_id"] == "a_idle")
	_check(walk["fallback_kind"] == "idle_icon")
	_check(not Activity.show_activity_icon(walk, 1.0))
	_check(not Activity.show_activity_icon(walk, 1.99))
	_check(Activity.show_activity_icon(walk, 2.0))
	var carry: Dictionary = Activity.choose_frame(groups, "adult_a", "carry", "S", 0.0, 0.0)
	_check(carry["sprite_id"] == "a_carry")
	_check(not Activity.show_activity_icon(carry, 4.0))
	var build: Dictionary = Activity.choose_frame(groups, "adult_a", "build", "S", 0.0, 0.0)
	_check(build["sprite_id"] == "a_idle")
	_check(Activity.choose_frame(groups, "child", "build", "S", 0.0, 0.0).is_empty())
	_check(Activity.choose_frame(groups, "adult_a", "walk", "N", 0.0, 0.0).is_empty())
	var actual: Dictionary = Activity.build_frame_groups(Library.entries())
	for body: String in ["adult_a", "adult_b", "child"]:
		for activity: String in ["idle", "walk", "carry", "build", "gather", "talk", "sit", "drink", "asleep"]:
			for view: String in ["S", "N", "E", "W"]:
				var choice: Dictionary = Activity.choose_frame(actual, body, activity, view, 1.375, 0.0)
				if choice.is_empty():
					continue
				_check(choice["body"] == body)
				if activity == "carry" and _has_real_carry(actual,body):
					_check(choice.get("fallback_kind","") != "idle_icon","live carry uses real frames in every facing")
					_check(not Activity.show_activity_icon(choice,4.0))
				_check(not bool(choice.get("contains_basket", false)) or activity == "carry")
				_check(not str(choice["sprite_id"]).begins_with("direction_"))
				_check(Activity.show_activity_icon(choice, 4.0) == (str(choice.get("fallback_kind", "")) == "idle_icon" and not str(choice.get("activity_icon", "")).is_empty()))

func _has_real_carry(groups: Dictionary, body: String) -> bool:
	for view: String in ["S","E","N","W"]:
		for entry: Dictionary in groups.get(body+"/carry/"+view,[]):
			if entry.get("qa_status","") == "accepted":
				return true
	return false

func _test_carry_other_views() -> void:
	for view: String in ["E","N","W"]:
		var entries: Dictionary = {
			"idle": {"body":"adult_a","activity":"idle","view":"S","qa_status":"accepted"},
			"old_placeholder": {"body":"adult_a","activity":"carry","view":"S","qa_status":"fallback","fallback_kind":"idle_icon","activity_icon":"U1_05"},
			"real_carry": {"body":"adult_a","activity":"carry","view":view,"qa_status":"accepted","contains_basket":true},
		}
		var groups: Dictionary = Activity.build_frame_groups(entries)
		var choice: Dictionary = Activity.choose_frame(groups,"adult_a","carry","S",0.0,0.0)
		_check(choice.get("sprite_id","") == "real_carry","carry borrows real same-body view before installed idle icon")
		_check(not Activity.show_activity_icon(choice,4.0))
		groups.erase("adult_a/carry/S")
		choice = Activity.choose_frame(groups,"adult_a","carry","S",0.0,0.0)
		_check(choice.get("sprite_id","") == "real_carry","missing carry group borrows any real view before idle")
		_check(not Activity.show_activity_icon(choice,4.0))

func _test_wood() -> void:
	for legacy: Dictionary in [{}, {"target_kinds": [7], "carried_woods": [10]}, {"target_kinds": PackedByteArray(), "carried_woods": PackedInt32Array()}]:
		_check(Activity.snapshot_target(legacy, 0) == -1)
		_check(Activity.snapshot_wood(legacy, 0) == 0)
	var typed: Dictionary = {"target_kinds": PackedByteArray([7]), "carried_woods": PackedInt32Array([10])}
	_check(Activity.snapshot_target(typed, 0) == 7 and Activity.snapshot_wood(typed, 0) == 10)
	_check(Activity.snapshot_target(typed, 1) == -1 and Activity.snapshot_wood(typed, 1) == 0)
	_check(Activity.resolve(3, 7, true, false, 0.0, {}, 12.0) == "gather_wood")
	_check(Activity.resolve(3, 0, false, false, 0.0, {"wood": true, "food": true}, 12.0) == "gather_wood")
	_check(Activity.resolve(3, 6, true, false, 0.0, {"wood": true}, 12.0) == "gather")
	_check(Activity.resolve(1, 0, false, true, 10.0, {}, 12.0, 2) == "carry_wood")
	_check(Activity.resolve(1, 0, false, true, 10.0, {}, 12.0, 0) == "carry")
	_check(Activity.wood_icon({}) == "S2_15")
	_check(Activity.wood_icon({"W1_12": {"qa_status": "rejected"}, "W1_16": {"qa_status": "accepted"}}) == "W1_16")
	_check(Activity.wood_icon({"W1_12": {"qa_status": "accepted"}, "W1_16": {"qa_status": "accepted"}}) == "W1_12")
	_check(Activity.wood_icon({"W1_16": {"qa_status": "accepted", "drawable": false}}) == "S2_15")
	_check(Activity.wood_icon({"W1_12": {"qa_status": "provisional"}}) == "S2_15")
	_check(Activity.wood_icon({"W1_12": {"qa_status": "accepted"}}) == "W1_12")
	_check(Activity.wood_icon({"W1_12": {"qa_status": "accepted", "drawable": false}}) == "S2_15")
	var entries: Dictionary = {
		"own": {"body": "adult_a", "activity": "carry_wood", "view": "N", "qa_status": "accepted"},
		"south": {"body": "adult_a", "activity": "carry_wood", "view": "S", "qa_status": "accepted"},
		"walk": {"body": "adult_a", "activity": "walk", "view": "S", "qa_status": "accepted"},
		"idle": {"body": "adult_a", "activity": "idle", "view": "S", "qa_status": "accepted"},
		"crouch": {"body": "adult_a", "activity": "crouch", "view": "S", "qa_status": "accepted"},
		"bush": {"body": "adult_a", "activity": "gather_wood", "view": "S", "qa_status": "accepted", "contains_bush": true},
		"basket": {"body": "adult_a", "activity": "gather_wood", "view": "S", "qa_status": "accepted", "contains_basket": true},
	}
	var groups: Dictionary = Activity.build_frame_groups(entries)
	_check(Activity.choose_frame(groups, "adult_a", "carry_wood", "N", 0.0, 0.0)["sprite_id"] == "own")
	_check(Activity.choose_frame(groups, "adult_a", "carry_wood", "E", 0.0, 0.0)["sprite_id"] == "south")
	groups.erase("adult_a/carry_wood/S")
	var choice: Dictionary = Activity.choose_frame(groups, "adult_a", "carry_wood", "E", 0.0, 0.0)
	_check(choice["sprite_id"] == "own", "real same-body north wood precedes walking fallback")
	_check(not Activity.show_activity_icon(choice, 2.0))
	groups.erase("adult_a/carry_wood/N")
	choice = Activity.choose_frame(groups, "adult_a", "carry_wood", "E", 0.0, 0.0)
	_check(choice["sprite_id"] == "walk")
	_check(Activity.show_activity_icon(choice, 2.0))
	_check(not Activity.show_activity_icon(choice, 1.0))
	choice = Activity.choose_frame(groups, "adult_a", "gather_wood", "N", 0.0, 0.0)
	_check(choice["sprite_id"] == "crouch", "wood never uses bush or basket")
	groups["adult_a/crouch/E"] = groups["adult_a/crouch/S"]
	groups.erase("adult_a/crouch/S")
	_check(Activity.choose_frame(groups, "adult_a", "gather_wood", "N", 0.0, 0.0)["sprite_id"] == "crouch", "any crouch view precedes idle")
	groups.erase("adult_a/crouch/E")
	choice = Activity.choose_frame(groups, "adult_a", "gather_wood", "E", 0.0, 0.0)
	_check(choice["sprite_id"] == "idle" and Activity.show_activity_icon(choice, 2.0))

func _test_copied_carry() -> void:
	for body: String in ["adult_a", "adult_b", "child"]:
		for copies: Array in [[2], [2, 3]]:
			var entries: Dictionary = {}
			for index: int in 4:
				entries[str(index)] = {"body": body, "activity": "carry", "view": "S", "frame": index, "qa_status": "derived" if index in copies else "accepted", "source": "derived:same_cycle_phase_copy" if index in copies else "generated:reviewed"}
			var groups: Dictionary = Activity.build_frame_groups(entries)
			for step: int in 8:
				var choice: Dictionary = Activity.choose_frame(groups, body, "carry", "S", float(step) / 8.0, 0.0)
				_check(choice["frame"] == step % 4, "same phase copies keep four-slot carry playback and loop")
				_check(not Activity.show_activity_icon(choice, 4.0), "real copy never uses floating icon")
	var unreviewed: Dictionary = {"body": "adult_a", "activity": "carry", "qa_status": "derived", "source": "procedural:other"}
	_check(not Activity._real_carry(unreviewed, "adult_a"), "unreviewed derivation is not real carry")
