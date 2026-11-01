# Windowed dispatched-input acceptance. Physical OS gestures remain outside this harness.
extends SceneTree

const Store = preload("res://scripts/ui/mux/settings_store.gd")
const LISTS: Array[String] = ["ChroniclePanel", "PriorityPanel", "TechPanel", "TeachPanel", "RoomPanel", "NeighboursPanel", "MigrantPanel", "TradePanel", "WorkshopPanel"]
const POPUPS: Array[String] = ["ItemPanel", "CausalPanel", "WhyPanel"]
var _main: Node
var _world: Node
var _camera: Camera2D
var _directory: String
var _rows: Array[Dictionary] = []
var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, ok: bool, detail: Dictionary = {}) -> void:
	var row: Dictionary = {"check": label, "pass": ok, "detail": detail}
	_rows.append(row)
	print("M6B_INPUT " + JSON.stringify(row))
	if not ok:
		_failures += 1


func _frames(count: int = 3) -> void:
	for _i: int in count:
		await process_frame


func _motion(point: Vector2) -> void:
	var event: InputEventMouseMotion = InputEventMouseMotion.new()
	event.position = point
	Input.parse_input_event(event)


func _button(button: MouseButton, pressed: bool, point: Vector2) -> void:
	var event: InputEventMouseButton = InputEventMouseButton.new()
	event.button_index = button
	event.pressed = pressed
	event.position = point
	Input.parse_input_event(event)


func _click(point: Vector2) -> void:
	_motion(point)
	await _frames()
	_button(MOUSE_BUTTON_LEFT, true, point)
	_button(MOUSE_BUTTON_LEFT, false, point)
	await _frames()


func _key(code: Key) -> void:
	for pressed: bool in [true, false]:
		var event: InputEventKey = InputEventKey.new()
		event.keycode = code
		event.physical_keycode = code
		event.pressed = pressed
		Input.parse_input_event(event)
	await _frames()


func _panel(name: String) -> Control:
	return _main.find_child(name, true, false) as Control


func _close_all() -> void:
	for name: String in LISTS + POPUPS + ["AgentInspectorPanel"]:
		var panel: Control = _panel(name)
		if panel != null:
			panel.hide()


func _visible_count(names: Array[String]) -> int:
	var count: int = 0
	for name: String in names:
		var panel: Control = _panel(name)
		if panel != null and panel.is_visible_in_tree():
			count += 1
	return count


func _scrolls(node: Node) -> Array[Control]:
	var found: Array[Control] = []
	if (node is ScrollContainer or node is RichTextLabel) and (node as Control).is_visible_in_tree():
		found.append(node as Control)
		return found
	for child: Node in node.get_children():
		found.append_array(_scrolls(child))
	return found


func _screen_center(control: Control) -> Vector2:
	return control.get_global_transform_with_canvas() * (control.size * 0.5)


func _wheel_panel(name: String) -> void:
	var panel: Control = _panel(name)
	_check(name + " opened", panel != null and panel.is_visible_in_tree())
	if panel == null or not panel.is_visible_in_tree():
		return
	await _frames(5)
	var scrolls: Array[Control] = _scrolls(panel)
	var bodies: Array[Control] = []
	for scroll: Control in scrolls:
		bodies.append(scroll)
	if name == "NeighboursPanel" and bodies.is_empty():
		var body: Control = panel.get_node_or_null("Panel") as Control
		if body != null:
			bodies.append(body)
		print("M6B_INPUT_SCOPE NeighboursPanel has no scroll container; testing its visible list body")
	_check(name + " visible input body exists", not bodies.is_empty())
	if bodies.is_empty():
		return
	for i: int in bodies.size():
		var body: Control = bodies[i]
		var point: Vector2 = _screen_center(body)
		_motion(point)
		await _frames()
		var hovered: Control = root.gui_get_hovered_control()
		var hit: bool = hovered != null and (hovered == panel or panel.is_ancestor_of(hovered))
		_check(name + " wheel hits visible UI body %d" % i, hit, {"point": str(point), "hovered": str(hovered.get_path()) if hovered != null else "none"})
		var zoom_before: Vector2 = _camera.zoom
		var scroll_before: float = _scroll_value(body)
		_button(MOUSE_BUTTON_WHEEL_DOWN, true, point)
		_button(MOUSE_BUTTON_WHEEL_DOWN, false, point)
		await create_timer(0.25).timeout
		_check(name + " wheel leaves camera zoom unchanged %d" % i, _camera.zoom.is_equal_approx(zoom_before), {"before": str(zoom_before), "after": str(_camera.zoom), "scroll_before": scroll_before, "scroll_after": _scroll_value(body), "surface": body.get_class()})
	await _capture(name)


