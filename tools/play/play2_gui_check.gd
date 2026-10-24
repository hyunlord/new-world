# V8-PLAY2 windowed input audit. Tooling only; never loaded by the game.
# Bridge reads: get_time_snapshot, get_world_info, get_agent_snapshot, get_drafted,
# get_building_snapshot, get_space, can_place_blueprint. Commands enter through real input handlers.
# Run with --script <absolute path> -- --out <absolute evidence directory>.
# Synthetic window input is not a human keyboard/OS shortcut acceptance test.
extends SceneTree

const ArtData = preload("res://scripts/world/art/art_data.gd")
const View = preload("res://scripts/world/colony/view.gd")
const PARAMS: String = "res://assets/v8art/render_params.json"
var _out: String = ""
var _main: Node
var _world: Node
var _camera: Camera2D
var _agents: Node
var _checks: Array[Dictionary] = []
var _started: int = 0
var _finished: bool = false
var _params_original: PackedByteArray = PackedByteArray()
var _params_changed: bool = false
var _save_path: String = ""
var _save_original: PackedByteArray = PackedByteArray()
var _save_existed: bool = false
var _save_touched: bool = false
var _wall_tile: Vector2i = Vector2i(-1, -1)
var _wall_before: Image

func _init() -> void:
	_started = Time.get_ticks_msec()
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--out" and i + 1 < args.size():
			_out = args[i + 1]
	call_deferred("_run")

func _process(_delta: float) -> bool:
	if not _finished and Time.get_ticks_msec() - _started > 180000:
		_check("watchdog", false, "180 second deadline")
		_finish()
	return false

func _check(label: String, ok: bool, detail: Variant = "") -> void:
	_checks.append({"check": label, "pass": ok, "detail": str(detail)})
	print("PLAY2_GUI_%s %s %s" % ["PASS" if ok else "FAIL", label, str(detail)])

func _write_bytes(path: String, bytes: PackedByteArray) -> bool:
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		return false
	file.store_buffer(bytes)
	file.close()
	return FileAccess.get_file_as_bytes(path) == bytes

func _cleanup() -> void:
	if _params_changed:
		_check("params restored byte exact", _write_bytes(PARAMS, _params_original))
		_params_changed = false
	if _save_touched:
		if _save_existed:
			_check("existing save restored byte exact", _write_bytes(_save_path, _save_original))
		else:
			var err: Error = DirAccess.remove_absolute(ProjectSettings.globalize_path(_save_path))
			_check("temporary save removed", err == OK or not FileAccess.file_exists(_save_path))
		_save_touched = false

func _finish() -> void:
	if _finished:
		return
	_finished = true
	_cleanup()
	var failures: int = 0
	for row: Dictionary in _checks:
		if not bool(row["pass"]):
			failures += 1
	var report: Dictionary = {"method": "windowed synthetic Input.parse_input_event; not human manual input", "display": DisplayServer.get_name(), "checks": _checks, "failures": failures, "elapsed_ms": Time.get_ticks_msec() - _started}
	if not _out.is_empty():
		_write_bytes(_out.path_join("gui-check.json"), JSON.stringify(report, "  ").to_utf8_buffer())
	print("PLAY2_GUI_DONE failures=%d" % failures)
	quit(0 if failures == 0 else 1)

func _frames(count: int = 5) -> void:
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
	await _frames()

func _move(at: Vector2) -> void:
	var event: InputEventMouseMotion = InputEventMouseMotion.new()
	event.position = at
	event.global_position = at
	Input.parse_input_event(event)
	await process_frame

func _mouse(button: MouseButton, pressed: bool, at: Vector2) -> void:
	var event: InputEventMouseButton = InputEventMouseButton.new()
	event.button_index = button
	event.pressed = pressed
	event.position = at
	event.global_position = at
	Input.parse_input_event(event)
	await process_frame

func _click(at: Vector2) -> void:
	await _move(at)
	await _mouse(MOUSE_BUTTON_LEFT, true, at)
	await _mouse(MOUSE_BUTTON_LEFT, false, at)
	await _frames()

