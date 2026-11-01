# tools/v8/loop_first10_capture.gd — M-LOOP §4 first-10-minutes capture.
#   Seed 0, steward on, first 10 in-game days at 1x. Captures every chapter/plaque/card
#   change and one screen per in-game day. KO, UI scale 1.0 and 1.25.
#   Output: images + manifest in the directory passed as --output.
# usage: $GODOT_BIN --path . --script res://tools/v8/loop_first10_capture.gd -- [--output DIR] [--scale FACTOR]
# NOTE: requires a WINDOWED build (not --headless) for screenshots.
extends SceneTree

const SEED: int = 0
const DAYS: int = 10
const TICKS_PER_DAY: int = 1440
const TOTAL_TICKS: int = DAYS * TICKS_PER_DAY
const TICKS_PER_HOUR: int = 60
const SETTLE_FRAMES: int = 10

var _output_dir: String = "/Users/rexxa/work/loop-data/first10/cand"
var _scale: float = 1.0
var _manifest: Array[Dictionary] = []
var _capture_index: int = 0
var _last_plaque: String = ""
var _last_card_ids: PackedStringArray = PackedStringArray()
var _last_chapter: int = -1


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--output" and i + 1 < args.size():
			_output_dir = args[i + 1]
		elif args[i] == "--scale" and i + 1 < args.size():
			_scale = float(args[i + 1])
	call_deferred("_run")


func _run() -> void:
	DirAccess.make_dir_recursive_absolute(_output_dir)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	for _i: int in SETTLE_FRAMES:
		await process_frame
	var world: Node = main.get_node_or_null("WorldSim")
	var reads: Node = main.get_node_or_null("SimReads")
	if world == null or reads == null:
		push_error("loop_first10_capture: missing WorldSim or SimReads")
		quit(1)
		return

	# Set UI scale.
	var hud: Node = main.find_child("MuxHud", true, false)
	if hud != null:
		hud.get("store").call("set_value", "ui_scale", _scale)
	# Set language to Korean.
	var locale: Node = root.get_node_or_null("Locale")
	if locale != null:
		locale.call("set_language", "ko")

	# Reset world with seed 0, steward on.
	reads.call("mutate_raw", "reset_world", [SEED])
	for _i: int in SETTLE_FRAMES:
		await process_frame
	SimReads.mutate(world, "cmd_set_steward", [true, 0])
	for _i: int in SETTLE_FRAMES:
		await process_frame

	# Capture initial state.
	await _capture(main, "initial", "Game start")

	# Advance hour by hour, capturing changes.
	@warning_ignore("integer_division")
	var total_hours: int = TOTAL_TICKS / TICKS_PER_HOUR
	var last_day: int = -1
	for hour: int in total_hours:
		reads.call("mutate_raw", "advance_ticks", [TICKS_PER_HOUR])
		for _i: int in 3:
			await process_frame

		var time: Dictionary = SimReads.read(world, "get_time_snapshot")
		var tick: int = int(time.get("tick", 0))
		@warning_ignore("integer_division")
		var day: int = tick / TICKS_PER_DAY

		# Check for plaque change.
		var plaque: Node = main.find_child("TaskPlaque", true, false)
		var plaque_text: String = str(plaque.call("plaque_text")) if plaque != null and plaque.has_method("plaque_text") else ""
		if plaque_text != _last_plaque and not plaque_text.is_empty():
			_last_plaque = plaque_text
			await _capture(main, "plaque_d%d_h%d" % [day, hour % 24], "Plaque: " + plaque_text)

		# Check for card change.
		var card_panel: Node = main.find_child("CardPanel", true, false)
		if card_panel != null and card_panel.has_method("active_ids"):
			var current_ids: PackedStringArray = card_panel.call("active_ids")
			if current_ids != _last_card_ids and not current_ids.is_empty():
				_last_card_ids = current_ids
				await _capture(main, "card_d%d_h%d" % [day, hour % 24], "Cards: " + ",".join(current_ids))

		# Check for chapter change.
		var goals: Node = main.find_child("Goals", true, false)
		if goals != null and goals.has_method("current_chapter"):
			var ch: int = goals.call("current_chapter")
			if ch != _last_chapter:
				_last_chapter = ch
				if ch > 0:
					await _capture(main, "chapter_%d_d%d" % [ch, day], "Chapter %d" % (ch + 1))

		# Daily capture.
		if day != last_day:
			last_day = day
			await _capture(main, "day_%02d" % day, "Day %d" % (day + 1))

	# Write manifest.
	var manifest_path: String = _output_dir.path_join("manifest.json")
	var file: FileAccess = FileAccess.open(manifest_path, FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify({"seed": SEED, "days": DAYS, "scale": _scale, "captures": _manifest}, "\t"))
		file.close()
	print("FIRST10_DONE captures=%d manifest=%s" % [_manifest.size(), manifest_path])
	quit(0)


func _capture(_main: Node, tag: String, description: String) -> void:
	for _i: int in 3:
		await process_frame
	var filename: String = "%03d_%s.png" % [_capture_index, tag]
	var path: String = _output_dir.path_join(filename)
	var image: Image = root.get_viewport().get_texture().get_image()
	if image != null:
		image.save_png(path)
	_manifest.append({"index": _capture_index, "file": filename, "tag": tag, "seen": description})
	_capture_index += 1
	print("  [%d] %s — %s" % [_capture_index - 1, filename, description])
