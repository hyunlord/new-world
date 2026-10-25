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
	for entry: Array in [["items", 65537, "test_hide"], ["recipes", 65538, "test_hide_recipe"], ["blueprints", 65539, "test_mudbrick_wall"], ["pressures", 4294967294, "test_cold"], ["animals", 4294967293, "test_grazer"]]:
		var row: Dictionary = Content.row(str(entry[0]), int(entry[1]))
		_check("selected DTO " + str(entry[0]), str(row.get("id", "")) == str(entry[2]) and int(row.get("code", -1)) == int(entry[1]))
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
	_finish()

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
