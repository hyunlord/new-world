# Generated art identity, missing-resource and layout contract checks.
# Bridge methods: optional saved-scene regression uses load_world, get_agent_snapshot; ordinary scene bootstrap reads.
# --visibility-save /absolute/save runs the isolated real-scene projection regression.
extends SceneTree

const Content = preload("res://scripts/ui/content_context.gd")
const Art = preload("res://scripts/ui/generated_art.gd")


func _initialize() -> void:
	call_deferred("_run")


func _run() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var save_arg: int = args.find("--visibility-save")
	if save_arg >= 0:
		if save_arg + 1 >= args.size() or not await _camp_projection(args[save_arg + 1]):
			push_error("Actual saved-scene camp projection failed")
			quit(1)
			return
		print("GENERATED_ART_PROJECTION_DONE")
		quit()
		return
	var snapshot: Dictionary = {"ids": PackedInt64Array([15]), "life_stages": PackedByteArray([1]), "sexes": PackedByteArray([1])}
	var tech: Dictionary = {"current_era_order": 2, "eras": [{"id": "bronze", "order": 2}]}
	var id: String = Art.portrait_id(snapshot, 15, tech)
	assert(id.begins_with("bronze_child_female_"))
	assert(id == Art.portrait_id(snapshot, 15, tech))
	assert(Art.portrait_id(snapshot, 99, tech).is_empty())
	assert(Art.portrait_id({"ids": [15]}, 15, tech).is_empty())
	for stage: int in 3:
		for sex: int in 2:
			for era: String in ["neolithic", "bronze", "iron"]:
				snapshot["life_stages"] = PackedByteArray([stage])
				snapshot["sexes"] = PackedByteArray([sex])
				tech["eras"] = [{"id": era, "order": 2}]
				var expected_age: String = ["adult", "child", "elder"][stage]
				var expected_sex: String = "female" if sex == 1 else "male"
				var selected: String = Art.portrait_id(snapshot, 15, tech)
				assert(selected.begins_with("%s_%s_%s_" % [era, expected_age, expected_sex]))
				assert(selected.right(1).to_int() in range(8))
				var reached: Dictionary = {}
				for entity: int in 128:
					snapshot["ids"] = PackedInt64Array([entity])
					reached[Art.portrait_id(snapshot, entity, tech).right(1)] = true
				assert(reached.size() == 8)
				snapshot["ids"] = PackedInt64Array([15])
	snapshot["life_stages"] = PackedByteArray([255])
	assert(Art.portrait_id(snapshot, 15, tech).is_empty())
	assert(Art.texture("portraits", "../../outside") == null)
	assert(Art.texture("portraits", "missing_fixture") == null)
	if "--require-texture" in OS.get_cmdline_user_args():
		var portrait: Texture2D = Art.texture("portraits", "neolithic_child_female_0")
		assert(portrait != null and portrait.get_size() == Vector2(512, 512))
		snapshot["life_stages"] = PackedByteArray([1])
		for entity: int in 128:
			snapshot["ids"] = PackedInt64Array([entity])
			var selected: String = Art.portrait_id(snapshot, entity, tech)
			if selected.right(1).to_int() < 4:
				continue
			var path: String = Art.ROOT + "portraits/" + selected + ".webp"
			var previous: Texture2D = Art.texture("portraits", selected)
			Art._textures[path] = null
			assert(Art.portrait(snapshot, entity, tech).resource_path.ends_with(selected.left(-1) + str(selected.right(1).to_int() % 4) + ".webp"))
			Art._textures[path] = previous
			break
	assert(Art.chronicle_art({"kind": 0}).get("id") == "birth")
	assert(Art.chronicle_art({"kind": 54}).get("id") == "first_field")
	assert(Art.chronicle_art({"kind": 11, "cause": 3}).get("id") == "elder_death")
	assert(Art.chronicle_art({"kind": 11, "cause": 0}).is_empty())
	assert(Art.chronicle_art({"kind": 999}).is_empty())
	assert(Art.chronicle_art({"kind": 67, "value": "bad"}).is_empty())
	var catalogue: Dictionary = {"schema_version": 1, "registry_revision": "art-test", "items": [{"code": 301, "id": "copper_ingot", "name_key": "ITEM_COPPER_INGOT"}, {"code": 302, "id": "iron_bloom", "name_key": "ITEM_IRON_BLOOM"}], "blueprints": [], "recipes": [], "pressures": [], "animals": [], "eras": [{"order": 7, "id": "iron"}]}
	assert(Content.install(catalogue, {}).is_empty())
	assert(Art.chronicle_art({"kind": 67, "value": 301}).get("id") == "first_copper")
	assert(Art.chronicle_art({"kind": 67, "value": 302}).get("id") == "first_iron_bloom")
	assert(Art.chronicle_art({"kind": 67, "value": 303}).is_empty())
	assert(Art.chronicle_art({"kind": 59, "value": 7}) == {"family": "eras", "id": "iron"})
	var rect: TextureRect = Art.image(Vector2(128, 128))
	Art.set_image(rect, "portraits", "missing_fixture")
	assert(not rect.visible)
	assert(rect.mouse_filter == Control.MOUSE_FILTER_IGNORE)
	rect.free()
	if "--require-texture" in OS.get_cmdline_user_args():
		if not await _chronicle_layout():
			push_error("Chronicle illustration layout/fallback failed")
			quit(1)
			return
	print("GENERATED_ART_TEST_DONE")
	quit()


