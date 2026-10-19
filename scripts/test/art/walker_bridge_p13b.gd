extends SceneTree
## Actual renderer inputs, actual bridge, both oracle modes; no fixture rewriting.
const Seats = preload("res://scripts/ui/camp_seats.gd")
const Adapter = preload("res://scripts/ui/art_walker_p13b.gd")
const Matrix = preload("res://scripts/test/art/perf_frames_p13.gd")
const Reads = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
const REQUIRED_CALLS: int = 12
var _main: Node
var _world: Node
var _renderer: Node
var _sink := BridgeSink.new()
var _output: String
var _sources: Dictionary = {}
var _head: String

class BridgeSink extends RefCounted:
	var world: Node
	var active: bool = false
	var calls: int = 0
	var comparisons: int = 0
	var nonempty: int = 0
	var last_frame: int = -1
	var observations: Array[Dictionary] = []
	var errors: Array[String] = []

	func record(rows: Array[Dictionary], positions: Dictionary, rects: Dictionary, terrain: Dictionary, blocked: Dictionary, canvas: Transform2D) -> void:
		if not active or calls >= REQUIRED_CALLS or not errors.is_empty():
			return
		var frame: int = Engine.get_process_frames()
		var population: int = Reads.read(world, "get_agent_snapshot", true)["agent_ids"].size()
		if frame == last_frame or rows.size() < 200 or population < 200:
			errors.append("Requires unique renderer frames and >=200 actual rows/population")
			return
		last_frame = frame
		var before: Array = [rows.duplicate(true), positions.duplicate(true), rects.duplicate(true), terrain.duplicate(true), blocked.duplicate(true), canvas]
		var modes: Array[Dictionary] = []
		for spatial: bool in [false, true]:
			var expected: Dictionary = Seats.walker_offsets(rows, positions, rects, terrain, blocked, canvas, spatial)
			var actual: Dictionary = Adapter.walker_offsets(world, rows, positions, rects, terrain, blocked, canvas, spatial)
			var equal: bool = Adapter.bit_equal(expected, actual)
			modes.append({"spatial": spatial, "ordered_ids": expected.keys(), "outputs": expected.size(), "bit_equal": equal})
			comparisons += 1
			if not expected.is_empty():
				nonempty += 1
			if not equal:
				errors.append("Ordered IDs or f32 output bits differ at frame%d spatial%s" % [frame, spatial])
		if before != [rows, positions, rects, terrain, blocked, canvas]:
			errors.append("Oracle or adapter mutated live renderer inputs")
		observations.append({"frame": frame, "tick": Reads.read(world, "get_time_snapshot", true)["tick"], "population": population, "rows": rows.size(), "modes": modes})
		calls += 1

func _initialize() -> void:
	_run.call_deferred()

