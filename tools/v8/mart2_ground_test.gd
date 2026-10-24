# mart2_ground_test.gd — CPU and optional windowed pixel tests of the M-ART2 ground and trees (goal §8, D19).
# Purpose: boots scenes/main.tscn on the colony world seed 0 (reset_world(0), the clock held and stepped to tick 1 so
#   the hearth exists) and checks, on the real GroundLayer:
#   1. chunk bake determinism: the hearth's chunk baked twice (the layer, then a fresh GroundBaker on the same map and
#      textures) is byte-identical, and its SHA-256 equals tools/v8/mart2_chunk_hashes.json (`-- --write` records it
#      after an intended change);
#   2. no square edges: every boundary between two different non-crisp land grounds in the chunk, and every straight
#      run of such boundaries, has no run of more than 24 identical non-zero luminance steps (ART-STRUCT's
#      instrument, edges.py `boundary_steps`);
#   3. the ground tone caps: every prepared ground texture passed tone.gd `cap_ground`, and the chunk's land mean
#      luminance stays under the cap;
#   4. banks: a river tile next to land has opaque bank pixels within 10 px of the land side and is transparent
#      beyond them; the land side carries ink;
#   5. a dig rebakes exactly one chunk: an undug hill tile whose 3 × 3 neighbourhood lies inside one cached chunk is
#      dug through GroundLayer.apply_space (the same path as a real get_space change);
#   6. the data-only wall: tools/v8/fixtures/data_only_wall.json adds a material and a blueprint by data; the unchanged
#      resolver resolves them to the expected style and the unchanged wall baker draws a 3 × 3 room whose strips have
#      the expected opaque coverage.
#   7. rounded shores (M-ART2.1 §5): a synthetic L-shaped river — the field's contour at the outer (convex) and inner
#      (concave) corner is a quarter circle of `water.shore_radius_px` (±3 px; rays from the expected centre), a
#      straight shore's contour stays on the tile edge (±1 px); in the baked image the visible edge (5 × 5 land
#      fraction 0.5) keeps the same offset from the contour at the corner arc as along the straight shore (±3 px:
#      the bank follows the contour; a square corner would sit ~7 px further out); hill tops round against open
#      ground (`hill.edge_radius_px`, crisp) except at a corner whose upper row hangs a face;
#   8. water: the uploaded water texture is the 05 texture itself (luminance std = the source's, ≥ 0.06), the lift
#      clips < 1 % of its pixels; on every seed-0 chunk with water, the baked land beside water against the lifted
#      water mean differs by ≥ 0.10 luminance at full night (day difference × the night multiplier's weakest
#      channel) and ≥ 0.12 turns of hue, by day, in winter and in drought (min values printed); the ford stones lie on
#      water pixels of the rounded shore; no run of more than 24 identical luminance steps along any water edge of
#      the river chunks (the STRUCT instrument on the chunk composited over the lifted water).
#   Prints one MART2_GROUND line per check and MART2_GROUND_DONE failures=N; exits 1 on any failure.
#   `godot --headless --path . --script res://tools/v8/mart2_ground_test.gd [-- --write]`
#   Add `-- --rendered` without --headless for GPU readbacks at 2×, 1×, ½× and ¼×: source saturation must survive
#   TreeLayer._standing; water must differ from every sampled land class by luminance ≥0.10 and circular HSV hue
#   ≥0.12 turns (43.2°), by day and at full night. `--cap-tree-control` deliberately caps test trees and must FAIL.
#   Below 1× the source reference is sampled at the identical nearest-filter phase: minification alone changes a
#   small crown's full-image mean. At 1× native fully opaque pixels are also checked; faint edge colours lose RGB
#   precision when an RGBA8 transparent viewport premultiplies alpha, and remain covered by the source render pair.
# Bridge methods called: reset_world, advance_ticks (direct); the layers read the rest.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Tone = preload("res://scripts/world/art/tone.gd")
const GroundBaker = preload("res://scripts/world/art/ground_baker.gd")
const WallBaker = preload("res://scripts/world/art/wall_baker.gd")
const MaterialResolver = preload("res://scripts/world/art/material_resolver.gd")
const Daylight = preload("res://scripts/world/daylight_controller.gd")
const HASHES: String = "res://tools/v8/mart2_chunk_hashes.json"
const TILE: int = View.TILE

var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(name: String, ok: bool, detail: String) -> void:
	print("MART2_GROUND %s %s — %s" % ["PASS" if ok else "FAIL", name, detail])
	if not ok:
		_failures += 1


func _sha(img: Image) -> String:
	var ctx: HashingContext = HashingContext.new()
	ctx.start(HashingContext.HASH_SHA256)
	ctx.update(img.get_data())
	return ctx.finish().hex_encode()


func _run() -> void:
	var write: bool = OS.get_cmdline_user_args().has("--write")
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	root.add_child(main)
	var world: Node = main.get_node("WorldSim")
	await process_frame
	world.call("reset_world", 0)
	world.process_mode = Node.PROCESS_MODE_DISABLED
	world.call("advance_ticks", 1)
	for _i: int in 20:
		await process_frame
	var layer: Node = main.get_node("ColonyView/GroundLayer")
	layer.set("keep_images", true)
	var map: GroundBaker.GroundMap = layer.call("ground_map")
	if map == null:
		_check("map", false, "the ground layer built no map")
		_finish()
		return
	var hearth: Vector2i = _hearth(world)
	var size: int = int(layer.call("baker").chunk_tiles)
	var c: Vector2i = Vector2i(floori(float(hearth.x) / size), floori(float(hearth.y) / size))
	layer.call("bake_chunk_now", c)
	var img: Image = layer.call("chunk_image", c)
	_check_determinism(layer, map, c, img, write)
	_check_edges(map, c, img, size)
	_check_tone(layer, map, c, img, size)
	_check_banks(layer, map, size)
	_check_dig(layer, world, map, size)
	_check_data_only()
	_check_shores_synthetic()
	_check_hills_synthetic()
	_check_water(layer, world, map, size)
	if OS.get_cmdline_user_args().has("--rendered"):
		if DisplayServer.get_name() == "headless":
			_check("rendered_backend", false, "--rendered requires a window and GPU; the headless renderer supplies no pixels")
		else:
			main.process_mode = Node.PROCESS_MODE_DISABLED
			await _check_rendered_trees(main)
			await _check_rendered_terrain(main, layer, map, size)
	else:
		print("MART2_GROUND SKIP rendered_pixels — run windowed with -- --rendered; headless checks do not prove tree saturation or water contrast")
	_finish()


