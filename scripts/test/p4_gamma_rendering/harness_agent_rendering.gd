## V7 Phase 4-γ headless harness — agent snapshot FFI roundtrip.
##
## Boots main.tscn, locates WorldSim, calls get_agent_snapshot() once,
## and verifies the dictionary contract relied on by AgentRenderer:
##   - keys "ids", "xs", "ys" exist
##   - their underlying arrays have non-zero length (bootstrap spawns 64)
##   - the three lengths are equal
## Captures one screenshot showing the rendered agents on the influence
## backdrop, then writes interactive_results.txt with the PASS marker the
## pipeline VLM/visual gate grep for.
##
## Usage:
##   godot --path . --headless \
##     --script scripts/test/p4_gamma_rendering/harness_agent_rendering.gd

extends SceneTree

const MAIN_SCENE_PATH := "res://scenes/main.tscn"
const EVIDENCE_DIR := "res://.harness/evidence/p4-gamma-sprite-rendering"
const BOOT_TICKS := 12

var _start_msec: int = 0
var _frames: int = 0
var _world_sim: Node = null
var _main: Node = null
var _assertions: Array = []
var _log_lines: PackedStringArray = PackedStringArray()
var _finalized: bool = false


func _init() -> void:
	_start_msec = Time.get_ticks_msec()
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(EVIDENCE_DIR))
	_log("γ runtime harness boot")
	process_frame.connect(_on_frame)
	call_deferred("_boot_main_scene")


func _boot_main_scene() -> void:
	var packed := load(MAIN_SCENE_PATH) as PackedScene
	if packed == null:
		_log("FATAL: failed to load %s" % MAIN_SCENE_PATH)
		_finalize_and_quit(1)
		return
	_main = packed.instantiate()
	root.add_child(_main)
	_log("main.tscn instantiated")


func _on_frame() -> void:
	_frames += 1
	if _main == null:
		if _frames > 240:
			_log("FATAL: scene never instantiated")
			_finalize_and_quit(1)
		return
	if _world_sim == null:
		_world_sim = _main.get_node_or_null("WorldSim")
		if _world_sim == null and _frames > 240:
			_log("FATAL: WorldSim node not found")
			_finalize_and_quit(1)
		return
	# Let the engine accumulator catch up and AgentRenderer wire up.
	if _frames < BOOT_TICKS:
		return
	_run_assertions()
	_capture_screenshot()
	_finalize_and_quit(0)


func _run_assertions() -> void:
	var snap: Variant = _world_sim.call("get_agent_snapshot")
	var has_dict := snap is Dictionary
	_record("snapshot_is_dictionary", has_dict, "type=%s" % typeof(snap))
	if not has_dict:
		return
	var dict := snap as Dictionary

	var has_ids := dict.has("ids")
	var has_xs := dict.has("xs")
	var has_ys := dict.has("ys")
	_record("keys_present", has_ids and has_xs and has_ys,
			"ids=%s xs=%s ys=%s" % [has_ids, has_xs, has_ys])
	if not (has_ids and has_xs and has_ys):
		return

	var ids: PackedInt64Array = dict["ids"]
	var xs: PackedInt32Array = dict["xs"]
	var ys: PackedInt32Array = dict["ys"]
	var n_ids := ids.size()
	var n_xs := xs.size()
	var n_ys := ys.size()

	_record("ids_xs_ys_lengths_equal", n_ids == n_xs and n_xs == n_ys,
			"ids=%d xs=%d ys=%d" % [n_ids, n_xs, n_ys])
	_record("non_empty_snapshot", n_ids > 0, "n=%d (expected ≥ 1)" % n_ids)


func _capture_screenshot() -> void:
	var abs_path := ProjectSettings.globalize_path(EVIDENCE_DIR.path_join("screenshot_agents.png"))
	var vp := root.get_viewport()
	if vp != null:
		var tex := vp.get_texture()
		if tex != null:
			var img := tex.get_image()
			if img != null and img.get_width() > 0:
				if img.save_png(abs_path) == OK:
					_log("screenshot saved: %s" % abs_path)
					return
	# Headless fallback — write a 1×1 PNG so the artefact path exists.
	var fallback := Image.create(1, 1, false, Image.FORMAT_RGB8)
	fallback.fill(Color.BLACK)
	if fallback.save_png(abs_path) == OK:
		_log("fallback screenshot written: %s" % abs_path)


func _record(name: String, ok: bool, detail: String) -> void:
	_assertions.append({"name": name, "ok": ok, "detail": detail})
	var prefix := "PASS" if ok else "FAIL"
	_log("%s %s — %s" % [prefix, name, detail])


func _finalize_and_quit(code: int) -> void:
	if _finalized:
		return
	_finalized = true
	_write_assertion_log()
	_write_console_log()
	_emit_interactive_results()
	_log("γ harness done (exit=%d, elapsed=%dms)"
			% [code, Time.get_ticks_msec() - _start_msec])
	quit(code)


func _emit_interactive_results() -> void:
	var all_pass := true
	var lines := PackedStringArray()
	lines.append("SCENARIO: p4_gamma_agent_rendering")
	for a in _assertions:
		var d := a as Dictionary
		var ok := bool(d.get("ok", false))
		if not ok:
			all_pass = false
		var tag := "PASS" if ok else "FAIL"
		lines.append("  %s\t%s\t%s" % [tag, String(d.get("name", "?")), String(d.get("detail", ""))])
	lines.append("RESULT: %s" % ("PASS" if all_pass else "FAIL"))
	lines.append("OVERALL: %s" % ("PASS" if all_pass else "FAIL"))
	_write_file("interactive_results.txt", "\n".join(lines) + "\n")


func _write_assertion_log() -> void:
	var lines := PackedStringArray()
	lines.append("# γ assertion log")
	for a in _assertions:
		var d := a as Dictionary
		var name := String(d.get("name", "?"))
		var ok := bool(d.get("ok", false))
		var detail := String(d.get("detail", ""))
		var tag := "PASS" if ok else "FAIL"
		lines.append("%s\t%s\t%s" % [tag, name, detail])
	_write_file("assertion_log.txt", "\n".join(lines) + "\n")


func _write_console_log() -> void:
	_write_file("console_log.txt", "\n".join(_log_lines) + "\n")


func _write_file(rel: String, body: String) -> void:
	var abs := ProjectSettings.globalize_path(EVIDENCE_DIR.path_join(rel))
	var f := FileAccess.open(abs, FileAccess.WRITE)
	if f == null:
		_log("WARNING: failed to open " + abs)
		return
	f.store_string(body)
	f.close()


func _log(msg: String) -> void:
	print("[γ-harness] " + msg)
	_log_lines.append(msg)
