extends SceneTree
const PanelScript = preload("res://scripts/ui/panels/chronicle_panel.gd")
var failures: int = 0
var checks: int = 0

func _initialize() -> void:
	_run.call_deferred()

func check(value: bool, message: String) -> void:
	checks += 1
	if not value:
		failures += 1
		push_error(message)

func _run() -> void:
	root.size = Vector2i(1920, 1080)
	var locale: Node = root.get_node("Locale")
	var original: String = str(locale.get("_current_lang"))
	var panel: Control = PanelScript.new()
	root.add_child(panel)
	panel.set_process(false)
	panel.show()
	var entries: Array[Dictionary] = []
	for index: int in 60:
		entries.append({"tick": index * 1440, "kind": 0, "subject": index + 25, "other": index, "third": index + 1, "settlement": 1, "cause": 0})
	panel.set("_entries", entries)
	var body: RichTextLabel = panel.get("_body") as RichTextLabel
	for language: String in ["en", "ko"]:
		locale.call("set_language", language)
		panel.call("_refresh")
		for _frame: int in 3:
			await process_frame
		var bar: VScrollBar = body.get_v_scroll_bar()
		for bottom: bool in [false, true]:
			bar.value = bar.max_value if bottom else 0.0
			await process_frame
			var style: StyleBox = body.get_theme_stylebox("normal")
			var first_y: float = style.get_content_margin(SIDE_TOP)
			var end_y: float = body.size.y - style.get_content_margin(SIDE_BOTTOM)
			var visible_lines: int = 0
			for line: int in body.get_line_count():
				var top: float = first_y + body.get_line_offset(line) - bar.value
				var end: float = top + body.get_line_height(line)
				if end > first_y + 0.01 and top < end_y - 0.01:
					visible_lines += 1
					check(top >= first_y - 0.01 and end <= end_y + 0.01, "%s bottom=%s line=%d rect=%s viewport=%s scroll=%s" % [language, bottom, line, Vector2(top, end), Vector2(first_y, end_y), bar.value])
			check(visible_lines > 0, "visible lines at each extreme")
			var output: String = OS.get_environment("P12_CHRONICLE_LAYOUT_OUT")
			if not output.is_empty():
				DirAccess.make_dir_recursive_absolute(output)
				RenderingServer.force_draw(false)
				await process_frame
				check(root.get_texture().get_image().save_png(output.path_join(language + ("_bottom.png" if bottom else "_top.png"))) == OK, "layout screenshot saved")
		print("CHRONICLE_LAYOUT geometry ", language, " size=", body.size, " lines=", body.get_line_count(), " content=", body.get_content_height(), " max=", bar.max_value, " page=", bar.page)
	panel.free()
	locale.call("set_language", original)
	print("CHRONICLE_LAYOUT_P12 checks=", checks, " failures=", failures)
	quit(0 if failures == 0 else 1)
