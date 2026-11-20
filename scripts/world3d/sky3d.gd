# World3D/Sky. Drives the sun, seasons and blended colour grade from the simulation clock.
# Bridge methods: SimReads get_time_snapshot and get_ui_facts; direct get_chronicle_since.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")
const CHRONICLE_DIED: int = 11

var sun: DirectionalLight3D
var environment: Environment
var _world_sim: Node = null
var _terrain: Node = null
var _night: float = 0.0
var _season: int = -1
var _hour_override: float = -1.0
## Negative uses the normal time-of-day sun path.
var slice_day_elevation: float = -1.0
var _hardship_override: float = -1.0
var _hardship: float = 0.0
var _target: float = 0.0
var _last_tick: int = -1
var _sample_tick: int = -1
var _cursor: int = -1
var _deaths: Array[int] = []
var _lut: Gradient = Gradient.new()
var _lut_key: Vector3i = Vector3i(-1, -1, -1)
var _profiles: Dictionary
var _hardship_data: Dictionary
var _sky_data: Dictionary
var _dawn: Array
var _dusk: Array
var _shadow_distance: float
var _previous_hour: float = -1.0
var _previous_hardship: float = -1.0
var _previous_grade: float = 1.0
var _grade_camera: Camera2D
var _grade_zoom: Vector2
var _map_light: Dictionary
var _s2_day: Dictionary = {}


func setup(world_sim: Node, terrain: Node) -> void:
	_world_sim = world_sim
	_terrain = terrain
	if OS.get_cmdline_user_args().has("--style=s2"):
		_grade_camera = get_parent().get("camera2d") as Camera2D
		_grade_zoom = Vector2(float(Data3D.mood("style2.grade_zoom_neutral", 0.25)), float(Data3D.mood("style2.grade_zoom_full", 0.75)))
		_map_light = Data3D.param("light", {})
		_s2_day = Data3D.mood("style2.day", {})
	if _world_sim != null:
		_world_sim.connect("world_reset", _reset_hardship)


func _ready() -> void:
	_profiles = Data3D.mood("grade", {})
	_hardship_data = Data3D.mood("hardship", {})
	_sky_data = Data3D.mood("sky", {})
	_dawn = Data3D.param("light.dawn_hours", [5.0, 7.0])
	_dusk = Data3D.param("light.dusk_hours", [18.0, 20.0])
	_shadow_distance = float(Data3D.param("light.shadow_distance", 90.0))
	environment = Environment.new()
	var sky_material: ShaderMaterial = ShaderMaterial.new()
	sky_material.shader = preload("res://shaders/world3d/sky.gdshader")
	sky_material.set_shader_parameter("sky_top", Data3D.colour(_sky_data["top"]))
	sky_material.set_shader_parameter("sky_horizon", Data3D.colour(_sky_data["horizon"]))
	for key: String in ["ground_horizon", "ground_bottom", "cloud_colour"]:
		sky_material.set_shader_parameter(key, Data3D.colour(_sky_data[key]))
	for key: String in ["cloud_scale", "cloud_opacity"]:
		sky_material.set_shader_parameter(key, float(_sky_data[key]))
	var thresholds: Array = _sky_data["cloud_thresholds"]
	sky_material.set_shader_parameter("cloud_thresholds", Vector2(float(thresholds[0]), float(thresholds[1])))
	var sky_res: Sky = Sky.new()
	sky_res.sky_material = sky_material
	environment.background_mode = Environment.BG_SKY
	environment.sky = sky_res
	environment.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	environment.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	environment.tonemap_white = float(_sky_data["tonemap_white"])
	environment.adjustment_enabled = true
	var correction: GradientTexture1D = GradientTexture1D.new()
	correction.gradient = _lut
	environment.adjustment_color_correction = correction
	environment.ssao_enabled = true
	environment.ssao_radius = float(_sky_data["ssao_radius"])
	environment.ssao_intensity = float(_sky_data["ssao_intensity"])
	environment.ssao_light_affect = float(_sky_data["ssao_light_affect"])
	environment.fog_enabled = true
	environment.fog_sky_affect = float(_sky_data["fog_sky_affect"])
	environment.fog_height = float(_sky_data["fog_height"])
	environment.fog_height_density = float(_sky_data["fog_height_density"])
	environment.glow_enabled = true
	environment.glow_intensity = 0.6
	environment.glow_hdr_threshold = 1.2
	var we: WorldEnvironment = WorldEnvironment.new()
	we.environment = environment
	add_child(we)
	sun = DirectionalLight3D.new()
	sun.name = "Sun"
	sun.shadow_enabled = true
	sun.shadow_blur = 2.0
	sun.shadow_bias = 0.04
	sun.shadow_normal_bias = 1.2
	sun.directional_shadow_blend_splits = true
	# Bigger atlas and soft PCF: clean shadow edges without acne or stair-stepping.
	RenderingServer.directional_shadow_atlas_set_size(int(Data3D.param("light.shadow_atlas", 8192)), true)
	RenderingServer.directional_soft_shadow_filter_set_quality(RenderingServer.SHADOW_QUALITY_SOFT_HIGH)
	sun.directional_shadow_mode = DirectionalLight3D.SHADOW_PARALLEL_2_SPLITS
	sun.directional_shadow_max_distance = _shadow_distance
	add_child(sun)
	_apply_hour(12.0, 0.0)