func _run() -> void:
	_output = OS.get_environment("P13B_WALKER_BRIDGE_OUT")
	if _output.is_empty() or FileAccess.file_exists(_output) or OS.has_feature("double") or DisplayServer.get_name() == "headless" or OS.get_cmdline_args().has("--fixed-fps"):
		push_error("Requires fresh JSON output, single-precision windowed Godot without fixed-fps")
		quit(2)
		return
	if DirAccess.dir_exists_absolute("/Users/rexxa/work/.mac_quiet.lock") or OS.get_environment("P13B_WALKER_KERNEL") == "1":
		push_error("Requires no measurement quiet hold and original renderer walker path")
		quit(2)
		return
	_sources = _hashes()
	_head = _git_head()
	var scene: PackedScene = load("res://scenes/main.tscn") as PackedScene
	_main = scene.instantiate()
	_world = _main.get_node("WorldSim")
	_renderer = _main.get_node("AgentRenderer")
	_world.call("set_sim_speed", 0.0)
	root.size = Vector2i(1920, 1080)
	root.add_child(_main)
	current_scene = _main
	if not _world.has_method("art_walker_offsets"):
		_sink.errors.append("Loaded actual bridge lacks art_walker_offsets")
		_finish()
		return
	var arity: int = -1
	for method: Dictionary in _world.get_method_list():
		if str(method["name"]) == "art_walker_offsets":
			arity = method["args"].size()
	if arity != 14:
		_sink.errors.append("Loaded wrapper does not expose exactly14 arguments")
		_finish()
		return
	var camera: Camera2D = _main.get_node("Camera2D") as Camera2D
	camera.call("set_auto_track", false)
	camera.position_smoothing_enabled = false
	camera.process_mode = Node.PROCESS_MODE_DISABLED
	var spec: Array = Matrix.WORLDS["W200"]
	if not bool(Reads.mutate(_world, "reset_world_stress", [spec[0], spec[2]])) or int(Reads.mutate(_world, "advance_ticks", [spec[1]])) != int(spec[1]):
		_sink.errors.append("W200 reset/advance failed")
		_finish()
		return
	var settlements: Dictionary = Reads.read(_world, "get_settlement_snapshot", true)
	var ids: PackedInt32Array = settlements["settlement_ids"]
	if ids.is_empty():
		_sink.errors.append("Actual W200 has no village")
		_finish()
		return
	var first: int = 0
	for index: int in ids.size():
		if ids[index] < ids[first]:
			first = index
	camera.position = Vector2(456, 36) + Vector2(settlements["formation_xs"][first], settlements["formation_ys"][first]) * 16.0
	camera.zoom = Vector2(4, 4)
	camera.force_update_scroll()
	Engine.max_fps = 60
	for warmup: int in 30:
		await process_frame
	_world.call("set_sim_speed", 2.0)
	for running_warmup: int in 10:
		await process_frame
	_sink.world = _world
	_renderer.set("_walker_fixture_sink_p13b", _sink)
	_sink.active = true
	for frame: int in 240:
		await process_frame
		if _sink.calls >= REQUIRED_CALLS or not _sink.errors.is_empty():
			break
	_world.call("set_sim_speed", 0.0)
	_sink.active = false
	_renderer.set("_walker_fixture_sink_p13b", null)
	if _sink.calls != REQUIRED_CALLS or _sink.comparisons != REQUIRED_CALLS * 2 or _sink.nonempty != REQUIRED_CALLS * 2:
		_sink.errors.append("Incomplete or empty real-call differential coverage")
	if _sink.observations.is_empty() or int(_sink.observations.back()["tick"]) <= int(spec[1]):
		_sink.errors.append("Running W200 simulation did not advance")
	_finish()

func _hashes() -> Dictionary:
	var result: Dictionary = {}
	for path: String in ["scripts/test/art/walker_bridge_p13b.gd", "scripts/ui/camp_seats.gd", "scripts/ui/art_walker_p13b.gd", "scripts/ui/art_declump_p12.gd", "scripts/ui/agent_renderer.gd", "scripts/test/art/perf_frames_p13.gd", "rust/target/debug/libsim_bridge.dylib", "rust/crates/sim-bridge/src/ffi/world_node.rs", "rust/crates/sim-bridge/src/ffi/art_kernels.rs"]:
		result[path] = FileAccess.get_sha256("res://" + path)
	return result

func _git_head() -> String:
	var lines: Array = []
	var code: int = OS.execute("git", ["-C", ProjectSettings.globalize_path("res://"), "rev-parse", "HEAD"], lines, true)
	return str(lines[0]).strip_edges() if code == 0 and not lines.is_empty() else ""

func _finish() -> void:
	_world.call("set_sim_speed", 0.0)
	if _head.is_empty() or _git_head() != _head or _hashes() != _sources or _sources.values().has(""):
		_sink.errors.append("Missing or changed source/bridge/HEAD provenance")
	var file := FileAccess.open(_output, FileAccess.WRITE)
	if file == null:
		push_error("Cannot write bridge differential evidence")
		quit(2)
		return
	file.store_string(JSON.stringify({"valid": _sink.errors.is_empty(), "world": "W200", "head": _head, "source_sha256": _sources,
		"calls": _sink.calls, "comparisons": _sink.comparisons, "nonempty_comparisons": _sink.nonempty,
		"observations": _sink.observations, "errors": _sink.errors, "godot": Engine.get_version_info(),
		"loaded_bridge_sha256": _sources["rust/target/debug/libsim_bridge.dylib"], "argv": OS.get_cmdline_args()}, "\t") + "\n")
	file.close()
	print("P13B_WALKER_BRIDGE calls=%d comparisons=%d valid=%s errors=%s" % [_sink.calls, _sink.comparisons, _sink.errors.is_empty(), _sink.errors])
	quit(0 if _sink.errors.is_empty() else 1)