func _world_point() -> Vector2:
	var view: Vector2 = root.get_visible_rect().size
	for y: float in [0.5, 0.4, 0.6, 0.3, 0.7]:
		for x: float in [0.5, 0.4, 0.6, 0.3, 0.7]:
			var point: Vector2 = view * Vector2(x, y)
			_motion(point)
			await _frames(1)
			if root.gui_get_hovered_control() == null:
				return point
	return Vector2(-1, -1)


func _capture(label: String) -> void:
	await RenderingServer.frame_post_draw
	var path: String = _directory.path_join(label + ".png")
	var result: Error = root.get_texture().get_image().save_png(path)
	_check("capture " + label, result == OK, {"path": path, "error": result})


func _run() -> void:
	_directory = OS.get_environment("M6B_INPUT_EVIDENCE_DIR")
	if _directory.is_empty():
		_directory = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	if _directory.is_empty() or not _directory.is_absolute_path():
		push_error("Set absolute M6B_INPUT_EVIDENCE_DIR or WORLDSIM_TEST_SETTINGS_DIR")
		quit(1)
		return
	_directory = _directory.path_join("input-%d" % OS.get_process_id())
	if DirAccess.make_dir_recursive_absolute(_directory) != OK:
		quit(1)
		return
	var store: RefCounted = Store.new(_directory.path_join("settings.cfg"))
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	_main.get_node("UI/MuxHud").set("store", store)
	root.add_child(_main)
	_world = _main.get_node("WorldSim")
	_world.set_process(false)
	_world.call("set_sim_speed", 0.0)
	_camera = _main.get_node("Camera2D") as Camera2D
	_camera.call("set_paused", true)
	await _frames(15)
	await _prepare_ground_stack()
	var initial_tick: int = int(_world.call("get_time_snapshot").get("tick", -1))
	var commands: Dictionary = _world.call("get_command_log", 0)
	var snapshot: Dictionary = _world.call("get_agent_snapshot")
	var ids: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	var xs: PackedInt32Array = snapshot.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snapshot.get("ys", PackedInt32Array())
	if not xs.is_empty() and not ys.is_empty():
		_camera.call("center_on_tile", Vector2i(xs[0], ys[0]))
	var chronicle: Control = _panel("ChroniclePanel")
	for i: int in 40:
		chronicle.call("add_presentation_entry", "M6B input observer row %02d" % i)
	chronicle.call("toggle_visible")
	await _wheel_panel("ChroniclePanel")
	_panel("PriorityPanel").call("open_roles")
	await _frames()
	_check("opening priorities replaces chronicle list", _visible_count(LISTS) == 1 and not chronicle.visible)
	await _wheel_panel("PriorityPanel")
	_panel("TechPanel").call("open")
	await _frames()
	_check("opening tech replaces priorities list", _visible_count(LISTS) == 1 and not _panel("PriorityPanel").visible)
	await _wheel_panel("TechPanel")
	_panel("NeighboursPanel").call("open")
	await _frames()
	_check("opening neighbours replaces tech list", _visible_count(LISTS) == 1 and not _panel("TechPanel").visible)
	await _wheel_panel("NeighboursPanel")
	_close_all()
	if not ids.is_empty():
		_panel("AgentInspectorPanel").call("display_agent", _world.call("get_agent_detail", ids[0]))
		await _wheel_panel("AgentInspectorPanel")
	else:
		_check("inspector coverage", false, {"status": "NOT_RUN", "reason": "no live founder"})
	_close_all()
	var point: Vector2 = await _world_point()
	_check("positive-control unobstructed world point", point.x >= 0)
	if point.x >= 0:
		var before: Vector2 = _camera.zoom
		_button(MOUSE_BUTTON_WHEEL_UP, true, point)
		_button(MOUSE_BUTTON_WHEEL_UP, false, point)
		await create_timer(0.25).timeout
		_check("world wheel changes camera zoom", not _camera.zoom.is_equal_approx(before), {"before": str(before), "after": str(_camera.zoom)})
	await _popup_checks()
	_close_all()
	var help: Control = _main.get_node("UI/MuxHud/Help") as Control
	if help.visible:
		help.hide()
	await _key(KEY_SLASH)
	_check("question-mark key opens full key help", help.is_visible_in_tree())
	await _capture("key-help-expanded")
	await _key(KEY_ESCAPE)
	_check("escape folds key help", not help.is_visible_in_tree())
	for scale_value: float in [1.0, 1.25]:
		store.call("set_value", "ui_scale", scale_value)
		_panel("PriorityPanel").call("open_roles")
		await _frames(8)
		var choice: OptionButton = _find_role_choice(_panel("PriorityPanel"))
		_check("role selector visible at %.2f" % scale_value, choice != null and choice.is_visible_in_tree())
		if choice != null:
			var ancestor: Node = choice.get_parent()
			while ancestor != null:
				if ancestor is ScrollContainer:
					(ancestor as ScrollContainer).ensure_control_visible(choice)
				ancestor = ancestor.get_parent()
			await _frames(5)
			var point_on_choice: Vector2 = _screen_center(choice)
			_motion(point_on_choice)
			await _frames()
			var hovered: Control = root.gui_get_hovered_control()
			var hit: bool = hovered != null and (hovered == choice or choice.is_ancestor_of(hovered))
			_check("role choice is within the scroll viewport at %.2f" % scale_value, _within_scroll_views(choice, point_on_choice))
			_check("role click hits actual selector at %.2f" % scale_value, hit, {"point": str(point_on_choice), "hovered": str(hovered.get_path()) if hovered != null else "none"})
			await _click(point_on_choice)
			_check("role menu opens from actual click at %.2f" % scale_value, choice.get_popup().visible and choice.item_count >= 5)
			await _capture("roles-%.2f" % scale_value)
			await _key(KEY_ESCAPE)
	_check("input checks preserve simulation tick", int(_world.call("get_time_snapshot").get("tick", -1)) == initial_tick)
	_check("observer inputs send no simulation commands", _world.call("get_command_log", 0) == commands)
	var manifest: FileAccess = FileAccess.open(_directory.path_join("manifest.json"), FileAccess.WRITE)
	if manifest == null:
		_failures += 1
	else:
		manifest.store_string(JSON.stringify({"rows": _rows, "failures": _failures, "initial_tick": initial_tick, "physical_os_gestures": "NOT_RUN", "viewport": str(root.get_visible_rect())}, "\t"))
		if manifest.get_error() != OK:
			_failures += 1
		manifest.close()
	print("M6B_INPUT_EVIDENCE " + _directory)
	quit(0 if _failures == 0 else 1)


