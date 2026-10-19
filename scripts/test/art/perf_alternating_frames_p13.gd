extends SceneTree

const Session = preload("res://scripts/test/art/frame_measure_session_p13.gd")
const Reads = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const Matrix = preload("res://scripts/test/art/perf_frames_p13.gd")

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
var _speed: float = 0.0
var _spec: Array
var _setter: String = "set_perf_candidate_p12"
var _getter: String = "get_perf_candidate_p12"
var _kernel_expected: bool = false
var _last_tick: int

func _initialize() -> void:
	_run.call_deferred()

func _run() -> void:
	if OS.get_environment("P13_BUILD_PROFILE") != "release" or FileAccess.get_sha256("res://rust/target/debug/libsim_bridge.dylib").is_empty() or FileAccess.get_sha256("res://rust/target/debug/libsim_bridge.dylib") != FileAccess.get_sha256("res://rust/target/release/libsim_bridge.dylib"):
		push_error("P12 requires release bridge with identical debug/release dylib SHA256")
		quit(2)
		return
	_change = OS.get_environment("P13_PAIR_CHANGE")
	if _change == "write-skip":
		_setter = "set_perf_candidate_p13"
		_getter = "get_perf_candidate_p13"
		var kernel_state: String = OS.get_environment("P13_PAIR_KERNEL_STATE")
		if kernel_state not in ["on", "off"]:
			push_error("write-skip requires explicit P13_PAIR_KERNEL_STATE")
			quit(2)
			return
		_kernel_expected = kernel_state == "on"
		if (OS.get_environment("P12_DECLUMP_KERNEL") == "1") != _kernel_expected:
			push_error("write-skip kernel environment disagrees with required state")
			quit(2)
			return
	_world_id = OS.get_environment("P13_PAIR_WORLD")
	_speed = 2.0 if _world_id == "W200" else 0.0
	if _change.is_empty() or not Matrix.WORLDS.has(_world_id) or OS.get_environment("P13_PAIR_OUT").is_empty():
		push_error("Use run_alternating_p13.py with a named candidate and matrix world")
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
	if not _renderer.has_method(_setter) or not _renderer.has_method(_getter) or not _renderer.has_method("get_perf_candidate_p12"):
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
	_spec = spec
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
	Engine.max_fps = 60 if _speed > 0.0 else 0
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
	_world.call("set_sim_speed", 0.0)
	if _speed == 0.0 and final != _initial:
		_errors.append("Frozen world state changed during alternating measurement")
	var output: Dictionary = {"schema_version": 1, "build_profile": OS.get_environment("P13_BUILD_PROFILE"), "release_sha256": FileAccess.get_sha256("res://rust/target/release/libsim_bridge.dylib"), "valid": _errors.is_empty(), "errors": _errors, "world": _world_id, "change_id": _change, "build_commit": OS.get_environment("P13_PAIR_HEAD"), "build_sha256": FileAccess.get_sha256("res://rust/target/debug/libsim_bridge.dylib"), "initial": _initial, "final": final, "restoration": restored, "blocks": _blocks, "contended": OS.get_environment("P13_CONTENDED") == "1", "metric": "process_ms + render_cpu_ms", "simulation_speed": _speed}
	if _change == "write-skip":
		output["kernel_enabled"] = _kernel_expected
	var file := FileAccess.open(OS.get_environment("P13_PAIR_OUT"), FileAccess.WRITE)
	if file == null:
		push_error("Cannot write alternating evidence")
		quit(2)
		return
	file.store_string(JSON.stringify(output, "\t") + "\n")
	file.close()
	quit(0 if _errors.is_empty() else 1)