func _screen(tile: Vector2i) -> Vector2:
	return root.get_canvas_transform() * View.tile_centre(tile)

func _tick() -> int:
	var time: Dictionary = _world.call("get_time_snapshot")
	return int(time.get("tick", -1))

func _shot(name: String) -> Image:
	await _frames()
	RenderingServer.force_draw(false)
	var picture: Image = root.get_texture().get_image()
	var ok: bool = picture != null and not picture.is_empty()
	if ok:
		ok = picture.save_png(_out.path_join(name + ".png")) == OK
	_check("screenshot " + name, ok)
	return picture

func _until(seconds: float) -> void:
	while float(Time.get_ticks_msec() - _started) / 1000.0 < seconds:
		await process_frame

func _run() -> void:
	if _out.is_empty() or not _out.is_absolute_path():
		push_error("--out absolute directory is required")
		quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	if DisplayServer.get_name() == "headless":
		_check("windowed display", false, "headless cannot verify pixels")
		_finish()
		return
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	await _frames(20)
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D") as Camera2D
	_agents = _main.get_node("ColonyView/AgentLayer")
	_check("default zoom 1x", is_equal_approx(_camera.zoom.x, 1.0), _camera.zoom)
	await _until(10.0)
	await _shot("launch-10s")
	await _until(30.0)
	await _shot("launch-30s")
	await _key(KEY_SPACE)
	var paused_tick: int = _tick()
	await create_timer(0.4).timeout
	_check("Space pauses simulation", bool(_camera.call("is_paused")) and _tick() == paused_tick, [_tick(), paused_tick])
	for code: Key in [KEY_1, KEY_2, KEY_3, KEY_4]:
		await _key(code)
		_check("speed " + str(code - KEY_1 + 1), int(_camera.call("get_speed")) == code - KEY_1 + 1, _camera.call("get_speed"))
	await _key(KEY_1)
	for row: Array in [[KEY_MINUS, 0.5, "½×"], [KEY_MINUS, 0.25, "¼×"], [KEY_EQUAL, 0.5, "½×"], [KEY_EQUAL, 1.0, "1×"], [KEY_EQUAL, 2.0, "2×"]]:
		await _key(int(row[0]) as Key)
		await _zoom_check(float(row[1]), str(row[2]), "keys")
	for row: Array in [[MOUSE_BUTTON_WHEEL_DOWN, 1.0, "1×"], [MOUSE_BUTTON_WHEEL_DOWN, 0.5, "½×"], [MOUSE_BUTTON_WHEEL_DOWN, 0.25, "¼×"], [MOUSE_BUTTON_WHEEL_UP, 0.5, "½×"], [MOUSE_BUTTON_WHEEL_UP, 1.0, "1×"], [MOUSE_BUTTON_WHEEL_UP, 2.0, "2×"]]:
		await _mouse(int(row[0]) as MouseButton, true, Vector2(960, 540))
		await _mouse(int(row[0]) as MouseButton, false, Vector2(960, 540))
		await _frames()
		await _zoom_check(float(row[1]), str(row[2]), "wheel")
	await _key(KEY_MINUS)
	for row: Array in [[KEY_C, "UI/ChroniclePanel"], [KEY_T, "UI/TechPanel"], [KEY_N, "UI/NeighboursPanel"], [KEY_P, "UI/PriorityPanel"], [KEY_R, "ColonyView/OverlayLayer/Rooms"]]:
		var panel: CanvasItem = _main.get_node(str(row[1])) as CanvasItem
		var before: bool = panel.visible
		await _key(int(row[0]) as Key)
		_check("toggle " + str(row[1]), panel.visible != before)
		await _shot(str(row[1]).get_file())
		await _key(int(row[0]) as Key)
		_check("close " + str(row[1]), panel.visible == before)
	await _person()
	await _save_load()
	await _make_wall()
	await _reload()
	_finish()

