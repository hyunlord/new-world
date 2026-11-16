# world3d_flythrough.gd — M3D-1 camera-move capture of the 3D view (windowed): `--save <wsave> --out <dir>
#   [--seconds 16] [--fps 20]` loads the save, lets the clock run at 1× and moves the camera through keyframes —
#   a far view of the map, a pan over the camp, a 90° turn (E), a zoom to Z5 (the cutaway into the smelter house)
#   and back out — saving `frame_%04d.png` (1280×720) per output frame; encode with
#   `ffmpeg -framerate 20 -i frame_%04d.png …`. Prints `W3D_FLY_DONE frames=<n>`.
# Bridge methods called: load_world (direct); via SimReads get_time_snapshot.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")

## Keyframes: [t (0–1), tile x, tile y, zoom, yaw steps].
const KEYS: Array = [
	[0.00, 96.0, 100.0, 0.25, 0],
	[0.18, 94.0, 103.0, 0.5, 0],
	[0.36, 92.0, 104.0, 1.0, 0],
	[0.52, 92.0, 104.0, 1.0, 1],
	[0.70, 95.0, 106.0, 2.0, 1],
	[0.84, 95.0, 106.0, 4.0, 1],
	[1.00, 94.0, 104.0, 1.0, 0],
]


func _init() -> void:
	call_deferred("_run")


func _arg(name: String, fallback: String) -> String:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var i: int = args.find(name)
	return args[i + 1] if i >= 0 and i + 1 < args.size() else fallback


func _run() -> void:
	var out: String = _arg("--out", "")
	DirAccess.make_dir_recursive_absolute(out)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	var camera: Camera2D = main.get_node("Camera2D")
	for _i: int in 10:
		await process_frame
	var w3d: Node = main.get_node_or_null("World3D")
	if w3d == null or not bool(world.call("load_world", _arg("--save", ""))):
		push_error("world3d_flythrough: needs --renderer=3d and a loadable --save")
		quit(1)
		return
	(main.get_node("UI") as CanvasLayer).visible = false
	w3d.get_node("Sky").call("set_hour_override", 9.5)
	w3d.get_node("Terrain").call("build_all")
	for _i: int in 60:
		await process_frame
	var rig: Node = w3d.get_node("CameraRig")
	var seconds: float = float(_arg("--seconds", "16"))
	var fps: float = float(_arg("--fps", "20"))
	var frames: int = int(seconds * fps)
	var yaw_steps: int = 0
	for f: int in frames:
		var t: float = float(f) / float(maxi(1, frames - 1))
		var k: int = 0
		while k < KEYS.size() - 2 and t > float(KEYS[k + 1][0]):
			k += 1
		var a: Array = KEYS[k]
		var b: Array = KEYS[k + 1]
		var u: float = clampf((t - float(a[0])) / maxf(0.0001, float(b[0]) - float(a[0])), 0.0, 1.0)
		u = u * u * (3.0 - 2.0 * u)
		var tile: Vector2 = Vector2(lerpf(float(a[1]), float(b[1]), u), lerpf(float(a[2]), float(b[2]), u))
		var z: float = exp(lerpf(log(float(a[3])), log(float(b[3])), u))
		camera.zoom = Vector2(z, z)
		camera.position = tile * View.TILE + Vector2(View.TILE, View.TILE) * 0.5
		if int(b[4]) != yaw_steps and u > 0.0:
			rig.call("rotate_view", int(b[4]) - yaw_steps)
			yaw_steps = int(b[4])
		# Real frames between captures so the clock, people and the eased turn move on screen.
		for _i: int in 2:
			await process_frame
		await RenderingServer.frame_post_draw
		var img: Image = root.get_viewport().get_texture().get_image()
		img.resize(1280, 720, Image.INTERPOLATE_BILINEAR)
		img.save_png(out.path_join("frame_%04d.png" % f))
	print("W3D_FLY_DONE frames=%d" % frames)
	quit(0)
