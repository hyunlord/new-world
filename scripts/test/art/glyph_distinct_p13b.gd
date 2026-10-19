extends SceneTree
## CPU TextServer atlas test: deliberately never requests a framebuffer or window.
const Inspector = preload("res://scripts/ui/panels/agent_inspector_panel.gd")
const Names = preload("res://scripts/ui/person_names.gd")
const DESIGN_SIZE: int = 12 # Both font files: head.unitsPerEm=1200 / outline coordinate GCD=100.
var _out: String
var _ts: TextServer
var _errors: Array[String] = []

func _initialize() -> void:
	_run.call_deferred()

func _run() -> void:
	_out = OS.get_environment("P13_GLYPH_OUT")
	if DisplayServer.get_name() != "headless" or _out.is_empty():
		push_error("Requires headless display and P13_GLYPH_OUT")
		quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	_ts = TextServerManager.get_primary_interface()
	var panel: Control = Inspector.new()
	root.add_child(panel)
	panel.hide()
	panel.set_process(false)
	var life: RichTextLabel = panel.get("_life_text") as RichTextLabel
	var actual_size: int = life.get_theme_font_size("normal_font_size")
	var bold_size: int = life.get_theme_font_size("bold_font_size")
	var fonts: Dictionary = {"regular": life.get_theme_font("normal_font"), "bold": life.get_theme_font("bold_font")}
	var rows: Array[Dictionary] = []
	var metadata: Dictionary = {}
	var actual_pass: bool = true
	for weight: String in fonts:
		var font: FontFile = fonts[weight] as FontFile
		metadata[weight] = {"actual_size": actual_size if weight == "regular" else bold_size, "path": font.resource_path, "sha256": FileAccess.get_sha256(font.resource_path), "antialiasing": font.antialiasing, "hinting": font.hinting, "oversampling": font.oversampling, "subpixel_positioning": font.subpixel_positioning, "force_autohinter": font.force_autohinter, "msdf": font.multichannel_signed_distance_field}
		for language: String in ["en", "ko"]:
			var row: Dictionary = _measure(font, actual_size if weight == "regular" else bold_size, weight, language)
			rows.append(row)
			actual_pass = actual_pass and bool(row["passed"])
	var result: Dictionary = {"actual_life_size": actual_size, "actual_bold_size": bold_size, "design_size": DESIGN_SIZE, "integer_scale": actual_size > 0 and bold_size > 0 and actual_size % DESIGN_SIZE == 0 and bold_size % DESIGN_SIZE == 0, "actual_pass": actual_pass and _errors.is_empty() and actual_size > 0 and bold_size > 0 and actual_size % DESIGN_SIZE == 0 and bold_size % DESIGN_SIZE == 0, "errors": _errors, "font_settings": metadata, "godot": Engine.get_version_info(), "display": DisplayServer.get_name(), "text_server": _ts.get_name(), "threshold_alpha": 0.5, "minimum_differing_design_pixels": 1, "rule": "Art Pass 13b section 4.2: distinct at design size; actual screen-size differences independently rendered", "alignment": "same origin and baseline; TextServer shaped advances and offsets; no crop normalization", "carry_closure": "Every suffix/carry assignment is (index+1)%46. Thus both tables are closed under carry; all46 entries included, plus all46 Latin initial-capital forms.", "rows": rows}
	var file := FileAccess.open(_out.path_join("glyph-result.json"), FileAccess.WRITE)
	file.store_string(JSON.stringify(result, "\t") + "\n")
	file.close()
	panel.queue_free()
	print("GLYPH_DISTINCT_P13B_COMPLETE actual_pass=", result["actual_pass"], " errors=", _errors.size())
	quit(0 if bool(result["actual_pass"]) else 1)

