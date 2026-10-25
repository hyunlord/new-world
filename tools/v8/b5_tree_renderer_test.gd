# B5 authoritative tree renderer contracts. Synthetic identity changes are renderer-only controls.
# Bridge methods: reset_world, cmd_set_steward, get_trees, get_world_info; normal scene reads.
# Run after root rebuild: godot --headless --path . --script tools/v8/b5_tree_renderer_test.gd
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const Trees = preload("res://scripts/world/colony/tree_layer.gd")
const Flora = preload("res://scripts/world/art/flora_resolver.gd")
var _failures: int = 0

class TreeReadFixture extends Node:
	var snapshot: Dictionary = {}
	var tick: int = 20
	func get_trees() -> Dictionary:
		return snapshot
	func get_colony_versions() -> Dictionary:
		return {"trees_version": snapshot.get("version", 0)}
	func get_world_info() -> Dictionary:
		return {"seed": 0}
	func get_time_snapshot() -> Dictionary:
		return {"tick": tick}

func _init() -> void:
	call_deferred("_run")

func _check(label: String, passed: bool) -> void:
	print("B5_TREE_%s %s" % ["PASS" if passed else "FAIL", label])
	if not passed:
		_failures += 1

func _frames(count: int) -> void:
	for _i: int in count:
		await process_frame

func _run() -> void:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	world.process_mode = Node.PROCESS_MODE_DISABLED
	SimReads.mutate(world, "reset_world", [0])
	SimReads.mutate(world, "cmd_set_steward", [false, 0])
	await _frames(20)
	var trees: Node = main.get_node("ColonyView/TreeLayer")
	var ground: Node = main.get_node("ColonyView/GroundLayer")
	var snapshot: Dictionary = SimReads.read(world, "get_trees")
	var xs: PackedInt32Array = snapshot["xs"]
	var ys: PackedInt32Array = snapshot["ys"]
	var species: PackedStringArray = snapshot["species"]
	_check("aligned_nonempty_saved_species", xs.size() > 0 and xs.size() == ys.size() and xs.size() == species.size() and not species.has(""))
	var actual: Dictionary = trees.get("_trees")
	var styles: Dictionary = ground.call("_species_styles")
	var canopy: Dictionary = ground.get("_tree_cells")
	var agreement: bool = actual.size() == xs.size() and canopy.size() == xs.size()
	for i: int in xs.size():
		var tile: Vector2i = Vector2i(xs[i], ys[i])
		agreement = agreement and str(actual.get(tile, "")) == species[i] and int(canopy.get(tile, 0)) == int(styles.get(species[i], -1))
	_check("standing_and_canopy_match_bridge", agreement)
	# Exercise the actual cache update code with one tile whose identity changes but coordinate does not.
	trees.process_mode = Node.PROCESS_MODE_DISABLED
	var fake: TreeReadFixture = TreeReadFixture.new()
	root.add_child(fake)
	trees.set("_world_sim", fake)
	trees.set("_ground", null)
	trees.call("_on_world_reset")
	var tile: Vector2i = Vector2i(8, 8)
	trees.set("_live", {Vector2i.ZERO: true})
	trees.call("_set_season", "spring|summer", 1.0)
	trees.set("_resources", {})
	fake.snapshot = {"version": 1, "xs": PackedInt32Array([tile.x]), "ys": PackedInt32Array([tile.y]), "species": PackedStringArray(["oak"])}
	trees.call("_rebuild_trees", false)
	var first: Sprite2D = (trees.get("_tree_sprites") as Dictionary).get(tile)
	_check("first_sprite_created", first != null)
	fake.snapshot["species"] = PackedStringArray(["pine"])
	fake.snapshot["version"] = 2
	trees.call("_rebuild_trees", true)
	var second: Sprite2D = (trees.get("_tree_sprites") as Dictionary).get(tile)
	var flora: Flora = trees.call("flora")
	var expected: Dictionary = flora.resolve("pine", Trees.tree_size(0, tile), "summer", Trees.tree_variant(0, tile))
	_check("same_tile_identity_refreshes_sprite", second != null and second != first and second.texture.resource_path == Flora.res_path(str(expected["path"])))
	fake.snapshot = {"version": 3, "xs": PackedInt32Array(), "ys": PackedInt32Array(), "species": PackedStringArray()}
	trees.call("_rebuild_trees", true)
	trees.call("_sync_nature")
	var stumps: Dictionary = trees.get("_stumps")
	_check("stump_retains_removed_identity", stumps.has(tile) and str((stumps[tile] as Dictionary)["species"]) == "pine")
	var stump: Sprite2D = (trees.get("_nature_sprites") as Dictionary).get(tile)
	_check("stump_uses_saved_species_frame", stump != null and stump.texture.resource_path == Flora.res_path(str(flora.stump("pine")["path"])))
	ground.process_mode = Node.PROCESS_MODE_DISABLED
	ground.set("_world_sim", fake)
	fake.snapshot = {"version": 4, "xs": PackedInt32Array([tile.x]), "ys": PackedInt32Array([tile.y]), "species": PackedStringArray(["fixture_flora"])}
	ground.call("_update_trees")
	var fallback_styles: Dictionary = ground.call("_species_styles")
	_check("unlisted_species_gets_canopy_fallback", fallback_styles.has("fixture_flora") and int((ground.get("_tree_cells") as Dictionary).get(tile, 0)) == int(fallback_styles["fixture_flora"]))
	fake.snapshot["species"] = PackedStringArray(["pine"])
	fake.snapshot["version"] = 5
	ground.call("_update_trees")
	_check("same_tile_canopy_identity_refreshes", int((ground.get("_tree_cells") as Dictionary).get(tile, 0)) == int(fallback_styles["pine"]))
	fake.tick = 100000
	trees.call("_expire_stumps")
	_check("stump_expiry_unchanged", (trees.get("_stumps") as Dictionary).is_empty())
	trees.call("_on_world_reset")
	_check("reset_clears_tree_and_stump_identity", (trees.get("_trees") as Dictionary).is_empty() and (trees.get("_stumps") as Dictionary).is_empty())
	fake.queue_free()
	print("B5_TREE_DONE failures=%d" % _failures)
	quit(1 if _failures else 0)
