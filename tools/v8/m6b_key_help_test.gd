# Isolated real OrderBar UI: dispatched keys, order signal and deterministic timer boundaries.
# No WorldSim instance or simulation mutation; elapsed time is injected into the frame callback.
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

func _init() -> void:
	call_deferred("_run")

func _check(label: String, passed: bool) -> void:
	print("M6B_KEY_HELP " + JSON.stringify({"check": label, "pass": passed}))
	if not passed:
		_failures += 1

func _key(code: Key, shift: bool = false) -> void:
	for down: bool in [true, false]:
		var event: InputEventKey = InputEventKey.new()
		event.keycode = code
		event.pressed = down
		event.shift_pressed = shift
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
	colony.add_child(source)
	var ui: Control = Control.new()
	ui.name = "UI"
	main.add_child(ui)
	var bar: Control = OrderBar.new()
	ui.add_child(bar)
	bar.set_process(false)
	await process_frame
	var hint: Label = bar.get("_hint") as Label
	_check("initial help expanded", hint.visible)
	bar.call("_process", 179.999)
	_check("before 180 seconds expanded", hint.visible)
	bar.call("_process", 0.001)
	await process_frame
	_check("at 180 seconds one row", not hint.visible)
	var compact_height: float = (bar.get("_bar") as Control).get_combined_minimum_size().y
	await _key(KEY_SLASH)
	_check("plain slash does not reopen", not hint.visible)
	await _key(KEY_SLASH, true)
	_check("question mark reopens", hint.visible)
	bar.call("_process", 1.0)
	_check("manual reopen survives expired timer", hint.visible)
	await process_frame
	_check("expanded panel has second row", (bar.get("_bar") as Control).get_combined_minimum_size().y > compact_height)
	source.order_sent.emit("cmd_designate_chop_tiles", {"accepted": 1})
	_check("first player order collapses", not hint.visible)
	await _key(KEY_F1)
	_check("action shortcut retained", source.mode == 1)
	_check("active tool contextual hint retained", hint.visible and not hint.text.is_empty())
	source.set_mode(0)
	_check("idle returns to one row", not hint.visible)
	var buttons: Array = bar.get("_buttons")
	_check("all action buttons remain accessible", buttons.size() == OrderBar.MODES.size() and (buttons[0] as Button).is_visible_in_tree())
	await _key(KEY_QUESTION)
	_check("question keycode reopens", hint.visible)
	source.order_sent.emit("cmd_designate_chop_tiles", {"accepted": 1})
	_check("later orders preserve manual reopen", hint.visible)
	print("M6B_KEY_HELP_DONE failures=%d" % _failures)
	quit(0 if _failures == 0 else 1)
