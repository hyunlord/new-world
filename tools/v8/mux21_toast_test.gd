# Focused HUD notification regression: real HUD/OrderBar, detached read-only chronicle fixture.
# No simulation writes. Real-scene boot uses its ordinary bridge reads; fixture APIs match
# get_steward/get_chronicle_since. This is UI logic evidence, not a natural defence capture.
# Run windowed or headless after import: --script res://tools/v8/mux21_toast_test.gd
extends SceneTree

const Store = preload("res://scripts/ui/mux/settings_store.gd")
const Chronicle = preload("res://scripts/ui/chronicle_text.gd")
const Capture = preload("res://tools/v8/mux21_play_capture.gd")
var failures: int = 0
var checks: int = 0

class ChronicleSource extends Node:
	var sequence: int = 4
	var entries: Array[Dictionary] = []
	func get_steward() -> Dictionary:
		return {"sequence": sequence, "rules": {1: "MUX_STEWARD_RULE_1"}}
	func get_chronicle_since(start: int, maximum: int) -> Dictionary:
		var end: int = mini(entries.size(), start + mini(maximum, 2))
		var page: Dictionary = {"next": end, "total": entries.size()}
		for key: String in Chronicle.COLUMNS:
			var column: PackedInt64Array = PackedInt64Array()
			for index: int in range(start, end):
				column.append(int(entries[index].get(key, index if key == "index" else -1)))
			page[key] = column
		return page

func _init() -> void:
	call_deferred("_run")

func _check(label: String, passed: bool) -> void:
	checks += 1
	if not passed: failures += 1
	print("MUX21_TOAST_%s %s" % ["PASS" if passed else "FAIL", label])

func _event_normalization() -> void:
	var live: Dictionary = {}
	for key: String in Chronicle.COLUMNS: live[key] = -1
	live.merge({"kind": 46, "tick": 18721, "index": 222, "value": 2, "cause": 7}, true)
	var parsed: Variant = JSON.parse_string(JSON.stringify(live))
	var normalized: Dictionary = Capture.normalize_event(parsed)
	_check("JSON roundtrip preserves every integer event field", bool(normalized.get("valid", false)) and normalized.get("event") == live)
	for invalid: Variant in [1.5, "2", true, null, INF, NAN, 9007199254740992.0]:
		var altered: Dictionary = live.duplicate()
		altered["value"] = invalid
		_check("reject nonintegral or unsafe event value " + str(invalid), not bool(Capture.normalize_event(altered).get("valid", true)))
	var missing: Dictionary = live.duplicate()
	missing.erase("index")
	_check("reject missing event key", not bool(Capture.normalize_event(missing).get("valid", true)))
	var extra: Dictionary = live.duplicate()
	extra["extra"] = 0
	_check("reject extra event key", not bool(Capture.normalize_event(extra).get("valid", true)))
	var changed: Dictionary = live.duplicate()
	changed["value"] = 3.0
	_check("changed integral value stays unequal", Capture.normalize_event(changed).get("event") != live)

func _run() -> void:
	_event_normalization()
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var hud: Node = main.get_node("UI/MuxHud")
	hud.set("store", Store.new("/tmp/mux21-toast-%d.cfg" % OS.get_process_id()))
	root.add_child(main)
	_check("real scene boot", bool(main.get("boot_ok")))
	if not bool(main.get("boot_ok")):
		_finish()
		return
	main.get_node("Camera2D").call("set_paused", true)
	main.get_node("WorldSim").process_mode = Node.PROCESS_MODE_DISABLED
	root.get_node("Locale").call("set_language", "ko")
	hud.set_process(false)
	var bar: Node = main.get_node("UI/OrderBar")
	bar.set_process(false)
	var original: Node = hud.get("_world")
	var source: ChronicleSource = ChronicleSource.new()
	source.entries.append({"kind": 46, "value": 99})
	hud.set("_world", source)
	hud.call("_baseline_notifications")
	_check("attach/load baseline excludes old defence", int(hud.get("_chronicle_cursor")) == 1 and (bar.get("_background_toasts") as Array).is_empty())
	source.entries.append_array([{"kind": 46, "value": 2}, {"kind": 47}, {"kind": 46, "value": 3}])
	hud.call("_poll_steward_notifications", source.get_steward())
	var queue: Array = bar.get("_background_toasts")
	_check("unchanged action sequence still queues new defence", queue.size() == 1 and int(hud.get("_sequence")) == 4 and int(hud.get("_chronicle_cursor")) == 3)
	if queue.size() == 1:
		var expected: String = str(root.get_node("Locale").call("ltr", "MUX_STEWARD_DEFENCE_ACTION")).format({"count": 2})
		_check("defence localized count comes from row", str(queue[0]["text"]).replace("\u2060", "") == expected)
	hud.call("_poll_steward_notifications", source.get_steward())
	_check("unchanged sequence drains next page", (bar.get("_background_toasts") as Array).size() == 2 and int(hud.get("_chronicle_cursor")) == 4)
	hud.call("_poll_steward_notifications", source.get_steward())
	_check("cursor deduplicates repeated polls", (bar.get("_background_toasts") as Array).size() == 2)
	source.sequence = 5
	source.entries.append({"kind": 45, "value": 5, "cause": 1})
	hud.call("_poll_steward_notifications", source.get_steward())
	_check("ordinary action still delivered", (bar.get("_background_toasts") as Array).size() == 3 and int(hud.get("_sequence")) == 5)
	source.sequence = 2
	hud.call("_poll_steward_notifications", source.get_steward())
	_check("rollback clears queue and baselines history", (bar.get("_background_toasts") as Array).is_empty() and int(hud.get("_sequence")) == 2 and int(hud.get("_chronicle_cursor")) == source.entries.size())
	source.entries.append({"kind": 46, "value": 1})
	bar.name = "UnavailableOrderBar"
	var cursor: int = int(hud.get("_chronicle_cursor"))
	hud.call("_poll_steward_notifications", source.get_steward())
	_check("missing destination preserves cursor", int(hud.get("_chronicle_cursor")) == cursor)
	bar.name = "OrderBar"
	hud.call("_poll_steward_notifications", source.get_steward())
	_check("restored destination receives pending defence", (bar.get("_background_toasts") as Array).size() == 1)
	hud.call("_baseline_notifications")
	hud.call("_poll_steward_notifications", source.get_steward())
	_check("replacement baseline clears pending and suppresses history", (bar.get("_background_toasts") as Array).is_empty())
	hud.set("_world", original)
	source.free()
	_finish()

func _finish() -> void:
	print("MUX21_TOAST_DONE " + JSON.stringify({"checks": checks, "failures": failures, "scope": "HUD cursor/queue logic and JSON comparison; not natural defence or pixel acceptance"}))
	quit(0 if failures == 0 else 1)
