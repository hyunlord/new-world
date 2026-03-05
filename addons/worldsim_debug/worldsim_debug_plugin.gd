@tool
class_name WorldSimDebugPlugin
extends EditorPlugin

var _dock: Control

func _enter_tree() -> void:
	_dock = preload("res://addons/worldsim_debug/editor_debug_dock.gd").new()
	add_control_to_bottom_panel(_dock, "WorldSim Debug")

func _exit_tree() -> void:
	if is_instance_valid(_dock):
		remove_control_from_bottom_panel(_dock)
		_dock.queue_free()
	_dock = null
