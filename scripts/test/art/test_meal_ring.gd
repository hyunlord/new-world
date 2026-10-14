extends SceneTree
const Seats = preload("res://scripts/ui/camp_seats.gd")
const Renderer = preload("res://scripts/ui/agent_renderer.gd")
const Selection = preload("res://scripts/core/settlement_selection.gd")
var failures: int = 0

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	var cells := PackedByteArray()
	cells.resize(64 * 64)
	var terrain: Dictionary = {"width": 64, "height": 64, "terrain": cells}
	var feet := Vector2(456, 42) + Vector2(30, 30) * 16.0
	var rows: Array[Dictionary] = [
		{"id": 2, "visible": true, "moving": true, "walking_direction": Vector2.DOWN, "ipos": feet},
		{"id": 3, "visible": true, "moving": false, "activity": "idle", "ipos": feet}]
	var positions: Dictionary = {2: feet, 3: feet}
	var rects: Dictionary = {2: Rect2(-8, -29, 16, 30), 3: Rect2(-8, -29, 16, 30)}
	for zoom: float in [2.0, 4.0, 6.0]:
		var canvas := Transform2D.IDENTITY.scaled(Vector2.ONE * zoom)
		for direction: Vector2 in [Vector2.DOWN, Vector2.UP, Vector2.LEFT, Vector2.RIGHT, Vector2(1, 1).normalized()]:
			rows[0]["walking_direction"] = direction
			var offsets: Dictionary = Seats.walker_offsets(rows, positions, rects, terrain, {}, canvas)
			var shift: Vector2 = offsets.get(2, Vector2.ZERO)
			var screen: Vector2 = canvas.basis_xform(shift)
			check(screen.length() > 0.0 and screen.length() <= 3.00001, "overlapping walker shifts at most three screen pixels at each zoom")
			check(absf(screen.dot(direction)) < 0.001, "lateral displacement perpendicular to movement")
			var before: float = rects[2].get_area()
			check((rects[2] as Rect2).intersection(Rect2(shift + (rects[3] as Rect2).position, (rects[3] as Rect2).size)).get_area() < before, "offset improves overlap")
			var reversed: Array[Dictionary] = rows.duplicate(true)
			reversed.reverse()
			check(Seats.walker_offsets(reversed, positions, rects, terrain, {}, canvas) == offsets, "ID determinism independent of input order")
			check(rows[0]["ipos"] == feet and positions[2] == feet, "simulation and caller positions never mutated")
	rows[0]["walking_direction"] = Vector2.DOWN
	rows[0]["moving"] = false
	rows[0]["activity"] = "walk"
	check(Seats.walker_offsets(rows, positions, rects, terrain, {}, Transform2D.IDENTITY).get(2, Vector2.ZERO) != Vector2.ZERO, "paused walk retains observed direction")
	positions[3] = feet + Vector2(100, 100)
	check(Seats.walker_offsets(rows, positions, rects, terrain, {}, Transform2D.IDENTITY).get(2, Vector2.ZERO) == Vector2.ZERO, "separate walkers never shift")
	positions[2] = feet + Vector2(7.5, 0)
	positions[3] = positions[2]
	var blocked: Dictionary = {Vector2i(31, 30): true}
	var safe: Vector2 = Seats.walker_offsets(rows, positions, rects, terrain, blocked, Transform2D.IDENTITY).get(2, Vector2.ZERO)
	check(safe.x < 0.0, "building rejects rightward candidate")
	cells[30 * 64 + 31] = 4
	terrain["terrain"] = cells
	safe = Seats.walker_offsets(rows, positions, rects, terrain, {}, Transform2D.IDENTITY).get(2, Vector2.ZERO)
	check(safe.x < 0.0, "water rejects rightward candidate")
	check(Seats.walker_offsets(rows, positions, rects, terrain, {Vector2i(30, 30): true}, Transform2D.IDENTITY).get(2, Vector2.ZERO) == Vector2.ZERO, "no safe candidate returns zero")
	_test_renderer(feet, terrain)
	print("MEAL_RING failures=", failures)
	quit(1 if failures else 0)

func _test_renderer(feet: Vector2, terrain: Dictionary) -> void:
	var renderer = Renderer.new()
	root.add_child(renderer)
	renderer.set_process(false)
	renderer.ingest_chill_snapshot({"agent_ids": PackedInt64Array([2]), "xs": PackedInt32Array([30]), "ys": PackedInt32Array([29])}, 1)
	renderer.ingest_chill_snapshot({"agent_ids": PackedInt64Array([2]), "xs": PackedInt32Array([30]), "ys": PackedInt32Array([30])}, 2)
	check(renderer._walking_directions[2] == Vector2.DOWN, "synchronous capture observes actual tile movement before drawing")
	renderer.ingest_chill_snapshot({"agent_ids": PackedInt64Array([2]), "xs": PackedInt32Array([30]), "ys": PackedInt32Array([30])}, 3)
	check(renderer._walking_directions[2] == Vector2.DOWN, "stationary capture preserves last observed direction")
	renderer._objects.scale = Vector2.ONE * 4.0
	var rows: Array[Dictionary] = []
	for id: int in [2, 3]:
		var sprite := Sprite2D.new()
		renderer._objects.add_child(sprite)
		sprite.set_meta("frame_size", Vector2i(16, 30))
		sprite.set_meta("anchor", Vector2(8, 29))
		renderer._sprites[id] = sprite
		rows.append({"id": id, "visible": true, "moving": true, "activity": "walk", "walking_direction": Vector2.DOWN, "ipos": feet, "tile": Vector2i(30, 30)})
	var context: Dictionary = {"hearths": [], "seat_blocked": {}, "terrain": terrain}
	renderer._apply_drawn_positions(rows, context, 1.15)
	var first: Vector2 = renderer._sprites[2].position
	check(not is_equal_approx(first.x, roundf(first.x)), "fractional world offset survives final world rounding at zoom four")
	check((renderer._sprites[2].get_meta("walker_lateral_screen_offset") as Vector2).length() <= 3.00001, "renderer screen bound")
	renderer._apply_drawn_positions(rows, context, 1.15)
	check(renderer._sprites[2].position == first, "lateral offsets do not accumulate into smoothing")
	Selection.layers_enabled = false
	var outline := Sprite2D.new()
	outline.name = "SettlementOutline"
	renderer._sprites[2].add_child(outline)
	Renderer._apply_settlement_outline(renderer._sprites[2], 2)
	check(not outline.visible and renderer._sprites[2].visible, "A30 hides outline while keeping base sprite")
	Selection.layers_enabled = true
	renderer.free()
