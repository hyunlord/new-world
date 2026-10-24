# mart21_trees_test.gd — the M-ART2.1 tree and season tests (goal §6), headless and windowed.
# Purpose: boots scenes/main.tscn on colony seed 0 (reset_world(0), the clock held, tick 1) and checks on the real
#   TreeLayer and GroundLayer:
#   1. every catalogue species × size resolves in all four seasons through TreeLayer.make_plant (the game's sprite
#      path): a deciduous tree's winter sprite is its bare frame (and no foliage frame), an evergreen's is a foliage
#      frame with a snow band, other seasons draw foliage frames;
#   2. the data-only species: assets/v8art/flora/tests/test_alder.json (ART-TREES' fixture, not in the catalogue) added
#      by data to the layer's resolver draws in all four seasons with its own season records (unchanged code);
#   3. ¼× canopy: at zoom 0.25 the chunks in view are retextured at factor 4 with the canopy on, a forest tile's interior
#      in the baked chunk is canopy (its mean hue lies within 0.06 turns of the canopy texture's, and differs from the
#      same tile baked at 1×), and every held tree sprite in those chunks is hidden; back at 1× they show again;
#   4. windowed (`-- --rendered`): each species' large summer frame drawn through its season material keeps canopy
#      saturation ≥ 0.30 (masked pixels; trees are never tone-capped) and autumn moves an oak's canopy mean the
#      way its Lab record says (b* up) while pine's stays; the alder's autumn differs from oak's (its own record).
#   3b. the canopy edge on a synthetic map: an isolated tree tile is interior colour at its centre, the darkened edge
#      colour 2 px inside each of its four sides, and ground at each of its four corners (rounded away);
#   3c. TreeLayer.season_state on hand-worked calendar snapshots (first season unblended, a season's first day blended
#      by the hour, the previous season by code);
#   5. households (§2.3): agent_layer.gd `household_keys` on hand-built families (a couple, their child, an orphan
#      child with a living father, a widow, a single) gives the keys worked out by hand; at ¼× every person gets a
#      household and a ring colour from `lod.household_ring_rgb` read back from the LOD markers' instance data, on the
#      1,000-person stress world loaded after the colony (a reset must not leave stale households): members of one
#      household share the colour, and more than one colour is in use;
#   6. windowed: the five ¼× map classes on the seed-0 screen (forest = tree tiles' canopy, meadow = the plain's ground,
#      bare earth = the camp clearing, rock = undug hill tops, water), in summer, each the mean colour of the central
#      half of ≥ 20 interior tiles (all eight neighbours of the same class; any river tile, rivers being 1–2 tiles wide)
#      away from the HUD, differ pairwise by CIE76 ΔE ≥ 10.
#   Prints one MART21_TREES line per check and MART21_TREES_DONE failures=N; exits 1 on any failure.
#   `godot --headless --path . --script res://tools/v8/mart21_trees_test.gd [-- --rendered (windowed)]`
# Bridge methods called: reset_world, advance_ticks (direct); the layers read the rest.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Tone = preload("res://scripts/world/art/tone.gd")
const FloraResolver = preload("res://scripts/world/art/flora_resolver.gd")
const GroundBaker = preload("res://scripts/world/art/ground_baker.gd")
const TILE: int = View.TILE

var _failures: int = 0
var _main: Node = null


func _init() -> void:
	call_deferred("_run")


func _check(name: String, ok: bool, detail: String) -> void:
	print("MART21_TREES %s %s — %s" % ["PASS" if ok else "FAIL", name, detail])
	if not ok:
		_failures += 1


func _frames(n: int) -> void:
	for _i: int in n:
		await process_frame


