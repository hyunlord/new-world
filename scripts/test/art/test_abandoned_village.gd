extends SceneTree
const Selection = preload("res://scripts/core/settlement_selection.gd")
const Labels = preload("res://scripts/ui/settlement_labels.gd")
const Wood = preload("res://scripts/ui/wood_visuals.gd")
var failures: int = 0
func _initialize() -> void:
	call_deferred("run")
func check(value: bool, message: String) -> void:
	if not value:
		failures += 1
		push_error(message)
func run() -> void:
	Selection.reset()
	Selection.observe({"settlement_ids": PackedInt32Array([7,42]), "parent_ids": PackedInt32Array([-1,7])}, {"settlement_ids": PackedInt32Array([7,42]), "formation_xs": PackedInt32Array([4,12]), "formation_ys": PackedInt32Array([4,4])})
	Selection.observe({"settlement_ids": PackedInt32Array([42]), "parent_ids": PackedInt32Array([7]), "retired_hearth_xs": PackedInt32Array([4]), "retired_hearth_ys": PackedInt32Array([4])}, {"settlement_ids": PackedInt32Array([42]), "formation_xs": PackedInt32Array([12]), "formation_ys": PackedInt32Array([4]), "fire_lit": PackedByteArray([1])})
	check(Selection.is_abandoned(7), "retired hearth retains old village number")
	check(Selection.retired_building(Vector2i(4,4)) and Selection.retired_building(Vector2i(5,4)), "retired hearth and nearest old shelter grey")
	check(not Selection.retired_building(Vector2i(8,4)), "equal-distance shared boundary never assigned abandoned")
	check(not Selection.retired_building(Vector2i(10,4)), "alive village nearby shelter remains colored")
	check(not Selection.retired_building(Vector2i(4,12)), "distant buildings outside old region unchanged")
	check(Wood.cold_ash({}, 2.0, 2), "retired unmatched hearth cold ash")
	check(Wood.glow_alpha({}, 1.0, 0.0) == 0.0, "retired unmatched hearth has no light")
	var parent := Node2D.new()
	root.add_child(parent)
	var objects := Node2D.new()
	objects.name = "WorldObjects"
	parent.add_child(objects)
	var sprite := Sprite2D.new()
	sprite.set_meta("art_kind", "building")
	sprite.set_meta("tile_x", 4)
	sprite.set_meta("tile_y", 4)
	objects.add_child(sprite)
	var layer := Labels.new()
	parent.add_child(layer)
	layer.set_process(false)
	layer._update_labels()
	layer._update_abandoned()
	check(sprite.material == layer._abandoned_material, "abandoned shader applied to rendered building")
	check(is_equal_approx(float(sprite.get_meta("abandoned_desaturation")), 0.6) and is_equal_approx(float(sprite.get_meta("abandoned_alpha")), 0.85), "requested grayscale and alpha")
	var rows: Array = layer.capture_metadata()["labels"]
	check(rows.size() == 2 and rows[0]["abandoned"], "abandoned label and live daughter both retained")
	parent.free()
	Selection.reset()
	print("ABANDONED_VILLAGE failures=", failures)
	quit(1 if failures else 0)
