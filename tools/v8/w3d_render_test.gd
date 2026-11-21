# w3d_render_test.gd — headless tests of the 3D view's ground, walls, people and trees (gate step 34 `w3d_render`,
#   M3D-SWITCH E2; replaces the 2D mart2_ground_test, mart2_people_test and mart21_trees_test). Meshes are built on the
#   CPU, so nothing here needs a window.
# Purpose: boots scenes/main.tscn on the colony world seed 0 (reset_world(0), the clock held and stepped to tick 1 so
#   the hearth exists), builds every terrain chunk, and checks:
#   GROUND 1. chunk determinism: the hearth's terrain chunk built twice by standalone Terrains fed the same snapshots has
#      byte-identical vertex and colour arrays, their SHA-256 equals tools/v8/mart2_chunk_hashes.json (`-- --write`
#      records it after an intended change), and the live World3D/Terrain chunk equals that fresh build (the view shows
#      the current world: the camp's packed earth around the hearth that appears at tick 1);
#   GROUND 2. a dig rebuilds exactly one chunk: an undug raised (hill) tile whose 3 × 3 neighbourhood lies inside one
#      chunk is dug in the standalone Terrain's get_space (the same path as a real get_space change): only that chunk's
#      mesh is replaced, its bytes change, and the dug tile stands lower than before;
#   GROUND 3. the data-only wall: tools/v8/fixtures/data_only_wall.json adds a material and a blueprint by data; the
#      merged looks keep the import unchanged (and an override keeps a material's identity), the unchanged resolver
#      resolves the fixture's style, material and sources, and the live World3D/Structures, given that resolver, draws a
#      3 × 3 ring of those walls with a door: 8 wall tiles, a wall mesh, the `walls.by_style` profile of the resolved
#      style, and wall vertices in the material's colour (bindings3d `materials` by id or category, the resolver tint);
#   PEOPLE 4. model by sex and life stage: every person World3D/People draws (`bodies`, `count`) uses bindings3d
#      `people.models` child/female/male for their snapshot row (or `people.model` when that character is missing), at
#      least one man and one woman are drawn, and a model id with no character falls back to `people.model`;
#   PEOPLE 5. appearance: every drawn person's look and face variant equal their snapshot row (`look_ids`, `variants`),
#      children are `people.child_scale` (×0.75 infants; 1 for a child-made model), adults 1, and sleepers play `sleep`;
#   TREES 6. four seasons per species: for every species of bindings3d `flora.species`, each season's tree mesh carries
#      that species' (or its habit's) canopy colour of the season and the species' trunk colour;
#   TREES 7. bare winter: a habit listed in `flora.bare_in_season` drops its canopy in that season (fewer vertices than
#      summer, no canopy colour), any other habit keeps it;
#   TREES 8. data-only species: a species id the bindings do not list draws the fallback habit's models and colours;
#      every tree of get_trees on the live map is drawn (tree instances = trees).
#   Prints one `W3D_RENDER <GROUND|PEOPLE|TREES> PASS|FAIL <name> <detail>` line per check and
#   `W3D_RENDER_DONE failures=N checks=M`; exits 1 on any failure.
#   `godot --headless --path . --script res://tools/v8/w3d_render_test.gd [-- --write]`
#   Private hooks used (no public equivalent yet): Terrain `_meshes`, Structures `_resolver` / `_rebuild_walls`,
#   People `_model_of`, Flora `_season` / `_mesh_variant` / `_species_models` / `_habit` / `_instances`.
# Bridge methods called: via SimReads.mutate_raw reset_world, advance_ticks; via SimReads get_terrain_snapshot,
#   get_world_info, get_space, get_field_tiles, get_building_snapshot, get_agent_snapshot, get_trees, get_time_snapshot.
extends SceneTree

