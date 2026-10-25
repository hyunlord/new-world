# ground_layer.gd (ColonyView/GroundLayer in scenes/main.tscn): the ground, water, banks, floors, walls, doors, fords
#   and hill faces of the colony view at 64-px tiles (M-ART2 §3.4, ADR 0034).
# Ground: a GroundMap (one ground index per tile) is built from the terrain snapshot, the camp clearing around the
#   hearth (value noise on the world seed), the built space (floors resolved through the material system, dug tiles)
#   and the bindings (assets/v8art/bindings.json: terrain code → ground id); 16 × 16-tile chunks are baked by
#   ground_baker.gd only for the view plus a one-chunk margin and kept in an LRU cache (`ground.chunk_cache_max`);
#   each chunk is a Sprite2D whose shader (shaders/ground_chunk.gdshader) draws the water beneath its transparent water.
# Built space: wall tiles resolve (style = the blueprint that built them, material, category) through
#   material_resolver.gd; wall_baker.gd assembles walls and hill faces into one strip per tile row, a Sprite2D in the
#   y-sorted ColonyView/World at the row's base y; doors stand there as their own sprites (open while a person is on
#   the tile); ford stones lie on the water (four sprites placed by code, edges.py `ford_overlay` positions).
# Rebaking: when `get_space`'s version moves, the tiles whose ground, crispness or wall changed mark their chunks
#   (and a neighbour chunk when the tile is on its border, since blends read one tile across) and only those rebake;
#   a season or drought change and an F12 params reload rebake every cached chunk.
# Tone: ground textures are resampled, season-toned and capped once per tone state, then given a neutral
#   luminance floor after material value factors. The floor makes dark-water night contrast feasible (M-UX D9).
#   Below 1× a data-driven terrain tint follows reduction; standing sprites remain independent.
# Water: source pixels remain unchanged on upload. Bounded channel attenuation produces the configured dark-blue
#   mean without clipping; water_lift reports signed land-minus-water contrast at every map tint for the gate.
# Snow: a world-space shader field (>=512 px, alpha<=0.45) with cached tree-base/wall occupancy masks.
# Canopy (M-ART2.1 §2.2, ¼× only, `trees.quarter_canopy`): the tree tiles (get_trees, persisted material species)
#   are a GroundMap layer; a chunk baked at texture factor 4 carries the canopy mass per species in its season
#   (the season's Lab shift per pixel, as the 1× sprites get it — a deciduous winter uses its bare frame's mean colour
#   modulated by the texture's luminance — on a ¼-size copy scaled back by nearest, so the ¼ texture keeps every
#   texel), its tiles leave the map tint, and the tree layer hides the sprites standing there only where a canopy was
#   actually baked (`canopy_baked_at`, `canopy_version`).
#   A changed tree set rebakes the ¼× chunks it touches; the canopy follows the season pair at half blend.
# Hooks for tests: `bake_counts` (chunk → bakes), `last_bake_usec`, `chunk_image(c)`, `ground_ids`.
# Bridge methods: via SimReads get_terrain_snapshot, get_building_snapshot, get_space, get_world_info,
#   get_time_snapshot, get_story, get_agent_snapshot, get_colony_versions, get_trees; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Tone = preload("res://scripts/world/art/tone.gd")
const GroundBaker = preload("res://scripts/world/art/ground_baker.gd")
const WallBaker = preload("res://scripts/world/art/wall_baker.gd")
const MaterialResolver = preload("res://scripts/world/art/material_resolver.gd")
const FloraResolver = preload("res://scripts/world/art/flora_resolver.gd")
const TreeLayer = preload("res://scripts/world/colony/tree_layer.gd")
const CHUNK_SHADER: Shader = preload("res://shaders/ground_chunk.gdshader")
const TILE: int = View.TILE
const STONE_SPOTS: Array[Vector2i] = [Vector2i(9, 38), Vector2i(24, 30), Vector2i(39, 35), Vector2i(54, 26)]

## Chunk (Vector2i) → number of bakes since the world reset (tests: a dig rebakes exactly one chunk).
var bake_counts: Dictionary = {}
## Microseconds of the last chunk bake (ground + strips + upload), and the slowest bake's breakdown.
var last_bake_usec: int = 0
var worst_bake: Dictionary = {}
var _mux2_profile: bool = OS.get_environment("WORLDSIM_MUX2_PROFILE") == "1"
var _mux2_frame: int = -1
var _mux2_usec: Dictionary = {}
## Microseconds spent precomputing the dither masks at the last configure (a world load or a params reload).
var warm_usec: int = 0
## Ground ids by ground index.
var ground_ids: Array[String] = []
## Counts changes of which chunks carry the baked ¼× canopy (the tree layer re-checks its sprites when it moves).
var canopy_version: int = 0

var _world_sim: Node = null
var _world: Node2D = null
var _fords: Node2D = null
var _baker: GroundBaker = GroundBaker.new()
var _walls: WallBaker = WallBaker.new()
var _resolver: MaterialResolver = MaterialResolver.new()
var _map: GroundBaker.GroundMap = null
var _ground_index: Dictionary = {}
var _terrain: PackedByteArray = PackedByteArray()
var _w: int = 0
var _h: int = 0
var _seed: int = 0
var _hearth: Vector2i = Vector2i(-1000, -1000)
var _space_version: int = -2
var _tone: String = ""
var _params_version: int = -1
var _chunks: Dictionary = {}
var _strips: Dictionary = {}
var _images: Dictionary = {}
var _lru: Array[Vector2i] = []
var _doors: Dictionary = {}
var _door_tex: Dictionary = {}
var _water_tex: ImageTexture = null
## The water texture's mean colour, and per ground index the prepared texture's mean colour (the lift's inputs).
var _water_mean: Color = Color.BLACK
var _ground_means: Array[Color] = []
## Chunk → [lift (Vector3), min luminance difference (day), min hue distance] of its last bake (tests, the report).
var _lifts: Dictionary = {}
var _material: ShaderMaterial = null
var _problems: Array[String] = []
var _grounds_dirty: bool = false
var _tone_report: Array[Dictionary] = []
var _last_cap: Dictionary = {}
## Keep each baked chunk's full image (tests read them with chunk_image); off in the game to save memory.
var keep_images: bool = false
## The texture resolution factor in use (1, 2, 4) and the factor each cached chunk was textured at.
var _res_factor: int = 1
var _res_of: Dictionary = {}
## Chunks whose current texture carries the baked ¼× canopy (the tree layer hides their sprites only then).
var _canopy_of: Dictionary = {}
const RETEXTURE_PER_FRAME: int = 4
## The built space last applied (dug tiles, floor tile → ground id) and whether the next apply covers every tile.
var _space_dug: Dictionary = {}
var _space_floors: Dictionary = {}
var _space_full: bool = true
var _clearing_id: String = ""
var _clearing_core: int = 4
var _clearing_reach: int = 8
var _clearing_threshold: float = 0.8
var _flora: FloraResolver = FloraResolver.new()
var _trees_version: int = -2
var _tree_cells: Dictionary = {}
var _canopy_season: String = ""
var _canopy_cache: Dictionary = {}
## Bound roster plus authoritative extra identities; resolver handles their habit fallback.
var _canopy_species: Array[String] = []


