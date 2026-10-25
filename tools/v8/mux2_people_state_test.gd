# Windowed M-UX2 selected/drafted state evidence, separate from natural-scene visibility.
# Bridge methods: load_world, get_agent_snapshot, cmd_draft, advance_ticks; normal UI performs selection.
# Run: --save /absolute/day1_noon.wsave --out /absolute/new-output-prefix (PID suffix added).
extends SceneTree

const Store = preload("res://scripts/ui/mux/settings_store.gd")
const Provenance = preload("res://tools/v8/mux2_visibility_provenance.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
var _main: Node
var _world: Node
var _camera: Camera2D
var _agents: Node2D
var _inspector: Control
var _out: String
var _checks: Array[Dictionary] = []
var _shots: Array[Dictionary] = []
var _bits: int = -1
var _agent_id: int = -1


func _init() -> void:
	call_deferred("_run")


func _arg(key: String) -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var at: int = args.find(key)
	return args[at + 1] if at >= 0 and at + 1 < args.size() else ""


func _check(label: String, ok: bool) -> void:
	_checks.append({"label": label, "pass": ok})
	print("MUX2_PEOPLE_STATE_%s %s" % ["PASS" if ok else "FAIL", label])


func _frames(count: int) -> void:
	for _i: int in count:
		await process_frame


func _run() -> void:
	var save: String = _arg("--save").simplify_path()
	_out = _arg("--out").simplify_path().trim_suffix("/") + "-%d" % OS.get_process_id()
	var project: String = ProjectSettings.globalize_path("res://").simplify_path().trim_suffix("/")
	if DisplayServer.get_name() == "headless" or not save.is_absolute_path() or not FileAccess.file_exists(save) or not _out.is_absolute_path() or _out.begins_with(project + "/") or DirAccess.dir_exists_absolute(_out):
		push_error("MUX2_PEOPLE_STATE requires windowed GPU, existing absolute save and new external output prefix")
		quit(1)
		return
	if DirAccess.make_dir_recursive_absolute(_out) != OK:
		quit(1)
		return
	print("MUX2_PEOPLE_STATE_OUTPUT " + _out)
	var before: Dictionary = _provenance(save)
	_write("provenance-start.json", before)
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	_main.get_node("UI/MuxHud").set("store", Store.new(_out.path_join("settings.cfg")))
	root.add_child(_main)
	if not bool(_main.get("boot_ok")) or not _main.has_node("WorldSim"):
		_check("bootstrap", false)
		quit(1)
		return
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D") as Camera2D
	_agents = _main.get_node("ColonyView/AgentLayer") as Node2D
	_inspector = _main.get_node("UI/AgentInspectorPanel") as Control
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	_camera.call("set_paused", true)
	for zoom: float in [1.0, 0.5, 0.25]:
		await _states(save, zoom)
	var after: Dictionary = _provenance(save)
	var changed: Array[String] = Provenance.changes(before["hashes"], after["hashes"])
	_check("immutable_inputs", changed.is_empty())
	_check("six_real_framebuffer_crops", _shots.size() == 6)
	var passed: bool = true
	for check: Dictionary in _checks:
		passed = passed and bool(check["pass"])
	_write("provenance-end.json", after)
	_write("people-states.json", {"pass": passed, "checks": _checks, "shots": _shots, "changed_inputs": changed, "provenance_start": before, "provenance_end": after, "policy": "UI click selection and recorded draft commands in isolated fixture; natural eight-scene visibility remains unchanged"})
	print("MUX2_PEOPLE_STATE_DONE " + JSON.stringify({"pass": passed, "checks": _checks.size(), "images": _shots.size(), "out": _out}))
	quit(0 if passed else 1)


func _provenance(save: String) -> Dictionary:
	var result: Dictionary = Provenance.capture(save)
	(result["hashes"] as Dictionary)["res://tools/v8/mux2_people_state_test.gd"] = FileAccess.get_sha256("res://tools/v8/mux2_people_state_test.gd")
	return result


func _states(save: String, zoom: float) -> void:
	_inspector.call("close")
	var loaded: bool = bool(SimReads.mutate(_world, "load_world", [save]))
	_check("load_%.2f" % zoom, loaded)
	if not loaded:
		return
	_camera.zoom = Vector2.ONE * zoom
	await _frames(10)
	if not _choose_adult():
		_check("awake_adult_fixture", false)
		return
	var person: Node2D = (_agents.get("_persons") as Dictionary)[_bits]
	_camera.position = person.position
	_camera.call("_clamp_position")
	await _frames(45)
	_check("lod_mode_%.2f" % zoom, bool(_agents.call("lod_active")) == (zoom == 0.25))
	var plain: Image = await _capture()
	var click: Vector2 = _agents.get_global_transform_with_canvas() * (_agents.call("drawn_rect", _bits) as Rect2).get_center()
	await _click(click)
	await _frames(8)
	_check("ui_selected_%.2f" % zoom, _inspector.visible and int(_inspector.call("get_selected_entity_bits")) == _bits)
	var selected: Image = await _capture()
	var yellow: int = _colour_pixels(selected, plain, _rgba("people.selection_rgba"), zoom)
	_check("selected_pixels_%.2f" % zoom, yellow >= 8)
	_save_shot("%s-selected" % ("full" if zoom == 1.0 else ("half" if zoom == 0.5 else "quarter")), selected, zoom, {"selected_pixels": yellow})
	var reference: Image = selected
	if zoom >= 0.5:
		_inspector.call("close")
		await _frames(6)
		reference = await _capture()
	var command: Dictionary = SimReads.mutate(_world, "cmd_draft", [_agent_id, true])
	_check("draft_accepted_%.2f" % zoom, bool(command.get("accepted", false)))
	_world.advance_ticks(1)
	await _frames(8)
	var marks: Dictionary = _agents.call("person_marks", _bits)
	_check("drafted_state_%.2f" % zoom, bool(marks.get("drafted", false)))
	var drafted: Image = await _capture()
	var red: int = _colour_pixels(drafted, reference, _rgba("people.drafted_rgba"), zoom)
	_check("drafted_red_pixels_%.2f" % zoom, red >= 8)
	var detail: Dictionary = {"drafted_red_pixels": red}
	if zoom == 0.25:
		var retained: int = _colour_pixels(drafted, plain, _rgba("people.selection_rgba"), zoom)
		detail["selected_pixels"] = retained
		_check("quarter_combined_selection_retained", retained >= 8 and _inspector.visible and int(_inspector.call("get_selected_entity_bits")) == _bits)
	_save_shot("%s-drafted" % ("full" if zoom == 1.0 else "half") if zoom >= 0.5 else "quarter-selected-drafted", drafted, zoom, detail)


func _choose_adult() -> bool:
	var snap: Dictionary = _world.get_agent_snapshot()
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var stages: PackedByteArray = snap.get("life_stages", PackedByteArray())
	var persons: Dictionary = _agents.get("_persons")
	var best: float = -1.0
	_bits = -1
	for i: int in mini(bits.size(), mini(ids.size(), stages.size())):
		if stages[i] != 0 or not persons.has(bits[i]):
			continue
		var person: Node2D = persons[bits[i]]
		if str(person.get("activity")) == "sleep" or bool(person.get("drafted")):
			continue
		var clearance: float = INF
		for other: Node2D in persons.values():
			if other != person:
				clearance = minf(clearance, person.position.distance_squared_to(other.position))
		if clearance > best:
			best = clearance
			_bits = bits[i]
			_agent_id = ids[i]
	return _bits >= 0


func _click(at: Vector2) -> void:
	var motion: InputEventMouseMotion = InputEventMouseMotion.new()
	motion.position = at
	motion.global_position = at
	Input.parse_input_event(motion)
	await process_frame
	for pressed: bool in [true, false]:
		var event: InputEventMouseButton = InputEventMouseButton.new()
		event.position = at
		event.global_position = at
		event.button_index = MOUSE_BUTTON_LEFT
		event.pressed = pressed
		Input.parse_input_event(event)
		await process_frame


func _capture() -> Image:
	_main.process_mode = Node.PROCESS_MODE_DISABLED
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var image: Image = root.get_texture().get_image()
	_main.process_mode = Node.PROCESS_MODE_INHERIT
	return image


func _rgba(key: String) -> Color:
	var channels: Array = ArtData.param(key, [])
	return Color8(int(channels[0]), int(channels[1]), int(channels[2]), int(channels[3]))


func _region(zoom: float) -> Rect2i:
	var person: Node2D = (_agents.get("_persons") as Dictionary)[_bits]
	var rectangle: Rect2
	if zoom == 0.25:
		rectangle = _agents.get_global_transform_with_canvas() * (_agents.call("drawn_rect", _bits) as Rect2)
	else:
		var body: Sprite2D = person.get("body") as Sprite2D
		rectangle = body.get_global_transform_with_canvas() * body.get_rect()
	return Rect2i(rectangle.grow(5))


func _colour_pixels(full: Image, reference: Image, wanted: Color, zoom: float) -> int:
	if full == null or reference == null or full.get_size() != Vector2i(1920, 1080) or reference.get_size() != full.get_size():
		return 0
	var region: Rect2i = _region(zoom).intersection(Rect2i(Vector2i.ZERO, full.get_size()))
	var centre: Vector2 = Vector2(region.get_center())
	var count: int = 0
	for y: int in range(region.position.y, region.end.y):
		for x: int in range(region.position.x, region.end.x):
			if zoom == 0.25 and Vector2(x + 0.5, y + 0.5).distance_to(centre) < 12.0:
				continue
			var pixel: Color = full.get_pixel(x, y)
			var old: Color = reference.get_pixel(x, y)
			var matches: bool = maxf(absf(pixel.r - wanted.r), maxf(absf(pixel.g - wanted.g), absf(pixel.b - wanted.b))) <= 0.08
			var changed: bool = maxf(absf(pixel.r - old.r), maxf(absf(pixel.g - old.g), absf(pixel.b - old.b))) > 2.0 / 255.0
			if matches and changed:
				count += 1
	return count


func _save_shot(name: String, image: Image, zoom: float, detail: Dictionary) -> void:
	var centre: Vector2i = _region(zoom).get_center()
	var crop: Rect2i = Rect2i(centre - Vector2i(96, 96), Vector2i(192, 192)).intersection(Rect2i(Vector2i.ZERO, image.get_size()))
	var path: String = _out.path_join(name + ".png")
	_check("save_" + name, image.get_region(crop).save_png(path) == OK)
	var digest: HashingContext = HashingContext.new()
	digest.start(HashingContext.HASH_SHA256)
	digest.update(image.get_data())
	detail.merge({"name": name, "zoom": zoom, "entity_bits": _bits, "agent_id": _agent_id, "crop": [crop.position.x, crop.position.y, crop.size.x, crop.size.y], "png_sha256": FileAccess.get_sha256(path), "frame_rgba_sha256": digest.finish().hex_encode(), "lod": bool(_agents.call("lod_active"))})
	_shots.append(detail)


func _write(name: String, value: Dictionary) -> void:
	var file: FileAccess = FileAccess.open(_out.path_join(name), FileAccess.WRITE)
	if file == null:
		_check("write_" + name, false)
		return
	file.store_string(JSON.stringify(value, "  "))
	file.close()
