extends SceneTree
const Warnings = preload("res://scripts/ui/need_bar_renderer.gd")
func _initialize() -> void:
	assert(Warnings.critical_need(74.99, 30.0, 20.0) == -1)
	assert(Warnings.critical_need(75.0, 30.0, 20.0) == 0)
	assert(Warnings.critical_need(80.0, 95.0, 90.0) == 1)
	assert(Warnings.critical_need(80.0, 95.0, 96.0) == 2)
	assert(Warnings.critical_need(90.0, 90.0, 90.0) == 0)
	var rows: Array[Dictionary] = Warnings.warning_rows({
		"agent_ids": PackedInt64Array([12, 13, 14, 15]),
		"hungers": PackedFloat32Array([74, 75, 80, 20]),
		"thirsts": PackedFloat32Array([74, 70, 90, 20]),
		"sleeps": PackedFloat32Array([74, 70, 99, 76]),
	})
	assert(rows.size() == 3)
	assert(rows[0] == {"agent_id": 13, "kind": 0})
	assert(rows[1] == {"agent_id": 14, "kind": 2})
	assert(rows[2] == {"agent_id": 15, "kind": 2})
	assert(Warnings.warning_rows({"agent_ids": PackedInt64Array([1])}).is_empty())
	var renderer: Node = Warnings.new()
	assert(renderer.get_drawn_icon_count() == 0)
	assert(renderer.get_drawn_icon_agent_ids().is_empty())
	renderer.free()
	print("NEED WARNING PASS: exact75 threshold, one worst icon, stable ids, no fake headless draw count")
	quit(0)