## A leading image must not turn the text-grid padding into an image-sized blank area.
func _chronicle_layout() -> bool:
	var panel := preload("res://scripts/ui/panels/chronicle_panel.gd").new()
	var body := RichTextLabel.new()
	body.theme = preload("res://assets/ui/worldsim_theme.tres")
	body.bbcode_enabled = true
	body.size = Vector2(400, 300)
	body.text = "[img=288x162]res://assets/ui/generated/cards/sowing_window.webp[/img]\nText"
	root.add_child(body)
	await process_frame
	await process_frame
	var passed: bool = body.get_line_height(0) >= 162
	panel._body = body
	panel._body_padding = StyleBoxEmpty.new()
	panel._fit_body_lines()
	var text_height: float = ceilf(body.get_theme_font("normal_font").get_height(body.get_theme_font_size("normal_font_size")))
	passed = passed and panel._body_padding.content_margin_bottom < text_height * 2
	panel._locale = root.get_node("Locale")
	var native_row: Dictionary = {"kind": 57, "tick": 100, "subject": -1}
	var original: Texture2D = Art.texture("events", "first_cloth")
	Art._textures[Art.ROOT + "events/first_cloth.webp"] = null
	var plain: String = preload("res://scripts/ui/chronicle_text.gd").line(native_row, -1, panel._locale, {}, true, true)
	passed = passed and panel._illustrated_line(native_row, {}) == plain
	Art._textures[Art.ROOT + "events/first_cloth.webp"] = original
	body.free()
	panel.free()
	return passed


## Use the actual failed visibility save; verify fresh projections, not predicted translations.
func _camp_projection(save_path: String) -> bool:
	if not save_path.is_absolute_path() or not FileAccess.file_exists(save_path):
		return false
	root.get_node("Locale").call("set_language", "ko")
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	main.get_node("UI/MuxHud").set("store", preload("res://scripts/ui/mux/settings_store.gd").new(save_path + ".art-test-settings.cfg"))
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	world.process_mode = Node.PROCESS_MODE_DISABLED
	if not SimReads.mutate(world, "load_world", [save_path]):
		main.free()
		return false
	var original_ids: PackedInt64Array = world.call("get_agent_snapshot").get("ids", PackedInt64Array())
	var camera: Camera2D = main.get_node("Camera2D")
	camera.call("set_paused", true)
	camera.zoom = Vector2.ONE * 0.5
	camera.call("center_on_camp")
	for _frame: int in 90:
		await process_frame
	var report: Dictionary = camera.call("camp_framing_report")
	var included: Array = report.get("ids", [])
	var passed: bool = report.get("status") == "placed" and not included.is_empty() and included.size() == original_ids.size()
	var obstacles: Array[Rect2] = []
	preload("res://scripts/world/colony/marker_layout.gd").ui_rects(main.get_node("UI"), obstacles)
	for bits: int in original_ids:
		var body: Rect2 = preload("res://scripts/world/view_projection.gd").person_rect(bits)
		var clear: bool = bits in included and body.has_area() and camera.get_viewport_rect().encloses(body)
		for obstacle: Rect2 in obstacles:
			clear = clear and not body.intersects(obstacle)
		passed = passed and clear
		print("CAMP_PROJECTION id=%d body=%s clear=%s" % [bits, body, clear])
	print("CAMP_PROJECTION_RESULT position=%s status=%s pass=%s" % [camera.position, report.get("status"), passed])
	main.queue_free()
	await process_frame
	return passed