func _find_role_choice(node: Node) -> OptionButton:
	if node is OptionButton and str(node.name).begins_with("Role_"):
		return node as OptionButton
	for child: Node in node.get_children():
		var found: OptionButton = _find_role_choice(child)
		if found != null:
			return found
	return null


func _popup_checks() -> void:
	var tile: Vector2i = _loose_stack_tile()
	if tile.x < 0:
		_check("item popup coverage", false, {"status": "NOT_RUN", "reason": "no inspectable loose ground stack"})
		return
	_camera.call("center_on_tile", tile)
	await _frames(10)
	var stack_point: Vector2 = root.get_canvas_transform() * (Vector2(tile) * 64.0 + Vector2(5.0, 59.0))
	await _click(stack_point)
	var item: Control = _panel("ItemPanel")
	var causal: Control = _panel("CausalPanel")
	_check("actual ground-stack click opens item popup", item != null and item.is_visible_in_tree(), {"tile": str(tile), "point": str(stack_point)})
	if item == null or not item.is_visible_in_tree():
		await _capture("item-open-failed")
		return
	var why: Button = item.get("_why") as Button
	var why_point: Vector2 = _screen_center(why)
	_motion(why_point)
	await _frames()
	var hovered: Control = root.gui_get_hovered_control()
	_check("why click hits actual item button", hovered == why or (hovered != null and why.is_ancestor_of(hovered)), {"hovered": str(hovered.get_path()) if hovered != null else "none", "point": str(why_point), "item_rect": str(item.get_global_rect())})
	await _capture("popup-item-before-why")
	await _click(why_point)
	if causal.is_visible_in_tree():
		_check("actual why click replaces item with causal popup", not item.visible and _visible_count(POPUPS) == 1)
	else:
		var history: Array = _world.call("get_tile_causal_history", tile.x, tile.y)
		var filtered: Array = causal.call("_filter_labeled", history)
		var toast: Label = _main.get_node("UI/OrderBar").get("_toast") as Label
		var expected: String = str(root.get_node("Locale").call("ltr", "MUX_WHY_EMPTY"))
		_check("empty item cause shows actual empty-state toast", filtered.is_empty() and str(causal.call("wait_text", tile)).is_empty() and toast.is_visible_in_tree() and toast.text.replace("\u2060", "") == expected.replace("\u2060", "") and item.visible)
	await _capture("popup-after-why")
	var people: Dictionary = _world.call("get_agent_snapshot")
	var bits: PackedInt64Array = people.get("ids", PackedInt64Array())
	_check("actual founder mind target exists", not bits.is_empty())
	if bits.is_empty():
		return
	var inspector: Control = _panel("AgentInspectorPanel")
	inspector.call("display_agent", _world.call("get_agent_detail", bits[0]))
	await _frames(5)
	var mind: Dictionary = inspector.get("_mind_snapshot")
	_check("founder causal fixture contains actual mind", bool(mind.get("found", false)))
	var person_why: Button = inspector.get("_why_button") as Button
	for scroll: Control in _scrolls(inspector):
		if scroll is ScrollContainer and scroll.is_ancestor_of(person_why):
			(scroll as ScrollContainer).ensure_control_visible(person_why)
	await _frames(5)
	_motion(_screen_center(person_why))
	await _frames()
	_check("founder why click hits actual button", root.gui_get_hovered_control() == person_why)
	await _click(_screen_center(person_why))
	_check("actual founder why click replaces item with causal popup", causal.is_visible_in_tree() and not item.visible and _visible_count(POPUPS) == 1)
	await _capture("popup-founder-diagnosis")
	inspector.hide()
	_camera.call("center_on_tile", tile)
	await _frames(10)
	stack_point = root.get_canvas_transform() * (Vector2(tile) * 64.0 + Vector2(5.0, 59.0))
	await _click(stack_point)
	_check("actual stack click replaces causal with item popup", item.is_visible_in_tree() and not causal.visible and _visible_count(POPUPS) == 1)
	var point: Vector2 = await _world_point()
	_check("outside popup click target available", point.x >= 0)
	if point.x >= 0:
		await _click(point)
		_check("actual outside click closes item popup", not item.is_visible_in_tree())
	await _capture("popup-after-outside-click")


