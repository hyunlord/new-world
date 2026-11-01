# Dynamic wrapped alert/fold heights at the two-card above-dock threshold.
# Bridge methods: set_sim_speed, advance_ticks, get_time_snapshot, get_command_log.
extends SceneTree
var failures: int = 0
func _init() -> void:
	call_deferred("_run")
func check(label: String, ok: bool) -> void:
	print("ALERT_LAYOUT %s %s" % ["PASS" if ok else "FAIL", label])
	if not ok:
		failures += 1
func frames() -> void:
	for _i: int in 12:
		await process_frame
func _run() -> void:
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
	var hud: Node = main.get_node("UI/MuxHud")
	var cards: Control = hud.get_node("CardPanel")
	var alerts: Control = hud.get_node("Alerts")
	alerts.set_process(false)
	var store: RefCounted = hud.get("store")
	var definitions: Array = JSON.parse_string(FileAccess.get_file_as_string("res://assets/ui/alerts.json"))["alerts"]
	for language: String in ["ko", "en"]:
		root.get_node("Locale").call("set_language", language)
		for factor: float in [1.0, 1.25]:
			store.call("set_value", "ui_scale", factor)
			await frames()
			hud.set("_poll_left", 1000.0)
			for count: int in [1, 2, 3, 2]:
				var active: Dictionary = {}
				for definition: Dictionary in cards.get("_card_defs"):
					if definition["id"] in ["trader", "tools_low", "need_food"] and active.size() < count:
						active[definition["id"]] = {"def": definition, "trigger_data": {}, "answered": false}
				cards.set("_active", active)
				cards.call("_rebuild_ui")
				for wrapped: bool in [false, true]:
					var list: VBoxContainer = alerts.get("_list")
					for child: Node in list.get_children():
						list.remove_child(child)
						child.queue_free()
					(alerts.get("_status") as Control).hide()
					(alerts.get("_scroll") as Control).show()
					alerts.set("_expanded", false)
					for definition: Dictionary in definitions:
						if definition["id"] in ["food_spoil", "food_winter", "trader_arrived"]:
							var row: Dictionary = definition.duplicate(true)
							row.merge({"icon": "!", "count": 1, "color": "d8a050", "facts": {"winter_in": 8, "stored_days": "3.0", "stored_units": 71, "winter_need": 80, "expiring_units": 45}}, true)
							alerts.call("_add_alert", row)
					# Third row guarantees the actual fold control exists, regardless of registry visitor ID.
					if list.get_child_count() == 2:
						alerts.call("_add_alert", {"id": "visitor_fixture", "text_key": "LOOP_CARD_TRADER", "explanation_key": "LOOP_CARD_TRADER", "fix_key": "LOOP_CARD_TRADER", "icon": "i", "color": "d8a050", "count": 1})
					if not wrapped:
						for row: Node in list.get_children():
							(row.get_child(0) as Button).text = "! 1"
					alerts.call("_fit_rows")
					await frames()
					var tag: String = "%s-%.2f-%d-%s" % [language, factor, count, "wrapped" if wrapped else "short"]
					for expanded: bool in [false, true]:
						if expanded:
							var fold: Button = alerts.get("_fold_button")
							if not fold.is_visible_in_tree():
								check(tag + " no fold needed when all rows visible", list.get_children().all(func(row: Node) -> bool: return row.visible))
								continue
							for pressed: bool in [true, false]:
								var event: InputEventMouseButton = InputEventMouseButton.new()
								event.position = fold.get_global_rect().get_center()
								event.button_index = MOUSE_BUTTON_LEFT
								event.pressed = pressed
								Input.parse_input_event(event)
								await process_frame
							await frames()
							check(tag + " fold input expands rows", bool(alerts.get("_expanded")))
							var scroll: ScrollContainer = alerts.get("_scroll")
							for row: Node in list.get_children():
								var button: Button = row.get_child(0)
								scroll.ensure_control_visible(button)
								await process_frame
								await process_frame
								var motion: InputEventMouseMotion = InputEventMouseMotion.new()
								motion.position = button.get_global_rect().get_center()
								Input.parse_input_event(motion)
								await process_frame
								check(tag + " expanded row hit " + button.name, root.gui_get_hovered_control() == button)
						check(tag + " alerts clear dock", not alerts.get_global_rect().intersects((hud.get_node("MapDock") as Control).get_global_rect()))
						check(tag + " alerts clear cards", not alerts.get_global_rect().intersects(cards.get_global_rect()))
						check(tag + " alerts on screen", root.get_visible_rect().encloses(alerts.get_global_rect()))
						check(tag + " requested scale", alerts.scale.is_equal_approx(Vector2.ONE * factor) and cards.scale.is_equal_approx(Vector2.ONE * factor))
						check(tag + " all alert rows retained", list.get_child_count() == 3)
						check(tag + " lifecycle unchanged", cards.get("_active") == active)
					if count == 2 and wrapped:
						await RenderingServer.frame_post_draw
						root.get_texture().get_image().save_png(OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR").path_join(tag + ".png"))
	check("ticks unchanged", world.call("get_time_snapshot") == tick)
	check("commands unchanged", world.call("get_command_log", 0) == log)
	quit(1 if failures else 0)
