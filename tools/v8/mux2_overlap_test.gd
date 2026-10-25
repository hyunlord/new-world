# M-UX2 exact overlap-membership regression against the frozen original all-pairs loop.
# Bridge methods: none. No simulation, generated art, or runtime FPS claims.
extends SceneTree

const Layout = preload("res://scripts/world/colony/marker_layout.gd")
var _checks: int = 0
var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, valid: bool) -> void:
	_checks += 1
	_failures += 0 if valid else 1
	print("MUX2_OVERLAP_%s %s" % ["PASS" if valid else "FAIL", label])


## Original incremental bucket insertion and all-pairs marking; keep independent of optimized helper.
func _reference(rectangles: Dictionary, cell_size: float) -> Dictionary:
	var buckets: Dictionary = {}
	var overlaps: Dictionary = {}
	for bits: int in rectangles:
		var rectangle: Rect2 = rectangles[bits]
		var first: Vector2i = Vector2i((rectangle.position / cell_size).floor())
		var last: Vector2i = Vector2i((rectangle.end / cell_size).floor())
		for y: int in range(first.y, last.y + 1):
			for x: int in range(first.x, last.x + 1):
				var cell: Vector2i = Vector2i(x, y)
				for other: int in buckets.get(cell, []):
					if rectangle.intersects(rectangles[other]):
						overlaps[bits] = true
						overlaps[other] = true
				if not buckets.has(cell):
					buckets[cell] = []
				buckets[cell].append(bits)
	return overlaps


func _compare(label: String, rectangles: Dictionary) -> void:
	var expected: Dictionary = _reference(rectangles, 64.0)
	_check(label + "_reference", Layout.overlapping_people(rectangles, 64.0) == expected)
	var keys: Array = rectangles.keys()
	keys.reverse()
	var reversed: Dictionary = {}
	for bits: int in keys:
		reversed[bits] = rectangles[bits]
	_check(label + "_reverse_order", Layout.overlapping_people(reversed, 64.0) == expected and _reference(reversed, 64.0) == expected)


func _run() -> void:
	var dense: Dictionary = {}
	var sparse: Dictionary = {}
	for i: int in 128:
		dense[i] = Rect2(63.5, 63.5, 32, 64)
		sparse[i] = Rect2(i * 100, i * 100, 32, 64)
	_compare("dense_identical", dense)
	_compare("sparse", sparse)
	var chain: Dictionary = {1: Rect2(0, 0, 10, 10), 2: Rect2(5, 0, 10, 10), 3: Rect2(12, 0, 10, 10), 4: Rect2(20, 0, 10, 10)}
	_compare("asymmetric_chain", chain)
	_check("every_chain_person_has_own_neighbor_query", Layout.overlapping_people(chain, 64.0).size() == 4)
	_compare("fractional_cross_cell", {1: Rect2(63.75, 63.75, 0.5, 0.5), 2: Rect2(64, 64, 32, 64), 3: Rect2(127.5, 127.5, 64, 64)})
	_compare("negative_cells", {1: Rect2(-65, -65, 2, 2), 2: Rect2(-64.5, -64.5, 1, 1), 3: Rect2(-1, -1, 2, 2), 4: Rect2(0, 0, 4, 4)})
	_compare("touching_and_empty_rect", {1: Rect2(0, 0, 64, 64), 2: Rect2(64, 0, 64, 64), 3: Rect2(0, 64, 64, 64), 4: Rect2(32, 32, 0, 0)})
	_compare("empty_set", {})
	_compare("single_person_self_is_not_overlap", {1: Rect2(0, 0, 100, 100)})
	for sample: int in 32:
		var rectangles: Dictionary = {}
		for i: int in 96:
			var x: float = float((i * 73 + sample * 19) % 401 - 200) * 0.5
			var y: float = float((i * 41 + sample * 31) % 307 - 150) * 0.5
			var width: float = float((i * 11 + sample * 7) % 79 + 1) * 0.5
			var height: float = float((i * 17 + sample * 13) % 131 + 1) * 0.5
			rectangles[i * 7 + 3] = Rect2(x, y, width, height)
		_compare("heterogeneous_%d" % sample, rectangles)
	print("MUX2_OVERLAP_DONE checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)
