extends SceneTree
const SEASON = preload("res://scripts/ui/art_season_tint.gd")
const LAYOUT = preload("res://scripts/ui/nature_layout.gd")
var _failures: int = 0

func _check(condition: bool, message: String = "density invariant") -> void:
	if not condition:
		push_error(message)
		_failures += 1

func _init() -> void:
	call_deferred("_run")
func _run() -> void:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var world: Node = main.get_node("WorldSim")
	world.call("set_sim_speed",0.0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(main)
	var tick: int = 0
	while tick < 1680:
		tick = int(world.call("advance_ticks",mini(8,1680-tick)))
	var building: Node = main.get_node("BuildingRenderer")
	_check(building.call("_available","S2_08") == "S2_08","suppressed structures never resurrect direction art")
	var nature: Node = main.get_node("NatureRenderer")
	nature.call("refresh_snapshot")
	await process_frame
	var terrain: Dictionary = world.call("get_terrain_snapshot")
	var codes: PackedByteArray = terrain["terrain"]
	var width: int = int(terrain["width"])
	var cleared: Dictionary = LAYOUT.clearings(world.call("get_building_snapshot"),world.call("get_construction_snapshot"),world.call("get_resource_snapshot"),world.call("get_settlement_snapshot"))
	nature.call("_update_canopies")
	var expected_tint: Color = SEASON.resolve(world.call("get_time_snapshot"))
	_check(SEASON.resolve({}) == Color.WHITE,"absent season keys remain white")
	var rock_counts: Dictionary = {2: 0, 3: 0}
	var rock_tiles: Dictionary = {}
	var trees: int = 0
	var placements: Dictionary = {}
	var sprites: Dictionary = nature.get("_sprites")
	var occupied: Dictionary = {}
	for key: String in sprites:
		var sprite: Sprite2D = sprites[key] as Sprite2D
		var tile: Vector2i = Vector2i(int(sprite.get_meta("tile_x")),int(sprite.get_meta("tile_y")))
		var kind: String = str(sprite.get_meta("art_kind"))
		_check(is_equal_approx(sprite.modulate.r,expected_tint.r) and is_equal_approx(sprite.modulate.g,expected_tint.g) and is_equal_approx(sprite.modulate.b,expected_tint.b),"actual nature uses live season tint")
		if kind in ["tree","shrub","log","rock","reed"]:
			_check(not cleared.has(tile),"clearable nature inside radius4 clearing")
		if kind == "rock":
			_check(not rock_tiles.has(tile),"at most one rock per tile")
			rock_tiles[tile] = true
			rock_counts[codes[tile.y*width+tile.x]] += 1
		if kind == "tree" and codes[tile.y*width+tile.x] == 1:
			trees += 1
			_check(not occupied.has(tile),"at most one tree per forest tile")
			occupied[tile] = true
		_check(not ArtSpriteLibrary.is_suppressed(str(sprite.get_meta("sprite_id"))))
		placements[key] = [sprite.position,sprite.get_meta("sprite_id")]
	for code: int in [3,2]:
		var share: float = float(rock_counts[code])/codes.count(code)
		_check(share >= (0.04 if code == 3 else 0.40) and share <= (0.08 if code == 3 else 0.50),"rock share over ALL terrain tiles")
		print("rock density code=",code," ",rock_counts[code],"/",codes.count(code)," = ",share)
	var decal_placements: Dictionary = await _decals(main,world)
	_check(decal_placements == await _decals(main,world),"decal refresh is deterministic")
	var manifest: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://assets/art/manifest.json"))
	var checked_native: Dictionary = {}
	for entry: Dictionary in manifest["assets"]:
		var sprite_id: String = str(entry.get("sprite_id",""))
		if sprite_id.begins_with("N3_") and not ArtSpriteLibrary.frame(sprite_id).is_empty():
			var size: Array = entry["native_size"]
			var limit: int = 6 if sprite_id in ["N3_14","N3_15"] else 8
			_check(int(size[0]) <= limit and int(size[1]) <= limit,"native decal size limit: " + sprite_id)
			var native_path: String = str(entry.get("native_path",""))
			_check(not native_path.is_empty(),"native decal file provenance: " + sprite_id)
			if not native_path.is_empty():
				var image: Image = Image.load_from_file("res://" + native_path)
				_check(image != null,"native decal PNG loads: " + sprite_id)
				if image != null:
					_check(image.get_width() <= limit and image.get_height() <= limit,"actual native decal PNG size: " + sprite_id)
					_check(image.get_width() == int(size[0]) and image.get_height() == int(size[1]),"native metadata matches PNG: " + sprite_id)
			checked_native[sprite_id] = true
	for sprite_id: String in ArtSpriteLibrary.entries():
		if sprite_id.begins_with("N3_"):
			_check(checked_native.has(sprite_id),"installed decal has native-size provenance")
	var coverage: float = float(trees)/codes.count(1)
	_check(coverage >= 0.25 and coverage <= 0.35,"density denominator is all forest tiles")
	nature.call("refresh_snapshot")
	_check(sprites.size() == placements.size())
	for key: String in sprites:
		_check(placements[key] == [(sprites[key] as Sprite2D).position,(sprites[key] as Sprite2D).get_meta("sprite_id")])
	print("forest density ",trees,"/",codes.count(1)," = ",coverage)
	main.free()
	var reloaded: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var reloaded_world: Node = reloaded.get_node("WorldSim")
	reloaded_world.call("set_sim_speed",0.0)
	reloaded_world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(reloaded)
	tick = 0
	while tick < 1680:
		tick = int(reloaded_world.call("advance_ticks",mini(8,1680-tick)))
	var reloaded_nature: Node = reloaded.get_node("NatureRenderer")
	reloaded_nature.call("refresh_snapshot")
	var reloaded_sprites: Dictionary = reloaded_nature.get("_sprites")
	_check(reloaded_sprites.size() == placements.size())
	for key: String in reloaded_sprites:
		_check(placements[key] == [(reloaded_sprites[key] as Sprite2D).position,(reloaded_sprites[key] as Sprite2D).get_meta("sprite_id")],"new world instance preserves deterministic nature")
	_check(decal_placements == await _decals(reloaded,reloaded_world),"new world preserves deterministic decals")
	reloaded.free()
	print("test_nature_density: ","PASS" if _failures == 0 else "FAIL", " failures=",_failures)
	quit(0 if _failures == 0 else 1)

func _decals(main: Node, world: Node) -> Dictionary:
	var renderer: Node = main.get_node("TerrainRenderer")
	renderer.call("refresh_snapshot")
	await process_frame
	var snapshot: Dictionary = world.call("get_terrain_snapshot")
	var codes: PackedByteArray = snapshot["terrain"]
	var width: int = int(snapshot["width"])
	var cleared: Dictionary = LAYOUT.clearings(world.call("get_building_snapshot"),world.call("get_construction_snapshot"),world.call("get_resource_snapshot"),world.call("get_settlement_snapshot"),3)
	var counts: Dictionary = {0: 0,1: 0,4: 0}
	var placements: Dictionary = {}
	for sprite: Sprite2D in (renderer.get("_decals") as Node2D).get_children():
		var tile: Vector2i = Vector2i(int(sprite.get_meta("tile_x")),int(sprite.get_meta("tile_y")))
		_check(not cleared.has(tile),"decal outside radius3 camp clearing")
		_check(not placements.has(tile),"one decal per tile")
		counts[codes[tile.y*width+tile.x]] += 1
		placements[tile] = [sprite.position,sprite.get_meta("sprite_id")]
	for code: int in [0,1]:
		var share: float = float(counts[code])/codes.count(code)
		_check(share <= (0.04 if code == 0 else 0.03),"decal share over ALL terrain tiles")
		print("decal density code=",code," ",counts[code],"/",codes.count(code)," = ",share)
	return placements
