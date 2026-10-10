extends SceneTree
const Moments = preload("res://scripts/ui/agent_pose_moments.gd")
var failures: int = 0

func check(value: bool, message: String) -> void:
	if not value:
		push_error(message)
		failures += 1

func snap(hunger: float, wood: int = 0) -> Dictionary:
	return {"hungers": PackedFloat32Array([hunger]), "carried_woods": PackedInt32Array([wood])}

func _initialize() -> void:
	var legacy_tracker = Moments.new()
	for legacy_wood: Dictionary in [{}, {"carried_woods": [0]}, {"carried_woods": PackedInt32Array()}]:
		legacy_tracker.clear()
		var old_snapshot: Dictionary = legacy_wood.duplicate()
		old_snapshot["hungers"] = PackedFloat32Array([90.0])
		legacy_tracker.update(2, 0, old_snapshot, Vector2i.ZERO, true, false, [Vector2i.ZERO], 0.0)
		old_snapshot["hungers"] = PackedFloat32Array([60.0])
		check(not legacy_tracker.update(2, 0, old_snapshot, Vector2i.ZERO, true, false, [Vector2i.ZERO], 0.0)["meal"], "legacy hunger drop without valid per-row Pass5 wood does not add meal pose")
	var tracker = Moments.new()
	var tile := Vector2i(8, 8)
	var hearths: Array = [Vector2i.ZERO]
	check(not tracker.update(1, 0, snap(90), tile, true, true, hearths, 0.0)["meal"], "first snapshot cannot imply a meal")
	check(tracker.update(1, 0, snap(60), tile, true, true, hearths, 0.1)["meal"], "30 hunger drop at inclusive radius eight starts meal")
	check(tracker.update(1, 0, snap(60), tile, true, true, hearths, 2.9)["meal"], "meal lasts three render seconds without retriggering")
	check(not tracker.update(1, 0, snap(60), tile, true, true, hearths, 0.11)["meal"], "meal expires")
	tracker.clear()
	tracker.update(1, 0, snap(90), tile, true, true, hearths, 0.0)
	check(not tracker.update(1, 0, snap(71), tile, true, true, hearths, 0.0)["meal"], "drop below threshold ignored")
	check(not tracker.update(1, 0, snap(40), tile, false, true, hearths, 0.0)["meal"], "moving agent cannot start meal")
	check(not tracker.update(1, 0, snap(10), Vector2i(9, 0), true, true, hearths, 0.0)["meal"], "outside radius ignored")
	tracker.clear()
	tracker.update(1, 0, snap(90), tile, true, true, hearths, 0.0)
	check(tracker.update(1, 0, snap(70), tile, true, true, hearths, 0.0)["meal"], "exact threshold starts meal")
	check(not tracker.update(1, 0, snap(70), tile, false, true, hearths, 0.0)["meal"], "movement ends meal")
	tracker.retire(1)
	check(not tracker.update(1, 0, snap(0), tile, true, true, hearths, 0.0)["meal"], "retired identity has no stale hunger")
	tracker.clear()
	tracker.update(1, 0, snap(90, 0), tile, false, true, hearths, 0.0)
	check(tracker.update(1, 0, snap(90, 10), tile, false, true, hearths, 0.1)["pickup"], "pickup starts while walking onto deadwood")
	check(tracker.update(1, 0, snap(90, 10), tile, false, true, hearths, 0.9)["pickup"], "pickup survives sub-tile interpolation")
	check(not tracker.update(1, 0, snap(90, 10), tile, true, true, hearths, 0.11)["pickup"], "pickup expires after one second")
	check(tracker.update(1, 0, snap(90, 20), tile, true, true, hearths, 0.0)["pickup"], "later rise retriggers pickup")
	check(not tracker.update(1, 0, snap(90, 20), tile + Vector2i.RIGHT, false, true, hearths, 0.0)["pickup"], "leaving tile cancels pickup")
	check(not tracker.update(1, 0, snap(90, 30), tile, true, false, hearths, 0.0)["pickup"], "wood rise away from deadwood ignored")
	for bad: Dictionary in [{}, {"hungers": [60], "carried_woods": [1]}, {"hungers": PackedFloat32Array(), "carried_woods": PackedInt32Array()}]:
		var result: Dictionary = tracker.update(1, 0, bad, tile, true, true, hearths, 0.0)
		check(not result["pickup"] and not result["meal"], "missing, wrongly typed or short optional fields preserve legacy poses")
	print("MEAL_POSE ", "PASS" if failures == 0 else "FAIL", ": failures=", failures)
	quit(0 if failures == 0 else 1)