## Pin the hour (tools: captures at a fixed light); -1 follows the clock.
func set_hour_override(hour: float) -> void:
	_hour_override = hour


## Debug capture override, or a negative value to follow the settlement.
func set_hardship_override(value: float) -> void:
	if OS.is_debug_build():
		_hardship_override = minf(value, 1.0)


func night_factor() -> float:
	return _night


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	# Shadows only as far as the view needs (the camera distance scales with the zoom).
	var cam: Camera3D = get_viewport().get_camera_3d()
	if cam != null:
		sun.directional_shadow_max_distance = clampf(cam.global_position.y * 2.2, 25.0, _shadow_distance)
	var time: Dictionary = SimReads.read(_world_sim, "get_time_snapshot")
	var hour: float = _hour_override if _hour_override >= 0.0 else float(time.get("time_of_day", 12.0))
	var season: int = int(time.get("season", 0))
	_update_hardship(time, season)
	var hardship: float = _hardship_override if _hardship_override >= 0.0 else _hardship
	var grade: float = 1.0
	if _grade_camera != null:
		grade = clampf(inverse_lerp(_grade_zoom.x, _grade_zoom.y, _grade_camera.zoom.x), 0.0, 1.0)
	if hour != _previous_hour or hardship != _previous_hardship or grade != _previous_grade:
		_apply_hour(hour, hardship, grade)
	if season != _season and _terrain != null:
		_season = season
		var mat: ShaderMaterial = _terrain.get("material") as ShaderMaterial
		if mat != null:
			var tint: Color = Data3D.colour((Data3D.param("terrain.season_tint", {}) as Dictionary).get(str(season), []), Color.WHITE)
			mat.set_shader_parameter("season_tint", Vector3(tint.r, tint.g, tint.b))
			mat.set_shader_parameter("snow", float((Data3D.param("terrain.snow_cover", {}) as Dictionary).get(str(season), 0.0)))
			mat.set_shader_parameter("autumn", 1.0 if season == 2 else 0.0)
		# Roofs and plants read the same cover (global `w3d_snow`).
		RenderingServer.global_shader_parameter_set("w3d_snow", float((Data3D.param("terrain.snow_cover", {}) as Dictionary).get(str(season), 0.0)))


func _reset_hardship() -> void:
	_last_tick = -1
	_sample_tick = -1
	_cursor = -1
	_deaths.clear()
	_season = -1


func _update_hardship(time: Dictionary, season: int) -> void:
	var tick: int = int(time.get("tick", 0))
	var ticks_per_day: int = maxi(1, int(time.get("ticks_per_day", 1)))
	if tick == _last_tick:
		return
	if tick < _last_tick:
		_reset_hardship()
	if _sample_tick < 0 or tick - _sample_tick >= int(ticks_per_day * float(_hardship_data["sample_days"])) or season != _season:
		_sample_tick = tick
		var facts: Dictionary = SimReads.read(_world_sim, "get_ui_facts")
		var food: Dictionary = facts.get("food_chain", {})
		var daily: float = float(food.get("daily_milli", 0))
		var food_hardship: float = 0.0
		if bool(food.get("found", false)) and daily > 0.0:
			var days: float = float(food.get("stored_units", 0)) * 1000.0 / daily
			food_hardship = clampf(1.0 - days / float(_hardship_data["food_days"]), 0.0, 1.0)
		var since: int = tick - int(ticks_per_day * float(_hardship_data["death_days"]))
		_read_deaths(since)
		var death_hardship: float = minf(1.0, _deaths.size() * float(_hardship_data["per_death"]))
		var winter: float = float(_hardship_data["winter"]) if season == 3 else 0.0
		_target = clampf(maxf(food_hardship, death_hardship) + winter, 0.0, 1.0)
	if _last_tick < 0:
		_hardship = _target
	else:
		var days_elapsed: float = float(tick - _last_tick) / ticks_per_day
		_hardship = lerpf(_hardship, _target, 1.0 - exp(-days_elapsed / float(_hardship_data["smooth_days"])))
	_last_tick = tick


