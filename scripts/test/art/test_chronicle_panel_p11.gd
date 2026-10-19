extends SceneTree
const PanelScript = preload("res://scripts/ui/panels/chronicle_panel.gd")
const Text = preload("res://scripts/ui/chronicle_text.gd")
var failures: int = 0
class FakeWorld extends Node:
	var tick: int = 1
	var calls: int = 0
	var starts: Array[int] = []
	func get_time_snapshot() -> Dictionary:
		return {"tick": tick}
	func get_world_info() -> Dictionary:
		return {"ticks_per_day": 100, "clock_offset_ticks": 0}
	func get_chronicle_since(start: int, _maximum: int) -> Dictionary:
		calls += 1
		starts.append(start)
		var end: int = mini(start + 512, 700)
		var data: Dictionary = {"next": end, "total": 700}
		for column: String in Text.COLUMNS:
			data[column] = PackedInt64Array()
		for index: int in range(start, end):
			var row: Dictionary = {"tick": index, "kind": 11, "subject": index, "other": -1, "third": -1, "settlement": -1, "settlement2": -1, "cause": 3, "value": 200}
			for column: String in Text.COLUMNS:
				data[column].append(row[column])
		return data
func _initialize() -> void:
	_run.call_deferred()
func check(value: bool, message: String) -> void:
	if not value:
		failures += 1
		push_error(message)
func _run() -> void:
	var host := Control.new()
	root.add_child(host)
	var causal := Control.new()
	causal.name = "CausalPanel"
	host.add_child(causal)
	var panel: Control = PanelScript.new()
	panel.name = "ChroniclePanel"
	host.add_child(panel)
	panel.set_process(false)
	var world := FakeWorld.new()
	panel.set("_world", world)
	for _frame: int in 120:
		panel.call("_process", 0.0)
	check(world.calls == 1, "one world page per fixed tick while hidden")
	check((panel.get("_entries") as Array).size() == 60, "latest60 cap")
	panel.call("toggle_visible")
	check(panel.visible and not causal.visible, "C closes causal")
	var body: RichTextLabel = panel.get("_body") as RichTextLabel
	check(body.text.begins_with(Text.loc(root.get_node("Locale"), "CHRONICLE_DAY_FMT") % 6), "world feed day")
	check(body.text.find("person:511") < body.text.find("person:510"), "newest read page first")
	check(int(panel.get("_next")) == 512 and int(panel.get("_total")) == 700, "paused backlog explicit; no extra getter at fixed tick")
	world.tick += 1
	panel.call("_process", 0.0)
	check(world.calls == 2 and world.starts == [0, 512], "next cursor advances once next tick")
	check(int(panel.get("_next")) == 700 and body.text.find("person:699") < body.text.find("person:698"), "next actual tick drains backlog and shows newest")
	panel.call("_reset")
	check(not panel.visible and int(panel.get("_next")) == 0 and (panel.get("_entries") as Array).is_empty(), "reset clears world history")
	host.free()
	world.free()
	print("CHRONICLE_PANEL_P11 failures=", failures)
	quit(0 if failures == 0 else 1)
