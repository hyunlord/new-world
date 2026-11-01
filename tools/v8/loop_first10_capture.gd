# tools/v8/loop_first10_capture.gd — M-LOOP §4 first-10-minutes capture.
#   Seed 0, steward on, exactly 14,400 explicitly stepped ticks (10 in-game days). Captures every chapter/plaque/card
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
var _initial_tick: int = -1
var _errors: Array[String] = []
var _preview_only: bool = false
var _camera_frame: Dictionary = {}


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--preview-only":
			_preview_only = true
		elif args[i] == "--output" and i + 1 < args.size():
			_output_dir = args[i + 1]
		elif args[i] == "--scale" and i + 1 < args.size():
			_scale = float(args[i + 1])
	call_deferred("_run")


func _run() -> void:
	var directory_error: Error = DirAccess.make_dir_recursive_absolute(_output_dir)
	if directory_error != OK:
		push_error("loop_first10_capture: output directory failed: %s" % error_string(directory_error))
		quit(1)
		return
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node_or_null("WorldSim")
	var reads: Node = main.get_node_or_null("SimReads")
	if world == null or reads == null:
		push_error("loop_first10_capture: missing WorldSim or SimReads")
		quit(1)
		return

	# Disable only the bridge wall clock; UI/render callbacks continue normally.
	# Camera world_reset callbacks restore their own speed, so pausing alone
	# before awaiting reset frames does not guarantee deterministic ticks.
	world.call("set_sim_speed", 0.0)
	world.set_process(false)
	for _i: int in SETTLE_FRAMES:
		await process_frame

	# Set UI scale.
	var hud: Node = main.find_child("MuxHud", true, false)
	if hud != null:
		hud.get("store").call("set_value", "ui_scale", _scale)
	# Set language to Korean.
	var locale: Node = root.get_node_or_null("Locale")
	if locale != null:
		locale.call("set_language", "ko")

	# Reset world with seed 0, steward on.
	world.call("set_sim_speed", 0.0)
	var reset_ok: bool = bool(reads.call("mutate_raw", "reset_world", [SEED]))
	world.call("set_sim_speed", 0.0)
	for _i: int in SETTLE_FRAMES:
		await process_frame
	world.call("set_sim_speed", 0.0)
	if not reset_ok:
		_errors.append("seed reset failed")
	var steward_result: Dictionary = SimReads.mutate(world, "cmd_set_steward", [true, 0])
	if not bool(steward_result.get("accepted", false)):
		_errors.append("steward setup command rejected: " + str(steward_result))
	for _i: int in SETTLE_FRAMES:
		await process_frame

	_initial_tick = int((world.call("get_time_snapshot") as Dictionary).get("tick", -1))
	if _initial_tick != 0:
		_errors.append("initial tick must be 0, got %d" % _initial_tick)

	_frame_camp(main, world)

	# Capture initial state.
	await _capture(main, "initial", "Game start")

	# Advance hour by hour, capturing changes.
	@warning_ignore("integer_division")
	var total_hours: int = 0 if _preview_only else TOTAL_TICKS / TICKS_PER_HOUR
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
			# §4 (d): one daily screen with the notable chronicle open, so the latest important event
			# and its "← cause" are on screen.
			var chron: Control = main.find_child("ChroniclePanel", true, false) as Control
			if chron != null and chron.has_method("toggle_visible"):
				if not chron.visible:
					chron.call("toggle_visible")
				await process_frame
				await _capture(main, "day_%02d_chronicle" % day, "Day %d chronicle" % (day + 1))
				chron.call("toggle_visible")
			else:
				_errors.append("chronicle user navigation unavailable")

	var final_tick: int = int((world.call("get_time_snapshot") as Dictionary).get("tick", -1))
	var requested_ticks: int = 0 if _preview_only else TOTAL_TICKS
	var exact_ticks: bool = _initial_tick == 0 and final_tick == requested_ticks and final_tick - _initial_tick == requested_ticks
	if not exact_ticks:
		_errors.append("tick drift: initial=%d final=%d requested=%d" % [_initial_tick, final_tick, requested_ticks])
	if _manifest.is_empty():
		_errors.append("no captures produced")
	var driver_path: String = (get_script() as Script).resource_path
	var driver_hash: String = FileAccess.get_sha256(driver_path)
	var manifest_path: String = _output_dir.path_join("manifest.json")
	var file: FileAccess = FileAccess.open(manifest_path, FileAccess.WRITE)
	if file == null:
		_errors.append("manifest open failed: " + error_string(FileAccess.get_open_error()))
	else:
		file.store_string(JSON.stringify({
			"seed": SEED, "days": DAYS, "scale": _scale, "language": "ko",
			"initial_tick": _initial_tick, "final_tick": final_tick,
			"requested_ticks": requested_ticks, "exact_ticks": exact_ticks,
			"preview_only": _preview_only, "camera": _camera_frame,
			"clock_mode": "explicit_steps_world_process_disabled",
			"source_sha": OS.get_environment("M6B_CAPTURE_SOURCE_SHA"),
			"driver_path": driver_path,
			"driver_sha256": driver_hash if driver_hash.length() == 64 else "UNKNOWN",
			"passed": _errors.is_empty(), "errors": _errors, "captures": _manifest,
		}, "\t"))
		file.flush()
		var write_error: Error = file.get_error()
		file.close()
		if write_error != OK:
			_errors.append("manifest write failed: " + error_string(write_error))
	for message: String in _errors:
		push_error("loop_first10_capture: " + message)
	print("FIRST10_%s captures=%d initial_tick=%d final_tick=%d requested=%d manifest=%s" % [
		("PREVIEW" if _preview_only else "DONE") if _errors.is_empty() else "FAIL", _manifest.size(), _initial_tick, final_tick, requested_ticks, manifest_path])
	quit(0 if _errors.is_empty() else 1)


