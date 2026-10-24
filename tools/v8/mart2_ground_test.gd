# mart2_ground_test.gd — headless tests of the M-ART2 ground, water, edges and built space (goal §8).
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
#   Prints one MART2_GROUND line per check and MART2_GROUND_DONE failures=N; exits 1 on any failure.
#   `godot --headless --path . --script res://tools/v8/mart2_ground_test.gd [-- --write]`
# Bridge methods called: reset_world, advance_ticks (direct); the layers read the rest.
extends SceneTree

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Tone = preload("res://scripts/world/art/tone.gd")
const GroundBaker = preload("res://scripts/world/art/ground_baker.gd")
const WallBaker = preload("res://scripts/world/art/wall_baker.gd")
const MaterialResolver = preload("res://scripts/world/art/material_resolver.gd")
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
	_finish()


func _finish() -> void:
	print("MART2_GROUND_DONE failures=%d" % _failures)
	quit(1 if _failures > 0 else 0)


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
	fresh.configure(ArtData.param("ground", {}), ArtData.param("water", {}))
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


func _check_tone(layer: Node, map: GroundBaker.GroundMap, c: Vector2i, img: Image, size: int) -> void:
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
