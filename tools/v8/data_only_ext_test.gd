# Seven-family selected-pack UI proof against the real scene and immutable bridge bootstrap.
# Bridge methods: bootstrap_catalog, get_content_catalog, get_ui_facts, advance_ticks.
# Run after serial bridge build with WORLDSIM_CONTENT_PACK=/absolute/ext_pack/pack.ron:
# godot --path . --script tools/v8/data_only_ext_test.gd -- /absolute/external/evidence
extends SceneTree

const Hud = preload("res://scripts/ui/mux/mux_hud.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
const Facts = preload("res://scripts/ui/mux/ui_facts_adapter.gd")
const Content = preload("res://scripts/ui/content_context.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Resolver = preload("res://scripts/world/art/material_resolver.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
const Icons = preload("res://scripts/ui/item_icons.gd")
const KnowledgeText = preload("res://scripts/ui/knowledge_text.gd")
const PresentationBootstrap = preload("res://scripts/core/presentation_bootstrap.gd")
var _rows: Array[Dictionary] = []
var _failures: int = 0
var _out: String = ""
var _jump: Dictionary = {}

func _init() -> void:
	call_deferred("_run")

func _check(label: String, passed: bool) -> void:
	_rows.append({"check": label, "passed": passed})
	if not passed:
		_failures += 1
	print("DATA_ONLY_EXT_%s %s" % ["PASS" if passed else "FAIL", label])

func _run() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	_out = args[0] if not args.is_empty() else ""
	_check("external evidence path", _out.is_absolute_path() and not _out.begins_with(ProjectSettings.globalize_path("res://")))
	_check("explicit selected pack", not OS.get_environment("WORLDSIM_CONTENT_PACK").is_empty())
	if _failures > 0:
		quit(1)
		return
	_check("evidence directory", DirAccess.make_dir_recursive_absolute(_out) == OK)
	var boot: Dictionary = WorldSimNode.bootstrap_catalog()
	_check("selected presentation preflight", bool(boot.get("ok", false)))
	if not bool(boot.get("ok", false)):
		push_error(str(boot.get("error", "bootstrap failed")))
		_finish()
		return
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var hud: Hud = main.get_node("UI/MuxHud") as Hud
	hud.store = Store.new(_out.path_join("selected-pack-settings-%d.cfg" % OS.get_process_id()))
	root.add_child(main)
	await process_frame
	_check("real selected scene boot", bool(main.get("boot_ok")))
	if not bool(main.get("boot_ok")):
		_finish()
		return
	var world: Node = main.get_node("WorldSim")
	main.get_node("Camera2D").call("set_paused", true)
	var dto: Dictionary = world.call("get_content_catalog")
	_check_story_controls(dto)
	var selected_ore: Dictionary = Content.row("outcrops", 65543)
	_check("selected ore identity and fallback art", selected_ore.get("id") == "test_outcrop" and selected_ore.get("art_shape") == "ore" and selected_ore.get("art_tint_material") == "test_ore")
	var ore_texture: Texture2D = Icons.item_texture(selected_ore)
	_check("data-only ore has real visible placeholder", ore_texture != null and ore_texture.get_image().get_used_rect().has_area())
	_check("selected ore recipe output", Content.row("recipes", 65542).get("id") == "test_smelt" and Content.row("items", 65541).get("art_shape") == "ingot")
	_check("real world has ore renderer", main.get_node_or_null("ColonyView/OutcropLayer") != null)

	for entry: Array in [["items", 65537, "test_hide"], ["recipes", 65538, "test_hide_recipe"], ["blueprints", 65539, "test_mudbrick_wall"], ["pressures", 4294967294, "test_cold"], ["animals", 4294967293, "test_grazer"]]:
		var row: Dictionary = Content.row(str(entry[0]), int(entry[1]))
		_check("selected DTO " + str(entry[0]), str(row.get("id", "")) == str(entry[2]) and int(row.get("code", -1)) == int(entry[1]))
	var selected_animal: Dictionary = Content.row("animals", 4294967293)
	_check("selected animal domestic policy exposed", bool(selected_animal.get("tameable", false)))
	_check("selected animal art selectors exposed", selected_animal.get("art_shape") == "cloth" and selected_animal.get("art_tint_material") == "limestone")
	var animal_texture: Texture2D = Icons.item_texture(selected_animal)
	_check("selected animal category art visible", animal_texture != null and animal_texture.get_image().get_used_rect().has_area())
	_check("selected domestic label exposed", selected_animal.get("domestic_name_key") == "EXT_DOMESTIC_ANIMAL")
	_check("selected knowledge DTO", dto.get("knowledge", []).any(func(row: Dictionary) -> bool: return str(row.get("id", "")) == "test_hide_craft"))
	_check("localized full-width pressure", StoryText.pressure(root.get_node("Locale"), 4294967294) != "" and not StoryText.pressure(root.get_node("Locale"), 4294967294).contains("4294967294"))
	_check("localized full-width animal", StoryText.animal(root.get_node("Locale"), 4294967293) != "" and not StoryText.animal(root.get_node("Locale"), 4294967293).contains("4294967293"))
	var resolver: RefCounted = Resolver.new()
	resolver.call("load_imported")
	_check("selected construction resolver loads", (resolver.get("problems") as Array).is_empty())
	var resolved: Dictionary = resolver.call("resolve", "test_mudbrick", "clay", "", "wall")
	_check("novel style identity uses existing art", str(resolved.get("style_id", "")) == "test_mudbrick" and not (resolved.get("textures", {}) as Dictionary).is_empty())
	_check_alias_negatives(dto)
	world.call("advance_ticks", 1)
	var facts: Dictionary = Facts.adapt(world.call("get_ui_facts"))
	_check("real world facts accepted", hud.alerts.refresh_from_facts(facts))
	var matching: Array[Dictionary] = hud.alerts.evaluator.active_alerts().filter(func(row: Dictionary) -> bool: return row["id"] == "test_supply")
	_check("new alert evaluates actual world facts", matching.size() == 1 and int(matching[0].get("count", 0)) > 0)
	await process_frame
	var alert_button: Button = hud.alerts.find_child("test_supply", true, false) as Button
	# M-LOOP §1.2e: at most 3 rows show; a folded alert stays reachable through the fold row.
	if alert_button != null and not alert_button.is_visible_in_tree():
		var fold: Button = hud.alerts.find_child("AlertsFold", true, false) as Button
		if fold != null and fold.is_visible_in_tree():
			fold.emit_signal("pressed")
			await process_frame
	_check("new alert has actual visible button", alert_button != null and alert_button.is_visible_in_tree())
	if alert_button != null and matching.size() == 1:
		_check_alert_label_fit(alert_button, matching[0])
	hud.alerts.jump_requested.connect(func(target: Dictionary) -> void: _jump = target)
	if alert_button != null:
		alert_button.pressed.emit()
	_check("new alert click emits real target", not _jump.is_empty())
	# Skip prior bundled lessons through their public UI action; keep selected lesson ordering intact.
	for _i: int in hud.helper.lessons.size():
		if hud.helper.current_id() == "test_pack_lesson" or hud.helper.current_id().is_empty():
			break
		hud.helper.skip_current()
	_check("selected lesson reaches real helper", hud.helper.current_id() == "test_pack_lesson" and hud.helper.is_visible_in_tree())
	_check("new lesson initially incomplete", not hud.helper.can_advance())
	# The helper observes the same named event emitted by the real camera signal.
	main.get_node("Camera2D").emit_signal("camera_activity", "pan")
	await process_frame
	_check("camera event completes added lesson", hud.helper.can_advance())
	var next: Button = hud.helper.find_child("Next", true, false) as Button
	_check("new lesson enables Next", next != null and not next.disabled)
	# Wait for the ordinary incremental minimap publisher, without changing its timer or snapshots.
	var minimap_frames: int = 0
	while hud.minimap.map_image() == null and minimap_frames < 600:
		await process_frame
		minimap_frames += 1
	var minimap_image: Image = hud.minimap.map_image()
	_check("minimap published before capture", minimap_image != null and not minimap_image.is_empty() and not hud.minimap.is_collapsed())
	_rows.append({"check": "minimap wait frames", "frames": minimap_frames})
	await process_frame
	if DisplayServer.get_name() != "headless":
		await RenderingServer.frame_post_draw
		_check("selected alert lesson capture", root.get_texture().get_image().save_png(_out.path_join("data-only-pack.png")) == OK)
	else:
		_rows.append({"check": "rendered capture", "status": "NOT_RUN_HEADLESS"})
	if next != null:
		next.pressed.emit()
	_check("added lesson advances", hud.helper.current_id() != "test_pack_lesson")
	await _check_spine(main, world)
	await _check_building_extension(main, world)
	_finish()

func _check_spine(main: Node, world: Node) -> void:
	var locale: Node = root.get_node("Locale")
	var tech: Dictionary = world.call("get_tech")
	var lanes: Array[Dictionary] = KnowledgeText.catalogue(tech, "lanes")
	_check("selected spine lane DTO", lanes.any(func(row: Dictionary) -> bool: return str(row.get("id", "")) == "ext_lane"))
	var node_id: int = KnowledgeText.node_id("test_inert_spine")
	_check("selected spine inert node DTO", node_id >= 0)
	var panel: Control = main.get_node("UI/TechPanel") as Control
	for language: String in ["ko", "en"]:
		locale.call("set_language", language)
		_check("selected spine locale " + language, str(locale.call("ltr", "EXT_LANE")) != "EXT_LANE" and str(locale.call("ltr", "EXT_INERT_SPINE")) != "EXT_INERT_SPINE")
		panel.call("open")
		await process_frame
		await process_frame
		var labels: Dictionary = panel.get("_lane_labels")
		var cards: Dictionary = panel.get("_cards")
		_check("selected spine lane rendered " + language, labels.has(10) and str((labels[10] as Label).text) == str(locale.call("ltr", "EXT_LANE")))
		_check("selected spine card rendered " + language, cards.has(node_id))
		if cards.has(node_id):
			var card: Button = cards[node_id]
			_check("selected spine grey localized card " + language, str((card.find_child("Name", true, false) as Label).text) == str(locale.call("ltr", "EXT_INERT_SPINE")) and bool(card.get_meta("inert", false)) and card.get_node_or_null("Hatch") != null)
			(card.get_parent().get_parent() as ScrollContainer).ensure_control_visible(card)
			(panel.get("_map_scroll") as ScrollContainer).ensure_control_visible(card)
			card.pressed.emit()
			await process_frame
			await process_frame
			var jump: OptionButton = panel.get("_jump")
			jump.select(jump.get_item_index(10))
			panel.call("_jump_lane", jump.selected)
			await process_frame
			await process_frame
			var scroll: ScrollContainer = panel.get("_map_scroll")
			scroll.scroll_vertical = int(scroll.get_v_scroll_bar().max_value)
			await process_frame
			_check("selected spine debt detail " + language, str((panel.get("_detail_read") as Dictionary).get("debt", "")).contains("fixture behavior deliberately unwired"))
		if DisplayServer.get_name() != "headless":
			await RenderingServer.frame_post_draw
			_check("selected spine capture " + language, root.get_texture().get_image().save_png(_out.path_join("spine-extension-" + language + ".png")) == OK)
		panel.call("close")

func _building_click(button: Button) -> void:
	var ancestor: Node = button.get_parent()
	while ancestor != null:
		if ancestor is ScrollContainer:
			ancestor.ensure_control_visible(button)
			await process_frame
			await process_frame
		ancestor = ancestor.get_parent()
	for pressed: bool in [true, false]:
		var event := InputEventMouseButton.new()
		event.position = root.get_final_transform() * button.get_global_rect().get_center()
		event.button_index = MOUSE_BUTTON_LEFT
		event.pressed = pressed
		Input.parse_input_event(event)
		await process_frame
	await process_frame

func _check_building_extension(main: Node, world: Node) -> void:
	var definitions: Array = world.call("get_content_catalog").get("buildings", [])
	var connected: Dictionary = {}
	var inert: Dictionary = {}
	for row: Dictionary in definitions:
		if str(row.get("id", "")) == "test_mudbrick_screen": connected = row
		elif str(row.get("id", "")) == "test_future_tower": inert = row
	_check("selected Connected building metadata", not connected.is_empty() and bool(connected.get("connected", false)) and int(connected.get("width", 0)) == 1 and str(connected.get("art_roof", "")).to_lower() == "round" and connected.get("art_tint_material") == "clay" and connected.get("art_icon") == "grain")
	_check("selected Inert building metadata", not inert.is_empty() and not bool(inert.get("connected", true)) and not str(inert.get("debt", "")).is_empty() and str(inert.get("art_roof", "")).to_lower() == "angular")
	if connected.is_empty() or inert.is_empty(): return
	var snapshot: Dictionary = world.call("get_building_snapshot")
	var option: Dictionary = {}
	for row: Dictionary in snapshot.get("template_options", []):
		if str(row.get("id", "")) == str(connected["id"]): option = row; break
	_check("selected Connected authoritative option known", bool(option.get("known", false)))
	var requirements: Array = option.get("requirements", [])
	_check("selected Connected existing two-clay cost", requirements.size() == 1 and int(requirements[0].get("item", -1)) == Content.code("items", "clay") and int(requirements[0].get("need", 0)) == 2)
	_rows.append({"check": "selected Connected physical supply observation", "requirements": requirements, "affordable": option.get("affordable", false)})
	var info: Dictionary = world.call("get_world_info")
	var preview: Dictionary = {}
	for y: int in range(0, mini(32, int(info["height"])), 2):
		for x: int in range(0, mini(32, int(info["width"])), 2):
			var candidate: Dictionary = world.call("get_build_template_preview", str(connected["id"]), x, y, 0, "")
			if bool(candidate.get("accepted", false)): preview = candidate; break
		if not preview.is_empty(): break
	var pieces: Array = preview.get("pieces", [])
	_check("selected Connected direct authoritative preview", bool(preview.get("accepted", false)) and pieces.size() == 1 and str(pieces[0].get("kind", "")) == "test_mudbrick_wall")
	var rejected: Dictionary = world.call("get_build_template_preview", str(inert["id"]), 0, 0, 0, "")
	_check("selected Inert direct preview rejects", not bool(rejected.get("accepted", true)))
	var panel: Control = main.get_node_or_null("UI/BuildPanel") as Control
	if panel == null:
		panel = (load("res://scripts/ui/panels/build_panel.gd") as GDScript).new() as Control
		panel.name = "BuildPanel"
		main.get_node("UI").add_child(panel)
	await process_frame
	var locale: Node = root.get_node("Locale")
	for language: String in ["ko", "en"]:
		locale.call("set_language", language)
		panel.call("open")
		await process_frame
		await process_frame
		var groups: Array = panel.get("_groups")
		_check("selected building catalogue rows " + language, groups.size() == 3 and groups[1].get_child_count() + groups[2].get_child_count() == definitions.size())
		for i: int in mini(3, groups.size()):
			var heading: Label = groups[i].get_parent().get_child(groups[i].get_index() - 1) as Label
			_check("selected localized building group " + language + str(i), heading != null and heading.text == str(locale.call("ltr", ["BUILD_GROUP_BUILT", "BUILD_GROUP_AVAILABLE", "BUILD_GROUP_NEXT"][i])))
		var card: Button = panel.find_child("Template_" + str(inert["id"]), true, false) as Button
		var screen: Button = panel.find_child("Template_" + str(connected["id"]), true, false) as Button
		_check("selected building real localized controls " + language, card != null and screen != null and card.text.contains(str(locale.call("ltr", inert["name_key"]))) and screen.text.contains(str(locale.call("ltr", connected["name_key"]))))
		if card == null: continue
		_check("selected Inert grey control " + language, card.get_theme_color("font_color") == Color("807970") and card.icon != null)
		if card.icon != null:
			var image: Image = card.icon.get_image()
			if image != null: _check_building_hatch(image, language)
			elif DisplayServer.get_name() != "headless": _check("native selected building texture readable", false)
			else: _rows.append({"check": "selected building texture pixels", "status": "NOT_RUN_HEADLESS"})
		if DisplayServer.get_name() != "headless":
			await _building_click(card)
		else:
			panel.call("_select", str(inert["id"]), -1)
		var place: Button = panel.get("_place") as Button
		_check("selected Inert debt and disabled action " + language, place.disabled and str((panel.get("_gate") as Label).text).contains(str(inert["debt"])))
		if DisplayServer.get_name() != "headless":
			var log: Dictionary = world.call("get_command_log", 0)
			var sites: Array = world.call("get_building_snapshot").get("template_sites", [])
			await _building_click(place)
			_check("native selected Inert cannot order or allocate " + language, world.call("get_command_log", 0) == log and world.call("get_building_snapshot").get("template_sites", []) == sites)
			await RenderingServer.frame_post_draw
			_check("selected building native capture " + language, root.get_texture().get_image().save_png(_out.path_join("building-extension-" + language + ".png")) == OK)
		else:
			_rows.append({"check": "native selected building click/capture " + language, "status": "NOT_RUN_HEADLESS"})
		panel.hide()

func _check_building_hatch(image: Image, language: String) -> void:
	var positive: int = 0
	var wrong_alpha: int = 0
	var stripe_colors: Dictionary = {}
	var other_colors: Dictionary = {}
	for y: int in image.get_height():
		for x: int in image.get_width():
			var color: Color = image.get_pixel(x, y)
			if color.a <= 0.0: continue
			positive += 1
			if absf(color.a - 0.5) > 1.0 / 255.0: wrong_alpha += 1
			if (x + y) % 8 < 2: stripe_colors[color.to_html(false)] = true
			else: other_colors[color.to_html(false)] = true
	_check("selected actual Inert texture half alpha " + language, positive > 0 and wrong_alpha == 0)
	_check("selected actual Inert diagonal hatch pixels " + language, stripe_colors.size() == 1 and other_colors.size() >= 2)

func _check_alert_label_fit(button: Button, alert: Dictionary) -> void:
	var font: Font = button.get_theme_font("font")
	var font_size: int = button.get_theme_font_size("font_size")
	var available: float = button.size.x - button.get_theme_stylebox("normal").get_minimum_size().x
	var actual_width: float = font.get_string_size(button.text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size).x
	_check("actual alert title and count fit button", actual_width <= available and button.text.ends_with(str(alert["count"])))
	var pack_dir: String = OS.get_environment("WORLDSIM_CONTENT_PACK").get_base_dir()
	for lang: String in ["en", "ko"]:
		var compiled: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://localization/compiled/%s.json" % lang))
		var selected: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(pack_dir.path_join("locale_%s.json" % lang)))
		var label: String = "%s %s" % [alert["icon"], str(compiled["strings"]["MUX_ALERTS_COUNT"]).format({"title": selected["strings"]["EXT_ALERT"], "count": alert["count"]})]
		var width: float = font.get_string_size(label, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size).x
		_check("alert title and count fit " + lang, width <= available)

func _check_story_controls(dto: Dictionary) -> void:
	var validator: RefCounted = PresentationBootstrap.new()
	var selected: Dictionary = {}
	var pack_dir: String = OS.get_environment("WORLDSIM_CONTENT_PACK").get_base_dir()
	for family: String in ["alerts", "lessons", "locale_en", "locale_ko", "bindings"]:
		selected[family] = FileAccess.get_file_as_string(pack_dir.path_join(family + ".json"))
	var locales: Dictionary = {}
	for lang: String in ["en", "ko"]:
		var compiled: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://localization/compiled/%s.json" % lang))
		locales[lang] = compiled["strings"].duplicate(true)
		var additions: Dictionary = JSON.parse_string(selected["locale_" + lang])
		locales[lang].merge(additions["strings"])
	var story: Dictionary = Content.document("story")
	var valid: Dictionary = validator.call("_validate_story_presentation", story.duplicate(true), dto.duplicate(true), locales)
	_check("valid story validation returns affirmative receipt", valid.get("ok", false) == true and valid.get("error", "missing") == "")
	var revision: String = Content.revision()
	var missing_label: Dictionary = dto.duplicate(true)
	for row: Dictionary in missing_label["pressures"]:
		if row["code"] == 4294967294:
			row["name_key"] = "TEST_MISSING_PRESSURE_LOCALE"
	var missing_receipt: Dictionary = validator.call("prepare", missing_label, selected)
	_check("prepare rejects missing novel pressure locale", missing_receipt.get("ok", true) == false and str(missing_receipt.get("error", "")).contains("TEST_MISSING_PRESSURE_LOCALE"))
	var unknown_category: Dictionary = dto.duplicate(true)
	for row: Dictionary in unknown_category["animals"]:
		if row["code"] == 4294967293:
			row["category"] = "test_unknown_category"
	var category_receipt: Dictionary = validator.call("prepare", unknown_category, selected)
	_check("prepare rejects unknown novel animal category", category_receipt.get("ok", true) == false and str(category_receipt.get("error", "")).contains("category"))
	var missing_category: Dictionary = story.duplicate(true)
	missing_category["pressure_categories"].erase("cold")
	var missing_category_receipt: Dictionary = validator.call("_validate_story_presentation", missing_category, dto.duplicate(true), locales)
	_check("story rejects missing category template", missing_category_receipt.get("ok", true) == false and str(missing_category_receipt.get("error", "")).contains("Missing story category"))
	var missing_language: Dictionary = story.duplicate(true)
	missing_language["animal_categories"]["grazer"]["hunted"].erase("en")
	var missing_language_receipt: Dictionary = validator.call("_validate_story_presentation", missing_language, dto.duplicate(true), locales)
	_check("story rejects missing English event template", missing_language_receipt.get("ok", true) == false and str(missing_language_receipt.get("error", "")).contains("template"))
	_check("negative presentation controls preserve installed catalogue", Content.revision() == revision and Content.row("pressures", 4294967294).get("name_key") == "EXT_COLD" and Content.row("animals", 4294967293).get("category") == "grazer" and Content.document("story") == story)

func _check_alias_negatives(dto: Dictionary) -> void:
	for row: Dictionary in [
		{"id": "test_mudbrick", "alias_id": "missing_style", "kind": "wall"},
		{"id": "drystone", "alias_id": "drystone", "kind": "wall"},
		{"id": "test_mudbrick", "alias_id": "drystone", "kind": "floor"},
		{"id": "unreferenced_style", "alias_id": "drystone", "kind": "wall"},
		{"id": "test_mudbrick", "alias_id": "drystone", "kind": "wall", "unexpected": true},
	]:
		var bad: Dictionary = {"schema_version": 1, "construction_styles": {"aliases": [row]}}
		_check("reject malformed selected style " + JSON.stringify(row), not ArtData.validate_extension(bad, dto).is_empty())

func _finish() -> void:
	var file: FileAccess = FileAccess.open(_out.path_join("data-only-ext-results.json"), FileAccess.WRITE)
	if file == null:
		quit(1)
		return
	file.store_string(JSON.stringify({"checks": _rows, "failures": _failures, "selected_pack": OS.get_environment("WORLDSIM_CONTENT_PACK"), "rendered": DisplayServer.get_name() != "headless"}, "\t"))
	file.close()
	quit(0 if _failures == 0 else 1)
