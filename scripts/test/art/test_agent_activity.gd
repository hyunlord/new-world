extends SceneTree
const Activity = preload("res://scripts/ui/agent_activity.gd")
const Library = preload("res://scripts/ui/art_sprite_library.gd")
func _initialize() -> void:
	call_deferred("_run")

func _run() -> void:
	_test_frame_lookup()
	var shelter: Dictionary = {"shelter": true}
	assert(Activity.resolve(3, 3, true, false, 0.0, shelter, 23.0) == "asleep")
	assert(Activity.hidden("asleep", true, shelter))
	assert(not Activity.hidden("asleep", false, shelter))
	assert(not Activity.hidden("asleep", true, {}))
	assert(Activity.resolve(3, 0, false, false, 0.0, shelter, 12.0) == "asleep")
	assert(Activity.resolve(3, 0, false, false, 0.0, {"construction": true, "food": true}, 12.0) == "build")
	assert(Activity.resolve(3, 0, false, false, 0.0, {"food": true, "water": true}, 12.0) == "gather")
	assert(Activity.resolve(3, 0, false, false, 0.0, {"water": true}, 12.0) == "drink")
	assert(Activity.resolve(3, 1, true, false, 0.0, shelter, 12.0) == "eat")
	assert(Activity.resolve(3, 6, true, false, 0.0, {}, 12.0) == "gather")
	assert(Activity.resolve(2, 0, false, true, 1.0, {}, 12.0) == "talk")
	assert(Activity.resolve(1, 0, false, false, 1.0, {}, 12.0) == "carry")
	assert(Activity.resolve(0, 0, false, true, 0.0, {}, 12.0) == "walk")
	assert(Activity.resolve(0, 0, false, false, 0.0, {"near_hearth": true}, 18.0) == "sit")
	assert(Activity.resolve(0, 0, false, false, 0.0, {"near_hearth": true}, 23.0) == "idle")
	assert(Activity.facing(Vector2.ZERO, "N") == "N")
	assert(Activity.facing(Vector2(-2, 1), "S") == "W")
	assert(Activity.facing(Vector2(0, -2), "S") == "N")
	assert(Activity.frame_index("walk", 0.125, 0.0, 4) == 1)
	assert(Activity.frame_index("idle", 0.7, 0.0, 2) == 1)
	assert(Activity.frame_index("build", 0.25, 0.0, 2) == 1)
	var sprite: Sprite2D = Sprite2D.new()
	assert(Library.apply(sprite, "direction_09_villager_gatherer"))
	assert(Library.apply_variant(sprite, {"skin": 3, "hair": 4, "clothing": 5}))
	assert(roundi(sprite.self_modulate.r * 15.0) == 3)
	assert(roundi(sprite.self_modulate.g * 15.0) == 4)
	assert(roundi(sprite.self_modulate.b * 15.0) == 5)
	assert(sprite.material is ShaderMaterial)
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
	assert(context.has("food") and context.has("hearths") and context.has("terrain"))
	assert((context["terrain"] as Dictionary).has("terrain"))
	var snapshot: Dictionary = world.call("get_agent_snapshot")
	var ids: PackedInt64Array = snapshot["ids"]
	var inspector: Node = main.get_node("UI/AgentInspectorPanel")
	inspector.call("display_agent", world.call("get_agent_detail", ids[0]))
	renderer.call("_update_selection_ring")
	var ring: Line2D = main.get_node("Shadows/SelectionRing") as Line2D
	assert(ring.visible)
	assert(ring.width == 1.0)
	assert(ring.get_parent().name == "Shadows")
	var agent_ids: PackedInt64Array = snapshot["agent_ids"]
	var selected_sprite: Sprite2D = renderer.call("get_agent_sprite", int(agent_ids[0])) as Sprite2D
	assert(ring.position == selected_sprite.position)
	var body: String = str(selected_sprite.get_meta("body"))
	var icon_choice: Dictionary = renderer.call("frame_choice", body, "carry", "S", 0.0)
	assert(icon_choice["fallback_kind"] == "idle_icon")
	var icon: Sprite2D = Sprite2D.new()
	selected_sprite.add_child(icon)
	assert(not Activity.apply_activity_icon(selected_sprite, icon, icon_choice, 1.0))
	assert(not icon.visible)
	assert(Activity.apply_activity_icon(selected_sprite, icon, icon_choice, 2.0))
	assert(icon.visible and icon.texture != null)
	assert(icon.position == Vector2(0, selected_sprite.offset.y - 2.0))
	assert(str(icon.get_meta("sprite_id")) == "U1_05")
	assert(not Activity.apply_activity_icon(selected_sprite, icon, {}, 4.0))
	assert(not icon.visible)
	icon.queue_free()
	renderer.set("_animation_time", 0.0)
	renderer.call("_update_selection_ring")
	assert(int(ring.get_meta("frame_id")) == 0)
	renderer.set("_animation_time", 0.5)
	renderer.call("_update_selection_ring")
	assert(int(ring.get_meta("frame_id")) == 1)
	inspector.call("close")
	renderer.call("_update_selection_ring")
	assert(not ring.visible)
	main.free()
	print("ACTIVITY PASS: target compatibility, tile priorities, sleeping visibility, facing, rates, shader codes")
	quit(0)

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
	assert(walk["sprite_id"] == "a_idle")
	assert(walk["fallback_kind"] == "idle_icon")
	assert(not Activity.show_activity_icon(walk, 1.0))
	assert(not Activity.show_activity_icon(walk, 1.99))
	assert(Activity.show_activity_icon(walk, 2.0))
	var carry: Dictionary = Activity.choose_frame(groups, "adult_a", "carry", "S", 0.0, 0.0)
	assert(carry["sprite_id"] == "a_carry")
	assert(not Activity.show_activity_icon(carry, 4.0))
	var build: Dictionary = Activity.choose_frame(groups, "adult_a", "build", "S", 0.0, 0.0)
	assert(build["sprite_id"] == "a_idle")
	assert(Activity.choose_frame(groups, "child", "build", "S", 0.0, 0.0).is_empty())
	assert(Activity.choose_frame(groups, "adult_a", "walk", "N", 0.0, 0.0).is_empty())
	var actual: Dictionary = Activity.build_frame_groups(Library.entries())
	for body: String in ["adult_a", "adult_b", "child"]:
		for activity: String in ["idle", "walk", "carry", "build", "gather", "talk", "sit", "drink", "asleep"]:
			for view: String in ["S", "N", "E", "W"]:
				var choice: Dictionary = Activity.choose_frame(actual, body, activity, view, 1.375, 0.0)
				if choice.is_empty():
					continue
				assert(choice["body"] == body)
				assert(not bool(choice.get("contains_basket", false)) or activity == "carry")
				assert(not str(choice["sprite_id"]).begins_with("direction_"))
				assert(Activity.show_activity_icon(choice, 4.0) == (str(choice.get("fallback_kind", "")) == "idle_icon" and not str(choice.get("activity_icon", "")).is_empty()))