func _ready() -> void:
	z_index = -10
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world = get_node_or_null("../World")
	_fords = Node2D.new()
	_fords.name = "Fords"
	_fords.z_index = 1
	add_child(_fords)
	_resolver.load_imported()
	_problems.append_array(_resolver.problems)
	_flora.load_imported()
	_problems.append_array(_flora.problems)
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	_map = null
	_trees_version = -2
	_tree_cells.clear()
	_canopy_species.clear()
	_canopy_cache.clear()
	_canopy_season = ""
	_space_version = -2
	_hearth = Vector2i(-1000, -1000)
	_drop_all()
	bake_counts.clear()


func _drop_all() -> void:
	for c: Vector2i in _chunks.keys():
		_drop_chunk(c)
	_lru.clear()
	for d: Node in _doors.values():
		(d as Node).queue_free()
	_doors.clear()
	for f: Node in _fords.get_children():
		f.queue_free()


func _drop_chunk(c: Vector2i) -> void:
	if _chunks.has(c):
		(_chunks[c] as Node).queue_free()
		_chunks.erase(c)
	for s: Node in _strips.get(c, []):
		s.queue_free()
	_strips.erase(c)
	_images.erase(c)
	_res_of.erase(c)
	_canopy_of.erase(c)


## The baked ground image of chunk `c` (null when it is not cached).
func chunk_image(c: Vector2i) -> Image:
	return _images.get(c, null)


## Problems found while loading the material data (the loader contract).
func problems() -> Array[String]:
	return _problems


## The ground map in use (null before the terrain arrived; tests).
func ground_map() -> GroundBaker.GroundMap:
	return _map


## The baker in use (tests: a second bake of the same map must be identical).
func baker() -> GroundBaker:
	return _baker


## The wall baker in use (tests).
func wall_baker() -> WallBaker:
	return _walls


## Bake chunk `c` now and keep it in the cache (tests and tools).
func bake_chunk_now(c: Vector2i) -> void:
	_bake_chunk(c)
	_touch(c)


## Whether the hill-face cap is the current tone's tiled hill ground (tests: a tone or params change must refresh it).
func hill_cap_is_current() -> bool:
	var cap_id: String = ""
	for r: Variant in (ArtData.bindings().get("terrain", {}) as Dictionary).get("codes", []):
		if r is Dictionary and bool((r as Dictionary).get("standing_face", false)):
			cap_id = str((r as Dictionary).get("ground", ""))
			break
	return cap_id != "" and _ground_index.has(cap_id) and _walls.hill_cap == _baker.tiled(_ground_index[cap_id])


## Wall and hill-face row strips drawn now (tests).
func strip_count() -> int:
	var n: int = 0
	for strips: Variant in _strips.values():
		n += (strips as Array).size()
	return n


## Apply a `get_space`-shaped dictionary (tests drive a dig through the same path as a real space change).
func apply_space(space: Dictionary) -> void:
	_apply_space(space)


## The tone cap result of each prepared ground texture: [{id, passed, after}] (tests).
func tone_report() -> Array[Dictionary]:
	return _tone_report


func _process(delta: float) -> void:
	if not _mux2_profile:
		_process_ground(delta)
		return
	var started: int = Time.get_ticks_usec()
	_process_ground(delta)
	_mux2_record("process", Time.get_ticks_usec() - started)


## Read-only frame-local timings; nested bake phases are already included in process.
func mux2_frame_timings() -> Dictionary:
	if not _mux2_profile: return {"status": "DISABLED"}
	_mux2_sync_frame()
	return {"status": "MEASURED", "frame": _mux2_frame, "usec": _mux2_usec.duplicate()}


func _mux2_sync_frame() -> void:
	var frame: int = Engine.get_process_frames()
	if frame != _mux2_frame:
		_mux2_frame = frame
		_mux2_usec = {"process": 0, "chunk_bakes": 0, "compose": 0, "upload": 0, "strips": 0}


func _mux2_record(metric: String, elapsed: int) -> void:
	_mux2_sync_frame()
	_mux2_usec[metric] = int(_mux2_usec.get(metric, 0)) + elapsed


func _process_ground(_delta: float) -> void:
	if _world_sim == null:
		return
	if _map == null and not _build_map():
		return
	if _hearth.x < 0 and _hearth_now().x >= 0:
		# Buildings register on the first tick: redraw with the camp clearing once the hearth exists.
		_drop_all()
		_map = null
		if not _build_map():
			return
	# A parameter reload (F12) or a tone change (season, drought) re-prepares the grounds and the wall sets; the space
	# is then applied in full (wall sets, hill caps and faces rebuilt from the new textures) and every cached chunk
	# rebaked.
	var refresh: bool = false
	if _params_version != ArtData.params_version():
		_params_version = ArtData.params_version()
		_configure()
		refresh = true
	var tone: String = _tone_now()
	if tone != _tone:
		_tone = tone
		_prepare_grounds()
		refresh = true
	var space: Dictionary = SimReads.read(_world_sim, "get_space")
	var version: int = int(space.get("version", -1))
	if version != _space_version or refresh:
		_space_version = version
		if refresh:
			_space_full = true
		_apply_space(space)
	if refresh:
		_rebake_all()
	_update_trees()
	_ensure_visible()
	_update_doors()


func _configure() -> void:
	_baker.configure(ArtData.param("ground", {}), ArtData.param("water", {}), ArtData.param("hill", {}), ArtData.param("trees", {}))
	_canopy_cache.clear()
	_canopy_season = ""
	var t0: int = Time.get_ticks_usec()
	_baker.warm()
	warm_usec = Time.get_ticks_usec() - t0
	_walls.chunk_tiles = _baker.chunk_tiles
	var ink: Variant = ArtData.param("walls.ink_rgba", [20, 18, 16, 204])
	if ink is Array and (ink as Array).size() == 4:
		_walls.ink = Color8(int(ink[0]), int(ink[1]), int(ink[2]), int(ink[3]))
	_walls.sets.clear()
	_door_tex.clear()
	if _material == null:
		_material = ShaderMaterial.new()
		_material.shader = CHUNK_SHADER
	_material.set_shader_parameter("drift_px", float(ArtData.param("water.drift_px", 6.0)))
	_material.set_shader_parameter("drift_hz", float(ArtData.param("water.drift_hz", 0.05)))
	_prepare_grounds()


func _tone_now() -> String:
	var t: Dictionary = SimReads.read(_world_sim, "get_time_snapshot")
	var tones: Dictionary = ArtData.bindings().get("tones", {})
	var by_season: Dictionary = tones.get("seasons", {})
	var tone: String = str(by_season.get(str(int(t.get("season", 0))), "day"))
	var story: Dictionary = SimReads.read(_world_sim, "get_story")
	var names: PackedStringArray = story.get("names", PackedStringArray())
	var by_pressure: Dictionary = tones.get("pressures", {})
	for n: String in names:
		if by_pressure.has(n) and tone == "day":
			tone = str(by_pressure[n])
	return tone


