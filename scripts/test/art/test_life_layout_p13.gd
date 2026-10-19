extends SceneTree
const Inspector = preload("res://scripts/ui/panels/agent_inspector_panel.gd")
const Fixture = preload("res://scripts/test/art/test_inspector_life_p11.gd")
const Names = preload("res://scripts/ui/person_names.gd")
var failures: int = 0

func _initialize() -> void:
	_run.call_deferred()

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)

func _run() -> void:
	root.size = Vector2i(1920, 1080)
	var panel: Control = Inspector.new()
	root.add_child(panel)
	panel.set_process(false)
	var world := Fixture.FakeWorld.new()
	panel.set("_world_sim", world)
	var locale: Node = root.get_node("Locale")
	for language: String in ["ko", "en"]:
		locale.call("set_language", language)
		panel.call("show_person", 1)
		var title: Label = panel.get("_label_id") as Label
		check(title.text.contains("\u2060"), "inspector names join across syllables: " + language)
		var buttons: Array = panel.get("_family_buttons")
		var mother: Button = buttons[1] as Button
		check(mother.text.contains("\u2060"), "family name uses joiners: " + language)
		check(mother.text.contains(str(locale.call("ltr", "UI_DECEASED"))), "dead parent retains full status")
		check(mother.text_overrun_behavior == TextServer.OVERRUN_NO_TRIMMING, "family status never ellipsized")
		check(mother.autowrap_mode == TextServer.AUTOWRAP_WORD_SMART, "family status wraps to next line")
		var life: RichTextLabel = panel.get("_life_text") as RichTextLabel
		check(life.get_theme_font("bold_font").resource_path.ends_with("Galmuri11-Bold.ttf"), "Life uses supplied real bold font")
		panel.call("_expand_life")
		await process_frame
		await process_frame
		var scroll: ScrollContainer = panel.get("_scroll") as ScrollContainer
		var box: VBoxContainer = panel.get("_vbox") as VBoxContainer
		var last: Control = box.get_child(box.get_child_count() - 1) as Control
		var line_height: float = life.get_theme_font("normal_font").get_height(life.get_theme_font_size("normal_font_size"))
		check(last != panel.get("_life_more") and last.custom_minimum_size.y >= line_height, "scroll ends with full line-height padding")
		scroll.scroll_vertical = 0
		await process_frame
		check(title.get_global_rect().position.y >= scroll.get_global_rect().position.y, "first line fully inside viewport at top")
		_scan_language(panel, language, scroll.get_global_rect())
		_check_boundary_lines(box, scroll.get_global_rect())
		await _capture(language + "-top")
		await _check_edge_stability(panel)
		scroll.scroll_vertical = int(scroll.get_v_scroll_bar().max_value)
		await process_frame
		check(life.get_global_rect().end.y <= scroll.get_global_rect().end.y, "last Life line fully inside viewport at bottom")
		_scan_language(panel, language, scroll.get_global_rect())
		_check_boundary_lines(box, scroll.get_global_rect())
		await _capture(language + "-bottom")
		await _check_edge_stability(panel)
		panel.call("show_person", 3)
		check((panel.get("_label_id") as Label).text.contains("\u2060"), "record name uses joiners")
	panel.free()
	world.free()
	print("LIFE_LAYOUT_P12 failures=", failures)
	quit(1 if failures else 0)

func _scan_language(node: Node, language: String, viewport: Rect2) -> void:
	var text: String = ""
	if node is Control and (node as Control).is_visible_in_tree() and viewport.intersects((node as Control).get_global_rect()):
		if node is Label:
			text = (node as Label).text
		elif node is RichTextLabel:
			text = (node as RichTextLabel).get_parsed_text()
		elif node is Button:
			text = (node as Button).text
	if not text.is_empty():
		var invalid := RegEx.new()
		invalid.compile("[가-힣ㄱ-ㅎㅏ-ㅣ]" if language == "en" else "[A-Z][A-Z0-9]*_[A-Z0-9_]+|%")
		check(invalid.search(text) == null, "visible language scan " + language + ": " + text)
	for child: Node in node.get_children():
		_scan_language(child, language, viewport)

func _check_boundary_lines(node: Node, viewport: Rect2) -> void:
	if node is Control and not (node as Control).is_visible_in_tree():
		return
	var rows: Array[Rect2] = []
	if node is Label:
		var label: Label = node as Label
		for index: int in label.text.length():
			var rect: Rect2 = label.get_character_bounds(index)
			if rect.has_area():
				rect.position += label.global_position
				rows.append(rect)
	elif node is RichTextLabel:
		var rich: RichTextLabel = node as RichTextLabel
		for index: int in rich.get_line_count():
			if rich.get_line_width(index) > 0:
				rows.append(Rect2(rich.global_position + Vector2(0, rich.get_line_offset(index)), Vector2(rich.get_line_width(index), rich.get_line_height(index))))
	elif node is Button:
		rows.append((node as Button).get_global_rect())
	for rect: Rect2 in rows:
		if rect.end.y > viewport.position.y and rect.position.y < viewport.end.y:
			check(rect.position.y >= viewport.position.y and rect.end.y <= viewport.end.y, "visible line crosses scroll edge: " + str(node.name) + " " + str(rect) + " viewport=" + str(viewport))
	for child: Node in node.get_children():
		_check_boundary_lines(child, viewport)

func _check_edge_stability(panel: Control) -> void:
	var scroll: ScrollContainer = panel.get("_scroll") as ScrollContainer
	var padding: Control = panel.get("_edge_padding") as Control
	var before: Vector3 = Vector3(scroll.size.y, scroll.scroll_vertical, padding.custom_minimum_size.y)
	for frame: int in 8:
		await process_frame
		check(Vector3(scroll.size.y, scroll.scroll_vertical, padding.custom_minimum_size.y) == before, "settled viewport does not oscillate on frame " + str(frame))
	check(not bool(panel.get("_edge_update_pending")), "no pending recursive edge adjustment")

func _capture(stem: String) -> void:
	var output: String = OS.get_environment("P13_LAYOUT_OUT")
	if output.is_empty():
		return
	check(DirAccess.make_dir_recursive_absolute(output) == OK, "create layout capture directory")
	await RenderingServer.frame_post_draw
	check(root.get_texture().get_image().save_png(output.path_join(stem + ".png")) == OK, "save layout capture " + stem)
