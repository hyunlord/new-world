extends SceneTree
## Windowed acceptance only. All tested actions enter through Input.parse_input_event.
const Reads = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
var _main: Node2D
var _world: Node
var _agents: Node
var _inspector: Control
var _chronicle: Control
var _camera: Camera2D
var _out: String
var _steps: Array[Dictionary] = []
var _errors: Array[String] = []
var _clicked_meta: String = ""
var _subject: int = -1
var _parent_id: int = -1
var _tick: int = 0

class InputWitness extends Node:
	var recorded: int = 0
	func _input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and recorded < 16:
			recorded += 1
			print("P13_INPUT_WITNESS ", JSON.stringify({"pressed": event.pressed, "event_position": str(event.position), "viewport_mouse": str(get_viewport().get_mouse_position())}))


func _initialize() -> void:
	_run.call_deferred()

func _run() -> void:
	_out = OS.get_environment("P13_REAL_INPUT_OUT")
	if DisplayServer.get_name() == "headless" or _out.is_empty() or DirAccess.dir_exists_absolute(_out):
		push_error("Real-input acceptance requires windowed Godot and a NEW P13_REAL_INPUT_OUT directory; NOT_RUN in headless")
		quit(2)
		return
	if DirAccess.make_dir_recursive_absolute(_out) != OK:
		quit(2)
		return
	root.size = Vector2i(1920, 1080)
	var scene: PackedScene = load("res://scenes/main.tscn") as PackedScene
	_main = scene.instantiate() as Node2D
	_world = _main.get_node("WorldSim")
	_world.call("set_sim_speed", 0.0)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(_main)
	current_scene = _main
	root.add_child(InputWitness.new())
	_agents = _main.get_node("AgentRenderer")
	_inspector = _main.get_node("UI/AgentInspectorPanel") as Control
	_chronicle = _main.get_node("UI/ChroniclePanel") as Control
	_camera = _main.get_node("Camera2D") as Camera2D
	_camera.call("set_auto_track", false)
	_camera.position_smoothing_enabled = false
	_camera.process_mode = Node.PROCESS_MODE_DISABLED
	_camera.zoom = Vector2(4, 4)
	await _settle()
	Reads.mutate(_world, "reset_world", [20261010])
	await _settle()
	var info: Dictionary = Reads.read(_world, "get_world_info", true)
	var tpd: int = int(info["ticks_per_day"])
	for day: int in range(60, 91):
		var next_tick: int = (day - 1) * tpd + floori(10.0 * tpd / 24.0) - int(info["clock_offset_ticks"])
		Reads.mutate(_world, "advance_ticks", [next_tick - _tick])
		_tick = next_tick
		await _settle()
		if _find_dead_parent():
			break
	if not await _step("00_fixed_world", _subject >= 0, {"seed": 20261010, "tick": _tick, "subject": _subject, "dead_parent": _parent_id, "world_info": info}):
		_finish()
		return
	# Arrangement is setup: real living sprites and ids; no selection or input handler calls.
	var sprites: Dictionary = _agents.get("_sprites")
	var chosen: Array[int] = [_subject]
	for id: int in sprites:
		if id != _subject and chosen.size() < 3 and (sprites[id] as Sprite2D).texture != null:
			chosen.append(id)
	if chosen.size() != 3:
		await _step("01_three_agents_available", false, {"ids": chosen})
		_finish()
		return
	_agents.set_process(false)
	var click_world := Vector2(936, 522)
	_camera.position = click_world
	for id: int in sprites:
		var sprite: Sprite2D = sprites[id]
		sprite.visible = id in chosen
		if id in chosen:
			var opaque: Rect2 = _opaque(sprite)
			sprite.position = click_world - opaque.get_center()
	chosen.sort_custom(func(a: int, b: int) -> bool:
		var ay: float = (sprites[a] as Sprite2D).position.y
		var by: float = (sprites[b] as Sprite2D).position.y
		return ay > by if ay != by else a > b)
	await _settle()
	var click_screen: Vector2 = _main.get_canvas_transform() * click_world
	for index: int in range(4):
		await _click(click_screen)
		var wanted: int = chosen[index % 3]
		if not await _step("02_click_%d" % index, _inspector.visible and int(_inspector.get("_selected_person")) == wanted, {"expected": wanted, "actual": _inspector.get("_selected_person"), "point": [click_screen.x, click_screen.y], "viewport_mouse": str(root.get_mouse_position()), "world_mouse": str((_main.get_node("WorldRenderer") as Node2D).get_global_mouse_position()), "drawn_pick_at_intended": _agents.call("drawn_entity_at", click_world), "chosen": chosen}):
			_finish()
			return
	for _attempt: int in range(3):
		if int(_inspector.get("_selected_person")) == _subject:
			break
		await _click(click_screen)
	var buttons: Array = _inspector.get("_family_buttons")
	var parent_button: Button
	for value: Variant in buttons:
		var button: Button = value as Button
		if int(button.get_meta("family_agent_id", -1)) == _parent_id:
			parent_button = button
	var scroll: ScrollContainer = _inspector.get("_scroll") as ScrollContainer
	if parent_button == null or not await _wheel_to_control(parent_button, scroll):
		await _step("03_parent_link_visible", false, {"parent": _parent_id})
		_finish()
		return
	await _click(parent_button.get_global_rect().get_center())
	if not await _step("03_dead_parent_link", bool(_inspector.get("_record_mode")) and int(_inspector.get("_selected_person")) == _parent_id, {"parent": _parent_id, "record_mode": _inspector.get("_record_mode")}):
		_finish()
		return
	await _wheel_end(scroll)
	var more: Button = _inspector.get("_life_more") as Button
	if more.visible and not more.disabled and scroll.get_global_rect().encloses(more.get_global_rect()):
		await _click(more.get_global_rect().get_center())
		await _wheel_end(scroll)
	var bar: VScrollBar = scroll.get_v_scroll_bar()
	var life: RichTextLabel = _inspector.get("_life_text") as RichTextLabel
	var last_line: int = life.get_line_count() - 1
	var line_height: float = life.get_theme_font("normal_font").get_height(life.get_theme_font_size("normal_font_size"))
	var last_top: float = life.global_position.y + life.get_line_offset(maxi(0, last_line))
	var viewport: Rect2 = scroll.get_global_rect()
	var fully_visible: bool = last_line >= 0 and last_top >= viewport.position.y and last_top + line_height <= viewport.end.y
	if not await _step("04_wheel_end_last_line", is_equal_approx(bar.value, bar.max_value - bar.page) and fully_visible, {"scroll": bar.value, "maximum": bar.max_value - bar.page, "last_line_top": last_top, "line_height": line_height, "viewport_bottom": viewport.end.y}):
		_finish()
		return
	await _key(KEY_ESCAPE)
	if not await _step("05_escape_record", not bool(_inspector.get("_record_mode")), {"record_mode": _inspector.get("_record_mode"), "visible": _inspector.visible}):
		_finish()
		return
	await _key(KEY_C)
	if not await _step("06_chronicle_open", _chronicle.visible, {}):
		_finish()
		return
	var body: RichTextLabel = _chronicle.get("_body") as RichTextLabel
	body.meta_clicked.connect(_on_meta_clicked)
	var link_point: Vector2 = await _find_person_link(body)
	var link: String = _clicked_meta
	if link_point.x < 0:
		await _step("07_chronicle_name_link", false, {"reason": "No visible real person link responded to input click probes"})
		_finish()
		return
	var linked_id: int = int(link.get_slice(":", 1))
	await _click(link_point)
	if not await _step("07_chronicle_name_link", _inspector.visible and int(_inspector.get("_selected_person")) == linked_id, {"link": link, "selected": _inspector.get("_selected_person")}):
		_finish()
		return
	await _key(KEY_C)
	if not await _step("08_chronicle_close", not _chronicle.visible, {}):
		_finish()
		return
	var locale: Node = root.get_node("Locale")
	if str(locale.get("_current_lang")) != "en":
		await _key(KEY_L)
	await _key(KEY_L)
	if not await _step("09_language_korean", str(locale.get("_current_lang")) == "ko", {"language": locale.get("_current_lang")}):
		_finish()
		return
	await _key(KEY_L)
	var hangul: Array[String] = []
	_scan_hangul(_main, hangul)
	await _step("10_language_english", str(locale.get("_current_lang")) == "en" and hangul.is_empty(), {"language": locale.get("_current_lang"), "visible_hangul": hangul})
	_finish()

