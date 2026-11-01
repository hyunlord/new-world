# Toast geometry preserves foreground controls, feedback lifetime, and paused native state.
# Bridge methods: set_sim_speed, advance_ticks, get_time_snapshot, get_command_log, save_world, get_building_snapshot.
# Geometry cases pause OrderBar processing; the dedicated lifetime case calls its real _process.
extends SceneTree
var failures: int = 0
func _init() -> void:
	call_deferred("_run")
func check(tag: String, ok: bool) -> void:
	print("TOAST_LAYOUT %s %s" % ["PASS" if ok else "FAIL", tag])
	if not ok:
		failures += 1
func frames() -> void:
	for _i: int in 12:
		await process_frame
func click(control: Control) -> void:
	await click_at(control.get_global_rect().get_center())
func click_at(position: Vector2) -> void:
	Input.warp_mouse(position)
	var motion: InputEventMouseMotion = InputEventMouseMotion.new()
	motion.position = position
	Input.parse_input_event(motion)
	await frames()
	for pressed: bool in [true, false]:
		var event: InputEventMouseButton = InputEventMouseButton.new()
		event.position = position
		event.button_index = MOUSE_BUTTON_LEFT
		event.pressed = pressed
		Input.parse_input_event(event)
		await process_frame
	await frames()