func _run() -> void:
	_main = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(_main)
	var world: Node = _main.get_node("WorldSim")
	await process_frame
	world.call("reset_world", 0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	world.call("advance_ticks", 1)
	await _frames(20)
	var trees: Node = _main.get_node("ColonyView/TreeLayer")
	var ground: Node = _main.get_node("ColonyView/GroundLayer")
	var flora: FloraResolver = trees.call("flora")
	_check("flora_loaded", flora.problems.is_empty() and flora.species_ids().size() >= 3, "%d species, problems %s" % [flora.species_ids().size(), flora.problems])
	await _check_seasons(trees, flora)
	await _check_alder(trees, flora)
	await _check_canopy(trees, ground)
	_check_canopy_edges()
	_check_season_state()
	_check_household_rule()
	await _check_households(world)
	if OS.get_cmdline_user_args().has("--rendered"):
		if DisplayServer.get_name() == "headless":
			_check("rendered_backend", false, "--rendered needs a window")
		else:
			await _check_rendered(trees, flora)
			await _check_map_classes(ground, "summer", true)
			await _check_map_classes(ground, "autumn", false)
			await _check_map_classes(ground, "winter", false)
	else:
		print("MART21_TREES SKIP rendered — run windowed with -- --rendered for canopy saturation and season colours")
	trees.call("force_season", "", "")
	print("MART21_TREES_DONE failures=%d" % _failures)
	quit(1 if _failures > 0 else 0)


## The res:// texture path a sprite draws.
func _path(s: Sprite2D) -> String:
	return s.texture.resource_path if s != null and s.texture != null else ""


func _check_seasons(trees: Node, flora: FloraResolver) -> void:
	var seasons: Array[String] = flora.seasons()
	var bad: Array[String] = []
	var drawn: int = 0
	for k: int in seasons.size():
		trees.call("force_season", seasons[posmod(k - 1, seasons.size())], seasons[k], 1.0)
		await _frames(2)
		for sp: String in flora.species_ids():
			for size: String in ["small", "large"]:
				var s: Sprite2D = trees.call("make_plant", sp, size, 0, "", Vector2(100, 100))
				if s == null:
					bad.append("%s %s %s: no sprite" % [sp, size, seasons[k]])
					continue
				drawn += 1
				var r: Dictionary = flora.resolve(sp, size, seasons[k], 0)
				var tint: Dictionary = r["tint"]
				var want: String = FloraResolver.res_path(str(r["path"]))
				if _path(s) != want:
					bad.append("%s %s %s draws %s, not %s" % [sp, size, seasons[k], _path(s), want])
				# A bare frame comes only from the catalogue's bare list; a foliage frame from its variants.
				var frames: Dictionary = (flora.get("_species")[sp]["sizes"] as Dictionary)[size]
				var listed: Array = frames["bare"] if str(tint["frame"]) == "bare" else frames["variants"]
				var in_list: bool = false
				for f: Variant in listed:
					in_list = in_list or FloraResolver.res_path(str((f as Dictionary)["path"])) == _path(s)
				if not in_list:
					bad.append("%s %s %s: frame not in its %s list" % [sp, size, seasons[k], tint["frame"]])
				var m: ShaderMaterial = s.material as ShaderMaterial
				var snow: bool = m != null and bool(m.get_shader_parameter("has_snow")) and float(m.get_shader_parameter("snow_to")) > 0.0
				if snow != (int((tint["snow_cap"] as Dictionary)["depth_px"]) > 0):
					bad.append("%s %s %s: snow %s against the record" % [sp, size, seasons[k], snow])
				s.free()
	# The deciduous winter rule itself, read from the data rather than assumed per species.
	var bare_winter: int = 0
	for sp: String in flora.species_ids():
		var r: Dictionary = flora.resolve(sp, "large", seasons[seasons.size() - 1], 0)
		if str(r["habit"]) == str((ArtData.bindings().get("trees", {}) as Dictionary).get("habit", "")) and str((r["tint"] as Dictionary)["frame"]) == "bare":
			bare_winter += 1
	_check("four_seasons_per_species", bad.is_empty() and drawn == seasons.size() * flora.species_ids().size() * 2 and bare_winter >= 2, "%d sprites over %d seasons; %d deciduous species bare in winter; problems %s" % [drawn, seasons.size(), bare_winter, bad.slice(0, 6)])


func _check_alder(trees: Node, flora: FloraResolver) -> void:
	var record: Variant = JSON.parse_string(FileAccess.get_file_as_string("res://assets/v8art/flora/tests/test_alder.json"))
	var id: String = str((record as Dictionary).get("id", "")) if record is Dictionary else ""
	var in_catalogue: bool = flora.species_ids().has(id)
	var added: bool = record is Dictionary and flora.add_species(record)
	var seasons: Array[String] = flora.seasons()
	var drawn: int = 0
	var own: int = 0
	for k: int in seasons.size():
		trees.call("force_season", seasons[posmod(k - 1, seasons.size())], seasons[k], 1.0)
		await _frames(2)
		var s: Sprite2D = trees.call("make_plant", id, "large", 0, "", Vector2(100, 100))
		if s != null and s.texture != null:
			drawn += 1
			var m: ShaderMaterial = s.material as ShaderMaterial
			var want: Array = (((record as Dictionary)["seasons"] as Dictionary)[seasons[k]] as Dictionary)["delta_lab"]
			var got: Vector3 = m.get_shader_parameter("delta_to") if m != null else Vector3(INF, 0, 0)
			if got.is_equal_approx(Vector3(float(want[0]), float(want[1]), float(want[2]))):
				own += 1
			# Its winter draws its own bare frame (the fixture's path, not the catalogue's oak record).
			if str((((record as Dictionary)["seasons"] as Dictionary)[seasons[k]] as Dictionary).get("frame", "foliage")) == "bare":
				var bare: String = FloraResolver.res_path(str(((((record as Dictionary)["sizes"] as Dictionary)["large"] as Dictionary)["bare"] as Array)[0]["path"]))
				if _path(s) != bare:
					own -= 1
			s.free()
	_check("data_only_species", not in_catalogue and added and drawn == seasons.size() and own == seasons.size(), "%s not in the catalogue, added by data: %s; drawn in %d/%d seasons, its own Lab record in %d" % [id, added, drawn, seasons.size(), own])


func _check_canopy(trees: Node, ground: Node) -> void:
	var camera: Camera2D = _main.get_node("Camera2D") as Camera2D
	trees.call("force_season", "spring", "summer", 1.0)
	await _frames(3)
	var map: GroundBaker.GroundMap = ground.call("ground_map")
	# A tree tile whose 3 × 3 neighbourhood is all trees (its quadrants are uniform canopy).
	var cell: Vector2i = Vector2i(-1, -1)
	for y: int in range(1, map.height - 1):
		for x: int in range(1, map.width - 1):
			var full: bool = true
			for d: Vector2i in [Vector2i(-1, -1), Vector2i(0, -1), Vector2i(1, -1), Vector2i(-1, 0), Vector2i(0, 0), Vector2i(1, 0), Vector2i(-1, 1), Vector2i(0, 1), Vector2i(1, 1)]:
				full = full and map.tree_at(x + d.x, y + d.y) > 0 and map.at(x + d.x, y + d.y) >= 0
			if full:
				cell = Vector2i(x, y)
				break
		if cell.x >= 0:
			break
	if cell.x < 0:
		_check("canopy_tile", false, "no tree tile surrounded by trees on seed 0")
		return
	camera.position = View.tile_centre(cell)
	camera.zoom = Vector2.ONE
	camera.force_update_scroll()
	await _frames(40)
	ground.set("keep_images", true)
	var size: int = int(ground.call("baker").chunk_tiles)
	var c: Vector2i = Vector2i(floori(float(cell.x) / size), floori(float(cell.y) / size))
	ground.call("bake_chunk_now", c)
	var at1: Color = _tile_mean(ground.call("chunk_image", c), cell, c, size)
	var shown1: int = int(trees.call("tree_sprites_visible"))
	camera.zoom = Vector2(0.25, 0.25)
	camera.force_update_scroll()
	await _frames(120)
	var chunks: Dictionary = ground.get("_chunks")
	var res: Dictionary = ground.get("_res_of")
	var quarter: int = 0
	for k: Vector2i in chunks.keys():
		if int(res.get(k, 1)) == 4:
			quarter += 1
	ground.call("bake_chunk_now", c)
	var at4: Color = _tile_mean(ground.call("chunk_image", c), cell, c, size)
	var canopy_img: Image = ArtData.image_at(FloraResolver.res_path(str((trees.call("flora") as FloraResolver).resolve(str((ArtData.bindings()["trees"]["species"][0] as Dictionary)["species"]), "large", "summer", 0)["map_canopy_path"])))
	var canopy_mean: Color = Tone.mean_rgb(canopy_img, 3)
	var hue: float = absf(at4.h - canopy_mean.h)
	hue = minf(hue, 1.0 - hue)
	var held: int = int(trees.call("tree_sprite_count"))
	var visible: int = int(trees.call("tree_sprites_visible"))
	var baked: int = 0
	for s_cell: Variant in (trees.get("_tree_sprites") as Dictionary).keys():
		if bool(ground.call("canopy_baked_at", s_cell)):
			baked += 1
	# Off-screen chunks kept in the LRU keep their old factor until they come into view (by design).
	_check("quarter_canopy_baked", int(res.get(c, 1)) == 4 and quarter > 0 and hue <= 0.06 and absf(at4.h - at1.h) > 0.03 and bool(ground.call("canopy_baked_at", cell)), "%d of %d cached chunks at factor 4 (the rest off-screen); forest tile %s mean hue %.3f at ¼× (canopy texture %.3f, distance %.3f ≤ 0.06), %.3f at 1×" % [quarter, chunks.size(), cell, at4.h, canopy_mean.h, hue, at1.h])
	_check("quarter_sprites_off", held > 0 and baked == held and visible == 0, "%d tree sprites held at ¼×, %d in canopy chunks, %d drawn (at 1× %d drawn)" % [held, baked, visible, shown1])
	camera.zoom = Vector2.ONE
	camera.position = View.tile_centre(cell)
	camera.force_update_scroll()
	await _frames(60)
	_check("sprites_back_at_1x", int(trees.call("tree_sprites_visible")) > 0, "%d tree sprites drawn after returning to 1×" % int(trees.call("tree_sprites_visible")))


func _tile_mean(img: Image, cell: Vector2i, c: Vector2i, size: int) -> Color:
	if img == null:
		return Color(0, 0, 0, 0)
	return Tone.mean_rgb(img.get_region(Rect2i((cell.x - c.x * size) * TILE + 8, (cell.y - c.y * size) * TILE + 8, 48, 48)))


## Draw `sprite` (its material) alone in a transparent SubViewport at 1× and read the pixels back.
func _render(sprite: Sprite2D) -> Image:
	var vp: SubViewport = SubViewport.new()
	vp.size = Vector2i(sprite.texture.get_size())
	vp.transparent_bg = true
	vp.disable_3d = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	root.add_child(vp)
	var copy: Sprite2D = Sprite2D.new()
	copy.texture = sprite.texture
	copy.material = sprite.material
	copy.centered = false
	copy.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	vp.add_child(copy)
	await process_frame
	await RenderingServer.frame_post_draw
	var img: Image = vp.get_texture().get_image()
	img.convert(Image.FORMAT_RGBA8)
	vp.queue_free()
	return img


## Mean colour and HSV saturation of the canopy-masked, opaque pixels of `img`.
func _masked(img: Image, mask: Image) -> Dictionary:
	var sum: Vector3 = Vector3.ZERO
	var sat: float = 0.0
	var n: int = 0
	for y: int in img.get_height():
		for x: int in img.get_width():
			var c: Color = img.get_pixel(x, y)
			if c.a < 1.0 or mask.get_pixel(x, y).r < 0.5:
				continue
			sum += Vector3(c.r, c.g, c.b)
			sat += c.s
			n += 1
	return {"mean": sum / maxf(1.0, n), "sat": sat / maxf(1.0, n), "n": n}


func _check_rendered(trees: Node, flora: FloraResolver) -> void:
	var low: Array[String] = []
	var sats: PackedStringArray = PackedStringArray()
	var autumn: Dictionary = {}
	for season: String in ["summer", "autumn"]:
		trees.call("force_season", season, season, 1.0)
		await _frames(2)
		for sp: String in flora.species_ids():
			var s: Sprite2D = trees.call("make_plant", sp, "large", 0, "", Vector2(100, 100))
			if s == null:
				continue
			var r: Dictionary = flora.resolve(sp, "large", season, 0)
			var mask: Image = ArtData.image_at(FloraResolver.res_path(str(r["canopy_mask"])))
			var m: Dictionary = _masked(await _render(s), mask)
			s.free()
			if season == "summer":
				sats.append("%s %.3f" % [sp, m["sat"]])
				if int(m["n"]) > 0 and float(m["sat"]) < 0.30:
					low.append(sp)
			autumn["%s/%s" % [sp, season]] = m["mean"]
	_check("summer_canopy_saturation", low.is_empty() and sats.size() >= 3, "masked canopy saturation by species: %s (≥ 0.30); below: %s" % [", ".join(sats), low])
	var moved: PackedStringArray = PackedStringArray()
	var ok: bool = true
	for sp: String in flora.species_ids():
		if not autumn.has("%s/summer" % sp) or not autumn.has("%s/autumn" % sp):
			continue
		var a: Vector3 = Tone.rgb_to_lab(autumn["%s/autumn" % sp]) - Tone.rgb_to_lab(autumn["%s/summer" % sp])
		var want: Array = (flora.resolve(sp, "large", "autumn", 0)["tint"] as Dictionary)["delta_lab"]
		moved.append("%s Δb* %.1f (record %.0f)" % [sp, a.z, float(want[2])])
		# The rendered mean must move with the record: b* up by at least half the record, or stay within 2 when 0.
		if float(want[2]) > 0.0:
			ok = ok and a.z >= float(want[2]) * 0.5
		else:
			ok = ok and absf(a.z) <= 2.0
	_check("autumn_follows_records", ok and moved.size() >= 3, "summer → autumn canopy mean: %s" % ", ".join(moved))


func _check_household_rule() -> void:
	var agent_layer: GDScript = load("res://scripts/world/colony/agent_layer.gd")
	# 1 & 2 a couple; 3 their child; 4 a child whose mother (9) is dead and father 5 lives; 5 a widower (partner 9
	# dead); 6 single; 7 a child of 6 and an unknown father.
	var ids: PackedInt64Array = PackedInt64Array([1, 2, 3, 4, 5, 6, 7])
	var stages: PackedByteArray = PackedByteArray([0, 0, 1, 1, 0, 2, 1])
	var mothers: PackedInt64Array = PackedInt64Array([-1, -1, 2, 9, -1, -1, 6])
	var fathers: PackedInt64Array = PackedInt64Array([-1, -1, 1, 5, -1, -1, -1])
	var partners: PackedInt64Array = PackedInt64Array([2, 1, -1, -1, 9, -1, -1])
	var keys: Dictionary = agent_layer.call("household_keys", ids, stages, mothers, fathers, partners)
	var want: Dictionary = {1: 1, 2: 1, 3: 1, 4: 5, 5: 5, 6: 6, 7: 6}
	var bad: Array = []
	for id: int in want.keys():
		if int(keys.get(id, -99)) != int(want[id]):
			bad.append("%d → %s (want %d)" % [id, keys.get(id, "none"), want[id]])
	_check("household_rule", bad.is_empty(), "7 people, hand-worked keys; differences %s" % [bad])


func _check_households(world: Node) -> void:
	var camera: Camera2D = _main.get_node("Camera2D") as Camera2D
	var agents: Node = _main.get_node("ColonyView/AgentLayer")
	camera.zoom = Vector2(0.25, 0.25)
	camera.force_update_scroll()
	await _frames(10)
	# A second world after the colony one: households must follow the reset.
	world.call("reset_world_stress", 0, 1000)
	world.call("advance_ticks", 1)
	await _frames(20)
	var markers: Node = agents.get_node("LodMarkers")
	var snap: Dictionary = SimReads.read(world, "get_agent_snapshot")
	var partners: PackedInt64Array = snap.get("partners", PackedInt64Array())
	var paired: int = 0
	for p: int in partners:
		paired += 1 if p >= 0 else 0
	# The markers' instance order is the agent layer's person order.
	var persons: Array = (agents.get("_persons") as Dictionary).values()
	var by_household: Dictionary = {}
	var colours: Dictionary = {}
	var shared: int = 0
	var bad: int = 0
	for i: int in persons.size():
		var key: int = int(agents.call("household_of", int((persons[i] as Object).get("agent_id"))))
		# The dummy renderer returns one constant for every MultiMesh custom-data read (not observable headless):
		# headless reads the colour the layer hands the markers, a window reads the markers' instance data.
		var ring: Color = agents.call("_ring_colour", key, ArtData.param("lod.household_ring_rgb", [])) if DisplayServer.get_name() == "headless" else markers.call("ring_of", i)
		if key < 0 or ring.a < 0.5:
			bad += 1
			continue
		if by_household.has(key):
			shared += 1
			if not (by_household[key] as Color).is_equal_approx(ring):
				bad += 1
		by_household[key] = ring
		colours[ring.to_html()] = true
	var source: String = "the layer's colours (headless: marker readback not observable)" if DisplayServer.get_name() == "headless" else "the marker instance data"
	_check("household_rings", persons.size() >= 1000 and bad == 0 and colours.size() >= 2 and (paired == 0 or shared > 0), "stress world: %d people, %d with a partner, %d households, %d people sharing one, %d ring colours; %d without a consistent ring in %s" % [persons.size(), paired, by_household.size(), shared, colours.size(), bad, source])
	# Back to the seed-0 colony for the map check.
	world.call("reset_world", 0)
	world.call("advance_ticks", 1)
	camera.zoom = Vector2.ONE
	camera.force_update_scroll()
	await _frames(20)


func _check_canopy_edges() -> void:
	var b: GroundBaker = GroundBaker.new()
	var g: Dictionary = (ArtData.param("ground", {}) as Dictionary).duplicate()
	g["chunk_tiles"] = 8
	b.configure(g, ArtData.param("water", {}), ArtData.param("hill", {}), ArtData.param("trees", {}))
	var land: Image = Image.create(128, 128, false, Image.FORMAT_RGBA8)
	land.fill(Color(0.5, 0.4, 0.3))
	b.set_grounds([land])
	var inner: Image = Image.create(64, 64, false, Image.FORMAT_RGBA8)
	inner.fill(Color(0.1, 0.6, 0.1))
	var edge: Image = Image.create(64, 64, false, Image.FORMAT_RGBA8)
	edge.fill(Color(0.05, 0.3, 0.05))
	b.set_canopies([inner], [edge])
	b.canopy_on = true
	var m: GroundBaker.GroundMap = GroundBaker.GroundMap.new()
	m.setup(8, 8)
	m.ground.fill(0)
	m.tree[3 * 8 + 3] = 1
	var img: Image = b.bake(m, 0, 0)
	var o: Vector2i = Vector2i(3 * TILE, 3 * TILE)
	var near: Callable = func(p: Vector2i, c: Color) -> bool:
		var q: Color = img.get_pixelv(p)
		return absf(q.r - c.r) < 0.02 and absf(q.g - c.g) < 0.02 and absf(q.b - c.b) < 0.02
	var ok: Array[String] = []
	if near.call(o + Vector2i(32, 32), inner.get_pixel(0, 0)):
		ok.append("centre")
	for side: Array in [["N", Vector2i(32, 2)], ["E", Vector2i(61, 32)], ["S", Vector2i(32, 61)], ["W", Vector2i(2, 32)]]:
		if near.call(o + (side[1] as Vector2i), edge.get_pixel(0, 0)):
			ok.append("edge " + str(side[0]))
	for corner: Array in [["NW", Vector2i(1, 1)], ["NE", Vector2i(62, 1)], ["SW", Vector2i(1, 62)], ["SE", Vector2i(62, 62)]]:
		if near.call(o + (corner[1] as Vector2i), land.get_pixel(0, 0)):
			ok.append("ground " + str(corner[0]))
	_check("canopy_edges_all_sides", ok.size() == 9, "isolated tree tile: %s (9 expected)" % [ok])


func _check_season_state() -> void:
	var tree_layer: GDScript = load("res://scripts/world/colony/tree_layer.gd")
	var names: Array[String] = ["spring", "summer", "autumn", "winter"]
	var cases: Array = [
		[{"season": 0, "day_of_season": 1, "day_index": 0, "days_per_season": 6, "time_of_day": 8.0}, "winter|spring", 1.0],
		[{"season": 1, "day_of_season": 1, "day_index": 6, "days_per_season": 6, "time_of_day": 6.0}, "spring|summer", 0.25],
		[{"season": 1, "day_of_season": 2, "day_index": 7, "days_per_season": 6, "time_of_day": 1.0}, "spring|summer", 1.0],
		[{"season": 3, "day_of_season": 1, "day_index": 18, "days_per_season": 6, "time_of_day": 12.0}, "autumn|winter", 0.5],
		[{"season": 0, "day_of_season": 1, "day_index": 24, "days_per_season": 6, "time_of_day": 18.0}, "winter|spring", 0.75],
	]
	var bad: Array = []
	for c: Array in cases:
		var got: Array = tree_layer.call("season_state", c[0], names)
		if got.size() != 2 or str(got[0]) != str(c[1]) or absf(float(got[1]) - float(c[2])) > 1e-6:
			bad.append("%s → %s (want %s %.2f)" % [c[0], got, c[1], c[2]])
	_check("season_state", bad.is_empty(), "%d calendar cases; differences %s" % [cases.size(), bad])


func _check_map_classes(ground: Node, season: String, gated: bool) -> void:
	var camera: Camera2D = _main.get_node("Camera2D") as Camera2D
	var map: GroundBaker.GroundMap = ground.call("ground_map")
	var ids: Array = ground.get("ground_ids")
	var terrain: Dictionary = ArtData.bindings().get("terrain", {})
	var plain: String = str(terrain.get("fallback", ""))
	var clearing: String = str((terrain.get("clearing", {}) as Dictionary).get("ground", ""))
	# A tile's class: 0 forest, 1 meadow, 2 bare earth, 3 rock, 4 water, -1 other.
	var cls: Callable = func(x: int, y: int) -> int:
		var g: int = map.at(x, y)
		if g == GroundBaker.WATER:
			return 4
		if g < 0:
			return -1
		if map.is_rock(x, y):
			return 3
		if map.is_crisp(x, y):
			return -1
		if map.tree_at(x, y) > 0:
			return 0
		var id: String = str(ids[g]) if g < ids.size() else ""
		return 1 if id == plain else (2 if id == clearing else -1)
	var hearth: Vector2i = Vector2i(map.width >> 1, map.height >> 1)
	(_main.get_node("ColonyView/TreeLayer") as Node).call("force_season", season, season, 1.0)
	await _frames(3)
	camera.zoom = Vector2(0.25, 0.25)
	camera.position = View.tile_centre(hearth)
	camera.force_update_scroll()
	await _frames(150)
	await RenderingServer.frame_post_draw
	var shot: Image = root.get_viewport().get_texture().get_image()
	shot.convert(Image.FORMAT_RGBA8)
	var xf: Transform2D = root.get_viewport().get_canvas_transform()
	var sums: Array = [Vector3.ZERO, Vector3.ZERO, Vector3.ZERO, Vector3.ZERO, Vector3.ZERO]
	var counts: Array[int] = [0, 0, 0, 0, 0]
	var tiles: Array[int] = [0, 0, 0, 0, 0]
	for y: int in range(1, map.height - 1):
		for x: int in range(1, map.width - 1):
			var k: int = cls.call(x, y)
			if k < 0:
				continue
			# Rivers are one or two tiles wide: a water tile's central half is water whatever its neighbours.
			var interior: bool = true
			if k != 4:
				for d: Vector2i in [Vector2i(-1, -1), Vector2i(0, -1), Vector2i(1, -1), Vector2i(-1, 0), Vector2i(1, 0), Vector2i(-1, 1), Vector2i(0, 1), Vector2i(1, 1)]:
					interior = interior and int(cls.call(x + d.x, y + d.y)) == k
			if not interior:
				continue
			var a: Vector2 = xf * Vector2(x * TILE + 16, y * TILE + 16)
			var b: Vector2 = xf * Vector2(x * TILE + 48, y * TILE + 48)
			if a.x < 0 or a.y < 80 or b.x >= shot.get_width() or b.y >= shot.get_height() - 140:
				continue
			tiles[k] += 1
			for py: int in range(floori(a.y), floori(b.y)):
				for px: int in range(floori(a.x), floori(b.x)):
					var c: Color = shot.get_pixel(px, py)
					sums[k] += Vector3(c.r, c.g, c.b)
					counts[k] += 1
	var names: Array[String] = ["forest", "meadow", "earth", "rock", "water"]
	var labs: Array[Vector3] = []
	var means: PackedStringArray = PackedStringArray()
	for k: int in 5:
		var m: Vector3 = (sums[k] as Vector3) / maxf(1.0, counts[k])
		labs.append(Tone.rgb_to_lab(m))
		means.append("%s %d tiles rgb(%.2f,%.2f,%.2f)" % [names[k], tiles[k], m.x, m.y, m.z])
	var worst: float = INF
	var pair: String = ""
	var all: PackedStringArray = PackedStringArray()
	for i: int in 5:
		for j: int in range(i + 1, 5):
			var de: float = labs[i].distance_to(labs[j])
			all.append("%s-%s %.1f" % [names[i], names[j], de])
			if de < worst:
				worst = de
				pair = "%s/%s" % [names[i], names[j]]
	var enough: bool = tiles.min() >= 20
	var line: String = "%s: %s; pairwise ΔE %s; min %.1f (%s), bar 10" % [season, ", ".join(means), ", ".join(all), worst, pair]
	if gated:
		_check("quarter_map_classes", enough and worst >= 10.0, line)
	else:
		print("MART21_TREES INFO quarter_map_classes_%s — %s (ground tone stays the day tone here: forced seasons move the trees only)" % [season, line])
	shot.save_png(OS.get_user_data_dir().path_join("mart21_quarter_map_%s.png" % season))
	camera.zoom = Vector2.ONE
	camera.force_update_scroll()