func _find_dead_parent() -> bool:
	var snapshot: Dictionary = Reads.read(_world, "get_agent_snapshot", true)
	var ids: Array = Array(snapshot["agent_ids"])
	ids.sort()
	for value: Variant in ids:
		var id: int = int(value)
		var person: Dictionary = _world.call("get_person", id)
		for key: String in ["mother", "father"]:
			var parent_id: int = int(person.get(key, -1))
			if parent_id < 0:
				continue
			var parent: Dictionary = _world.call("get_person", parent_id)
			if bool(parent.get("found", false)) and not bool(parent.get("alive", false)):
				_subject = id
				_parent_id = parent_id
				return true
	return false

func _opaque(sprite: Sprite2D) -> Rect2:
	var pixels: Image = sprite.texture.get_image()
	if sprite.region_enabled:
		pixels = pixels.get_region(Rect2i(sprite.region_rect))
	var opaque := Rect2(pixels.get_used_rect())
	var frame: Rect2 = sprite.get_rect()
	if sprite.flip_h:
		opaque.position.x = frame.size.x - opaque.end.x
	if sprite.flip_v:
		opaque.position.y = frame.size.y - opaque.end.y
	opaque.position += frame.position
	return opaque

func _settle() -> void:
	await process_frame
	await process_frame