func _finish() -> void:
	print("MART2_GROUND_DONE failures=%d" % _failures)
	quit(1 if _failures > 0 else 0)


## Render the supplied production sprite through its material in an isolated, transparent canvas. Canvas zoom
## preserves MODEL_MATRIX/world sampling; the crop selects world pixels without moving the production sprite.
func _read_pixels(sprite: Sprite2D, crop: Rect2, zoom: float, light: Color = Color.WHITE) -> Image:
	var viewport: SubViewport = SubViewport.new()
	viewport.size = Vector2i(ceili(crop.size.x * zoom), ceili(crop.size.y * zoom))
	viewport.world_2d = World2D.new()
	viewport.transparent_bg = true
	viewport.disable_3d = true
	viewport.canvas_item_default_texture_filter = Viewport.DEFAULT_CANVAS_ITEM_TEXTURE_FILTER_NEAREST
	viewport.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	root.add_child(viewport)
	viewport.canvas_transform = Transform2D(Vector2(zoom, 0.0), Vector2(0.0, zoom), -crop.position * zoom)
	var tint: CanvasModulate = CanvasModulate.new()
	tint.color = light
	viewport.add_child(tint)
	var copy: Sprite2D = sprite.duplicate() as Sprite2D
	copy.texture_filter = sprite.get_texture_filter()
	viewport.add_child(copy)
	await process_frame
	await RenderingServer.frame_post_draw
	var img: Image = viewport.get_texture().get_image()
	img.convert(Image.FORMAT_RGBA8)
	viewport.queue_free()
	return img


func _opaque_metrics(img: Image) -> Dictionary:
	var opaque: Image = img.duplicate()
	var data: PackedByteArray = opaque.get_data()
	for i: int in range(3, data.size(), 4):
		if data[i] < 255:
			data[i] = 0
	opaque.set_data(opaque.get_width(), opaque.get_height(), false, Image.FORMAT_RGBA8, data)
	return Tone.metrics(opaque)


func _check_rendered_trees(main: Node) -> void:
	var layer: Node = main.get_node("ColonyView/TreeLayer")
	var ids: Array[String] = []
	for row: Dictionary in (ArtData.bindings().get("trees", {}) as Dictionary).get("species", []):
		for key: String in ["large", "small"]:
			var id: String = str(row.get(key, ""))
			if id != "" and not ids.has(id):
				ids.append(id)
	_check("tree_source_count", ids.size() >= 6, "%d independently loaded standing-tree sources" % ids.size())
	for id: String in ids:
		# Bypass ArtData's mutable caches: the reference must still expose an accidental cached-image transform.
		var reference: Texture2D = ResourceLoader.load(ArtData.path(id), "Texture2D", ResourceLoader.CACHE_MODE_IGNORE) as Texture2D
		var sprite: Sprite2D = layer.call("_standing", id, Vector2(ArtData.anchor(id)))
		if reference == null or sprite == null:
			_check("tree_source_" + id, false, "missing imported source or production standing sprite")
			continue
		var source: Image = reference.get_image()
		if source.is_compressed():
			source.decompress()
		source.convert(Image.FORMAT_RGBA8)
		var native: Dictionary = Tone.metrics(source)
		var native_opaque: Dictionary = _opaque_metrics(source)
		if OS.get_cmdline_user_args().has("--cap-tree-control"):
			var cap: Dictionary = Tone.cap_ground(source, 0.17, 0.38, 0.25, 0.30)
			sprite.texture = ImageTexture.create_from_image(cap["image"])
		var control: Sprite2D = Sprite2D.new()
		control.texture = reference
		control.centered = false
		control.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		var crop: Rect2 = Rect2(Vector2.ZERO, reference.get_size())
		for zoom: float in [2.0, 1.0, 0.5, 0.25]:
			var actual_image: Image = await _read_pixels(sprite, crop, zoom)
			var actual: Dictionary = Tone.metrics(actual_image)
			var expected: Dictionary = Tone.metrics(await _read_pixels(control, crop, zoom))
			var delta: float = absf(float(actual["sat"]) - float(expected["sat"]))
			var ok: bool = int(actual["n"]) > 0 and int(actual["n"]) == int(expected["n"]) and delta <= 0.02
			_check("tree_saturation_%s_%.2fx" % [id, zoom], ok, "rendered %.5f, phase/alpha-matched source %.5f, raw native %.5f; delta %.5f ≤0.02 (%d pixels)" % [actual["sat"], expected["sat"], native["sat"], delta, actual["n"]])
			if zoom == 1.0:
				var actual_opaque: Dictionary = _opaque_metrics(actual_image)
				var native_delta: float = absf(float(actual_opaque["sat"]) - float(native_opaque["sat"]))
				_check("tree_native_opaque_" + id, int(actual_opaque["n"]) > 0 and int(native_opaque["n"]) > 0 and native_delta <= 0.02, "native opaque %.5f vs rendered opaque %.5f; delta %.5f ≤0.02; all alpha edges checked in the paired source render" % [native_opaque["sat"], actual_opaque["sat"], native_delta])
		sprite.queue_free()
		control.free()


## One real tile of every ground class and water, preferring a same-class neighbourhood so edge dither is excluded.
func _terrain_samples(map: GroundBaker.GroundMap) -> Dictionary:
	var samples: Dictionary = {}
	var interior: Dictionary = {}
	for y: int in range(1, map.height - 1):
		for x: int in range(1, map.width - 1):
			var g: int = map.at(x, y)
			if g < 0 and g != GroundBaker.WATER:
				continue
			if not samples.has(g):
				samples[g] = Vector2i(x, y)
			if not interior.has(g) and map.at(x - 1, y) == g and map.at(x + 1, y) == g and map.at(x, y - 1) == g and map.at(x, y + 1) == g:
				samples[g] = Vector2i(x, y)
				interior[g] = true
	return samples


func _mean_colour(img: Image) -> Color:
	var sum: Vector3 = Vector3.ZERO
	var count: int = 0
	for y: int in img.get_height():
		for x: int in img.get_width():
			var c: Color = img.get_pixel(x, y)
			if c.a > 0.0:
				sum += Vector3(c.r, c.g, c.b)
				count += 1
	if count == 0:
		return Color(0.0, 0.0, 0.0, 0.0)
	sum /= count
	return Color(sum.x, sum.y, sum.z, 1.0)


func _hue_distance(a: Color, b: Color) -> float:
	var distance: float = absf(a.h - b.h)
	return minf(distance, 1.0 - distance)