func _build_map() -> bool:
	var snap: Dictionary = SimReads.read(_world_sim, "get_terrain_snapshot")
	_w = int(snap.get("width", 0))
	_h = int(snap.get("height", 0))
	_terrain = snap.get("terrain", PackedByteArray())
	if _w <= 0 or _terrain.size() != _w * _h:
		return false
	_hearth = _hearth_now()
	_seed = int((SimReads.read(_world_sim, "get_world_info") as Dictionary).get("seed", 0))
	_map = GroundBaker.GroundMap.new()
	_map.setup(_w, _h)
	_trees_version = -2
	_tree_cells.clear()
	_space_full = true
	_space_dug = {}
	_space_floors = {}
	var clearing: Dictionary = (ArtData.bindings().get("terrain", {}) as Dictionary).get("clearing", {})
	_clearing_id = str(clearing.get("ground", ""))
	_clearing_core = int(clearing.get("core_tiles", 4))
	_clearing_reach = int(clearing.get("reach_tiles", 8))
	_clearing_threshold = float(ArtData.param("ground.clearing_threshold", 0.8))
	if _params_version != ArtData.params_version():
		_params_version = ArtData.params_version()
		_configure()
	_space_version = -2
	return true


## The tile of the first building whose binding gives light (the hearth); (-1000, -1000) before it exists.
func _hearth_now() -> Vector2i:
	var buildings: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var kinds: PackedByteArray = buildings.get("kinds", PackedByteArray())
	var bxs: PackedInt32Array = buildings.get("xs", PackedInt32Array())
	var bys: PackedInt32Array = buildings.get("ys", PackedInt32Array())
	for i: int in mini(kinds.size(), mini(bxs.size(), bys.size())):
		if bool(ArtData.binding("buildings", kinds[i]).get("light", false)):
			return Vector2i(bxs[i], bys[i])
	return Vector2i(-1000, -1000)


## The ground index of `id`, adding it (and preparing its texture) when new.
func _index_of(id: String) -> int:
	if _ground_index.has(id):
		return _ground_index[id]
	ground_ids.append(id)
	_ground_index[id] = ground_ids.size() - 1
	_grounds_dirty = true
	return _ground_index[id]


func _prepare_grounds() -> void:
	var period: int = int(ArtData.param("ground.texture_period_px", 128))
	var tone: Dictionary = ArtData.param("ground.tone", {})
	var season: Dictionary = ArtData.param("season", {})
	var person_std: float = float((ArtData.manifest().get("tone_reference", {}) as Dictionary).get("person_luminance_std", 0.17))
	var images: Array[Image] = []
	_tone_report.clear()
	_ground_means.clear()
	for id: String in ground_ids:
		var prepared: Image = _ground_image(id, period, tone, season, person_std)
		images.append(prepared)
		_ground_means.append(Tone.mean_rgb(prepared, 3))
		_tone_report.append({"id": id, "passed": _last_cap.get("passed", false), "after": _last_cap.get("after", {})})
	_baker.set_grounds(images)
	var water: Image = ArtData.image(str(ArtData.bindings().get("terrain", {}).get("water", "")))
	if water != null:
		_water_mean = Tone.mean_rgb(water, 1)
		_water_tex = ImageTexture.create_from_image(water)
		if _material != null:
			_material.set_shader_parameter("water_tex", _water_tex)
			_material.set_shader_parameter("water_period", float(water.get_width()))
			var rgb: Array = ArtData.param("water.map_mode_colour", [0.015, 0.05, 0.12])
			if rgb.size() == 3:
				_material.set_shader_parameter("water_colour", Vector3(float(rgb[0]), float(rgb[1]), float(rgb[2])))
			_material.set_shader_parameter("water_map_tint_strength", float(ArtData.param("water.map_mode_tint_strength", 0.3)))


## The weakest channel of the full-night multiplier: a luminance difference shrinks at most by this factor at night.
static func night_weakest() -> float:
	var night: Array = ArtData.param("light.night_tint", [0.86, 0.94, 0.96])
	var weakest: float = clampf(float(ArtData.param("light.night_value", 0.55)), 0.01, 1.0)
	if night.size() == 3:
		weakest *= minf(1.0, minf(float(night[0]), minf(float(night[1]), float(night[2]))))
	return weakest


## The map-tint strength of the land at each texture factor (1, 2, 4) under the current tone.
func _strengths() -> Array[float]:
	var strengths: Dictionary = ArtData.param("ground.map_mode.tint_strength", {})
	var winter: float = float(ArtData.param("ground.map_mode.winter_strength", 0.25)) if _tone == "winter" else 1.0
	return [0.0, clampf(float(strengths.get("0.5", 0.0)) * winter, 0.0, 1.0), clampf(float(strengths.get("0.25", 0.0)) * winter, 0.0, 1.0)]


## Compatibility report: [zero legacy offset, signed land-minus-water day luminance, hue distance].
## Measures every adjacent ground and all map tints; the shader now uses bounded water_gain instead of an offset.
func water_lift(c: Vector2i) -> Array:
	var size: int = _baker.chunk_tiles
	var touching: Dictionary = {}
	for ty: int in range(c.y * size, c.y * size + size):
		for tx: int in range(c.x * size, c.x * size + size):
			if _map.at(tx, ty) != GroundBaker.WATER:
				continue
			for dy: int in range(-1, 2):
				for dx: int in range(-1, 2):
					var g: int = _map.at(tx + dx, ty + dy)
					if g >= 0 and g < _ground_means.size():
						touching[g] = not _map.is_crisp(tx + dx, ty + dy) or bool(touching.get(g, false))
	if touching.is_empty():
		return [Vector3.ZERO, INF, INF]
	var rgb: Array = ArtData.param("water.map_mode_colour", [0.015, 0.05, 0.12])
	var map_colour: Color = Color(float(rgb[0]), float(rgb[1]), float(rgb[2])) if rgb.size() == 3 else _water_mean
	var water_strength: float = float(ArtData.param("water.map_mode_tint_strength", 0.3))
	var palette: Array[Color] = _palette()
	var strengths: Array[float] = _strengths()
	var lands: Array[Color] = []
	var weights: Array[float] = []
	for g: int in touching.keys():
		for k: int in strengths.size():
			var s: float = strengths[k] if bool(touching[g]) and g < palette.size() and palette[g].a > 0.0 else 0.0
			lands.append(_ground_means[g].lerp(Color(palette[g], 1.0) if g < palette.size() else _ground_means[g], s))
			weights.append(strengths[k] * water_strength)
	# Multiplicative channels preserve the source texture without clipping its dark pixels.
	var gain: Vector3 = water_gain()
	var shown_mean: Color = Color(_water_mean.r * gain.x, _water_mean.g * gain.y, _water_mean.b * gain.z)
	var min_dy: float = INF
	var min_dh: float = INF
	for i: int in lands.size():
		var shown: Color = shown_mean.lerp(map_colour, clampf(weights[i], 0.0, 1.0))
		min_dy = minf(min_dy, lands[i].get_luminance() - shown.get_luminance())
		min_dh = minf(min_dh, _hue_distance(shown, lands[i]))
	return [Vector3.ZERO, min_dy, min_dh]


## Per-channel attenuation only: source art remains unchanged, no RGB clipping is introduced.
func water_gain() -> Vector3:
	var rgb: Array = ArtData.param("water.mean_colour", [0.015, 0.05, 0.12])
	return Vector3(clampf(float(rgb[0]) / maxf(_water_mean.r, 0.001), 0.0, 1.0), clampf(float(rgb[1]) / maxf(_water_mean.g, 0.001), 0.0, 1.0), clampf(float(rgb[2]) / maxf(_water_mean.b, 0.001), 0.0, 1.0))


