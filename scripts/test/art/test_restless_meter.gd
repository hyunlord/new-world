extends SceneTree
const Selection = preload("res://scripts/core/settlement_selection.gd")
const Labels = preload("res://scripts/ui/settlement_labels.gd")
const GodPanel = preload("res://scripts/ui/panels/god_hand_panel.gd")
var failures: int = 0
class World extends Node:
	signal world_reset
	var pressure: int = 0
	var threshold: int = 1000
	func get_society_snapshot() -> Dictionary:
		return {"settlement_ids": PackedInt32Array([7]), "fission_pressures": PackedInt64Array([pressure]), "fission_thresholds": PackedInt64Array([threshold])}
	func get_settlement_snapshot() -> Dictionary:
		return {"settlement_ids": PackedInt32Array([7]), "formation_xs": PackedInt32Array([4]), "formation_ys": PackedInt32Array([5])}
	func get_intervention_state() -> Dictionary: return {}
	func get_time_snapshot() -> Dictionary: return {"tick": 0}
func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)
func _initialize() -> void:
	call_deferred("run")
func run() -> void:
	Selection.reset()
	var world := World.new()
	var labels := Labels.new()
	root.add_child(labels)
	labels.set_process(false)
	labels.configure(world)
	for sample: Array in [[0, 0], [333, 0], [334, 1], [666, 1], [667, 2], [1000, 2], [1200, 2]]:
		world.pressure = sample[0]
		Selection.observe(SimReads.read(world, "get_society_snapshot"), SimReads.read(world, "get_settlement_snapshot"))
		var state: Dictionary = Selection.restlessness_for(7)
		check(state["stage"] == sample[1] and state["pressure"] == sample[0] and state["threshold"] == 1000, "exact rational thirds and raw values")
	world.threshold = 9223372036854775807
	world.pressure = 9223372036854775807
	Selection.observe(SimReads.read(world, "get_society_snapshot"), SimReads.read(world, "get_settlement_snapshot"))
	check(Selection.restlessness_for(7)["stage"] == 2, "saturated int64 pressure and theta do not overflow")
	world.threshold = 1000
	Selection.observe(SimReads.read(world, "get_society_snapshot"), SimReads.read(world, "get_settlement_snapshot"))
	check(Selection.restlessness_for(7)["stage"] == 2, "saturated pressure with ordinary theta remains ready")
	world.threshold = 3
	for pressure: int in [0, 1, 2]:
		world.pressure = pressure
		Selection.observe(SimReads.read(world, "get_society_snapshot"), SimReads.read(world, "get_settlement_snapshot"))
		check(Selection.restlessness_for(7)["stage"] == pressure, "exact third equality advances stage")
	world.threshold = 1000
	world.pressure = 700
	Selection.observe(SimReads.read(world, "get_society_snapshot"), SimReads.read(world, "get_settlement_snapshot"))
	for zoom: float in [1.0, 2.0, 4.0, 8.0]:
		labels.scale = Vector2.ONE * zoom
		labels._update_labels()
		var meter: Control = labels._meters[7]
		check(meter.visible == (zoom >= 2.0), "meter zoom visibility")
		check(meter.color == Selection.color_for(7), "actual meter village colour")
		var row: Dictionary = labels.capture_metadata()["meters"][0]
		check(is_equal_approx(row["screen_rect"][2], 26.0), "native meter width independent of zoom")
	var panel := GodPanel.new()
	root.add_child(panel)
	panel.set_process(false)
	panel.configure(world)
	Selection.select(7)
	for language: String in ["en", "ko"]:
		var locale: Node = root.get_node("Locale")
		locale.call("set_language", language)
		for sample: Array in [[0, "CALM"], [400, "UNEASY"], [700, "READY"]]:
			world.pressure = sample[0]
			panel.refresh_state()
			var expected: String = str(locale.call("ltr", "VILLAGE_RESTLESS_TITLE")) + ": " + str(locale.call("ltr", "VILLAGE_RESTLESS_" + str(sample[1])))
			check(panel._restless_row.visible and panel._restless_label.text == expected, "selected inspector localized title/state")
	Selection.clear()
	panel.refresh_state()
	check(not panel._restless_row.visible, "no selected village hides inspector")
	world.threshold = 0
	Selection.select(7)
	panel.refresh_state()
	labels.refresh_snapshot()
	check(not panel._restless_row.visible and not labels._meters[7].visible, "theta zero hides both surfaces")
	Selection.observe({"settlement_ids": PackedInt32Array([7])})
	check(not Selection.restlessness_for(7)["visible"], "missing fields hide meter")
	Selection.observe({"settlement_ids": PackedInt32Array([7]), "fission_thresholds": PackedInt64Array([1000])})
	check(not Selection.restlessness_for(7)["visible"], "missing pressure hides meter despite theta")
	world.world_reset.emit()
	Selection.reset()
	check(not Selection.restlessness_for(7)["visible"], "reset clears pressure")
	panel.free()
	labels.free()
	world.free()
	print("RESTLESS_METER failures=", failures)
	quit(1 if failures else 0)
