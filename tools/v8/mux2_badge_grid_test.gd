# Exact bitmap-grid and row-pruning comparisons against the frozen pre-optimization implementation.
# Bridge methods: none. Synthetic equivalence does not establish actual FPS or visual acceptance.
extends SceneTree

const Layout = preload("res://scripts/world/colony/marker_layout.gd")
const Reference = preload("res://tools/v8/fixtures/mux2_badge_grid_reference.gd")
var _checks: int = 0
var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, valid: bool) -> void:
	_checks += 1
	_failures += 0 if valid else 1
	print("MUX2_BADGE_GRID_%s %s" % ["PASS" if valid else "FAIL", label])


func _run() -> void:
	# Given fractional/negative/tiny viewports, duplicates and boundary-touching blockers.
	for case_id: int in 64:
		var viewport: Rect2 = Rect2(-100.5 + (case_id % 4) * 50, -80.25, (case_id % 8) * 47, (case_id % 7) * 41)
		var maximum: Vector2 = Vector2(20.5 + case_id % 5, 28.25 + case_id % 3)
		var blocked: Array[Rect2] = []
		for j: int in case_id % 17:
			var rectangle: Rect2 = Rect2(-120 + (j * 37 + case_id * 3) % 350, -100 + (j * 43) % 280, 12.5 + j, 31.25)
			blocked.append(rectangle)
			if j % 3 == 0:
				blocked.append(rectangle)
		var slot_origin: Vector2 = viewport.position.ceil() + Vector2.ONE
		blocked.append(Rect2(slot_origin + Vector2(maximum.ceil().x + 2.0, 0), Vector2(9, 9)))
		# When both initial grids and their subsequent dynamic reservations are evaluated.
		var actual: Dictionary = Layout.badge_grid(maximum, viewport, blocked)
		var expected: Dictionary = Reference.badge_grid(maximum, viewport, blocked)
		# Then every sorted row and all metadata agree exactly, including empty grids.
		_check("initial_grid_%d" % case_id, actual == expected)
		var reservation: Rect2 = Rect2(slot_origin + Vector2(case_id % 11, case_id % 13), maximum * 0.75)
		Layout.badge_grid_reserve(actual, reservation)
		Reference.badge_grid_reserve(expected, reservation)
		_check("dynamic_reservation_%d" % case_id, actual == expected)
	# Given ordered variable-size groups, empty/sparse rows and eventual exhaustion.
	for case_id: int in 16:
		var viewport: Rect2 = Rect2(-80.5, -60.5, 160 + (case_id % 4) * 80, 120 + (case_id % 3) * 60)
		var maximum: Vector2 = Vector2(44.5, 30.5)
		var blocked: Array[Rect2] = [Rect2(-20, -15, 100, 75), Rect2(-80.5, -60.5, viewport.size.x, 20)]
		if case_id % 4 == 0:
			blocked.append(viewport)
		var actual_grid: Dictionary = Layout.badge_grid(maximum, viewport, blocked)
		var expected_grid: Dictionary = Reference.badge_grid(maximum, viewport, blocked)
		var actual_index: Dictionary = Layout.badge_index(blocked)
		var expected_index: Dictionary = Layout.badge_index(blocked)
		for group: int in 24:
			var size: Vector2 = Vector2(24.5 + (group % 6) * 4, 28.5 + group % 3)
			var figure: Rect2 = Rect2(-100 + (group * 31 + case_id * 13) % 400, -90 + (group * 37) % 300, 16, 32)
			# When each implementation selects and reserves its next label in original order.
			var actual: Rect2 = Layout.badge_packed(figure, size, viewport, actual_index, actual_grid)
			var expected: Rect2 = Reference.badge_packed(figure, size, viewport, expected_index, expected_grid)
			# Then exact rectangles, tie choices, remaining rows and exhaustion agree.
			_check("ordered_packing_%d_%d" % [case_id, group], actual == expected and actual_grid == expected_grid)
			Layout.badge_index_add(actual_index, actual)
			Layout.badge_index_add(expected_index, expected)
	# Given symmetric sparse rows: preserve the first row/column on exact distance ties.
	var tie_viewport: Rect2 = Rect2(0, 0, 120, 136)
	var tie_size: Vector2 = Vector2(24, 30)
	var tie_blocked: Array[Rect2] = [Rect2(0, 33, 120, 68)]
	for center_x: float in [41.0, 55.0]:
		var tie_figure: Rect2 = Rect2(center_x - 8, 51, 16, 32)
		var tie_actual_grid: Dictionary = Layout.badge_grid(tie_size, tie_viewport, tie_blocked)
		var tie_expected_grid: Dictionary = Reference.badge_grid(tie_size, tie_viewport, tie_blocked)
		var tie_actual: Rect2 = Layout.badge_packed(tie_figure, tie_size, tie_viewport, Layout.badge_index(tie_blocked), tie_actual_grid)
		var tie_expected: Rect2 = Reference.badge_packed(tie_figure, tie_size, tie_viewport, Layout.badge_index(tie_blocked), tie_expected_grid)
		_check("exact_row_column_tie_%s" % center_x, tie_actual == Rect2(29, 1, 24, 30) and tie_actual == tie_expected and tie_actual_grid == tie_expected_grid)
	print("MUX2_BADGE_GRID_DONE checks=%d failures=%d" % [_checks, _failures])
	quit(0 if _failures == 0 else 1)
