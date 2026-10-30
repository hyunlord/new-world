# R1 actual-framebuffer regression: floating Actions must not erase the half-zoom person behind them.
# Bridge methods: load_world via SimReads, get_time_snapshot/get_world_info; no simulation advancement.
extends "res://tools/v8/mux2_visibility_test.gd"


func _run() -> void:
	_out = _arg("--out").simplify_path()
	var manifest: String = _arg("--manifest")
	if not _out.is_absolute_path() or _out.begins_with(ProjectSettings.globalize_path("res://")) or DirAccess.dir_exists_absolute(_out) or DisplayServer.get_name() == "headless":
		push_error("M6A_ACTIONS requires GPU, external NEW output directory and original visibility save manifest")
		quit(1)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	var provenance: Dictionary = Provenance.capture(manifest)
	provenance["hashes"][get_script().resource_path] = FileAccess.get_sha256(get_script().resource_path)
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(manifest))
	if not parsed is Dictionary or not parsed.get("scenes") is Array:
		quit(1)
		return
	var row: Dictionary = {}
	for entry: Dictionary in parsed["scenes"]:
		if str(entry.get("id", "")) == "day1_noon":
			row = entry
	var save: String = str(row.get("save", ""))
	if save.is_empty() or FileAccess.get_sha256(save) != str(row.get("sha256", "missing")):
		quit(1)
		return
	provenance["hashes"][save] = FileAccess.get_sha256(save)
	root.get_node("Locale").call("set_language", "ko")
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	_main.get_node("UI/MuxHud").set("store", Store.new(_out.path_join("settings.cfg")))
	root.add_child(_main)
	_world = _main.get_node("WorldSim")
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	_camera = _main.get_node("Camera2D") as Camera2D
	_agents = _main.get_node("ColonyView/AgentLayer") as Node2D
	if not bool(SimReads.mutate(_world, "load_world", [save])):
		quit(1)
		return
	_camera.call("set_paused", true)
	_camera.zoom = Vector2.ONE * 0.5
	_camera.call("center_on_camp")
	root.warp_mouse(Vector2(700, 800))
	for _i: int in 45:
		await process_frame
	_main.process_mode = Node.PROCESS_MODE_DISABLED
	var clock: Dictionary = _world.get_time_snapshot()
	if int(clock.get("tick", -1)) != 240 or int(_world.get_world_info().get("seed", -1)) != 0:
		_failures.append("wrong original seed/tick")
	if _camera.position != Vector2(5856, 6496):
		_failures.append("original fixture camera changed")
	var ui: CanvasLayer = _main.get_node("UI") as CanvasLayer
	var persons: Dictionary = _agents.get("_persons")
	if not persons.has(4294967305):
		quit(1)
		return
	var person: Node2D = persons[4294967305] as Node2D
	var body: Sprite2D = person.get("body") as Sprite2D
	var results: Array[Dictionary] = []
	for phase: float in [0.0, 0.3]:
		_agents.set("_clock", phase)
		_agents.call("_process", 0.0)
		var name: String = "idle" if phase == 0.0 else "work"
		var hidden: Array[CanvasItem] = []
		for child: Node in _main.get_node("ColonyView").get_children():
			if child is CanvasItem and child.name != "GroundLayer" and child.visible:
				hidden.append(child)
				child.hide()
		ui.hide()
		var ground: Image = await _capture(_out.path_join(name + "-ground.png"))
		for child: CanvasItem in hidden:
			child.show()
		ui.show()
		var bounds: Rect2i = _bounds(person, null, 0, false)
		var expected_bounds: Rect2i = Rect2i(792, 132, 16, 32) if phase == 0.0 else Rect2i(789, 127, 21, 38)
		if bounds != expected_bounds:
			_failures.append("original fixture body bounds changed: " + name)
		var full: Image = await _capture(_out.path_join(name + "-full.png"))
		body.hide()
		var removed: Image = await _capture(_out.path_join(name + "-removed.png"))
		body.show()
		var restored: Image = await _capture(_out.path_join(name + "-restored.png"))
		var result: Dictionary = Metrics.measure(full, removed, ground, bounds, 60, restored)
		(result["mask"] as Image).save_png(_out.path_join(name + "-mask.png"))
		result.erase("mask")
		result["pose"] = person.get("pose")
		results.append(result)
		if not bool(result.get("pass", false)) or int(result.get("unstable_pixels", -1)) != 0:
			_failures.append("person behind Actions unreadable: " + name)
	var actions: Control = _main.get_node("UI/MuxHud/Actions") as Control
	var controls: Array[Dictionary] = []
	for child: Node in actions.get_children():
		var button: Button = child as Button
		controls.append({"name": button.name, "text": button.text, "rect": str(button.get_global_rect()), "font_size": button.get_theme_font_size("font_size")})
		if button.text.is_empty() or not button.visible:
			_failures.append("missing visible action " + str(button.name))
		for key: String in ["normal", "hover", "pressed", "hover_pressed", "disabled", "focus"]:
			ResourceSaver.save(button.get_theme_stylebox(key), _out.path_join(str(button.name) + "-" + key + ".tres"))
	var finish: Dictionary = Provenance.capture(manifest)
	finish["hashes"][get_script().resource_path] = FileAccess.get_sha256(get_script().resource_path)
	finish["hashes"][save] = FileAccess.get_sha256(save)
	var changes: Array[String] = Provenance.changes(provenance["hashes"], finish["hashes"])
	_failures.append_array(changes)
	_write(_out.path_join("result.json"), {"clock": clock, "camera": [_camera.position.x, _camera.position.y], "zoom": 0.5, "results": results, "controls": controls, "actions_rect": str(actions.get_global_rect()), "provenance_start": provenance, "provenance_end": finish, "changes": changes, "failures": _failures, "pass": _failures.is_empty()})
	print("M6A_ACTIONS_VISIBILITY_DONE " + JSON.stringify({"rows": results.size(), "failures": _failures, "pass": _failures.is_empty()}))
	quit(0 if _failures.is_empty() else 1)
