# M-UX A10 focused renderer contracts: signed water contrast, bounded colour transform,
# exact 32 px shore radius and existing-data snow masks including chunk neighbors.
# Bridge methods: get_space for a labeled render-only built-wall fixture; main scene normal reads.
# Synthetic renderer fixtures do not change simulation state or establish visual acceptance.
extends SceneTree

const Ground = preload("res://scripts/world/colony/ground_layer.gd")
const Baker = preload("res://scripts/world/art/ground_baker.gd")
const Tone = preload("res://scripts/world/art/tone.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Daylight = preload("res://scripts/world/daylight_controller.gd")
const Store = preload("res://scripts/ui/mux/settings_store.gd")
var _failures: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(label: String, ok: bool) -> void:
	print("MUX_RENDERER_%s %s" % ["PASS" if ok else "FAIL", label])
	if not ok:
		_failures += 1


func _run() -> void:
	var layer: Node2D = Ground.new()
	var baker: RefCounted = layer.get("_baker")
	baker.call("configure", ArtData.param("ground", {}), ArtData.param("water", {}))
	_check("shore radius reaches 32 rather than clamping to 31", is_equal_approx(float(baker.get("shore_radius")), 32.0))
	_check_rock_shore()
	var source: Image = Image.create(8, 8, false, Image.FORMAT_RGBA8)
	source.fill(Color(0.15, 0.17, 0.12))
	source.set_pixel(0, 0, Color(0.18, 0.20, 0.15))
	var original: PackedByteArray = source.get_data()
	var lifted: Image = layer.call("_ground_floor", source)
	_check("land floor satisfies night budget", Tone.mean_rgb(lifted, 1).get_luminance() >= 0.28)
	_check("prepared floor leaves source unchanged", source.get_data() == original)
	_check("neutral floor retains texture variation", absf((lifted.get_pixel(0, 0).r - lifted.get_pixel(1, 1).r) - (source.get_pixel(0, 0).r - source.get_pixel(1, 1).r)) < 0.005)
	layer.set("_water_mean", Color(0.35, 0.55, 0.65))
	var gain: Vector3 = layer.call("water_gain")
	var water: Color = Color(0.35 * gain.x, 0.55 * gain.y, 0.65 * gain.z)
	_check("dark blue mean remains bounded", water.b > water.g and water.g > water.r and water.get_luminance() <= 0.055)
	_check("water attenuation cannot clip source channels", gain.x >= 0.0 and gain.y >= 0.0 and gain.z >= 0.0 and gain.x <= 1.0 and gain.y <= 1.0 and gain.z <= 1.0)
	_check("signed night contrast keeps original bar", (0.28 - water.get_luminance()) * float(layer.call("night_weakest")) >= 0.10)
	_check("negative control bright water fails signed bar", (0.28 - 0.6) * float(layer.call("night_weakest")) < 0.10)
	var map: Baker.GroundMap = Baker.GroundMap.new()
	map.setup(32, 32)
	map.ground.fill(0)
	map.tree[15 * 32 + 15] = 1
	map.ground[2 * 32 + 2] = Baker.WATER
	map.crisp[3 * 32 + 3] = 1
	layer.set("_map", map)
	var walls: RefCounted = layer.get("_walls")
	walls.set("occupancy", {Vector2i(16, 15): true, Vector2i(17, 15): true})
	walls.set("openings", {Vector2i(17, 15): true})
	var a: Image = (layer.call("snow_mask", Vector2i.ZERO) as ImageTexture).get_image()
	var b: Image = (layer.call("snow_mask", Vector2i(1, 0)) as ImageTexture).get_image()
	_check("tree base comes from existing occupancy", a.get_pixel(16, 16).r > 0.9)
	_check("tree mask agrees across chunk seam", a.get_pixel(16, 16) == b.get_pixel(0, 16))
	_check("wall mask crosses chunk seam", a.get_pixel(17, 16).g > 0.9 and a.get_pixel(17, 16) == b.get_pixel(1, 16))
	_check("door openings do not collect wall drifts", b.get_pixel(2, 16).g == 0.0)
	_check("water excludes snow", a.get_pixel(3, 3).b == 0.0)
	_check("constructed floor excludes snow", a.get_pixel(4, 4).b == 0.0)
	map.tree[15 * 32 + 15] = 0
	var removed: Image = (layer.call("snow_mask", Vector2i.ZERO) as ImageTexture).get_image()
	_check("removed tree drops accumulation mask", removed.get_pixel(16, 16).r == 0.0)
	_check("snow feature and alpha contract", float(ArtData.param("season.snow_feature_px", 0)) >= 512.0 and float(ArtData.param("season.snow_alpha", 1)) <= 0.45)
	var snow_rgb: Array = ArtData.param("season.snow_colour", [])
	_check("D18 snow pigment contract", snow_rgb == [0.78, 0.82, 0.86])
	if snow_rgb.size() == 3:
		var snow: Color = Color(float(snow_rgb[0]), float(snow_rgb[1]), float(snow_rgb[2]))
		var drift_low: float = lerpf(0.28, snow.get_luminance(), 0.45)
		var drift_high: float = lerpf(0.38, snow.get_luminance(), 0.45)
		_check("snow drift is brighter while preserving signed water separation", drift_low >= 0.52 and drift_high <= 0.58 and (drift_low - water.get_luminance()) * float(layer.call("night_weakest")) >= 0.10)
	layer.free()
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var output_index: int = args.find("--capture-out")
	if output_index >= 0 and output_index + 1 < args.size():
		await _capture(args[output_index + 1])
	print("MUX_RENDERER_DONE failures=%d" % _failures)
	quit(0 if _failures == 0 else 1)


func _frames(count: int) -> void:
	for _i: int in count:
		await process_frame


func _check_rock_shore() -> void:
	var baker: Baker = Baker.new()
	baker.configure(ArtData.param("ground", {}), ArtData.param("water", {}))
	baker.chunk_tiles = 4
	var land: Image = Image.create(64, 64, false, Image.FORMAT_RGBA8)
	land.fill(Color(0.3, 0.3, 0.3))
	baker.set_grounds([land])
	var map: Baker.GroundMap = Baker.GroundMap.new()
	map.setup(4, 4)
	map.ground.fill(0)
	map.crisp.fill(1)
	map.rock.fill(1)
	map.ground[2 * 4 + 2] = Baker.WATER
	map.rock[2 * 4 + 2] = 0
	map.crisp[2 * 4 + 2] = 0
	var corner: Vector2i = Vector2i(130, 130)
	var rounded: Image = baker.bake(map, 0, 0)
	_check("natural rock shore class participates", baker._water_class(map, 1, 1) == 0)
	_check("natural rock water corner rounds in field and bake", baker.shore_distance(map, corner.x, corner.y) > 0.0 and rounded.get_pixelv(corner).a > 0.9)
	map.rock.fill(0)
	var constructed: Image = baker.bake(map, 0, 0)
	_check("constructed crisp wall excluded from shore class", baker._water_class(map, 1, 1) == -1)
	_check("constructed crisp bank retains square water corner", constructed.get_pixelv(corner).a < 0.1)


## Comparison fixture only: pause simulation, force existing season hooks and daytime/nighttime modulation.
func _capture(output: String) -> void:
	if DisplayServer.get_name() == "headless" or not output.is_absolute_path():
		_check("capture requires a window and absolute output path", false)
		return
	DirAccess.make_dir_recursive_absolute(output)
	var main: Node = (load("res://scenes/main.tscn") as PackedScene).instantiate()
	var settings_path: String = output.path_join("renderer-test-settings-%d.cfg" % OS.get_process_id())
	main.get_node("UI/MuxHud").set("store", Store.new(settings_path))
	print("MUX_RENDERER_SETTINGS " + settings_path)
	root.add_child(main)
	await _frames(20)
	var camera: Camera2D = main.get_node("Camera2D") as Camera2D
	camera.call("set_paused", true)
	var ground: Node = main.get_node("ColonyView/GroundLayer")
	var trees: Node = main.get_node("ColonyView/TreeLayer")
	var minimap: Node = main.get_node("UI/MuxHud").get("minimap")
	for _i: int in 600:
		if minimap.call("map_image") != null and not bool(minimap.call("is_building")):
			break
		await process_frame
	var published: bool = minimap.call("map_image") != null and not bool(minimap.call("is_building"))
	_check("minimap published before comparison capture", published)
	if not published:
		return
	var map: Baker.GroundMap = ground.call("ground_map")
	var river: Vector2i = Vector2i(-1, -1)
	for y: int in range(16, map.height - 16):
		for x: int in range(16, map.width - 16):
			if map.at(x, y) == Baker.WATER:
				river = Vector2i(x, y)
				break
		if river.x >= 0:
			break
	_check("river comparison fixture exists", river.x >= 0)
	main.get_node("DaylightController").set_process(false)
	var light: CanvasModulate = main.get_node("DaylightController/CanvasModulate") as CanvasModulate
	for location: String in ["camp", "river"]:
		for level: float in [1.0, 0.25]:
			for tone: String in ["day", "winter"]:
				ground.set_process(true)
				var season: String = "winter" if tone == "winter" else "summer"
				trees.call("force_season", season, season)
				camera.zoom = Vector2.ONE * level
				if location == "camp":
					camera.call("center_on_camp")
				else:
					camera.call("center_on_tile", river)
				await _frames(15)
				ground.set_process(false)
				ground.call("_update_trees")
				ground.call("force_tone", tone)
				_check("capture canopy season " + location + " " + tone, str(ground.get("_canopy_season")) == season)
				for night: bool in [false, true]:
					light.color = Daylight.night_colour() if night else Color.WHITE
					await _frames(5)
					await RenderingServer.frame_post_draw
					var image: Image = root.get_texture().get_image()
					var name: String = "%s_%s_%s_%.2f.png" % [location, tone, "night" if night else "day", level]
					_check("capture " + name, image.save_png(output.path_join(name)) == OK)
	await _capture_drift_fixture(main, ground, trees, camera, light, output)


## Render-only snapshot fixture: a real existing tree plus three built wall tiles,
## resolved through the normal material/wall baker. No command or simulation mutation.
func _capture_drift_fixture(main: Node, ground: Node, trees: Node, camera: Camera2D, light: CanvasModulate, output: String) -> void:
	var map: Baker.GroundMap = ground.call("ground_map")
	var tree: Vector2i = Vector2i(-1, -1)
	for y: int in range(16, map.height - 16):
		for x: int in range(16, map.width - 16):
			if map.tree_at(x, y) == 0:
				continue
			var clear: bool = true
			for dx: int in range(-3, 0):
				clear = clear and map.at(x + dx, y + 2) >= 0 and not map.is_crisp(x + dx, y + 2) and map.tree_at(x + dx, y + 2) == 0
			if clear:
				tree = Vector2i(x, y)
				break
		if tree.x >= 0:
			break
	_check("drift fixture has tree beside open wall site", tree.x >= 0)
	if tree.x < 0:
		return
	var space: Dictionary = (main.get_node("WorldSim").call("get_space") as Dictionary).duplicate(true)
	var cfg: Dictionary = ArtData.bindings().get("space", {})
	var style: String = ""
	for candidate: String in cfg.get("wall_kind_styles", []):
		if not (cfg.get("door_styles", []) as Array).has(candidate):
			style = candidate
			break
	_check("drift fixture resolves a data-bound wall style", style != "")
	if style == "":
		return
	space["wall_xs"] = PackedInt32Array([tree.x - 3, tree.x - 2, tree.x - 1])
	space["wall_ys"] = PackedInt32Array([tree.y + 2, tree.y + 2, tree.y + 2])
	space["wall_styles"] = PackedStringArray([style, style, style])
	space["wall_materials"] = PackedInt32Array([-1, -1, -1])
	space["wall_categories"] = PackedStringArray()
	space["wall_kinds"] = PackedInt32Array()
	var label: Label = Label.new()
	label.text = "RENDER-ONLY FIXTURE: existing tree + 3 built wall tiles; winter"
	label.position = Vector2(460, 160)
	main.get_node("UI").add_child(label)
	print("MUX_RENDERER_DRIFT_FIXTURE tree=%s walls=%s style=%s simulation_unchanged=true" % [tree, space["wall_xs"], style])
	for level: float in [1.0, 0.25]:
		ground.set_process(true)
		trees.call("force_season", "winter", "winter")
		camera.zoom = Vector2.ONE * level
		camera.call("center_on_tile", tree + Vector2i(-1, 1))
		await _frames(15)
		ground.set_process(false)
		ground.call("_update_trees")
		ground.call("force_tone", "winter")
		ground.call("_apply_space", space)
		ground.call("_rebake_all")
		var walls: RefCounted = ground.call("wall_baker")
		_check("drift capture contains three built walls %.2f" % level, (walls.get("occupancy") as Dictionary).size() == 3)
		for night: bool in [false, true]:
			light.color = Daylight.night_colour() if night else Color.WHITE
			await _frames(5)
			await RenderingServer.frame_post_draw
			var image: Image = root.get_texture().get_image()
			var name: String = "drift_fixture_winter_%s_%.2f.png" % ["night" if night else "day", level]
			_check("capture " + name, image.save_png(output.path_join(name)) == OK)
