extends SceneTree

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
	SimReads.mutate(world, "advance_ticks", [1])
	var renderer: Node = main.get_node("TerrainRenderer")
	renderer.call("refresh_snapshot")
	var layers: Dictionary = renderer.get("_layers")
	_check(layers.size() == 6, "six independent dual-grid terrain layers")
	for name: String in layers:
		var layer: TileMapLayer = layers[name] as TileMapLayer
		_check(layer.position == Vector2(440,20), "dual-grid layer origin offset")
		_check(layer.texture_filter == CanvasItem.TEXTURE_FILTER_NEAREST, "terrain nearest filtering")
		_check(layer.tile_set.tile_size == Vector2i(16,16), "native 16x16 terrain cells")
	var water: TileSetAtlasSource = (layers["water"] as TileMapLayer).tile_set.get_source(0) as TileSetAtlasSource
	_check(water.get_tile_animation_frames_count(Vector2i(0,15)) == 4, "water four animation frames")
	_check(is_equal_approx(water.get_tile_animation_speed(Vector2i(0,15)),3.0), "water animation 3fps")
	var snapshot: Dictionary = SimReads.read(world, "get_terrain_snapshot")
	var codes: PackedByteArray = snapshot["terrain"]
	var width: int = int(snapshot["width"])
	for index: int in range(codes.size()):
		var x: int = index%width
		var y: int = floori(float(index)/width)
		_check(bool(renderer.call("_member","forest_floor",x,y)) == (codes[index] == 1), "forest membership excludes hill")
		_check(bool(renderer.call("_member","hill",x,y)) == (codes[index] == 2 or codes[index] == 3), "hill membership includes mountain")
	var camp: Dictionary = renderer.get("_camp")
	for tile: Vector2i in camp:
		_check(codes[tile.y*width+tile.x] == 0, "camp earth only on Plain")
	_check(not camp.is_empty(), "real hearth establishes camp earth")
	main.free()
	print("test_terrain_layers: ", "PASS" if _failures == 0 else "FAIL")
	quit(0 if _failures == 0 else 1)