const Data3D = preload("res://scripts/world3d/data3d.gd")
const People3D = preload("res://scripts/world3d/people3d.gd")
const TerrainScript = preload("res://scripts/world3d/terrain3d.gd")
const FloraScript = preload("res://scripts/world3d/flora3d.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const MaterialResolver = preload("res://scripts/world/art/material_resolver.gd")
const HASHES: String = "res://tools/v8/mart2_chunk_hashes.json"
const COLOUR_EPS: float = 0.004

var _failures: int = 0
var _checks: int = 0
var _write: bool = false


## The world reads a standalone Terrain sees (no SimReads beside it, so SimReads.read calls these directly).
class FakeWorld extends Node:
	@warning_ignore("unused_signal")
	signal world_reset
	var reads: Dictionary = {}

	func get_terrain_snapshot() -> Dictionary:
		return reads.get("get_terrain_snapshot", {})

	func get_colony_versions() -> Dictionary:
		return reads.get("get_colony_versions", {})

	func get_world_info() -> Dictionary:
		return reads.get("get_world_info", {})

	func get_space() -> Dictionary:
		return reads.get("get_space", {})

	func get_field_tiles() -> Dictionary:
		return reads.get("get_field_tiles", {})

	func get_building_snapshot() -> Dictionary:
		return reads.get("get_building_snapshot", {})

	func get_time_snapshot() -> Dictionary:
		return reads.get("get_time_snapshot", {})

	# Style 2 terrain keeps workshop footprints out of the grass mask.
	func get_workshops() -> Dictionary:
		return reads.get("get_workshops", {})


## World3D's stand-in for a standalone layer (the layers read `stamp` from their parent; an empty stamp polls).
class FakeView extends Node3D:
	var stamp: Dictionary = {}


func _init() -> void:
	_write = OS.get_cmdline_user_args().has("--write")
	call_deferred("_run")


func _check(group: String, name: String, ok: bool, detail: String = "") -> void:
	_checks += 1
	print("W3D_RENDER %s %s %s %s" % [group, "PASS" if ok else "FAIL", name, detail])
	if not ok:
		_failures += 1


func _run() -> void:
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	for _i: int in 5:
		await process_frame
	var world: Node = main.get_node("WorldSim")
	var w3d: Node = main.get_node_or_null("World3D")
	if w3d == null:
		_check("GROUND", "world3d_present", false)
		_finish()
		return
	world.process_mode = Node.PROCESS_MODE_DISABLED
	SimReads.mutate(world, "reset_world", [0])
	for _i: int in 5:
		await process_frame
	SimReads.mutate(world, "advance_ticks", [1])
	for _i: int in 10:
		await process_frame
	var terrain: Node = w3d.get_node("Terrain")
	terrain.call("build_all")
	for _i: int in 5:
		await process_frame
	await _ground(world, terrain)
	_grass(terrain)
	_data_only_wall(w3d)
	_people(world, w3d)
	_trees(world, w3d)
	_finish()


func _finish() -> void:
	print("W3D_RENDER_DONE failures=%d checks=%d" % [_failures, _checks])
	quit(1 if _failures > 0 else 0)


static func _mesh_sha(mesh: Mesh) -> String:
	var ctx: HashingContext = HashingContext.new()
	ctx.start(HashingContext.HASH_SHA256)
	if mesh == null:
		return ""
	for s: int in mesh.get_surface_count():
		var arrays: Array = mesh.surface_get_arrays(s)
		ctx.update(var_to_bytes(arrays[Mesh.ARRAY_VERTEX]))
		ctx.update(var_to_bytes(arrays[Mesh.ARRAY_COLOR]))
	return ctx.finish().hex_encode()


static func _part_equal(a: Mesh, b: Mesh, part: int) -> String:
	if a == null or b == null:
		return "missing"
	var x: Variant = a.surface_get_arrays(0)[part]
	var y: Variant = b.surface_get_arrays(0)[part]
	if x == y:
		return "equal"
	var n: int = 0
	var first: int = -1
	for i: int in mini(x.size(), y.size()):
		if x[i] != y[i]:
			n += 1
			if first < 0:
				first = i
	return "%d of %d/%d differ, first %d: %s vs %s" % [n, x.size(), y.size(), first, x[first] if first >= 0 else "", y[first] if first >= 0 else ""]


func _chunk_mesh(terrain: Node, key: Vector2i) -> Mesh:
	var mi: MeshInstance3D = (terrain.get("_meshes") as Dictionary).get(key)
	return mi.mesh if mi != null else null


## GROUND 1 and 2.
func _ground(world: Node, terrain: Node) -> void:
	var chunk: int = maxi(4, int(Data3D.param("terrain.chunk_tiles", 32)))
	var buildings: Dictionary = SimReads.read(world, "get_building_snapshot")
	var kinds: PackedByteArray = buildings.get("kinds", PackedByteArray())
	var bxs: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var bys: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	var hearth: Vector2i = Vector2i(-1, -1)
	for i: int in mini(kinds.size(), mini(bxs.size(), bys.size())):
		if kinds[i] == 0:
			hearth = Vector2i(bxs[i], bys[i])
			break
	_check("GROUND", "hearth_exists", hearth.x >= 0, str(hearth))
	if hearth.x < 0:
		return
	var key: Vector2i = Vector2i(floori(float(hearth.x) / chunk), floori(float(hearth.y) / chunk))
	var live: String = _mesh_sha(_chunk_mesh(terrain, key))
	# A second Terrain fed the same snapshots (the GroundBaker re-bake of the 2D test).
	var fake: FakeWorld = FakeWorld.new()
	for getter: String in ["get_terrain_snapshot", "get_colony_versions", "get_world_info", "get_space", "get_field_tiles", "get_building_snapshot", "get_time_snapshot"]:
		fake.reads[getter] = SimReads.read(world, getter).duplicate(true)
	var view: FakeView = FakeView.new()
	root.add_child(view)
	view.add_child(fake)
	var second: Node3D = TerrainScript.new()
	view.add_child(second)
	second.call("setup", fake)
	second.call("build_all")
	var fresh: String = _mesh_sha(_chunk_mesh(second, key))
	var third: Node3D = TerrainScript.new()
	view.add_child(third)
	third.call("setup", fake)
	third.call("build_all")
	var again: String = _mesh_sha(_chunk_mesh(third, key))
	third.free()
	var name: String = "w3d_seed0_tick1_chunk_%d_%d" % [key.x, key.y]
	var pinned: Dictionary = {}
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(HASHES))
	if parsed is Dictionary:
		pinned = parsed
	if _write:
		var file: FileAccess = FileAccess.open(HASHES, FileAccess.WRITE)
		file.store_string(JSON.stringify({"about": "SHA-256 of the hearth's 3D terrain chunk vertex and colour arrays (w3d_render_test.gd); rewrite with -- --write after an intended change", name: fresh}, "  ") + "\n")
		file.close()
		pinned[name] = fresh
	_check("GROUND", "chunk_deterministic", not fresh.is_empty() and fresh == again, "%s vs %s" % [fresh.left(16), again.left(16)])
	_check("GROUND", "chunk_hash_pinned", str(pinned.get(name, "")) == fresh, "%s %s pinned %s" % [name, fresh.left(16), str(pinned.get(name, "")).left(16)])
	# The live layer shows the current world: its chunk equals a fresh build of the same snapshots.
	_check("GROUND", "live_chunk_current", live == fresh, "live %s fresh %s (vertices %s; colours %s)" % [live.left(16), fresh.left(16),
		_part_equal(_chunk_mesh(terrain, key), _chunk_mesh(second, key), Mesh.ARRAY_VERTEX), _part_equal(_chunk_mesh(terrain, key), _chunk_mesh(second, key), Mesh.ARRAY_COLOR)])
	# GROUND 2: dig one raised tile inside a chunk.
	var width: int = int(second.get("width"))
	var height: int = int(second.get("height"))
	var path_mesh: Mesh = _chunk_mesh(second, key)
	var paths: PackedByteArray = fake.reads["get_terrain_snapshot"].get("path_stage", PackedByteArray()).duplicate()
	paths.resize(width * height)
	paths[hearth.y * width + hearth.x] = 2
	fake.reads["get_terrain_snapshot"]["path_stage"] = paths
	fake.reads["get_colony_versions"]["path_version"] = int(fake.reads["get_colony_versions"].get("path_version", 0)) + 1
	second.call("_poll_paths")
	_check("GROUND", "path_update_without_remesh", second.get("_path_stage") == paths
		and _chunk_mesh(second, key) == path_mesh)
	var dug_tiles: Dictionary = {}
	var space: Dictionary = fake.reads["get_space"]
	var dxs: PackedInt32Array = space.get("dug_xs", PackedInt32Array())
	var dys: PackedInt32Array = space.get("dug_ys", PackedInt32Array())
	for i: int in mini(dxs.size(), dys.size()):
		dug_tiles[Vector2i(dxs[i], dys[i])] = true
	var target: Vector2i = Vector2i(-1, -1)
	for ty: int in range(2, height - 2):
		for tx: int in range(2, width - 2):
			var t: Vector2i = Vector2i(tx, ty)
			if posmod(tx, chunk) < 2 or posmod(tx, chunk) > chunk - 3 or posmod(ty, chunk) < 2 or posmod(ty, chunk) > chunk - 3:
				continue
			if dug_tiles.has(t) or float(second.call("stand_height", t)) <= float(second.call("base_height", t)) + 0.05:
				continue
			target = t
			break
		if target.x >= 0:
			break
	_check("GROUND", "dig_candidate", target.x >= 0, str(target))
	if target.x < 0:
		return
	var before: Dictionary = {}
	for k: Vector2i in second.get("_meshes"):
		before[k] = [_chunk_mesh(second, k), _mesh_sha(_chunk_mesh(second, k))]
	var stand_before: float = float(second.call("stand_height", target))
	var dug: Dictionary = space.duplicate(true)
	var new_xs: PackedInt32Array = dxs.duplicate()
	var new_ys: PackedInt32Array = dys.duplicate()
	new_xs.append(target.x)
	new_ys.append(target.y)
	dug["dug_xs"] = new_xs
	dug["dug_ys"] = new_ys
	fake.reads["get_space"] = dug
	await process_frame
	second.call("build_all")
	var replaced: Array[Vector2i] = []
	var changed: Array[Vector2i] = []
	for k: Vector2i in before:
		var now: Mesh = _chunk_mesh(second, k)
		if now != before[k][0]:
			replaced.append(k)
		if _mesh_sha(now) != str(before[k][1]):
			changed.append(k)
	var want: Vector2i = Vector2i(floori(float(target.x) / chunk), floori(float(target.y) / chunk))
	_check("GROUND", "dig_rebuilds_one_chunk", replaced == [want] and changed == [want]
		and float(second.call("stand_height", target)) < stand_before - 0.05,
		"tile %s: replaced %s changed %s (want %s); stand %.2f -> %.2f" % [target, replaced, changed, want, stand_before, float(second.call("stand_height", target))])
	view.queue_free()