func _check_rendered_terrain(main: Node, layer: Node, map: GroundBaker.GroundMap, size: int) -> void:
	var samples: Dictionary = _terrain_samples(map)
	var minimum_luminance: float = maxf(0.10, float(ArtData.param("water.minimum_land_luminance_difference", 0.10)))
	var minimum_hue: float = maxf(0.12, float(ArtData.param("water.minimum_land_hue_distance", 0.12)))
	_check("terrain_sample_classes", samples.has(GroundBaker.WATER) and samples.size() >= 5, "%d real classes including water" % samples.size())
	if not samples.has(GroundBaker.WATER):
		return
	var camera: Camera2D = main.get_node("Camera2D") as Camera2D
	_check_zoom_tint_atomic(camera, layer)
	var ids: Array = layer.get("ground_ids")
	var keys: Array = samples.keys()
	keys.sort()
	for zoom: float in [2.0, 1.0, 0.5, 0.25]:
		camera.zoom = Vector2(zoom, zoom)
		camera.force_update_scroll()
		layer.call("_ensure_visible")
		var day: Dictionary = {}
		var night: Dictionary = {}
		for g: int in keys:
			var tile: Vector2i = samples[g]
			var chunk: Vector2i = Vector2i(floori(float(tile.x) / size), floori(float(tile.y) / size))
			layer.call("bake_chunk_now", chunk)
			var sprite: Sprite2D = (layer.get("_chunks") as Dictionary).get(chunk, null)
			if sprite == null:
				_check("terrain_sprite", false, "no real chunk for %s" % tile)
				continue
			# The central 32×32 world pixels exclude the 12px dither band and 10px riverbank at every zoom.
			var crop: Rect2 = Rect2(Vector2(tile * TILE + Vector2i(16, 16)), Vector2(32, 32))
			var day_image: Image = await _read_pixels(sprite, crop, zoom)
			day[g] = _mean_colour(day_image)
			night[g] = _mean_colour(await _read_pixels(sprite, crop, zoom, Daylight.night_colour()))
			var factor: int = 1 if zoom >= 1.0 else (2 if zoom >= 0.5 else 4)
			var expected_width: int = floori(float(size * TILE) / factor)
			_check("reduced_texture_%d_%.2fx" % [g, zoom], sprite.texture.get_width() == expected_width, "texture %dpx, expected %dpx" % [sprite.texture.get_width(), expected_width])
			if g >= 0:
				var metrics: Dictionary = Tone.metrics(day_image)
				_check("rendered_land_caps_%s_%.2fx" % [ids[g], zoom], int(metrics["n"]) > 0 and float(metrics["mean"]) <= 0.38 and float(metrics["sat"]) <= 0.30, "rendered tile interior Y %.5f ≤0.38, mean saturation %.5f ≤0.30" % [metrics["mean"], metrics["sat"]])
			if g >= 0 and zoom < 1.0:
				await _check_map_tint(sprite, crop, zoom, str(ids[g]), day[g], not map.is_crisp(tile.x, tile.y))
		for light_name: String in ["day", "night"]:
			var colours: Dictionary = day if light_name == "day" else night
			if not colours.has(GroundBaker.WATER):
				_check("water_pixels", false, "missing water readback")
				continue
			var water: Color = colours[GroundBaker.WATER]
			for g: int in keys:
				if g < 0 or not colours.has(g):
					continue
				var land: Color = colours[g]
				var luminance_delta: float = absf(water.get_luminance() - land.get_luminance())
				var hue_delta: float = _hue_distance(water, land)
				_check("water_contrast_%s_%s_%.2fx" % [ids[g], light_name, zoom], water.a > 0.0 and land.a > 0.0 and water.s > 0.10 and luminance_delta >= minimum_luminance and hue_delta >= minimum_hue, "water Y=%.4f H=%.4f; land Y=%.4f H=%.4f; deltaY %.4f ≥%.3f, deltaH %.4f ≥%.3f turns" % [water.get_luminance(), water.h, land.get_luminance(), land.h, luminance_delta, minimum_luminance, hue_delta, minimum_hue])


## Changing zoom must update palette weights on every cached chunk immediately, even while its texture awaits
## the four-per-frame retexture queue; otherwise half/quarter-strength rectangles persist during zoom transitions.
func _check_zoom_tint_atomic(camera: Camera2D, layer: Node) -> void:
	var strengths: Dictionary = ArtData.param("ground.map_mode.tint_strength", {})
	if not strengths.has("0.25"):
		return
	camera.zoom = Vector2(0.5, 0.5)
	camera.force_update_scroll()
	layer.call("_ensure_visible")
	var cached: Dictionary = (layer.get("_chunks") as Dictionary).duplicate()
	camera.zoom = Vector2(0.25, 0.25)
	camera.force_update_scroll()
	layer.call("_ensure_visible")
	var expected: float = float(strengths["0.25"])
	var pending: int = 0
	var wrong: int = 0
	var resolutions: Dictionary = layer.get("_res_of")
	for c: Vector2i in cached.keys():
		var sprite: Sprite2D = cached[c]
		var material: ShaderMaterial = sprite.material as ShaderMaterial
		if material == null or not is_equal_approx(float(material.get_shader_parameter("map_tint_strength")), expected):
			wrong += 1
		if int(resolutions.get(c, 1)) != 4:
			pending += 1
	_check("map_tint_atomic_zoom", cached.size() > 4 and pending > 0 and wrong == 0, "%d cached chunks, %d awaiting quarter textures; %d stale tint strengths (expected %.2f everywhere)" % [cached.size(), pending, wrong, expected])


## Counterfactual readback: disabling only this chunk's map tint must move a configured class away from its colour.
func _check_map_tint(sprite: Sprite2D, crop: Rect2, zoom: float, id: String, actual: Color, allowed: bool) -> void:
	var palette: Dictionary = ArtData.param("ground.map_mode.tint_by_ground", {})
	if not palette.has(id):
		return
	var rgb: Array = palette[id]
	var target: Vector3 = Vector3(float(rgb[0]), float(rgb[1]), float(rgb[2]))
	var control: Sprite2D = sprite.duplicate() as Sprite2D
	var material: ShaderMaterial = sprite.material.duplicate() as ShaderMaterial
	material.set_shader_parameter("map_tint_strength", 0.0)
	control.material = material
	var without: Color = _mean_colour(await _read_pixels(control, crop, zoom))
	control.free()
	var before: float = Vector3(without.r, without.g, without.b).distance_to(target)
	var after: float = Vector3(actual.r, actual.g, actual.b).distance_to(target)
	if allowed:
		_check("map_tint_%s_%.2fx" % [id, zoom], actual.a > 0.0 and without.a > 0.0 and before - after > 0.005, "RGB distance to declared class colour %.4f → %.4f with map tint" % [before, after])
	else:
		_check("map_tint_excludes_crisp_%s_%.2fx" % [id, zoom], actual.a > 0.0 and actual.is_equal_approx(without), "crisp ground unchanged with map tint on/off: %s / %s" % [actual, without])


