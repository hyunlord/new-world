extends SceneTree

const Cold = preload("res://scripts/ui/shelter_cold.gd")
const ART = preload("res://scripts/ui/art_sprite_library.gd")

class FakeWorld extends Node:
	var agents: Dictionary = {}
	var buildings: Dictionary = {}
	func get_agent_snapshot() -> Dictionary:
		return agents
	func get_building_snapshot() -> Dictionary:
		return buildings

var checks: int = 0

func _init() -> void:
	call_deferred("_run")

func _run() -> void:
	var source: String = FileAccess.get_file_as_string("res://scripts/ui/building_renderer.gd")
	var script := GDScript.new()
	script.source_code = source.replace("var _world_sim: WorldSimNode", "var _world_sim: Node")
	assert(script.reload() == OK)
	var renderer := Node2D.new()
	renderer.set_script(script)
	var world := FakeWorld.new()
	world.agents = {"xs": PackedInt32Array([1]), "ys": PackedInt32Array([4]), "states": PackedByteArray([3]), "target_kinds": PackedByteArray([3]), "colds": PackedByteArray([1])}
	world.buildings = {"xs": PackedInt32Array([1]), "ys": PackedInt32Array([4]), "kinds": PackedByteArray([1])}
	renderer.set("_world_sim", world)
	var sprite := Sprite2D.new()
	ART.apply(sprite, "S1_11")
	sprite.set_meta("building_kind", 1)
	sprite.set_meta("tile_x", 1)
	sprite.set_meta("tile_y", 4)
	sprite.set_meta("art_id", 0)
	renderer.set("_buildings", {1: sprite})
	_observe(renderer, world, sprite, 0.2, 0.2)
	assert(is_equal_approx(float(sprite.get_meta("frost_alpha")), 0.5))
	var cached: Dictionary = renderer.get("_sleeper_counts")
	_observe(renderer, world, sprite, 0.4, 0.2)
	assert(is_same(cached, renderer.get("_sleeper_counts")))
	assert(is_equal_approx(float(sprite.get_meta("frost_alpha")), 1.0))
	var breath: Sprite2D = sprite.get_node("DoorwayColdBreath")
	var before: Vector2 = breath.position
	_observe(renderer, world, sprite, 0.6, 0.0)
	assert(breath.position != before)
	for snapshot: Dictionary in [world.agents, world.buildings]:
		for key: String in snapshot.keys():
			var original: Variant = snapshot[key].duplicate()
			snapshot[key][0] = int(snapshot[key][0]) + 1
			_observe(renderer, world, sprite, 0.7, 0.1)
			snapshot[key] = original.duplicate()
			_observe(renderer, world, sprite, 0.8, 0.1)
			snapshot.erase(key)
			_observe(renderer, world, sprite, 0.9, 0.1)
			snapshot[key] = null
			_observe(renderer, world, sprite, 1.0, 0.1)
			snapshot[key] = [1]
			_observe(renderer, world, sprite, 1.1, 0.1)
			snapshot[key] = original
			_observe(renderer, world, sprite, 1.2, 0.1)
	world.buildings = {"xs": PackedInt32Array(), "ys": PackedInt32Array(), "kinds": PackedByteArray()}
	_observe(renderer, world, sprite, 1.3, 0.4)
	world.buildings = {"xs": PackedInt32Array([1,2]), "ys": PackedInt32Array([4,4]), "kinds": PackedByteArray([1,1])}
	_observe(renderer, world, sprite, 1.4, 0.4)
	renderer.call("_reset_cold")
	assert(float(sprite.get_meta("frost_alpha")) == 0.0)
	_observe(renderer, world, sprite, 1.5, 0.4)
	var replacement := FakeWorld.new()
	replacement.agents = world.agents.duplicate(true)
	replacement.buildings = world.buildings.duplicate(true)
	renderer.set("_world_sim", replacement)
	_observe(renderer, replacement, sprite, 1.6, 0.0)
	replacement.agents["colds"][0] = 0
	_observe(renderer, replacement, sprite, 1.7, 0.4)
	assert(float(sprite.get_meta("frost_alpha")) == 0.0)
	sprite.free()
	renderer.free()
	world.free()
	replacement.free()
	print("BUILDING_DERIVED_CACHE_P9 PASS checks=", checks)
	quit()

func _observe(renderer: Node, world: FakeWorld, sprite: Sprite2D, elapsed: float, delta: float) -> void:
	var expected: Dictionary = Cold.sleeper_counts(world.agents, world.buildings)
	renderer.set("_elapsed", elapsed)
	renderer.call("_update_shelter_cold", delta)
	assert(int(sprite.get_meta("cold_sleepers")) == int(expected.get(Vector2i(1,4), 0)))
	checks += 1