## GROUND 3: the data-only wall through the unchanged resolver into the live Structures layer.
func _data_only_wall(w3d: Node) -> void:
	var fixture: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://tools/v8/fixtures/data_only_wall.json"))
	var base: String = ArtData.ROOT + "materials/"
	var styles: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(base + "construction_styles.json"))
	var looks: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(base + "material_looks.json"))
	var defaults: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(base + "category_defaults.json"))
	var additions: Dictionary = {}
	for m: Dictionary in fixture.get("materials", []):
		additions[str(m["id"])] = m
	var original: Dictionary = looks.duplicate(true)
	var extended: Dictionary = ArtData.material_looks(looks, additions)
	_check("GROUND", "data_only_looks_preserve_import", looks == original and extended["materials"].size() == looks["materials"].size() + additions.size())
	var existing: Dictionary = original["materials"][0]
	var tint: Dictionary = {"dL": -25, "da": 12, "db": 18}
	var overridden: Dictionary = ArtData.material_looks(original, {str(existing["id"]): {"tint": tint}})
	var first: Dictionary = overridden["materials"][0]
	_check("GROUND", "data_only_looks_override_keeps_identity", overridden["materials"].size() == original["materials"].size()
		and first["id"] == existing["id"] and first["sim_id"] == existing["sim_id"] and first["category"] == existing["category"]
		and first["tint"] == tint and original == looks)
	var added: Dictionary = fixture.get("blueprints", {})
	for s: Variant in styles.get("styles", []):
		var d: Dictionary = s
		if added.has(str(d.get("id", ""))):
			(d["sim_blueprints"] as Array).append_array(added[str(d["id"])])
	var resolver: MaterialResolver = MaterialResolver.new()
	resolver.load_schema_defaults(JSON.parse_string(FileAccess.get_file_as_string(base + "construction_styles.schema.json")))
	resolver.load_documents(styles, extended, defaults)
	var blueprint: String = str((added.values()[0] as Array)[0])
	var sim_id: int = int((fixture["materials"] as Array)[0]["sim_id"])
	var r: Dictionary = resolver.resolve(blueprint, sim_id, "", "wall")
	var expect: Dictionary = fixture.get("expect", {})
	var match_all: bool = resolver.problems.is_empty()
	for k: String in expect.keys():
		match_all = match_all and str(r.get(k, "")) == str(expect[k])
	_check("GROUND", "data_only_resolves", match_all, "%s + %d -> style %s (%s), material %s (%s); problems %s" % [blueprint, sim_id,
		r.get("style_id"), r.get("style_source"), r.get("material_key"), r.get("tint_source"), resolver.problems])
	# The live layer draws a ring of those walls (an int32 column holds the u32 material id, as get_space sends it).
	var structures: Node = w3d.get_node("Structures")
	var shipped: Variant = structures.get("_resolver")
	structures.set("_resolver", resolver)
	var raw: int = sim_id - 4294967296 if sim_id >= 2147483648 else sim_id
	var wxs: PackedInt32Array = PackedInt32Array()
	var wys: PackedInt32Array = PackedInt32Array()
	var doors: PackedByteArray = PackedByteArray()
	var wkinds: PackedInt32Array = PackedInt32Array()
	var mats: PackedInt32Array = PackedInt32Array()
	var wstyles: PackedStringArray = PackedStringArray()
	var cats: PackedStringArray = PackedStringArray()
	for y: int in range(4, 7):
		for x: int in range(4, 7):
			if x == 5 and y == 5:
				continue
			wxs.append(x)
			wys.append(y)
			doors.append(1 if x == 5 and y == 6 else 0)
			wkinds.append(2 if x == 5 and y == 6 else 1)
			mats.append(raw)
			wstyles.append(blueprint)
			cats.append("")
	var space: Dictionary = {"wall_xs": wxs, "wall_ys": wys, "wall_is_door": doors, "wall_kinds": wkinds, "wall_materials": mats,
		"wall_styles": wstyles, "wall_categories": cats}
	structures.call("_rebuild_walls", space, {})
	var entry: Dictionary = structures.call("_wall_entry", blueprint, raw & 0xFFFFFFFF, "", false)
	var by_style: Dictionary = Data3D.binding("walls.by_style", {})
	var want_profile: String = str(by_style.get(str(r.get("style_id", "")), Data3D.binding("walls.fallback", "")))
	var colour: Color = Data3D.material_colour(str(r.get("material_key", "")), str(r.get("category", "")))
	colour = colour * (1.0 + float((r.get("tint", {}) as Dictionary).get("dL", 0)) / 70.0)
	var mesh_colours: int = 0
	var vertices: int = 0
	for child: Node in structures.get_children():
		if child is MeshInstance3D and (child as MeshInstance3D).mesh != null and str(child.name).begins_with("Walls"):
			var mesh: Mesh = (child as MeshInstance3D).mesh
			for s: int in mesh.get_surface_count():
				var arrays: Array = mesh.surface_get_arrays(s)
				vertices += (arrays[Mesh.ARRAY_VERTEX] as PackedVector3Array).size()
				var cols: Variant = arrays[Mesh.ARRAY_COLOR]
				if cols is PackedColorArray:
					for c: Color in cols:
						if absf(c.r - colour.r) < COLOUR_EPS * 4 and absf(c.g - colour.g) < COLOUR_EPS * 4 and absf(c.b - colour.b) < COLOUR_EPS * 4:
							mesh_colours += 1
	_check("GROUND", "data_only_wall_renders", int(structures.call("wall_tile_count")) == 8 and vertices > 0 and mesh_colours > 0
		and str(entry.get("profile", "")) == want_profile and not want_profile.is_empty() and str(entry.get("material", "")) == str(r.get("material_key", "")),
		"walls %d, vertices %d, in material colour %d, profile %s (want %s), material %s" % [int(structures.call("wall_tile_count")), vertices,
		mesh_colours, entry.get("profile", ""), want_profile, entry.get("material", "")])
	structures.set("_resolver", shipped)


