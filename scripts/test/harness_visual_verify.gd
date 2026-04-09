## Harness visual verification runner.
## Boots the full game, advances N ticks, captures screenshots + data evidence.
##
## Usage (windowed — for screenshots):
##   godot --path . --script scripts/test/harness_visual_verify.gd -- --feature <name> --ticks <n>
## Usage (headless — text data only):
##   godot --headless --path . --script scripts/test/harness_visual_verify.gd -- --feature <name> --ticks <n>
##
## Output: .harness/evidence/<feature>/
##   screenshot_tick0000.png   — initial state
##   screenshot_tick<NN>.png   — at 25%, 50%, 75% progress
##   screenshot_tickFINAL.png  — final state
##   entity_summary.txt        — agent counts, jobs, position spread
##   performance.txt           — tick timing stats
##   console_log.txt           — captured errors/warnings from Godot log
##   ffi_verify.txt            — FFI method chain + data sanity verification
##   manifest.txt              — list of all evidence files produced

extends SceneTree

## Phase constants
const PHASE_WAIT_SCENE: int = 0
const PHASE_WAIT_SETUP: int = 1
const PHASE_RUNNING: int = 2
const PHASE_SCREENSHOT_WAIT: int = 3
const PHASE_FINAL: int = 4

var _feature: String = "unknown"
var _total_ticks: int = 4380
var _evidence_dir: String = ""
var _ticks_done: int = 0
var _screenshot_ticks: Array[int] = []
var _next_screenshot_idx: int = 0
var _tick_times: PackedFloat64Array = PackedFloat64Array()
var _sim_engine: RefCounted = null
var _main_node: Node = null
var _phase: int = PHASE_WAIT_SCENE
var _wait_frames: int = 0
var _is_headless: bool = false
var _pending_screenshot_label: String = ""
var _batch_size: int = 20
var _ffi_verified: bool = false


func _init() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--feature":
				i += 1
				if i < args.size():
					_feature = args[i]
			"--ticks":
				i += 1
				if i < args.size():
					_total_ticks = int(args[i])
		i += 1

	# Resolve evidence directory (absolute path from project root)
	var project_path := ProjectSettings.globalize_path("res://")
	_evidence_dir = project_path.path_join(".harness/evidence/%s" % _feature)
	DirAccess.make_dir_recursive_absolute(_evidence_dir)

	# Screenshot at 0%, 25%, 50%, 75%, 100%
	_screenshot_ticks = [0, _total_ticks / 4, _total_ticks / 2, _total_ticks * 3 / 4, _total_ticks]
	# Remove duplicates for very short runs
	var seen := {}
	var unique: Array[int] = []
	for t in _screenshot_ticks:
		if not seen.has(t):
			seen[t] = true
			unique.append(t)
	_screenshot_ticks = unique

	_is_headless = DisplayServer.get_name() == "headless"
	print("[visual-verify] Feature: %s, Ticks: %d, Headless: %s" % [_feature, _total_ticks, str(_is_headless)])


func _initialize() -> void:
	var err := change_scene_to_file("res://scenes/main/main.tscn")
	if err != OK:
		_write_text("error.txt", "Failed to load main scene: %s" % error_string(err))
		quit(1)


