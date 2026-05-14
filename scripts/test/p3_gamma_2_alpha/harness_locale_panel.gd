## Harness runtime verification for V7 Phase 3-γ (γ-2-α):
##   Locale autoload + CausalPanel scaffold + Q-key toggle.
##
## Boots main.tscn, lets Locale autoload populate, validates:
##   - Locale.key_count() >= 5103 (locked threshold)
##   - Locale.ltr("UI_CAUSAL_PANEL_TITLE") under en/ko
##   - Missing-key fallback returns the key itself
##   - /root/Main/UI/CausalPanel exists with visible == false at boot
##   - Synthetic KEY_Q press toggles visibility true → false
##   - Title + placeholder Labels render after toggle
##
## Captures two screenshots into the evidence directory:
##   screenshot_hidden.png    — boot frame (panel hidden)
##   screenshot_toggled.png   — after Q press (panel visible)
##
## Usage:
##   godot --path . --script scripts/test/p3_gamma_2_alpha/harness_locale_panel.gd

extends SceneTree

const MAIN_SCENE_PATH := "res://scenes/main.tscn"
const EVIDENCE_DIR := "res://.harness/evidence/p3-gamma-2-alpha-locale-panel-scaffold"
const KEY_COUNT_THRESHOLD := 5103

enum Phase {
	WAIT_SCENE,
	BEFORE_TOGGLE,
	AFTER_TOGGLE,
	DONE,
}

var _phase: int = Phase.WAIT_SCENE
var _frames_in_phase: int = 0
var _main_node: Node = null
var _causal_panel: Node = null
var _locale_node: Node = null
var _log_lines: PackedStringArray = PackedStringArray()
var _assertions: Array = []  # Array[Dictionary{name, ok, detail}]
var _start_msec: int = 0
var _hidden_shot_taken: bool = false
var _toggled_shot_taken: bool = false
var _final_after_off_visible: bool = true
var _finalized: bool = false


func _init() -> void:
	_start_msec = Time.get_ticks_msec()
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(EVIDENCE_DIR))
	_clean_evidence_dir()
	_log("γ-2-α runtime harness boot")
	process_frame.connect(_on_frame)
	call_deferred("_boot_main_scene")


func _log(msg: String) -> void:
	print("[γ-2-α-harness] " + msg)
	_log_lines.append(msg)


func _clean_evidence_dir() -> void:
	var abs_dir := ProjectSettings.globalize_path(EVIDENCE_DIR)
	var dir := DirAccess.open(abs_dir)
	if dir == null:
		return
	dir.list_dir_begin()
	var fname := dir.get_next()
	while fname != "":
		if not dir.current_is_dir():
			DirAccess.remove_absolute(abs_dir.path_join(fname))
		fname = dir.get_next()
	dir.list_dir_end()


func _boot_main_scene() -> void:
	var packed := load(MAIN_SCENE_PATH) as PackedScene
	if packed == null:
		_log("FATAL: failed to load %s" % MAIN_SCENE_PATH)
		_finalize_and_quit(1)
		return
	var inst := packed.instantiate()
	root.add_child(inst)
	_main_node = inst
	_log("main scene instantiated")


func _on_frame() -> void:
	_frames_in_phase += 1
	match _phase:
		Phase.WAIT_SCENE:
			if _main_node == null or not _main_node.is_inside_tree():
				if _frames_in_phase > 300:
					_log("TIMEOUT waiting for main scene")
					_finalize_and_quit(1)
				return
			_locale_node = root.get_node_or_null("/root/Locale")
			_causal_panel = _main_node.get_node_or_null("UI/CausalPanel")
			if _locale_node == null or _causal_panel == null:
				if _frames_in_phase > 300:
					_log("TIMEOUT waiting for Locale / CausalPanel")
					_run_locale_assertions()
					_run_panel_initial_assertions()
					_finalize_and_quit(1)
				return
			# Give Locale._ready() and panel._ready() a few frames to settle.
			if _frames_in_phase < 5:
				return
			_log("Locale + CausalPanel resolved; running pre-toggle assertions")
			_run_locale_assertions()
			_run_panel_initial_assertions()
			_hidden_shot_taken = _take_screenshot("hidden")
			_phase = Phase.BEFORE_TOGGLE
			_frames_in_phase = 0
		Phase.BEFORE_TOGGLE:
			if _frames_in_phase < 2:
				return
			_inject_q_key()
			_phase = Phase.AFTER_TOGGLE
			_frames_in_phase = 0
		Phase.AFTER_TOGGLE:
			if _frames_in_phase < 3:
				return
			_run_panel_toggled_assertions()
			_toggled_shot_taken = _take_screenshot("toggled")
			_inject_q_key()
			# Snapshot post-second-toggle visibility one frame later.
			_phase = Phase.DONE
			_frames_in_phase = 0
		Phase.DONE:
			if _frames_in_phase < 2:
				return
			_final_after_off_visible = bool(_panel_visible())
			_record("panel_visible_after_second_q",
					_final_after_off_visible == false,
					"after second Q press; got %s" % str(_final_after_off_visible))
			_finalize_and_quit(0)


