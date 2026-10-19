extends SceneTree

const Session = preload("res://scripts/test/art/frame_measure_session_p11.gd")
const Reads = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const Matrix = preload("res://scripts/test/art/perf_frames_p11.gd")

var _main: Node
var _world: Node
var _camera: Camera2D
var _renderer: Node
var _runtime: Session.Runtime
var _errors: Array[String] = []
var _blocks: Array[Dictionary] = []
var _initial: Dictionary
var _change: String
var _world_id: String

func _initialize() -> void:
	_run.call_deferred()

func _run() -> void:
	if OS.get_environment("P11_BUILD_PROFILE") != "release" or FileAccess.get_sha256("res://rust/target/debug/libsim_bridge.dylib").is_empty() or FileAccess.get_sha256("res://rust/target/debug/libsim_bridge.dylib") != FileAccess.get_sha256("res://rust/target/release/libsim_bridge.dylib"):
		push_error("P11 requires release bridge with identical debug/release dylib SHA256")
		quit(2)
		return
	_change = OS.get_environment("P11_PAIR_CHANGE")
	_world_id = OS.get_environment("P11_PAIR_WORLD")
	if _change.is_empty() or not Matrix.WORLDS.has(_world_id) or OS.get_environment("P11_PAIR_OUT").is_empty():
		push_error("Use run_alternating_p11.py with a named candidate and matrix world")
		quit(2)
		return
	if DisplayServer.get_name() == "headless" or OS.get_cmdline_args().has("--fixed-fps"):
		push_error("Alternating performance requires windowed Godot without fixed FPS")
		quit(2)
		return
	var scene: PackedScene = load("res://scenes/main.tscn") as PackedScene
	_main = scene.instantiate()
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D") as Camera2D
	_renderer = _main.get_node("AgentRenderer")
	if not _renderer.has_method("set_perf_candidate_p11") or not _renderer.has_method("get_perf_candidate_p11"):
		push_error("Renderer lacks the temporary real control/candidate switch; no measurements collected")
		_main.free()
		quit(2)
		return
	_world.call("set_sim_speed", 0.0)
	root.size = Vector2i(1920, 1080)
	root.add_child(_main)
	current_scene = _main
	_camera.call("set_auto_track", false)
	_camera.position_smoothing_enabled = false
	var previous_camera_mode: int = _camera.process_mode
	_camera.process_mode = Node.PROCESS_MODE_DISABLED
	var spec: Array = Matrix.WORLDS[_world_id]
	Reads.mutate(_world, "reset_world_stress" if int(spec[2]) > 0 else "reset_world", [spec[0], spec[2]] if int(spec[2]) > 0 else [spec[0]])
	_world.call("set_sim_speed", 0.0)
	Reads.mutate(_world, "advance_ticks", [spec[1]])
	Selection.set_layers_enabled(self, true)
	var settlements: Dictionary = Reads.read(_world, "get_settlement_snapshot")
	var ids: PackedInt32Array = settlements.get("settlement_ids", PackedInt32Array())
	if ids.is_empty():
		push_error("Alternating hearth framing requires a settlement")
		quit(2)
		return
	var first: int = 0
	for index: int in ids.size():
		if ids[index] < ids[first]:
			first = index
	_camera.position = Vector2(448, 28) + Vector2(settlements["formation_xs"][first], settlements["formation_ys"][first]) * 16.0 + Vector2(8, 8)
	_camera.zoom = Vector2(4, 4)
	_camera.reset_smoothing()
	_camera.force_update_scroll()
	# Deferred world_reset replaces renderer helpers; settle before setting controls.
	await process_frame
	await process_frame
	var previous_fps: int = Engine.max_fps
	Engine.max_fps = 0
	_runtime = Session.Runtime.new()
	_runtime.tree = self
	_runtime.world = _world
	_runtime.camera = _camera
	_initial = _runtime.begin()
	_runtime.prior_camera_mode = previous_camera_mode
	if int(_initial["tick"]) != int(spec[1]):
		_errors.append("World did not reach the requested tick")
	for index: int in range(6):
		if not _errors.is_empty():
			break
		await _block(index)
	var final: Dictionary = _runtime.snapshot()
	var restored: Dictionary = _runtime.restore()
	Engine.max_fps = previous_fps
	for key: String in ["vsync_restored", "layers_restored", "camera_process_restored", "probes_removed"]:
		if restored.get(key) != true:
			_errors.append("Runtime restoration failed: " + key)
	if final != _initial:
		_errors.append("Frozen world state changed during alternating measurement")
	var output: Dictionary = {"schema_version": 1, "build_profile": OS.get_environment("P11_BUILD_PROFILE"), "release_sha256": FileAccess.get_sha256("res://rust/target/release/libsim_bridge.dylib"), "valid": _errors.is_empty(), "errors": _errors, "world": _world_id, "change_id": _change, "build_commit": OS.get_environment("P11_PAIR_HEAD"), "build_sha256": FileAccess.get_sha256("res://rust/target/debug/libsim_bridge.dylib"), "initial": _initial, "final": final, "restoration": restored, "blocks": _blocks, "contended": OS.get_environment("P11_CONTENDED") == "1", "metric": "process_ms + render_cpu_ms", "simulation_speed": 0.0}
	var file := FileAccess.open(OS.get_environment("P11_PAIR_OUT"), FileAccess.WRITE)
	if file == null:
		push_error("Cannot write alternating evidence")
		quit(2)
		return
	file.store_string(JSON.stringify(output, "\t") + "\n")
	file.close()
	quit(0 if _errors.is_empty() else 1)

