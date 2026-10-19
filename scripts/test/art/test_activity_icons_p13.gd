extends SceneTree
## Headless Image/transform contract, not a framebuffer or blind-recognition test.
const Icons = preload("res://scripts/ui/activity_icons_p11.gd")
const Activity = preload("res://scripts/ui/agent_activity.gd")
const Summary = preload("res://scripts/ui/village_activity_summary_p11.gd")
const NAMES: Array[String] = ["TendFire", "Craft", "Repair", "HearthCircle", "Explore", "Play", "SitByFire"]
var failures: int = 0

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)

func _initialize() -> void:
	call_deferred("run")

func aligned_mask(image: Image) -> Array[String]:
	var rows: Array[String] = []
	var origin := Vector2i((18 - image.get_width()) / 2, (18 - image.get_height()) / 2)
	for y: int in 18:
		var row: String = ""
		for x: int in 18:
			var p := Vector2i(x, y) - origin
			var inside: bool = p.x >= 0 and p.y >= 0 and p.x < image.get_width() and p.y < image.get_height()
			row += "1" if inside and image.get_pixelv(p).a >= 0.5 else "0"
		rows.append(row)
	return rows

func run() -> void:
	var output: String = OS.get_environment("P13_ICON_EXPORT")
	if not output.is_empty():
		if DirAccess.dir_exists_absolute(output):
			check(false, "refusing to overwrite icon evidence directory")
			print("ACTIVITY_ICONS_P13 failures=", failures)
			quit(1)
			return
		check(DirAccess.make_dir_recursive_absolute(output) == OK, "create fresh export directory")
	var sprite := Sprite2D.new()
	var icon := Sprite2D.new()
	root.add_child(sprite)
	sprite.add_child(icon)
	var prior_canvas: Transform2D = root.canvas_transform
	root.canvas_transform = Transform2D(0.0, Vector2(4, 4), 0.0, Vector2.ZERO)
	var records: Array[Dictionary] = []
	var masks: Array = []
	for kind: int in 7:
		var texture: Texture2D = Icons.texture(kind)
		check(texture == Icons.texture(kind), "shared texture cache is stable")
		var image: Image = texture.get_image()
		var pictogram: Image = Icons.pictogram_image(kind)
		var size: int = 22 if kind == 1 else 20
		check(image.get_size() == Vector2i(size, size), "badge canvas is pictogram plus 2px each side")
		check(image.get_pixel(0, 0).a == 0.0 and image.get_pixel(1, 0).a == 0.0, "rounded corners stay transparent")
		check(image.get_pixel(2, 0).to_html(false) == "211c25", "badge RGB")
		check(absf(image.get_pixel(2, 0).a - 0.85) <= 1.0 / 255.0, "badge alpha85 percent quantized to RGBA8")
		var tones: Dictionary = {}
		var colored_pixels: int = 0
		for y: int in pictogram.get_height():
			for x: int in pictogram.get_width():
				var color: Color = pictogram.get_pixel(x, y)
				if color.a > 0.0:
					colored_pixels += 1
					tones[color.to_html()] = true
					check(color.a == 1.0, "pictogram pixels are opaque")
					check(image.get_pixel(x + 2, y + 2) == color, "badge composition preserves pictogram pixel")
		check(tones.size() == 2 and tones.has(Icons.COLORS[kind].to_html()) and tones.has(Icons.HIGHLIGHT.to_html()), "exactly main and highlight tones")
		for y: int in size:
			for x: int in size:
				if x < 2 or y < 2 or x >= size - 2 or y >= size - 2:
					check(image.get_pixel(x, y).a < 1.0, "two-source-pixel margin contains no pictogram")
		if kind == 1:
			var original: Image = Icons.CRAFT.get_image()
			for y: int in 18:
				for x: int in 18:
					check((original.get_pixel(x, y).a > 0.0) == (pictogram.get_pixel(x, y).a > 0.0), "Craft preserves every original silhouette pixel")
		var choice: Dictionary = Activity.with_daily_icon({"sprite_id": "fixture"}, kind)
		check(Activity.apply_activity_icon(sprite, icon, choice, 4.0), "Z4 world icon visible")
		check(icon.texture == texture, "world and summary use shared badge texture including Craft")
		check(icon.scale == Vector2.ONE and icon.texture_filter == CanvasItem.TEXTURE_FILTER_NEAREST, "world integer nearest scaling")
		check(icon.offset == Vector2(-size * 0.5, -size), "center/bottom badge anchor")
		check(icon.get_meta("sprite_id") == Icons.icon_id(kind), "original icon ID retained")
		var rect: Rect2 = icon.get_global_transform_with_canvas() * icon.get_rect()
		check(rect.size == Vector2(size * 4, size * 4) and rect.size.x >= 20.0, "headless Z4 transform geometry")
		var mask: Array[String] = aligned_mask(pictogram)
		masks.append(mask)
		var main: Color = Icons.COLORS[kind]
		records.append({"kind": kind, "name": NAMES[kind], "icon_id": Icons.icon_id(kind), "source_size": [size, size], "pictogram_size": [pictogram.get_width(), pictogram.get_height()], "main": main.to_html(false), "highlight": Icons.HIGHLIGHT.to_html(false), "main_hue_degrees": main.h * 360.0, "badge": image.get_pixel(2, 0).to_html(), "margin": 2, "pictogram_pixels": colored_pixels, "aligned_pictogram_mask_18": mask, "world_z4_transform_rect": [rect.position.x, rect.position.y, rect.size.x, rect.size.y], "summary_z1_texture_size": [size * 2, size * 2], "framebuffer_status": "NOT_MEASURED_headless_dummy"})
		if not output.is_empty():
			check(image.save_png(output.path_join("icon-%d.png" % kind)) == OK, "save actual texture Image")
			check(pictogram.save_png(output.path_join("pictogram-%d.png" % kind)) == OK, "save actual pictogram Image")
			var enlarged: Image = image.duplicate()
			enlarged.resize(size * 4, size * 4, Image.INTERPOLATE_NEAREST)
			check(enlarged.save_png(output.path_join("icon-%d-z4-source.png" % kind)) == OK, "save source resized to measured transform extent")
	root.canvas_transform = prior_canvas
	sprite.visible = false
	check(not Activity.apply_activity_icon(sprite, icon, Activity.with_daily_icon({"sprite_id": "fixture"}, 1), 4.0), "hidden person hides Craft badge")
	sprite.free()
	var pairs: Array[Dictionary] = []
	for a: int in 7:
		for b: int in range(a + 1, 7):
			var union_pixels: int = 0
			var xor_pixels: int = 0
			for y: int in 18:
				for x: int in 18:
					var left: bool = masks[a][y][x] == "1"
					var right: bool = masks[b][y][x] == "1"
					union_pixels += int(left or right)
					xor_pixels += int(left != right)
			var ratio: float = float(xor_pixels) / float(union_pixels)
			var hue: float = absf(float(records[a]["main_hue_degrees"]) - float(records[b]["main_hue_degrees"]))
			hue = minf(hue, 360.0 - hue)
			check(ratio >= 0.25, "aligned pictogram XOR/union >=25 percent: %d/%d" % [a, b])
			check(hue >= 40.0, "main hue separation >=40 degrees: %d/%d" % [a, b])
			pairs.append({"a": a, "b": b, "xor_pixels": xor_pixels, "union_pixels": union_pixels, "xor_union": ratio, "hue_distance_degrees": hue})
	var summary := Summary.new()
	root.add_child(summary)
	summary.display([{"kind": 1, "count": 2}, {"kind": 0, "count": 1}], 1.0, false)
	check(Summary.ICON_SCALE == 2.0 and Summary.HEIGHT >= 44.0, "summary largest badge fits at integer2x")
	check(summary.visible and summary.texture_filter == CanvasItem.TEXTURE_FILTER_NEAREST, "Z1 summary uses nearest shared textures")
	summary.free()
	if not output.is_empty():
		var report: Dictionary = {"schema_version": 1, "valid": failures == 0, "failures": failures, "evidence_scope": "headless_real_Image_and_controlled_canvas_transform_not_framebuffer", "godot": Engine.get_version_info(), "source_sha256": FileAccess.get_sha256("res://scripts/ui/activity_icons_p11.gd"), "craft_original_sha256": FileAccess.get_sha256("res://assets/art/production/U1/04.png"), "alignment": "center original16/18 pictogram canvases on18x18; alpha>=0.5; exclude badge", "icons": records, "pairs": pairs, "blind_stage_a": "NOT_RUN", "blind_stage_b": "NOT_RUN", "terrain_contrast": "NOT_MEASURED_no_framebuffer"}
		var file := FileAccess.open(output.path_join("metrics.json"), FileAccess.WRITE)
		check(file != null, "open metrics report")
		if file != null:
			file.store_string(JSON.stringify(report, "\t") + "\n")
			file.close()
	print("ACTIVITY_ICONS_P13 failures=", failures)
	quit(0 if failures == 0 else 1)