func _hearth(world: Node) -> Vector2i:
	var b: Dictionary = SimReads.read(world, "get_building_snapshot")
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	for i: int in kinds.size():
		if bool(ArtData.binding("buildings", kinds[i]).get("light", false)):
			return Vector2i(b["xs"][i], b["ys"][i])
	return Vector2i(96, 96)


func _check_determinism(layer: Node, map: GroundBaker.GroundMap, c: Vector2i, img: Image, write: bool) -> void:
	var fresh: GroundBaker = GroundBaker.new()
	var used: GroundBaker = layer.call("baker")
	fresh.configure(ArtData.param("ground", {}), ArtData.param("water", {}), ArtData.param("hill", {}))
	var images: Array[Image] = []
	for k: int in used.ground_count():
		var t: Image = used.tiled(k)
		# The fresh baker re-tiles from one period of the same prepared texture.
		var period: Image = t.get_region(Rect2i(0, 0, used.period(k), used.period(k)))
		images.append(period)
	fresh.set_grounds(images)
	var again: Image = fresh.bake(map, c.x, c.y)
	var a: String = _sha(img)
	var b: String = _sha(again)
	_check("bake_twice", a == b, "chunk %s sha %s vs fresh %s" % [c, a.substr(0, 12), b.substr(0, 12)])
	var key: String = "seed0_tick1_chunk_%d_%d" % [c.x, c.y]
	if write:
		var out: FileAccess = FileAccess.open(HASHES, FileAccess.WRITE)
		out.store_string(JSON.stringify({key: a, "about": "SHA-256 of the hearth chunk's baked RGBA8 bytes (mart2_ground_test.gd); rewrite with -- --write after an intended change"}, "  ") + "\n")
		out.close()
		print("MART2_GROUND wrote %s = %s" % [key, a])
		return
	var expected: Variant = JSON.parse_string(FileAccess.get_file_as_string(HASHES))
	var want: String = str((expected as Dictionary).get(key, "")) if expected is Dictionary else ""
	_check("bake_stable_across_runs", want == a, "%s expected %s got %s" % [key, want.substr(0, 12), a.substr(0, 12)])


## edges.py `boundary_steps`: luminance steps across a boundary, rounded to 1/1000; the longest run of one non-zero value.
func _longest_step_run(img: Image, start: Vector2i, length: int, vertical: bool) -> int:
	var longest: int = 0
	var run: int = 0
	var previous: float = INF
	for k: int in length:
		var p: Vector2i = Vector2i(start.x, start.y + k) if vertical else Vector2i(start.x + k, start.y)
		var q: Vector2i = p - (Vector2i(1, 0) if vertical else Vector2i(0, 1))
		var a: Color = img.get_pixelv(p)
		var b: Color = img.get_pixelv(q)
		var la: float = 255.0 * (0.2126 * a.r + 0.7152 * a.g + 0.0722 * a.b)
		var lb: float = 255.0 * (0.2126 * b.r + 0.7152 * b.g + 0.0722 * b.b)
		var step: float = snappedf(la - lb, 0.001)
		if step != 0.0 and step == previous:
			run += 1
		else:
			run = 1 if step != 0.0 else 0
		previous = step
		longest = maxi(longest, run)
	return longest


func _check_edges(map: GroundBaker.GroundMap, c: Vector2i, img: Image, size: int) -> void:
	var x0: int = c.x * size
	var y0: int = c.y * size
	var worst: int = 0
	var boundaries: int = 0
	# Straight runs: per boundary line, consecutive differing pairs are joined.
	for vertical: bool in [true, false]:
		for line: int in range(1, size):
			var run_start: int = -1
			for along: int in range(0, size + 1):
				var ok: bool = false
				if along < size:
					var a: Vector2i = Vector2i(x0 + line - 1, y0 + along) if vertical else Vector2i(x0 + along, y0 + line - 1)
					var b: Vector2i = a + (Vector2i(1, 0) if vertical else Vector2i(0, 1))
					var ga: int = map.at(a.x, a.y)
					var gb: int = map.at(b.x, b.y)
					ok = ga >= 0 and gb >= 0 and ga != gb and not map.is_crisp(a.x, a.y) and not map.is_crisp(b.x, b.y)
				if ok and run_start < 0:
					run_start = along
				elif not ok and run_start >= 0:
					boundaries += 1
					var start: Vector2i = Vector2i(line * TILE, run_start * TILE) if vertical else Vector2i(run_start * TILE, line * TILE)
					worst = maxi(worst, _longest_step_run(img, start, (along - run_start) * TILE, vertical))
					run_start = -1
	_check("no_square_edges", boundaries > 0 and worst <= 24, "%d boundary runs between grounds, longest identical step run %d px (bar 24)" % [boundaries, worst])
	_check_dither(map, c, img, size)


## The step-run instrument passes square edges between two textured grounds (steps vary pixel by pixel), so this
## check can fail: next to a boundary with another ground, the tile's 12-px band must hold pixels that differ from its
## own texture (taken from the neighbour), and its far side none.
func _check_dither(map: GroundBaker.GroundMap, c: Vector2i, img: Image, size: int) -> void:
	var baker: GroundBaker = (root.get_node("Main/ColonyView/GroundLayer") as Node).call("baker")
	var tested: int = 0
	var band_mixed: float = 0.0
	var far_mixed: float = 0.0
	for ty: int in range(c.y * size, c.y * size + size):
		for tx: int in range(c.x * size + 1, c.x * size + size - 1):
			var g: int = map.at(tx, ty)
			var e: int = map.at(tx + 1, ty)
			if g < 0 or e < 0 or g == e or map.is_crisp(tx, ty) or map.is_crisp(tx + 1, ty):
				continue
			if map.at(tx - 1, ty) != g or map.at(tx, ty - 1) != g or map.at(tx, ty + 1) != g:
				continue
			var src: Image = baker.tiled(g)
			var p: int = baker.period(g)
			var ox: int = (tx - c.x * size) * TILE
			var oy: int = (ty - c.y * size) * TILE
			var diff_band: int = 0
			var diff_far: int = 0
			for py: int in range(8, 56):
				for px: int in range(TILE - 12, TILE):
					if img.get_pixel(ox + px, oy + py) != src.get_pixel(posmod(tx * TILE + px, p), posmod(ty * TILE + py, p)):
						diff_band += 1
				for px: int in range(16, 40):
					if img.get_pixel(ox + px, oy + py) != src.get_pixel(posmod(tx * TILE + px, p), posmod(ty * TILE + py, p)):
						diff_far += 1
			band_mixed += float(diff_band) / (48.0 * 12.0)
			far_mixed += float(diff_far) / (48.0 * 24.0)
			tested += 1
	if tested == 0:
		_check("edges_dithered", false, "no east boundary between two grounds in chunk %s" % c)
		return
	band_mixed /= tested
	far_mixed /= tested
	_check("edges_dithered", band_mixed > 0.08 and band_mixed < 0.6 and far_mixed == 0.0, "%d tiles with another ground east: %.2f of the 12-px band taken from it, %.2f of the far side" % [tested, band_mixed, far_mixed])


