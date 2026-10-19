extends SceneTree
const Icons = preload("res://scripts/ui/activity_icons_p11.gd")
const Summary = preload("res://scripts/ui/village_activity_summary_p11.gd")
const Labels = preload("res://scripts/ui/settlement_labels.gd")
const Activity = preload("res://scripts/ui/agent_activity.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
var failures: int = 0

class FakeWorld extends Node:
	var tick: int = 10
	var calls: int = 0
	func get_time_snapshot() -> Dictionary:
		return {"tick": tick}
	func get_society_snapshot() -> Dictionary:
		return {"current_tick": tick, "settlement_ids": PackedInt32Array([7]), "home_agent_ids": PackedInt64Array([100]), "home_settlement_ids": PackedInt32Array([7])}
	func get_settlement_snapshot() -> Dictionary:
		return {"settlement_ids": PackedInt32Array([7]), "formation_xs": PackedInt32Array([1]), "formation_ys": PackedInt32Array([1])}
	func get_agent_snapshot() -> Dictionary:
		return {"ids": PackedInt64Array([900]), "agent_ids": PackedInt64Array([100])}
	func get_activity_snapshot() -> Dictionary:
		calls += 1
		return {"ids": PackedInt64Array([900]), "kinds": PackedByteArray([2]), "until_tick": PackedInt64Array([30])}

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	var sprite := Sprite2D.new()
	var icon := Sprite2D.new()
	root.add_child(sprite)
	sprite.add_child(icon)
	var shapes: Dictionary = {}
	for kind: int in 7:
		var texture: Texture2D = Icons.texture(kind)
		check(texture != null and texture.get_width() == (22 if kind == 1 else 20) and texture.get_height() == (22 if kind == 1 else 20), "badge adds two source pixels around 16/18 pictogram")
		var image: Image = Icons.pictogram_image(kind)
		var silhouette: String = ""
		for y: int in image.get_height():
			for x: int in image.get_width():
				silhouette += "1" if image.get_pixel(x, y).a > 0.0 else "0"
		check(not shapes.has(silhouette), "each pictogram has a distinct alpha silhouette excluding common badge")
		shapes[silhouette] = true
		var choice: Dictionary = Activity.with_daily_icon({"sprite_id": "fixture"}, kind)
		check(choice["activity_icon"] == Icons.icon_id(kind), "all seven use the existing icon path")
		check(not Activity.apply_activity_icon(sprite, icon, choice, 1.99), "icons hidden below zoom2")
		check(Activity.apply_activity_icon(sprite, icon, choice, 2.0), "icons visible at zoom2")
		check(icon.position == Vector2(0, sprite.offset.y - 2.0), "existing anchor retained")
	check(Activity.with_daily_icon({}, 0).is_empty(), "no invented pose on missing sprite")
	sprite.free()
	Selection.reset()
	Selection.observe({"settlement_ids": PackedInt32Array([7, 8]), "home_agent_ids": PackedInt64Array([10, 11, 12, 13, 14, 15, 16, 17]), "home_settlement_ids": PackedInt32Array([7, 7, 7, 7, 7, 7, 8, 7])})
	var agents: Dictionary = {"ids": PackedInt64Array([100, 101, 102, 103, 104, 105, 106, 107]), "agent_ids": PackedInt64Array([10, 11, 12, 13, 14, 15, 16, 17])}
	var activities: Dictionary = {"ids": PackedInt64Array([107, 106, 105, 104, 103, 102, 101, 100, 999]), "kinds": PackedByteArray([6, 5, 4, 3, 2, 2, 0, 0, 1]), "until_tick": PackedInt64Array([-1, 30, 30, 30, 30, 30, 30, 30, 30])}
	var counts: Dictionary = Summary.collect(activities, agents)
	check(counts[7] == [{"kind": 0, "count": 2}, {"kind": 2, "count": 2}, {"kind": 3, "count": 1}], "identity join, arrival filter, top3 and ties by kind")
	check(counts[8] == [{"kind": 5, "count": 1}], "separate village despite snapshot order")
	var summary := Summary.new()
	root.add_child(summary)
	summary.display(counts[7], 1.0, false)
	check(summary.visible and summary.width > 0.0, "summary visible at Z1")
	summary.display(counts[7], 2.0, false)
	check(not summary.visible, "summary hides at Z2")
	summary.display(counts[7], 1.0, true)
	check(not summary.visible, "abandoned village hides summary")
	summary.display([], 1.0, false)
	check(not summary.visible, "empty summary hides")
	summary.free()
	Selection.reset()
	var world := FakeWorld.new()
	var layer := Labels.new()
	root.add_child(layer)
	layer.set_process(false)
	layer.configure(world)
	for _frame: int in 120:
		layer._process(0.016)
	check(world.calls == 1, "120 fixed-tick frames read activities once")
	world.tick += 1
	layer._process(0.016)
	check(world.calls == 2, "new tick refreshes activities")
	check(layer.capture_metadata()["activity_summaries"].size() == 1, "sidecar includes summary geometry and rows")
	layer._reset()
	check(layer._activity_counts.is_empty() and layer._summaries.is_empty(), "world reset clears summary cache")
	layer.free()
	world.free()
	Selection.reset()
	print("ACTIVITY_P11 failures=", failures)
	quit(0 if failures == 0 else 1)
