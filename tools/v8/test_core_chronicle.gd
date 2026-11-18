# test_core_chronicle.gd: critical mind entries stay localized; the growth filter and card collapse work.
# Bridge methods: none; uses the real Chronicle formatter, feed ingestion and Locale autoload.
extends SceneTree

const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const Feed = preload("res://scripts/ui/panels/chronicle_panel.gd")
var _failures: int = 0

func _init() -> void:
	call_deferred("_run")

func _check(label: String, ok: bool) -> void:
	print("CORE_CHRONICLE %s %s" % ["PASS" if ok else "FAIL", label])
	if not ok:
		_failures += 1

func _run() -> void:
	var locale: Node = root.get_node("Locale")
	var row: Dictionary = {"tick": 12, "kind": 47, "subject": 8, "other": -1, "third": -1,
		"cause": 3, "value": 1000, "index": 17, "link": 1, "link_entry": 6, "link_code": -1}
	var feed: Control = Feed.new()
	feed.call("_ingest_row", row)
	_check("critical choice retained", (feed.get("_entries") as Array).size() == 1)
	_check("critical choice is known", int(feed.get("_unknown_kinds")) == 0)
	var future: Dictionary = row.duplicate()
	future["kind"] = 999
	feed.call("_ingest_row", future)
	_check("future kind still rejected", int(feed.get("_unknown_kinds")) == 1)
	_check("future kind not retained", (feed.get("_entries") as Array).size() == 1)
	for language: String in ["en", "ko"]:
		locale.call("set_language", language)
		var expected: String = "Responded to an urgent need" if language == "en" else "긴급한 필요에 대응함"
		_check(language + " critical sentence", Chronicle.sentence(row, 8, locale, {}).replace("\u2060", "") == expected)
		_check(language + " unrelated person excluded", Chronicle.sentence(row, 9, locale, {}).is_empty())
		var line: String = Chronicle.line(row, -1, locale, {}, true, true)
		_check(language + " person retained", "person:8" in line)
		_check(language + " causal link retained", "why:17" in line)
	root.add_child(feed)
	feed.show()
	var zone: Dictionary = {"kind": 70, "tick": 12, "growth_evidence": {}, "subject": -1}
	var stage: Dictionary = {"kind": 71, "tick": 13, "growth_evidence": {}, "subject": -1}
	var wave: Dictionary = {"kind": 72, "tick": 14, "growth_evidence": {}, "subject": -1}
	var family: Dictionary = {"kind": 2, "tick": 15, "subject": 8, "other": 9}
	for event: Dictionary in [zone, stage, wave, family]:
		feed.call("_ingest_row", event)
	var body: RichTextLabel = feed.get("_body")
	var growth: Button = feed.find_child("GrowthFilter", true, false) as Button
	var collapse: Button = feed.find_child("CollapseChronicle", true, false) as Button
	for language: String in ["en", "ko"]:
		locale.call("set_language", language)
		growth.button_pressed = true
		var zone_line: String = Chronicle.line(zone, -1, locale, {}, true, true)
		var stage_line: String = Chronicle.line(stage, -1, locale, {}, true, true)
		var wave_line: String = Chronicle.line(wave, -1, locale, {}, true, true)
		var family_line: String = Chronicle.line(family, -1, locale, {}, true, true)
		_check(language + " zone, stage and material events have visible pictograms", body.text.count("[img ") == 3
			and body.text.contains(zone_line) and body.text.contains(stage_line) and body.text.contains(wave_line))
		_check(language + " growth filter excludes family", not body.text.contains(family_line))
		_check(language + " growth control localized", growth.text == ("Growth" if language == "en" else "성장"))
		collapse.button_pressed = true
		await process_frame
		_check(language + " collapse hides content and shrinks card", feed.visible and not body.is_visible_in_tree() and feed.size.y < 100.0)
		collapse.button_pressed = false
		await process_frame
		_check(language + " expand restores growth filter", body.is_visible_in_tree() and growth.button_pressed
			and body.text.contains(zone_line) and feed.size.y >= 290.0)
		growth.button_pressed = false
		_check(language + " all restores family alongside growth", body.text.contains(family_line) and body.text.contains(stage_line))
	for _index: int in Feed.FEED_LIMIT:
		feed.call("_ingest_row", family)
	feed.call("set_filter", 0)
	var old_zone_line: String = Chronicle.line(zone, -1, locale, {}, true, true)
	_check("ordinary feed still capped", not body.text.contains(old_zone_line))
	growth.button_pressed = true
	_check("growth stays findable after family fills ordinary feed", body.text.contains(old_zone_line))
	feed.free()
	print("CORE_CHRONICLE_DONE failures=%d" % _failures)
	quit(0 if _failures == 0 else 1)
