# B5 windowed evidence from ordinary Rust gameplay exports, never manufactured material snapshots.
# Bridge methods: load_world, cmd_designate_chop, advance_ticks, get_trees, get_space, get_time_snapshot.
# Run after root rebuild with isolated settings: godot --path . --script tools/v8/b5_tree_visual.gd -- /fixture/dir /evidence/dir
# Load/reset clears visual stumps; the stump capture therefore observes actual chop continuation before loading completed.save.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const Flora = preload("res://scripts/world/art/flora_resolver.gd")
const Trees = preload("res://scripts/world/colony/tree_layer.gd")
const Materials = preload("res://scripts/world/art/material_resolver.gd")
var _failures: int = 0
var _world: Node
var _trees: Node
var _ground: Node
var _camera: Camera2D
var _out: String
var _receipt: Dictionary
var _evidence: Dictionary = {"checks": [], "captures": []}

func _init() -> void:
	call_deferred("_run")

func _check(label: String, passed: bool) -> void:
	print("B5_VISUAL_%s %s" % ["PASS" if passed else "FAIL", label])
	(_evidence["checks"] as Array).append({"label": label, "passed": passed})
	if not passed:
		_failures += 1

func _frames(count: int = 4) -> void:
	for _i: int in count:
		await process_frame

func _tile(value: Array) -> Vector2i:
	return Vector2i(int(value[0]), int(value[1]))

func _identity(tile: Vector2i) -> String:
	var snapshot: Dictionary = SimReads.read(_world, "get_trees")
	var xs: PackedInt32Array = snapshot["xs"]
	var ys: PackedInt32Array = snapshot["ys"]
	var species: PackedStringArray = snapshot["species"]
	for i: int in xs.size():
		if xs[i] == tile.x and ys[i] == tile.y:
			return species[i]
	return ""

func _wall(tile: Vector2i) -> Dictionary:
	var space: Dictionary = SimReads.read(_world, "get_space")
	var xs: PackedInt32Array = space["wall_xs"]
	var ys: PackedInt32Array = space["wall_ys"]
	var materials: PackedInt32Array = space["wall_materials"]
	var styles: PackedStringArray = space["wall_styles"]
	var categories: PackedStringArray = space["wall_categories"]
	for i: int in xs.size():
		if xs[i] == tile.x and ys[i] == tile.y:
			return {"material": int(materials[i]) & 0xffffffff, "style": styles[i], "category": categories[i]}
	return {}

func _focus(tile: Vector2i, zoom: float) -> void:
	# The fixture owns focus even when a newly built hearth enables deferred camp centering.
	_camera.set("_center_pending", false)
	_camera.zoom = Vector2(zoom, zoom)
	_camera.call("center_on_tile", tile)
	_camera.force_update_scroll()
	await _frames(40)
	_check("focus_zoom_settled", _camera.zoom.is_equal_approx(Vector2(zoom, zoom)))
	_check("focus_pending_cleared", not bool(_camera.get("_center_pending")))

func _frame_facts(tile: Vector2i) -> Dictionary:
	var projected: Vector2 = root.get_canvas_transform() * View.tile_centre(tile)
	var sprite: Sprite2D = (_trees.get("_nature_sprites") as Dictionary).get(tile)
	var resources: Dictionary = _trees.get("_resources")
	var rows: Array = []
	var xs: PackedInt32Array = resources.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = resources.get("ys", PackedInt32Array())
	for i: int in mini(xs.size(), ys.size()):
		if Vector2i(xs[i], ys[i]) == tile:
			rows.append({"kind": resources["kinds"][i], "amount": resources["amounts"][i]})
	return {"camera_position": str(_camera.position), "screen_center": str(_camera.get_screen_center_position()), "zoom": str(_camera.zoom), "center_pending": _camera.get("_center_pending"), "projected_target": str(projected), "viewport": str(root.get_visible_rect()), "live_chunk": (_trees.get("_live") as Dictionary).has(_trees.call("_chunk_of", tile)), "nature_present": sprite != null, "nature_texture": sprite.texture.resource_path if sprite != null and sprite.texture != null else "", "nature_id": str(sprite.get_meta("id", "")) if sprite != null else "", "nature_visible": sprite.is_visible_in_tree() if sprite != null else false, "resources": rows}