func _zoom_check(expected: float, word: String, input_kind: String) -> void:
	var hud: Node = _main.get_node("UI/HudTopbar")
	var text: String = str(hud.call("zoom_text"))
	_check(input_kind + " zoom " + word, is_equal_approx(_camera.zoom.x, expected) and text.contains(word), text)
	_check(input_kind + " LOD " + word, bool(_agents.call("lod_active")) == (expected < 1.0))
	await _shot(input_kind + "-zoom-" + str(expected))

func _person() -> void:
	var snap: Dictionary = _world.call("get_agent_snapshot")
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var inspector: Control = _main.get_node("UI/AgentInspectorPanel") as Control
	var picked: int = -1
	for id: int in bits:
		var rect: Rect2 = _agents.call("drawn_rect", id)
		var at: Vector2 = root.get_canvas_transform() * rect.get_center()
		if Rect2(120, 170, 1400, 570).has_point(at):
			await _click(at)
			if inspector.visible and int(inspector.call("get_selected_entity_bits")) >= 0:
				picked = int(inspector.call("get_selected_entity_bits"))
				break
	var portrait: Dictionary = inspector.call("portrait_state")
	_check("person click opens portrait", picked >= 0 and str(portrait.get("path", "")) != "" and portrait.get("size") == Vector2(128, 128), portrait)
	await _shot("person-portrait")
	if picked >= 0:
		await _key(KEY_D)
		var marks: Dictionary = _agents.call("person_marks", picked)
		_check("D drafts selected person", bool(marks.get("drafted", false)), marks)
		await _shot("person-drafted")
		await _key(KEY_D)
		marks = _agents.call("person_marks", picked)
		_check("D releases selected person", not bool(marks.get("drafted", true)), marks)
	await _key(KEY_ESCAPE)

func _save_load() -> void:
	var info: Dictionary = _world.call("get_world_info")
	var saved_tick: int = _tick()
	_save_path = "user://saves/%d-%d.wsave" % [int(info.get("seed", 0)), saved_tick]
	_save_existed = FileAccess.file_exists(_save_path)
	if _save_existed:
		_save_original = FileAccess.get_file_as_bytes(_save_path)
		if not _write_bytes(_out.path_join("save-original.wsave"), _save_original):
			_check("save backup", false)
			return
	_write_bytes(_out.path_join("save-journal.json"), JSON.stringify({"path": ProjectSettings.globalize_path(_save_path), "existed": _save_existed}).to_utf8_buffer())
	_save_touched = true
	await _key(KEY_F5)
	var bar: Node = _main.get_node("UI/OrderBar")
	var recorded: String = str(bar.get("_last_save"))
	_check("F5 writes save", recorded == _save_path and FileAccess.file_exists(_save_path), recorded)
	if recorded != _save_path:
		return
	await _key(KEY_SPACE)
	await create_timer(0.5).timeout
	await _key(KEY_SPACE)
	_check("clock advanced before F9", _tick() > saved_tick, [_tick(), saved_tick])
	await _key(KEY_F9)
	_check("F9 restores saved tick", _tick() == saved_tick, [_tick(), saved_tick])
	await _shot("save-load")
	_cleanup()

