extends SceneTree

const Names = preload("res://scripts/ui/person_names.gd")
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const PanelScript = preload("res://scripts/ui/panels/chronicle_panel.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
var failures: int = 0
var checks: int = 0

func _initialize() -> void:
	_run.call_deferred()

func check(value: bool, message: String) -> void:
	checks += 1
	if not value:
		failures += 1
		push_error(message)

func _joined(value: String) -> String:
	return "\u2060".join(value.split(""))

func _run() -> void:
	var locale: Node = root.get_node("Locale")
	var original: String = str(locale.get("_current_lang"))
	var helper: RefCounted = Names.new()
	for method: String in ["unbroken", "plain", "present"]:
		check(helper.has_method(method), "name presentation API exists: " + method)
	if helper.has_method("unbroken") and helper.has_method("plain"):
		for name_text: String in ["카루미", "Karumi", "Village 12", "", "가"]:
			var joined: String = str(helper.call("unbroken", name_text))
			check(joined == _joined(name_text), "join every character without changing spelling")
			check(str(helper.call("plain", joined)) == name_text, "copied name has no joiners")
			check(str(helper.call("unbroken", joined)) == joined, "presentation is idempotent")
	for language: String in ["en", "ko"]:
		locale.call("set_language", language)
		Selection.reset()
		var canonical: String = Names.display_name(25, language)
		check(not canonical.contains("\u2060"), "canonical name stays presentation free")
		var expected_person: String = "[url=person:25][b]%s[/b][/url]" % _joined(canonical)
		check(Chronicle.person(25, locale) == expected_person, "person link is unbroken and bold: " + language)
		Selection.register(99)
		var expected_village: String = "[url=village:99][b]%s[/b][/url]" % _joined(Selection.name_for(99, locale, false))
		check(Chronicle.village(99, locale) == expected_village, "village link is unbroken and bold: " + language)
		var row: Dictionary = {"tick": 0, "kind": 0, "subject": 25, "other": 0, "third": 1, "settlement": 99, "cause": 0}
		var line: String = Chronicle.line(row, 25, locale, {"ticks_per_day": 1440})
		check(line.contains(expected_village), "Life sentence uses protected village link")
		check(line.contains("[url=person:0][b]") and line.contains("[url=person:1][b]"), "Life sentence parents use bold names")
		check(Chronicle.line(row, -1, locale, {"ticks_per_day": 1440}, true).contains(expected_person), "world feed subject uses protected name")
	var panel: Control = PanelScript.new()
	root.add_child(panel)
	panel.set_process(false)
	panel.show()
	await process_frame
	var body: RichTextLabel = panel.get("_body") as RichTextLabel
	check(body.get_theme_font("bold_font").resource_path.ends_with("Galmuri11-Bold.ttf"), "chronicle bold face is Galmuri11-Bold")
	var line_height: float = body.get_theme_font("normal_font").get_height(body.get_theme_font_size("normal_font_size"))
	var normal: StyleBox = body.get_theme_stylebox("normal")
	check(normal.get_content_margin(SIDE_BOTTOM) >= line_height, "chronicle scroll has a full line of bottom padding")
	check(normal.get_content_margin(SIDE_TOP) > 0.0, "chronicle first line has top inset")
	check(body.selection_enabled, "chronicle supports clean copied text")
	panel.free()
	locale.call("set_language", original)
	print("NAME_PRESENTATION_P12 checks=", checks, " failures=", failures)
	quit(0 if failures == 0 else 1)
