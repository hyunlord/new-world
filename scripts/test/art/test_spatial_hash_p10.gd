extends SceneTree
const Candidate = preload("res://scripts/ui/camp_seats.gd")
const Reference = preload("res://scripts/test/art/camp_seats_reference_p10.gd")
var failures: int = 0
var comparisons: int = 0

func check(ok: bool, message: String) -> void:
	if not ok:
		push_error(message)
		failures += 1

func row(agent_id: int, walking: bool = false) -> Dictionary:
	return {"id": agent_id, "visible": true, "moving": walking,
		"activity": "walk" if walking else "idle", "walking_direction": Vector2.RIGHT}

func _initialize() -> void:
	_test_index()
	var cells := PackedByteArray()
	cells.resize(64 * 64)
	var terrain: Dictionary = {"width": 64, "height": 64, "terrain": cells}
	_test_nearest_global(terrain)
	_test_varied_histories(terrain)
	print("SPATIAL HASH P10 %s comparisons=%d" % ["PASS" if failures == 0 else "FAIL", comparisons])
	quit(1 if failures else 0)

func _test_index() -> void:
	var index := Candidate.SpatialIndex.new(16.0)
	var rectangles: Array[Rect2] = [Rect2(-40.5, -30.25, 90.75, 80.125),
		Rect2(-17.2, -9.3, 32.75, 19.25), Rect2(-16, -16, 0, 12), Rect2(16, 0, 16, 16)]
	for ordinal: int in range(rectangles.size()):
		index.insert(rectangles[ordinal], ordinal)
	for probe: Rect2 in [Rect2(-16.125, -10.5, 40.75, 25.125), Rect2(-100, -100, 1, 1), Rect2(0, 0, 16, 16)]:
		var expected: float = 0.0
		for rect: Rect2 in rectangles:
			expected += probe.intersection(rect).get_area()
		check(index.overlap(probe) == expected, "exact ordinal sum, negative cells, boundaries and multi-cell deduplication")
		var found: Array = index.query(probe)
		var sorted: Array = found.duplicate()
		sorted.sort()
		check(found == sorted, "query preserves original ordinal order")
	index.move_agent(0, Rect2(200, 200, 10, 10))
	check(not index.query(Rect2(-40, -30, 1, 1)).has(0), "move removes old buckets")
	check(index.query(Rect2(200, 200, 1, 1)).has(0), "move inserts new buckets")
	index.insert(Rect2(200, 200, 10, 10), 0)
	index.move_agent(0, Rect2(300, 300, 10, 10))
	check(index.overlap(Rect2(300, 300, 10, 10), 0, true) == 0.0, "duplicate agent IDs excluded together")
	check(index.overlap(Rect2(300, 300, 10, 10)) == 200.0, "duplicate rows retain independent contributions")
	for width: float in [2.9999, 3.0, 3.0001]:
		var threshold := Candidate.SpatialIndex.new(18.0)
		var front: Rect2 = Rect2(0, 0, width, 10)
		threshold.insert(front, 1)
		var fronts: Array[Dictionary] = [{"target": Vector2.ZERO, "rect": front}]
		var probe: Rect2 = Rect2(0, 0, 10, 10)
		check(Candidate._covered(probe, fronts, threshold) == Reference._covered(probe, fronts), "fractional coverage around 0.30 is exact")
	check(Candidate._spatial_cell_size({1: Rect2(-90, -20, 180.2, 30)}) >= 189.0, "cell size includes largest merged footprint plus eight")

