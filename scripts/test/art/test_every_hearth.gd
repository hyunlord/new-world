extends SceneTree

const Wood = preload("res://scripts/ui/wood_visuals.gd")
const Effects = preload("res://scripts/ui/world_effects.gd")

class WorldDouble extends Node:
	var settlements: Dictionary = {
		"settlement_ids": PackedInt32Array([9, 2]),
		"formation_xs": PackedInt32Array([10, 30]),
		"formation_ys": PackedInt32Array([10, 30]),
		"wood_stocks": PackedInt32Array([28, 287]),
		"fire_lit": PackedByteArray([0, 1]),
	}
	func get_settlement_snapshot() -> Dictionary:
		return settlements
	func get_time_snapshot() -> Dictionary:
		return {"time_of_day": 2.0, "winter_severity": 1}

var failures: int = 0

func check(value: bool, message: String) -> void:
	if not value:
		failures += 1
		push_error(message)

func _initialize() -> void:
	var source: String = FileAccess.get_file_as_string("res://scripts/ui/building_renderer.gd")
	var renderer_script := GDScript.new()
	renderer_script.source_code = source.replace("var _world_sim: WorldSimNode", "var _world_sim: Node")
	if renderer_script.reload() != OK:
		push_error("cannot load building renderer with snapshot test double")
		quit(1)
		return
	var renderer := Node2D.new()
	renderer.set_script(renderer_script)
	var world := WorldDouble.new()
	renderer._world_sim = world
	var objects := Node2D.new()
	var tiles: Array[Vector2i] = [Vector2i(10, 10), Vector2i(30, 30), Vector2i(29, 30)]
	var sprites: Array[Sprite2D] = []
	for index: int in tiles.size():
		var sprite := Sprite2D.new()
		sprite.set_meta("art_kind", "building")
		sprite.set_meta("building_kind", 0)
		sprite.set_meta("art_id", index)
		sprite.set_meta("tile_x", tiles[index].x)
		sprite.set_meta("tile_y", tiles[index].y)
		objects.add_child(sprite)
		renderer._buildings[index] = sprite
		sprites.append(sprite)
	renderer._animate_hearths()
	check(sprites[0].get_meta("settlement_id") == 9 and sprites[0].get_meta("wood_stock") == 28, "first hearth uses its own unlit stock")
	check(sprites[1].get_meta("settlement_id") == 2 and sprites[1].get_meta("wood_stock") == 287, "second hearth uses its own lit stock")
	check(not sprites[0].get_meta("fire_lit") and sprites[1].get_meta("fire_lit"), "different fire flags reach rendered sprites")
	check(sprites[0].get_meta("sprite_id") == "S1_16" and sprites[1].get_meta("sprite_id") == "S1_02", "unlit winter ash and lit fire differ")
	check(Wood.settlement(world.settlements, tiles[2]).is_empty(), "adjacent orphan never borrows a neighbour")
	check(sprites[2].get_meta("settlement_id") == -1 and not sprites[2].get_meta("fire_lit"), "orphan has no settlement and no fire")
	check(sprites[2].get_meta("sprite_id") == "S1_16" and sprites[2].get_meta("cold_ash"), "orphan renders actual cold ash")
	for hour: float in [2.0, 12.0, 23.0]:
		check(Wood.cold_ash({}, hour, -1), "orphan remains ash outside winter")
		check(Wood.hearth_frame({}, hour, 0.0) == "S1_16", "orphan never animates flames")
	check(Wood.glow_alpha(Wood.settlement(world.settlements, tiles[0]), 1.0, 0.0) == 0.0, "unlit own hearth has no glow")
	check(Wood.glow_alpha(Wood.settlement(world.settlements, tiles[1]), 1.0, 0.0) > 0.0, "lit own hearth has glow")
	check(Wood.glow_alpha(Wood.settlement(world.settlements, tiles[2]), 1.0, 0.0) == 0.0, "orphan has no glow")
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var effects: Node2D = Effects.new()
	effects._world = main.get_node("WorldSim")
	effects._objects = objects
	effects._process(0.0)
	check(not effects._persistent.has("smoke_0") and effects._persistent.has("smoke_1") and not effects._persistent.has("smoke_2"), "only the lit settlement hearth emits smoke")
	world.settlements["settlement_ids"] = PackedInt32Array([9])
	world.settlements["formation_xs"] = PackedInt32Array([10])
	world.settlements["formation_ys"] = PackedInt32Array([10])
	renderer._animate_hearths()
	effects._process(0.0)
	check(not sprites[1].get_meta("fire_lit") and sprites[1].get_meta("cold_ash"), "retiring a formerly lit hearth clears fire and shows ash")
	check(not effects._persistent.has("smoke_1"), "retiring a hearth removes existing smoke")
	effects.free()
	objects.free()
	main.free()
	world.free()
	renderer.free()
	print("EVERY_HEARTH failures=", failures)
	quit(1 if failures else 0)
