extends SceneTree
const Effects = preload("res://scripts/rendering/art/god_hand_effects.gd")
var _failures: int = 0

class SnapshotWorld extends Node:
	var state: Dictionary = {"log_ids": PackedInt64Array(), "log_settlement_ids": PackedInt32Array(), "log_kinds": PackedStringArray(), "log_ticks": PackedInt64Array(), "log_amounts": PackedInt64Array(), "log_applied": PackedInt64Array()}
	var settlements: Dictionary = {"settlement_ids": PackedInt32Array([7, 42]), "formation_xs": PackedInt32Array([0, 20]), "formation_ys": PackedInt32Array([0, 0])}
	func get_intervention_state() -> Dictionary: return state
	func get_settlement_snapshot() -> Dictionary: return settlements
	func get_resource_snapshot() -> Dictionary: return {}
	func cast(kind: String, target: int) -> void:
		state["log_ids"].append(state["log_ids"].size())
		state["log_settlement_ids"].append(target)
		state["log_kinds"].append(kind)
		state["log_ticks"].append(100)
		state["log_amounts"].append(40)
		state["log_applied"].append(40)

func _initialize() -> void: call_deferred("_run")
func _check(value: bool, message: String) -> void:
	if not value:
		_failures += 1
		push_error(message)

func _sprite(objects: Node2D, id: int, tile: Vector2i, kind: String, building_kind: int = -1) -> Sprite2D:
	var sprite := Sprite2D.new()
	sprite.position = Vector2(tile) * 16.0
	sprite.set_meta("art_kind", kind)
	sprite.set_meta("art_id", id)
	sprite.set_meta("tile_x", tile.x)
	sprite.set_meta("tile_y", tile.y)
	sprite.set_meta("building_kind", building_kind)
	objects.add_child(sprite)
	return sprite

func _run() -> void:
	var world := SnapshotWorld.new()
	var objects := Node2D.new()
	root.add_child(objects)
	_sprite(objects, 1, Vector2i.ZERO, "building", 0)
	_sprite(objects, 2, Vector2i(20, 0), "building", 0)
	var food_left := _sprite(objects, 10, Vector2i(2, 0), "building", 2)
	var food_right := _sprite(objects, 20, Vector2i(18, 0), "building", 2)
	var wood_left := _sprite(objects, 101, Vector2i(0, 1), "camp_prop")
	wood_left.set_meta("woodpile_stage", 1)
	var wood_right := _sprite(objects, 201, Vector2i(20, 1), "camp_prop")
	wood_right.set_meta("woodpile_stage", 2)
	var wood_base := Color(0.8, 0.9, 0.7)
	wood_right.self_modulate = wood_base
	var effects := Effects.new()
	root.add_child(effects)
	effects.set_process(false)
	effects._objects = objects
	effects.configure(world)
	world.cast("spoil_food", 7)
	world.cast("spoil_food", 42)
	world.cast("rot_wood", 42)
	effects._process(0.0)
	var rows: Array = effects.metadata()["world_effects"]
	_check(rows.size() == 3, "distinct source IDs preserve identical same-tick casts")
	_check(rows[0]["pile_art_id"] == 10 and rows[1]["pile_art_id"] == 20 and rows[2]["pile_art_id"] == 201, "actual source target selects food and own hearth wood")
	_check(rows[0]["particles"].size() == 6 and rows[0]["particles"][0]["color"] == "6b4e2a", "spoil produces six brown native particles")
	_check(rows[2]["particles"].size() == 4 and rows[2]["particles"][0]["color"] == "a3a3a3", "rot produces four grey particles")
	_check(wood_right.self_modulate == wood_base * Effects.WOOD_ROT_COLOR and wood_left.self_modulate == Color.WHITE, "rot multiplies only actual target wood")
	_check(food_left.self_modulate != Color.WHITE and food_right.self_modulate != Color.WHITE, "both actual food piles tinted")
	var initial_y: float = rows[0]["particles"][0]["position"][1]
	effects._process(0.6)
	rows = effects.metadata()["world_effects"]
	_check(rows.size() == 3 and float(rows[0]["particles"][0]["position"][1]) < initial_y, "puffs rise without duplicate replay")
	effects._process(0.61)
	_check(effects.metadata()["world_effects"].is_empty(), "all pile effects end after 1.2 seconds")
	_check(wood_right.self_modulate == wood_base and food_left.self_modulate == Color.WHITE, "expired tints restore sprites")
	world.cast("gift_food", 7)
	world.cast("gift_wood", 42)
	effects._process(0.0)
	rows = effects.metadata()["world_effects"]
	_check(rows.size() == 2 and rows[0]["particles"].size() == 6 and rows[1]["particles"].size() == 6, "both gifts emit six light particles")
	_check(rows[0]["particles"][0]["color"] == "fff1c7" and rows[0]["particles"][0]["size"] == 1.0, "gift native pixel color and size")
	initial_y = rows[1]["particles"][0]["position"][1]
	effects._process(0.6)
	_check(float(effects.metadata()["world_effects"][1]["particles"][0]["position"][1]) > initial_y, "gift particles fall onto pile")
	effects._process(0.61)
	world.cast("rot_wood", 42)
	effects.notify_cast("rot_wood", 42)
	wood_right.free()
	effects._process(0.0)
	rows = effects.metadata()["world_effects"]
	_check(rows[0]["target_found"] and not rows[0]["sprite_alive"] and rows[0]["particles"].size() == 4, "depleted pile retains cached real location without phantom sprite")
	effects._process(1.21)
	world.cast("gift_wood", 42)
	effects._process(0.0)
	rows = effects.metadata()["world_effects"]
	_check(not rows[0]["target_found"] and rows[0]["particles"].is_empty(), "absent real pile never invents an effect target")
	food_right.free()
	_check(effects._capture_pile("gift_food", 42, world.settlements)["art_id"] == 10, "shared nearest existing food pile remains usable")
	var tied := _sprite(objects, 5, Vector2i(-2, 0), "building", 2)
	_check(effects._capture_pile("spoil_food", 7, world.settlements)["art_id"] == 5, "Chebyshev tie follows building ID order")
	world.cast("spoil_food", 7)
	effects._process(0.0)
	_check(tied.self_modulate != Color.WHITE, "reset fixture has active real tint")
	effects.configure(world)
	_check(tied.self_modulate == Color.WHITE and effects.metadata()["world_effects"].is_empty(), "reset restores tint and removes effects")
	effects.free()
	objects.free()
	world.free()
	print("INTERVENTION_WORLD_EFFECTS failures=", _failures)
	quit(1 if _failures else 0)
