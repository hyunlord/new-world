# Real visitor with synthetic pressure/draft presentation stress fixtures. No simulation commands.
# Bridge methods: set_sim_speed, advance_ticks, load_world, save_world, get_neighbours,
#   get_time_snapshot, get_command_log, get_agent_snapshot.
extends SceneTree
var failures: int = 0
func _init() -> void:
	call_deferred("_run")
func check(label: String, ok: bool) -> void:
	print("STORY_LAYOUT %s %s" % ["PASS" if ok else "FAIL", label])
	if not ok:
		failures += 1
func frames() -> void:
	for _i: int in 12:
		await process_frame
func controls(node: Node) -> Array[Control]:
	var result: Array[Control] = []
	for child: Node in node.get_children():
		if child is Control and child.is_visible_in_tree() and (child is Button or child is Label):
			result.append(child)
		result.append_array(controls(child))
	return result
func reveal(item: Control) -> void:
	var ancestor: Node = item.get_parent()
	while ancestor != null:
		if ancestor is ScrollContainer:
			ancestor.ensure_control_visible(item)
			await process_frame
			await process_frame
		ancestor = ancestor.get_parent()
func hit(button: Button, tag: String) -> void:
	await reveal(button)
	var event: InputEventMouseMotion = InputEventMouseMotion.new()
	event.position = button.get_global_rect().get_center()
	Input.parse_input_event(event)
	await process_frame
	await process_frame
	check(tag + " mouse hits " + button.name, root.gui_get_hovered_control() == button)
func click(button: Button, tag: String) -> void:
	await hit(button, tag)
	for pressed: bool in [true, false]:
		var event: InputEventMouseButton = InputEventMouseButton.new()
		event.position = button.get_global_rect().get_center()
		event.button_index = MOUSE_BUTTON_LEFT
		event.pressed = pressed
		Input.parse_input_event(event)
		await process_frame
	await frames()
