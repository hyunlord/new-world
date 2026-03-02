extends SceneTree

const SimulationEngineScript = preload("res://scripts/core/simulation/simulation_engine.gd")
const SimBridgeScript = preload("res://scripts/core/simulation/sim_bridge.gd")
const SimulationBusScript = preload("res://scripts/core/simulation/simulation_bus.gd")
const SimulationBusV2Script = preload("res://scripts/core/simulation/simulation_bus_v2.gd")

const SHADOW_TEST_SEED: int = 20260302
const SHADOW_TEST_FRAMES: int = 800
const SHADOW_TEST_DELTA_SEC: float = 0.1


func _init() -> void:
	call_deferred("_run_smoke")


func _run_smoke() -> void:
	_ensure_autoload_node("SimBridge", SimBridgeScript)
	_ensure_autoload_node("SimulationBus", SimulationBusScript)
	_ensure_autoload_node("SimulationBusV2", SimulationBusV2Script)
	print("CLASS_EXISTS_WORLDSIMRUNTIME=" + str(ClassDB.class_exists("WorldSimRuntime")))
	var sim_bridge_node: Node = root.get_node_or_null("SimBridge")
	if sim_bridge_node != null:
		print("SIMBRIDGE_HAS_RUNTIME_INIT=" + str(sim_bridge_node.has_method("runtime_init")))
		print("SIMBRIDGE_RUNTIME_INIT_PROBE=" + str(sim_bridge_node.call("runtime_init", SHADOW_TEST_SEED, "{}")))
		print("SIMBRIDGE_RUNTIME_IS_INITIALIZED_PROBE=" + str(sim_bridge_node.call("runtime_is_initialized")))

	var sim_engine: RefCounted = SimulationEngineScript.new()
	sim_engine.init_with_seed(SHADOW_TEST_SEED)

	for frame_idx: int in range(SHADOW_TEST_FRAMES):
		sim_engine.update(SHADOW_TEST_DELTA_SEC)

	var reporter_raw: Variant = sim_engine.get("_shadow_reporter")
	if reporter_raw != null and reporter_raw is RefCounted:
		var reporter: RefCounted = reporter_raw
		if reporter.has_method("flush_now"):
			reporter.call("flush_now", int(sim_engine.current_tick))

	var report_path: String = ProjectSettings.globalize_path(GameConfig.RUST_SHADOW_REPORT_PATH)
	print("SHADOW_REPORT_PATH=" + report_path)

	if not FileAccess.file_exists(report_path):
		push_error("[rust_shadow_smoke] report not generated: %s" % report_path)
		quit(2)
		return

	quit(0)


func _ensure_autoload_node(node_name: String, script_ref: Script) -> void:
	if root.get_node_or_null(node_name) != null:
		return
	var node: Node = script_ref.new()
	node.name = node_name
	root.add_child(node)