func _run() -> void:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	main.get_node("Camera2D").call("set_paused", true)
	world.call("advance_ticks", 1)
	await frames()
	var directory: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	var before: String = directory.path_join("before.save")
	check("initial save", bool(world.call("save_world", before)))
	var tick: Dictionary = world.call("get_time_snapshot")
	var log: Dictionary = world.call("get_command_log", 0)
	var ui: Node = main.get_node("UI")
	var hud: Node = ui.get_node("MuxHud")
	var legacy: Node = hud.get("_legacy")
	var bar: Node = ui.get_node("OrderBar")
	bar.set_process(false)
	bar.call("clear_background_toasts")
	var toast: Control = bar.get("_toast_panel")
	var label: Label = bar.get("_toast")
	var store: RefCounted = hud.get("store")
	var buildings: Dictionary = world.call("get_building_snapshot")
	var hearth: Vector2i
	for i: int in buildings["kinds"].size():
		if buildings["kinds"][i] == 0:
			hearth = Vector2i(buildings["xs"][i], buildings["ys"][i])
			break
	var panels: Array = []
	for candidate: String in legacy.get_script().get("PANELS"):
		if ui.get_node_or_null(candidate) != null:
			panels.append(candidate)
	panels.append("CausalPanel")
	for language: String in ["ko", "en"]:
		root.get_node("Locale").call("set_language", language)
		for factor: float in [1.0, 1.25]:
			store.call("set_value", "ui_scale", factor)
			for name: String in panels:
				toast.hide()
				for other: String in panels:
					(ui.get_node(other) as Control).hide()
				var panel: Control = ui.get_node(name)
				if name != "CausalPanel":
					panel.set_process(false)
				if name == "PriorityPanel":
					panel.call("open_roles")
				elif name == "CausalPanel":
					var camera: Camera2D = main.get_node("Camera2D")
					var point: Vector2 = root.get_canvas_transform() * (Vector2(hearth) * 64.0 + Vector2(32, 32))
					camera.position += root.get_canvas_transform().affine_inverse().basis_xform(point - Vector2(800, 850))
					camera.force_update_scroll()
					await frames()
					await click(hud.get("_why_button") as Control)
					point = root.get_canvas_transform() * (Vector2(hearth) * 64.0 + Vector2(32, 32))
					check("real hearth onscreen", root.get_visible_rect().has_point(point))
					await click_at(point)
					check("actual Why hearth click opens diagnosis", panel.visible and panel.get("_anchor_tile") == hearth)
					for down: bool in [true, false]:
						var key: InputEventKey = InputEventKey.new()
						key.keycode = KEY_T
						key.pressed = down
						Input.parse_input_event(key)
						await process_frame
					await frames()
					check("actual T opens Tech alongside diagnosis", panel.visible and (ui.get_node("TechPanel") as Control).visible)
				elif name == "TechPanel":
					panel.call("open")
				else:
					panel.show()
				await frames()
				if name == "WhyPanel":
					(ui.get_node("AgentInspectorPanel") as Control).show()
					await frames()
				var original_height: float = panel.size.y
				for repetitions: int in [1, 4, 30]:
					var text: String = str(root.get_node("Locale").call("ltr", "MUX_WHY_EMPTY"))
					text = text.repeat(repetitions)
					bar.call("show_toast", text)
					var lifetime: float = float(bar.get("_toast_left"))
					await frames()
					var tag: String = "%s-%.2f-%s-%s" % [language, factor, name, str(repetitions)]
					var report: Dictionary = legacy.call("layout_report")
					check(tag + " foreground visible", panel.is_visible_in_tree())
					check(tag + " visible text", toast.is_visible_in_tree() and label.text == text)
					check(tag + " lifetime unchanged by layout", float(bar.get("_toast_left")) == lifetime)
					check(tag + " toast onscreen", root.get_visible_rect().encloses(toast.get_global_rect()))
					for other: String in panels:
						var item: Dictionary = report.get(other, {})
						if bool(item.get("visible", false)):
							check(tag + " clear " + other, not toast.get_global_rect().intersects(item["rect"]))
					var toast_report: Dictionary = report.get("OrderBar/Toast", {})
					check(tag + " requested toast scale", is_equal_approx(toast.scale.x, factor))
					if name == "PriorityPanel":
						if repetitions in [1, 30]:
							check(tag + " gutter or height fallback", bool(toast_report.get("reserved_row", false)) if repetitions == 30 else not bool(toast_report.get("reserved_row", true)))
						var choices: Array[Node] = panel.find_children("*", "OptionButton", true, false)
						check(tag + " actual role controls", not choices.is_empty())
						if not choices.is_empty():
							var choice: OptionButton = choices.back() as OptionButton
							var ancestor: Node = choice.get_parent()
							while ancestor != null and not ancestor is ScrollContainer:
								ancestor = ancestor.get_parent()
							if ancestor is ScrollContainer:
								(ancestor as ScrollContainer).ensure_control_visible(choice)
							await frames()
							await click(choice)
							check(tag + " actual role click opens menu", choice.get_popup().visible)
							choice.get_popup().hide()
					if name in ["TechPanel", "CausalPanel"]:
						check(tag + " dedicated row", bool(toast_report.get("reserved_row", false)))
						print("TOAST_FIXED_LIMIT ", tag, " ", report.get(name, {}))
						if name == "CausalPanel":
							print("TOAST_CAUSAL_SATURATION ", tag, " placement_exhausted=", panel.get("_placement_exhausted"))
							check(tag + " actual causal slot available", not bool(panel.get("_placement_exhausted")))
							var causal_rect: Rect2 = panel.get_global_rect()
							for obstacle: String in ["TechPanel", "HudTopbar", "OrderBar", "OrderBar/Toast"]:
								check(tag + " causal clear " + obstacle, not causal_rect.intersects(report[obstacle]["rect"]))
					var rect: Rect2 = toast.get_global_rect()
					for frame: int in 12:
						await process_frame
						check(tag + " consecutive frame %d" % frame, toast.get_global_rect().is_equal_approx(rect))
					if name in ["PriorityPanel", "TechPanel", "CausalPanel"]:
						await RenderingServer.frame_post_draw
						check(tag + " screenshot", root.get_texture().get_image().save_png(directory.path_join(tag + ".png")) == OK)
				toast.hide()
				await frames()
				check(name + " hidden toast restores height", is_equal_approx(panel.size.y, original_height))
				for other: String in panels:
					(ui.get_node(other) as Control).hide()
				bar.call("show_toast", "OK")
				await frames()
				check(name + " close resets wrap", label.autowrap_mode == TextServer.AUTOWRAP_OFF and toast.custom_minimum_size.x == 0.0)
				check(name + " close resets width", toast.size.x < 160.0)
	bar.call("clear_background_toasts")
	bar.call("show_toast", "foreground feedback")
	bar.call("queue_background_toast", "queued feedback", 123)
	var lifetime: float = float(bar.get("_toast_left"))
	bar.call("_process", 0.1)
	check("timer decreases normally", is_equal_approx(float(bar.get("_toast_left")), lifetime - 0.1))
	check("queue retained", (bar.get("_background_toasts") as Array).size() == 1)
	bar.call("_process", lifetime)
	check("queued notification delivered", label.text == "queued feedback" and toast.visible)
	check("queue consumed normally", (bar.get("_background_toasts") as Array).is_empty())
	bar.call("_process", float(bar.get("_toast_left")) + 0.1)
	check("natural expiry", not toast.visible)
	check("native tick unchanged", world.call("get_time_snapshot") == tick)
	check("native log unchanged", world.call("get_command_log", 0) == log)
	var after: String = directory.path_join("after.save")
	check("final save", bool(world.call("save_world", after)))
	check("native save bytes unchanged", FileAccess.get_file_as_bytes(before) == FileAccess.get_file_as_bytes(after))
	print("TOAST_LAYOUT failures=", failures)
	quit(0 if failures == 0 else 1)
