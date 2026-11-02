# main_bootstrap.gd — selected content preflight before gameplay children enter the tree.
# Bridge methods: WorldSimNode.bootstrap_catalog, WorldSimNode.create.
# Optional gate boot observer delegates read-only native evidence to the tooling helper.
extends Node2D

var boot_ok: bool = false
var boot_error: String = ""
var _gate_boot_observer: RefCounted


func _enter_tree() -> void:
	if boot_ok:
		return
	var receipt: Dictionary = WorldSimNode.bootstrap_catalog()
	if not bool(receipt.get("ok", false)):
		_fail(str(receipt.get("error", "Content bootstrap failed")))
		return
	var world: WorldSimNode = WorldSimNode.create()
	if world == null:
		_fail("WorldSim factory rejected content bootstrap")
		return
	world.name = "WorldSim"
	add_child(world)
	move_child(world, 0)
	boot_ok = true
	# Observe the actual plain-boot process; script-driven producers own their own receipts.
	if OS.get_environment("WORLDSIM_GATE_OBSERVER") == "1" and OS.get_environment("WORLDSIM_GATE_UI_BOOT") == "1" and not "--script" in OS.get_cmdline_args() and not "-s" in OS.get_cmdline_args():
		var observer_script: GDScript = load("res://tools/v8/gate_ui_observer.gd") as GDScript
		_gate_boot_observer = observer_script.new("smoke.boot", "res://scripts/core/main_bootstrap.gd")
		_gate_boot_observer.capture(self, "bootstrap")
		world.tree_exiting.connect(_gate_boot_terminal)


func _gate_boot_terminal() -> void:
	if _gate_boot_observer != null:
		_gate_boot_observer.shutdown(self)


func _fail(message: String) -> void:
	boot_error = message
	for child: Node in get_children():
		remove_child(child)
		child.queue_free()
	var error_label: Label = Label.new()
	error_label.text = "WorldSim startup failed\n" + message
	error_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	error_label.position = Vector2(24, 24)
	error_label.size = Vector2(900, 300)
	add_child(error_label)
	push_error(message)
