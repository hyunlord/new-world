# test_core_chronicle.gd: critical mind entries stay visible and localized in the durable feed.
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
	feed.free()
	print("CORE_CHRONICLE_DONE failures=%d" % _failures)
	quit(0 if _failures == 0 else 1)
