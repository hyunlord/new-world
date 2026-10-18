extends SceneTree

const DrawClock = preload("res://scripts/test/art/identity_draw_clock_p12.gd")
var _draw_clock: DrawClock = DrawClock.new()

const FixedDelta = preload("res://scripts/test/art/test_fixed_delta_p9.gd")
var _identity_delta_samples: int = 0
var _identity_delta_valid: bool = true

const ReadRoute = preload("res://scripts/test/art/test_sim_reads_equivalence_p9.gd")

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
	_draw_clock.begin()
	call_deferred("_run")

func _run() -> void:
	var framing: String = OS.get_environment("P12_IDENTITY_ID")
	var output: String = OS.get_environment("P12_IDENTITY_OUT")
	if not CASES.has(framing) or output.is_empty():
		_fail("requires P12_IDENTITY_ID=P1|P2|P4|P5|P6 and P12_IDENTITY_OUT")
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
	_reads = ReadRoute.resolve(_world)
	_camera.call("set_auto_track", false)
	_camera.position_smoothing_enabled = false
	_camera.process_mode = Node.PROCESS_MODE_DISABLED
	await _until_frame(10)
	if Engine.get_process_frames() != 10:
		_fail("startup exceeded absolute state-change frame 10")
		return
	var spec: Array = CASES[framing]
	if int(spec[3]) > 0:
		ReadRoute.mutate(_world, "reset_world_stress", [int(spec[0]), int(spec[3])])
	else:
		ReadRoute.mutate(_world, "reset_world", [int(spec[0])])
	ReadRoute.mutate(_world, "advance_ticks", [int(spec[1])])
	_world.call("set_sim_speed", 0.0)
	Selection.set_layers_enabled(self, true)
	var settlements: Dictionary = ReadRoute.read(_world, "get_settlement_snapshot")
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
	var state_frame: int = Engine.get_process_frames()
	if framing == "P6":
		# reset_world emits a deferred signal that closes the inspector.
		await _until_frame(12)
		var agents: Dictionary = ReadRoute.read(_world, "get_agent_snapshot")
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
	await _until_frame(130)
	if framing == "P6" and not _main.get_node("UI/AgentInspectorPanel").is_visible_in_tree():
		_fail("P6 requires the inspector to remain visible at capture")
		return
	_draw_clock.draw_frame(Engine.get_process_frames())
	var image: Image = root.get_texture().get_image()
	var time: Dictionary = ReadRoute.read(_world, "get_time_snapshot")
	if not _identity_delta_valid or _identity_delta_samples == 0 or Engine.get_process_frames() - state_frame != 120 or int(time["tick"]) != int(spec[1]) or image.get_size() != Vector2i(1920, 1080):
		_fail("capture frame, tick, or framebuffer invariant failed")
		return
	if DirAccess.make_dir_recursive_absolute(output) != OK or image.save_png(output.path_join(framing + ".png")) != OK:
		_fail("cannot save capture")
		return
	var draw_evidence: Dictionary = _draw_clock.evidence()
	if not bool(draw_evidence["valid"]):
		_fail("unexpected render steps; capture invalid: " + JSON.stringify(draw_evidence))
		return
	var manifest: Dictionary = {"census": _identity_census(), "draw_clock": draw_evidence, "framing": framing, "seed": spec[0], "tick": time["tick"], "state_change_frame": state_frame, "capture_frame": Engine.get_process_frames(), "settle_frames": 120, "fixed_fps": 60, "validated_delta_samples": _identity_delta_samples, "delta_tolerance": 0.000000001, "camera": [_camera.position.x, _camera.position.y], "zoom": spec[2], "debug_build": OS.is_debug_build(), "sim_reads": _reads != null}
	var file := FileAccess.open(output.path_join(framing + ".json"), FileAccess.WRITE)
	if file == null:
		_fail("cannot save manifest")
		return
	file.store_string(JSON.stringify(manifest, "\t") + "\n")
	quit(0)

func _until_frame(target: int) -> void:
	while Engine.get_process_frames() < target:
		await process_frame
		call_deferred("_settled")
		await frame_settled

func _settled() -> void:
	frame_settled.emit()

func _fail(message: String) -> void:
	var output: String = OS.get_environment("P12_IDENTITY_OUT")
	var framing: String = OS.get_environment("P12_IDENTITY_ID")
	if not output.is_empty() and DirAccess.make_dir_recursive_absolute(output) == OK:
		var rejected := FileAccess.open(output.path_join(framing + ".rejected-clock.json"), FileAccess.WRITE)
		if rejected != null:
			rejected.store_string(JSON.stringify({"valid": false, "reason": message, "capture_frame": Engine.get_process_frames(), "draw_clock": _draw_clock.evidence()}, "\t") + "\n")
			rejected.close()
	push_error("identity-p10: " + message)
	quit(1)


func _process(delta: float) -> bool:
	_draw_clock.schedule(Engine.get_process_frames(), delta)
	if true:
		_identity_delta_samples += 1
		if not FixedDelta.accepts_delta(delta):
			_identity_delta_valid = false
			_fail("identity requires actual process delta 1/60 on every frame; observed %.15f" % delta)
	return false


func _identity_census() -> Dictionary:
	var agents: Dictionary = ReadRoute.read(_world, "get_agent_snapshot")
	var agent_ids: PackedInt64Array = agents.get("agent_ids", PackedInt64Array())
	return {"world_info": ReadRoute.read(_world, "get_world_info"),
		"population": agent_ids.size(), "settlements": ReadRoute.read(_world, "get_settlement_snapshot"),
		"clock": ReadRoute.read(_world, "get_time_snapshot"),
		"loaded_bridge_sha256": FileAccess.get_sha256("res://rust/target/debug/libsim_bridge.dylib")}
