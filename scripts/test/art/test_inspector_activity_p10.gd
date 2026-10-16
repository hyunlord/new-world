extends SceneTree

var _failures: int = 0
var _checks: int = 0

class FakeWorld extends Node:
	var activity: Dictionary = {"ids": PackedInt64Array([999, 101]), "kinds": PackedByteArray([6, 0]), "place_x": PackedInt32Array([99, 7]), "place_y": PackedInt32Array([99, 8])}
	var activity_calls: int = 0
	func get_agent_snapshot() -> Dictionary:
		return {"agent_ids": PackedInt64Array([11]), "ids": PackedInt64Array([101])}
	func get_activity_snapshot() -> Dictionary:
		activity_calls += 1
		return activity
	func detail(state: int, target: int) -> Dictionary:
		return {"found": true, "agent_id": 11, "x": 7, "y": 8, "state_tag": state, "target_kind": target, "hunger": 30.0, "thirst": 40.0, "sleep": 50.0}

func _initialize() -> void:
	_run.call_deferred()

func _check(condition: bool, message: String) -> void:
	_checks += 1
	if not condition:
		_failures += 1
		push_error("INSPECTOR_ACTIVITY_P10: " + message)

func _run() -> void:
	var path: String = OS.get_environment("P10_INSPECTOR_DRAFT")
	if path.is_empty():
		path = "res://scripts/ui/panels/agent_inspector_panel.gd"
	var script: Script = load(path) as Script
	var panel: Control = script.new()
	root.add_child(panel)
	panel.set_process(false)
	var world := FakeWorld.new()
	panel.set("_world_sim", world)
	var locale: Node = root.get_node("Locale")
	var language_before: String = str(locale.get("_current_lang"))
	var keys: Array[String] = ["UI_ACTIVITY_TEND_FIRE", "UI_ACTIVITY_CRAFT", "UI_ACTIVITY_REPAIR", "UI_ACTIVITY_HEARTH_CIRCLE", "UI_ACTIVITY_EXPLORE", "UI_ACTIVITY_PLAY", "UI_ACTIVITY_SIT_BY_FIRE"]
	for language: String in ["en", "ko"]:
		locale.call("set_language", language)
		for kind: int in 7:
			world.activity["kinds"] = PackedByteArray([6, kind])
			panel.call("display_agent", world.detail(4, 8))
			var translated: String = str(locale.call("ltr", keys[kind]))
			_check(translated != keys[kind], "activity translation exists")
			_check((panel.get("_label_state") as Label).text == str(locale.call("ltr", "INSPECTOR_STATE_FMT")) % translated, "state4 uses correct entity-bit joined activity")
			var place: String = str(locale.call("ltr", "INSPECTOR_TARGET_ACTIVITY_PLACE_FMT")) % [7, 8]
			_check((panel.get("_label_target") as Label).text == str(locale.call("ltr", "INSPECTOR_TARGET_FMT")) % place, "target8 uses selected place")
		_check(world.detail(4, 8).size() == 9, "detail schema remains nine fields")
		var calls: int = world.activity_calls
		for state: int in 4:
			panel.call("display_agent", world.detail(state, 0))
			_check(panel.call("_state_text", state, {}) == locale.call("ltr", ["UI_ACTION_IDLE", "INSPECTOR_STATE_SEEKING", "UI_AGENT_STATE_SOCIALIZING", "INSPECTOR_STATE_WORKING"][state]), "old state label preserved")
		for target: int in 8:
			panel.call("display_agent", world.detail(0, target))
			_check(panel.call("_target_text", target, {}) == locale.call("ltr", ["INSPECTOR_TARGET_NONE","INSPECTOR_TARGET_FOOD","INSPECTOR_TARGET_WATER","INSPECTOR_TARGET_SLEEP","INSPECTOR_TARGET_BUILD","INSPECTOR_TARGET_AGENT","INSPECTOR_TARGET_FOOD","INSPECTOR_TARGET_WOOD",][target]), "old target label preserved")
		_check(world.activity_calls == calls, "ordinary state and target do not request activities")
		for tag: int in [-1, 99]:
			_check(panel.call("_state_text", tag, {}) == locale.call("ltr", "UI_UNKNOWN"), "unknown state safe")
			_check(panel.call("_target_text", tag, {}) == locale.call("ltr", "UI_UNKNOWN"), "unknown target safe")
		_check(panel.call("_state_text", 4, {"kind": 99}) == locale.call("ltr", "UI_UNKNOWN"), "unknown activity safe")
		_check(panel.call("_target_text", 8, {}) == locale.call("ltr", "UI_UNKNOWN"), "missing activity place safe")
	world.activity["ids"] = PackedInt64Array([999])
	panel.call("display_agent", world.detail(4, 8))
	_check(panel.call("_selected_activity").is_empty(), "unrelated agent never used as fallback")
	world.activity["ids"] = PackedInt64Array([101])
	world.activity["place_y"] = PackedInt32Array()
	_check(panel.call("_selected_activity").is_empty(), "truncated columns safe")
	panel.call("close")
	var calls: int = world.activity_calls
	panel.call("_process", 1.0 / 60.0)
	_check(world.activity_calls == calls, "closed panel does not poll activity")
	locale.call("set_language", language_before)
	panel.free()
	world.free()
	print("INSPECTOR_ACTIVITY_P10 checks=", _checks, " failures=", _failures)
	quit(0 if _failures == 0 else 1)
