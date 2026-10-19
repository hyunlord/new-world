extends SceneTree
const PanelScript = preload("res://scripts/ui/panels/agent_inspector_panel.gd")
const Fixture = preload("res://scripts/test/art/test_inspector_life_p11.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
var checks: int = 0
var failures: int = 0

class World extends Fixture.FakeWorld:
	var stage: int = 0
	var kind: int = 1
	var active: bool = true
	func get_porter_snapshot() -> Dictionary:
		return {"ids": PackedInt64Array([202, 101]) if active else PackedInt64Array(), "from": PackedInt32Array([1, 2]), "to": PackedInt32Array([2, 1]), "kind": PackedByteArray([0, kind]), "amount": PackedInt32Array([99, 12]), "stage": PackedByteArray([0, stage])}

func _initialize() -> void:
	_run.call_deferred()

func check(ok: bool, message: String) -> void:
	checks += 1
	if not ok:
		failures += 1
		push_error(message)

func _run() -> void:
	var panel: Control = PanelScript.new()
	root.add_child(panel)
	panel.set_process(false)
	var world := World.new()
	panel.set("_world_sim", world)
	var locale: Node = root.get_node("Locale")
	var original: String = str(locale.get("_current_lang"))
	Selection._numbers = {1: 1, 2: 2}
	Selection._history = {1: {"parent_id": -1}, 2: {"parent_id": -1}}
	panel.call("show_person", 1)
	var status: Label = panel.get("_porter_status") as Label
	check(status != null, "porter status exists under the name")
	if status != null:
		check(status.get_index() == (panel.get("_label_id") as Label).get_index() + 1, "status immediately follows name")
		for language: String in ["en", "ko"]:
			locale.call("set_language", language)
			for kind: int in [0, 1]:
				world.kind = kind
				world.stage = 0
				world.tick += 1
				panel.call("show_person", 1)
				var load_name: String = str(locale.call("ltr", "CHRON_LOAD_FOOD" if kind == 0 else "CHRON_LOAD_WOOD"))
				var expected: String = str(locale.call("ltr", "INSPECTOR_PORTER_OUT_FMT")) % [load_name, 12, Selection.name_for(1, locale)]
				check(status.visible and Names.plain(status.text) == expected, "outbound entity join and authoritative amount " + language)
				check(Names.plain((panel.get("_label_target") as Label).text) == str(locale.call("ltr", "INSPECTOR_TARGET_FMT")) % Selection.name_for(1, locale), "outbound target is receiver hearth rather than internal self-target")
				world.stage = 1
				world.tick += 1
				panel.call("_process", 0.0)
				expected = str(locale.call("ltr", "INSPECTOR_PORTER_BACK_FMT")) % Selection.name_for(2, locale)
				check(status.visible and Names.plain(status.text) == expected, "return sender " + language)
				check(Names.plain((panel.get("_label_target") as Label).text) == str(locale.call("ltr", "INSPECTOR_TARGET_FMT")) % Selection.name_for(2, locale), "return target is sender hearth")
			panel.call("show_person", 3)
			check(not status.visible, "record hides porter status")
			world.active = false
			world.tick += 1
			panel.call("show_person", 1)
			check(not status.visible and status.text.is_empty(), "nonporter clears stale text")
			check(Names.plain((panel.get("_label_target") as Label).text) == str(locale.call("ltr", "INSPECTOR_TARGET_FMT")) % str(locale.call("ltr", "INSPECTOR_TARGET_NONE")), "nonporter restores ordinary target")
			world.active = true
			panel.call("show_person", 1)
			panel.call("_world_reset")
			check(not panel.visible, "world reset hides status panel")
	locale.call("set_language", original)
	Selection.reset()
	panel.free()
	world.free()
	print("INSPECTOR_PORTER_P12 checks=", checks, " failures=", failures)
	quit(0 if failures == 0 else 1)
