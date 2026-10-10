extends SceneTree

class SnapshotDouble extends Node:
	var terrain: Dictionary = {}
	var buildings: Dictionary = {}
	var sites: Dictionary = {}
	var resources: Dictionary = {}

	func get_terrain_snapshot() -> Dictionary:
		return terrain

	func get_building_snapshot() -> Dictionary:
		return buildings

	func get_construction_snapshot() -> Dictionary:
		return sites

	func get_resource_snapshot() -> Dictionary:
		return resources

var _failures: int = 0

func _init() -> void:
	call_deferred("_run")

func _check(condition: bool, message: String) -> void:
	if not condition:
		push_error(message)
		_failures += 1

func _run() -> void:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var world: Node = main.get_node("WorldSim")
	world.call("set_sim_speed", 0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	await process_frame
	world.call("advance_ticks", 1)
	var nature: Node = main.get_node("NatureRenderer")
	nature.call("refresh_snapshot")
	await process_frame
	var excluded: Dictionary = {}
	for method: String in ["get_building_snapshot","get_construction_snapshot"]:
		var snapshot: Dictionary = world.call(method)
		var xs: PackedInt32Array = snapshot["xs"]
		var ys: PackedInt32Array = snapshot["ys"]
		for i: int in range(xs.size()):
			excluded[Vector2i(xs[i],ys[i])] = true
	var food: Dictionary = world.call("get_resource_snapshot")
	var food_tiles: Dictionary = {}
	for i: int in range(food["xs"].size()):
		if int(food["kinds"][i]) == 0:
			food_tiles[Vector2i(food["xs"][i],food["ys"][i])] = true
	var sprites: Dictionary = nature.get("_sprites")
	_check(not sprites.is_empty(), "nature populated from real terrain")
	var trees: int = 0
	var before: Dictionary = {}
	for key: String in sprites:
		var sprite: Sprite2D = sprites[key] as Sprite2D
		var tile: Vector2i = Vector2i(int(sprite.get_meta("tile_x")),int(sprite.get_meta("tile_y")))
		_check(not excluded.has(tile), "nature clears occupied building/site tiles")
		_check(sprite.get_parent() == main.get_node("WorldObjects"), "nature shares ysorted object parent")
		var jitter: Vector2i = sprite.get_meta("jitter")
		_check(sprite.position == Vector2(448,28)+Vector2(tile*16+Vector2i(8,15)+jitter), "nature base point includes stable jitter")
		if str(sprite.get_meta("art_kind")) == "tree":
			trees += 1
			_check(not food_tiles.has(tile), "trees leave food source tiles clear")
		before[key] = [sprite.position,sprite.get_meta("sprite_id")]
	_check(trees > 0 and trees < 1500, "tree node population budget")
	var terrain: Dictionary = world.call("get_terrain_snapshot")
	var width: int = int(terrain["width"])
	var codes: PackedByteArray = terrain["terrain"]
	var expected_trees: int = 0
	for i: int in range(codes.size()):
		var tile: Vector2i = Vector2i(i%width, floori(float(i)/width))
		if codes[i] == 1 and not excluded.has(tile) and not food_tiles.has(tile):
			var h: int = ((tile.x*73856093) ^ (tile.y*19349663)) & 0x7fffffff
			expected_trees += 0 if h%100 < 12 else (1 if h%100 < 85 else 2)
	_check(trees == expected_trees, "tree count follows every forest tile hash")
	nature.call("refresh_snapshot")
	for key: String in sprites:
		var sprite: Sprite2D = sprites[key] as Sprite2D
		_check(before[key] == [sprite.position,sprite.get_meta("sprite_id")], "nature refresh is deterministic")
	await _check_construction_clearance(nature, world)
	main.free()
	print("test_nature_clearance: ", "PASS" if _failures == 0 else "FAIL")
	quit(0 if _failures == 0 else 1)

func _check_construction_clearance(original: Node, world: Node) -> void:
	var source: String = FileAccess.get_file_as_string("res://scripts/ui/nature_renderer.gd")
	_check(source.count("var _world_sim: WorldSimNode") == 1, "test seam preserves exactly one snapshot dependency")
	var script: GDScript = GDScript.new()
	script.source_code = source.replace("var _world_sim: WorldSimNode", "var _world_sim: Node")
	_check(script.reload() == OK, "nature source compiles with snapshot test double")
	var mock: SnapshotDouble = SnapshotDouble.new()
	mock.terrain = world.call("get_terrain_snapshot")
	mock.buildings = world.call("get_building_snapshot")
	mock.sites = world.call("get_construction_snapshot").duplicate(true)
	mock.resources = world.call("get_resource_snapshot")
	var renderer: Node2D = Node2D.new()
	renderer.set_script(script)
	renderer.set("_world_sim", mock)
	var objects: Node2D = Node2D.new()
	var shadows: Node2D = Node2D.new()
	renderer.set("_objects", objects)
	renderer.set("_shadows", shadows)
	renderer.set("_shadow_texture", original.get("_shadow_texture"))
	renderer.call("refresh_snapshot")
	var sprites: Dictionary = renderer.get("_sprites")
	var contacts: Dictionary = renderer.get("_contacts")
	var occupied: Vector2i = Vector2i(-1,-1)
	for key: String in sprites:
		var sprite: Sprite2D = sprites[key] as Sprite2D
		if str(sprite.get_meta("art_kind")) == "tree":
			occupied = Vector2i(int(sprite.get_meta("tile_x")),int(sprite.get_meta("tile_y")))
			break
	_check(occupied.x >= 0, "construction test starts on populated nature tile")
	var removed_ids: Array[int] = []
	var removed_keys: Array[String] = []
	for key: String in sprites:
		var sprite: Sprite2D = sprites[key] as Sprite2D
		if Vector2i(int(sprite.get_meta("tile_x")),int(sprite.get_meta("tile_y"))) == occupied:
			removed_ids.append(sprite.get_instance_id())
			removed_ids.append((contacts[key] as Sprite2D).get_instance_id())
			removed_keys.append(key)
	var xs: PackedInt32Array = mock.sites.get("xs",PackedInt32Array())
	var ys: PackedInt32Array = mock.sites.get("ys",PackedInt32Array())
	xs.append(occupied.x)
	ys.append(occupied.y)
	mock.sites["xs"] = xs
	mock.sites["ys"] = ys
	# Reconcile, not reset: the occupancy fingerprint must detect the added site.
	renderer.call("_reconcile")
	for key: String in removed_keys:
		_check(not sprites.has(key) and not contacts.has(key), "site clears both sprite and shadow pools")
	for key: String in sprites:
		var sprite: Sprite2D = sprites[key] as Sprite2D
		_check(Vector2i(int(sprite.get_meta("tile_x")),int(sprite.get_meta("tile_y"))) != occupied, "occupied site has no remaining nature props")
	await process_frame
	for instance_id: int in removed_ids:
		_check(not is_instance_id_valid(instance_id), "removed nature and shadow nodes freed after refresh")
	objects.free()
	shadows.free()
	renderer.free()
	mock.free()