func _check_tone(layer: Node, _map: GroundBaker.GroundMap, c: Vector2i, img: Image, size: int) -> void:
	var report: Array = layer.call("tone_report")
	var failed: Array[String] = []
	for r: Variant in report:
		if not bool((r as Dictionary).get("passed", false)):
			failed.append(str((r as Dictionary).get("id", "")))
	_check("tone_caps_per_texture", report.size() > 0 and failed.is_empty(), "%d textures, failed %s" % [report.size(), failed])
	var land: Image = Image.create(size * TILE, size * TILE, false, Image.FORMAT_RGBA8)
	land.blit_rect(img, Rect2i(0, 0, size * TILE, size * TILE), Vector2i.ZERO)
	var m: Dictionary = Tone.metrics(land, 7)
	var cap: float = float(ArtData.param("ground.tone.mean_max", 0.32))
	_check("tone_chunk_mean", m["n"] > 0 and m["mean"] <= cap, "chunk %s land mean luminance %.3f (cap %.2f), std %.3f, saturation %.3f" % [c, m["mean"], cap, m["std"], m["sat"]])


func _check_banks(layer: Node, map: GroundBaker.GroundMap, size: int) -> void:
	# The first river tile with land to its west, its chunk baked.
	for y: int in map.height:
		for x: int in range(1, map.width):
			if posmod(x, size) == 0 or map.at(x, y) != GroundBaker.WATER or map.at(x - 1, y) < 0 or map.is_crisp(x - 1, y) or map.at(x + 1, y) != GroundBaker.WATER:
				continue
			var c: Vector2i = Vector2i(floori(float(x) / size), floori(float(y) / size))
			layer.call("bake_chunk_now", c)
			var img: Image = layer.call("chunk_image", c)
			var ox: int = (x - c.x * size) * TILE
			var oy: int = (y - c.y * size) * TILE
			var near: int = 0
			var far: int = 0
			for py: int in range(oy + 12, oy + 52):
				for px: int in range(ox, ox + 10):
					if img.get_pixel(px, py).a > 0.5:
						near += 1
				for px: int in range(ox + 20, ox + 44):
					if img.get_pixel(px, py).a > 0.5:
						far += 1
			# The ink stroke (2 px, displaced 0–4 px) darkens the land tile's last 6 columns against its interior.
			var edge: float = 0.0
			var inner: float = 0.0
			for py: int in range(oy + 12, oy + 52):
				for px: int in range(ox - 6, ox):
					edge += img.get_pixel(px, py).get_luminance()
				for px: int in range(ox - 30, ox - 18):
					inner += img.get_pixel(px, py).get_luminance()
			edge /= 40.0 * 6.0
			inner /= 40.0 * 12.0
			_check("water_bank", near > 40 and near < 360 and far == 0 and edge < inner * 0.95, "river tile %s: %d opaque bank px in the 10-px band, %d beyond it; land edge band luminance %.3f vs interior %.3f (ink)" % [Vector2i(x, y), near, far, edge, inner])
			return
	_check("water_bank", false, "no river tile with land to its west")


func _check_dig(layer: Node, world: Node, map: GroundBaker.GroundMap, size: int) -> void:
	var space: Dictionary = (SimReads.read(world, "get_space") as Dictionary).duplicate(true)
	for y: int in range(1, map.height - 1):
		for x: int in range(1, map.width - 1):
			if not map.is_crisp(x, y) or posmod(x, size) < 2 or posmod(x, size) > size - 3 or posmod(y, size) < 2 or posmod(y, size) > size - 3:
				continue
			var c: Vector2i = Vector2i(floori(float(x) / size), floori(float(y) / size))
			layer.call("bake_chunk_now", c)
			for d: Vector2i in [Vector2i(-1, 0), Vector2i(1, 0), Vector2i(0, -1), Vector2i(0, 1)]:
				layer.call("bake_chunk_now", c + d)
			var before: Dictionary = (layer.get("bake_counts") as Dictionary).duplicate()
			var dxs: PackedInt32Array = space.get("dug_xs", PackedInt32Array())
			var dys: PackedInt32Array = space.get("dug_ys", PackedInt32Array())
			dxs.append(x)
			dys.append(y)
			space["dug_xs"] = dxs
			space["dug_ys"] = dys
			space["version"] = int(space.get("version", 0)) + 1000
			var t0: int = Time.get_ticks_usec()
			layer.call("apply_space", space)
			var usec: int = Time.get_ticks_usec() - t0
			var after: Dictionary = layer.get("bake_counts")
			var rebaked: Array[Vector2i] = []
			for k: Vector2i in after.keys():
				if int(after[k]) != int(before.get(k, 0)):
					rebaked.append(k)
			_check("dig_rebakes_one_chunk", rebaked == [c], "dug %s: rebaked %s (expected [%s]); space applied and rebaked in %d us, last chunk bake %d us" % [Vector2i(x, y), rebaked, c, usec, int(layer.get("last_bake_usec"))])
			return
	_check("dig_rebakes_one_chunk", false, "no interior hill tile found")