func _make_wall() -> void:
	await _key(KEY_H)
	var hearth: Vector2i = Vector2i(_camera.position / float(View.TILE))
	await _key(KEY_MINUS)
	await _key(KEY_MINUS)
	await _key(KEY_F1)
	await _move(_screen(hearth - Vector2i(18, 18)))
	await _mouse(MOUSE_BUTTON_LEFT, true, _screen(hearth - Vector2i(18, 18)))
	await _move(_screen(hearth + Vector2i(18, 18)))
	await _mouse(MOUSE_BUTTON_LEFT, false, _screen(hearth + Vector2i(18, 18)))
	var tool: Node = _main.get_node("ColonyView/OrderTool")
	var chop: Dictionary = tool.call("last_result")
	_check("chop supply accepted", bool(chop.get("accepted", false)) and int(chop.get("count", 0)) > 0, chop)
	await _shot("chop-supply")
	await _key(KEY_ESCAPE)
	await _key(KEY_F2)
	await _move(_screen(hearth + Vector2i(-4, 1)))
	await _mouse(MOUSE_BUTTON_LEFT, true, _screen(hearth + Vector2i(-4, 1)))
	await _move(_screen(hearth + Vector2i(-2, 3)))
	await _mouse(MOUSE_BUTTON_LEFT, false, _screen(hearth + Vector2i(-2, 3)))
	var zone: Dictionary = tool.call("last_result")
	_check("stockpile accepted", bool(zone.get("accepted", false)) and int(zone.get("count", 0)) > 0, zone)
	await _key(KEY_ESCAPE)
	for dy: int in range(-4, 5):
		for dx: int in range(-4, 5):
			var tile: Vector2i = hearth + Vector2i(dx, dy)
			if _wall_tile.x < 0 and bool(_world.call("can_place_blueprint", "log_wall", tile.x, tile.y)):
				_wall_tile = tile
	if _wall_tile.x < 0:
		_check("wall site available", false)
		return
	await _key(KEY_F3)
	await _key(KEY_5)
	await _click(_screen(_wall_tile))
	var placement: Dictionary = tool.call("last_result")
	_check("wall blueprint accepted", bool(placement.get("accepted", false)) and int(placement.get("count", 0)) == 1, placement)
	await _shot("wall-blueprint")
	await _key(KEY_ESCAPE)
	await _key(KEY_4)
	await _key(KEY_SPACE)
	var deadline: int = Time.get_ticks_msec() + 60000
	while Time.get_ticks_msec() < deadline:
		var space: Dictionary = _world.call("get_space")
		if (space.get("wall_xs", PackedInt32Array()) as PackedInt32Array).size() > 0:
			break
		await process_frame
	await _key(KEY_SPACE)
	await _key(KEY_EQUAL)
	await _key(KEY_EQUAL)
	var space: Dictionary = _world.call("get_space")
	_check("constructed wall for F12", (space.get("wall_xs", PackedInt32Array()) as PackedInt32Array).size() > 0, space.get("wall_xs"))
	_wall_before = await _shot("wall-before")

func _reload() -> void:
	_params_original = FileAccess.get_file_as_bytes(PARAMS)
	if not _write_bytes(_out.path_join("render_params.original.json"), _params_original):
		_check("F12 backup", false)
		return
	var parsed: Variant = JSON.parse_string(_params_original.get_string_from_utf8())
	if not parsed is Dictionary:
		_check("F12 input JSON", false)
		return
	var changed: Dictionary = parsed
	var walls: Dictionary = changed.get("walls", {})
	var original_value: float = float(walls.get("top_value", 0.0))
	walls["top_value"] = 0.15 if original_value > 0.4 else 0.9
	changed["walls"] = walls
	var before_version: int = ArtData.params_version()
	_params_changed = true
	if not _write_bytes(PARAMS, JSON.stringify(changed, "  ").to_utf8_buffer()):
		_check("F12 file write", false)
		_cleanup()
		return
	await _key(KEY_F12)
	var observed: float = float(ArtData.param("walls.top_value", -1.0))
	var toast: String = str(_camera.call("toast_text"))
	_check("F12 reload value and toast", ArtData.params_version() > before_version and is_equal_approx(observed, float(walls["top_value"])) and toast == "렌더 설정 다시 읽음", {"value": observed, "toast": toast})
	var after: Image = await _shot("wall-after-f12")
	if _wall_before != null and after != null and _wall_tile.x >= 0:
		var at: Vector2 = _screen(_wall_tile)
		var crop: Rect2i = Rect2i(Vector2i(at) - Vector2i(32, 64), Vector2i(64, 96)).intersection(Rect2i(Vector2i.ZERO, after.get_size()))
		var before_crop: Image = _wall_before.get_region(crop)
		var after_crop: Image = after.get_region(crop)
		_check("F12 wall pixels changed", before_crop.get_data() != after_crop.get_data(), crop)
		before_crop.save_png(_out.path_join("wall-crop-before.png"))
		after_crop.save_png(_out.path_join("wall-crop-after.png"))
	_cleanup()
	await _key(KEY_F12)
	_check("F12 restores value", is_equal_approx(float(ArtData.param("walls.top_value", -1.0)), original_value))
	await _shot("wall-restored")
