extends SceneTree

const Seats = preload("res://scripts/ui/camp_seats.gd")
var failures: int = 0

func check(ok: bool, message: String) -> void:
	if not ok:
		failures += 1
		push_error(message)

func _initialize() -> void:
	check(not OS.has_feature("double"), "P12 differential requires single-precision Vector2")
	if not ResourceLoader.exists("res://scripts/ui/art_declump_p12.gd"):
		check(false, "P12 packed declump adapter is not implemented")
		quit(1)
		return
	var adapter: GDScript = load("res://scripts/ui/art_declump_p12.gd") as GDScript
	var bridge: Node = ClassDB.instantiate("WorldSimNode") as Node
	if bridge == null or not bridge.has_method("art_declump"):
		check(false, "P12 requires the actual art_declump bridge method")
		if bridge != null:
			bridge.free()
		quit(1)
		return
	var cells := PackedByteArray()
	cells.resize(64 * 64)
	var terrain: Dictionary = {"width": 64, "height": 64, "terrain": cells}
	var feet := Vector2(936, 522)
	var rows: Array[Dictionary] = [
		{"id": 2, "visible": true, "moving": false},
		{"id": 3, "visible": true, "moving": false},
		{"id": 2, "visible": true, "moving": false},
		{"id": -9, "visible": true, "moving": false, "activity": "carry_wood"},
		{"id": 4, "visible": true, "moving": false, "activity_place_known": true},
		{"id": 5}, {"id": 6, "visible": true}]
	var targets: Dictionary = {2: feet, 3: feet + Vector2(0, 16), -9: feet, 4: feet, 6: feet, 999: feet}
	var rects: Dictionary = {2: Rect2(-8, -29, 16, 30), 3: Rect2(-8, -29, 16, 30), -9: Rect2(-4, -10, 8, 12), 4: Rect2(-4, -10, 8, 12), 888: Rect2(-80, 0, 160, 20)}
	var original_rows: Array[Dictionary] = rows.duplicate(true)
	var original_targets: Dictionary = targets.duplicate(true)
	var original_rects: Dictionary = rects.duplicate(true)
	var blocked: Dictionary = {Vector2i(-3, 4): false, Vector2i(31, 30): true}
	var packed: Dictionary = adapter.pack_inputs(rows, targets, terrain, blocked, rects)
	check(packed["row_ids"] == PackedInt64Array([2, 3, 2, -9, 4, 5, 6]), "packing retains duplicate rows, signed ids and row order")
	check(packed["walking"] == PackedByteArray([0, 0, 0, 1, 0, 1, 1]), "walking matches paused carry and default-moving semantics")
	check(packed["target_ids"] == PackedInt64Array(targets.keys()), "packing retains every target in dictionary order")
	check(packed["rect_ids"] == PackedInt64Array(rects.keys()), "packing retains rectangles outside ordered rows")
	check(packed["blocked_xy"] == PackedInt32Array([-3, 4, 31, 30]), "blocked values irrelevant, signed key coordinates retained")
	for spatial: bool in [false, true]:
		var expected: Dictionary = Seats.declump(rows, {}, targets, terrain, blocked, rects, spatial)
		var actual: Dictionary = adapter.declump(bridge, rows, targets, terrain, blocked, rects, spatial)
		check(adapter.bit_equal(expected, actual), "actual bridge matches original key order and exact f32 output bits mode=%s" % spatial)
	check(rows == original_rows and targets == original_targets and rects == original_rects, "adapter preserves original inputs")
	check(not adapter.bit_equal({1: Vector2.ZERO}, {}), "absent result differs from zero result")
	var zero_bytes := PackedByteArray()
	zero_bytes.resize(8)
	zero_bytes.encode_u32(0, 0)
	zero_bytes.encode_u32(4, 0x80000000)
	check(not adapter.bit_equal({1: Vector2(zero_bytes.decode_float(0), zero_bytes.decode_float(0))}, {1: Vector2(zero_bytes.decode_float(4), zero_bytes.decode_float(0))}), "bit comparison distinguishes runtime-decoded signed zero")
	bridge.free()
	print("DECLUMP ADAPTER P12 failures=", failures)
	quit(1 if failures else 0)