## PEOPLE 4 and 5.
func _people(world: Node, w3d: Node) -> void:
	var people: Node = w3d.get_node("People")
	var snap: Dictionary = SimReads.read(world, "get_agent_snapshot")
	var ids: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var stages: PackedByteArray = snap.get("life_stages", PackedByteArray())
	var sexes: PackedByteArray = snap.get("sexes", PackedByteArray())
	var looks: PackedInt32Array = snap.get("look_ids", PackedInt32Array())
	var variants: PackedByteArray = snap.get("variants", PackedByteArray())
	var infants: PackedByteArray = snap.get("infants", PackedByteArray())
	var states: PackedByteArray = snap.get("states", PackedByteArray())
	var targets: PackedByteArray = snap.get("target_kinds", PackedByteArray())
	var models: Dictionary = Data3D.binding("people.models", {})
	var fallback: String = str(Data3D.binding("people.model", ""))
	var child_scale: float = float(Data3D.param("people.child_scale", 0.62))
	var bodies: Array = people.call("bodies", 0)
	_check("PEOPLE", "everyone_drawn", ids.size() > 0 and bodies.size() == ids.size() and int(people.call("count")) == ids.size(),
		"%d rows, %d bodies, count %d" % [ids.size(), bodies.size(), int(people.call("count"))])
	var wrong_model: Array[String] = []
	var wrong_look: Array[String] = []
	var drawn: Dictionary = {}
	for i: int in ids.size():
		var state: Dictionary = people.call("state_of", ids[i])
		var child: bool = i < stages.size() and stages[i] == 1
		var key: String = "child" if child else ("female" if i < sexes.size() and sexes[i] == 1 else "male")
		var want: String = str(models.get(key, ""))
		if want.is_empty() or not FileAccess.file_exists("res://assets/3d/vat/%s_vat.json" % want):
			want = fallback
		# Style 2 (the default) picks the kit body by age and sex; its scale is the kit's own.
		var kit: int = People3D.STYLE2_MODELS.find(People3D.style2_model(snap, i)) if Data3D.style2() else -1
		if kit >= 0:
			want = People3D.STYLE2_MODELS[kit]
		var got: String = str(state.get("model", ""))
		drawn[key] = true
		if got != want:
			wrong_model.append("%d %s: %s want %s" % [ids[i], key, got, want])
		var own_child: bool = got == str(models.get("child", ""))
		var scale: float = ((1.0 if own_child else child_scale) * (0.75 if i < infants.size() and infants[i] == 1 else 1.0)) if child else 1.0
		if kit >= 0:
			scale = People3D.STYLE2_SCALES[kit]
		var asleep: bool = i < states.size() and i < targets.size() and states[i] == 3 and targets[i] == 3
		if int(state.get("look_id", -1)) != (looks[i] if i < looks.size() else 0) or int(state.get("variant", -1)) != (int(variants[i]) if i < variants.size() else 0) \
				or not is_equal_approx(float(state.get("scale", -1.0)), scale) or (asleep and str(state.get("clip", "")) != "sleep"):
			wrong_look.append("%d: %s" % [ids[i], state])
	_check("PEOPLE", "model_by_sex_and_stage", wrong_model.is_empty() and drawn.has("male") and drawn.has("female"), "%s; drawn %s" % [wrong_model, drawn.keys()])
	_check("PEOPLE", "appearance_scale_clip", wrong_look.is_empty() and not ids.is_empty(), str(wrong_look.slice(0, 3)))
	# A model id with no character falls back to `people.model` (data only: a binding may name art not built yet).
	var people_bindings: Dictionary = (Data3D.binding("people", {}) as Dictionary)
	var saved: Dictionary = (people_bindings.get("models", {}) as Dictionary).duplicate()
	(people_bindings["models"] as Dictionary)["male"] = "no_such_character_e2"
	(people_bindings["models"] as Dictionary)["female"] = "no_such_character_e2"
	var adult: int = -1
	for i: int in ids.size():
		if i >= stages.size() or stages[i] != 1:
			adult = i
			break
	# The binding fallback is the legacy path; Style 2 never reads `people.models`.
	var style2: bool = bool(people.get("_style2"))
	people.set("_style2", false)
	var model_now: String = str(people.call("_model_of", snap, adult)) if adult >= 0 else ""
	people.set("_style2", style2)
	people_bindings["models"] = saved
	_check("PEOPLE", "missing_character_falls_back", adult >= 0 and model_now == fallback, "%s want %s" % [model_now, fallback])


