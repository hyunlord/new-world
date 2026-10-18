extends SceneTree

const Recorder = preload("res://scripts/test/art/declump_fixture_recorder_p12.gd")
const Matrix = preload("res://scripts/test/art/perf_frames_p12.gd")
const Reads = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
var _main: Node
var _world: Node
var _camera: Camera2D
var _renderer: Node
var _recorder = Recorder.new()

func _initialize() -> void:
	_run.call_deferred()

func _run() -> void:
	var output: String = OS.get_environment("P12_DECLUMP_FIXTURE_OUT")
	if output.is_empty() or OS.get_environment("P12_DECLUMP_KERNEL") == "1" or OS.has_feature("double"):
		push_error("Fixture capture requires fresh P12_DECLUMP_FIXTURE_OUT, original declump and single precision")
		quit(2)
		return
	if DirAccess.dir_exists_absolute("/Users/rexxa/work/.mac_quiet.lock"):
		push_error("Declump fixtures run outside measurement quiet holds")
		quit(2)
		return
	if not _recorder.begin(output):
		push_error(str(_recorder.errors))
		quit(2)
		return
	var scene: PackedScene = load("res://scenes/main.tscn") as PackedScene
	_main = scene.instantiate()
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D") as Camera2D
	_renderer = _main.get_node("AgentRenderer")
	_world.call("set_sim_speed", 0.0)
	root.size = Vector2i(1920, 1080)
	root.add_child(_main)
	current_scene = _main
	_camera.call("set_auto_track", false)
	_camera.position_smoothing_enabled = false
	_camera.process_mode = Node.PROCESS_MODE_DISABLED
	_recorder.world = _world
	_renderer.set("_declump_fixture_sink_p12", _recorder)
	Engine.max_fps = 60
	for label: String in Recorder.WORLDS:
		for running: bool in [false, true]:
			await _capture_block(label, running)
			if not _recorder.errors.is_empty():
				break
		if not _recorder.errors.is_empty():
			break
	_world.call("set_sim_speed", 0.0)
	_renderer.set("_declump_fixture_sink_p12", null)
	var valid: bool = _recorder.finish()
	print("P12_DECLUMP_FIXTURE valid=%s calls=%d records=%d errors=%s" % [valid, _recorder.calls, _recorder.records, _recorder.errors])
	quit(0 if valid else 1)

func _capture_block(label: String, running: bool) -> void:
	_recorder.active = false
	_world.call("set_sim_speed", 0.0)
	var spec: Array = Matrix.WORLDS[label]
	var reset_ok: bool = bool(Reads.mutate(_world, "reset_world_stress" if int(spec[2]) > 0 else "reset_world", [spec[0], spec[2]] if int(spec[2]) > 0 else [spec[0]]))
	_world.call("set_sim_speed", 0.0)
	if not reset_ok or int(Reads.mutate(_world, "advance_ticks", [spec[1]])) != int(spec[1]):
		_recorder.errors.append("Fixture world reset/advance failed: " + label)
		return
	var settlements: Dictionary = Reads.read(_world, "get_settlement_snapshot", true)
	var ids: PackedInt32Array = settlements.get("settlement_ids", PackedInt32Array())
	if ids.is_empty():
		_recorder.errors.append("Fixture world has no hearth: " + label)
		return
	var first: int = 0
	for index: int in ids.size():
		if ids[index] < ids[first]:
			first = index
	_camera.position = Vector2(448, 28) + Vector2(settlements["formation_xs"][first], settlements["formation_ys"][first]) * 16.0 + Vector2(8, 8)
	_camera.zoom = Vector2(4, 4)
	_camera.force_update_scroll()
	for warmup: int in 30:
		await process_frame
	var start_tick: int = int(Reads.read(_world, "get_time_snapshot", true)["tick"])
	if start_tick != int(spec[1]):
		_recorder.errors.append("Fixture fresh setup missed target tick: " + label)
		return
	_recorder.start_block(label, running, int(spec[0]))
	_world.call("set_sim_speed", 2.0 if running else 0.0)
	for frame: int in 600:
		await process_frame
		if _recorder.block_calls >= Recorder.REQUIRED_CALLS or not _recorder.errors.is_empty():
			break
	_world.call("set_sim_speed", 0.0)
	_recorder.active = false
	var end_tick: int = int(Reads.read(_world, "get_time_snapshot", true)["tick"])
	if _recorder.block_calls != Recorder.REQUIRED_CALLS or (running and end_tick <= start_tick) or (not running and end_tick != start_tick):
		_recorder.errors.append("Fixture incomplete calls or wrong tick progression: " + label)
	print("P12_DECLUMP_FIXTURE_BLOCK world=%s running=%s calls=%d ticks=%d..%d" % [label, running, _recorder.block_calls, start_tick, end_tick])
