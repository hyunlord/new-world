extends SceneTree
const Wood = preload("res://scripts/ui/wood_visuals.gd")
const Effects = preload("res://scripts/ui/world_effects.gd")
const Renderer = preload("res://scripts/ui/building_renderer.gd")
var failures: int = 0

func check(value: bool, message: String) -> void:
	if not value:
		failures += 1
		push_error(message)

func _initialize() -> void:
	var out: Dictionary = {"fire_lit": false, "wood_stock": 40}
	for hour: float in [18.0, 23.0, 0.0, 6.99]:
		check(Wood.cold_ash(out, hour, 0), "winter night ash with stocked rain-out hearth")
		check(Wood.hearth_frame(out, hour, 0.0, 0) == "S1_16", "cold ring despite wood")
	for hour: float in [7.0, 12.0, 17.99]:
		check(not Wood.cold_ash(out, hour, 0), "no daytime cold ash")
	check(not Wood.cold_ash(out, 2.0, -1), "no summer cold ash")
	check(not Wood.cold_ash({"fire_lit": true}, 2.0, 2), "lit hearth no cold ash")
	check(not Wood.cold_ash({}, 2.0, 2), "missing flag no invented cold")
	var booked: Dictionary = {"season": 0, "winter_severity": 1, "time_of_day": 2.0}
	check(Wood.cold_ash(out, booked.time_of_day, booked.winter_severity), "spring calendar booked winter night keeps ash")
	var renderer: Node2D = Renderer.new()
	var sprite := Sprite2D.new()
	renderer._update_cold_ash(sprite, true)
	var ash: Sprite2D = sprite.get_node("ColdAsh") as Sprite2D
	check(ash.visible and ash.texture.get_size() == Vector2(3,1), "three native ash pixels")
	check(ash.texture.get_image().get_pixel(1,0).is_equal_approx(Color("8A8A8A")), "ash exact grey")
	renderer._update_cold_ash(sprite, false)
	check(not ash.visible and not sprite.get_meta("cold_ash"), "ash clears")
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var effects: Node2D = Effects.new()
	var objects := Node2D.new()
	effects._world = main.get_node("WorldSim")
	effects._objects = objects
	objects.add_child(sprite)
	sprite.set_meta("art_kind", "building")
	sprite.set_meta("building_kind", 0)
	sprite.set_meta("art_id", 99)
	sprite.set_meta("fire_lit", false)
	effects._process(0.0)
	check(not effects._persistent.has("smoke_99"), "extinguished hearth emits no smoke")
	sprite.set_meta("fire_lit", true)
	effects._process(0.0)
	check(effects._persistent.has("smoke_99"), "lit hearth still smokes")
	sprite.set_meta("fire_lit", false)
	effects._process(0.0)
	check(not effects._persistent.has("smoke_99"), "fire-out transition removes existing smoke")
	effects.free()
	objects.free()
	main.free()
	renderer.free()
	print("COLD_HEARTH failures=", failures)
	quit(1 if failures else 0)