## Ground-cover exclusions use actual construction and field snapshots.
func _grass(terrain: Node) -> void:
	var space: Dictionary = {
		"wall_xs": PackedInt32Array([2]), "wall_ys": PackedInt32Array([3]),
		"floor_xs": PackedInt32Array([7]), "floor_ys": PackedInt32Array([4]),
	}
	var fields: Dictionary = {"xs": PackedInt64Array([5]), "ys": PackedInt64Array([7])}
	var rooms: Dictionary = {
		"ids": PackedInt32Array([10, 11]), "roofed": PackedByteArray([1, 0]),
		"tile_xs": PackedInt32Array([9, 11]), "tile_ys": PackedInt32Array([9, 11]),
		"tile_rooms": PackedInt32Array([10, 11]),
	}
	var blocked: Dictionary = TerrainScript._grass_occupied_tiles(space, fields, {}, rooms)
	_check("GRASS", "built_field_roof_exclusions", blocked.size() == 4 and blocked.has(Vector2i(2, 3))
		and blocked.has(Vector2i(7, 4)) and blocked.has(Vector2i(5, 7)) and blocked.has(Vector2i(9, 9))
		and not blocked.has(Vector2i(11, 11)))
	var grass: Node = terrain.get_node("Grass")
	var reserved: int = 0
	for count: int in (grass.get("_desired") as Dictionary).values():
		reserved += count
	var cap: int = int(Data3D.mood("detail.grass.cap", 150000))
	_check("GRASS", "camera_budget", reserved > 0 and reserved <= cap, "%d / %d" % [reserved, cap])


