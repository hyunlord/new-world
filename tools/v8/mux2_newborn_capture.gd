# ISS275/#612 Korean newborn alert capture using the real HUD and alert fact boundary.
# Bridge: main scene normal reads only; no fixture simulation mutations or alert clicks.
extends "res://tools/v8/mux2_text_capture.gd"

const AlertCases = preload("res://tools/v8/mux_alerts_test.gd")
var newborn_checks: Array[Dictionary] = []

func run() -> void:
	out = "/Users/rexxa/work/mux2-data/ui-newborn/run-%d" % OS.get_process_id()
	DirAccess.make_dir_recursive_absolute(out)
	print("MUX2_NEWBORN_OUTPUT " + out)
	check("actual GPU renderer required", DisplayServer.get_name() != "headless")
	main = load("res://scenes/main.tscn").instantiate()
	var hud: Node = main.get_node("UI/MuxHud")
	hud.set("store", Store.new(out.path_join("settings.cfg")))
	root.add_child(main)
	await frames(25)
	main.get_node("Camera2D").call("set_paused", true)
	root.get_node("Locale").call("set_language", "ko")
	var panel: Control = hud.get("alerts") as Control
	# Freeze only the panel polling while explicit test facts own its read boundary.
	# The real panel, evaluator, registry, translations, theme and HUD layout remain intact.
	panel.set_process(false)
	var facts: Dictionary = AlertCases._facts()
	facts["households"] = [
		{"id": 100001, "x": 0, "y": 0, "newborns": 2, "beds": 0},
		{"id": 100002, "x": 1, "y": 0, "newborns": 1, "beds": 0}
	]
	for factor: float in [1.0, 1.25]:
		hud.get("store").set_value("ui_scale", factor)
		check("real panel accepts newborn facts", bool(panel.call("refresh_from_facts", facts)))
		await frames(15)
		var shot: String = "newborn-ko-" + str(factor).replace(".", "_")
		var button: Button = panel.find_child("newborn_without_bed", true, false) as Button
		check(shot + " newborn button exists", button != null)
		if button == null: continue
		var text: String = button.text.replace("\u2060", "")
		check(shot + " exact complete Korean title and household count", text == "! 갓난아기 가구에 침상 없음 · 2")
		check(shot + " unclipped WORD wrapping", not button.clip_text and button.autowrap_mode == TextServer.AUTOWRAP_WORD)
		check(shot + " requested actual panel scale", panel.scale.is_equal_approx(Vector2.ONE * factor))
		var shaping: Dictionary = shaped_lines(button, button.text, button.autowrap_mode)
		check(shot + " no midword break", bool(shaping["word_boundaries_intact"]))
		check(shot + " every line fits width", bool(shaping["line_widths_fit"]))
		var font: Font = button.get_theme_font("font")
		var font_size: int = button.get_theme_font_size("font_size")
		var lines: PackedStringArray = shaping["lines"]
		var style: StyleBox = button.get_theme_stylebox("normal")
		var required_height: float = font.get_height(font_size) * lines.size() + button.get_theme_constant("line_spacing") * maxi(0, lines.size() - 1)
		var available_height: float = button.size.y - style.get_minimum_size().y
		check(shot + " all shaped lines fit content height", required_height <= available_height + 1.0)
		var rect: Rect2 = button.get_global_rect()
		var scroll: ScrollContainer = panel.get("_scroll") as ScrollContainer
		check(shot + " whole row inside scroll", scroll.get_global_rect().encloses(rect))
		check(shot + " whole panel inside viewport", root.get_visible_rect().encloses(panel.get_global_rect()))
		var clipped_by: Array[String] = []
		var ancestor: Node = button.get_parent()
		while ancestor != null:
			if ancestor is Control and (ancestor as Control).clip_contents and not (ancestor as Control).get_global_rect().encloses(rect):
				clipped_by.append(str(ancestor.get_path()))
			ancestor = ancestor.get_parent()
		check(shot + " no clipping ancestor", clipped_by.is_empty())
		newborn_checks.append({"shot": shot, "scale": factor, "path": str(button.get_path()), "text": text, "shaping": shaping, "button_rect": str(rect), "scroll_rect": str(scroll.get_global_rect()), "panel_rect": str(panel.get_global_rect()), "required_text_height": required_height, "available_text_height": available_height, "clipped_by": clipped_by, "count_semantics": "2 households, 3 newborns"})
		await capture(shot)
	var file: FileAccess = FileAccess.open(out.path_join("text-nodes.json"), FileAccess.WRITE)
	check("inventory writable", file != null)
	if file != null:
		file.store_string(JSON.stringify({"scope": "synthetic household facts through real Alerts.refresh_from_facts, not live birth evidence", "facts": facts, "newborn_checks": newborn_checks, "nodes": inventory, "failures": failures, "renderer": DisplayServer.get_name(), "visual_verdict": "PENDING_ACTUAL_IMAGE_REVIEW"}, "\t"))
	print("MUX2_NEWBORN_DONE failures=%d nodes=%d" % [failures, inventory.size()])
	quit(1 if failures else 0)