func _scroll_value(body: Control) -> float:
	if body is ScrollContainer:
		return float((body as ScrollContainer).scroll_vertical)
	if body is RichTextLabel:
		return (body as RichTextLabel).get_v_scroll_bar().value
	return -1.0


func _within_scroll_views(control: Control, point: Vector2) -> bool:
	var ancestor: Node = control.get_parent()
	while ancestor != null:
		if ancestor is ScrollContainer:
			var scroll: ScrollContainer = ancestor as ScrollContainer
			var local: Vector2 = scroll.get_global_transform_with_canvas().affine_inverse() * point
			if not Rect2(Vector2.ZERO, scroll.size).has_point(local):
				return false
		ancestor = ancestor.get_parent()
	return true


func _prepare_ground_stack() -> void:
	var start: int = int(_world.call("get_time_snapshot").get("tick", -1))
	var found: bool = false
	for step: int in 25:
		if _loose_stack_tile().x >= 0:
			found = true
			break
		if step < 24:
			SimReads.mutate(_world, "advance_ticks", [60])
			await _frames(2)
	_world.set_process(false)
	_world.call("set_sim_speed", 0.0)
	_check("deterministic setup reaches actual ground stack", found, {"initial_tick": start, "fixture_tick": int(_world.call("get_time_snapshot").get("tick", -1)), "maximum_advance": 1440})
	print("M6B_INPUT_SETUP_COMMAND_LOG " + JSON.stringify(_world.call("get_command_log", 0)))


func _loose_stack_tile() -> Vector2i:
	var overlay: Dictionary = _world.call("get_colony_overlay")
	var xs: PackedInt32Array = overlay.get("item_xs", PackedInt32Array())
	var ys: PackedInt32Array = overlay.get("item_ys", PackedInt32Array())
	var zones_x: PackedInt32Array = overlay.get("zone_xs", PackedInt32Array())
	var zones_y: PackedInt32Array = overlay.get("zone_ys", PackedInt32Array())
	var buildings: Dictionary = _world.call("get_building_snapshot")
	var bx: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var by: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	var kinds: PackedByteArray = buildings.get("kinds", PackedByteArray())
	for i: int in mini(xs.size(), ys.size()):
		var blocked: bool = false
		for j: int in mini(zones_x.size(), zones_y.size()):
			if xs[i] == zones_x[j] and ys[i] == zones_y[j]:
				blocked = true
		for j: int in mini(kinds.size(), mini(bx.size(), by.size())):
			if kinds[j] == 0 and absi(xs[i] - bx[j]) <= 1 and absi(ys[i] - by[j]) <= 1:
				blocked = true
		if not blocked:
			return Vector2i(xs[i], ys[i])
	return Vector2i(-1, -1)