static func _hue_distance(a: Color, b: Color) -> float:
	var d: float = absf(a.h - b.h)
	return minf(d, 1.0 - d)


## The signed water-contrast reports by baked chunk (legacy water_lifts API retained for tooling).
func water_lifts() -> Dictionary:
	return _lifts


## Re-prepare the grounds under tone `name` ("day", "winter", "drought") and rebake the cache (tests and captures).
func force_tone(name: String) -> void:
	_tone = name
	_prepare_grounds()
	_space_full = true
	_apply_space(SimReads.read(_world_sim, "get_space"))
	_rebake_all()


func _ground_image(id: String, period: int, tone: Dictionary, season: Dictionary, person_std: float) -> Image:
	var src: Image
	var native: bool = false
	if id.begins_with("floor:"):
		var parts: PackedStringArray = id.split(":")
		src = ArtData.image_at(MaterialResolver.res_path(parts[1]))
		if src != null and parts.size() >= 5:
			src = Tone.lab_shift(src, float(parts[2]), float(parts[3]), float(parts[4]))
		native = true
	else:
		src = ArtData.image(id)
	if src == null:
		src = Image.create(TILE, TILE, false, Image.FORMAT_RGBA8)
		src.fill(Color(0.24, 0.23, 0.2))
	var img: Image = src.duplicate()
	img.convert(Image.FORMAT_RGBA8)
	if not native and period > 0 and img.get_width() != period:
		img.resize(period, period, Image.INTERPOLATE_LANCZOS)
	img = _toned(img, tone, season, person_std)
	# `ground.value_by_ground` (data): a value factor after the cap, so a raised ground (hill rock) reads as a mass
	# against the calm ground around it (MART2_DECISIONS D18); the ENV1 caps are maxima, a darker ground stays inside.
	var value: float = float((ArtData.param("ground.value_by_ground", {}) as Dictionary).get(id, 1.0))
	if absf(value - 1.0) > 0.001:
		img.adjust_bcs(clampf(value, 0.0, 1.0), 1.0, 1.0)
	return _ground_floor(img)


## Neutral lift after material value factors keeps the signed night-water contrast feasible (M-UX D9).
func _ground_floor(src: Image) -> Image:
	var minimum: float = float(ArtData.param("ground.tone.mean_min", 0.28))
	var lift: float = maxf(0.0, minimum - Tone.mean_rgb(src, 1).get_luminance())
	if lift <= 0.0:
		return src
	var bytes: PackedByteArray = src.get_data()
	var amount: int = ceili(lift * 255.0)
	for i: int in range(0, bytes.size(), 4):
		for channel: int in 3:
			bytes[i + channel] = mini(255, int(bytes[i + channel]) + amount)
	return Image.create_from_data(src.get_width(), src.get_height(), false, Image.FORMAT_RGBA8, bytes)


func _toned(src: Image, tone: Dictionary, season: Dictionary, person_std: float) -> Image:
	var img: Image = src
	if _tone == "winter":
		img = Tone.season(img, "winter", float(season.get("winter_saturation", 0.35)))
	elif _tone == "drought":
		img = Tone.season(img, "drought", float(season.get("drought_mix", 0.35)), season.get("drought_tint", []))
	var r: Dictionary = Tone.cap_ground(img, person_std, float(tone.get("mean_max", 0.38)), float(tone.get("std_ratio", 0.25)), float(tone.get("saturation_max", 0.30)), float(tone.get("margin", 0.9)))
	_last_cap = r
	return r["image"]


func _terrain_row(code: int) -> Dictionary:
	return ArtData.binding("terrain", code)


func _blocking(i: int) -> bool:
	return bool(_terrain_row(_terrain[i]).get("blocks", false))


## Rebuild the space-dependent parts of the map, find what changed and rebake those chunks.
func _apply_space(space: Dictionary) -> void:
	var terrain_cfg: Dictionary = ArtData.bindings().get("terrain", {})
	var dug: Dictionary = {}
	for i: int in mini((space.get("dug_xs", PackedInt32Array()) as PackedInt32Array).size(), (space.get("dug_ys", PackedInt32Array()) as PackedInt32Array).size()):
		dug[Vector2i(space["dug_xs"][i], space["dug_ys"][i])] = true
	var floors: Dictionary = _floor_grounds(space, dug)
	var old_walls: Dictionary = _walls.occupancy.duplicate()
	var old_open: Dictionary = _walls.openings.duplicate()
	var old_sets: Dictionary = _walls.cell_set.duplicate()
	var old_faces: Dictionary = _walls.faces.duplicate()
	var walls: Dictionary = _wall_cells(space)
	# Tiles whose space state changed (all tiles on the first space of a world); a hill's face depends on its south
	# neighbour, so the tile north of a change is recomputed too.
	var changed: Dictionary = {}
	if _space_full:
		for y: int in _h:
			for x: int in _w:
				changed[Vector2i(x, y)] = true
	else:
		for t: Vector2i in _union(_space_dug, dug):
			if _space_dug.has(t) != dug.has(t):
				changed[t] = true
		for t: Vector2i in _union(_space_floors, floors):
			if str(_space_floors.get(t, "")) != str(floors.get(t, "")):
				changed[t] = true
		for t: Vector2i in _union(old_walls, walls):
			if old_walls.has(t) != walls.has(t) or old_open.has(t) != _walls.openings.has(t) or str(old_sets.get(t, "")) != str(_walls.cell_set.get(t, "")):
				changed[t] = true
		for t: Vector2i in changed.keys():
			changed[t + Vector2i(0, -1)] = true
	_space_full = false
	_space_dug = dug
	_space_floors = floors
	var faces: Dictionary = old_faces
	var dirty: Dictionary = {}
	var size: int = _baker.chunk_tiles
	for t: Vector2i in changed.keys():
		if t.x < 0 or t.y < 0 or t.x >= _w or t.y >= _h:
			continue
		var i: int = t.y * _w + t.x
		var before: int = _map.ground[i]
		var before_crisp: int = _map.crisp[i]
		var had_face: bool = faces.has(t)
		var before_rock: int = _map.rock[i]
		_compute_tile(t, terrain_cfg, dug, floors, walls, faces)
		if before != _map.ground[i] or before_crisp != _map.crisp[i] or before_rock != _map.rock[i] or had_face != faces.has(t) or old_walls.has(t) != walls.has(t) or str(old_sets.get(t, "")) != str(_walls.cell_set.get(t, "")) or old_open.has(t) != _walls.openings.has(t):
			_mark(dirty, t.x, t.y, size)
	_walls.faces = faces
	if _grounds_dirty:
		_grounds_dirty = false
		_prepare_grounds()
	var cap_id: String = ""
	for r: Variant in terrain_cfg.get("codes", []):
		if r is Dictionary and bool((r as Dictionary).get("standing_face", false)):
			cap_id = str((r as Dictionary).get("ground", ""))
			break
	if cap_id != "" and _ground_index.has(cap_id):
		_walls.hill_cap = _baker.tiled(_ground_index[cap_id])
	_walls.rock_face = ArtData.image(str(terrain_cfg.get("face", "")))
	_update_fords(space)
	for c: Vector2i in dirty.keys():
		if _chunks.has(c):
			_bake_chunk(c)


