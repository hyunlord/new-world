# M-SPINE tech-map regression and real-window capture; motivated by the missing
# four-era, grey-content and prerequisite-chain UI coverage. No simulation writes.
# Bridge methods: get_tech/get_tech_node/get_time_snapshot; simulation runs at normal 1x speed.
# Run with external WORLDSIM_TEST_SETTINGS_DIR and -- --out /absolute/evidence/dir.
extends SceneTree

const Text = preload("res://scripts/ui/knowledge_text.gd")
var _failures: int = 0
var _out: String = ""
var _window_size: Vector2i = Vector2i.ZERO
var _main: Node
var _panel: Control
var _locale: Node
var _pacing: Dictionary = {}


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size() - 1:
		if args[i] == "--out":
			_out = args[i + 1]
		if args[i] == "--window-size":
			var dimensions: PackedStringArray = args[i + 1].split("x")
			if dimensions.size() == 2:
				_window_size = Vector2i(int(dimensions[0]), int(dimensions[1]))
	call_deferred("_run")


func _check(label: String, ok: bool) -> void:
	print("SPINE_TECH %s %s" % ["PASS" if ok else "FAIL", label])
	if not ok:
		_failures += 1


func _frames(count: int = 4) -> void:
	for _i: int in count:
		await process_frame


func _key(code: Key) -> void:
	for pressed: bool in [true, false]:
		var event: InputEventKey = InputEventKey.new()
		event.keycode = code
		event.physical_keycode = code
		event.pressed = pressed
		Input.parse_input_event(event)
		await process_frame


func _click(button: Button) -> void:
	var ancestor: Node = button.get_parent()
	while ancestor != null:
		if ancestor is ScrollContainer:
			ancestor.ensure_control_visible(button)
			await _frames(2)
		ancestor = ancestor.get_parent()
	await _frames()
	for pressed: bool in [true, false]:
		var event: InputEventMouseButton = InputEventMouseButton.new()
		event.button_index = MOUSE_BUTTON_LEFT
		event.position = root.get_final_transform() * button.get_global_rect().get_center()
		event.pressed = pressed
		Input.parse_input_event(event)
		await process_frame
	await _frames()


func _observe_frames(language: String, context: String) -> void:
	await process_frame
	var world: Node = _main.get_node("WorldSim")
	var start_tick: int = int(world.call("get_time_snapshot").get("tick", 0))
	var samples: Array[int] = []
	var previous: int = Time.get_ticks_usec()
	for _i: int in 120:
		await process_frame
		var now: int = Time.get_ticks_usec()
		samples.append(now - previous)
		previous = now
	var sorted: Array[int] = samples.duplicate()
	sorted.sort()
	var mean: float = 0.0
	for sample: int in samples:
		mean += float(sample) / float(samples.size())
	var fps: float = 1000000.0 / mean if mean > 0.0 else 0.0
	var end_tick: int = int(world.call("get_time_snapshot").get("tick", 0))
	var key: String = language + "_" + context
	_pacing[key] = {"display_server": DisplayServer.get_name(), "windowed": DisplayServer.get_name() != "headless",
		"context": context, "delta_us": samples, "mean_us": mean, "p95_us": sorted[113], "max_us": sorted[-1],
		"observed_fps": fps, "target_fps": 60, "target_met": fps >= 60.0, "start_tick": start_tick, "end_tick": end_tick}
	_check("simulation_advances_" + key, end_tick > start_tick)
	_check("60fps_" + key, fps >= 60.0)
	print("SPINE_TECH_PACING %s fps=%.3f mean_us=%.3f p95_us=%d max_us=%d ticks=%d..%d" % [key, fps, mean, sorted[113], sorted[-1], start_tick, end_tick])


func _capture(name: String) -> void:
	if _out.is_empty():
		return
	await RenderingServer.frame_post_draw
	var image: Image = root.get_texture().get_image()
	_check("capture_" + name, image != null and not image.is_empty())
	if image != null and not image.is_empty():
		_check("save_" + name, image.save_png(_out.path_join(name + ".png")) == OK)


