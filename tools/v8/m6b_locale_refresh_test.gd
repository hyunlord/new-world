# Paused locale changes refresh existing presentation without advancing card lifecycle.
extends SceneTree
const KoreanText = preload("res://scripts/ui/mux/korean_text.gd")
var failures: int = 0
func _init() -> void:
	call_deferred("_run")
func check(label: String, ok: bool) -> void:
	print("LOCALE_REFRESH %s %s" % ["PASS" if ok else "FAIL", label])
	if not ok:
		failures += 1
func frames() -> void:
	for _i: int in 10:
		await process_frame
func buttons(node: Node) -> Array[String]:
	var result: Array[String] = []
	if node is Button:
		result.append(KoreanText.raw(node.text))
	for child: Node in node.get_children():
		result.append_array(buttons(child))
	return result
func _run() -> void:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	world.set_process(false)
	world.call("set_sim_speed", 0.0)
	main.get_node("Camera2D").call("set_paused", true)
	world.call("advance_ticks", 1)
	await frames()
	var hud: Node = main.get_node("UI/MuxHud")
	var cards: Node = hud.get_node("CardPanel")
	var plaque: Node = main.get_node("UI/TaskPlaque")
	var locale: Node = root.get_node("Locale")
	var active: Dictionary = {}
	for definition: Dictionary in cards.get("_card_defs"):
		if definition["id"] == "need_food":
			active["need_food"] = {"def": definition, "trigger_data": {}, "opened_tick": 1, "expires_tick": 721, "answered": false, "clear_count": 2}
	cards.set("_active", active)
	# Only fixture construction refreshes explicitly; language transitions below use production signals.
	cards.call("_rebuild_ui")
	plaque.call("refresh_cards")
	await frames()
	var lifecycle: Dictionary = cards.get("_active").duplicate(true)
	var cooldown: Dictionary = cards.get("_cooldown").duplicate(true)
	var tick: Dictionary = world.call("get_time_snapshot")
	var log: Dictionary = world.call("get_command_log", 0)
	var index: int = 0
	for language: String in ["ko", "en", "ko"]:
		locale.call("set_language", language)
		await frames()
		var tag: String = "%d-%s" % [index, language]
		check(tag + " plaque title", KoreanText.raw((plaque.get("_title_label") as Label).text) == str(locale.call("ltr", "LOOP_PLAQUE_TITLE")))
		check(tag + " chapter", KoreanText.raw((plaque.get("_chapter_label") as Label).text) == KoreanText.raw(str(hud.get_node("Goals").call("chapter_line"))))
		check(tag + " plaque body", KoreanText.raw((plaque.get("_text_label") as Label).text) == str(locale.call("ltr", "LOOP_PLAQUE_CARD_FMT")).format({"title": str(locale.call("ltr", "LOOP_CARD_NEED_FOOD"))}))
		check(tag + " existing card button", buttons(cards).has(str(locale.call("ltr", "LOOP_CARD_STEWARD")) + " " + str(locale.call("ltr", "LOOP_CARD_RECOMMENDED"))))
		check(tag + " lifecycle unchanged", cards.get("_active") == lifecycle and cards.get("_cooldown") == cooldown)
		check(tag + " tick unchanged", world.call("get_time_snapshot") == tick)
		check(tag + " log unchanged", world.call("get_command_log", 0) == log)
		await RenderingServer.frame_post_draw
		check(tag + " screenshot", root.get_texture().get_image().save_png(OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR").path_join(tag + ".png")) == OK)
		index += 1
	quit(1 if failures else 0)
