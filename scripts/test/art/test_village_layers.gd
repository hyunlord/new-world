extends SceneTree
const Selection = preload("res://scripts/core/settlement_selection.gd")
var failures: int = 0
class Partial extends Node:
	var enabled: bool = true
	func set_village_layers_enabled(value: bool) -> void:
		enabled = value
func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)
func _initialize() -> void:
	call_deferred("run")
func run() -> void:
	Selection.reset()
	var shown := Node2D.new()
	root.add_child(shown)
	shown.add_to_group("art_society_layers")
	shown.process_mode = Node.PROCESS_MODE_ALWAYS
	var hidden := Node2D.new()
	root.add_child(hidden)
	hidden.add_to_group("art_society_layers")
	hidden.hide()
	hidden.process_mode = Node.PROCESS_MODE_DISABLED
	var unrelated := Node2D.new()
	root.add_child(unrelated)
	var inspector := Partial.new()
	root.add_child(inspector)
	inspector.add_to_group("art_society_inspectors")
	Selection.set_layers_enabled(self, false)
	check(not Selection.layers_enabled and not shown.visible and shown.process_mode == Node.PROCESS_MODE_DISABLED, "off hides and disables updates")
	check(not inspector.enabled and inspector.process_mode == Node.PROCESS_MODE_INHERIT, "inspector toggles only village row")
	check(unrelated.visible and unrelated.process_mode == Node.PROCESS_MODE_INHERIT, "unrelated world layers untouched")
	Selection.set_layers_enabled(self, false)
	Selection.set_layers_enabled(self, true)
	check(shown.visible and shown.process_mode == Node.PROCESS_MODE_ALWAYS, "original enabled state restored after repeated off")
	check(not hidden.visible and hidden.process_mode == Node.PROCESS_MODE_DISABLED, "originally hidden and disabled remains so")
	check(inspector.enabled, "inspector restored")
	Selection.set_layers_enabled(self, false)
	hidden.free()
	Selection.reset()
	check(Selection.layers_enabled and shown.visible and shown.process_mode == Node.PROCESS_MODE_ALWAYS, "world reset restores existing nodes and ignores freed nodes")
	shown.free()
	unrelated.free()
	inspector.free()
	print("VILLAGE_LAYERS failures=", failures)
	quit(1 if failures else 0)
