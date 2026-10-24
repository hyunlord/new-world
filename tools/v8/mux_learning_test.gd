# mux_learning_test.gd: executable UI-state tests for registries, settings and lesson progression.
# Bridge methods: none. Uses isolated temporary settings and never changes the owner's progress.
extends SceneTree

const Catalog = preload("res://scripts/ui/mux/controls_catalog.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
const Helper = preload("res://scripts/ui/mux/learning_helper.gd")
const Help = preload("res://scripts/ui/mux/help_panel.gd")
const Goals = preload("res://scripts/ui/mux/goals_panel.gd")
const Settings = preload("res://scripts/ui/mux/settings_panel.gd")
const Facts = preload("res://scripts/ui/mux/learning_facts.gd")
const Highlight = preload("res://scripts/ui/mux/lesson_highlight.gd")
var failures: int = 0
var checks: int = 0

func _initialize() -> void:
	call_deferred("_run")

func _check(condition: bool, name_text: String) -> void:
	checks += 1
	if not condition:
		failures += 1
	print("MUX_LEARNING %s %s" % ["PASS" if condition else "FAIL", name_text])

func _run() -> void:
	_test_settings_isolation()
	_test_facts()
	await _test_observed_learning()
	await _test_highlight()
	var path: String = "user://mux-learning-test-%s.cfg" % OS.get_process_id()
	var data_path: String = "user://mux-lessons-test-%s.json" % OS.get_process_id()
	var store: Store = Store.new(path)
	_check(store.last_error == OK, "missing settings uses defaults")
	_check(store.value("helper_on") == true, "helper defaults on")
	_check(store.value("edge_pan") == false, "edge pan defaults off")
	_check(store.value("steward_default") == true, "new villages default steward on")
	_check(not store.set_value("ui_scale", 2.0), "unsupported scale rejected")
	_check(not store.set_value("edge_pan", "false"), "invalid setting type rejected")
	_check(not store.set_value("unknown", true), "unknown setting rejected")
	_check(store.set_value("ui_scale", 1.25), "supported scale accepted")
	_check(Store.new(path).value("ui_scale") == 1.25, "scale persisted")
	var catalog: Catalog = Catalog.new()
	_check(catalog.error.is_empty(), "controls registry valid")
	_check(catalog.entries.size() >= 40, "full control coverage")
	_check(catalog.tooltip("pause").contains("Space"), "pause tooltip names shortcut")
	_check(catalog.tooltip("home").contains("Home"), "home tooltip names shortcut")
	var markdown: String = catalog.markdown_table(func(key: String) -> String: return key)
	_check(markdown.contains("F12") and markdown.contains("Ctrl+Cmd+F"), "markdown uses shared table")
	_check(not Catalog.valid_condition({"any": []}), "empty condition rejected")
	_check(not Catalog.valid_condition({"unknown": "x"}), "unknown condition rejected")
	_check(not Catalog.condition_met({"fact": "missing"}, {}, {}), "missing fact cannot complete")
	_check(not Catalog.condition_met({"all": [{"event": "a"}, {"event": "b"}]}, {}, {"a": true}), "partial action cannot complete")
	_check(Catalog.condition_met({"all": [{"event": "a"}, {"event": "b"}]}, {}, {"a": true, "b": true}), "all real events complete")
	var invalid: Dictionary = {"schema_version": 2, "controls": []}
	_check(not str(Catalog.parse_registry(invalid, "controls", ["id"])["error"]).is_empty(), "unknown schema rejected")
	invalid = {"schema_version": 1, "controls": [{"id": "x"}, {"id": "x"}]}
	_check(not str(Catalog.parse_registry(invalid, "controls", ["id"])["error"]).is_empty(), "duplicate id rejected")
	var helper: Helper = Helper.new()
	helper.setup(store)
	root.add_child(helper)
	await process_frame
	_check(helper.current_id() == "camera", "camera first")
	_check(not helper.can_advance(), "unperformed lesson incomplete")
	helper.advance()
	_check(helper.current_id() == "camera", "next cannot fake completion")
	helper.observe("camera_moved")
	helper.observe("camera_zoomed")
	_check(not helper.can_advance(), "camera includes minimap")
	helper.observe("minimap_used")
	_check(helper.can_advance(), "camera completes on all actions")
	helper.advance()
	_check(helper.current_id() == "speed", "next lesson after completion")
	helper.skip_current()
	_check(helper.current_id() == "person", "explicit skip advances")
	var saved: Dictionary = Store.new(path).progress()
	_check(saved["completed"].has("camera"), "completed progress persisted")
	_check(saved["skipped"].has("speed"), "skip distinguished from completion")
	helper.observe("person_selected")
	helper.advance()
	_check(helper.current_id() == "stockpile", "select person then stockpile")
	helper.update_facts({"stockpile_exists": false, "steward_accepted_rules": [1]})
	_check(not helper.can_advance(), "accepted steward command does not imply completion")
	helper.update_facts({"stockpile_exists": true, "steward_accepted_rules": [1]})
	_check(helper.can_advance(), "steward actual completed work recognized")
	_check(helper._body.text == Catalog._text("MUX_LESSON_STOCKPILE_STEWARD", Callable()), "steward explanation shown")
	helper.turn_off()
	_check(not helper.visible, "helper off hides card")
	_check(Store.new(path).value("helper_on") == false, "helper off persisted")
	store.set_value("helper_on", true)
	_check(helper.visible, "helper can resume")
	var extension: Dictionary = {"schema_version": 1, "lessons": [{"id": "fixture_extension", "title_key": "MUX_LEARNING_TITLE", "body_key": "MUX_LESSON_WAIT", "trigger": {"always": true}, "completion": {"event": "fixture_action"}, "highlight": "minimap", "steward_key": "MUX_LESSON_DONE", "steward_rule": "none"}]}
	var file: FileAccess = FileAccess.open(data_path, FileAccess.WRITE)
	if file != null:
		file.store_string(JSON.stringify(extension))
		file.close()
	_check(helper.load_lessons(data_path), "new lesson loads by data alone")
	_check(helper.current_id() == "fixture_extension" and not helper.can_advance(), "extension uses generic condition")
	helper.observe("fixture_action")
	_check(helper.can_advance(), "extension completes without code branch")
	var help_panel: Help = Help.new()
	root.add_child(help_panel)
	help_panel.toggle_panel()
	_check(help_panel.visible, "help opens")
	_check(help_panel._rows.get_child_count() == catalog.entries.size() * 2, "help renders every control")
	for _frame: int in 3:
		await process_frame
	_check(root.get_visible_rect().encloses(help_panel.get_global_rect()), "fresh help settles within three frames without HUD poll")
	help_panel._scroll.custom_minimum_size.y = 1400
	await process_frame
	await process_frame
	_check(help_panel.size.y > 680, "fixture recreates inflated container minimum")
	help_panel._scroll.custom_minimum_size.y = 0
	for _frame: int in 3:
		await process_frame
	_check(help_panel.size.y <= 680, "released child minimum shrinks help without coordinator poll")
	_check(root.get_visible_rect().encloses(help_panel.get_global_rect()), "released child minimum recenters help in viewport")
	for factor: float in [1.0, 1.25, 1.5]:
		help_panel.scale = Vector2.ONE * factor
		help_panel.size.y = 4000
		help_panel.center_in_viewport()
		await process_frame
		await process_frame
		var bounds: Rect2 = Rect2(help_panel.position, help_panel.size * help_panel.scale)
		_check(root.get_visible_rect().encloses(bounds), "help bounds fit viewport at %s scale" % factor)
		_check(help_panel.size.y <= 680, "help resets inflated content height at %s scale" % factor)
		_check(help_panel._scroll.get_v_scroll_bar().max_value > help_panel._scroll.get_v_scroll_bar().page, "full help table scrolls at %s scale" % factor)
	help_panel.toggle_panel()
	_check(not help_panel.visible, "help closes")
	var goals: Goals = Goals.new()
	root.add_child(goals)
	goals.update_facts({"stockpile_exists": true, "era_progress_permille": 450})
	_check(goals.goal_complete("stockpile"), "goal from real fact")
	_check(not goals.goal_complete("house"), "unmet goal remains open")
	_check(goals._progress.value == 450, "era progress from supplied value")
	var settings: Settings = Settings.new()
	settings.setup(store)
	root.add_child(settings)
	settings.toggle_panel()
	_check(settings.visible, "settings opens")
	for key: String in settings.BOOLEAN_KEYS:
		var check: CheckButton = settings._checks[key]
		check.mouse_entered.emit()
		_check(check.tooltip_text.contains(catalog.tooltip("ui_activate")), "setting tooltip uses shared input: " + key)
	settings._scale.mouse_entered.emit()
	_check(settings._scale.tooltip_text.contains(catalog.tooltip("ui_activate")), "scale tooltip uses shared input")
	settings._scale_selected(2)
	_check(store.value("ui_scale") == 1.5, "settings changes shared store")
	settings._toggle_setting(true, "edge_pan")
	_check(store.value("edge_pan") == true, "settings toggles edge pan")
	for node: Node in [helper, help_panel, goals, settings]:
		node.queue_free()
	await process_frame
	DirAccess.remove_absolute(ProjectSettings.globalize_path(path))
	DirAccess.remove_absolute(ProjectSettings.globalize_path(data_path))
	print("MUX_LEARNING_DONE checks=%s failures=%s" % [checks, failures])
	quit(1 if failures > 0 else 0)

func _owner_settings_state() -> Dictionary:
	var path: String = "user://settings.cfg"
	var exists: bool = FileAccess.file_exists(path)
	return {"exists": exists, "sha256": FileAccess.get_sha256(path) if exists else "", "modified": FileAccess.get_modified_time(path) if exists else 0}

func _test_settings_isolation() -> void:
	var owner_before: Dictionary = _owner_settings_state()
	var had_override: bool = OS.has_environment("WORLDSIM_TEST_SETTINGS_DIR")
	var previous_override: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	var directory: String = "/tmp/worldsim-mux-settings-%d-%d" % [OS.get_process_id(), Time.get_ticks_usec()]
	var created: Error = DirAccess.make_dir_recursive_absolute(directory)
	_check(created == OK, "isolated settings directory created")
	if created != OK:
		return
	OS.unset_environment("WORLDSIM_TEST_SETTINGS_DIR")
	_check(Store.resolve_path() == "user://settings.cfg", "production settings path unchanged without constructing owner store")
	OS.set_environment("WORLDSIM_TEST_SETTINGS_DIR", directory)
	var store: Store = Store.new()
	var expected_path: String = directory.path_join("settings-%d.cfg" % OS.get_process_id())
	_check(store.path == expected_path and store.last_error == OK, "default test store uses external per-process path")
	_check(store.value("steward_default") == true, "isolated settings preserve production steward ON default")
	_check(store.set_value("ui_scale", 1.25) and store.last_error == OK, "default test store persists settings")
	_check(FileAccess.file_exists(expected_path) and Store.new().value("ui_scale") == 1.25, "default test store reloads isolated settings")
	var progress_error: Error = store.save_progress(PackedStringArray(["camera"]), PackedStringArray(), {})
	_check(progress_error == OK and Store.new().progress()["completed"].has("camera"), "learning progress persists only in isolated store")
	var explicit_path: String = directory.path_join("explicit.cfg")
	var explicit: Store = Store.new(explicit_path)
	_check(explicit.path == explicit_path and explicit.value("ui_scale") == 1.0, "explicit constructor path wins without inheriting default test settings")
	_check(explicit.set_value("ui_scale", 1.5) and Store.new(explicit_path).value("ui_scale") == 1.5, "explicit path persists independently")
	_check(Store.new().value("ui_scale") == 1.25, "explicit store cannot overwrite default test store")
	OS.set_environment("WORLDSIM_TEST_SETTINGS_DIR", "relative-settings-path")
	var invalid: Store = Store.new()
	_check(invalid.path.is_empty() and invalid.last_error == ERR_INVALID_PARAMETER, "invalid test override cannot fall back to owner settings")
	if had_override:
		OS.set_environment("WORLDSIM_TEST_SETTINGS_DIR", previous_override)
	else:
		OS.unset_environment("WORLDSIM_TEST_SETTINGS_DIR")
	_check(_owner_settings_state() == owner_before, "owner settings presence bytes and modification time unchanged")
	_check(DirAccess.remove_absolute(expected_path) == OK, "isolated default settings cleaned up")
	_check(DirAccess.remove_absolute(explicit_path) == OK, "isolated explicit settings cleaned up")
	_check(DirAccess.remove_absolute(directory) == OK, "isolated settings directory cleaned up")

func _test_observed_learning() -> void:
	var path: String = "user://mux-observed-test-%s.cfg" % OS.get_process_id()
	var store: Store = Store.new(path)
	var helper: Helper = Helper.new()
	helper.setup(store)
	root.add_child(helper)
	await process_frame
	helper.update_facts({"chop_marked": false, "steward_accepted_rules": [2]})
	_check(not store.progress()["observed"].has("chop"), "accepted rule alone never latches completion")
	helper.update_facts({"chop_marked": true, "steward_accepted_rules": [2]})
	_check(store.progress()["observed"].has("chop"), "real satisfaction observed before lesson is retained")
	_check(not store.progress()["completed"].has("chop"), "observation never presses Next")
	helper.update_facts({"chop_marked": false, "steward_accepted_rules": []})
	for _index: int in 4:
		helper.skip_current()
	_check(helper.current_id() == "chop" and helper.can_advance(), "finished marks disappearing cannot revoke Next")
	_check(helper._body.text == Catalog._text("MUX_LESSON_CHOP_STEWARD", Callable()), "observed steward explanation survives disappearing marks")
	var catalog: Catalog = Catalog.new()
	for button: Button in [helper._next, helper._skip, helper._off]:
		button.mouse_entered.emit()
		_check(button.tooltip_text.contains(catalog.tooltip("ui_activate")), "lesson button tooltip uses shared input: " + str(button.name))
	helper.queue_free()
	await process_frame
	var resumed: Helper = Helper.new()
	resumed.setup(Store.new(path))
	root.add_child(resumed)
	await process_frame
	resumed.update_facts({})
	_check(resumed.current_id() == "chop" and resumed.can_advance(), "observed training survives restart and different world facts")
	_check(resumed._body.text == Catalog._text("MUX_LESSON_CHOP_STEWARD", Callable()), "steward observation survives settings reload")
	resumed.advance()
	_check(Store.new(path).progress()["completed"].has("chop"), "explicit Next records completed progress after restart")
	_check(not resumed.can_advance(), "unobserved housing does not inherit chop completion")
	resumed.queue_free()
	await process_frame
	DirAccess.remove_absolute(ProjectSettings.globalize_path(path))

func _test_facts() -> void:
	var people: Dictionary = {"ids": [1], "bed_xs": [3], "bed_ys": [4], "roofed": [0]}
	var rooms: Dictionary = {"ids": [7], "roofed": [1], "tile_xs": [3], "tile_ys": [4], "tile_rooms": [7]}
	_check(Facts.beds_ready(people, rooms), "bed roof uses assigned bed not person current position")
	rooms["roofed"] = [0]
	people["roofed"] = [1]
	_check(not Facts.beds_ready(people, rooms), "person under roof does not prove roofed bed")
	_check(not Facts.beds_ready({}, {}), "empty population cannot complete housing")
	var winter: Dictionary = {"season": 2, "season_day": 1, "days_per_season": 12, "story": {"firewood": 20}, "settlements": {"ids": [1], "wood": [999], "winter_wood_need": [20.0]}}
	_check(not Facts.winter_wood_ready(winter), "one daily ration cannot prepare a multi-day winter")
	_check(not Facts.from_snapshots(winter, {})["winter_firewood_ready"], "goal cannot substitute settlement stock for usable whole-winter fuel")
	winter["story"]["firewood"] = 240
	winter["settlements"]["wood"] = [0]
	_check(Facts.winter_wood_ready(winter), "exact whole-winter usable story fuel suffices despite settlement wood")
	_check(Facts.from_snapshots(winter, {})["winter_firewood_ready"], "goal fact receives full calendar and usable supply")
	winter["settlements"] = {"ids": [1, 2], "winter_wood_need": [20.0, 21.0]}
	_check(not Facts.winter_wood_ready(winter), "each settlement needs its whole-winter requirement")
	winter["story"]["firewood"] = 491
	_check(not Facts.winter_wood_ready(winter), "one unit below combined settlement requirement is insufficient")
	winter["story"]["firewood"] = 492
	_check(Facts.winter_wood_ready(winter), "exact summed settlement winter requirement suffices")
	winter["settlements"] = {"ids": [1], "winter_wood_need": [0.0]}
	_check(not Facts.winter_wood_ready(winter), "zero winter requirement does not award preparation")
	winter["settlements"] = {"ids": [1]}
	_check(not Facts.winter_wood_ready(winter), "missing winter requirement is unknown not complete")
	winter["settlements"] = {"ids": [1], "winter_wood_need": [20.0]}
	winter["story"] = {}
	_check(not Facts.winter_wood_ready(winter), "missing usable supply does not award preparation")
	winter["story"] = {"firewood": 240}
	winter.erase("season")
	_check(not Facts.winter_wood_ready(winter), "missing calendar does not award preparation")
	winter["season"] = 3
	winter["season_day"] = 10
	winter["story"]["firewood"] = 60
	_check(Facts.winter_wood_ready(winter), "winter remaining days include current day like alert")
	winter["story"]["firewood"] = 59
	_check(not Facts.winter_wood_ready(winter), "one unit below remaining-winter requirement is insufficient")
	winter["days_per_season"] = 0
	_check(not Facts.winter_wood_ready(winter), "zero winter duration does not award preparation")
	_check(not Facts.winter_wood_ready({}), "missing settlement cannot complete winter preparation")
	var bulk: Dictionary = {"people": people, "taught_count": 1, "story": {"wolves": 1}, "stockpile_tiles": {"zones": [8]}, "first_winter_all_alive": true}
	var facts: Dictionary = Facts.from_snapshots(bulk, {"designation_kinds": [0]})
	_check(facts["stockpile_exists"] and facts["chop_marked"], "actual stockpile and chop designation facts")
	_check(facts["taught_technique"] and facts["wolves_seen"], "teaching tally and present wolves facts")
	_check(facts["first_winter_all_alive"], "historical winter fact comes from bridge")
	_check(facts["steward_accepted_rules"].is_empty(), "state never implies steward attribution")
	bulk["deaths_combat"] = 1
	facts = Facts.from_snapshots(bulk, {})
	_check(facts["first_winter_all_alive"], "later deaths do not override historical bridge fact")
	bulk.erase("first_winter_all_alive")
	facts = Facts.from_snapshots(bulk, {})
	_check(not facts["first_winter_all_alive"], "missing historical fact cannot prove first winter survival")
	var markers: Array[Dictionary] = Facts.era_markers({"marker_weights": [0, 10], "statuses": [3, 3], "names": ["a", "new data node"], "name_keys": ["A", "NEW_NODE"]})
	_check(markers.size() == 1 and markers[0]["name_key"] == "NEW_NODE" and markers[0]["done"], "era marker comes from registry rows")

func _test_highlight() -> void:
	var ui: Control = Control.new()
	root.add_child(ui)
	var button: Button = Button.new()
	button.name = "PrioritiesButton"
	button.position = Vector2(20, 20)
	button.size = Vector2(120, 40)
	ui.add_child(button)
	var highlight: Highlight = Highlight.new()
	highlight.setup(ui)
	ui.add_child(highlight)
	await process_frame
	_check(highlight.mouse_filter == Control.MOUSE_FILTER_IGNORE, "highlight never captures input")
	_check(highlight.show_target("priorities"), "actual named launcher resolves")
	_check(highlight.resolved_path == button.get_path(), "highlight identifies actual control")
	button.hide()
	_check(not highlight.refresh_target(), "hidden control cannot be highlighted")
	button.show()
	button.set_meta("mux_lesson_target", "data_added_target")
	_check(highlight.show_target("data_added_target"), "metadata target resolves without new code branch")
	_check(not highlight.show_target("unknown_target"), "unknown target does not fake resolution")
	_check(not highlight.show_target(""), "empty target clears highlight")
	var blocker: PanelContainer = PanelContainer.new()
	blocker.position = Vector2(200, 200)
	blocker.size = Vector2(120, 120)
	ui.add_child(blocker)
	_check(not highlight._map_visible(Rect2(220, 220, 24, 48)), "map entity behind panel is excluded")
	blocker.hide()
	_check(highlight._map_visible(Rect2(220, 220, 24, 48)), "same visible map entity resolves when panel hides")
	ui.queue_free()
	await process_frame