func _read_deaths(since: int) -> void:
	if not _world_sim.has_method("get_chronicle_since"):
		return
	if _cursor < 0:
		# A loaded late-game save only needs the recent tail of its chronicle.
		var first: int = 0
		var end: int = int(_world_sim.get_chronicle_since(0, 1).get("total", 0))
		while first < end:
			@warning_ignore("integer_division")
			var mid: int = (first + end) / 2
			var probe: PackedInt64Array = _world_sim.get_chronicle_since(mid, 1).get("tick", PackedInt64Array())
			if not probe.is_empty() and probe[0] < since:
				first = mid + 1
			else:
				end = mid
		_cursor = first
	while true:
		var page: Dictionary = _world_sim.get_chronicle_since(_cursor, 512)
		var ticks: PackedInt64Array = page.get("tick", PackedInt64Array())
		var kinds: PackedInt32Array = page.get("kind", PackedInt32Array())
		for i: int in mini(ticks.size(), kinds.size()):
			if kinds[i] == CHRONICLE_DIED and ticks[i] >= since:
				_deaths.append(ticks[i])
		var next: int = int(page.get("next", _cursor))
		if next <= _cursor:
			break
		_cursor = next
		if _cursor >= int(page.get("total", 0)):
			break
	while not _deaths.is_empty() and _deaths[0] < since:
		_deaths.pop_front()


func _apply_hour(hour: float, hardship: float, grade: float = 1.0) -> void:
	var grade_changed: bool = grade != _previous_grade
	_previous_hour = hour
	_previous_hardship = hardship
	_previous_grade = grade
	var day: float = 1.0
	if hour < float(_dawn[0]) or hour >= float(_dusk[1]):
		day = 0.0
	elif hour < float(_dawn[1]):
		day = inverse_lerp(float(_dawn[0]), float(_dawn[1]), hour)
	elif hour >= float(_dusk[0]):
		day = inverse_lerp(float(_dusk[1]), float(_dusk[0]), hour)
	_night = 1.0 - day
	RenderingServer.global_shader_parameter_set("w3d_night", _night)
	# The existing sun path crosses the south; the moon holds its direction at night.
	var frac: float = clampf((hour - 6.0) / 12.0, -0.2, 1.2)
	var peak: float = float(_sky_data["sun_max_elevation"])
	if grade < 1.0:
		peak = lerpf(58.0, peak, grade)
	var elevation: float = deg_to_rad(lerpf(float(_sky_data["sun_min_elevation"]), peak, sin(clampf(frac, 0.0, 1.0) * PI)))
	var azimuth: float = lerpf(-PI * 0.5, PI * 0.5, frac)
	if slice_day_elevation >= 0.0 and day >= 1.0:
		elevation = deg_to_rad(slice_day_elevation)
		azimuth = deg_to_rad(-32.0)
	elif day <= 0.0:
		elevation = deg_to_rad(float(_sky_data["moon_elevation"]))
		azimuth = deg_to_rad(float(_sky_data["moon_azimuth"]))
	if not _s2_day.is_empty():
		elevation *= lerpf(1.0, float(_s2_day["elevation_scale"]), smoothstep(0.6, 1.0, day) * grade)
	var to_sun: Vector3 = Vector3(sin(azimuth) * cos(elevation), sin(elevation), cos(azimuth) * cos(elevation))
	sun.look_at_from_position(Vector3.ZERO, -to_sun, Vector3.UP)
	var steps: int = maxi(1, int(_profiles["lut_steps"]))
	var key: Vector3i = Vector3i(roundi(day * steps), roundi(hardship * steps), 0)
	if key == _lut_key and not grade_changed:
		return
	_lut_key = key
	var daylight: float = float(key.x) / steps
	var stress: float = float(key.y) / steps
	var a: Dictionary = _profiles["night"] if daylight < 0.5 else _profiles["dusk"]
	var b: Dictionary = _profiles["dusk"] if daylight < 0.5 else _profiles["day"]
	var grim: Dictionary = _profiles["hardship"]
	var blend: float = fposmod(daylight * 2.0, 1.0) if daylight < 1.0 else 1.0
	var stress_blend: float = stress * smoothstep(0.5, 1.0, daylight)
	sun.light_energy = _number(a, b, grim, "sun_energy", blend, stress_blend)
	sun.light_color = _colour(a, b, grim, "sun", blend, stress_blend)
	environment.ambient_light_energy = _number(a, b, grim, "ambient_energy", blend, stress_blend)
	environment.ambient_light_color = _colour(a, b, grim, "ambient", blend, stress_blend)
	environment.fog_density = _number(a, b, grim, "fog_density", blend, stress_blend)
	environment.fog_light_color = _colour(a, b, grim, "fog", blend, stress_blend)
	environment.background_energy_multiplier = _number(a, b, grim, "sky_energy", blend, stress_blend)
	environment.adjustment_saturation = _number(a, b, grim, "saturation", blend, stress_blend)
	environment.adjustment_contrast = _number(a, b, grim, "contrast", blend, stress_blend)
	environment.tonemap_exposure = _number(a, b, grim, "exposure", blend, stress_blend)
	if not _s2_day.is_empty():
		var warmth: float = smoothstep(0.6, 1.0, daylight) * (1.0 - stress_blend)
		sun.light_color = sun.light_color.lerp(Data3D.colour(_s2_day["sun"]), warmth)
		environment.ambient_light_energy = lerpf(environment.ambient_light_energy, float(_s2_day["ambient_energy"]), warmth)
		environment.ambient_light_color = environment.ambient_light_color.lerp(Data3D.colour(_s2_day["ambient"]), warmth)
		environment.fog_light_color = environment.fog_light_color.lerp(Data3D.colour(_s2_day["fog"]), warmth)
		var sky_material: ShaderMaterial = environment.sky.sky_material as ShaderMaterial
		for sky_key: String in ["top", "horizon"]:
			var neutral: Color = Data3D.colour(_sky_data[sky_key])
			sky_material.set_shader_parameter("sky_" + sky_key, neutral.lerp(Data3D.colour(_s2_day["sky_" + sky_key]), warmth))
	if _grade_camera != null:
		_blend_map_grade(day, frac, grade)
	var offsets: PackedFloat32Array = []
	var colours: PackedColorArray = []
	var points: int = (a["lut"] as Array).size()
	for i: int in points:
		var x: float = float(i) / (points - 1)
		var tint: Color = Data3D.colour(a["lut"][i]).lerp(Data3D.colour(b["lut"][i]), blend).lerp(Data3D.colour(grim["lut"][i]), stress_blend)
		offsets.append(x)
		colours.append(Color(x, x, x).lerp(tint, grade) if grade < 1.0 else tint)
	_lut.offsets = offsets
	_lut.colors = colours