func _move(point: Vector2) -> void:
	var event := InputEventMouseMotion.new()
	event.position = point
	event.global_position = point
	Input.parse_input_event(event)
	Input.flush_buffered_events()

func _click_now(point: Vector2) -> void:
	_move(point)
	for pressed: bool in [true, false]:
		var event := InputEventMouseButton.new()
		event.position = point
		event.global_position = point
		event.button_index = MOUSE_BUTTON_LEFT
		event.button_mask = MOUSE_BUTTON_MASK_LEFT if pressed else 0
		event.pressed = pressed
		Input.parse_input_event(event)
		Input.flush_buffered_events()

func _click(point: Vector2) -> void:
	_click_now(point)
	await _settle()

func _key(code: Key) -> void:
	for pressed: bool in [true, false]:
		var event := InputEventKey.new()
		event.keycode = code
		event.physical_keycode = code
		event.pressed = pressed
		Input.parse_input_event(event)
		Input.flush_buffered_events()
	await _settle()

func _wheel(point: Vector2, down: bool = true) -> void:
	_move(point)
	var event := InputEventMouseButton.new()
	event.position = point
	event.global_position = point
	event.button_index = MOUSE_BUTTON_WHEEL_DOWN if down else MOUSE_BUTTON_WHEEL_UP
	event.pressed = true
	Input.parse_input_event(event)
	Input.flush_buffered_events()
	await _settle()

func _wheel_to_control(control: Control, scroll: ScrollContainer) -> bool:
	for _attempt: int in range(160):
		if control.is_visible_in_tree() and scroll.get_global_rect().encloses(control.get_global_rect()):
			return true
		await _wheel(scroll.get_global_rect().get_center(), control.global_position.y > scroll.global_position.y)
	return false

func _wheel_end(scroll: ScrollContainer) -> void:
	for _attempt: int in range(200):
		var bar: VScrollBar = scroll.get_v_scroll_bar()
		if is_equal_approx(bar.value, bar.max_value - bar.page):
			return
		await _wheel(scroll.get_global_rect().get_center())

func _on_meta_clicked(meta: Variant) -> void:
	_clicked_meta = str(meta)

func _find_person_link(body: RichTextLabel) -> Vector2:
	var bounds: Rect2 = body.get_global_rect()
	for y: int in range(ceili(bounds.position.y) + 5, floori(bounds.end.y) - 5, 8):
		for x: int in range(ceili(bounds.position.x) + 5, floori(bounds.end.x) - 5, 8):
			var point := Vector2(x, y)
			_clicked_meta = ""
			_click_now(point)
			if _clicked_meta.begins_with("person:"):
				await _settle()
				return point
		await process_frame
	return Vector2(-1, -1)

func _scan_hangul(node: Node, found: Array[String]) -> void:
	if node is Control and (node as Control).is_visible_in_tree():
		var text: String = ""
		if node is Label or node is Button:
			text = str(node.get("text"))
		elif node is RichTextLabel:
			text = (node as RichTextLabel).get_parsed_text()
		for index: int in text.length():
			var code: int = text.unicode_at(index)
			if (code >= 0xAC00 and code <= 0xD7AF) or (code >= 0x1100 and code <= 0x11FF) or (code >= 0x3130 and code <= 0x318F):
				found.append(str(node.get_path()) + ": " + text)
				break
	for child: Node in node.get_children():
		_scan_hangul(child, found)

func _step(label: String, passed: bool, state: Dictionary) -> bool:
	await _settle()
	RenderingServer.force_draw(false)
	var path: String = _out.path_join(label + ".png")
	var image: Image = root.get_texture().get_image()
	var saved: bool = image != null and image.get_size() == Vector2i(1920, 1080) and image.save_png(path) == OK
	var row: Dictionary = {"step": label, "passed": passed and saved, "state": state, "screenshot": path, "sha256": FileAccess.get_sha256(path) if saved else ""}
	_steps.append(row)
	if not passed or not saved:
		_errors.append(label)
	print("REAL_INPUT_P12 ", JSON.stringify(row))
	return passed and saved

func _finish() -> void:
	var result: Dictionary = {"valid": _errors.is_empty(), "errors": _errors, "steps": _steps, "input_route": "Input.parse_input_event", "windowed": true, "seed": 20261010, "tick": _tick, "subject": _subject, "dead_parent": _parent_id, "godot": Engine.get_version_info(), "test_sha256": FileAccess.get_sha256("res://scripts/test/art/test_real_input_p13.gd"), "bridge_sha256": FileAccess.get_sha256("res://rust/target/debug/libsim_bridge.dylib")}
	var file := FileAccess.open(_out.path_join("result.json"), FileAccess.WRITE)
	if file == null:
		quit(1)
		return
	file.store_string(JSON.stringify(result, "\t") + "\n")
	file.close()
	quit(0 if _errors.is_empty() else 1)
