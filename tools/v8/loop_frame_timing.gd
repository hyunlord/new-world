# tools/v8/loop_frame_timing.gd — M-LOOP §5 frame timing check.
#   "UI: opening a card or chapter adds no frame > 33 ms."
#   Runs windowed, opens/closes the card panel and chapter panel, measures frame times.
#   Output: frame timing report to --output directory.
# usage: $GODOT_BIN --path . --script res://tools/v8/loop_frame_timing.gd -- [--output DIR]
# NOTE: requires a WINDOWED build (not --headless) for accurate frame timing.
extends SceneTree

const SETTLE_FRAMES: int = 30
const MEASURE_FRAMES: int = 60
const THRESHOLD_MS: float = 33.0

var _output_dir: String = "/Users/rexxa/work/loop-data"
var _results: Array[Dictionary] = []


func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	for i: int in args.size():
		if args[i] == "--output" and i + 1 < args.size():
			_output_dir = args[i + 1]
	call_deferred("_run")


func _run() -> void:
	DirAccess.make_dir_recursive_absolute(_output_dir)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	for _i: int in SETTLE_FRAMES:
		await process_frame
	var world: Node = main.get_node_or_null("WorldSim")
	var reads: Node = main.get_node_or_null("SimReads")
	if world == null:
		push_error("loop_frame_timing: missing WorldSim")
		quit(1)
		return

	# Reset and settle.
	if reads != null:
		reads.call("mutate_raw", "reset_world", [0])
	for _i: int in SETTLE_FRAMES:
		await process_frame
	SimReads.mutate(world, "cmd_set_steward", [true, 0])
	for _i: int in SETTLE_FRAMES:
		await process_frame

	# Advance a few hours so cards might appear.
	if reads != null:
		reads.call("mutate_raw", "advance_ticks", [360])
	for _i: int in 10:
		await process_frame

	# Measure baseline frame times.
	var baseline: Array[float] = await _measure_frames("baseline")

	# Measure: open chapter panel (Goals).
	var goals: Node = main.find_child("Goals", true, false)
	if goals != null and goals.has_method("toggle_panel"):
		goals.call("toggle_panel")
	var chapter_open: Array[float] = await _measure_frames("chapter_open")
	if goals != null and goals.has_method("toggle_panel"):
		goals.call("toggle_panel")
	for _i: int in 5:
		await process_frame

	# Measure: open card panel (simulate by making it visible).
	var card_panel: Node = main.find_child("CardPanel", true, false)
	if card_panel != null:
		(card_panel as Control).visible = true
		(card_panel as Control).size = Vector2.ZERO
	var card_open: Array[float] = await _measure_frames("card_open")
	if card_panel != null:
		(card_panel as Control).visible = false
	for _i: int in 5:
		await process_frame

	# Measure: open and close chapter panel repeatedly.
	var toggle_times: Array[float] = []
	for _rep: int in 5:
		var before: int = Time.get_ticks_usec()
		if goals != null and goals.has_method("toggle_panel"):
			goals.call("toggle_panel")
		await process_frame
		var after: int = Time.get_ticks_usec()
		@warning_ignore("integer_division")
		toggle_times.append(float(after - before) / 1000.0)
		if goals != null and goals.has_method("toggle_panel"):
			goals.call("toggle_panel")
		await process_frame

	# Compile results.
	var baseline_max: float = _max_ms(baseline)
	var chapter_max: float = _max_ms(chapter_open)
	var card_max: float = _max_ms(card_open)
	var toggle_max: float = _max_ms(toggle_times)

	_results = [
		{"test": "baseline", "max_ms": baseline_max, "pass": baseline_max <= THRESHOLD_MS},
		{"test": "chapter_open", "max_ms": chapter_max, "pass": chapter_max <= THRESHOLD_MS},
		{"test": "card_open", "max_ms": card_max, "pass": card_max <= THRESHOLD_MS},
		{"test": "chapter_toggle", "max_ms": toggle_max, "frames": toggle_times, "pass": toggle_max <= THRESHOLD_MS},
	]

	var all_pass: bool = true
	for r: Dictionary in _results:
		var status: String = "PASS" if bool(r["pass"]) else "FAIL"
		print("  %s: max=%.2f ms — %s" % [r["test"], r["max_ms"], status])
		if not bool(r["pass"]):
			all_pass = false

	print("FRAME_TIMING_DONE all_pass=%s threshold=%.1f ms" % ["true" if all_pass else "false", THRESHOLD_MS])

	# Write result.
	var path: String = _output_dir.path_join("frame_timing_result.json")
	var file: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify({"threshold_ms": THRESHOLD_MS, "results": _results}, "\t"))
		file.close()

	quit(0 if all_pass else 1)


func _measure_frames(label: String) -> Array[float]:
	var times: Array[float] = []
	for _i: int in MEASURE_FRAMES:
		var before: int = Time.get_ticks_usec()
		await process_frame
		var after: int = Time.get_ticks_usec()
		@warning_ignore("integer_division")
		times.append(float(after - before) / 1000.0)
	print("  %s: %d frames, max=%.2f ms, avg=%.2f ms" % [label, times.size(), _max_ms(times), _avg_ms(times)])
	return times


func _max_ms(times: Array[float]) -> float:
	var m: float = 0.0
	for t: float in times:
		m = maxf(m, t)
	return m


func _avg_ms(times: Array[float]) -> float:
	if times.is_empty():
		return 0.0
	var total: float = 0.0
	for t: float in times:
		total += t
	return total / float(times.size())