func _measure(font: Font, size: int, weight: String, language: String) -> Dictionary:
	var syllables: Array[String] = []
	syllables.assign(Names.EN if language == "en" else Names.KO)
	if language == "en":
		for value: String in Names.EN:
			syllables.append(value.left(1).to_upper() + value.substr(1))
	var design_masks: Array[Dictionary] = _render_masks(font, DESIGN_SIZE, weight, language, syllables)
	var screen_masks: Array[Dictionary] = _render_masks(font, size, weight, language, syllables)
	var pairs: Array[Dictionary] = []
	var failed: int = 0
	for left: int in syllables.size():
		for right: int in range(left + 1, syllables.size()):
			var difference: int = _difference(design_masks[left], design_masks[right])
			pairs.append({"left": syllables[left], "right": syllables[right], "differing_design_pixels": difference, "differing_screen_pixels": _difference(screen_masks[left], screen_masks[right])})
			failed += int(difference < 1)
	pairs.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		if int(a["differing_design_pixels"]) != int(b["differing_design_pixels"]):
			return int(a["differing_design_pixels"]) < int(b["differing_design_pixels"])
		return str(a["left"]) + "/" + str(a["right"]) < str(b["left"]) + "/" + str(b["right"]))
	var stem: String = "%s-%s-design%d-screen%d" % [language, weight, DESIGN_SIZE, size]
	var file := FileAccess.open(_out.path_join(stem + "-all-pairs.json"), FileAccess.WRITE)
	if file == null:
		_errors.append("Failed pair report save " + stem)
	else:
		file.store_string(JSON.stringify(pairs) + "\n")
		file.close()
	return {"script": language, "weight": weight, "design_size": DESIGN_SIZE, "screen_size": size, "syllables": syllables, "pair_count": pairs.size(), "failed_pairs": failed, "passed": failed == 0, "closest10": pairs.slice(0, 10), "atlas_columns": 10, "design_atlas_cell_size": DESIGN_SIZE * 4, "screen_atlas_cell_size": size * 4}

func _render_masks(font: Font, size: int, weight: String, language: String, syllables: Array[String]) -> Array[Dictionary]:
	var masks: Array[Dictionary] = []
	var cell: int = size * 4
	var atlas: Image = Image.create(cell * 10, cell * ceili(syllables.size() / 10.0), false, Image.FORMAT_RGBA8)
	atlas.fill(Color.TRANSPARENT)
	for value: String in syllables:
		var mask: Dictionary = _mask(font, size, value)
		masks.append(mask)
		var origin := Vector2i((masks.size() - 1) % 10 * cell + size, floori(float(masks.size() - 1) / 10.0) * cell + size * 2)
		for position: Vector2i in mask:
			atlas.set_pixelv(origin + position, Color.WHITE)
	var stem: String = "%s-%s-%d" % [language, weight, size]
	if atlas.save_png(_out.path_join(stem + "-binary-atlas.png")) != OK:
		_errors.append("Failed atlas save " + stem)
	return masks

func _difference(left: Dictionary, right: Dictionary) -> int:
	var difference: int = 0
	for position: Vector2i in left:
		difference += int(not right.has(position))
	for position: Vector2i in right:
		difference += int(not left.has(position))
	return difference

func _mask(font: Font, size: int, value: String) -> Dictionary:
	var line := TextLine.new()
	line.add_string(value, font, size)
	var glyphs: Array[Dictionary] = _ts.shaped_text_get_glyphs(line.get_rid())
	var mask: Dictionary = {}
	var pen: float = 0.0
	for glyph: Dictionary in glyphs:
		if not glyph.has("offset"):
			print("GLYPH_KEYS ", glyph)
			_errors.append("Glyph dictionary shape")
			return {}
		var rid: RID = glyph["font_rid"]
		if rid not in font.get_rids():
			_errors.append("Unexpected fallback font " + value)
		var index: int = glyph["index"]
		if not rid.is_valid() or index == 0:
			_errors.append("Missing glyph " + value)
			continue
		_ts.font_render_glyph(rid, Vector2i(size, 0), index)
		var texture_index: int = _ts.font_get_glyph_texture_idx(rid, Vector2i(size, 0), index)
		var image: Image = _ts.font_get_texture_image(rid, Vector2i(size, 0), texture_index)
		if image == null or image.is_empty():
			_errors.append("Empty CPU texture " + value)
			continue
		var uv: Rect2 = _ts.font_get_glyph_uv_rect(rid, Vector2i(size, 0), index)
		var offset: Vector2 = _ts.font_get_glyph_offset(rid, Vector2i(size, 0), index)
		for repetition: int in int(glyph["repeat"]):
			for y: int in int(uv.size.y):
				for x: int in int(uv.size.x):
					if image.get_pixel(int(uv.position.x) + x, int(uv.position.y) + y).a >= 0.5:
						mask[Vector2i(roundi(pen + Vector2(glyph["offset"]).x + offset.x) + x, roundi(Vector2(glyph["offset"]).y + offset.y) + y)] = true
			pen += float(glyph["advance"])
	if mask.is_empty():
		_errors.append("Empty mask " + value)
	return mask
