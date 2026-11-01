# Windowed foreground regression; uses isolated settings and preserves simulation state.
extends SceneTree

const Store = preload("res://scripts/ui/mux/settings_store.gd")
var failures: int = 0

func _init() -> void:
	call_deferred("_run")

func check(label: String, ok: bool) -> void:
	print("M6B_FOREGROUND %s %s" % ["PASS" if ok else "FAIL", label])
	if not ok:
		failures += 1

func frames() -> void:
	for _i: int in 8:
		await process_frame

func _run() -> void:
	var directory: String = OS.get_environment("M6B_FOREGROUND_EVIDENCE_DIR")
	if not directory.is_absolute_path():
		quit(2)
		return
	DirAccess.make_dir_recursive_absolute(directory)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var hud: Control = main.get_node("UI/MuxHud")
	var store: RefCounted = Store.new(directory.path_join("settings.cfg"))
	hud.set("store", store)
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	main.get_node("Camera2D").call("set_paused", true)
	# Establish an actual founder mind before freezing observer invariants.
	world.call("advance_ticks", 1)
	await frames()
	var cards: Control = hud.get_node("CardPanel")
	cards.call("_poll_triggers")
	await frames()
	var tick: Dictionary = world.call("get_time_snapshot")
	var commands: Dictionary = world.call("get_command_log", 0)
	var state: Dictionary = cards.get("_active").duplicate(true)
	check("fixture has active cards", cards.call("has_active_cards"))
	var panel: Control = main.get_node("UI/PriorityPanel")
	for factor: float in [1.0, 1.25]:
		store.call("set_value", "ui_scale", factor)
		panel.call("open_roles")
		await frames()
		check("cards obscured %.2f" % factor, not cards.is_visible_in_tree())
		check("alerts obscured %.2f" % factor, not hud.get_node("Alerts").is_visible_in_tree())
		check("helper obscured %.2f" % factor, not hud.get_node("LearningHelper").is_visible_in_tree())
		check("map dock obscured", not hud.get_node("MapDock").is_visible_in_tree())
		check("plaque obscured", not main.get_node("UI/TaskPlaque").is_visible_in_tree())
		cards.call("_rebuild_ui")
		check("card rebuild stays obscured", not cards.is_visible_in_tree())
		await RenderingServer.frame_post_draw
		check("screenshot %.2f" % factor, root.get_texture().get_image().save_png(directory.path_join("roles-%.2f.png" % factor)) == OK)
		panel.hide()
		await frames()
		check("cards restored", cards.is_visible_in_tree())
		check("alerts restored", hud.get_node("Alerts").is_visible_in_tree())
		check("card state unchanged", cards.get("_active") == state)
	panel.show()
	await frames()
	cards.set("_active", {})
	cards.call("_rebuild_ui")
	panel.hide()
	await frames()
	check("empty cards stay hidden after close", not cards.is_visible_in_tree())
	cards.set("_active", state.duplicate(true))
	cards.call("_rebuild_ui")
	var inspector: Control = main.get_node("UI/AgentInspectorPanel")
	var people: Dictionary = world.call("get_agent_snapshot")
	var ids: PackedInt64Array = people.get("ids", PackedInt64Array())
	check("founder fixture exists", not ids.is_empty())
	if not ids.is_empty():
		inspector.call("display_agent", world.call("get_agent_detail", ids[0]))
		await frames()
		check("inspector obscures cards", not cards.is_visible_in_tree())
		var why: Button = inspector.get("_why_button") as Button
		why.pressed.emit()
		await frames()
		var causal: Control = main.get_node("UI/CausalPanel")
		check("inspector and actual diagnosis coexist", inspector.is_visible_in_tree() and causal.is_visible_in_tree())
		check("diagnosis obscures ambient overlays", not hud.get_node("LearningHelper").is_visible_in_tree() and not hud.get_node("MapDock").is_visible_in_tree())
		await RenderingServer.frame_post_draw
		check("inspector screenshot", root.get_texture().get_image().save_png(directory.path_join("inspector-diagnosis.png")) == OK)
		inspector.hide()
		await frames()
		check("popup alone keeps cards obscured", not cards.is_visible_in_tree())
		causal.hide()
		await frames()
		check("closing last popup restores cards", cards.is_visible_in_tree())
		check("popup card state unchanged", cards.get("_active") == state)
	check("tick unchanged", world.call("get_time_snapshot") == tick)
	check("commands unchanged", world.call("get_command_log", 0) == commands)
	quit(1 if failures else 0)
