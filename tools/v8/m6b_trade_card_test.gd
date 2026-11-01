# Trader card opens native terms without submitting an exchange.
extends SceneTree

var failures: int = 0

func _init() -> void:
	call_deferred("_run")

func check(label: String, ok: bool) -> void:
	print("M6B_TRADE_CARD %s %s" % ["PASS" if ok else "FAIL", label])
	if not ok:
		failures += 1

func frames() -> void:
	for _i: int in 8:
		await process_frame

func find_button(node: Node, label: String) -> Button:
	if node is Button and (node as Button).text.replace("\u2060", "").begins_with(label.replace("\u2060", "")):
		return node as Button
	for child: Node in node.get_children():
		var found: Button = find_button(child, label)
		if found != null:
			return found
	return null

func motion(point: Vector2) -> void:
	var event: InputEventMouseMotion = InputEventMouseMotion.new()
	event.position = point
	Input.parse_input_event(event)
	await frames()

func click(button: Button) -> void:
	var point: Vector2 = button.get_global_transform_with_canvas() * (button.size * 0.5)
	await motion(point)
	check("actual click hits rendered recommended button", root.gui_get_hovered_control() == button)
	for pressed: bool in [true, false]:
		var event: InputEventMouseButton = InputEventMouseButton.new()
		event.button_index = MOUSE_BUTTON_LEFT
		event.pressed = pressed
		event.position = point
		Input.parse_input_event(event)
		await process_frame
	await frames()

func capture(name: String) -> void:
	await RenderingServer.frame_post_draw
	var directory: String = OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR")
	check("capture " + name, root.get_texture().get_image().save_png(directory.path_join(name + ".png")) == OK)

func _run() -> void:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	main.get_node("Camera2D").call("set_paused", true)
	for _i: int in 3:
		await process_frame
	var fixture: String = OS.get_environment("M6B_TRADE_CARD_SAVE")
	if not fixture.is_empty():
		check("load real caravan fixture", world.call("load_world", fixture))
	var cards: Node = main.get_node("UI/MuxHud/CardPanel")
	var trade: Control = main.get_node("UI/TradePanel")
	var definition: Dictionary = {}
	for entry: Dictionary in cards.get("_card_defs"):
		if str(entry.get("id")) == "trader":
			definition = entry
	var caravan: int = -1
	for interval: int in 720:
		var neighbours: Dictionary = world.call("get_neighbours")
		caravan = cards.call("_visitor_id", neighbours, "caravan")
		if caravan >= 0:
			break
		world.call("advance_ticks", 60)
		await process_frame
	check("live caravan fixture exists", caravan >= 0)
	if caravan < 0:
		quit(1)
		return
	for _i: int in 3:
		await process_frame
	var tick: Dictionary = world.call("get_time_snapshot")
	print("M6B_TRADE_FIXTURE " + JSON.stringify({"tick": tick, "caravan": caravan}))
	var commands: Dictionary = world.call("get_command_log", 0)
	var option: int = -1
	var options: Array = definition.get("options", [])
	for index: int in options.size():
		if bool(options[index].get("recommended", false)):
			option = index
	check("authored recommended option exists", option >= 0)
	var hud: Control = main.get_node("UI/MuxHud")
	var store: RefCounted = hud.get("store")
	var label: String = str(root.get_node("Locale").call("ltr", str(options[option]["text_key"])))
	for factor: float in [1.0, 1.25]:
		store.call("set_value", "ui_scale", factor)
		var card: Dictionary = {"def": definition, "trigger_data": {"caravan_id": caravan}, "answered": false}
		cards.set("_active", {"trader": card})
		cards.call("_rebuild_ui")
		await frames()
		await RenderingServer.frame_post_draw
		var button: Button = find_button(cards, label)
		check("visible labeled recommended button %.2f" % factor, button != null and button.is_visible_in_tree())
		await capture("before-%.2f" % factor)
		if button != null:
			await click(button)
		check("recommended click submits no command", world.call("get_command_log", 0) == commands)
		check("recommended click opens pinned caravan", trade.is_visible_in_tree() and int(trade.call("shown_caravan")) == caravan)
		check("opening terms clears card", not cards.get("_active").has("trader"))
		check("review starts with no selected terms", (trade.get("_give") as Dictionary).is_empty() and (trade.get("_take") as Dictionary).is_empty())
		var close_button: Button = trade.get("_close") as Button
		await motion(close_button.get_global_transform_with_canvas() * (close_button.size * 0.5))
		check("trade pane close control unobstructed", root.gui_get_hovered_control() == close_button)
		check("trade pane ambient overlays suppressed", not hud.get_node("CardPanel").is_visible_in_tree() and not hud.get_node("Alerts").is_visible_in_tree() and not hud.get_node("LearningHelper").is_visible_in_tree())
		await capture("after-%.2f" % factor)
		check("rendered review preserves tick", world.call("get_time_snapshot") == tick)
		trade.call("close")
		await frames()
	commands = world.call("get_command_log", 0)
	var stale: Dictionary = {"def": definition, "trigger_data": {"caravan_id": 2147483647}, "answered": false}
	cards.set("_active", {"trader": stale})
	cards.call("answer_card", "trader", option)
	check("stale caravan keeps card open", cards.get("_active").has("trader") and not bool(stale.get("answered", false)))
	check("stale caravan shows failure", not str(stale.get("failure_reason", "")).is_empty())
	check("stale caravan never opens another visitor", not trade.is_visible_in_tree())
	check("stale click submits no command", world.call("get_command_log", 0) == commands)
	var missing: Dictionary = {"def": definition, "trigger_data": {}, "answered": false}
	cards.set("_active", {"trader": missing})
	cards.call("answer_card", "trader", option)
	check("missing pinned ID never chooses live caravan", cards.get("_active").has("trader") and not trade.is_visible_in_tree() and world.call("get_command_log", 0) == commands)
	check("clicks preserve tick", world.call("get_time_snapshot") == tick)
	quit(1 if failures else 0)
