extends SceneTree

const Selection = preload("res://scripts/core/settlement_selection.gd")
const CASES: Array[Dictionary] = [
	{"name": "W0", "seed": 20261010, "agents": 0, "tick": 40560},
	{"name": "W500", "seed": 15100, "agents": 500, "tick": 1440},
]
const FRAME_COUNT: int = 600
signal frame_settled
var _failed: bool = false

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	root.size = Vector2i(1920, 1080)
	for spec: Dictionary in CASES:
		await _exercise(spec)
		if _failed:
			quit(1)
			return
	print("P9_CONSUMERS_PASS worlds=W0,W500 frames_per_world=600 reset_after_cached_state=true")
	quit(0)

func _exercise(spec: Dictionary) -> void:
	var scene: PackedScene = load("res://scenes/main.tscn") as PackedScene
	var main: Node = scene.instantiate()
	var world: Node = main.get_node("WorldSim")
	world.call("set_sim_speed", 0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	current_scene = main
	var camera: Camera2D = main.get_node("Camera2D") as Camera2D
	camera.call("set_auto_track", false)
	camera.position_smoothing_enabled = false
	camera.process_mode = Node.PROCESS_MODE_DISABLED
	var reset_ok: bool
	if int(spec["agents"]) > 0:
		reset_ok = bool(SimReads.mutate(world, "reset_world_stress", [int(spec["seed"]), int(spec["agents"])]))
	else:
		reset_ok = bool(SimReads.mutate(world, "reset_world", [int(spec["seed"])]))
	_check(reset_ok, str(spec["name"]) + " reset")
	_check(int(SimReads.mutate(world, "advance_ticks", [int(spec["tick"])])) == int(spec["tick"]), str(spec["name"]) + " explicit target tick")
	await _settle_frame()
	Selection.set_layers_enabled(self, true)
	main.get_node("ZoomLodController").set_process(false)
	# Exercise both close and overview consumers together, even when normal LOD hides one.
	for node: Node in main.get_children():
		if node is CanvasItem:
			node.show()
	main.get_node("WorldRenderer").set("current_channel", 0)
	var settlements: Dictionary = SimReads.read(world, "get_settlement_snapshot")
	var xs: PackedInt32Array = settlements.get("formation_xs", PackedInt32Array())
	var ys: PackedInt32Array = settlements.get("formation_ys", PackedInt32Array())
	_check(not xs.is_empty() and xs.size() == ys.size(), "settlement framing available")
	if _failed:
		main.free()
		return
	camera.position = Vector2(448, 28) + Vector2(xs[0], ys[0]) * 16.0 + Vector2(8, 8)
	var agents: Dictionary = SimReads.read(world, "get_agent_snapshot")
	var ids: PackedInt64Array = agents.get("ids", PackedInt64Array())
	_check(not ids.is_empty(), "inspector resident available")
	if _failed:
		main.free()
		return
	var detail: Dictionary = world.call("get_agent_detail", ids[0])
	_check(bool(detail.get("found", false)), "inspector resident found")
	main.get_node("UI/AgentInspectorPanel").call("display_agent", detail)
	main.get_node("UI/GodHandPanel").call("show")
	main.get_node("UI/GodHandPanel").call("set_log_open", true)
	main.get_node("UI/CausalPanel").call("show")
	main.get_node("UI/CausalPanel").call("display_history", world.call("get_tile_causal_history", xs[0], ys[0]), xs[0], ys[0])
	var start_frame: int = Engine.get_process_frames()
	for index: int in FRAME_COUNT:
		camera.zoom = Vector2.ONE * (4.0 if index < 300 else 1.0)
		camera.force_update_scroll()
		await _settle_frame()
		_check(int(SimReads.read(world, "get_time_snapshot")["tick"]) == int(spec["tick"]), "frozen explicit tick unchanged")
		_check(bool(main.get_node("UI/AgentInspectorPanel").get("visible")), "inspector remains open")
	_check(Engine.get_process_frames() - start_frame == FRAME_COUNT, "exactly 600 engine frames")
	# GodHandEffects has retained the cached read-only intervention dictionary by now.
	var state: Dictionary = SimReads.read(world, "get_intervention_state")
	_check(state.is_read_only(), "real intervention snapshot is read-only before reset")
	_check(bool(SimReads.mutate(world, "reset_world", [int(spec["seed"])])), "reset after retained cache succeeds")
	await _settle_frame()
	await _settle_frame()
	_check(int(SimReads.read(world, "get_time_snapshot")["tick"]) == 0, "reset state observed by consumers")
	print("P9_CONSUMER_WORLD ", JSON.stringify({"world": spec["name"], "target_tick": spec["tick"], "frames": FRAME_COUNT, "start_frame": start_frame, "zoom4_frames": 300, "zoom1_frames": 300, "all_layers_enabled": true, "inspector_open": true, "god_hand_open": true, "overlay_channel": 0, "reset_after_cache": true, "passed": not _failed}))
	main.free()
	await _settle_frame()

func _settle_frame() -> void:
	await process_frame
	call_deferred("_emit_settled")
	await frame_settled

func _emit_settled() -> void:
	frame_settled.emit()

func _check(condition: bool, message: String) -> void:
	if not condition:
		_failed = true
		push_error("P9 consumer check: " + message)