func _process(delta: float) -> bool:
	match _phase:
		PHASE_WAIT_SCENE:
			_main_node = current_scene
			if _main_node and _main_node.is_node_ready():
				# Wait for setup phase to complete (entity spawning, map gen, etc.)
				_phase = PHASE_WAIT_SETUP
				_wait_frames = 60
				print("[visual-verify] Main scene loaded, waiting for setup...")

		PHASE_WAIT_SETUP:
			_wait_frames -= 1
			if _wait_frames <= 0:
				_sim_engine = _main_node.get("sim_engine") as RefCounted
				if _sim_engine == null:
					_write_text("error.txt", "sim_engine not found on main scene node")
					quit(1)
					return true
				# Pause so main.gd _process() won't advance ticks via update()
				_sim_engine.is_paused = true
				_phase = PHASE_RUNNING
				_next_screenshot_idx = 0
				# Capture initial state screenshot
				if _should_screenshot(0):
					_pending_screenshot_label = "tick0000"
					_phase = PHASE_SCREENSHOT_WAIT
					_wait_frames = 2
				print("[visual-verify] Setup complete, starting tick loop (paused main _process)...")

		PHASE_RUNNING:
			if _ticks_done >= _total_ticks:
				# All ticks done
				_pending_screenshot_label = "tickFINAL"
				_phase = PHASE_SCREENSHOT_WAIT
				_wait_frames = 2
				return false

			# Find next screenshot boundary
			var next_boundary: int = _total_ticks
			if _next_screenshot_idx < _screenshot_ticks.size():
				next_boundary = mini(_screenshot_ticks[_next_screenshot_idx], _total_ticks)

			# How many ticks until that boundary?
			var ticks_remaining := next_boundary - _ticks_done
			var batch := mini(_batch_size, ticks_remaining)
			if batch <= 0:
				batch = mini(_batch_size, _total_ticks - _ticks_done)

			if batch > 0:
				var tick_start := Time.get_ticks_usec()
				_sim_engine.advance_ticks(batch)
				var tick_end := Time.get_ticks_usec()
				var total_ms := float(tick_end - tick_start) / 1000.0
				_tick_times.append(total_ms / float(batch))
				_ticks_done += batch

			# FFI verification at ~50% progress
			if _ticks_done >= _total_ticks / 2 and not _ffi_verified:
				_ffi_verified = true
				var ffi_results: Dictionary = _verify_ffi_chain()
				_write_ffi_verify(ffi_results)

			# Progress logging
			if _ticks_done % 500 < _batch_size or _ticks_done >= _total_ticks:
				print("[visual-verify] Tick %d / %d" % [_ticks_done, _total_ticks])

			# Check if we hit a screenshot tick
			if _should_screenshot(_ticks_done):
				if _ticks_done >= _total_ticks:
					_pending_screenshot_label = "tickFINAL"
				else:
					_pending_screenshot_label = "tick%04d" % _ticks_done
				_phase = PHASE_SCREENSHOT_WAIT
				_wait_frames = 2

		PHASE_SCREENSHOT_WAIT:
			# Wait frames for the renderer to update before capturing
			_wait_frames -= 1
			if _wait_frames <= 0:
				_capture_screenshot(_pending_screenshot_label)
				_pending_screenshot_label = ""
				# Write partial data after each screenshot so timeout doesn't lose all data
				_write_partial_data()
				if _ticks_done >= _total_ticks:
					_phase = PHASE_FINAL
				else:
					_phase = PHASE_RUNNING

		PHASE_FINAL:
			_write_entity_summary()
			_write_performance_data()
			_write_console_log()
			_write_manifest()
			print("[visual-verify] Evidence captured to: %s" % _evidence_dir)
			quit(0)
			return true

	return false


## Check if we should capture a screenshot at the given tick.
func _should_screenshot(tick: int) -> bool:
	while _next_screenshot_idx < _screenshot_ticks.size():
		if tick >= _screenshot_ticks[_next_screenshot_idx]:
			_next_screenshot_idx += 1
			return true
		break
	return false


## Capture viewport screenshot (windowed mode only).
func _capture_screenshot(label: String) -> void:
	if _is_headless:
		print("[visual-verify] Screenshot skipped (headless): %s" % label)
		return
	var viewport := root.get_viewport()
	if viewport == null:
		return
	var img := viewport.get_texture().get_image()
	if img == null or img.get_size() == Vector2i.ZERO:
		print("[visual-verify] Screenshot failed (no image data): %s" % label)
		return
	var path := _evidence_dir.path_join("screenshot_%s.png" % label)
	var err := img.save_png(path)
	if err == OK:
		print("[visual-verify] Screenshot: %s" % path)
	else:
		print("[visual-verify] Screenshot save error: %s" % error_string(err))


## Write data files with current progress. Called after each screenshot so
## data survives even if the process is killed by timeout.
func _write_partial_data() -> void:
	_write_entity_summary()
	_write_performance_data()
	# Console log only at final (reads log file which grows during execution)


