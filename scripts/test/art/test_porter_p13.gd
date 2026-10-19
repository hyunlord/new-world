extends SceneTree
var failures: int = 0
func check(value: bool, message: String) -> void:
	if not value:
		failures += 1
		push_error(message)
func _initialize() -> void:
	call_deferred("run")
func run() -> void:
	var porter: GDScript = load("res://scripts/ui/porter_presentation_p12.gd")
	check(porter != null, "porter presentation helper exists")
	if porter == null:
		quit(1)
		return
	var rows: Dictionary = porter.rows({"ids": PackedInt64Array([9002, -9001]), "from": PackedInt64Array([3, 4]), "to": PackedInt64Array([4, 3]), "kind": PackedByteArray([0, 1]), "amount": PackedInt32Array([12, 9]), "stage": PackedByteArray([0, 1])})
	check(rows.has(-9001) and not rows.has(1), "join uses signed entity bits, never row index or AgentId")
	check(porter.activity(rows[9002], 0, 0) == "walk", "departure amount does not imply actual load")
	check(porter.activity(rows[-9001], 0, 4) == "carry_wood", "failed return keeps actual wood load")
	check(porter.activity(rows[-9001], 2, 0) == "carry", "actual food chooses food pose despite row kind")
	check(porter.activity(rows[-9001], 0, 0) == "walk", "successful return walks")
	var settlements := {"settlement_ids": PackedInt32Array([4, 3]), "formation_xs": PackedInt32Array([20, 10]), "formation_ys": PackedInt32Array([21, 11]), "centroid_xs": PackedInt32Array([999, 999])}
	check(porter.destination(rows[9002], settlements) == Vector2i(20, 21), "outbound destination is receiver hearth, not centroid")
	check(porter.destination(rows[-9001], settlements) == Vector2i(20, 21), "return destination is sender hearth")
	var body := Sprite2D.new()
	var icon := Sprite2D.new()
	root.add_child(body)
	body.add_child(icon)
	porter.apply_icon(body, icon, rows[9002], 4.0)
	check(icon.visible and icon.texture.get_size() == Vector2(16,16) and icon.modulate.a == 1.0, "outbound native glyph")
	porter.apply_icon(body, icon, rows[-9001], 1.0)
	check(icon.scale == Vector2(2,2) and is_equal_approx(icon.modulate.a,0.6), "low zoom return enlarged and translucent")
	check(porter.texture(0).get_image().get_pixel(10,8) != porter.texture(1).get_image().get_pixel(10,8), "food and wood hues differ")
	body.free()
	print("PORTER_P12 failures=", failures)
	quit(1 if failures else 0)
