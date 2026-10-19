extends SceneTree
const Text = preload("res://scripts/ui/chronicle_text.gd")
const Names = preload("res://scripts/ui/person_names.gd")
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
func _run() -> void:
	var locale: Node = root.get_node("Locale")
	var original: String = str(locale.get("_current_lang"))
	var info: Dictionary = {"ticks_per_day": 100, "clock_offset_ticks": 25}
	check(Text.day(75, info) == 2 and Text.day(0, info) == 1, "clock offset day")
	var expected: Array[String] = ["CHRON_BORN_FMT", "CHRON_FOUNDER_FMT", "CHRON_PAIRED_FMT", "CHRON_CAME_OF_AGE", "CHRON_BECAME_ELDER", "CHRON_DEPARTED_FMT", "CHRON_DEPARTURE_CANCELLED_FMT", "CHRON_SETTLED_FMT", "CHRON_JOINED_FMT", "CHRON_LEFT_BROKE_UP_FMT", "CHRON_WIDOWED_FMT", "CHRON_DIED_FMT", "CHRON_SETTLEMENT_FORMED_FMT", "CHRON_FISSION_STARTED_FMT", "CHRON_DAUGHTER_FOUNDED_FMT", "CHRON_FISSION_CANCELLED_FMT", "CHRON_SETTLEMENT_DISSOLVED_FMT"]
	for language: String in ["en", "ko"]:
		locale.call("set_language", language)
		Selection.reset()
		for kind: int in 17:
			var row: Dictionary = {"tick": 75, "kind": kind, "subject": 11 if kind < 12 else -1, "other": 22, "third": 33, "settlement": 99, "settlement2": 98, "cause": 0, "value": 250}
			var text: String = Text.sentence(row, 11 if kind < 12 else -1, locale, info)
			check(not text.is_empty() and not text.contains("CHRON_") and not text.contains("%"), "kind translated %d %s" % [kind, language])
			var template: String = Text.loc(locale, expected[kind])
			check(template != expected[kind], "key exists")
			var prefix: String = template.split("%")[0]
			check(text.begins_with(prefix), "subject key matches kind %d" % kind)
			var line: String = Text.line(row, 11 if kind < 12 else -1, locale, info)
			check(line.begins_with((Text.loc(locale, "CHRONICLE_DAY_FMT") % 2) + " · "), "day formatting")
			if kind < 12:
				check(Text.sentence(row, 44, locale, info).is_empty(), "unrelated omitted")
				for viewed: int in [22, 33]:
					var related: String = Text.sentence(row, viewed, locale, info)
					if kind == 0:
						check(related == Text.loc(locale, "CHRON_CHILD_BORN_FMT") % Text.person(11, locale), "both parents child perspective")
					elif kind == 2:
						check(related == Text.loc(locale, "CHRON_PAIRED_FMT") % Text.person(11, locale), "paired reverse perspective")
					else:
						check(related.is_empty(), "non-subject omitted %d" % kind)
			if kind == 0:
				row["cause"] = 1
				check(Text.sentence(row, 11, locale, info).ends_with(Text.loc(locale, "CHRON_BORN_FALLBACK_NOTE")), "fallback note")
			if kind == 5:
				for cause: int in 5:
					row["cause"] = cause
					check(Text.sentence(row, 11, locale, info).ends_with(Text.loc(locale, Text.DEPART[cause])), "departure cause %d" % cause)
			if kind == 9:
				row["cause"] = 2
				check(Text.sentence(row, 11, locale, info) == Text.loc(locale, "CHRON_LEFT_FMT") % Text.village(99, locale), "left roster cause")
			if kind == 11:
				for cause: int in 5:
					row["cause"] = cause
					check(Text.sentence(row, 11, locale, info) == Text.loc(locale, "CHRON_DIED_FMT") % [2, Text.loc(locale, Text.DEATH[cause])], "death age/cause")
					row["value"] = -1
					check(Text.sentence(row, 11, locale, info) == Text.loc(locale, "CHRON_DIED_NO_AGE_FMT") % Text.loc(locale, Text.DEATH[cause]), "death absent age")
					row["value"] = 250
		check(Text.person(22, locale) == "[url=person:22][b]%s[/b][/url]" % Names.present(22, language), "person link")
		check(Selection.number_for(99) > 0 and Text.village(99, locale).contains("url=village:99"), "unseen settlement registered/link")
		check(not Selection.name_for(99, locale, false).contains("%"), "plain village name")
	locale.call("set_language", original)
	print("CHRONICLE_TEXT_P11 checks=", checks, " failures=", failures)
	quit(0 if failures == 0 else 1)
