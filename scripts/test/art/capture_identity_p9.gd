extends SceneTree

# One framing per launch. Identical absolute frames keep accumulated visual clocks aligned.
signal frame_settled
const Selection = preload("res://scripts/core/settlement_selection.gd")
const CASES: Dictionary = {
	"P1": [20261010, 40560, 4, 0], "P2": [20261010, 31320, 4, 0],
	"P4": [20261013, 126994, 1, 0], "P5": [15100, 1440, 4, 500],
	"P6": [20261010, 40560, 4, 0],
}
var _main: Node
var _world: Node
var _reads: Node
var _camera: Camera2D

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	var framing: String = OS.get_environment("P9_IDENTITY_ID")
	var output: String = OS.get_environment("P9_IDENTITY_OUT")
	var args: PackedStringArray = OS.get_cmdline_args()
	var fps_index: int = args.find("--fixed-fps")
	if not CASES.has(framing) or output.is_empty() or fps_index < 0 or fps_index + 1 >= args.size() or args[fps_index + 1] != "60":
		_fail("requires P9_IDENTITY_ID=P1|P2|P4|P5|P6, P9_IDENTITY_OUT and --fixed-fps 60")
		return
	root.size = Vector2i(1920, 1080)
	var scene: PackedScene = load("res://scenes/main.tscn") as PackedScene
	_main = scene.instantiate()
	_world = _main.get_node("WorldSim")
	_camera = _main.get_node("Camera2D") as Camera2D
	_world.call("set_sim_speed", 0.0)
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(_main)
	current_scene = _main
	_reads = _main.get_node_or_null("SimReads")
	_camera.call("set_auto_track", false)
	_camera.position_smoothing_enabled = false
	_camera.process_mode = Node.PROCESS_MODE_DISABLED
	await _until_frame(10)
	if Engine.get_process_frames() != 10:
		_fail("startup exceeded absolute state-change frame 10")
		return
	var spec: Array = CASES[framing]
	if int(spec[3]) > 0:
		_mutate("reset_world_stress", [int(spec[0]), int(spec[3])])
	else:
		_mutate("reset_world", [int(spec[0])])
	_mutate("advance_ticks", [int(spec[1])])
	_world.call("set_sim_speed", 0.0)
	Selection.set_layers_enabled(self, true)
	var settlements: Dictionary = _world.call("get_settlement_snapshot")
	var ids: PackedInt32Array = settlements["settlement_ids"]
	if ids.is_empty():
		_fail("framing requires first settlement")
		return
	var first: int = 0
	for index: int in ids.size():
		if ids[index] < ids[first]:
			first = index
	var tile := Vector2(settlements["formation_xs"][first], settlements["formation_ys"][first])
	_camera.position = Vector2(448, 28) + tile * 16.0 + Vector2(8, 8)
	_camera.zoom = Vector2.ONE * int(spec[2])
	_camera.reset_smoothing()
	_camera.force_update_scroll()
	if framing == "P6":
		var agents: Dictionary = _world.call("get_agent_snapshot")
		var agent_ids: PackedInt64Array = agents["agent_ids"]
		if agent_ids.is_empty():
			_fail("P6 requires a living agent")
			return
		var lowest: int = 0
		for index: int in agent_ids.size():
			if agent_ids[index] < agent_ids[lowest]:
				lowest = index
		var detail: Dictionary = _world.call("get_agent_detail", agents["ids"][lowest])
		detail["entity_bits"] = agents["ids"][lowest]
		_main.get_node("UI/AgentInspectorPanel").call("display_agent", detail)
		_main.get_node("UI/GodHandPanel").call("show")
	for name: String in ["TerrainRenderer", "BuildingRenderer", "AgentRenderer"]:
		var renderer: Node = _main.get_node(name)
		if renderer.has_method("refresh_snapshot"):
			renderer.call("refresh_snapshot")
	var state_frame: int = Engine.get_process_frames()
	await _until_frame(130)
	RenderingServer.force_draw(false)
	var image: Image = root.get_texture().get_image()
	var time: Dictionary = _world.call("get_time_snapshot")
	if Engine.get_process_frames() - state_frame != 120 or int(time["tick"]) != int(spec[1]) or image.get_size() != Vector2i(1920, 1080):
		_fail("capture frame, tick, or framebuffer invariant failed")
		return
	if DirAccess.make_dir_recursive_absolute(output) != OK or image.save_png(output.path_join(framing + ".png")) != OK:
		_fail("cannot save capture")
		return
	var manifest: Dictionary = {"framing": framing, "seed": spec[0], "tick": time["tick"], "state_change_frame": state_frame, "capture_frame": Engine.get_process_frames(), "settle_frames": 120, "fixed_fps": 60, "camera": [_camera.position.x, _camera.position.y], "zoom": spec[2], "debug_build": OS.is_debug_build(), "sim_reads": _reads != null}
	var file := FileAccess.open(output.path_join(framing + ".json"), FileAccess.WRITE)
	if file == null:
		_fail("cannot save manifest")
		return
	file.store_string(JSON.stringify(manifest, "\t") + "\n")
	quit(0)

func _mutate(method: String, arguments: Array) -> void:
	# Runtime lookup keeps this script runnable unchanged on the pre-SimReads checkout.
	if _reads != null and _reads.has_method(method):
		_reads.callv(method, arguments)
	else:
		_world.callv(method, arguments)

func _until_frame(target: int) -> void:
	while Engine.get_process_frames() < target:
		await process_frame
		call_deferred("_settled")
		await frame_settled

func _settled() -> void:
	frame_settled.emit()

func _fail(message: String) -> void:
	push_error("identity-p9: " + message)
	quit(1)