func _capture(name: String, facts: Dictionary) -> void:
	await _frames()
	await RenderingServer.frame_post_draw
	var tile: Vector2i = _tile(facts["tile"])
	facts["frame_facts"] = _frame_facts(tile)
	var projected: Vector2 = root.get_canvas_transform() * View.tile_centre(tile)
	# Central horizontal strip avoids top HUD, lower-left lesson and right-side panels.
	var unobscured: Rect2 = Rect2(32.0, 280.0, root.get_visible_rect().size.x - 464.0, 320.0)
	facts["unobscured_capture_region"] = str(unobscured)
	_check("capture_target_visible_" + name, unobscured.has_point(projected))
	var path: String = _out.path_join(name + ".png")
	_check("capture_" + name, root.get_texture().get_image().save_png(path) == OK)
	(_evidence["captures"] as Array).append({"name": name, "facts": facts, "tick": SimReads.read(_world, "get_time_snapshot").get("tick", -1), "image_sha256": FileAccess.get_sha256(path)})

func _finish() -> void:
	_evidence["failures"] = _failures
	var file: FileAccess = FileAccess.open(_out.path_join("receipt.json"), FileAccess.WRITE)
	if file == null:
		_failures += 1
	else:
		file.store_string(JSON.stringify(_evidence, "\t"))
	print("B5_VISUAL_DONE failures=%d" % _failures)
	quit(1 if _failures else 0)