func compare_case(rows: Array[Dictionary], positions: Dictionary, rects: Dictionary, terrain: Dictionary, blocked: Dictionary, canvas: Transform2D, label: String) -> void:
	var declump: Dictionary = Reference.declump(rows, {}, positions, terrain, blocked, rects)
	var walkers: Dictionary = Reference.walker_offsets(rows, positions, rects, terrain, blocked, canvas)
	check(Candidate.declump(rows, {}, positions, terrain, blocked, rects, true) == declump, label + " hash declump")
	check(Candidate.declump(rows, {}, positions, terrain, blocked, rects, false) == declump, label + " reference branch declump")
	check(Candidate.walker_offsets(rows, positions, rects, terrain, blocked, canvas, true) == walkers, label + " hash walkers")
	check(Candidate.walker_offsets(rows, positions, rects, terrain, blocked, canvas, false) == walkers, label + " reference branch walkers")
	comparisons += 4

func _test_nearest_global(terrain: Dictionary) -> void:
	var feet := Vector2(936, 522)
	var rows: Array[Dictionary] = [row(1), row(2), row(3)]
	var positions: Dictionary = {1: feet + Vector2(0, 1), 2: feet, 3: feet + Vector2(20, 2)}
	var rects: Dictionary = {1: Rect2(1000, 0, 16, 30), 2: Rect2(-8, -20, 16, 30), 3: Rect2(-28, -20, 40, 30)}
	compare_case(rows, positions, rects, terrain, {}, Transform2D.IDENTITY, "nearest feet has nonoverlapping distant rectangle")
	positions[3] = feet + Vector2(0, 1)
	compare_case(rows, positions, rects, terrain, {}, Transform2D.IDENTITY, "nearest distance and y ties")
	rows[0]["moving"] = true
	rows[2]["activity"] = "carry_wood"
	compare_case(rows, positions, rects, terrain, {}, Transform2D.IDENTITY, "walking fronts and paused carrying")

func _test_varied_histories(terrain: Dictionary) -> void:
	var feet := Vector2(936, 522)
	var canvases: Array[Transform2D] = [Transform2D.IDENTITY,
		Transform2D(Vector2(0.05, 0), Vector2(0, 0.04), Vector2(200, -50)),
		Transform2D(Vector2(2, 0), Vector2(0, 0.7), Vector2.ZERO),
		Transform2D(0.37, Vector2(100, -200))]
	var directions: Array[Vector2] = [Vector2.RIGHT, Vector2.DOWN, Vector2(1, 1), Vector2.ZERO, Vector2(-1, 2)]
	for scenario: int in range(96):
		var rows: Array[Dictionary] = []
		var positions: Dictionary = {}
		var rects: Dictionary = {}
		for ordinal: int in range(18):
			var agent_id: int = ordinal + 1
			var value: Dictionary = row(agent_id, ordinal % 3 == 0)
			value["walking_direction"] = directions[(ordinal + scenario) % directions.size()]
			value["visible"] = (ordinal + scenario) % 11 != 0
			if ordinal % 5 == 0:
				value["moving"] = false
				value["activity"] = "walk"
			rows.append(value)
			positions[agent_id] = feet + Vector2(float((ordinal * 7 + scenario) % 31) - 15.25, float((ordinal * 3) % 19) - 9.125)
			var size := Vector2(8.5 + (ordinal + scenario) % 35, 24.75 + ordinal % 15)
			if scenario % 13 == 0 and ordinal == 4:
				size.x = 0.0
			rects[agent_id] = Rect2(Vector2(-size.x / 2.0, -size.y + 0.125), size)
		if scenario % 4 == 0:
			rows.reverse()
		if scenario % 7 == 0:
			rows.append(rows[0].duplicate())
		if scenario % 9 == 0:
			positions.erase(2)
		if scenario % 10 == 0:
			rects.erase(3)
		var blocked: Dictionary = {}
		if scenario % 3 == 0:
			blocked[Vector2i(30, 30)] = true
		var cells: PackedByteArray = terrain["terrain"]
		cells[30 * 64 + 31] = 4 if scenario % 5 == 0 else 0
		terrain["terrain"] = cells
		compare_case(rows, positions, rects, terrain, blocked, canvases[scenario % canvases.size()], "varied %d" % scenario)
