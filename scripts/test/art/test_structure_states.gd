extends SceneTree
const Structures = preload("res://scripts/ui/building_renderer.gd")
const Wood = preload("res://scripts/ui/wood_visuals.gd")
class ResourceDouble extends Node:
	var snapshot: Dictionary = {}
	var tick: int = 1
	var buildings: Dictionary = {}
	var settlements: Dictionary = {}
	func get_time_snapshot() -> Dictionary:
		return {"tick": tick, "time_of_day": 12.0}
	func get_building_snapshot() -> Dictionary:
		return buildings
	func get_construction_snapshot() -> Dictionary:
		return {}
	func get_settlement_snapshot() -> Dictionary:
		return settlements
	func get_recent_deaths() -> Dictionary:
		return {"current_tick": tick}
	func get_world_info() -> Dictionary:
		return {"ticks_per_day": 1440}
	func get_resource_snapshot() -> Dictionary:
		return snapshot

func _initialize() -> void:
	assert(Structures.construction_stage(0,100) == 0)
	assert(Structures.construction_stage(25,100) == 1)
	assert(Structures.construction_stage(50,100) == 2)
	assert(Structures.construction_stage(75,100) == 3)
	assert(Structures.construction_stage(200,100) == 3)
	assert(Structures.hearth_frame(0,0) == "S1_06")
	assert(Structures.hearth_frame(4.99,0.25) == "S1_07")
	assert(Structures.hearth_frame(5,0) == "S1_02")
	assert(Structures.hearth_frame(23.99,0.375) == "S1_05")
	assert(Structures.storage_frame(0) == "S1_08")
	assert(Structures.storage_frame(39) == "S1_09")
	assert(Structures.storage_frame(40) == "S1_10")
	assert(Structures.cairn_alpha(0,100) == 1.0)
	assert(Structures.cairn_alpha(75,100) == 1.0)
	assert(is_equal_approx(Structures.cairn_alpha(87,100),0.52))
	assert(Structures.cairn_alpha(100,100) == 0.0)
	var snapshot: Dictionary = {"settlement_ids": PackedInt32Array([9, 2]), "ids": PackedInt64Array([1, 8]), "formation_xs": PackedInt32Array([0, 2]), "formation_ys": PackedInt32Array([0, 0]), "wood_stocks": PackedInt32Array([28, 287]), "fire_lit": PackedByteArray([0, 1])}
	assert(Wood.settlement(snapshot, Vector2i(1, 0))["settlement_id"] == 2)
	assert(Wood.settlement(snapshot, Vector2i.ZERO)["wood_stock"] == 28)
	assert(Wood.settlement({}, Vector2i.ZERO).is_empty())
	snapshot["wood_stocks"] = PackedInt32Array([28])
	snapshot["fire_lit"] = PackedByteArray([0])
	var partial: Dictionary = Wood.settlement(snapshot, Vector2i(2, 0))
	assert(not partial.has("wood_stock") and not partial.has("fire_lit"))
	snapshot["wood_stocks"] = [1, 2]
	snapshot["fire_lit"] = [0, 1]
	assert(not Wood.settlement(snapshot, Vector2i.ZERO).has("wood_stock"))
	assert(not Wood.settlement(snapshot, Vector2i.ZERO).has("fire_lit"))
	for hour: float in [0.0, 12.0, 23.0]:
		assert(Wood.hearth_frame({}, hour, 0.0) == Structures.hearth_frame(hour, 0.0))
		assert(Wood.hearth_frame({"fire_lit": false, "wood_stock": 0}, hour, 0.0) == "S1_16")
		assert(Wood.hearth_frame({"fire_lit": false, "wood_stock": 28}, hour, 0.0) == "S1_01")
		for i: int in range(4):
			assert(Wood.hearth_frame({"fire_lit": true}, hour, float(i)/8.0) == "S1_%02d" % (2+i))
	assert(Wood.glow_alpha({"fire_lit": false}, 1.0, 0.0) == 0.0)
	assert(Wood.glow_alpha({"fire_lit": true}, 0.0, 0.0) == 0.0)
	assert(is_equal_approx(Wood.glow_alpha({"fire_lit": true}, 1.0, 0.0), 0.55))
	assert(is_equal_approx(Wood.glow_alpha({}, 1.0, 0.0), 0.55))
	for pair: Array in [[0,0], [1,1], [28,1], [39,1], [40,2], [149,2], [150,3], [287,3], [311,3], [158,3]]:
		assert(Wood.pile_stage(pair[0]) == pair[1])
	for pair: Array in [[0,0], [1,1], [10,1], [11,2], [20,2], [21,3], [30,3], [100,3]]:
		assert(Wood.deadwood_stage(pair[0],30) == pair[1])
	assert(Wood.pile_frame(0) == "")
	var art = preload("res://scripts/ui/art_sprite_library.gd")
	art._ensure_loaded()
	var saved: Dictionary = art._entries.duplicate(true)
	for stage: int in range(4):
		var id: String = "W1_%02d" % (7-2*stage)
		var alternate: String = "W1_%02d" % (8-2*stage)
		var fallback: String = "N1_15" if stage >= 2 else "S2_15"
		art._entries[id] = {"qa_status": "fallback", "drawable": true}
		art._entries[alternate] = {"qa_status": "fallback", "drawable": true}
		assert(Wood.deadwood_frame(stage) == fallback)
		art._entries[alternate] = {"qa_status": "accepted", "drawable": true}
		assert(Wood.deadwood_frame(stage) == alternate)
		art._entries[id] = {"qa_status": "accepted", "drawable": true}
		assert(Wood.deadwood_frame(stage) == id)
		art._entries[id]["drawable"] = false
		assert(Wood.deadwood_frame(stage) == alternate)
		art._entries[alternate]["drawable"] = false
		assert(Wood.deadwood_frame(stage) == fallback)
	for stage: int in range(1,4):
		var id: String = "W1_%02d" % (8+stage)
		var alternate: String = "W1_%02d" % (12+stage)
		var fallback: String = "S2_15" if stage == 1 else "S1_14"
		art._entries[id] = {"qa_status": "fallback", "drawable": true}
		art._entries[alternate] = {"qa_status": "fallback", "drawable": true}
		assert(Wood.pile_frame(stage) == fallback)
		art._entries[alternate] = {"qa_status": "accepted", "drawable": true}
		assert(Wood.pile_frame(stage) == alternate)
		art._entries[id] = {"qa_status": "accepted", "drawable": true}
		assert(Wood.pile_frame(stage) == id)
		art._entries[id]["drawable"] = false
		assert(Wood.pile_frame(stage) == alternate)
		art._entries[alternate]["drawable"] = false
		assert(Wood.pile_frame(stage) == fallback)
	art._entries = saved
	_test_deadwood_rows()
	print("ART STRUCTURE STATES PASS: construction, hearth clock, storage tiers, day-long cairn fade")
	quit(0)

