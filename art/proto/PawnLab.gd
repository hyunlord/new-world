# Purpose: standalone pawn layer-composite rehearsal and visible 200-pawn measurement.
# Bridge methods called: none. External PNGs are checked exports, not project resources.
extends Node2D

var output: String = ""
var seconds: float = 60.0
var mode: String = "crowd"
var data: Dictionary = {}
var textures: Array[Array] = []
var ground: Texture2D
var elapsed: float = 0.0
var started_usec: int = 0
var previous_draw_usec: int = 0
var frames: int = 0
var deltas: Array[float] = []
var finished: bool = false
var hidden_process_frames: int = 0
var visibility_samples: Array[Dictionary] = []
var script_hash: String = ""
var captures: int = 0
var visible_bounds: bool = true

func _ready() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var manifest: String = ""
	for index: int in range(0, args.size() - 1, 2):
		match args[index]:
			"--manifest": manifest = args[index + 1]
			"--out": output = args[index + 1]
			"--seconds": seconds = float(args[index + 1])
			"--mode": mode = args[index + 1]
	if manifest.is_empty() or output.is_empty():
		push_error("PawnLab requires --manifest and --out")
		get_tree().quit(1)
		return
	data = JSON.parse_string(FileAccess.get_file_as_string(manifest))
	for path: String in data["source_hashes"]:
		if FileAccess.get_sha256(path) != data["source_hashes"][path]:
			push_error("Pawn source hash mismatch: " + path)
			get_tree().quit(1)
			return
	for specimen: Dictionary in data["sprites"]:
		var sequence: Array = []
		for path: String in specimen["files"]:
			if FileAccess.get_sha256(path) != data["textures_sha256"][path]:
				push_error("Pawn texture hash mismatch: " + path)
				get_tree().quit(1)
				return
			sequence.append(ImageTexture.create_from_image(Image.load_from_file(path)))
		textures.append(sequence)
	ground = ImageTexture.create_from_image(Image.load_from_file(data["ground"]))
	DisplayServer.window_set_size(Vector2i(1600, 1080))
	get_viewport().size = Vector2i(1600, 1080)
	if DisplayServer.get_name() != "headless":
		get_window().always_on_top = true
		DisplayServer.window_move_to_foreground()
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	script_hash = FileAccess.get_sha256("res://PawnLab.gd")
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	DirAccess.make_dir_recursive_absolute(output)
	started_usec = Time.get_ticks_usec()
	previous_draw_usec = started_usec
	RenderingServer.frame_post_draw.connect(_after_draw)

func _process(_delta: float) -> void:
	if finished or textures.is_empty():
		return
	elapsed = float(Time.get_ticks_usec() - started_usec) / 1000000.0
	var drawable: bool = DisplayServer.window_can_draw()
	if not drawable:
		hidden_process_frames += 1
	if visibility_samples.size() <= floori(elapsed):
		visibility_samples.append({"wall_seconds": elapsed, "can_draw": drawable, "focused": DisplayServer.window_is_focused(), "rendered_frames": frames})
	queue_redraw()
	if elapsed >= seconds:
		finished = true
		_finish.call_deferred()

func _draw() -> void:
	if textures.is_empty():
		return
	for row: int in range(10):
		for column: int in range(16):
			draw_texture(ground, Vector2(column * 104, row * 112))
	var count: int = 200 if mode == "crowd" else 24
	var frame: int = int(elapsed * 8.0) % 7
	for index: int in range(count):
		var tex: Texture2D = textures[index % textures.size()][frame]
		var position_px: Vector2 = Vector2(12 + (index % 20) * 78, 8 + int(index / 20.0) * 104)
		if mode != "crowd":
			position_px = Vector2(40 + (index % 12) * 128, 200 + int(index / 12.0) * 256)
		var bounds: Rect2 = Rect2(position_px, tex.get_size())
		visible_bounds = visible_bounds and get_viewport_rect().encloses(bounds)
		if tex.get_width() == 64:
			draw_set_transform(position_px + Vector2(32, 88), 0.0, Vector2(12, 4))
			draw_circle(Vector2.ZERO, 1.0, Color(0, 0, 0, 0.4))
			draw_set_transform(Vector2.ZERO)
		draw_texture(tex, position_px)

func _after_draw() -> void:
	if finished or textures.is_empty():
		return
	var now_usec: int = Time.get_ticks_usec()
	deltas.append(float(now_usec - previous_draw_usec) / 1000.0)
	previous_draw_usec = now_usec
	elapsed = float(now_usec - started_usec) / 1000000.0
	frames += 1
	if captures == 0 or (elapsed >= seconds * 0.5 and captures == 1):
		get_viewport().get_texture().get_image().save_png(output.path_join("capture_%02d.png" % captures))
		captures += 1

func _finish() -> void:
	var headless: bool = DisplayServer.get_name() == "headless"
	var report: Dictionary = {
		"status": data["status"], "surface": "Godot windowed standalone pawn lab",
		"headless": headless, "display_server": DisplayServer.get_name(),
		"clock": "Time.get_ticks_usec monotonic wall clock",
		"window_position": [get_window().position.x, get_window().position.y],
		"screen_size": [DisplayServer.screen_get_size().x, DisplayServer.screen_get_size().y],
		"window_visible": not headless and DisplayServer.window_get_mode() != DisplayServer.WINDOW_MODE_MINIMIZED,
		"draw_count": 200 if mode == "crowd" else 24,
		"scale": 1, "requested_viewport": [1600, 1080],
		"viewport": [get_viewport_rect().size.x, get_viewport_rect().size.y],
		"window_size": [get_window().size.x, get_window().size.y],
		"all_sprite_bounds_in_viewport": visible_bounds,
		"elapsed_seconds": elapsed, "rendered_frames": frames, "rendered_fps": frames / elapsed,
		"frame_deltas_ms": deltas, "source_hashes": data["source_hashes"],
		"visibility_samples": visibility_samples, "hidden_process_frames": hidden_process_frames,
		"always_on_top": get_window().always_on_top, "script_sha256": script_hash,
		"source_unchanged_during_run": script_hash == FileAccess.get_sha256("res://PawnLab.gd"),
		"texture_hashes": data["textures_sha256"],
		"limitations": "Bounds prove on-screen placement, not freedom from pawn-to-pawn overlap; readability judged separately."
	}
	var file: FileAccess = FileAccess.open(output.path_join("runtime.json"), FileAccess.WRITE)
	file.store_string(JSON.stringify(report, "\t") + "\n")
	print("PAWN_LAB_DONE fps=", frames / elapsed)
	get_tree().quit(0)
