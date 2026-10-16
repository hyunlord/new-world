extends SceneTree

const ReadRoute = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")
const Session = preload("res://scripts/test/art/frame_measure_session_p10.gd")
const Loader = preload("res://scripts/test/art/section_profile_loader_p10.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
const Clock = preload("res://scripts/test/art/agent_section_clock_p10.gd")
var _runtime: Session.Runtime
var _renderer: Node
var _errors: Array[String] = []
var _output: String
var _result: Dictionary = {}
var _last_frame: int = -1

func _initialize() -> void:
	_run.call_deferred()

func _run() -> void:
	_output = OS.get_environment("P10_SECTIONS_OUT")
	if _output.is_empty() or OS.get_cmdline_args().has("--fixed-fps") or DisplayServer.get_name() == "headless":
		push_error("sections-p10 requires output path and windowed debug Godot without fixed FPS")
		quit(2)
		return
	var packed: PackedScene = load("res://scenes/main.tscn") as PackedScene
	var main: Node = packed.instantiate()
	var installed: Dictionary = Loader.install(main, OS.get_environment("P10_SECTIONS_SCRIPT"), OS.get_environment("P10_SECTIONS_MANIFEST"))
	if not bool(installed.get("valid", false)):
		_errors.append(str(installed.get("error", "installation failed")))
		main.free()
		_finish()
		return
	var world: Node = main.get_node("WorldSim")
	var camera: Camera2D = main.get_node("Camera2D") as Camera2D
	_renderer = main.get_node("AgentRenderer")
	world.call("set_sim_speed", 0.0)
	root.size = Vector2i(1920, 1080)
	root.add_child(main)
	current_scene = main
	camera.call("set_auto_track", false)
	camera.position_smoothing_enabled = false
	var prior_camera_mode: int = camera.process_mode
	camera.process_mode = Node.PROCESS_MODE_DISABLED
	ReadRoute.mutate(world, "reset_world_stress", [15100, 500])
	world.call("set_sim_speed", 0.0)
	ReadRoute.mutate(world, "advance_ticks", [1440])
	var settlements: Dictionary = ReadRoute.read(world, "get_settlement_snapshot", true)
	var ids: PackedInt32Array = settlements.get("settlement_ids", PackedInt32Array())
	if ids.is_empty():
		_errors.append("W500 requires a settlement for the matrix camera framing")
		_finish()
		return
	var first: int = 0
	for index: int in ids.size():
		if ids[index] < ids[first]:
			first = index
	camera.position = Vector2(448, 28) + Vector2(settlements["formation_xs"][first], settlements["formation_ys"][first]) * 16.0 + Vector2(8, 8)
	camera.zoom = Vector2(4, 4)
	_runtime = Session.Runtime.new()
	_runtime.tree = self
	_runtime.world = world
	_runtime.camera = camera
	var prior_max_fps: int = Engine.max_fps
	Engine.max_fps = 0
	var initial: Dictionary = _runtime.begin()
	_runtime.prior_camera_mode = prior_camera_mode
	_result = {"schema_version": 1, "diagnostic_only": true, "instrumented": true,
		"world": "W500", "seed": 15100, "requested_tick": 1440, "requested_population": 500,
		"zoom": 4, "window_size": [1920, 1080], "framing_settlement_id": ids[first],
		"framing_tile": [settlements["formation_xs"][first], settlements["formation_ys"][first]],
		"conditions": _conditions(), "speed_setting": 0.0, "speed_evidence": "set_sim_speed(0.0); no speed getter; fixed tick across all samples",
		"warmup_frames": 30, "counted_frames": 300, "installation": installed,
		"build_commit": OS.get_environment("P10_SECTIONS_HEAD"), "build_sha256": OS.get_environment("P10_SECTIONS_BUILD_SHA256"),
		"initial": initial, "godot": Engine.get_version_info(), "build_profile": "debug",
		"metric": "instrumented AgentRenderer exclusive microseconds; not H1-H4 evidence",
		"process_priorities": [-1000, 1000], "forced_draw": true,
		"contended": OS.get_environment("P10_CONTENDED") == "1",
		"hold_owner": OS.get_environment("P10_HOLD_OWNER"), "hold_start": OS.get_environment("P10_HOLD_START"),
		"founding_groups_status": "unavailable_separate_callback"}
	if int(initial.get("tick", -1)) != 1440 or int(initial.get("population", -1)) != 500:
		_errors.append("world must have tick 1440 and population 500")
	var warmup: Array[Dictionary] = []
	var samples: Array[Dictionary] = []
	for index: int in 330:
		if not _errors.is_empty():
			break
		var sample: Dictionary = await _runtime.next_frame()
		var sections: Dictionary = Loader.collect(_renderer, int(sample.get("process_frame", -1)))
		sample["agent_sections"] = sections
		sample["conditions"] = _conditions()
		sample["sample_index"] = index - 30
		_validate(sample, initial)
		if index < 30:
			warmup.append(sample)
		else:
			samples.append(sample)
	_result["discarded"] = warmup
	_result["samples"] = samples
	_result["final"] = _runtime.snapshot()
	_result["restoration"] = _runtime.restore()
	Engine.max_fps = prior_max_fps
	_result["max_fps_restored"] = Engine.max_fps == prior_max_fps
	if samples.size() != 300 or warmup.size() != 30 or _result["final"] != initial:
		_errors.append("incomplete frame counts or changed frozen world")
	for key: String in ["vsync_restored", "layers_restored", "camera_process_restored", "probes_removed"]:
		if not bool(_result["restoration"].get(key, false)):
			_errors.append("restoration failed: " + key)
	_finish()

func _conditions() -> Dictionary:
	var main: Node = _renderer.get_parent()
	var inspector: CanvasItem = main.get_node("UI/AgentInspectorPanel") as CanvasItem
	var influence: Node = main.get_node("WorldRenderer")
	return {"inspector_visible": inspector.visible, "influence_channel": int(influence.get("current_channel")), "layers_enabled": Selection.layers_enabled, "renderer_callback_enabled": _renderer.can_process() and _renderer.is_processing(), "world_callback_enabled": _runtime.world.can_process() and _runtime.world.is_processing(), "speed_setting": 0.0}

func _validate(sample: Dictionary, initial: Dictionary) -> void:
	var expected_conditions: Dictionary = {"inspector_visible": false, "influence_channel": -1, "layers_enabled": true, "renderer_callback_enabled": true, "world_callback_enabled": true, "speed_setting": 0.0}
	if sample.get("conditions", {}) != expected_conditions or _result.get("conditions", {}) != expected_conditions:
		_errors.append("UI, layer or callback conditions differ from matrix")
	var frame: int = int(sample.get("process_frame", -1))
	if not bool(sample.get("paired", false)) or frame <= _last_frame or sample.get("state", {}) != initial:
		_errors.append("unpaired, stale or changed-state process sample")
	_last_frame = frame
	var sections: Dictionary = sample["agent_sections"]
	if not bool(sections.get("valid", false)) or not bool(sections.get("complete", false)) or int(sections.get("process_frame", -1)) != frame:
		_errors.append("invalid, incomplete or stale section sample")
	var exclusive: Dictionary = sections.get("exclusive_usec", {})
	var total: int = 0
	if exclusive.size() != Clock.NAMES.size():
		_errors.append("section key count mismatch")
	for key: String in Clock.NAMES:
		if not exclusive.has(key) or not is_finite(float(exclusive.get(key, -1))) or float(exclusive.get(key, -1)) < 0.0:
			_errors.append("missing or negative section: " + key)
		total += int(exclusive.get(key, 0))
	if total != int(sections.get("total_usec", -1)) or int(sections.get("remainder_usec", -1)) != 0:
		_errors.append("exclusive sum differs from callback total")
	for key: String in ["process_ms", "render_cpu_ms", "setup_cpu_ms", "primary_cpu_ms"]:
		if not is_finite(float(sample.get(key, -1))) or float(sample.get(key, -1)) < 0.0:
			_errors.append("invalid frame metric: " + key)

func _finish() -> void:
	_result["errors"] = _errors
	_result["valid"] = _errors.is_empty()
	var file := FileAccess.open(_output, FileAccess.WRITE)
	if file == null:
		push_error("sections-p10 cannot write output")
		quit(1)
		return
	file.store_string(JSON.stringify(_result, "\t", true, true) + "\n")
	file.close()
	quit(0 if _errors.is_empty() else 1)