func _test_deadwood_rows() -> void:
	var source: String = FileAccess.get_file_as_string("res://scripts/ui/building_renderer.gd")
	var script := GDScript.new()
	script.source_code = source.replace("var _world_sim: WorldSimNode", "var _world_sim: Node")
	assert(script.reload() == OK)
	var renderer := Node2D.new()
	renderer.set_script(script)
	var world := ResourceDouble.new()
	world.snapshot = {"xs": PackedInt32Array([1,2,3,4]), "ys": PackedInt32Array([1,1,1,1]), "kinds": PackedInt32Array([3,3,3,3]), "amounts": PackedInt32Array([0,10,20,30]), "maxes": PackedInt32Array([30,30,30,30])}
	var objects := Node2D.new()
	var shadows := Node2D.new()
	renderer.set("_world_sim", world)
	renderer.set("_objects", objects)
	renderer.set("_shadows", shadows)
	renderer.call("_update_deadwood")
	var sprites: Dictionary = renderer.get("_deadwood")
	assert(sprites.size() == 4)
	for sprite: Sprite2D in sprites.values():
		var stage: int = int(sprite.get_meta("tile_x"))-1
		assert(int(sprite.get_meta("deadwood_stage")) == stage)
		assert(str(sprite.get_meta("sprite_id")) == Wood.deadwood_frame(stage))
		assert(is_equal_approx(sprite.modulate.a, 0.35 if stage == 0 and Wood.deadwood_frame(stage) == "S2_15" else 1.0))
	world.snapshot["maxes"] = PackedInt32Array([30,30])
	renderer.call("_update_deadwood")
	assert(sprites.size() == 2)
	world.snapshot.erase("maxes")
	renderer.call("_update_deadwood")
	assert(sprites.is_empty())
	world.buildings = {"ids": PackedInt64Array([0,28]), "xs": PackedInt32Array([23,25]), "ys": PackedInt32Array([18,20]), "kinds": PackedByteArray([0,2])}
	world.settlements = {"settlement_ids": PackedInt32Array([1]), "formation_xs": PackedInt32Array([23]), "formation_ys": PackedInt32Array([18]), "wood_stocks": PackedInt32Array([30]), "food_stocks": PackedInt32Array([65]), "fire_lit": PackedByteArray([0])}
	world.snapshot = {"xs": PackedInt32Array([37]), "ys": PackedInt32Array([9]), "kinds": PackedInt32Array([3]), "amounts": PackedInt32Array([30]), "maxes": PackedInt32Array([30])}
	renderer.call("_process", 1.0/60.0)
	var props: Dictionary = renderer.get("_props")
	assert(props.has(1), "nonzero stock gets a pile even when the west footprint touches the hearth")
	assert(int((props[1] as Sprite2D).get_meta("woodpile_stage")) == 1)
	var labels: Dictionary = renderer.get("_labels")
	assert(int((labels.values()[0] as Label).get_meta("wood_stock")) == 30)
	assert(int((sprites.values()[0] as Sprite2D).get_meta("deadwood_stage")) == 3)
	world.tick = 2
	world.settlements["wood_stocks"] = PackedInt32Array([40])
	world.snapshot["amounts"] = PackedInt32Array([0])
	renderer.call("_process", 1.0/60.0)
	assert(int(renderer.get("_frame")) == 2, "resource changes update before the old 30-frame reconciliation")
	assert(int((labels.values()[0] as Label).get_meta("wood_stock")) == 40)
	assert(int((props[1] as Sprite2D).get_meta("woodpile_stage")) == 2)
	assert(int((sprites.values()[0] as Sprite2D).get_meta("deadwood_stage")) == 0)
	var pile: Sprite2D = props[1]
	var pile_tile := Vector2i(int(pile.get_meta("tile_x")), int(pile.get_meta("tile_y")))
	var frame: Dictionary = preload("res://scripts/ui/art_sprite_library.gd").frame(str(pile.get_meta("sprite_id")))
	assert(Structures.prop_footprint_allowed(pile_tile, Vector2.ZERO, int(frame["rect"][2]), int(frame["anchor"][0]), {Vector2i(23,18): true, Vector2i(25,20): true}))
	world.tick = 3
	world.settlements["wood_stocks"] = PackedInt32Array([0])
	renderer.call("_process", 1.0/60.0)
	assert(not props.has(1), "empty stock removes the pile on its simulation tick")
	objects.free()
	shadows.free()
	renderer.free()
	world.free()