## Recompute one tile of the ground map (and whether it draws a hill face) from the terrain and the built space.
func _compute_tile(t: Vector2i, terrain_cfg: Dictionary, dug: Dictionary, floors: Dictionary, walls: Dictionary, faces: Dictionary) -> void:
	var i: int = t.y * _w + t.x
	var row: Dictionary = _terrain_row(_terrain[i])
	var g: int
	var crisp: int = 0
	var rock: int = 0
	faces.erase(t)
	if bool(row.get("water", false)):
		g = GroundBaker.WATER
	elif floors.has(t):
		g = _index_of(floors[t])
	elif dug.has(t):
		g = _index_of(str(terrain_cfg.get("dug_ground", row.get("ground", ""))))
	else:
		g = _index_of(str(row.get("ground", terrain_cfg.get("fallback", ""))))
		if bool(row.get("blocks", false)):
			crisp = 1
			rock = 1
			var south: Vector2i = t + Vector2i(0, 1)
			if t.y + 1 < _h and bool(row.get("standing_face", false)) and (dug.has(south) or not _blocking(i + _w)) and not walls.has(south):
				faces[t] = true
		elif _clearing_id != "" and _worn(t.x, t.y, _clearing_core, _clearing_reach, _clearing_threshold):
			g = _index_of(_clearing_id)
	if walls.has(t):
		crisp = 1
		rock = 0
	_map.ground[i] = g
	_map.crisp[i] = crisp
	_map.rock[i] = rock
	_map.face[i] = 1 if faces.has(t) else 0


func _union(a: Dictionary, b: Dictionary) -> Array:
	var out: Dictionary = a.duplicate()
	out.merge(b)
	return out.keys()


func _mark(dirty: Dictionary, x: int, y: int, size: int) -> void:
	# All eight neighbours: blends reach across edges, and a wall's end cap reads its diagonal south neighbours.
	for d: Vector2i in [Vector2i.ZERO, Vector2i(-1, 0), Vector2i(1, 0), Vector2i(0, -1), Vector2i(0, 1),
			Vector2i(-1, -1), Vector2i(1, -1), Vector2i(-1, 1), Vector2i(1, 1)]:
		var nx: int = x + d.x
		var ny: int = y + d.y
		if nx < 0 or ny < 0 or nx >= _w or ny >= _h:
			continue
		dirty[Vector2i(floori(float(nx) / size), floori(float(ny) / size))] = true


func _worn(x: int, y: int, core: int, reach: int, threshold: float) -> bool:
	var i: int = y * _w + x
	if not bool(_terrain_row(_terrain[i]).get("clearing_ok", false)):
		return false
	var d: int = maxi(absi(x - _hearth.x), absi(y - _hearth.y))
	if d <= core:
		return true
	if d > reach:
		return false
	var falloff: float = 1.0 - float(d - core) / float(reach - core + 1)
	return View.cluster_noise(_seed, x + 1000, y + 1000) + 0.7 * falloff > threshold


## Floor tile → ground id: a floor style the material data knows draws its resolved floor texture (with the
## material's tint); one listed in bindings `space.floor_grounds` draws that ground.
func _floor_grounds(space: Dictionary, dug: Dictionary) -> Dictionary:
	var out: Dictionary = {}
	var xs: PackedInt32Array = space.get("floor_xs", PackedInt32Array())
	var ys: PackedInt32Array = space.get("floor_ys", PackedInt32Array())
	var styles: PackedStringArray = space.get("floor_styles", PackedStringArray())
	var mats: PackedInt32Array = space.get("floor_materials", PackedInt32Array())
	var cats: PackedStringArray = space.get("floor_categories", PackedStringArray())
	var kinds: PackedInt32Array = space.get("floor_kinds", PackedInt32Array())
	var fallback_styles: Array = (ArtData.bindings().get("space", {}) as Dictionary).get("floor_kind_styles", [])
	var grounds: Dictionary = {}
	for r: Variant in (ArtData.bindings().get("space", {}) as Dictionary).get("floor_grounds", []):
		if r is Dictionary:
			grounds[str(r.get("style", ""))] = str(r.get("ground", ""))
	for i: int in mini(xs.size(), ys.size()):
		var t: Vector2i = Vector2i(xs[i], ys[i])
		var style: String = styles[i] if i < styles.size() else (str(fallback_styles[kinds[i]]) if i < kinds.size() and kinds[i] < fallback_styles.size() else "")
		if grounds.has(style):
			out[t] = grounds[style]
			continue
		if dug.has(t) and style == "":
			continue
		var mat: Variant = (mats[i] & 0xFFFFFFFF) if i < mats.size() and mats[i] != -1 else ""
		var r: Dictionary = _resolver.resolve(style, mat, cats[i] if i < cats.size() else "", "floor")
		var tint: Dictionary = r.get("tint", {})
		out[t] = "floor:%s:%s:%s:%s" % [str((r.get("textures", {}) as Dictionary).get("floor", "")), str(tint.get("dL", 0)), str(tint.get("da", 0)), str(tint.get("db", 0))]
	return out


## Wall and door tiles → texture sets (resolved and prepared once per set key).
func _wall_cells(space: Dictionary) -> Dictionary:
	var xs: PackedInt32Array = space.get("wall_xs", PackedInt32Array())
	var ys: PackedInt32Array = space.get("wall_ys", PackedInt32Array())
	var styles: PackedStringArray = space.get("wall_styles", PackedStringArray())
	var mats: PackedInt32Array = space.get("wall_materials", PackedInt32Array())
	var cats: PackedStringArray = space.get("wall_categories", PackedStringArray())
	var kinds: PackedInt32Array = space.get("wall_kinds", PackedInt32Array())
	var cfg: Dictionary = ArtData.bindings().get("space", {})
	var fallback_styles: Array = cfg.get("wall_kind_styles", [])
	var door_styles: Array = cfg.get("door_styles", [])
	var door_flags: PackedByteArray = space.get("wall_is_door", PackedByteArray())
	var occupancy: Dictionary = {}
	var openings: Dictionary = {}
	var cell_set: Dictionary = {}
	var top_override: Dictionary = ArtData.param("walls", {})
	for i: int in mini(xs.size(), ys.size()):
		var t: Vector2i = Vector2i(xs[i], ys[i])
		var style: String = styles[i] if i < styles.size() else (str(fallback_styles[kinds[i]]) if i < kinds.size() and kinds[i] < fallback_styles.size() else "")
		var mat: Variant = (mats[i] & 0xFFFFFFFF) if i < mats.size() and mats[i] != -1 else ""
		var cat: String = cats[i] if i < cats.size() else ""
		occupancy[t] = true
		if (door_flags[i] != 0 if i < door_flags.size() else door_styles.has(style)):
			openings[t] = {"style": style, "material": mat, "category": cat}
			continue
		var r: Dictionary = _resolver.resolve(style, mat, cat, "wall")
		var key: String = "%s|%s|%s" % [str(r.get("style_id", "")), str(r.get("material_key", "")), str(r.get("tint", {}))]
		if not _walls.sets.has(key):
			var probs: Array[String] = []
			_walls.sets[key] = WallBaker.prepare_set(r, top_override, probs)
			_problems.append_array(probs)
		cell_set[t] = key
	_walls.occupancy = occupancy
	_walls.openings = openings
	_walls.cell_set = cell_set
	return occupancy


