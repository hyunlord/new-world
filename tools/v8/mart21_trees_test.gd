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
	if OS.get_cmdline_user_args().has("--rendered"):
		if DisplayServer.get_name() == "headless":
			_check("rendered_backend", false, "--rendered needs a window")
		else:
			await _check_rendered(trees, flora)
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