func _block(index: int) -> void:
	var enabled: bool = index % 2 == 1
	if _renderer.call("set_perf_candidate_p11", _change, enabled) != true:
		_errors.append("Renderer rejected named candidate switch")
		return
	var state: Dictionary = _renderer.call("get_perf_candidate_p11")
	if state.get("change_id") != _change or state.get("candidate_enabled") != enabled:
		_errors.append("Renderer switch readback does not match requested candidate")
		return
	var discarded: Array[Dictionary] = []
	var samples: Array[Dictionary] = []
	for frame: int in range(180):
		var sample: Dictionary = await _runtime.next_frame()
		sample["ticks_run"] = int(sample["state"]["tick"]) - int(_initial["tick"])
		var inspector: CanvasItem = _main.get_node("UI/AgentInspectorPanel") as CanvasItem
		var overlay: Node = _main.get_node("WorldRenderer")
		sample["direct_read_context"] = {"inspector_visible": inspector.visible, "influence_channel": overlay.get("current_channel"), "layers_enabled": Selection.layers_enabled}
		if inspector.visible or int(overlay.get("current_channel")) != -1 or not Selection.layers_enabled:
			_errors.append("Inspector/overlay/layer controls changed from matrix defaults")
		if sample.get("state") != _initial or not sample.get("paired", false):
			_errors.append("Unpaired frame or changed frozen world state")
		for metric: String in Matrix.METRICS:
			if not is_finite(float(sample[metric])) or float(sample[metric]) < 0:
				_errors.append("Invalid frame metric: " + metric)
		if frame < 30:
			discarded.append(sample)
		else:
			samples.append(sample)
	var after: Dictionary = _renderer.call("get_perf_candidate_p11")
	if after != state:
		_errors.append("Renderer switch changed inside a measured block")
	_blocks.append({"index": index, "mode": "candidate" if enabled else "control", "switch_state": state, "discarded": discarded, "samples": samples, "max_fps": Engine.max_fps, "vsync_disabled": DisplayServer.window_get_vsync_mode() == DisplayServer.VSYNC_DISABLED, "camera_process_frozen": _camera.process_mode == Node.PROCESS_MODE_DISABLED, "window_size": [root.size.x, root.size.y]})
