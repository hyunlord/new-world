extends SceneTree
const Inspector = preload("res://scripts/ui/panels/agent_inspector_panel.gd")
const REGULAR = preload("res://assets/fonts/galmuri/Galmuri11.ttf")
const BOLD = preload("res://assets/fonts/galmuri/Galmuri11-Bold.ttf")
var _label: Label
var _output: String

func _initialize() -> void:
	_run.call_deferred()

func _run() -> void:
	_output = OS.get_environment("P12_FONT_OUT")
	if _output.is_empty() or DirAccess.make_dir_recursive_absolute(_output) != OK:
		push_error("requires fresh P12_FONT_OUT directory")
		quit(2)
		return
	root.size = Vector2i(1920, 1080)
	var panel: Control = Inspector.new()
	root.add_child(panel)
	panel.hide()
	panel.set_process(false)
	var life: RichTextLabel = panel.get("_life_text") as RichTextLabel
	var actual_size: int = life.get_theme_font_size("normal_font_size")
	_label = Label.new()
	_label.position = Vector2(16, 16)
	_label.size = Vector2(700, 90)
	_label.add_theme_color_override("font_color", Color.WHITE)
	_label.add_theme_color_override("font_shadow_color", Color.TRANSPARENT)
	_label.add_theme_constant_override("outline_size", 0)
	root.add_child(_label)
	var rows: Array[Dictionary] = []
	var passed: bool = true
	for weight: String in ["regular", "bold"]:
		_label.add_theme_font_override("font", REGULAR if weight == "regular" else BOLD)
		_label.add_theme_font_size_override("font_size", actual_size)
		var plain: Image = await _capture("카루미", weight + "-plain")
		var joined: Image = await _capture("카\u2060루\u2060미", weight + "-joined")
		var ti: Image = await _capture("티", weight + "-ti")
		var di: Image = await _capture("디", weight + "-di")
		await _capture("오후티 오후디 티 디 트 드 키 기", weight + "-glyph-line")
		var join_difference: int = _different(plain, joined)
		var glyph_difference: int = _different(ti, di)
		rows.append({"weight": weight, "size": actual_size, "joiner_differing_pixels": join_difference, "ti_di_differing_pixels": glyph_difference})
		passed = passed and join_difference == 0 and glyph_difference >= 2
	var result := FileAccess.open(_output.path_join("font-result.json"), FileAccess.WRITE)
	if result == null:
		push_error("cannot save font probe report")
		quit(2)
		return
	result.store_string(JSON.stringify({"passed": passed, "actual_life_size": actual_size, "rows": rows}, "\t") + "\n")
	result.close()
	print("LIFE_FONT_P12 passed=", passed, " size=", actual_size)
	quit(0 if passed else 1)

func _capture(value: String, stem: String) -> Image:
	_label.text = value
	await process_frame
	await process_frame
	await RenderingServer.frame_post_draw
	var pixels: Image = root.get_texture().get_image().get_region(Rect2i(16, 16, 700, 90))
	if pixels.save_png(_output.path_join(stem + ".png")) != OK:
		push_error("cannot save " + stem)
	return pixels

func _different(left: Image, right: Image) -> int:
	var count: int = 0
	for y: int in left.get_height():
		for x: int in left.get_width():
			count += int(left.get_pixel(x, y) != right.get_pixel(x, y))
	return count
