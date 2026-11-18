# Isolated real command card: localized actions, tooltips and dispatched shortcuts.
# Bridge methods: none; the fake order source only tracks the selected mode.
extends SceneTree

const OrderBar = preload("res://scripts/ui/hud/order_bar.gd")
var _failures: int = 0

class OrderSource extends Node:
	signal mode_changed(mode: int)
	signal order_sent(command: String, result: Dictionary)
	var mode: int = 0
	var build_kind: String = "hut"
	func set_mode(value: int) -> void:
		mode = value
		mode_changed.emit(value)
	func selected_housing_zone() -> int:
		return -1
	func template_active() -> bool:
		return false

func _init() -> void:
	call_deferred("_run")

func _check(label: String, passed: bool) -> void:
	print("M6B_KEY_HELP " + JSON.stringify({"check": label, "pass": passed}))
	if not passed:
		_failures += 1

func _key(code: Key) -> void:
	for down: bool in [true, false]:
		var event: InputEventKey = InputEventKey.new()
		event.keycode = code
		event.pressed = down
		Input.parse_input_event(event)
	await process_frame

func _run() -> void:
	var main: Node = Node.new()
	main.name = "Main"
	root.add_child(main)
	var colony: Node = Node.new()
	colony.name = "ColonyView"
	main.add_child(colony)
	var source: OrderSource = OrderSource.new()
	source.name = "OrderTool"
	source.add_user_signal("field_selected")
	colony.add_child(source)
	var world: Node = Node.new()
	world.name = "WorldSim"
	world.add_user_signal("world_reset")
	main.add_child(world)
	var ui: Control = Control.new()
	ui.name = "UI"
	main.add_child(ui)
	var bar: Control = OrderBar.new()
	ui.add_child(bar)
	bar.set_process(false)
	await process_frame
	var hint: Label = bar.get("_hint") as Label
	_check("idle has no shortcut wall", not hint.visible and hint.text.is_empty())
	var buttons: Array = bar.get("_buttons")
	_check("all action buttons remain accessible", buttons.size() == OrderBar.MODES.size())
	for i: int in buttons.size():
		var button: Button = buttons[i] as Button
		var shortcut: String = OS.get_keycode_string(int(OrderBar.MODES[i]["hotkey"]))
		_check("action %d has a label below its icon and a shortcut tooltip" % i,
			button.is_visible_in_tree() and button.icon != null and button.vertical_icon_alignment == VERTICAL_ALIGNMENT_TOP
			and shortcut not in button.text and shortcut in button.tooltip_text)
	var group_names: PackedStringArray = PackedStringArray()
	for label: Label in bar.get("_group_labels"):
		group_names.append(label.text)
	_check("four localized command headings", group_names.size() == 4 and not "UI1_" in " ".join(group_names))
	await _key(KEY_F1)
	_check("action shortcut retained", source.mode == 1)
	_check("active tool contextual hint retained", hint.visible and not hint.text.is_empty())
	source.set_mode(0)
	_check("idle hides contextual hint", not hint.visible and hint.text.is_empty())
	await _key(KEY_F7)
	_check("gather shortcut opens submenu", bool(bar.call("dig_menu_open")))
	source.order_sent.emit("cmd_designate_quarry_tiles", {"accepted": 1})
	_check("dispatching order closes gather submenu", not bool(bar.call("dig_menu_open")))
	print("M6B_KEY_HELP_DONE failures=%d" % _failures)
	quit(0 if _failures == 0 else 1)