func _update_fords(space: Dictionary) -> void:
	for f: Node in _fords.get_children():
		f.queue_free()
	var stones: Array = (ArtData.bindings().get("terrain", {}) as Dictionary).get("ford_stones", [])
	var xs: PackedInt32Array = space.get("ford_xs", PackedInt32Array())
	var ys: PackedInt32Array = space.get("ford_ys", PackedInt32Array())
	for i: int in mini(xs.size(), ys.size()):
		var shift: int = posmod(xs[i] * 7 + ys[i] * 11, 5) - 2
		for k: int in mini(stones.size(), STONE_SPOTS.size()):
			var tex: Texture2D = ArtData.texture(str(stones[k]))
			if tex == null:
				continue
			var s: Sprite2D = Sprite2D.new()
			s.texture = tex
			s.centered = true
			s.position = Vector2(xs[i] * TILE + STONE_SPOTS[k].x, ys[i] * TILE + STONE_SPOTS[k].y + shift)
			_fords.add_child(s)


func _ensure_visible() -> void:
	var size: int = _baker.chunk_tiles * TILE
	var view: Rect2 = get_viewport().get_canvas_transform().affine_inverse() * get_viewport_rect()
	var c0: Vector2i = Vector2i(floori(view.position.x / size) - 1, floori(view.position.y / size) - 1)
	var c1: Vector2i = Vector2i(floori(view.end.x / size) + 1, floori(view.end.y / size) + 1)
	var cmax: Vector2i = Vector2i(ceili(float(_w) / _baker.chunk_tiles) - 1, ceili(float(_h) / _baker.chunk_tiles) - 1)
	var wanted: Array[Vector2i] = []
	for cy: int in range(maxi(0, c0.y), mini(cmax.y, c1.y) + 1):
		for cx: int in range(maxi(0, c0.x), mini(cmax.x, c1.x) + 1):
			wanted.append(Vector2i(cx, cy))
	var factor: int = _factor_for_zoom()
	if factor != _res_factor:
		_res_factor = factor
		# Colour is a camera mode, independent of the staggered texture-resolution update below.
		var strength: float = _map_tint_strength()
		for cached: Sprite2D in _chunks.values():
			(cached.material as ShaderMaterial).set_shader_parameter("map_tint_strength", strength)
	var retextured: int = 0
	for c: Vector2i in wanted:
		if not _chunks.has(c):
			_bake_chunk(c)
		elif int(_res_of.get(c, 1)) != _res_factor and retextured < RETEXTURE_PER_FRAME:
			# A zoom step changed the resolution: re-bake a few chunks a frame (the old texture shows meanwhile).
			_bake_chunk(c)
			retextured += 1
		_touch(c)
	var cap: int = maxi(wanted.size(), int(ArtData.param("ground.chunk_cache_max", 48)))
	while _lru.size() > cap:
		var old: Vector2i = _lru.pop_front()
		_drop_chunk(old)


func _touch(c: Vector2i) -> void:
	var at: int = _lru.find(c)
	if at >= 0:
		_lru.remove_at(at)
	_lru.append(c)


## Bake every chunk in the cache again (a tone or parameter change).
func _rebake_all() -> void:
	for c: Vector2i in _chunks.keys():
		_bake_chunk(c)


func _bake_chunk(c: Vector2i) -> void:
	if _map == null:
		return
	var t0: int = Time.get_ticks_usec()
	var f: int = _res_factor
	_baker.canopy_on = f == 4 and _canopy_enabled() and _baker.canopy_count() > 0
	var img: Image = _baker.bake(_map, c.x, c.y)
	var t_compose: int = Time.get_ticks_usec()
	var size: int = _baker.chunk_tiles * TILE
	var sprite: Sprite2D = _chunks.get(c, null)
	if sprite == null:
		sprite = Sprite2D.new()
		sprite.centered = false
		sprite.position = Vector2(c.x * size, c.y * size)
		add_child(sprite)
		_chunks[c] = sprite
	var chunk_material: ShaderMaterial = _material.duplicate() as ShaderMaterial
	chunk_material.set_shader_parameter("terrain_tint_tex", _terrain_tint(c, _baker.canopy_on))
	var lift: Array = water_lift(c)
	_lifts[c] = lift
	chunk_material.set_shader_parameter("water_gain", water_gain())
	chunk_material.set_shader_parameter("snow_mask_tex", snow_mask(c))
	chunk_material.set_shader_parameter("snow_alpha", clampf(float(ArtData.param("season.snow_alpha", 0.45)), 0.0, 0.45) if _tone == "winter" else 0.0)
	chunk_material.set_shader_parameter("snow_feature_px", maxf(512.0, float(ArtData.param("season.snow_feature_px", 512.0))))
	chunk_material.set_shader_parameter("snow_drift_strength", clampf(float(ArtData.param("season.snow_drift_strength", 0.7)), 0.0, 1.0))
	var snow_rgb: Array = ArtData.param("season.snow_colour", [0.78, 0.82, 0.86])
	chunk_material.set_shader_parameter("snow_colour", Vector3(float(snow_rgb[0]), float(snow_rgb[1]), float(snow_rgb[2])))
	chunk_material.set_shader_parameter("chunk_tiles", float(_baker.chunk_tiles))
	chunk_material.set_shader_parameter("map_tint_strength", _map_tint_strength())
	sprite.material = chunk_material
	var reduced: Image = _reduced(img, f)
	var tex: ImageTexture = sprite.texture as ImageTexture
	if tex != null and tex.get_size() == Vector2(reduced.get_size()):
		tex.update(reduced)
	else:
		sprite.texture = ImageTexture.create_from_image(reduced)
	sprite.scale = Vector2(f, f)
	var t_upload: int = Time.get_ticks_usec()
	if keep_images:
		_images[c] = img
	for s: Node in _strips.get(c, []):
		s.queue_free()
	var strips: Array = []
	if _world != null:
		var rows: Dictionary = _walls.bake(c.x, c.y)
		for ty: int in rows.keys():
			var strip: Sprite2D = Sprite2D.new()
			strip.texture = ImageTexture.create_from_image(_reduced(rows[ty], f))
			strip.centered = false
			strip.scale = Vector2(f, f)
			strip.position = Vector2(c.x * size, ty * TILE + View.BASE.y)
			strip.offset = Vector2(0, -float(WallBaker.CAP + View.BASE.y) / f)
			strip.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
			_world.add_child(strip)
			strips.append(strip)
	_strips[c] = strips
	var baked: bool = _baker.canopy_on and _baker.canopy_count() > 0
	if int(_res_of.get(c, 0)) != f or bool(_canopy_of.get(c, false)) != baked:
		canopy_version += 1
	_res_of[c] = f
	_canopy_of[c] = baked
	bake_counts[c] = int(bake_counts.get(c, 0)) + 1
	var t_end: int = Time.get_ticks_usec()
	last_bake_usec = t_end - t0
	if _mux2_profile:
		_mux2_record("chunk_bakes", last_bake_usec)
		_mux2_record("compose", t_compose - t0)
		_mux2_record("upload", t_upload - t_compose)
		_mux2_record("strips", t_end - t_upload)
	if last_bake_usec >= worst_bake.get("total", 0):
		worst_bake = {"total": last_bake_usec, "compose": t_compose - t0, "upload": t_upload - t_compose, "strips": t_end - t_upload, "chunk": c, "rows": strips.size(), "factor": f}


