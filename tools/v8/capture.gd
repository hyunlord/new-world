# tools/v8/capture.gd — two screenshots of the live game for pass reports.
# Purpose: load res://scenes/main.tscn, advance the simulation to a tick through
#   the bridge's tooling call, and save a zoomed-in ("near") and a zoomed-out
#   ("far") frame. Tooling only: the game never runs this script.
# Bridge methods called: advance_ticks (tooling), get_settlement_snapshot,
#   get_time_snapshot.
# usage: Godot --path . --rendering-driver opengl3 --resolution 1920x1080 \
#          --script res://tools/v8/capture.gd -- --out <abs dir> [--tick 1440]
extends SceneTree

const ORIGIN: Vector2 = Vector2(448, 28)
const TILE_SIZE: float = 16.0

var _out: String = ""
var _tick: int = 1440


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--out" and i + 1 < args.size():
			_out = args[i + 1]
		elif args[i] == "--tick" and i + 1 < args.size():
			_tick = int(args[i + 1])
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
	var village: Vector2 = Vector2(32, 32)
	var snap: Dictionary = world.call("get_settlement_snapshot")
	var xs: PackedInt32Array = snap.get("formation_xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("formation_ys", PackedInt32Array())
	if not xs.is_empty():
		village = Vector2(xs[0], ys[0])
	var shots: Array[Array] = [["near", village, 4.0], ["far", Vector2(32, 32), 1.0]]
	for shot: Array in shots:
		camera.position = ORIGIN + (shot[1] as Vector2) * TILE_SIZE + Vector2(8, 8)
		camera.zoom = Vector2.ONE * float(shot[2])
		camera.reset_smoothing()
		camera.force_update_scroll()
		for node_name: String in ["TerrainRenderer", "BuildingRenderer", "AgentRenderer"]:
			var renderer: Node = main.get_node_or_null(node_name)
			if renderer != null and renderer.has_method("refresh_snapshot"):
				renderer.call("refresh_snapshot")
		for _i: int in 30:
			await process_frame
		await RenderingServer.frame_post_draw
		var image: Image = root.get_texture().get_image()
		var path: String = _out.path_join("%s_t%d.png" % [shot[0], _tick])
		if image == null or image.save_png(path) != OK:
			push_error("capture: could not save " + path)
			quit(1)
			return
		print("capture: saved ", path)
	quit(0)
