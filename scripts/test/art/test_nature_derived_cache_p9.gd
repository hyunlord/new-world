extends SceneTree

const CANOPY = preload("res://scripts/ui/nature_canopy.gd")
const SEASON = preload("res://scripts/ui/art_season_tint.gd")

class FakeWorld extends Node:
	var clock: Dictionary = {"season": 3, "day_of_season": 1, "days_per_season": 30, "time_of_day": 0.0}
	func get_time_snapshot() -> Dictionary:
		return clock
	func get_terrain_snapshot() -> Dictionary:
		return {}
	func get_building_snapshot() -> Dictionary:
		return {}
	func get_construction_snapshot() -> Dictionary:
		return {}
	func get_resource_snapshot() -> Dictionary:
		return {}
	func get_settlement_snapshot() -> Dictionary:
		return {}

var checks: int = 0

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	var source: String = FileAccess.get_file_as_string("res://scripts/ui/nature_renderer.gd")
	var script := GDScript.new()
	script.source_code = source.replace("var _world_sim: WorldSimNode", "var _world_sim: Node")
	assert(script.reload() == OK)
	var renderer := Node2D.new()
	renderer.set_script(script)
	var world := FakeWorld.new()
	var objects := Node2D.new()
	renderer.set("_world_sim", world)
	renderer.set("_objects", objects)
	var image := Image.create(32, 40, false, Image.FORMAT_RGBA8)
	image.fill(Color.WHITE)
	var tree := Sprite2D.new()
	tree.texture = ImageTexture.create_from_image(image)
	tree.centered = false
	tree.offset = Vector2(-16, -39)
	tree.position = Vector2(100, 100)
	tree.set_meta("art_kind", "tree")
	objects.add_child(tree)
	var agent := Sprite2D.new()
	agent.position = Vector2(100, 90)
	agent.set_meta("art_kind", "agent")
	objects.add_child(agent)
	var sprites: Dictionary = {"tree": tree}
	renderer.set("_sprites", sprites)
	_observe(renderer, world, objects, sprites)
	var cached: Array = renderer.get("_canopy_geometry")[tree.get_instance_id()]
	_observe(renderer, world, objects, sprites)
	assert(is_same(cached, renderer.get("_canopy_geometry")[tree.get_instance_id()]))
	for feet: Vector2 in [Vector2(100, 101), Vector2(140, 90), Vector2(100, 90)]:
		agent.position = feet
		_observe(renderer, world, objects, sprites)
	agent.visible = false
	_observe(renderer, world, objects, sprites)
	agent.visible = true
	_observe(renderer, world, objects, sprites)
	tree.position.x += 80
	_observe(renderer, world, objects, sprites)
	tree.position.x -= 80
	tree.offset.x += 80
	_observe(renderer, world, objects, sprites)
	tree.offset.x -= 80
	tree.region_enabled = true
	tree.region_rect = Rect2(0, 0, 4, 4)
	_observe(renderer, world, objects, sprites)
	tree.region_enabled = false
	tree.texture = ImageTexture.create_from_image(Image.create(4, 4, false, Image.FORMAT_RGBA8))
	_observe(renderer, world, objects, sprites)
	tree.texture = ImageTexture.create_from_image(image)
	tree.set_meta("art_kind", "shrub")
	_observe(renderer, world, objects, sprites)
	tree.set_meta("art_kind", "tree")
	world.clock["time_of_day"] = 6.0
	_observe(renderer, world, objects, sprites)
	var extra: Sprite2D = tree.duplicate()
	objects.add_child(extra)
	sprites["extra"] = extra
	_observe(renderer, world, objects, sprites)
	sprites.erase("tree")
	_observe(renderer, world, objects, sprites)
	assert(not renderer.get("_canopy_geometry").has(tree.get_instance_id()))
	sprites["tree"] = tree
	_observe(renderer, world, objects, sprites)
	# A reset refresh may clear geometry even when the next inputs are identical.
	sprites.clear()
	renderer.call("refresh_snapshot")
	assert(renderer.get("_canopy_geometry").is_empty())
	sprites["tree"] = tree
	_observe(renderer, world, objects, sprites)
	renderer.call("_reconcile")
	_observe(renderer, world, objects, sprites)
	renderer.free()
	objects.free()
	world.free()
	print("NATURE_DERIVED_CACHE_P9 PASS checks=", checks)
	quit()

func _observe(renderer: Node, world: FakeWorld, objects: Node2D, sprites: Dictionary) -> void:
	var heads: Dictionary = CANOPY.buckets(objects)
	var tint: Color = SEASON.resolve(world.clock)
	renderer.call("_update_canopies")
	for sprite: Sprite2D in sprites.values():
		var opacity: float = CANOPY.alpha(sprite, heads) if str(sprite.get_meta("art_kind")) == "tree" else 1.0
		assert(sprite.modulate == Color(tint.r, tint.g, tint.b, opacity))
		checks += 1
