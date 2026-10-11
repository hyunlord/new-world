extends SceneTree
const Inspector = preload("res://scripts/ui/panels/agent_inspector_panel.gd")
var failures: int = 0
class SnapshotWorld extends Node:
	var snapshot: Dictionary = {}
	func get_agent_snapshot() -> Dictionary: return snapshot

func _initialize() -> void:
	call_deferred("run")

func check(value: bool, message: String) -> void:
	if not value:
		failures += 1
		push_error(message)

func run() -> void:
	var panel: Control = Inspector.new()
	root.add_child(panel)
	panel.set_process(false)
	var world: SnapshotWorld = SnapshotWorld.new()
	panel._world_sim = world
	world.snapshot = {"agent_ids": PackedInt64Array([11, 22]), "colds": PackedByteArray([1, 0])}
	var locale: Node = root.get_node("Locale")
	var original: String = str(locale.get("_current_lang"))
	for lang: String in ["en", "ko"]:
		locale.call("set_language", lang)
		panel.display_agent({"found": true, "agent_id": 11})
		check(panel._cold_label.visible and panel._cold_label.text == ("Cold — the fire is out" if lang == "en" else "불이 꺼져 추위에 떨고 있음"), "cold line localized")
		panel.display_agent({"found": true, "agent_id": 22})
		check(not panel._cold_label.visible, "warm hides line")
	for invalid: Variant in [null, [1], PackedInt32Array([1]), PackedByteArray()]:
		world.snapshot["colds"] = invalid
		panel.display_agent({"found": true, "agent_id": 11})
		check(not panel._cold_label.visible, "invalid or short optional cold column hides line")
	world.snapshot.erase("colds")
	panel.display_agent({"found": true, "agent_id": 11})
	check(not panel._cold_label.visible, "legacy snapshot unchanged")
	locale.call("set_language", original)
	panel.free()
	world.free()
	print("INSPECTOR_COLD failures=", failures)
	quit(1 if failures else 0)
