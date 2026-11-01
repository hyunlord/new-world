# Windowed real-input regression for sticky work headers. No automatic commands.
# Bridge: set_sim_speed, get_work_priorities, get_time_snapshot, get_command_log;
# an explicit cell click exercises the existing priority command handler.
extends SceneTree

const Store = preload("res://scripts/ui/mux/settings_store.gd")
const Priority = preload("res://scripts/ui/panels/priority_panel.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
var _failed: int = 0
var _checks: Array[Dictionary] = []
var _directory: String

func _init() -> void:
	call_deferred("_run")

func _check(label: String, ok: bool, details: Dictionary = {}) -> void:
	_checks.append({"check": label, "pass": ok, "details": details})
	print("%s: %s %s" % ["PASS" if ok else "FAIL", label, JSON.stringify(details)])
	if not ok:
		_failed += 1

func _frames(count: int = 4) -> void:
	for _i: int in count:
		await process_frame

func _rect(control: Control) -> Rect2:
	return control.get_global_transform_with_canvas() * Rect2(Vector2.ZERO, control.size)

func _move(point: Vector2) -> void:
	var event: InputEventMouseMotion = InputEventMouseMotion.new()
	event.position = point
	Input.parse_input_event(event)
	await _frames()

func _button(button: MouseButton, point: Vector2) -> void:
	for pressed: bool in [true, false]:
		var event: InputEventMouseButton = InputEventMouseButton.new()
		event.button_index = button
		event.position = point
		event.pressed = pressed
		Input.parse_input_event(event)
	await _frames()

func _capture(label: String) -> void:
	await RenderingServer.frame_post_draw
	_check("capture " + label, root.get_texture().get_image().save_png(_directory.path_join(label + ".png")) == OK)

func _run() -> void:
	var base: String = OS.get_environment("M6B_PRIORITY_EVIDENCE_DIR")
	if not base.is_absolute_path():
		push_error("Set absolute M6B_PRIORITY_EVIDENCE_DIR")
		quit(1)
		return
	_directory = base.path_join("run-%d" % OS.get_process_id())
	if DirAccess.make_dir_recursive_absolute(_directory) != OK:
		quit(1)
		return
	var store: RefCounted = Store.new(_directory.path_join("settings.cfg"))
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	main.get_node("UI/MuxHud").set("store", store)
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	var camera: Camera2D = main.get_node("Camera2D") as Camera2D
	camera.call("set_paused", true)
	await _frames(15)
	var panel: Control = main.find_child("PriorityPanel", true, false) as Control
	var locale: Node = root.get_node("Locale")
	for scale_value: float in [1.0, 1.25]:
		store.call("set_value", "ui_scale", scale_value)
		panel.call("open_roles")
		await _frames(12)
		var scroll: ScrollContainer = panel.find_children("*", "ScrollContainer", true, false)[0] as ScrollContainer
		scroll.scroll_vertical = 0
		await _frames()
		var grid: GridContainer = panel.get("_grid") as GridContainer
		var data: Dictionary = Priority.table(world.call("get_work_priorities"))
		var works: PackedStringArray = data["works"]
		# Old implementation's first grid row is the header; new header is external.
		var header: GridContainer = panel.find_child("PriorityHeaders", true, false) as GridContainer
		var titles: Array[Control] = []
		for w: int in works.size():
			titles.append((header.get_child(w + 1) if header != null else grid.get_child(w + 1)) as Control)
		var before_y: float = _rect(titles[0]).position.y
		var tick: int = int((world.call("get_time_snapshot") as Dictionary)["tick"])
		var log_before: Dictionary = (world.call("get_command_log", 0) as Dictionary).duplicate(true)
		var table_before: Dictionary = Priority.table(world.call("get_work_priorities"))
		var zoom_before: Vector2 = camera.zoom
		var point: Vector2 = _rect(scroll).get_center()
		await _move(point)
		var hovered: Control = root.gui_get_hovered_control()
		_check("wheel hits priority body %.2f" % scale_value, hovered != null and (hovered == scroll or scroll.is_ancestor_of(hovered)))
		for _i: int in 4:
			await _button(MOUSE_BUTTON_WHEEL_DOWN, point)
		_check("actual wheel scrolls rows %.2f" % scale_value, scroll.scroll_vertical > 0, {"scroll": scroll.scroll_vertical})
		_check("header outside vertical scrolling %.2f" % scale_value, not scroll.is_ancestor_of(titles[0]))
		_check("header remains fixed %.2f" % scale_value, absf(_rect(titles[0]).position.y - before_y) < 1.0)
		var body_rect: Rect2 = _rect(scroll)
		for w: int in works.size():
			var title: Label = titles[w] as Label
			_check("visible work header %.2f %s" % [scale_value, works[w]], title.is_visible_in_tree() and _rect(panel).encloses(_rect(title)) and _rect(title).end.y <= body_rect.position.y + 1.0)
			_check("header names bridge column %.2f %s" % [scale_value, works[w]], title.text == EconomyText.work(locale, works[w]))
		_check("scroll leaves tick unchanged %.2f" % scale_value, tick == int((world.call("get_time_snapshot") as Dictionary)["tick"]))
		_check("scroll leaves command log unchanged %.2f" % scale_value, log_before == world.call("get_command_log", 0))
		_check("scroll leaves priorities unchanged %.2f" % scale_value, table_before == Priority.table(world.call("get_work_priorities")))
		_check("scroll leaves camera zoom unchanged %.2f" % scale_value, zoom_before.is_equal_approx(camera.zoom))
		await _capture("scrolled-%.2f" % scale_value)
		var chosen_row: int = -1
		var target_work: int = works.find("chop")
		for r: int in (data["rows"] as Array).size():
			var cell: Button = panel.find_child("Cell_%d_%d" % [r, target_work], true, false) as Button
			if cell != null and body_rect.encloses(_rect(cell)):
				chosen_row = r
				break
		_check("fully visible actual row exists %.2f" % scale_value, chosen_row >= 0 and target_work >= 0)
		if chosen_row < 0 or target_work < 0:
			continue
		for w: int in works.size():
			var cell: Control = panel.find_child("Cell_%d_%d" % [chosen_row, w], true, false) as Control
			_check("header aligned to cell %.2f %s" % [scale_value, works[w]], absf(_rect(titles[w]).get_center().x - _rect(cell).get_center().x) < 1.0 and absf(_rect(titles[w]).size.x - _rect(cell).size.x) < 1.0)
		var target: Button = panel.find_child("Cell_%d_%d" % [chosen_row, target_work], true, false) as Button
		await _move(_rect(target).get_center())
		_check("actual cell receives pointer %.2f" % scale_value, root.gui_get_hovered_control() == target)
		await _button(MOUSE_BUTTON_LEFT, _rect(target).get_center())
		var after: Dictionary = Priority.table(world.call("get_work_priorities"))
		var only_expected: bool = true
		for r: int in (data["rows"] as Array).size():
			for w: int in works.size():
				var previous: int = int(data["rows"][r]["levels"][w])
				var expected: int = Priority.next_level(previous) if r == chosen_row and w == target_work else previous
				only_expected = only_expected and int(after["rows"][r]["levels"][w]) == expected
		_check("click changes only labelled person/work %.2f" % scale_value, only_expected, {"agent": data["rows"][chosen_row]["agent"], "work": works[target_work]})
		var delta: Dictionary = world.call("get_command_log", int(log_before.get("total", 0)))
		var accepted: PackedByteArray = delta.get("accepted", PackedByteArray())
		_check("explicit click sends one accepted command %.2f" % scale_value, accepted.size() == 1 and accepted[0] == 1)
		var role: OptionButton = panel.find_child("Role_%d" % int(data["rows"][chosen_row]["agent"]), true, false) as OptionButton
		_check("roles remain available %.2f" % scale_value, role != null and role.item_count >= 5)
		await _capture("clicked-%.2f" % scale_value)
	var file: FileAccess = FileAccess.open(_directory.path_join("receipt.json"), FileAccess.WRITE)
	_check("receipt opened", file != null)
	if file != null:
		file.store_string(JSON.stringify({"failures": _failed, "checks": _checks, "physical_os_gestures": "NOT_RUN"}, "\t"))
		file.flush()
		_check("receipt written", file.get_error() == OK)
		file.close()
	print("PRIORITY_HEADER_DONE checks=%d fail=%d evidence=%s" % [_checks.size(), _failed, _directory])
	quit(0 if _failed == 0 else 1)
