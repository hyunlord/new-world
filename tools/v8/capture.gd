# tools/v8/capture.gd — screenshots of the live game for pass reports (M1 colony view).
# Purpose: load res://scenes/main.tscn, advance the simulation to a tick through the bridge's tooling call,
#   and save a "near" frame (the camp at the default 2× zoom) and a "far" frame (1× zoom, the map centre).
#   Tooling only: the game never runs this script.
# Bridge methods called: advance_ticks (tooling), get_time_snapshot, get_agent_snapshot, get_building_snapshot.
# --panels adds three shots at the near framing: "inspect" (a click on the first agent through
#   OrderTool.click_world opens the inspector), "causal" (a click on an empty tile opens the causal "why?"
#   panel) and "chronicle" (the chronicle panel toggled as the C key does).
# usage: Godot --path . --rendering-driver opengl3 --resolution 1920x1080 \
#          --script res://tools/v8/capture.gd -- --out <abs dir> [--tick 1440] [--panels] [--zoom 2]
extends SceneTree

const TILE: float = 16.0

var _out: String = ""
var _tick: int = 1440
var _panels: bool = false
var _zoom: float = 2.0


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--out" and i + 1 < args.size():
			_out = args[i + 1]
		elif args[i] == "--tick" and i + 1 < args.size():
			_tick = int(args[i + 1])
		elif args[i] == "--zoom" and i + 1 < args.size():
			_zoom = float(args[i + 1])
		elif args[i] == "--panels":
			_panels = true
	call_deferred("_run")


func _run() -> void:
	if _out.is_empty():
		push_error("capture: --out is required")
		quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	for _i: int in 10:
		await process_frame
	var world: Node = main.get_node_or_null("WorldSim")
	var camera: Camera2D = main.get_node_or_null("Camera2D") as Camera2D
	if world == null or camera == null:
		push_error("capture: missing WorldSim or Camera2D")
		quit(1)
		return
	world.set("process_mode", Node.PROCESS_MODE_DISABLED)
	var now: int = int((world.call("get_time_snapshot") as Dictionary).get("tick", 0))
	if _tick > now:
		world.call("advance_ticks", _tick - now)
	var camp: Vector2i = _hearth(world)
	var shots: Array[Array] = [["near", Vector2(camp), _zoom], ["far", Vector2(96, 96), 1.0]]
	for shot: Array in shots:
		_frame(camera, shot[1] as Vector2, float(shot[2]))
		for _i: int in 30:
			await process_frame
		await _save("%s_t%d.png" % [shot[0], _tick])
	if _panels:
		await _panel_shots(main, world, camera, camp)
	quit(0)


func _hearth(world: Node) -> Vector2i:
	var b: Dictionary = world.call("get_building_snapshot")
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	var xs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = b.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[i] == 0:
			return Vector2i(xs[i], ys[i])
	return Vector2i(96, 96)


func _frame(camera: Camera2D, tile: Vector2, zoom: float) -> void:
	camera.set_process(false)
	camera.zoom = Vector2.ONE * zoom
	camera.position = tile * TILE + Vector2(8, 8)
	camera.reset_smoothing()
	camera.force_update_scroll()


func _save(name: String) -> void:
	await RenderingServer.frame_post_draw
	var image: Image = root.get_texture().get_image()
	var path: String = _out.path_join(name)
	if image == null or image.save_png(path) != OK:
		push_error("capture: could not save " + path)
	else:
		print("capture: saved ", path)


func _panel_shots(main: Node, world: Node, camera: Camera2D, camp: Vector2i) -> void:
	_frame(camera, Vector2(camp), _zoom)
	for _i: int in 10:
		await process_frame
	var tool: Node = main.get_node_or_null("ColonyView/OrderTool")
	var snap: Dictionary = world.call("get_agent_snapshot")
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var steps: Array[Array] = []
	if tool != null and not xs.is_empty():
		steps.append(["inspect", Vector2i(xs[0], ys[0])])
	if tool != null:
		steps.append(["causal", camp + Vector2i(1, 1)])
	steps.append(["chronicle", Vector2i.ZERO])
	for step: Array in steps:
		if step[0] == "chronicle":
			var chronicle: Node = main.get_node_or_null("UI/ChroniclePanel")
			if chronicle != null and chronicle.has_method("toggle_visible"):
				chronicle.call("toggle_visible")
		else:
			var tile: Vector2i = step[1]
			tool.call("click_world", Vector2(tile) * TILE + Vector2(8, 8), tile)
		for _i: int in 30:
			await process_frame
		await _save("%s_t%d.png" % [step[0], _tick])
