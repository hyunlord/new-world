# mart2_old_shots.gd — the M-ART2 comparison frames drawn by the OLD renderer (goal §11 "each scene rendered by the
#   old renderer at its commit").
# Purpose: run against a checkout of the base commit (the M1 16-px sheet renderer; `../new-world-mart2-base` at
#   29f720aa) with this file passed by absolute path: for every shot of a mart2_set.json written by
#   tools/v8/mart2_shots.gd, load that shot's world save (`<name>.wsave`, the exact moment; saves are compatible
#   because M-ART2 changes no simulation state), stop the clock, frame the same world tile so the same world area
#   fills the window (old zoom = new zoom × 64 / 16, since the old tile is 16 px), and save `old_<name>.png` beside it.
#   Prints MART2_OLD_DONE n=<shots>.
#   `Godot --path ../new-world-mart2-base --rendering-driver opengl3 --resolution 1920x1080 \
#      --script /abs/path/tools/v8/mart2_old_shots.gd -- --set <abs dir with mart2_set.json>`
# Bridge methods called: load_world (direct).
extends SceneTree

const OLD_TILE: float = 16.0
const NEW_TILE: float = 64.0


func _init() -> void:
	call_deferred("_run")


func _run() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = args.find("--set")
	if i < 0 or i + 1 >= args.size():
		push_error("mart2_old_shots: --set <dir> is required")
		quit(2)
		return
	var dir: String = args[i + 1]
	var set_json: Variant = JSON.parse_string(FileAccess.get_file_as_string(dir.path_join("mart2_set.json")))
	if not set_json is Dictionary:
		push_error("mart2_old_shots: no mart2_set.json in %s" % dir)
		quit(1)
		return
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	var camera: Camera2D = main.get_node("Camera2D")
	for _f: int in 10:
		await process_frame
	var n: int = 0
	for shot: Variant in (set_json as Dictionary).get("shots", []):
		var s: Dictionary = shot
		world.call("load_world", str(s["save"]))
		world.process_mode = Node.PROCESS_MODE_DISABLED
		for _f: int in 40:
			await process_frame
		var zoom: float = float(s["zoom"]) * NEW_TILE / OLD_TILE
		camera.zoom = Vector2(zoom, zoom)
		var tile: Array = s["tile"]
		camera.position = Vector2(float(tile[0]), float(tile[1])) * OLD_TILE + Vector2(OLD_TILE, OLD_TILE) * 0.5
		for _f: int in 20:
			await process_frame
		await RenderingServer.frame_post_draw
		root.get_viewport().get_texture().get_image().save_png(dir.path_join("old_%s.png" % str(s["name"])))
		n += 1
	print("MART2_OLD_DONE n=%d" % n)
	quit(0)
