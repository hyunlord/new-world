extends SceneTree

const Inspector = preload("res://scripts/ui/panels/agent_inspector_panel.gd")
var _failures: int = 0

class SnapshotWorld extends Node:
	var snapshot: Dictionary = {}
	func get_agent_snapshot() -> Dictionary:
		return snapshot

func _initialize() -> void:
	call_deferred("_run")

func _check(condition: bool, message: String) -> void:
	if not condition:
		_failures += 1
		push_error(message)

func _run() -> void:
	var panel: Control = Inspector.new()
	root.add_child(panel)
	var world: SnapshotWorld = SnapshotWorld.new()
	panel.set("_world_sim", world)
	var locale: Node = root.get_node("Locale")
	var original_language: String = str(locale.get("_current_lang"))
	world.snapshot = {
		"ids": PackedInt64Array([101, 102, 103]),
		"agent_ids": PackedInt64Array([11, 22, 33]),
		"life_stages": PackedByteArray([1, 0, 2]),
		"ages_days": PackedInt32Array([2, 20, 40]),
	}
	for language: String in ["en", "ko"]:
		locale.call("set_language", language)
		var expected: Array = ["Child · Age 2 days", "Adult · Age 20 days", "Elder · Age 40 days"] if language == "en" else ["아이 · 나이 2일", "어른 · 나이 20일", "노인 · 나이 40일"]
		for index: int in 3:
			panel.call("display_agent", {"found": true, "agent_id": [11, 22, 33][index]})
			var label: Label = panel.get("_label_age") as Label
			_check(label.visible and label.text == expected[index], "localized stage and age for selected identity")
	_check(panel.call("_age_line", 99) == "", "missing identity skipped")
	world.snapshot["ages_days"] = PackedInt32Array([2, 20, -1])
	panel.call("display_agent", {"found": true, "agent_id": 33})
	_check(not (panel.get("_label_age") as Label).visible, "unknown age hides previous line")
	world.snapshot.erase("ages_days")
	_check(panel.call("_age_line", 11) == "", "absent ages skipped")
	world.snapshot["ages_days"] = PackedInt32Array([2])
	_check(panel.call("_age_line", 33) == "", "short ages skipped")
	world.snapshot.erase("life_stages")
	_check(panel.call("_age_line", 11) == "", "absent stages skipped")
	world.snapshot["life_stages"] = PackedByteArray([0])
	var strings: Dictionary = locale.get("_strings")
	strings.erase("INSPECTOR_AGE_FMT")
	_check(panel.call("_age_line", 11) == "", "missing translation skips unsafe format")
	locale.call("set_language", original_language)
	panel.free()
	world.free()
	print("INSPECTOR_AGE failures=", _failures)
	quit(1 if _failures > 0 else 0)
