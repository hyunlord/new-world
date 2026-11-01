# Group summaries preserve real localized event sentences; details retain every original Why link.
# Bridge methods: set_sim_speed, advance_ticks, get_world_info, get_time_snapshot, get_command_log, save_world.
# Synthetic presentation entries only; no native event/command injection. Compare native save bytes before/after.
extends SceneTree
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
var failures: int = 0
var hovered_meta: String = ""
func _init() -> void:
	call_deferred("_run")
func check(label: String, ok: bool) -> void:
	print("CHRONICLE_GROUPS %s %s" % ["PASS" if ok else "FAIL", label])
	if not ok:
		failures += 1
func frames() -> void:
	for _i: int in 5:
		await process_frame
func _meta_hover(meta: Variant) -> void:
	hovered_meta = str(meta)

func click_group(body: RichTextLabel, tag: String) -> void:
	# Find the rendered first header through Godot's real metadata hover hit test.
	var outside: InputEventMouseMotion = InputEventMouseMotion.new()
	outside.position = Vector2(1900, 1050)
	Input.warp_mouse(outside.position)
	Input.parse_input_event(outside)
	await frames()
	hovered_meta = ""
	var target: Vector2 = Vector2.ZERO
	for y: int in range(8, 56, 8):
		var motion: InputEventMouseMotion = InputEventMouseMotion.new()
		motion.position = body.get_global_transform_with_canvas() * Vector2(48, y)
		Input.warp_mouse(motion.position)
		Input.parse_input_event(motion)
		await frames()
		if hovered_meta.begins_with("group:"):
			target = motion.position
			break
	check(tag + " actual header hover", not target.is_zero_approx())
	if target.is_zero_approx():
		print("CHRONICLE_GROUPS_HIT ", body.get_global_rect(), " hovered=", root.gui_get_hovered_control(), " meta=", hovered_meta)
		return
	for pressed: bool in [true, false]:
		var event: InputEventMouseButton = InputEventMouseButton.new()
		event.position = target
		event.button_index = MOUSE_BUTTON_LEFT
		event.pressed = pressed
		Input.parse_input_event(event)
		await process_frame
	await frames()
func capture(tag: String) -> void:
	await RenderingServer.frame_post_draw
	check(tag + " capture", root.get_texture().get_image().save_png(OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR").path_join(tag + ".png")) == OK)
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
	check("initial native save", world.call("save_world", before))
	var tick: Dictionary = world.call("get_time_snapshot")
	var log: Dictionary = world.call("get_command_log", 0)
	var info: Dictionary = world.call("get_world_info")
	var panel: Control = main.get_node("UI/ChroniclePanel")
	panel.set_process(false)
	panel.set("_show_all", true)
	panel.set("_filter", 0)
	panel.set("_person_only", false)
	panel.show()
	var locale: Node = root.get_node("Locale")
	var body: RichTextLabel = panel.get("_body")
	for language: String in ["ko", "en"]:
		locale.call("set_language", language)
		# Dehydration must not be summarized as starvation; drought onset/end must not become cold injuries.
		for scenario: Vector2i in [Vector2i(0, 1), Vector2i(29, 1), Vector2i(30, 1), Vector2i(34, 0), Vector2i(34, 1), Vector2i(34, 2)]:
			panel.call("_reset")
			panel.set("_show_all", true)
			panel.show()
			var rows: Array[Dictionary] = []
			for i: int in 2:
				rows.append({"index": 100 + i, "kind": scenario.x, "cause": scenario.y, "tick": i + 1, "subject": i, "other": -1, "third": -1, "settlement": 0, "settlement2": -1, "value": 0, "link": 0, "link_entry": 0, "link_code": 0})
			panel.set("_entries", rows)
			panel.call("_refresh")
			await frames()
			var tag: String = "%s-kind%d-cause%d" % [language, scenario.x, scenario.y]
			var expected: String = Chronicle.line(rows[1], -1, locale, info, true, true)
			check(tag + " collapsed actual representative", body.text.contains(expected))
			var links: RegEx = RegEx.new()
			links.compile("\\[url=(group:[^\\]]+)\\]")
			var match_link: RegExMatch = links.search(body.text)
			check(tag + " details link exists", match_link != null)
			if match_link == null:
				continue
			body.meta_clicked.emit(match_link.get_string(1))
			await frames()
			for row: Dictionary in rows:
				check(tag + " expanded exact entry %d" % row["index"], body.text.contains(Chronicle.line(row, -1, locale, info, true, true)))
				check(tag + " expanded Why metadata %d" % row["index"], body.text.contains("[url=why:%d]" % row["index"]))
			check(tag + " subjects highlighted", (panel.get("_highlighted_subjects") as PackedInt64Array).size() == 2)
			body.meta_clicked.emit(match_link.get_string(1))
			await frames()
			check(tag + " collapse retains representative", body.text.contains(expected) and not body.text.contains("[url=why:100]"))
			check(tag + " collapse clears highlights", (panel.get("_highlighted_subjects") as PackedInt64Array).is_empty())
			body.meta_clicked.emit(match_link.get_string(1))
			var appended: Dictionary = rows[1].duplicate()
			appended["index"] = 102
			appended["subject"] = 2
			rows.append(appended)
			panel.set("_entries", rows)
			panel.call("_refresh")
			await frames()
			check(tag + " append refreshes highlights", (panel.get("_highlighted_subjects") as PackedInt64Array).size() == 3)
			panel.set("_entries", [] as Array[Dictionary])
			panel.call("_refresh")
			await frames()
			check(tag + " removed group clears highlights", (panel.get("_highlighted_subjects") as PackedInt64Array).is_empty())
	# Matching-surface check: actual first-header press/release, independently of direct metadata checks above.
	body.meta_hover_started.connect(_meta_hover)
	var store: RefCounted = main.get_node("UI/MuxHud").get("store")
	for language: String in ["ko", "en"]:
		locale.call("set_language", language)
		for factor: float in [1.0, 1.25]:
			store.call("set_value", "ui_scale", factor)
			panel.call("_reset")
			panel.set("_show_all", true)
			panel.show()
			var rows: Array[Dictionary] = []
			for i: int in 2:
				rows.append({"index": 100 + i, "kind": 34, "cause": 1, "tick": i + 1, "subject": i, "other": -1, "third": -1, "settlement": 0, "settlement2": -1, "value": 0, "link": 0, "link_entry": 0, "link_code": 0})
			panel.set("_entries", rows)
			panel.call("_refresh")
			await frames()
			var tag: String = "%s-%.2f" % [language, factor]
			check(tag + " requested text scale", (panel as Control).scale.is_equal_approx(Vector2.ONE * factor))
			await capture(tag + "-collapsed")
			await click_group(body, tag + " expand")
			check(tag + " actual click expanded both entries", body.text.contains("[url=why:100]") and body.text.contains("[url=why:101]"))
			await capture(tag + "-expanded")
			await click_group(body, tag + " collapse")
			check(tag + " actual click collapsed", not body.text.contains("[url=why:100]") and body.text.contains("[url=why:101]"))
	check("tick unchanged", world.call("get_time_snapshot") == tick)
	check("command log unchanged", world.call("get_command_log", 0) == log)
	var after: String = directory.path_join("after.save")
	check("final native save", world.call("save_world", after))
	check("native serialized state unchanged", FileAccess.get_file_as_bytes(before) == FileAccess.get_file_as_bytes(after))
	quit(1 if failures else 0)