func _check_data_only() -> void:
	var fixture: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://tools/v8/fixtures/data_only_wall.json"))
	var base: String = ArtData.ROOT + "materials/"
	var styles: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(base + "construction_styles.json"))
	var looks: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(base + "material_looks.json"))
	var defaults: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(base + "category_defaults.json"))
	for m: Variant in fixture.get("materials", []):
		(looks["materials"] as Array).append(m)
	var added: Dictionary = fixture.get("blueprints", {})
	for s: Variant in styles.get("styles", []):
		var d: Dictionary = s
		if added.has(str(d.get("id", ""))):
			(d["sim_blueprints"] as Array).append_array(added[str(d["id"])])
	var resolver: MaterialResolver = MaterialResolver.new()
	resolver.load_schema_defaults(JSON.parse_string(FileAccess.get_file_as_string(base + "construction_styles.schema.json")))
	resolver.load_documents(styles, looks, defaults)
	var blueprint: String = str((added.values()[0] as Array)[0])
	var mat: int = int((fixture["materials"] as Array)[0]["sim_id"])
	var r: Dictionary = resolver.resolve(blueprint, mat, "", "wall")
	var expect: Dictionary = fixture.get("expect", {})
	var match_all: bool = resolver.problems.is_empty()
	for k: String in expect.keys():
		match_all = match_all and str(r.get(k, "")) == str(expect[k])
	_check("data_only_resolves", match_all, "%s + %d → style %s (%s), material %s, tint %s (%s); loader problems %s" % [blueprint, mat, r.get("style_id"), r.get("style_source"), r.get("material_key"), r.get("tint"), r.get("tint_source"), resolver.problems])
	var probs: Array[String] = []
	var set_images: Dictionary = WallBaker.prepare_set(r, ArtData.param("walls", {}), probs)
	var walls: WallBaker = WallBaker.new()
	walls.chunk_tiles = 16
	walls.sets["test"] = set_images
	for y: int in range(4, 7):
		for x: int in range(4, 7):
			if x == 5 and y == 5:
				continue
			var t: Vector2i = Vector2i(x, y)
			walls.occupancy[t] = true
			walls.cell_set[t] = "test"
	walls.openings[Vector2i(5, 6)] = {"style": "Door"}
	var rows: Dictionary = walls.bake(0, 0)
	var coverage: Array[String] = []
	var ok: bool = probs.is_empty() and rows.size() == 3
	for ty: int in [4, 5, 6]:
		var strip: Image = rows.get(ty, null)
		if strip == null:
			ok = false
			continue
		var opaque: int = 0
		for py: int in strip.get_height():
			for px: int in range(4 * TILE, 7 * TILE):
				if strip.get_pixel(px, py).a > 0.5:
					opaque += 1
		var total: int = 3 * TILE * strip.get_height()
		coverage.append("%d: %.2f" % [ty, float(opaque) / total])
		# North row: three tops + faces? (the middle has a wall south of it: no face) → most of 3 × 64 × 64 opaque.
		var want: float = 0.45 if ty == 4 else (0.40 if ty == 5 else 0.55)
		ok = ok and float(opaque) / total >= want
	_check("data_only_wall_renders", ok, "texture set %s; strip coverage %s; problems %s" % [(r.get("textures", {}) as Dictionary).get("face", ""), coverage, probs])


## Fraction of land (opaque) pixels in the 5 × 5 box around (x, y).
func _land_fraction(img: Image, x: int, y: int) -> float:
	var n: int = 0
	var land: int = 0
	for dy: int in range(-2, 3):
		for dx: int in range(-2, 3):
			var px: int = x + dx
			var py: int = y + dy
			if px < 0 or py < 0 or px >= img.get_width() or py >= img.get_height():
				continue
			n += 1
			if img.get_pixel(px, py).a > 0.5:
				land += 1
	return float(land) / maxf(1.0, float(n))


## Distance from `centre` along `dir` to the first sign change of the baker's field (sampled every 0.25 px).
func _field_radius(baker: GroundBaker, map: GroundBaker.GroundMap, centre: Vector2, dir: Vector2, pair: int) -> float:
	var prev: float = baker.shore_distance(map, floori(centre.x), floori(centre.y), pair)
	for k: int in range(1, 240):
		var p: Vector2 = centre + dir * (k * 0.25)
		var d: float = baker.shore_distance(map, floori(p.x), floori(p.y), pair)
		if d != INF and prev != INF and signf(d) != signf(prev):
			return k * 0.25
		prev = d
	return INF


## Distance from `centre` along `dir` to where the image's land fraction first crosses 0.5 (going from `inside_land`).
func _image_radius(img: Image, centre: Vector2, dir: Vector2, inside_land: bool) -> float:
	for k: int in range(0, 80):
		var p: Vector2 = centre + dir * float(k)
		var f: float = _land_fraction(img, floori(p.x), floori(p.y))
		if (f > 0.5) != inside_land:
			return float(k)
	return INF


func _synthetic_baker() -> GroundBaker:
	var b: GroundBaker = GroundBaker.new()
	var ground: Dictionary = (ArtData.param("ground", {}) as Dictionary).duplicate()
	ground["chunk_tiles"] = 8
	b.configure(ground, ArtData.param("water", {}), ArtData.param("hill", {}))
	var a: Image = Image.create(128, 128, false, Image.FORMAT_RGBA8)
	a.fill(Color(0.40, 0.50, 0.20))
	var c: Image = Image.create(128, 128, false, Image.FORMAT_RGBA8)
	c.fill(Color(0.55, 0.45, 0.35))
	b.set_grounds([a, c])
	return b


