extends SceneTree

const Inspector = preload("res://scripts/ui/panels/agent_inspector_panel.gd")
var _failures: int = 0

class SnapshotWorld extends Node:
	var snapshot: Dictionary = {}
	func get_agent_snapshot() -> Dictionary: return snapshot

func _initialize() -> void:
	call_deferred("_run")

func _check(condition: bool, message: String) -> void:
	if not condition:
		_failures += 1
		push_error(message)

func _run() -> void:
	var panel: Control = Inspector.new()
	root.add_child(panel)
	panel.set_process(false)
	var world: SnapshotWorld = SnapshotWorld.new()
	panel._world_sim = world
	var locale: Node = root.get_node("Locale")
	var original: String = str(locale.get("_current_lang"))
	world.snapshot = {"ids": PackedInt64Array([101, 102]), "agent_ids": PackedInt64Array([11, 22]), "carried_foods": PackedInt32Array([5, 8]), "carried_woods": PackedInt32Array([3, 0])}
	for lang: String in ["en", "ko"]:
		locale.call("set_language", lang)
		panel.display_agent({"found": true, "agent_id": 11, "target_kind": 7})
		_check(panel._carried_label.text == ("Carrying 3 wood" if lang == "en" else "나무 3개 운반 중"), "localized wood takes precedence over food")
		_check(panel._label_target.text.ends_with("Firewood" if lang == "en" else "땔감"), "target seven localized")
		panel.display_agent({"found": true, "agent_id": 22})
		_check(panel._carried_label.text == locale.ltr("INSPECTOR_CARRYING_FMT") % 8, "zero wood keeps food line")
	for invalid: Variant in [null, [3, 4], PackedByteArray([3, 4]), "wood"]:
		world.snapshot["carried_woods"] = invalid
		panel.display_agent({"found": true, "agent_id": 11})
		_check(panel._carried_label.text == locale.ltr("INSPECTOR_CARRYING_FMT") % 5, "wrong optional type falls back food")
	world.snapshot["carried_woods"] = PackedInt32Array([3])
	panel.display_agent({"found": true, "agent_id": 22})
	_check(panel._carried_label.text == locale.ltr("INSPECTOR_CARRYING_FMT") % 8, "short wood array falls back per row")
	_check(panel._carried_amount(99, "carried_woods") == 0, "unknown identity no wood")
	world.snapshot.erase("carried_woods")
	panel.display_agent({"found": true, "agent_id": 11})
	_check(panel._carried_label.text == locale.ltr("INSPECTOR_CARRYING_FMT") % 5, "missing key legacy line")
	world.snapshot["colds"] = PackedByteArray([1, 0])
	for lang: String in ["en", "ko"]:
		locale.call("set_language", lang)
		panel.display_agent({"found": true, "agent_id": 11})
		_check(panel._cold_label.visible and panel._cold_label.text == locale.ltr("INSPECTOR_COLD"), "localized cold inspector line")
		panel.display_agent({"found": true, "agent_id": 22})
		_check(not panel._cold_label.visible, "warm agent line hidden")
	world.snapshot.erase("colds")
	panel.display_agent({"found": true, "agent_id": 11})
	_check(not panel._cold_label.visible, "legacy inspector no cold line")
	locale.call("set_language", original)
	panel.free()
	world.free()
	print("INSPECTOR_WOOD failures=", _failures)
	quit(1 if _failures else 0)