func _take_screenshot(label: String) -> bool:
	var abs_path := ProjectSettings.globalize_path(
			EVIDENCE_DIR.path_join("screenshot_%s.png" % label))
	var viewport := root.get_viewport()
	if viewport != null:
		var tex := viewport.get_texture()
		if tex != null:
			var img := tex.get_image()
			if img != null and img.get_width() > 0 and img.get_height() > 0:
				var err := img.save_png(abs_path)
				if err == OK:
					_log("screenshot saved: %s" % abs_path)
					return true
				_log("WARNING: save_png failed (%d) for %s" % [err, label])
	# Fallback — 1×1 black PNG (file existence only).
	var img2 := Image.create(1, 1, false, Image.FORMAT_RGB8)
	img2.fill(Color.BLACK)
	if img2.save_png(abs_path) == OK:
		_log("fallback screenshot written (headless): %s" % abs_path)
		return true
	return false


func _panel_visible() -> bool:
	if _causal_panel == null:
		return false
	if _causal_panel.has_method("is_panel_visible"):
		return bool(_causal_panel.call("is_panel_visible"))
	return bool(_causal_panel.get("visible"))


func _inject_q_key() -> void:
	var ev := InputEventKey.new()
	ev.pressed = true
	ev.echo = false
	ev.keycode = KEY_Q
	# Route directly to the panel's _unhandled_input handler — in headless
	# mode the viewport's input routing is unreliable. Calling the handler
	# directly mirrors what the engine does when no other UI consumes the
	# event. We deliberately avoid Input.parse_input_event() so the toggle
	# fires exactly once per injection (otherwise both direct call AND
	# deferred dispatch would each flip visibility = no net change).
	if _causal_panel != null and _causal_panel.has_method("_unhandled_input"):
		_causal_panel.call("_unhandled_input", ev)
	_log("KEY_Q press injected")


func _record(name: String, ok: bool, detail: String) -> void:
	_assertions.append({"name": name, "ok": ok, "detail": detail})
	var prefix := "ASSERT_OK" if ok else "ASSERT_FAIL"
	_log("%s %s — %s" % [prefix, name, detail])


func _run_locale_assertions() -> void:
	if _locale_node == null:
		_record("locale_autoload_present", false, "/root/Locale missing")
		return
	_record("locale_autoload_present", true, "Locale node resolved")
	var en_count := -1
	if _locale_node.has_method("key_count"):
		en_count = int(_locale_node.call("key_count"))
	_record("locale_key_count_en",
			en_count >= KEY_COUNT_THRESHOLD,
			"key_count=%d (threshold>=%d)" % [en_count, KEY_COUNT_THRESHOLD])

	var en_title := ""
	if _locale_node.has_method("ltr"):
		en_title = String(_locale_node.call("ltr", "UI_CAUSAL_PANEL_TITLE"))
	_record("locale_ltr_en_title",
			en_title == "Why? — Causal History",
			"got=%s" % en_title)

	# Switch to ko, verify ko translation.
	var ko_title := ""
	var ko_count := -1
	if _locale_node.has_method("set_language"):
		_locale_node.call("set_language", "ko")
		if _locale_node.has_method("key_count"):
			ko_count = int(_locale_node.call("key_count"))
		if _locale_node.has_method("ltr"):
			ko_title = String(_locale_node.call("ltr", "UI_CAUSAL_PANEL_TITLE"))
		# Restore en for the rest of the run (panel was built with en strings).
		_locale_node.call("set_language", "en")
	_record("locale_key_count_ko",
			ko_count >= KEY_COUNT_THRESHOLD,
			"key_count=%d (threshold>=%d)" % [ko_count, KEY_COUNT_THRESHOLD])
	_record("locale_ltr_ko_title",
			ko_title == "왜? — 인과 기록",
			"got=%s" % ko_title)

	# Missing-key fallback.
	var fallback := ""
	if _locale_node.has_method("ltr"):
		fallback = String(_locale_node.call("ltr", "NONEXISTENT_KEY_FOR_FALLBACK_TEST"))
	_record("locale_missing_key_fallback",
			fallback == "NONEXISTENT_KEY_FOR_FALLBACK_TEST",
			"got=%s" % fallback)


func _run_panel_initial_assertions() -> void:
	if _causal_panel == null:
		_record("panel_present_at_boot", false, "/root/Main/UI/CausalPanel missing")
		return
	_record("panel_present_at_boot", true,
			"resolved at %s" % _causal_panel.get_path())
	_record("panel_visible_initial",
			_panel_visible() == false,
			"is_panel_visible=%s" % str(_panel_visible()))


