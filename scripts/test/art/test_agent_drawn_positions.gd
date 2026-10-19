extends SceneTree
const Renderer = preload("res://scripts/ui/agent_renderer.gd")
var failures: int = 0

func check(ok: bool, message: String) -> void:
	if not ok:
		push_error(message)
		failures += 1

func _initialize() -> void:
	var renderer = Renderer.new()
	var bridge := ClassDB.instantiate("WorldSimNode") as WorldSimNode
	renderer.world_sim = bridge
	var sprite := Sprite2D.new()
	renderer._sprites[1] = sprite
	var rows: Array[Dictionary] = [{"id": 1, "ipos": Vector2(500, 200), "tile": Vector2i(3, 10), "visible": true, "moving": true, "stationary_seconds": 0.0, "pose": "walk", "meal": false}]
	var context: Dictionary = {"hearths": [], "seat_blocked": {}, "terrain": {}}
	renderer._apply_drawn_positions(rows, context, 0.0)
	check(renderer.get_drawn_position(1) == Vector2(500, 200), "new actor starts at target without flying from origin")
	rows[0]["ipos"] = Vector2(516, 200)
	renderer._apply_drawn_positions(rows, context, 0.08)
	check(is_equal_approx(renderer._drawn_positions[1].x, 500.0 + 16.0 * (1.0 - exp(-1.0))), "drawn movement follows specified render-time damping")
	check(rows[0]["ipos"] == Vector2(516, 200), "drawn smoothing never modifies interpolated simulation position")
	renderer._apply_drawn_positions(rows, context, 1.15)
	check(renderer.get_drawn_position(1) == Vector2(516, 200), "capture settle reaches target exactly")
	check(not sprite.get_meta("seated"), "moving actor is not seated")
	var cells := PackedByteArray()
	cells.resize(400)
	cells.fill(4)
	context = {"hearths": [Vector2i(10, 10)], "seat_blocked": {}, "terrain": {"width": 20, "height": 20, "terrain": cells}}
	rows[0].merge({"ipos": Vector2(600, 202), "tile": Vector2i(9, 10), "moving": false, "stationary_seconds": 1.0, "pose": "idle"}, true)
	renderer._apply_drawn_positions(rows, context, 1.15)
	check(not sprite.get_meta("seated"), "water blocks every otherwise eligible camp seat")
	cells.fill(0)
	context["terrain"]["terrain"] = cells
	context["seat_blocked"] = {}
	renderer._apply_drawn_positions(rows, context, 1.15)
	check(sprite.get_meta("seated"), "terrain change rebuilds blocked seats")
	check(sprite.has_meta("seat") and sprite.get_meta("seat") == sprite.get_meta("drawn_target"), "capture records the persistent seat coordinate")
	sprite.free()
	renderer.free()
	bridge.free()
	print("agent drawn positions: %d failures" % failures)
	quit(1 if failures else 0)