func capture(tag: String) -> void:
	await RenderingServer.frame_post_draw
	check(tag + " capture", root.get_texture().get_image().save_png(OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR").path_join(tag + ".png")) == OK)
func audit(story: Node, obstacles: Array[Control], factor: float, tag: String) -> void:
	var column: Control = story.get_node("Column")
	check(tag + " requested story text scale", column.scale.is_equal_approx(Vector2.ONE * factor))
	for item: Control in controls(column):
		await reveal(item)
		check(tag + " onscreen " + item.name, root.get_visible_rect().encloses(item.get_global_rect()))
		for obstacle: Control in obstacles:
			if obstacle.is_visible_in_tree():
				check(tag + " clear " + item.name + "/" + obstacle.name, not item.get_global_rect().intersects(obstacle.get_global_rect()))
		if item is Button and not item.disabled:
			await hit(item, tag)
	for obstacle: Control in obstacles:
		if not obstacle.is_visible_in_tree():
			continue
		for item: Control in controls(obstacle):
			if item is Button and not item.disabled:
				var ancestor: Node = item.get_parent()
				while ancestor != null and ancestor != obstacle:
					if ancestor is ScrollContainer:
						ancestor.ensure_control_visible(item)
						await process_frame
						await process_frame
					ancestor = ancestor.get_parent()
				for story_item: Control in controls(column):
					check(tag + " button clear of story " + item.name, not item.get_global_rect().intersects(story_item.get_global_rect().intersection(column.get_global_rect())))
				# Hover only: trade, migration and undraft confirmation must never be submitted.
				await hit(item, tag)
func _run() -> void:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	main.get_node("Camera2D").call("set_paused", true)
	await frames()
	var hud: Node = main.get_node("UI/MuxHud")
	var cards: Control = hud.get_node("CardPanel")
	var story: Node = main.get_node("UI/StoryBanner")
	var store: RefCounted = hud.get("store")
	# Separate routes can be launched with M6B_STORY_VISITOR=wave or caravan and isolated settings.
	var kind: String = OS.get_environment("M6B_STORY_VISITOR")
	if kind.is_empty():
		kind = "caravan"
	var fixture: String = OS.get_environment("M6B_STORY_SAVE")
	if not fixture.is_empty():
		check("load live visitor fixture", world.call("load_world", fixture))
	var visitor_id: int = -1
	for _interval: int in 720:
		visitor_id = cards.call("_visitor_id", world.call("get_neighbours"), kind)
		if visitor_id >= 0:
			break
		world.call("advance_ticks", 60)
		await process_frame
	check("real waiting " + kind, visitor_id >= 0)
	if visitor_id < 0:
		quit(1)
		return
	await frames()
	print("STORY_FIXTURE_SAVE ", world.call("save_world", OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR").path_join("visitor.save")))
	story.set_process(false)
	var tick: Dictionary = world.call("get_time_snapshot")
	var log: Dictionary = world.call("get_command_log", 0)
	var people: Dictionary = world.call("get_agent_snapshot")
	var ids: PackedInt64Array = people.get("ids", PackedInt64Array())
	check("living draft display subject", not ids.is_empty())
	if ids.is_empty():
		quit(1)
		return
	var panel: Control = main.get_node("UI/TradePanel" if kind == "caravan" else "UI/MigrantPanel")
	panel.call("close")
	await frames()
	var stress: bool = OS.get_environment("M6B_STORY_STRESS") == "1"
	for language: String in ["ko", "en"]:
		root.get_node("Locale").call("set_language", language)
		story.call("_refresh_visitors", language)
		if stress:
			# Existing product builders display stress fixtures; no native pressure or draft state is changed.
			story.call("_fill_banners", {"on": true, "kinds": PackedInt64Array([0, 1, 3]), "days_of": PackedInt64Array([2, 3, 2]), "firewood": 12, "firewood_day_need": 18, "sick": 1})
			story.call("_fill_drafted", {"ids": PackedInt64Array([ids[0]]), "hps": PackedInt32Array([92]), "weapons": PackedInt64Array([-1])})
		for factor: float in [1.0, 1.25]:
			store.call("set_value", "ui_scale", factor)
			var active: Dictionary = {}
			for definition: Dictionary in cards.get("_card_defs"):
				if definition["id"] in ["trader", "need_food", "tools_low"]:
					active[definition["id"]] = {"def": definition, "trigger_data": {"caravan_id": visitor_id}, "answered": false}
			cards.set("_active", active)
			cards.call("_rebuild_ui")
			await frames()
			hud.set("_poll_left", 1000.0)
			if stress:
				story.call("_fill_banners", {"on": false})
				story.call("_fill_drafted", {"ids": PackedInt64Array()})
				await frames()
				story.call("_fill_banners", {"on": true, "kinds": PackedInt64Array([0, 1, 3]), "days_of": PackedInt64Array([2, 3, 2]), "firewood": 12, "firewood_day_need": 18, "sick": 1})
				story.call("_fill_drafted", {"ids": PackedInt64Array([ids[0]]), "hps": PackedInt32Array([92]), "weapons": PackedInt64Array([-1])})
				await frames()
			var tag: String = "%s-%s-%.2f" % [kind, language, factor]
			check(tag + " pressure and draft retained", not stress or (story.call("banner_texts").size() == 3 and story.call("drafted_texts").size() == 1))
			await audit(story, [cards, hud.get_node("Alerts"), main.get_node("UI/TaskPlaque").get("_panel"), hud.get_node("MapDock"), hud.get_node("LearningHelper")], factor, tag + "-ambient")
			check(tag + " positive card height", cards.size.y > 0)
			check(tag + " alerts clear dock", not (hud.get_node("Alerts") as Control).get_global_rect().intersects((hud.get_node("MapDock") as Control).get_global_rect()))
			if stress:
				var scroll: ScrollContainer = story.get_node("Column")
				scroll.scroll_vertical = 0
				await frames()
				await capture(tag + "-story-top")
				var camera: Camera2D = main.get_node("Camera2D")
				var zoom: Vector2 = camera.zoom
				for pressed: bool in [true, false]:
					var wheel: InputEventMouseButton = InputEventMouseButton.new()
					wheel.button_index = MOUSE_BUTTON_WHEEL_DOWN
					wheel.pressed = pressed
					wheel.position = scroll.get_global_rect().get_center()
					Input.parse_input_event(wheel)
				await frames()
				check(tag + " story wheel reaches overflow", scroll.scroll_vertical > 0)
				check(tag + " story wheel does not zoom", camera.zoom == zoom)
			await capture(tag + "-ambient")
			var button: Button = story.get_node("Column/Content/Visitors/" + (("Caravan_%d" if kind == "caravan" else "Wave_%d") % visitor_id))
			await click(button, tag)
			check(tag + " visitor click opens matching panel", panel.is_visible_in_tree() and int(panel.get("_caravan" if kind == "caravan" else "_wave")) == visitor_id)
			check(tag + " foreground requested text scale", panel.scale.is_equal_approx(Vector2.ONE * factor))
			await audit(story, [panel], factor, tag + "-foreground")
			await capture(tag + "-foreground")
			panel.call("close")
			await frames()
			check(tag + " cards restored unchanged", cards.is_visible_in_tree() and cards.get("_active") == active)
			check(tag + " ticks unchanged", world.call("get_time_snapshot") == tick)
			check(tag + " commands unchanged", world.call("get_command_log", 0) == log)
	quit(1 if failures else 0)
