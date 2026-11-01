# Stage -1 density regressions. Read-only bridge snapshots; no simulation commands.
# Run with isolated WORLDSIM_TEST_SETTINGS_DIR and bounded headless Godot.
extends SceneTree

const Facts = preload("res://scripts/ui/mux/learning_facts.gd")
const Legacy = preload("res://scripts/ui/mux/legacy_layout.gd")
var _passed: int = 0
var _failed: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(ok: bool, label: String, detail: Variant = "") -> void:
	if ok:
		_passed += 1
	else:
		_failed += 1
	print("DENSITY %s %s %s" % ["PASS" if ok else "FAIL", label, str(detail)])


func _run() -> void:
	root.size = Vector2i(1920, 1080)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	main.process_mode = Node.PROCESS_MODE_DISABLED
	for _frame: int in 4:
		await process_frame
	var world: Node = main.get_node("WorldSim")
	var ui: Node = main.get_node("UI")
	var topbar: Control = ui.get_node("HudTopbar") as Control
	var mux: Control = ui.get_node("MuxHud") as Control
	var before: Dictionary = world.call("get_command_log", 0)
	var before_time: Dictionary = world.call("get_time_snapshot")
	var tech: Dictionary = world.call("get_tech")
	_check(not (tech.get("ids", PackedInt64Array()) as PackedInt64Array).is_empty(), "real_known_tech_fixture")
	for _refresh: int in 3:
		topbar.call("_refresh")
	_check(not (topbar.get("_era_label") as Label).visible, "era_stays_out_of_topbar_after_refresh")
	_check(not (topbar.get("_risk_button") as Button).visible, "risk_stays_out_of_topbar_after_refresh")
	_check(not (topbar.get("_lower_row") as Control).visible, "birth_death_row_stays_hidden")
	_check(not str(topbar.call("era_text")).is_empty() and not str(topbar.call("risk_text")).is_empty(), "legacy_knowledge_text_accessors_retained")
	_check(int(tech.get("at_risk", 0)) == 0 or not str(topbar.call("warning_text")).is_empty(), "legacy_risk_warning_text_retained")
	var layout: Node = mux.get("_legacy") as Node
	var panel: Control = Legacy._first_panel(topbar)
	for factor: float in [0.85, 1.0, 1.15, 1.25]:
		layout.call("apply_scale", factor, Vector2(1920, 1080))
		await process_frame
		var report: Dictionary = layout.call("layout_report")
		var row: Dictionary = report.get("HudTopbar", {})
		_check(is_equal_approx(float(row.get("requested", 0.0)), factor), "requested_scale_%s" % factor, row)
		_check(is_equal_approx(float(row.get("effective", 0.0)), factor), "effective_scale_%s" % factor, row)
		_check(is_equal_approx(panel.size.y, panel.get_combined_minimum_size().y) and panel.size.y < 100.0, "compact_row_%s" % factor, {"height": panel.size.y, "minimum": panel.get_combined_minimum_size().y})
	var native_tallies: Dictionary = world.call("get_tallies")
	var actual_facts: Dictionary = Facts.collect(world)
	var expected_deaths: int = 0
	for key: String in ["deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age", "deaths_exposure"]:
		expected_deaths += int(native_tallies.get(key, 0))
	_check(int(actual_facts.get("births", -1)) == int(native_tallies.get("births", 0)), "real_births_collected")
	_check(int(actual_facts.get("deaths", -1)) == expected_deaths, "real_deaths_collected")
	_check(int(actual_facts.get("tech_at_risk", -1)) == int(tech.get("at_risk", 0)), "real_risk_collected")
	var nonzero: Dictionary = Facts.from_snapshots({"tech": {"progress": 643, "at_risk": 3}}, {}, {}, {"births": 7, "deaths_starvation": 1, "deaths_dehydration": 2, "deaths_combat": 3, "deaths_old_age": 4, "deaths_exposure": 5})
	_check(int(nonzero.get("births", -1)) == 7 and int(nonzero.get("deaths", -1)) == 15 and int(nonzero.get("tech_at_risk", -1)) == 3, "nonzero_goal_facts", nonzero)
	var goals: Control = mux.get("goals") as Control
	goals.call("update_facts", nonzero)
	goals.call("toggle_panel")
	_check(goals.is_visible_in_tree(), "goals_are_accessible")
	for entry: Array in [["_era", "643/1000"], ["_births_label", "7"], ["_deaths_label", "15"], ["_tech_risk_label", "3"]]:
		var label: Label = goals.get(str(entry[0])) as Label
		_check(label.is_visible_in_tree() and label.text.contains(str(entry[1])), "goals_visible_" + str(entry[0]), label.text)
	goals.call("update_facts", actual_facts)
	_check(world.call("get_command_log", 0) == before, "density_observation_emits_no_commands")
	_check(world.call("get_time_snapshot") == before_time, "density_observation_does_not_tick")
	print("M6B_DENSITY_RESULT " + JSON.stringify({"passed": _passed, "failed": _failed, "mode": "headless"}))
	main.queue_free()
	await process_frame
	quit(1 if _failed > 0 else 0)