## Write entity summary: agent counts, job distribution, position spread.
func _write_entity_summary() -> void:
	var lines := PackedStringArray()
	lines.append("# Entity Summary at tick %d" % _ticks_done)
	lines.append("")

	if _sim_engine and _sim_engine.has_method("get_agent_snapshots"):
		var snapshots: Array = _sim_engine.get_agent_snapshots()
		lines.append("Total agents: %d" % snapshots.size())

		var jobs := {}
		var alive_count := 0
		var positions: Array[Vector2] = []

		for snap in snapshots:
			if snap is Dictionary:
				if snap.get("alive", false):
					alive_count += 1
				var job: String = str(snap.get("job", "unknown"))
				jobs[job] = jobs.get(job, 0) + 1
				if snap.has("x") and snap.has("y"):
					var sx: Variant = snap.get("x", 0.0)
					var sy: Variant = snap.get("y", 0.0)
					if (sx is float or sx is int) and (sy is float or sy is int):
						positions.append(Vector2(float(sx), float(sy)))

		lines.append("Alive: %d" % alive_count)
		lines.append("")
		lines.append("## Job Distribution")
		var sorted_jobs := jobs.keys()
		sorted_jobs.sort()
		for job_name in sorted_jobs:
			lines.append("- %s: %d" % [job_name, jobs[job_name]])

		if positions.size() > 1:
			var min_x := INF
			var max_x := -INF
			var min_y := INF
			var max_y := -INF
			for pos in positions:
				min_x = minf(min_x, pos.x)
				max_x = maxf(max_x, pos.x)
				min_y = minf(min_y, pos.y)
				max_y = maxf(max_y, pos.y)
			var spread_x := max_x - min_x
			var spread_y := max_y - min_y
			lines.append("")
			lines.append("## Position Spread")
			lines.append("- X range: %.1f to %.1f (spread: %.1f)" % [min_x, max_x, spread_x])
			lines.append("- Y range: %.1f to %.1f (spread: %.1f)" % [min_y, max_y, spread_y])
			if spread_x < 5.0 and spread_y < 5.0:
				lines.append("- WARNING: All agents clustered within 5 tiles — possible stuck behavior")

	# Band info
	if _sim_engine and _sim_engine.has_method("get_band_list"):
		var bands: Array = _sim_engine.get_band_list()
		lines.append("")
		lines.append("## Bands")
		lines.append("- Total bands: %d" % bands.size())
		for band in bands:
			if band is Dictionary:
				lines.append("- Band %s: %d members" % [
					str(band.get("id", "?")),
					band.get("member_count", 0),
				])

	# World summary
	if _sim_engine and _sim_engine.has_method("get_world_summary"):
		var summary: Dictionary = _sim_engine.get_world_summary()
		if not summary.is_empty():
			lines.append("")
			lines.append("## World Summary")
			for key in summary:
				lines.append("- %s: %s" % [str(key), str(summary[key])])

	_write_text("entity_summary.txt", "\n".join(lines))


## Write performance data: tick timing stats.
func _write_performance_data() -> void:
	var lines := PackedStringArray()
	lines.append("# Performance at tick %d" % _ticks_done)
	lines.append("")

	if _tick_times.size() > 0:
		var total_ms := 0.0
		var max_ms := 0.0
		for t in _tick_times:
			total_ms += t
			max_ms = maxf(max_ms, t)
		var avg_ms := total_ms / float(_tick_times.size())
		var est_tps := 1000.0 / avg_ms if avg_ms > 0.0 else 9999.0

		lines.append("Samples: %d (batched measurements)" % _tick_times.size())
		lines.append("Avg tick: %.3f ms" % avg_ms)
		lines.append("Max tick: %.3f ms" % max_ms)
		lines.append("Est. TPS (sim only): %.1f" % est_tps)
		lines.append("")

		if avg_ms > 50.0:
			lines.append("WARNING: Average tick > 50ms — below 20 TPS target")
		elif avg_ms > 33.33:
			lines.append("WARNING: Average tick > 33ms — may cause frame stutter")

		if max_ms > 100.0:
			lines.append("WARNING: Max tick spike > 100ms — severe frame stutter")

	lines.append("")
	lines.append("Engine.get_frames_per_second(): %d" % Engine.get_frames_per_second())
	lines.append("Headless mode: %s" % str(_is_headless))

	_write_text("performance.txt", "\n".join(lines))


## Write console log: extract errors/warnings from Godot's log file.
func _write_console_log() -> void:
	var lines := PackedStringArray()
	lines.append("# Console Log (captured at tick %d)" % _ticks_done)
	lines.append("")

	var log_dir := OS.get_user_data_dir().path_join("logs")
	if DirAccess.dir_exists_absolute(log_dir):
		var dir := DirAccess.open(log_dir)
		if dir:
			dir.list_dir_begin()
			var latest_log := ""
			var latest_time := 0
			var fname := dir.get_next()
			while fname != "":
				if fname.ends_with(".log"):
					var full_path := log_dir.path_join(fname)
					var mod_time := FileAccess.get_modified_time(full_path)
					if mod_time > latest_time:
						latest_time = mod_time
						latest_log = full_path
				fname = dir.get_next()
			dir.list_dir_end()

			if latest_log != "":
				var file := FileAccess.open(latest_log, FileAccess.READ)
				if file:
					var content := file.get_as_text()
					var error_count := 0
					var warn_count := 0
					for line in content.split("\n"):
						var lower := line.to_lower()
						if "error" in lower and "visual-verify" not in lower:
							lines.append("[ERROR] %s" % line.strip_edges())
							error_count += 1
						elif "warning" in lower or "warn" in lower:
							lines.append("[WARN] %s" % line.strip_edges())
							warn_count += 1
					lines.append("")
					lines.append("Total: %d errors, %d warnings" % [error_count, warn_count])

	if lines.size() <= 2:
		lines.append("No errors or warnings found in Godot log.")

	_write_text("console_log.txt", "\n".join(lines))