## TREES 6, 7 and 8.
func _trees(world: Node, w3d: Node) -> void:
	var flora: Node3D = FloraScript.new()
	var species: Dictionary = Data3D.binding("flora.species", {})
	var by_species: Dictionary = Data3D.binding("flora.species_canopy_by_season", {})
	var by_habit: Dictionary = Data3D.binding("flora.canopy_by_season", {})
	var bare: Dictionary = Data3D.binding("flora.bare_in_season", {})
	var trunks: Dictionary = Data3D.binding("flora.trunk", {})
	var bad: Array[String] = []
	var bare_ok: Array[String] = []
	var shell_ok: bool = true
	var ids: Array = species.keys()
	ids.append("fixture_alder_e2")
	for sp: String in ids:
		var model_list: Array = flora.call("_species_models", sp)
		var habit: String = str(flora.call("_habit", sp))
		var model: String = str(model_list[0])
		var table: Array = by_species.get(sp, by_habit.get(habit, []))
		var summer: int = -1
		for season: int in 4:
			flora.set("_season", season)
			var mesh: Mesh = flora.call("_mesh_variant", model, "species=" + sp)
			var cols: PackedColorArray = PackedColorArray()
			var leaf_cards: bool = false
			if mesh != null and mesh.get_surface_count() > 0:
				var arrays: Array = mesh.surface_get_arrays(0)
				cols = arrays[Mesh.ARRAY_COLOR]
				var flags: PackedVector2Array = arrays[Mesh.ARRAY_TEX_UV2]
				for flag: Vector2 in flags:
					if flag.x < -1.5:
						leaf_cards = true
						break
			var n: int = cols.size()
			if season == 1:
				summer = n
			# Numbers in JSON are floats: compare by value (a bare Array.has(int) never matches them).
			var is_bare: bool = (bare.get(habit, []) as Array).any(func(v: Variant) -> bool: return int(v) == season)
			shell_ok = shell_ok and leaf_cards != is_bare
			var canopy: Color = Data3D.colour(table[season] if season < table.size() else [], Color(-1, -1, -1))
			var has_canopy: bool = canopy.r >= 0.0 and _has_colour(cols, canopy)
			var trunk: Color = Data3D.colour(trunks.get(sp, trunks.get("default", [])), Color(-1, -1, -1))
			if (is_bare and has_canopy) or (not is_bare and not has_canopy) or (trunk.r >= 0.0 and not _has_colour(cols, trunk)):
				bad.append("%s s%d (habit %s): canopy %s present %s, trunk present %s" % [sp, season, habit, canopy, has_canopy, _has_colour(cols, trunk)])
			if is_bare and n < summer:
				bare_ok.append("%s s%d %d<%d" % [sp, season, n, summer])
			elif is_bare:
				bad.append("%s s%d not bare (%d vs %d)" % [sp, season, n, summer])
	_check("TREES", "four_seasons_per_species", bad.is_empty() and species.size() > 0, str(bad.slice(0, 4)))
	_check("TREES", "bare_winter", not bare_ok.is_empty(), str(bare_ok))
	_check("TREES", "seasonal_leaf_shell", shell_ok)
	var fallback_habit: String = str(Data3D.binding("flora.fallback_habit", ""))
	var habit_models: Array = ((Data3D.binding("flora.habits", {}) as Dictionary).get(fallback_habit, {}) as Dictionary).get("models", [])
	_check("TREES", "data_only_species", str(flora.call("_habit", "fixture_alder_e2")) == fallback_habit and flora.call("_species_models", "fixture_alder_e2") == habit_models and not habit_models.is_empty(),
		"habit %s models %s" % [flora.call("_habit", "fixture_alder_e2"), flora.call("_species_models", "fixture_alder_e2")])
	flora.free()
	# Every live tree is drawn once (stumps aside: none at tick 1).
	var live: Node = w3d.get_node("Flora")
	var trees: int = (SimReads.read(world, "get_trees").get("xs", PackedInt32Array()) as PackedInt32Array).size()
	var drawn: int = 0
	var instances: Dictionary = live.get("_instances")
	for k: String in instances:
		if k.begins_with("trees:"):
			drawn += (instances[k] as MultiMeshInstance3D).multimesh.instance_count
	_check("TREES", "every_tree_drawn", trees > 0 and drawn == trees, "%d trees, %d tree instances" % [trees, drawn])


static func _has_colour(cols: PackedColorArray, want: Color) -> bool:
	for c: Color in cols:
		if absf(c.r - want.r) < COLOUR_EPS and absf(c.g - want.g) < COLOUR_EPS and absf(c.b - want.b) < COLOUR_EPS:
			return true
	return false