func _check_shores_synthetic() -> void:
	var b: GroundBaker = _synthetic_baker()
	var r: float = float(ArtData.param("water.shore_radius_px", 22))
	var m: GroundBaker.GroundMap = GroundBaker.GroundMap.new()
	m.setup(8, 8)
	m.ground.fill(0)
	# The L: a vertical river in column 2 (rows 0–5) turning east along row 5 to the map's edge.
	for y: int in range(0, 6):
		m.ground[y * 8 + 2] = GroundBaker.WATER
	for x: int in range(2, 8):
		m.ground[5 * 8 + x] = GroundBaker.WATER
	var img: Image = b.bake(m, 0, 0)
	# Outer corner: tile (2, 5)'s south-west corner (128, 384), a convex water corner centred r inside the water.
	var outer: Vector2 = Vector2(2 * TILE + r, 6 * TILE - r)
	# Inner corner: tile (3, 4)'s south-west corner (192, 320), a convex land corner centred r inside the land.
	var inner: Vector2 = Vector2(3 * TILE + r, 5 * TILE - r)
	var radii: Array[float] = []
	var worst: float = 0.0
	for k: int in 7:
		var a: float = deg_to_rad(15.0 + k * 10.0)
		var to_outer: Vector2 = Vector2(-cos(a), sin(a))
		var to_inner: Vector2 = Vector2(-cos(a), sin(a))
		var ro: float = _field_radius(b, m, outer, to_outer, 0)
		var ri: float = _field_radius(b, m, inner, to_inner, 0)
		radii.append(ro)
		radii.append(ri)
		worst = maxf(worst, maxf(absf(ro - r), absf(ri - r)))
	_check("shore_quarter_circle", worst <= 3.0, "contour radius at the outer and inner corner along 7 rays each: %s (expected %.0f ±3 px; worst %.2f)" % [_fmt(radii), r, worst])
	# A straight shore: tile (2, 2) water, (1, 2) land — the contour on x = 128 for every row away from corners.
	var straight_worst: float = 0.0
	var offsets: Array[float] = []
	for y: int in range(2 * TILE + 8, 3 * TILE - 8, 4):
		var x_field: float = _field_radius(b, m, Vector2(2 * TILE - 20, y + 0.5), Vector2.RIGHT, 0) + 2 * TILE - 20
		straight_worst = maxf(straight_worst, absf(x_field - 2 * TILE))
		offsets.append(_image_radius(img, Vector2(2 * TILE - 20, y), Vector2.RIGHT, true) - 20.0)
	_check("shore_straight_on_tile_edge", straight_worst <= 1.0, "straight shore contour within %.2f px of the tile edge x = %d (bar 1)" % [straight_worst, 2 * TILE])
	var straight_offset: float = _mean(offsets)
	# The visible edge (land fraction 0.5) at the outer arc against the straight shore's offset (bank into water).
	var arc: Array[float] = []
	for k: int in 5:
		var a: float = deg_to_rad(25.0 + k * 10.0)
		arc.append(_image_radius(img, outer, Vector2(-cos(a), sin(a)), false))
	var arc_offset: float = r - _mean(arc)
	var square_offset: float = r * sqrt(2.0) - _mean(arc)
	_check("shore_bank_follows_contour", absf(arc_offset - straight_offset) <= 3.0, "visible edge %.1f px into the water along the straight shore, %.1f px at the rounded corner (±3; a square corner would read %.1f)" % [straight_offset, arc_offset, square_offset])
	img.save_png(OS.get_user_data_dir().path_join("mart21_shore_l.png"))


func _check_hills_synthetic() -> void:
	var b: GroundBaker = _synthetic_baker()
	var r: float = float(ArtData.param("hill.edge_radius_px", 16))
	var m: GroundBaker.GroundMap = GroundBaker.GroundMap.new()
	m.setup(8, 8)
	m.ground.fill(0)
	for y: int in range(2, 5):
		for x: int in range(2, 5):
			var i: int = y * 8 + x
			m.ground[i] = 1
			m.crisp[i] = 1
			m.rock[i] = 1
			if y == 4:
				m.face[i] = 1
	var img: Image = b.bake(m, 0, 0)
	var rock: Color = Color(0.55, 0.45, 0.35)
	# North-west top corner (128, 128): rounded with radius r around (128 + r, 128 + r).
	var centre: Vector2 = Vector2(2 * TILE + r, 2 * TILE + r)
	var dir: Vector2 = Vector2(-1, -1).normalized()
	var reach: float = INF
	for k: int in range(0, 40):
		var p: Vector2 = centre + dir * float(k)
		if not _near_colour(img.get_pixel(floori(p.x), floori(p.y)), rock):
			reach = float(k)
			break
	# South-west corner (128, 320) hangs a face: the top stays square there.
	var square: bool = _near_colour(img.get_pixel(2 * TILE + 1, 5 * TILE - 2), rock)
	_check("hill_top_rounded", absf(reach - r) <= 3.0 and square, "north-west top corner: rock ends %.0f px from the arc centre along the diagonal (expected %.0f ±3; a square corner reads %.1f); face corner square: %s" % [reach, r, r * sqrt(2.0), square])


## RGBA8 quantises a fill colour by up to 1/510 per channel.
func _near_colour(a: Color, b: Color) -> bool:
	return absf(a.r - b.r) < 0.01 and absf(a.g - b.g) < 0.01 and absf(a.b - b.b) < 0.01 and a.a > 0.5


func _mean(xs: Array[float]) -> float:
	var t: float = 0.0
	var n: int = 0
	for v: float in xs:
		if v != INF:
			t += v
			n += 1
	return t / n if n > 0 else INF


func _fmt(xs: Array[float]) -> String:
	var out: PackedStringArray = PackedStringArray()
	for v: float in xs:
		out.append("%.1f" % v)
	return ", ".join(out)