func _capture(main: Node, tag: String, description: String) -> void:
	for _i: int in 3:
		await process_frame
	var filename: String = "%03d_%s.png" % [_capture_index, tag]
	var path: String = _output_dir.path_join(filename)
	var image: Image = root.get_viewport().get_texture().get_image()
	var save_error: Error = ERR_INVALID_DATA
	if image != null and not image.is_empty():
		save_error = image.save_png(path)
	if save_error != OK:
		_errors.append("capture %s failed: %s" % [filename, error_string(save_error)])
	var world: Node = main.get_node("WorldSim")
	var tick: int = int((world.call("get_time_snapshot") as Dictionary).get("tick", -1))
	_manifest.append({
		"index": _capture_index, "file": filename, "tag": tag, "seen": description,
		"tick": tick, "saved": save_error == OK, "save_error": int(save_error),
	})
	_capture_index += 1
	print("  [%d] %s — %s (tick=%d saved=%s)" % [_capture_index - 1, filename, description, tick, str(save_error == OK)])



## Same framing for BASE and candidate; mirrors capture.gd's hearth lookup/frame.
func _frame_camp(main: Node, world: Node) -> void:
	var camera: Camera2D = main.get_node_or_null("Camera2D") as Camera2D
	if camera == null:
		_errors.append("missing Camera2D for matched camp framing")
		return
	var buildings: Dictionary = world.call("get_building_snapshot")
	var kinds: PackedByteArray = buildings.get("kinds", PackedByteArray())
	var xs: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	var camp: Vector2i = Vector2i(-1, -1)
	var strategy: String = "building_hearth"
	var founder_bounds: Array = []
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[i] == 0:
			camp = Vector2i(xs[i], ys[i])
			break
	if camp.x < 0:
		# At tick zero the building-event queue need not be projected yet.
		# Founders spawn within the actual riverside camp, which is chosen by
		# worldgen/camp.rs and is not necessarily the map midpoint.
		var agents: Dictionary = world.call("get_agent_snapshot")
		var ax: PackedInt32Array = agents.get("xs", PackedInt32Array())
		var ay: PackedInt32Array = agents.get("ys", PackedInt32Array())
		if not ax.is_empty() and ax.size() == ay.size():
			var low: Vector2i = Vector2i(ax[0], ay[0])
			var high: Vector2i = low
			for i: int in ax.size():
				low = Vector2i(mini(low.x, ax[i]), mini(low.y, ay[i]))
				high = Vector2i(maxi(high.x, ax[i]), maxi(high.y, ay[i]))
			camp = Vector2i((Vector2(low) + Vector2(high)) * 0.5)
			strategy = "initial_founder_bounds_center"
			founder_bounds = [low.x, low.y, high.x, high.y]
		else:
			var info: Dictionary = world.call("get_world_info")
			var width: int = int(info.get("width", 0))
			var height: int = int(info.get("height", 0))
			if width <= 0 or height <= 0:
				_errors.append("no hearth, founder positions, or valid map bounds for framing")
				return
			camp = Vector2i(Vector2(width, height) * 0.5)
			strategy = "world_bounds_center_no_founders"
	camera.set_process(false)
	camera.position_smoothing_enabled = false
	camera.zoom = Vector2.ONE
	camera.position = Vector2(camp) * 64.0 + Vector2(32, 32)
	camera.reset_smoothing()
	camera.force_update_scroll()
	_camera_frame = {
		"focus_tile": [camp.x, camp.y], "resolution": strategy, "founder_bounds": founder_bounds, "position": [camera.position.x, camera.position.y],
		"zoom": [camera.zoom.x, camera.zoom.y], "viewport": [root.size.x, root.size.y],
	}