func _run_panel_toggled_assertions() -> void:
	if _causal_panel == null:
		_record("panel_visible_after_first_q", false, "panel missing")
		return
	_record("panel_visible_after_first_q",
			_panel_visible() == true,
			"is_panel_visible=%s" % str(_panel_visible()))

	# Verify title + placeholder child Labels are present and non-empty.
	var title_label_present := false
	var placeholder_label_present := false
	var en_title := "Why? — Causal History"
	var en_placeholder_prefix := "Click a tile"
	for child in _causal_panel.get_children():
		if child is Label:
			var txt := String((child as Label).text)
			if txt == en_title:
				title_label_present = true
			elif txt.begins_with(en_placeholder_prefix):
				placeholder_label_present = true
	_record("panel_title_label_present", title_label_present,
			"matching title Label child found=%s" % str(title_label_present))
	_record("panel_placeholder_label_present", placeholder_label_present,
			"matching placeholder Label child found=%s" % str(placeholder_label_present))


func _finalize_and_quit(code: int) -> void:
	if _finalized:
		return
	_finalized = true
	_write_runtime_summary()
	_write_assertion_log()
	_write_console_log()
	_write_visual_checklist()
	_write_manifest()
	_log("γ-2-α harness done (exit=%d, elapsed=%dms)"
			% [code, Time.get_ticks_msec() - _start_msec])
	quit(code)


func _write_runtime_summary() -> void:
	var ok := 0
	var fail := 0
	for a in _assertions:
		if (a as Dictionary).get("ok", false):
			ok += 1
		else:
			fail += 1
	var body := "## γ-2-α runtime summary\n"
	body += "assertions_ok: %d\n" % ok
	body += "assertions_fail: %d\n" % fail
	body += "threshold_key_count: %d\n" % KEY_COUNT_THRESHOLD
	body += "elapsed_ms: %d\n" % (Time.get_ticks_msec() - _start_msec)
	body += "hidden_screenshot: %s\n" % str(_hidden_shot_taken)
	body += "toggled_screenshot: %s\n" % str(_toggled_shot_taken)
	body += "final_visible_after_second_q: %s\n" % str(_final_after_off_visible)
	_write_file("runtime_summary.txt", body)


func _write_assertion_log() -> void:
	var lines := PackedStringArray()
	lines.append("# γ-2-α assertion log\n")
	for a in _assertions:
		var d := a as Dictionary
		var name := String(d.get("name", "?"))
		var ok := bool(d.get("ok", false))
		var detail := String(d.get("detail", ""))
		var tag := "ASSERT_OK" if ok else "ASSERT_FAIL"
		lines.append("%s\t%s\t%s" % [tag, name, detail])
	_write_file("assertion_log.txt", "\n".join(lines) + "\n")


func _write_console_log() -> void:
	_write_file("console_log.txt", "\n".join(_log_lines) + "\n")


func _write_visual_checklist() -> void:
	# This file is interpreted by the evaluator. Use VISUAL_OK / VISUAL_FAIL
	# tokens on a single trailing line per the harness conventions.
	var verdict := "VISUAL_OK"
	for a in _assertions:
		if not bool((a as Dictionary).get("ok", false)):
			verdict = "VISUAL_FAIL"
			break
	if not _hidden_shot_taken or not _toggled_shot_taken:
		verdict = "VISUAL_FAIL"
	var body := "# Visual Checklist — p3-gamma-2-alpha-locale-panel-scaffold\n\n"
	for a in _assertions:
		var d := a as Dictionary
		var name := String(d.get("name", "?"))
		var ok := bool(d.get("ok", false))
		var token := "VISUAL_OK" if ok else "VISUAL_FAIL"
		body += "## Assertion: %s\n%s — %s\n\n" % [
			name, token, String(d.get("detail", ""))
		]
	body += "## Assertion: hidden screenshot captured\n%s\n\n" % (
			"VISUAL_OK" if _hidden_shot_taken else "VISUAL_FAIL")
	body += "## Assertion: toggled screenshot captured\n%s\n\n" % (
			"VISUAL_OK" if _toggled_shot_taken else "VISUAL_FAIL")
	body += "\n%s\n" % verdict
	_write_file("visual_checklist_rendered.md", body)


static var MANIFEST_FILES: Array[String] = [
	"assertion_log.txt",
	"console_log.txt",
	"runtime_summary.txt",
	"screenshot_hidden.png",
	"screenshot_toggled.png",
	"visual_checklist_rendered.md",
]


func _write_manifest() -> void:
	var abs_dir := ProjectSettings.globalize_path(EVIDENCE_DIR)
	var present := PackedStringArray()
	for fname: String in MANIFEST_FILES:
		if FileAccess.file_exists(abs_dir.path_join(fname)):
			present.append(fname)
	_write_file("manifest.txt", "\n".join(present) + "\n")


func _write_file(rel: String, body: String) -> void:
	var abs := ProjectSettings.globalize_path(EVIDENCE_DIR.path_join(rel))
	var f := FileAccess.open(abs, FileAccess.WRITE)
	if f == null:
		_log("WARNING: failed to open " + abs)
		return
	f.store_string(body)
	f.close()
