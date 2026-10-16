extends SceneTree
const Order = preload("res://scripts/test/art/capture_sprite_order_p11.gd")
var _failed: bool = false

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	var objects := Node2D.new()
	objects.name = "Objects"
	objects.y_sort_enabled = true
	objects.z_index = 4
	root.add_child(objects)
	var body: Sprite2D = _sprite("Body", 10.0)
	objects.add_child(body)
	var back: Sprite2D = _sprite("Behind", -500.0)
	back.show_behind_parent = true
	body.add_child(back)
	var icon: Sprite2D = _sprite("Icon", -1000.0)
	body.add_child(icon)
	var tree: Sprite2D = _sprite("Tree", 11.0)
	objects.add_child(tree)
	var elevated: Sprite2D = _sprite("ElevatedBehind", 0.0)
	elevated.show_behind_parent = true
	elevated.z_index = 1
	body.add_child(elevated)
	var absolute: Sprite2D = _sprite("Absolute", 20.0)
	absolute.z_as_relative = false
	absolute.z_index = 0
	objects.add_child(absolute)
	var order := Order.new()
	var rows: Array[Dictionary] = order.collect(objects)
	_check(order.errors.is_empty(), "supported hierarchy")
	var names: Array[String] = []
	for row: Dictionary in rows:
		names.append(String(row["node_path"]).get_file())
	_check(names == ["Absolute", "Behind", "Body", "Icon", "Tree", "ElevatedBehind"], "z buckets and parent-relative group order")
	_check(int(rows[2]["draw_rank"]) < int(rows[3]["draw_rank"]), "own icon above body")
	_check(int(rows[1]["draw_rank"]) < int(rows[2]["draw_rank"]), "own behind child below body")
	body.y_sort_enabled = true
	order.collect(objects)
	_check(not order.errors.is_empty(), "unsupported nested y-sort refuses completeness")
	body.y_sort_enabled = false
	objects.queue_free()
	quit(1 if _failed else 0)

func _sprite(label: String, y: float) -> Sprite2D:
	var sprite := Sprite2D.new()
	sprite.name = label
	sprite.position.y = y
	var pixels: Image = Image.create(10, 10, false, Image.FORMAT_RGBA8)
	pixels.fill(Color.WHITE)
	sprite.texture = ImageTexture.create_from_image(pixels)
	return sprite

func _check(passed: bool, label: String) -> void:
	if not passed:
		_failed = true
		push_error(label)