func _block(index: int) -> void:
	_world.call("set_sim_speed", 0.0)
	Reads.mutate(_world, "reset_world_stress" if int(_spec[2]) > 0 else "reset_world", [_spec[0], _spec[2]] if int(_spec[2]) > 0 else [_spec[0]])
	_world.call("set_sim_speed", 0.0)
	Reads.mutate(_world, "advance_ticks", [_spec[1]])
	await process_frame
	await process_frame
	var block_start: Dictionary = _runtime.snapshot()
	_last_tick = int(block_start["tick"])
	if block_start != _initial:
		_errors.append("Fresh alternating setup differs from initial state")
	_world.call("set_sim_speed", _speed)
	var enabled: bool = index % 2 == 1
	if _renderer.call(_setter, _change, enabled) != true:
		_errors.append("Renderer rejected named candidate switch")
		return
	var state: Dictionary = _renderer.call(_getter)
	if state.get("change_id") != _change or not (state.get("candidate_enabled") is bool) or state.get("candidate_enabled") != enabled:
		_errors.append("Renderer switch readback does not match requested candidate")
		return
	if _change == "write-skip" and not bool(_write_skip_evidence(enabled)["write_skip"]["valid"]):
		_errors.append("Write-skip initial readback differs from fixed kernel requirement")
		return
	var discarded: Array[Dictionary] = []
	var samples: Array[Dictionary] = []
	for frame: int in range(180):
		var sample: Dictionary = await _runtime.next_frame()
		if _change == "write-skip":
			var evidence: Dictionary = _write_skip_evidence(enabled)
			sample["perf_candidate_p13"] = evidence["write_skip"]
			sample["perf_candidate_p12"] = evidence["kernel"]
			if not bool(evidence["write_skip"]["valid"]):
				_errors.append("Write-skip or fixed kernel changed during frame")
		sample["ticks_run"] = int(sample["state"]["tick"]) - _last_tick
		_last_tick = int(sample["state"]["tick"])
		var inspector: CanvasItem = _main.get_node("UI/AgentInspectorPanel") as CanvasItem
		var overlay: Node = _main.get_node("WorldRenderer")
		sample["direct_read_context"] = {"inspector_visible": inspector.visible, "influence_channel": overlay.get("current_channel"), "layers_enabled": Selection.layers_enabled}
		if inspector.visible or int(overlay.get("current_channel")) != -1 or not Selection.layers_enabled:
			_errors.append("Inspector/overlay/layer controls changed from matrix defaults")
		if (_speed == 0.0 and sample.get("state") != _initial) or not sample.get("paired", false):
			_errors.append("Unpaired frame or changed frozen world state")
		for metric: String in Matrix.METRICS:
			if not is_finite(float(sample[metric])) or float(sample[metric]) < 0:
				_errors.append("Invalid frame metric: " + metric)
		if frame < 30:
			discarded.append(sample)
		else:
			samples.append(sample)
	var after: Dictionary = _renderer.call(_getter)
	if after != state:
		_errors.append("Renderer switch changed inside a measured block")
	var block_end: Dictionary = _runtime.snapshot()
	_world.call("set_sim_speed", 0.0)
	_blocks.append({"fresh_setup": true, "start_state": block_start, "end_state": block_end, "population_start": block_start["population"], "population_end": block_end["population"], "population_flagged": float(block_end["population"]) < 0.9 * float(block_start["population"]), "index": index, "mode": "candidate" if enabled else "control", "switch_state": state, "discarded": discarded, "samples": samples, "max_fps": Engine.max_fps, "vsync_disabled": DisplayServer.window_get_vsync_mode() == DisplayServer.VSYNC_DISABLED, "camera_process_frozen": _camera.process_mode == Node.PROCESS_MODE_DISABLED, "window_size": [root.size.x, root.size.y]})

func _write_skip_evidence(enabled: bool) -> Dictionary:
	var kernel_actual: Dictionary = _renderer.call("get_perf_candidate_p12")
	var actual: Dictionary = _renderer.call("get_perf_candidate_p13")
	var kernel_valid: bool = kernel_actual.get("change_id") == "declump-kernel" and kernel_actual.get("candidate_enabled") is bool and kernel_actual.get("candidate_enabled") == _kernel_expected
	var valid: bool = kernel_valid and actual.get("change_id") == "write-skip" and actual.get("candidate_enabled") is bool and actual.get("candidate_enabled") == enabled and actual.get("kernel_enabled") is bool and actual.get("kernel_enabled") == _kernel_expected
	return {"kernel": {"requested_kernel": _kernel_expected, "method_available": true, "actual": kernel_actual, "valid": kernel_valid}, "write_skip": {"requested_write_skip": enabled, "requested_kernel": _kernel_expected, "method_available": true, "actual": actual, "valid": valid}}