func _run() -> void:
	_check("real_window_required", DisplayServer.get_name() != "headless")
	if DisplayServer.get_name() == "headless":
		_finish()
		return
	var settings: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	if not settings.is_absolute_path() or not DirAccess.dir_exists_absolute(settings):
		_check("external_settings_required", false)
		_finish()
		return
	if not _out.is_empty():
		if not _out.is_absolute_path() or _out.begins_with(ProjectSettings.globalize_path("res://")):
			_check("external_evidence_required", false)
			_out = ""
			_finish()
			return
		DirAccess.make_dir_recursive_absolute(_out)
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	await _frames(20)
	if _window_size.x > 0 and _window_size.y > 0:
		root.size = _window_size
		await _frames(4)
		_check("requested_window_size", DisplayServer.window_get_size() == _window_size)
	_check("window_width_capped", DisplayServer.window_get_size().x <= 1920)
	var world: Node = _main.get_node_or_null("WorldSim")
	_panel = _main.get_node_or_null("UI/TechPanel") as Control
	_locale = root.get_node_or_null("Locale")
	_check("scene", world != null and _panel != null and _locale != null)
	if world == null or _panel == null or _locale == null:
		_finish()
		return
	var camera: Node = _main.get_node("Camera2D")
	camera.call("set_paused", false)
	camera.call("set_speed", 1.0)
	var tech: Dictionary = world.call("get_tech")
	var eras: Array[Dictionary] = Text.catalogue(tech, "eras")
	var lanes: Array[Dictionary] = Text.catalogue(tech, "lanes")
	var rows: Array[Dictionary] = Text.nodes(tech)
	_check("four_eras", eras.size() == 4)
	_check("ten_lanes", lanes.size() == 10)
	_check("authored_nodes", rows.size() == 43)
	var inert: int = -1
	for row: Dictionary in rows:
		if not bool(row.get("connected", true)):
			inert = int(row["id"])
			break
	_check("inert_authored", inert >= 0)
	for language: String in ["ko", "en"]:
		_locale.call("set_language", language)
		_panel.call("select_node", -1)
		await _key(KEY_T)
		await _frames()
		_check("open_" + language, _panel.visible)
		var map: Control = _panel.find_child("Map", true, false) as Control
		var grid: GridContainer = map.get_child(0) as GridContainer
		_check("columns_" + language, grid.columns == 5)
		var cells: int = 0
		for lane: Dictionary in lanes:
			for era: Dictionary in eras:
				if grid.get_node_or_null("Cell_%d_%d" % [lane["order"], era["order"]]) != null:
					cells += 1
		_check("forty_cells_" + language, cells == 40)
		_check("gauges_" + language, _panel.find_children("EraGauge", "ProgressBar", true, false).size() == 4)
		var cards: int = 0
		for row: Dictionary in rows:
			if _panel.call("card", int(row["id"])) != null:
				cards += 1
		_check("every_card_" + language, cards == rows.size())
		_panel.call("reset_overview")
		await _frames()
		await _observe_frames(language, "tech_open")
		await _capture("tech_" + language)
		var button: Button = _panel.call("card", inert)
		_check("inert_hatch_" + language, button != null and bool(button.get_meta("inert", false)) and button.get_node_or_null("Hatch") != null)
		if button != null:
			await _click(button)
			var detail: Dictionary = _panel.call("detail")
			_check("inert_selected_" + language, int(_panel.call("selected_node")) == inert)
			_check("inert_debt_" + language, not str(detail.get("debt", "")).is_empty() and not bool(detail.get("connected", true)))
			var teach: Button = _panel.find_child("Teach", true, false) as Button
			_check("inert_not_teachable_" + language, teach != null and not teach.visible)
			var chain: Dictionary = _panel.get("_chain")
			_check("prerequisite_chain_" + language, chain.has(inert) and chain.size() >= 2)
			for prerequisite: int in detail.get("prereqs", PackedInt64Array()):
				_check("clicked_chain_contains_%d_" % prerequisite + language, chain.has(prerequisite))
			var headers: Control = _panel.find_child("EraHeaders", true, false) as Control
			_check("headers_pinned_" + language, headers != null and headers.get_global_rect().position.y > _panel.get_global_rect().position.y)
			var detail_box: Control = _panel.find_child("Detail", true, false) as Control
			var detail_fits: bool = true
			for line: Node in detail_box.get_children():
				if line is Label:
					detail_fits = detail_fits and line.get_global_rect().end.x <= detail_box.get_global_rect().end.x + 1.0
			_check("detail_labels_fit_" + language, detail_fits)
			await _capture("tech_" + language + "_inert")
		var jump: OptionButton = _panel.find_child("LaneJump", true, false) as OptionButton
		_check("ten_lane_jumps_" + language, jump != null and jump.item_count == 10)
		if jump != null:
			await _click(jump)
			for _step: int in 10:
				await _key(KEY_DOWN)
			await _key(KEY_ENTER)
			await _frames()
			_check("lane_jump_input_" + language, jump.selected == 9 and (_panel.get("_map_scroll") as ScrollContainer).scroll_vertical > 0)
		await _key(KEY_T)
		_check("close_" + language, not _panel.visible)
		await _observe_frames(language, "tech_closed")
	_finish()


func _finish() -> void:
	if _out.is_absolute_path() and not _out.begins_with(ProjectSettings.globalize_path("res://")):
		var receipt: FileAccess = FileAccess.open(_out.path_join("tech_result.json"), FileAccess.WRITE)
		if receipt != null:
			receipt.store_string(JSON.stringify({"failures": _failures, "pacing": _pacing,
				"window_size": [DisplayServer.window_get_size().x, DisplayServer.window_get_size().y],
				"requested_window_size": [_window_size.x, _window_size.y],
				"screen_size": [DisplayServer.screen_get_size().x, DisplayServer.screen_get_size().y]}, "\t"))
	print("SPINE_TECH_DONE failures=%d" % _failures)
	quit(0 if _failures == 0 else 1)