## MOOD1 R4 map endpoint, using the same 0.25–0.75 zoom interpolation as the LUT.
func _blend_map_grade(day: float, frac: float, grade: float) -> void:
	environment.fog_height_density = float(_sky_data["fog_height_density"]) * grade
	environment.ssao_intensity = lerpf(1.8, float(_sky_data["ssao_intensity"]), grade)
	environment.ssao_light_affect = lerpf(0.15, float(_sky_data["ssao_light_affect"]), grade)
	if grade >= 1.0:
		return
	var map_sun: float = lerpf(float(_map_light["night_sun_energy"]), float(_map_light["sun_energy"]), day)
	var warm: float = clampf(1.0 - absf(frac - 0.5) * 2.0, 0.0, 1.0)
	var map_sun_color: Color = Color(1.0, 0.78, 0.6).lerp(Color(1.0, 0.97, 0.9), warm).lerp(Color(0.6, 0.7, 1.0), _night)
	var map_ambient: float = lerpf(float(_map_light["night_ambient_energy"]), float(_map_light["ambient_energy"]), day)
	var map_ambient_color: Color = Data3D.colour(_map_light["night_ambient"]).lerp(Data3D.colour(_map_light["day_ambient"]), day)
	sun.light_energy = lerpf(map_sun, sun.light_energy, grade)
	sun.light_color = map_sun_color.lerp(sun.light_color, grade)
	environment.ambient_light_energy = lerpf(map_ambient, environment.ambient_light_energy, grade)
	environment.ambient_light_color = map_ambient_color.lerp(environment.ambient_light_color, grade)
	environment.fog_density = lerpf(float(_map_light["fog_density"]), environment.fog_density, grade)
	environment.fog_light_color = Color(0.12, 0.14, 0.22).lerp(Color(0.82, 0.78, 0.70), day).lerp(environment.fog_light_color, grade)
	environment.background_energy_multiplier = lerpf(lerpf(0.12, 1.0, day), environment.background_energy_multiplier, grade)
	environment.adjustment_saturation = lerpf(float(_map_light["saturation"]), environment.adjustment_saturation, grade)
	environment.adjustment_contrast = lerpf(1.03, environment.adjustment_contrast, grade)
	environment.tonemap_exposure = lerpf(1.0, environment.tonemap_exposure, grade)


static func _number(a: Dictionary, b: Dictionary, grim: Dictionary, key: String, blend: float, stress: float) -> float:
	return lerpf(lerpf(float(a[key]), float(b[key]), blend), float(grim[key]), stress)


static func _colour(a: Dictionary, b: Dictionary, grim: Dictionary, key: String, blend: float, stress: float) -> Color:
	return Data3D.colour(a[key]).lerp(Data3D.colour(b[key]), blend).lerp(Data3D.colour(grim[key]), stress)