func _run() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	if args.size() != 2 or not args[0].is_absolute_path() or not args[1].is_absolute_path():
		push_error("B5 visual requires absolute fixture and evidence directories")
		quit(1)
		return
	_out = args[1]
	DirAccess.make_dir_recursive_absolute(_out)
	_check("windowed_backend", DisplayServer.get_name() != "headless")
	_check("isolated_settings", OS.get_environment("WORLDSIM_TEST_SETTINGS_DIR").is_absolute_path())
	_check("external_evidence", not _out.begins_with(ProjectSettings.globalize_path("res://")))
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(args[0].path_join("receipt.json")))
	_check("fixture_receipt", parsed is Dictionary)
	if _failures:
		_finish()
		return
	_receipt = parsed
	for field: String in ["tree", "species", "wall", "wall_material", "oak_wall", "oak_wall_material", "chop_already_designated"]:
		_check("receipt_" + field, _receipt.has(field))
	if _failures:
		_finish()
		return
	_evidence["fixture"] = _receipt
	_check("original_worldgen_coordinates", str(_receipt.get("population_provenance", "")) == "original_worldgen_coordinates")
	_check("ordinary_gameplay_commands", bool(_receipt.get("ordinary_commands", false)))
	if _failures:
		_finish()
		return
	var before: String = args[0].path_join("before_chop.save")
	var completed: String = args[0].path_join("completed.save")
	_evidence["before_sha256"] = FileAccess.get_sha256(before)
	_evidence["completed_sha256"] = FileAccess.get_sha256(completed)
	root.size = Vector2i(1600, 1000)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	_world = main.get_node("WorldSim")
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	_trees = main.get_node("ColonyView/TreeLayer")
	_ground = main.get_node("ColonyView/GroundLayer")
	_camera = main.get_node("Camera2D") as Camera2D
	_check("load_before_chop", bool(SimReads.mutate(_world, "load_world", [before])))
	await _frames(20)
	if _failures:
		_finish()
		return
	var tile: Vector2i = _tile(_receipt["tree"])
	var identity: String = str(_receipt["species"])
	_check("actual_standing_species", identity == "pine" and _identity(tile) == identity)
	await _focus(tile, 1.0)
	var sprites: Dictionary = _trees.get("_tree_sprites")
	var sprite: Sprite2D = sprites.get(tile)
	var flora: Flora = _trees.call("flora")
	var pair: PackedStringArray = _trees.call("season_pair")
	var seed: int = int(SimReads.read(_world, "get_world_info").get("seed", 0))
	var frame: Dictionary = flora.resolve(identity, Trees.tree_size(seed, tile), pair[1], Trees.tree_variant(seed, tile))
	_check("standing_draws_actual_species", sprite != null and sprite.texture.resource_path == Flora.res_path(str(frame["path"])))
	await _capture("standing-pine", {"tile": [tile.x, tile.y], "species": _identity(tile), "frame": frame.get("path", "")})
	await _focus(tile, 0.25)
	var styles: Dictionary = _ground.call("_species_styles")
	_check("canopy_saved_species", int((_ground.get("_tree_cells") as Dictionary).get(tile, 0)) == int(styles.get(identity, -1)) and bool(_ground.call("canopy_baked_at", tile)))
	await _capture("quarter-canopy", {"tile": [tile.x, tile.y], "species": identity, "style": styles.get(identity, -1)})
	await _focus(tile, 1.0)
	if not bool(_receipt["chop_already_designated"]):
		var command: Dictionary = SimReads.mutate(_world, "cmd_designate_chop", [tile.x, tile.y, tile.x, tile.y])
		_check("ordinary_chop_accepted", bool(command.get("accepted", false)))
	var elapsed: int = 0
	while _identity(tile) != "" and elapsed < 1440:
		SimReads.mutate(_world, "advance_ticks", [12])
		elapsed += 12
		await _frames(2)
	_check("ordinary_chop_removes_tree", _identity(tile) == "")
	await _frames(4)
	var stumps: Dictionary = _trees.get("_stumps")
	_check("observed_stump_remembers_species", stumps.has(tile) and str((stumps[tile] as Dictionary)["species"]) == identity)
	var stump: Sprite2D = (_trees.get("_nature_sprites") as Dictionary).get(tile)
	_check("observed_stump_draws_species", stump != null and stump.is_visible_in_tree() and stump.texture.resource_path == Flora.res_path(str(flora.stump(identity)["path"])))
	await _capture("observed-pine-stump", {"tile": [tile.x, tile.y], "species": identity, "elapsed_ticks": elapsed, "provenance": "ordinary chop after before_chop load; no reset between standing and removal"})
	var stockpile: Dictionary = SimReads.mutate(_world, "cmd_add_stockpile", [tile.x + 6, tile.y - 2, tile.x + 12, tile.y + 6])
	_check("ordinary_clearance_stockpile", bool(stockpile.get("accepted", false)))
	var clearance: Dictionary = SimReads.mutate(_world, "cmd_designate_chop", [maxi(0, tile.x - 2), maxi(0, tile.y - 1), tile.x + 3, tile.y + 5])
	_check("ordinary_occluder_chop_accepted", bool(clearance.get("accepted", false)))
	var clearance_ticks: int = 0
	while clearance_ticks < 900:
		SimReads.mutate(_world, "advance_ticks", [12])
		clearance_ticks += 12
		await _frames(2)
	await _focus(tile, 1.0)
	var exposed: Sprite2D = (_trees.get("_nature_sprites") as Dictionary).get(tile)
	_check("original_stump_survives_clearance", exposed != null and exposed.texture.resource_path == Flora.res_path(str(flora.stump(identity)["path"])))
	await _capture("ordinary-cleared-pine-stump", {"tile": [tile.x, tile.y], "species": identity, "clearance_ticks": clearance_ticks, "provenance": "additional ordinary chop rectangle and east stockpile for hauling around original stump; no sprite hiding or inserted state"})
	_check("load_completed_gameplay", bool(SimReads.mutate(_world, "load_world", [completed])))
	await _frames(20)
	_check("load_reset_clears_visual_stumps", (_trees.get("_stumps") as Dictionary).is_empty())
	var resolver: Materials = Materials.new()
	resolver.load_imported()
	for key: String in ["wall", "oak_wall"]:
		var wall_tile: Vector2i = _tile((_receipt["pine_walls"] as Array)[1]) if key == "wall" else _tile(_receipt[key])
		var look: Dictionary = _wall(wall_tile)
		_check(key + "_actual_material", not look.is_empty() and int(look.get("material", -1)) == int(_receipt[key + "_material"]))
		if look.is_empty():
			continue
		var resolved: Dictionary = resolver.resolve(str(look["style"]), int(look["material"]), str(look["category"]), "wall")
		_check(key + "_exact_material_tint", str(resolved.get("material_key", "")) == ("pine" if key == "wall" else "oak") and str(resolved.get("tint_source", "")) == "material")
		await _focus(wall_tile, 1.0)
		await _capture(key + "-material", {"tile": [wall_tile.x, wall_tile.y], "saved_look": look, "resolved_look": resolved, "provenance": "completed ordinary gameplay save; loading cleared transient stumps"})
	_finish()