func _check_water(layer: Node, world: Node, map: GroundBaker.GroundMap, size: int) -> void:
	var water_src: Image = ArtData.image(str((ArtData.bindings().get("terrain", {}) as Dictionary).get("water", "")))
	var material: ShaderMaterial = layer.get("_material")
	var uploaded: Texture2D = material.get_shader_parameter("water_tex") if material != null else null
	var up_img: Image = uploaded.get_image() if uploaded != null else null
	if water_src == null or up_img == null:
		_check("water_texture", false, "no water texture")
		return
	up_img.convert(Image.FORMAT_RGBA8)
	var src_std: float = float(Tone.metrics(water_src)["std"])
	var up_std: float = float(Tone.metrics(up_img)["std"])
	_check("water_texture_restored", absf(src_std - up_std) < 0.001 and up_std >= 0.06, "uploaded water luminance std %.4f = the 05 texture's %.4f (≥ 0.06; M-ART2 D19 compressed it to 0.025)" % [up_std, src_std])
	var chunks: Array[Vector2i] = []
	for cy: int in ceili(float(map.height) / size):
		for cx: int in ceili(float(map.width) / size):
			var has: bool = false
			for ty: int in range(cy * size, mini(map.height, cy * size + size)):
				for tx: int in range(cx * size, mini(map.width, cx * size + size)):
					has = has or map.at(tx, ty) == GroundBaker.WATER
			if has:
				chunks.append(Vector2i(cx, cy))
	var weakest: float = float(layer.call("night_weakest"))
	for tone: String in ["day", "winter", "drought"]:
		if tone != "day":
			layer.call("force_tone", tone)
		var min_night: float = INF
		var min_hue: float = INF
		var clipped: float = 0.0
		var measured: int = 0
		var data: PackedByteArray = up_img.get_data()
		for c: Vector2i in chunks:
			layer.call("bake_chunk_now", c)
			var img: Image = layer.call("chunk_image", c)
			var lift: Vector3 = ((layer.call("water_lifts") as Dictionary).get(c, [Vector3.ZERO]) as Array)[0]
			var water: Color = Tone.mean_rgb(up_img)
			water = Color(clampf(water.r + lift.x, 0, 1), clampf(water.g + lift.y, 0, 1), clampf(water.b + lift.z, 0, 1))
			# Land pixels of tiles beside water (8-neighbourhood), read from the baked chunk itself.
			var by_ground: Dictionary = {}
			for ty: int in range(c.y * size, c.y * size + size):
				for tx: int in range(c.x * size, c.x * size + size):
					var g: int = map.at(tx, ty)
					if g < 0:
						continue
					var beside: bool = false
					for d: Vector2i in [Vector2i(-1, -1), Vector2i(0, -1), Vector2i(1, -1), Vector2i(-1, 0), Vector2i(1, 0), Vector2i(-1, 1), Vector2i(0, 1), Vector2i(1, 1)]:
						beside = beside or map.at(tx + d.x, ty + d.y) == GroundBaker.WATER
					if not beside:
						continue
					var tile: Image = img.get_region(Rect2i((tx - c.x * size) * TILE + 16, (ty - c.y * size) * TILE + 16, 32, 32))
					if not by_ground.has(g):
						by_ground[g] = []
					(by_ground[g] as Array).append(Tone.mean_rgb(tile))
			for g: int in by_ground.keys():
				var sum: Vector3 = Vector3.ZERO
				for col: Color in by_ground[g]:
					sum += Vector3(col.r, col.g, col.b)
				sum /= float((by_ground[g] as Array).size())
				var land: Color = Color(sum.x, sum.y, sum.z)
				min_night = minf(min_night, absf(water.get_luminance() - land.get_luminance()) * weakest)
				var dh: float = absf(water.h - land.h)
				min_hue = minf(min_hue, minf(dh, 1.0 - dh))
				measured += 1
			var over: int = 0
			for i: int in range(0, data.size(), 4):
				if data[i] / 255.0 + lift.x > 1.0 or data[i + 1] / 255.0 + lift.y > 1.0 or data[i + 2] / 255.0 + lift.z > 1.0:
					over += 1
			clipped = maxf(clipped, float(over) / (data.size() / 4.0))
		_check("water_contrast_%s" % tone, measured > 0 and min_night >= 0.10 and min_hue >= 0.12 and clipped < 0.01, "%d chunks with water, %d land classes beside water: min luminance difference at full night %.4f (≥ 0.10), min hue distance %.4f turns (≥ 0.12), lifted texture clipped %.4f (< 0.01)" % [chunks.size(), measured, min_night, min_hue, clipped])
	layer.call("force_tone", "day")
	_check_fords(layer, world, map, size)
	_check_water_steps(layer, map, size, chunks, up_img)


func _check_fords(layer: Node, world: Node, map: GroundBaker.GroundMap, size: int) -> void:
	var space: Dictionary = SimReads.read(world, "get_space")
	var xs: PackedInt32Array = space.get("ford_xs", PackedInt32Array())
	var ys: PackedInt32Array = space.get("ford_ys", PackedInt32Array())
	var spots: Array = layer.get("STONE_SPOTS")
	var total: int = 0
	var on_water: int = 0
	var rounded: int = 0
	for i: int in mini(xs.size(), ys.size()):
		var t: Vector2i = Vector2i(xs[i], ys[i])
		var c: Vector2i = Vector2i(floori(float(t.x) / size), floori(float(t.y) / size))
		layer.call("bake_chunk_now", c)
		var img: Image = layer.call("chunk_image", c)
		for spot: Variant in spots:
			var p: Vector2i = (t - c * size) * TILE + (spot as Vector2i)
			total += 1
			if img.get_pixelv(p).a < 0.5:
				on_water += 1
		var b: GroundBaker = layer.call("baker")
		for corner: Vector2i in [Vector2i(2, 2), Vector2i(61, 2), Vector2i(2, 61), Vector2i(61, 61)]:
			var d: float = b.shore_distance(map, t.x * TILE + corner.x, t.y * TILE + corner.y, 0)
			if d != INF and d > 0.0:
				rounded += 1
	_check("fords_on_rounded_water", total > 0 and on_water * 10 >= total * 9, "%d ford tiles: %d of %d stone spots on water pixels (≥ 90 %%); %d ford-tile corners turned to land by rounding" % [xs.size(), on_water, total, rounded])


func _check_water_steps(layer: Node, map: GroundBaker.GroundMap, size: int, chunks: Array[Vector2i], water: Image) -> void:
	var worst: int = 0
	var runs: int = 0
	var p: int = water.get_width()
	for c: Vector2i in chunks.slice(0, 6):
		layer.call("bake_chunk_now", c)
		var img: Image = (layer.call("chunk_image", c) as Image).duplicate()
		var lift: Vector3 = ((layer.call("water_lifts") as Dictionary).get(c, [Vector3.ZERO]) as Array)[0]
		var x0: int = c.x * size
		var y0: int = c.y * size
		for y: int in img.get_height():
			for x: int in img.get_width():
				var col: Color = img.get_pixel(x, y)
				if col.a < 1.0:
					var w: Color = water.get_pixel(posmod(x0 * TILE + x, p), posmod(y0 * TILE + y, p))
					w = Color(clampf(w.r + lift.x, 0, 1), clampf(w.g + lift.y, 0, 1), clampf(w.b + lift.z, 0, 1))
					img.set_pixel(x, y, w.lerp(Color(col, 1.0), col.a))
		for vertical: bool in [true, false]:
			for line: int in range(1, size):
				var run_start: int = -1
				for along: int in range(0, size + 1):
					var ok: bool = false
					if along < size:
						var a: Vector2i = Vector2i(x0 + line - 1, y0 + along) if vertical else Vector2i(x0 + along, y0 + line - 1)
						var b: Vector2i = a + (Vector2i(1, 0) if vertical else Vector2i(0, 1))
						var wa: bool = map.at(a.x, a.y) == GroundBaker.WATER
						var wb: bool = map.at(b.x, b.y) == GroundBaker.WATER
						ok = wa != wb and map.at(a.x, a.y) != GroundBaker.VOID and map.at(b.x, b.y) != GroundBaker.VOID
					if ok and run_start < 0:
						run_start = along
					elif not ok and run_start >= 0:
						runs += 1
						var start: Vector2i = Vector2i(line * TILE, run_start * TILE) if vertical else Vector2i(run_start * TILE, line * TILE)
						worst = maxi(worst, _longest_step_run(img, start, (along - run_start) * TILE, vertical))
						run_start = -1
	_check("water_edges_not_square", runs > 0 and worst <= 24, "%d water-edge runs in %d river chunks (composited over the lifted water): longest identical luminance step run %d px (bar 24)" % [runs, mini(6, chunks.size()), worst])
