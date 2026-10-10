extends SceneTree
const LAYOUT = preload("res://scripts/ui/nature_layout.gd")
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
	assert(building.call("_available","S2_08") == "S2_08","suppressed structures never resurrect direction art")
	var nature: Node = main.get_node("NatureRenderer")
	nature.call("refresh_snapshot")
	await process_frame
	var terrain: Dictionary = world.call("get_terrain_snapshot")
	var codes: PackedByteArray = terrain["terrain"]
	var width: int = int(terrain["width"])
	var cleared: Dictionary = LAYOUT.clearings(world.call("get_building_snapshot"),world.call("get_construction_snapshot"),world.call("get_resource_snapshot"),world.call("get_settlement_snapshot"))
	var trees: int = 0
	var placements: Dictionary = {}
	var sprites: Dictionary = nature.get("_sprites")
	var occupied: Dictionary = {}
	for key: String in sprites:
		var sprite: Sprite2D = sprites[key] as Sprite2D
		var tile: Vector2i = Vector2i(int(sprite.get_meta("tile_x")),int(sprite.get_meta("tile_y")))
		var kind: String = str(sprite.get_meta("art_kind"))
		assert(sprite.modulate.r == 1.0 and sprite.modulate.g == 1.0 and sprite.modulate.b == 1.0,"absent season keys leave actual nature RGB white")
		if kind in ["tree","shrub","log"]:
			assert(not cleared.has(tile),"clearable nature inside radius4 clearing")
		if kind == "tree" and codes[tile.y*width+tile.x] == 1:
			trees += 1
			assert(not occupied.has(tile),"at most one tree per forest tile")
			occupied[tile] = true
		assert(not ArtSpriteLibrary.is_suppressed(str(sprite.get_meta("sprite_id"))))
		placements[key] = [sprite.position,sprite.get_meta("sprite_id")]
	var coverage: float = float(trees)/codes.count(1)
	assert(coverage >= 0.25 and coverage <= 0.35,"density denominator is all forest tiles")
	nature.call("refresh_snapshot")
	assert(sprites.size() == placements.size())
	for key: String in sprites:
		assert(placements[key] == [(sprites[key] as Sprite2D).position,(sprites[key] as Sprite2D).get_meta("sprite_id")])
	print("test_nature_density: PASS ",trees,"/",codes.count(1)," = ",coverage)
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
	assert(reloaded_sprites.size() == placements.size())
	for key: String in reloaded_sprites:
		assert(placements[key] == [(reloaded_sprites[key] as Sprite2D).position,(reloaded_sprites[key] as Sprite2D).get_meta("sprite_id")],"new world instance preserves deterministic nature")
	reloaded.free()
	print("test_nature_density reload: PASS")
	quit()
