# Rendered card layout regression; no card actions or simulation commands during assertions.
extends SceneTree

var failures: int = 0

func _init() -> void:
	call_deferred("_run")

func check(label: String, ok: bool) -> void:
	print("CARD_LAYOUT %s %s" % ["PASS" if ok else "FAIL", label])
	if not ok:
		failures += 1

func frames(count: int = 8) -> void:
	for _i: int in count:
		await process_frame

func buttons(node: Node) -> Array[Button]:
	var found: Array[Button] = []
	if node is Button:
		found.append(node)
	for child: Node in node.get_children():
		found.append_array(buttons(child))
	return found

func _run() -> void:
	var directory: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	main.get_node("Camera2D").call("set_paused", true)
	world.call("advance_ticks", 1)
	await frames()
	var tick: Dictionary = world.call("get_time_snapshot")
	var log: Dictionary = world.call("get_command_log", 0)
	var hud: Control = main.get_node("UI/MuxHud")
	var cards: Control = hud.get_node("CardPanel")
	var alerts: Control = hud.get_node("Alerts")
	var store: RefCounted = hud.get("store")
	for language: String in ["ko", "en"]:
		root.get_node("Locale").call("set_language", language)
		for factor: float in [1.0, 1.25]:
			store.call("set_value", "ui_scale", factor)
			await frames()
			# Rebuild after the ordinary layout; no half-second poll may rescue a stale placement.
			hud.set("_poll_left", 1000.0)
			var active: Dictionary = {}
			for definition: Dictionary in cards.get("_card_defs"):
				if str(definition["id"]) in ["need_food", "knowledge_risk", "tools_low"]:
					active[definition["id"]] = {"def": definition, "trigger_data": {"holder": "Long founder name / 긴 창립자 이름", "knowledge": "Door construction", "count": 7}, "answered": false}
			for count: int in [1, 2]:
				var partial: Dictionary = {}
				for key: String in active.keys().slice(0, count):
					partial[key] = active[key]
				cards.set("_active", partial)
				cards.call("_rebuild_ui")
				await frames()
				check("%s %.2f %d cards within viewport" % [language, factor, count], root.get_visible_rect().encloses(cards.get_global_rect()))
				check("%s %.2f %d cards above alerts" % [language, factor, count], alerts.get_global_rect().position.y >= cards.get_global_rect().end.y + 7)
			cards.set("_active", active)
			cards.call("_rebuild_ui")
			await frames()
			var tag: String = "%s-%.2f" % [language, factor]
			var card_rect: Rect2 = cards.get_global_rect()
			var alert_rect: Rect2 = alerts.get_global_rect()
			print("CARD_LAYOUT_RECTS " + tag + " " + str(card_rect) + " alerts=" + str(alert_rect))
			check(tag + " cards on screen", root.get_visible_rect().encloses(card_rect))
			check(tag + " requested card text scale preserved", cards.scale.is_equal_approx(Vector2.ONE * factor))
			check(tag + " alerts below cards", alert_rect.position.y >= card_rect.end.y + 7)
			check(tag + " alerts on screen", root.get_visible_rect().encloses(alert_rect))
			check(tag + " alerts clear order bar", alert_rect.end.y + 7 <= (main.get_node("UI/OrderBar/Bar") as Control).get_global_rect().position.y)
			check(tag + " plaque remains visible and clear", main.get_node("UI/TaskPlaque").is_visible_in_tree() and not card_rect.intersects((main.get_node("UI/TaskPlaque").get("_panel") as Control).get_global_rect()))
			var plaque_rect: Rect2 = (main.get_node("UI/TaskPlaque").get("_panel") as Control).get_global_rect()
			var actions_rect: Rect2 = (hud.get_node("Actions") as Control).get_global_rect()
			check(tag + " rendered plaque clear of action row", not plaque_rect.intersects(actions_rect))
			check(tag + " helper clear of cards", not card_rect.intersects((hud.get_node("LearningHelper") as Control).get_global_rect()))
			check(tag + " map dock separate", not card_rect.intersects((hud.get_node("MapDock") as Control).get_global_rect()) and not alert_rect.intersects((hud.get_node("MapDock") as Control).get_global_rect()))
			var scroll: ScrollContainer = cards.get_node("CardScroll")
			scroll.scroll_vertical = 0
			await frames(2)
			await RenderingServer.frame_post_draw
			check(tag + " top capture", root.get_texture().get_image().save_png(directory.path_join(tag + "-top.png")) == OK)
			for button: Button in buttons(cards):
				if not button.is_visible_in_tree():
					continue
				scroll.ensure_control_visible(button)
				await frames(2)
				check(tag + " button fully inside card", card_rect.encloses(button.get_global_rect()))
				var event: InputEventMouseMotion = InputEventMouseMotion.new()
				event.position = button.get_global_rect().get_center()
				Input.parse_input_event(event)
				await frames(2)
				check(tag + " button hit " + button.text, root.gui_get_hovered_control() == button)
			if float(cards.call("content_height")) > cards.size.y + 1:
				scroll.scroll_vertical = 0
				await frames(2)
				var camera: Camera2D = main.get_node("Camera2D")
				var zoom: Vector2 = camera.zoom
				var point: Vector2 = scroll.get_global_rect().get_center()
				for pressed: bool in [true, false]:
					var wheel: InputEventMouseButton = InputEventMouseButton.new()
					wheel.button_index = MOUSE_BUTTON_WHEEL_DOWN
					wheel.pressed = pressed
					wheel.position = point
					Input.parse_input_event(wheel)
				await frames(2)
				check(tag + " card wheel scrolls overflow", scroll.scroll_vertical > 0)
				check(tag + " card wheel does not zoom world", camera.zoom.is_equal_approx(zoom))
			await RenderingServer.frame_post_draw
			check(tag + " capture", root.get_texture().get_image().save_png(directory.path_join(tag + ".png")) == OK)
			check(tag + " card state unchanged", cards.get("_active") == active)
	check("tick unchanged", world.call("get_time_snapshot") == tick)
	check("command log unchanged", world.call("get_command_log", 0) == log)
	quit(1 if failures else 0)