## Neighbor-padded occupancy, rebuilt with the chunk: R tree bases, G walls, B exposed natural ground.
## Uses existing cached tree and structure data; no additional bridge read or per-frame image work.
func snow_mask(c: Vector2i) -> ImageTexture:
	var size: int = _baker.chunk_tiles
	var img: Image = Image.create(size + 2, size + 2, false, Image.FORMAT_RGBA8)
	for y: int in range(-1, size + 1):
		for x: int in range(-1, size + 1):
			var tile: Vector2i = c * size + Vector2i(x, y)
			var exposed: bool = _map.at(tile.x, tile.y) >= 0 and (not _map.is_crisp(tile.x, tile.y) or _map.is_rock(tile.x, tile.y))
			img.set_pixel(x + 1, y + 1, Color(float(_map.tree_at(tile.x, tile.y) > 0), float(_walls.occupancy.has(tile) and not _walls.openings.has(tile)), float(exposed), 1.0))
	return ImageTexture.create_from_image(img)


func _map_tint_strength() -> float:
	var strengths: Dictionary = ArtData.param("ground.map_mode.tint_strength", {})
	var strength: float = float(strengths.get("0.5" if _res_factor == 2 else "0.25", 0.0)) if _res_factor > 1 else 0.0
	if _tone == "winter":
		strength *= float(ArtData.param("ground.map_mode.winter_strength", 0.25))
	return clampf(strength, 0.0, 1.0)


## One colour per terrain tile plus its neighbours: filtering crosses chunk borders without a seam. Alpha opts
## a tile into the map tint; built floors and water opt out. Only the ground chunk shader consumes this texture.
func _terrain_tint(c: Vector2i, canopy: bool = false) -> ImageTexture:
	var size: int = _baker.chunk_tiles
	var img: Image = Image.create(size + 2, size + 2, false, Image.FORMAT_RGBA8)
	var mean_max: float = clampf(float(ArtData.param("ground.tone.mean_max", 0.38)), 0.0, 1.0)
	img.fill(Color(mean_max * 0.9, mean_max * 0.9, mean_max * 0.9, 0.0))
	var palette: Array[Color] = _palette()
	for y: int in range(-1, size + 1):
		for x: int in range(-1, size + 1):
			var tx: int = c.x * size + x
			var ty: int = c.y * size + y
			var index: int = _map.at(tx, ty)
			if index < 0 or index >= ground_ids.size() or _map.is_crisp(tx, ty):
				continue
			# A ¼× canopy tile opts out of the tint (alpha 0) but keeps its ground's colour, so the linear filter
			# does not pull the neutral fill into the meadow around a forest.
			img.set_pixel(x + 1, y + 1, Color(palette[index], 0.0) if canopy and _map.tree_at(tx, ty) > 0 else palette[index])
	return ImageTexture.create_from_image(img)


func _canopy_enabled() -> bool:
	return bool(ArtData.param("trees.quarter_canopy", true))


## Whether the chunk holding tile `cell` is drawn now with the baked ¼× canopy (its tree sprites are hidden).
func canopy_baked_at(cell: Vector2i) -> bool:
	if _map == null or not _canopy_enabled():
		return false
	var size: int = _baker.chunk_tiles
	var c: Vector2i = Vector2i(floori(float(cell.x) / size), floori(float(cell.y) / size))
	return _chunks.has(c) and bool(_canopy_of.get(c, false))


## The tree tiles (GroundMap.tree = species style + 1) from get_trees when its version moves, and the season's
## canopy styles when the drawn season changes; the ¼× chunks that change are rebaked.
func _update_trees() -> void:
	if _map == null:
		return
	var versions: Dictionary = SimReads.read(_world_sim, "get_colony_versions")
	var version: int = int(versions.get("trees_version", -1))
	var dirty: Dictionary = {}
	var size: int = _baker.chunk_tiles
	if version != _trees_version:
		var styles: Dictionary = _species_styles()
		var trees: Dictionary = SimReads.read(_world_sim, "get_trees")
		var xs: PackedInt32Array = trees.get("xs", PackedInt32Array())
		var ys: PackedInt32Array = trees.get("ys", PackedInt32Array())
		var species: PackedStringArray = trees.get("species", PackedStringArray())
		if xs.size() != ys.size() or xs.size() != species.size() or species.has(""):
			push_error("get_trees requires aligned nonempty species identities")
			return
		_trees_version = version
		for identity: String in species:
			if not styles.has(identity):
				_canopy_species.append(identity)
				styles[identity] = _canopy_species.size()
				_canopy_cache.clear()
				_canopy_season = ""
		var now: Dictionary = {}
		for i: int in xs.size():
			var t: Vector2i = Vector2i(xs[i], ys[i])
			if t.x < 0 or t.y < 0 or t.x >= _w or t.y >= _h:
				continue
			now[t] = int(styles[species[i]])
		for t: Vector2i in _union(_tree_cells, now):
			if int(_tree_cells.get(t, 0)) != int(now.get(t, 0)):
				_map.tree[t.y * _w + t.x] = int(now.get(t, 0))
				_mark(dirty, t.x, t.y, size)
		_tree_cells = now
	var season: String = _season_for_canopy()
	if season != _canopy_season:
		_canopy_season = season
		_prepare_canopies(season)
		for c: Vector2i in _chunks.keys():
			dirty[c] = true
	for c: Vector2i in dirty.keys():
		if _chunks.has(c) and (int(_res_of.get(c, 1)) == 4 or _tone == "winter"):
			_bake_chunk(c)


## Species id → canopy style (1-based, bindings `trees.species` order).
func _species_styles() -> Dictionary:
	if _canopy_species.is_empty():
		for row: Variant in (ArtData.bindings().get("trees", {}) as Dictionary).get("species", []):
			if row is Dictionary:
				_canopy_species.append(str((row as Dictionary)["species"]))
	var out: Dictionary = {}
	for i: int in _canopy_species.size():
		out[_canopy_species[i]] = i + 1
	return out


## The season the ¼× canopy shows: the tree layer's season pair, switching at half blend.
func _season_for_canopy() -> String:
	var trees: Node = get_node_or_null("../TreeLayer")
	if trees == null:
		return ""
	var pair: PackedStringArray = trees.call("season_pair")
	if pair.size() != 2:
		return ""
	return pair[1] if float(trees.call("season_blend")) >= 0.5 else pair[0]


