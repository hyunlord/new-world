extends SceneTree

const SimulationEngineScript = preload("res://scripts/core/simulation/simulation_engine.gd")
const SimBridgeScript = preload("res://scripts/core/simulation/sim_bridge.gd")
const SimulationBusScript = preload("res://scripts/core/simulation/simulation_bus.gd")
const SimulationBusV2Script = preload("res://scripts/core/simulation/simulation_bus_v2.gd")

const SHADOW_TEST_SEED_DEFAULT: int = 20260302
const SHADOW_TEST_FRAMES_DEFAULT: int = 800
const SHADOW_TEST_DELTA_SEC_DEFAULT: float = 0.1
const SHADOW_RUNTIME_MODE_DEFAULT: String = "rust_shadow"


func _init() -> void:
	call_deferred("_run_smoke")


func _run_smoke() -> void:
	var runtime_args: Dictionary = _parse_runtime_args()
	var seed: int = int(runtime_args.get("seed", SHADOW_TEST_SEED_DEFAULT))
	var frames: int = int(runtime_args.get("frames", SHADOW_TEST_FRAMES_DEFAULT))
	var delta_sec: float = float(runtime_args.get("delta", SHADOW_TEST_DELTA_SEC_DEFAULT))
	var runtime_mode: String = str(runtime_args.get("runtime_mode", SHADOW_RUNTIME_MODE_DEFAULT))
	var report_path: String = _resolve_report_path(str(runtime_args.get("report_path", "")))

	_ensure_autoload_node("SimBridge", SimBridgeScript)
	_ensure_autoload_node("SimulationBus", SimulationBusScript)
	_ensure_autoload_node("SimulationBusV2", SimulationBusV2Script)
	print("CLASS_EXISTS_WORLDSIMRUNTIME=" + str(ClassDB.class_exists("WorldSimRuntime")))
	var sim_bridge_node: Node = root.get_node_or_null("SimBridge")
	if sim_bridge_node != null:
		print("SIMBRIDGE_HAS_RUNTIME_INIT=" + str(sim_bridge_node.has_method("runtime_init")))
		print("SIMBRIDGE_RUNTIME_INIT_PROBE=" + str(sim_bridge_node.call("runtime_init", seed, "{}")))
		print("SIMBRIDGE_RUNTIME_IS_INITIALIZED_PROBE=" + str(sim_bridge_node.call("runtime_is_initialized")))

	print(
		"SHADOW_SMOKE_CONFIG="
		+ JSON.stringify({
			"seed": seed,
			"frames": frames,
			"delta_sec": delta_sec,
			"runtime_mode": runtime_mode,
			"report_path": report_path,
		})
	)
	var sim_engine: RefCounted = SimulationEngineScript.new()
	if sim_engine.has_method("set_runtime_mode_override"):
		sim_engine.call("set_runtime_mode_override", runtime_mode)
	sim_engine.init_with_seed(seed)

	for frame_idx: int in range(frames):
		sim_engine.update(delta_sec)

	var reporter_raw: Variant = sim_engine.get("_shadow_reporter")
	if reporter_raw != null and reporter_raw is RefCounted:
		var reporter: RefCounted = reporter_raw
		if reporter.has_method("flush_now"):
			reporter.call("flush_now", int(sim_engine.current_tick))

	print("SHADOW_REPORT_PATH=" + report_path)

	if not FileAccess.file_exists(report_path):
		push_error("[rust_shadow_smoke] report not generated: %s" % report_path)
		quit(2)
		return

	quit(0)


func _parse_runtime_args() -> Dictionary:
	var parsed: Dictionary = {
		"seed": SHADOW_TEST_SEED_DEFAULT,
		"frames": SHADOW_TEST_FRAMES_DEFAULT,
		"delta": SHADOW_TEST_DELTA_SEC_DEFAULT,
		"runtime_mode": SHADOW_RUNTIME_MODE_DEFAULT,
		"report_path": "",
	}
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i in range(args.size()):
		var arg: String = str(args[i]).strip_edges()
		if arg.begins_with("--seed="):
			var raw_seed: String = arg.substr(7)
			if raw_seed.is_valid_int():
				parsed["seed"] = int(raw_seed)
		elif arg.begins_with("--frames="):
			var raw_frames: String = arg.substr(9)
			if raw_frames.is_valid_int():
				parsed["frames"] = maxi(1, int(raw_frames))
		elif arg.begins_with("--delta="):
			var raw_delta: String = arg.substr(8)
			if raw_delta.is_valid_float():
				parsed["delta"] = maxf(0.0001, float(raw_delta))
		elif arg.begins_with("--runtime-mode="):
			var mode: String = arg.substr(15).strip_edges()
			if _is_supported_runtime_mode(mode):
				parsed["runtime_mode"] = mode
		elif arg.begins_with("--report-path="):
			parsed["report_path"] = arg.substr(14).strip_edges()
	return parsed


func _is_supported_runtime_mode(mode: String) -> bool:
	return (
		mode == "gdscript"
		or mode == "rust_shadow"
		or mode == "rust_primary"
	)


func _resolve_report_path(raw_path: String) -> String:
	if raw_path.is_empty():
		return ProjectSettings.globalize_path(GameConfig.RUST_SHADOW_REPORT_PATH)
	if raw_path.begins_with("user://") or raw_path.begins_with("res://"):
		return ProjectSettings.globalize_path(raw_path)
	return raw_path


func _ensure_autoload_node(node_name: String, script_ref: Script) -> void:
	if root.get_node_or_null(node_name) != null:
		return
	var node: Node = script_ref.new()
	node.name = node_name
	root.add_child(node)