## Write manifest listing all evidence files.
func _write_manifest() -> void:
	var lines := PackedStringArray()
	lines.append("# Evidence Manifest — %s" % _feature)
	lines.append("# Generated at tick %d" % _ticks_done)
	lines.append("")

	var dir := DirAccess.open(_evidence_dir)
	if dir:
		dir.list_dir_begin()
		var fname := dir.get_next()
		while fname != "":
			if not dir.current_is_dir():
				var full := _evidence_dir.path_join(fname)
				var f := FileAccess.open(full, FileAccess.READ)
				var size: int = f.get_length() if f else 0
				lines.append("%s  (%d bytes)" % [fname, size])
			fname = dir.get_next()
		dir.list_dir_end()

	_write_text("manifest.txt", "\n".join(lines))


## Verify that all expected FFI methods are callable and return sane data.
func _verify_ffi_chain() -> Dictionary:
	var results: Dictionary = {}

	# Core methods that must always work
	var required_methods: Array[String] = [
		"get_minimap_snapshot",
		"get_world_summary",
		"get_agent_snapshots",
		"get_frame_snapshots",
	]

	# Feature-specific methods (added by recent features)
	var feature_methods: Array[String] = [
		"get_tile_grid_walls",
		"get_wall_plans_count",
	]

	for method in required_methods:
		var has: bool = _sim_engine != null and _sim_engine.has_method(method)
		results["ffi_required_" + method] = "OK" if has else "MISSING"

	for method in feature_methods:
		var has: bool = _sim_engine != null and _sim_engine.has_method(method)
		results["ffi_feature_" + method] = "OK" if has else "MISSING"

	# Data sanity: call methods and check non-empty returns
	if _sim_engine != null:
		if _sim_engine.has_method("get_minimap_snapshot"):
			var snap: Dictionary = _sim_engine.get_minimap_snapshot()
			results["data_minimap_buildings"] = str(snap.get("buildings", []).size())
			results["data_minimap_entities"] = str(snap.get("entities", []).size())

		if _sim_engine.has_method("get_tile_grid_walls"):
			var tile_data: Dictionary = _sim_engine.get_tile_grid_walls()
			var wall_x_raw: Variant = tile_data.get("wall_x", PackedInt32Array())
			var wall_count: int = (wall_x_raw as PackedInt32Array).size() if wall_x_raw is PackedInt32Array else 0
			var floor_x_raw: Variant = tile_data.get("floor_x", PackedInt32Array())
			var floor_count: int = (floor_x_raw as PackedInt32Array).size() if floor_x_raw is PackedInt32Array else 0
			results["data_tile_grid_walls"] = str(wall_count)
			results["data_tile_grid_floors"] = str(floor_count)

		if _sim_engine.has_method("get_wall_plans_count"):
			results["data_wall_plans"] = str(_sim_engine.get_wall_plans_count())

	print("[visual-verify] FFI chain verified: %d checks" % results.size())
	return results


## Write FFI verification results to evidence directory.
func _write_ffi_verify(results: Dictionary) -> void:
	var lines := PackedStringArray()
	lines.append("=== FFI Chain Verification ===")
	lines.append("tick: %d" % _ticks_done)
	lines.append("")

	var all_ok: bool = true
	var sorted_keys: Array = results.keys()
	sorted_keys.sort()
	for key in sorted_keys:
		var value: String = str(results[key])
		var status: String = "OK"
		if value == "MISSING":
			status = "FAIL"
			all_ok = false
		elif key.begins_with("data_") and value == "0":
			status = "WARN"
		lines.append("%s: %s [%s]" % [key, value, status])

	lines.append("")
	lines.append("overall: %s" % ("PASS" if all_ok else "FAIL"))

	_write_text("ffi_verify.txt", "\n".join(lines))


## Write a text file to the evidence directory.
func _write_text(filename: String, content: String) -> void:
	var path := _evidence_dir.path_join(filename)
	var file := FileAccess.open(path, FileAccess.WRITE)
	if file:
		file.store_string(content)
	else:
		push_error("[visual-verify] Failed to write: %s" % path)