## One canopy style per bound species in `season` (see the header), and its edge copy (`trees.canopy_edge_value`).
func _prepare_canopies(season: String) -> void:
	if _canopy_cache.has(season):
		var cached: Array = _canopy_cache[season]
		_baker.set_canopies(cached[0], cached[1])
		return
	var images: Array[Image] = []
	var edges: Array[Image] = []
	var cfg: Dictionary = ArtData.bindings().get("trees", {})
	var habit: String = str(cfg.get("habit", ""))
	var edge_value: float = clampf(float(ArtData.param("trees.canopy_edge_value", 0.7)), 0.0, 1.0)
	for identity: String in _canopy_species:
		var r: Dictionary = _flora.resolve(identity, TreeLayer.SIZE_LARGE, season, 0, habit) if season != "" else {}
		var src: Image = ArtData.image_at(FloraResolver.res_path(str(r.get("map_canopy_path", "")))) if not r.is_empty() else null
		if src == null:
			src = Image.create(TILE, TILE, false, Image.FORMAT_RGBA8)
			src.fill(Color(0.2, 0.3, 0.15))
		var w: int = src.get_width()
		var h: int = src.get_height()
		var small: Image = src.duplicate()
		small.convert(Image.FORMAT_RGBA8)
		small.resize(maxi(1, w >> 2), maxi(1, h >> 2), Image.INTERPOLATE_BILINEAR)
		if not r.is_empty():
			var tint: Dictionary = r["tint"]
			if str(tint["frame"]) != "foliage":
				# Leafless: the bare frame's mean colour, modulated by the canopy texture's own luminance (texture kept).
				var bare: Image = ArtData.image_at(FloraResolver.res_path(str(r["path"])))
				small = _bare_mass(small, Tone.mean_rgb(bare, 1) if bare != null else Tone.mean_rgb(small, 1))
			else:
				# The season's CIELAB shift per pixel, exactly as the 1× sprites get it (seasons.py), on the ¼ copy.
				var d: Array = tint["delta_lab"]
				small = Tone.lab_shift(small, float(d[0]), float(d[1]), float(d[2]))
		var data: PackedByteArray = small.get_data()
		var dark: PackedByteArray = data.duplicate()
		for i: int in range(0, data.size(), 4):
			for k: int in 3:
				dark[i + k] = int(roundf(data[i + k] * edge_value))
			data[i + 3] = 255
			dark[i + 3] = 255
		for pair: Array in [[data, images], [dark, edges]]:
			var img: Image = Image.create_from_data(small.get_width(), small.get_height(), false, Image.FORMAT_RGBA8, pair[0])
			img.resize(w, h, Image.INTERPOLATE_NEAREST)
			(pair[1] as Array).append(img)
	_canopy_cache[season] = [images, edges]
	_baker.set_canopies(images, edges)


## `img` recoloured to `colour` scaled by each pixel's luminance relative to the image's mean (a leafless mass).
static func _bare_mass(img: Image, colour: Color) -> Image:
	var out: Image = img.duplicate()
	var mean: float = maxf(1e-6, float(Tone.metrics(out)["mean"]))
	var data: PackedByteArray = out.get_data()
	for i: int in range(0, data.size(), 4):
		var l: float = (Tone.LR * data[i] + Tone.LG * data[i + 1] + Tone.LB * data[i + 2]) / 255.0 / mean
		data[i] = int(roundf(clampf(colour.r * l, 0.0, 1.0) * 255.0))
		data[i + 1] = int(roundf(clampf(colour.g * l, 0.0, 1.0) * 255.0))
		data[i + 2] = int(roundf(clampf(colour.b * l, 0.0, 1.0) * 255.0))
	return Image.create_from_data(out.get_width(), out.get_height(), false, Image.FORMAT_RGBA8, data)


## The map-mode colour per ground index (`ground.map_mode.tint_by_ground`, held under the tone caps); alpha 0 for a
## ground with no declared colour.
func _palette() -> Array[Color]:
	var mean_max: float = clampf(float(ArtData.param("ground.tone.mean_max", 0.38)), 0.0, 1.0)
	var saturation_max: float = clampf(float(ArtData.param("ground.tone.saturation_max", 0.30)), 0.0, 1.0)
	var neutral: Color = Color(mean_max * 0.9, mean_max * 0.9, mean_max * 0.9, 0.0)
	var colours: Dictionary = ArtData.param("ground.map_mode.tint_by_ground", {})
	var palette: Array[Color] = []
	for id: String in ground_ids:
		var rgb: Variant = colours.get(id, null)
		var colour: Color = neutral
		if rgb is Array and (rgb as Array).size() == 3:
			colour = Color(clampf(float(rgb[0]), 0.0, 1.0), clampf(float(rgb[1]), 0.0, 1.0), clampf(float(rgb[2]), 0.0, 1.0))
			colour = Color.from_hsv(colour.h, minf(colour.s, saturation_max), colour.v)
			var luminance: float = Tone.LR * colour.r + Tone.LG * colour.g + Tone.LB * colour.b
			var value: float = minf(1.0, mean_max / maxf(luminance, 1e-12))
			colour = Color(colour.r * value, colour.g * value, colour.b * value, 1.0)
		var lift: float = maxf(0.0, float(ArtData.param("ground.tone.mean_min", 0.28)) - colour.get_luminance())
		palette.append(Color(colour.r + lift, colour.g + lift, colour.b + lift, colour.a))
	return palette


## `img` reduced by `f` (1 = itself): below 1× a chunk is drawn at the screen's resolution (½ or ¼ size, scaled
## back up by the sprite), so the cache costs a quarter or a sixteenth of the memory and minification does not alias.
func _reduced(img: Image, f: int) -> Image:
	if f <= 1:
		return img
	var small: Image = img.duplicate()
	small.resize(maxi(1, floori(float(img.get_width()) / f)), maxi(1, floori(float(img.get_height()) / f)), Image.INTERPOLATE_BILINEAR)
	return small


## The texture resolution factor for the camera zoom: 1 at 1× and above, 2 at ½×, 4 at ¼×.
func _factor_for_zoom() -> int:
	var camera: Camera2D = get_viewport().get_camera_2d()
	var z: float = camera.zoom.x if camera != null else 1.0
	return 1 if z >= 0.99 else (2 if z >= 0.49 else 4)


func _door_texture(info: Dictionary, open: bool) -> Texture2D:
	var key: String = "%s|%s|%s|%s" % [str(info.get("style", "")), str(info.get("material", "")), str(info.get("category", "")), str(open)]
	if _door_tex.has(key):
		return _door_tex[key]
	var r: Dictionary = _resolver.resolve(str(info.get("style", "")), info.get("material", ""), str(info.get("category", "")), "door")
	var tint: Dictionary = r.get("tint", {})
	var img: Image = ArtData.image_at(MaterialResolver.res_path(str((r.get("textures", {}) as Dictionary).get("door_open" if open else "door_closed", ""))))
	var tex: Texture2D = null
	if img != null:
		tex = ImageTexture.create_from_image(Tone.lab_shift(img, float(tint.get("dL", 0)), float(tint.get("da", 0)), float(tint.get("db", 0))))
	_door_tex[key] = tex
	return tex


func _update_doors() -> void:
	if _world == null:
		return
	var agents: Dictionary = SimReads.read(_world_sim, "get_agent_snapshot")
	var axs: PackedInt32Array = agents.get("xs", PackedInt32Array())
	var ays: PackedInt32Array = agents.get("ys", PackedInt32Array())
	var occupied: Dictionary = {}
	for i: int in mini(axs.size(), ays.size()):
		occupied[Vector2i(axs[i], ays[i])] = true
	for t: Vector2i in _doors.keys():
		if not _walls.openings.has(t):
			(_doors[t] as Node).queue_free()
			_doors.erase(t)
	for t: Vector2i in _walls.openings.keys():
		var sprite: Sprite2D = _doors.get(t, null)
		if sprite == null:
			sprite = Sprite2D.new()
			sprite.centered = false
			sprite.position = Vector2(t.x * TILE, t.y * TILE + View.BASE.y)
			sprite.offset = Vector2(0, -(WallBaker.CAP + View.BASE.y))
			sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
			_world.add_child(sprite)
			_doors[t] = sprite
		sprite.texture = _door_texture(_walls.openings[t], occupied.has(t))
