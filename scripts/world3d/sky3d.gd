# sky3d.gd (World3D/Sky, made by world3d.gd): light, sky and seasons of the 3D view (M3D-1 scope 9).
# Purpose: a DirectionalLight3D sun driven by the sim clock (get_time_snapshot `time_of_day`): it rises in the east,
#   crosses the south and sets in the west; its energy and the ambient light fade to a dim blue night through
#   params3d `light.dawn_hours` / `light.dusk_hours` (the same hours as the 2D CanvasModulate night); soft PCF shadows
#   up to `light.shadow_distance`. The season (`season`) sets the terrain's season tint and snow cover
#   (params3d `terrain.season_tint`, `terrain.snow_cover`). A procedural sky, filmic tonemapping, SSAO and a light
#   distance fog give depth. `night_factor()` is 0 by day and 1 at full night (lights of lit stations follow it).
# Bridge methods called: via SimReads get_time_snapshot.
extends Node3D

const Data3D = preload("res://scripts/world3d/data3d.gd")

var sun: DirectionalLight3D
var environment: Environment
var _world_sim: Node = null
var _terrain: Node = null
var _night: float = 0.0
var _season: int = -1
var _hour_override: float = -1.0


func setup(world_sim: Node, terrain: Node) -> void:
	_world_sim = world_sim
	_terrain = terrain


func _ready() -> void:
	environment = Environment.new()
	var sky_material: ProceduralSkyMaterial = ProceduralSkyMaterial.new()
	sky_material.sky_top_color = Color(0.36, 0.55, 0.78)
	sky_material.sky_horizon_color = Color(0.72, 0.78, 0.82)
	sky_material.ground_horizon_color = Color(0.55, 0.58, 0.55)
	sky_material.ground_bottom_color = Color(0.3, 0.32, 0.3)
	var sky_res: Sky = Sky.new()
	sky_res.sky_material = sky_material
	environment.background_mode = Environment.BG_SKY
	environment.sky = sky_res
	environment.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	environment.ambient_light_color = Data3D.colour(Data3D.param("light.day_ambient", []), Color(0.8, 0.82, 0.86))
	environment.ambient_light_energy = float(Data3D.param("light.ambient_energy", 0.55))
	environment.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	environment.tonemap_white = 8.0
	environment.adjustment_enabled = true
	environment.adjustment_saturation = 1.15
	environment.adjustment_contrast = 1.06
	environment.ssao_enabled = true
	environment.ssao_radius = 1.2
	environment.ssao_intensity = 1.6
	environment.fog_enabled = true
	environment.fog_light_color = Color(0.7, 0.76, 0.82)
	environment.fog_density = 0.0012
	environment.fog_sky_affect = 0.0
	environment.glow_enabled = true
	environment.glow_intensity = 0.6
	environment.glow_hdr_threshold = 1.2
	var we: WorldEnvironment = WorldEnvironment.new()
	we.environment = environment
	add_child(we)
	sun = DirectionalLight3D.new()
	sun.name = "Sun"
	sun.shadow_enabled = true
	sun.shadow_blur = 1.5
	sun.directional_shadow_mode = DirectionalLight3D.SHADOW_PARALLEL_2_SPLITS
	sun.directional_shadow_max_distance = float(Data3D.param("light.shadow_distance", 90.0))
	sun.light_color = Color(1.0, 0.96, 0.88)
	add_child(sun)


## Pin the hour (tools: captures at a fixed light); -1 follows the clock.
func set_hour_override(hour: float) -> void:
	_hour_override = hour


func night_factor() -> float:
	return _night


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	# Shadows only as far as the view needs (the camera distance scales with the zoom).
	var cam: Camera3D = get_viewport().get_camera_3d()
	if cam != null:
		sun.directional_shadow_max_distance = clampf(cam.global_position.y * 2.2, 25.0, float(Data3D.param("light.shadow_distance", 90.0)))
	var time: Dictionary = SimReads.read(_world_sim, "get_time_snapshot")
	var hour: float = _hour_override if _hour_override >= 0.0 else float(time.get("time_of_day", 12.0))
	_apply_hour(hour)
	var season: int = int(time.get("season", 0))
	if season != _season and _terrain != null:
		_season = season
		var mat: ShaderMaterial = _terrain.get("material") as ShaderMaterial
		if mat != null:
			var tint: Color = Data3D.colour((Data3D.param("terrain.season_tint", {}) as Dictionary).get(str(season), []), Color.WHITE)
			mat.set_shader_parameter("season_tint", Vector3(tint.r, tint.g, tint.b))
			mat.set_shader_parameter("snow", float((Data3D.param("terrain.snow_cover", {}) as Dictionary).get(str(season), 0.0)))
		# Roofs and plants read the same cover (global `w3d_snow`).
		RenderingServer.global_shader_parameter_set("w3d_snow", float((Data3D.param("terrain.snow_cover", {}) as Dictionary).get(str(season), 0.0)))


func _apply_hour(hour: float) -> void:
	var dawn: Array = Data3D.param("light.dawn_hours", [5.0, 7.0])
	var dusk: Array = Data3D.param("light.dusk_hours", [18.0, 20.0])
	var day: float = 1.0
	if hour < float(dawn[0]) or hour >= float(dusk[1]):
		day = 0.0
	elif hour < float(dawn[1]):
		day = (hour - float(dawn[0])) / maxf(0.01, float(dawn[1]) - float(dawn[0]))
	elif hour >= float(dusk[0]):
		day = 1.0 - (hour - float(dusk[0])) / maxf(0.01, float(dusk[1]) - float(dusk[0]))
	_night = 1.0 - day
	# Sun path: azimuth east (6h) → south (12h) → west (18h); elevation peaks at noon. At night a low moon.
	var frac: float = clampf((hour - 6.0) / 12.0, -0.2, 1.2)
	var elevation: float = deg_to_rad(lerpf(8.0, 58.0, sin(clampf(frac, 0.0, 1.0) * PI)))
	var azimuth: float = lerpf(-PI * 0.5, PI * 0.5, frac)
	if day <= 0.0:
		elevation = deg_to_rad(40.0)
		azimuth = deg_to_rad(25.0)
	# Direction the light travels (from the sun toward the ground); south is +Z.
	var to_sun: Vector3 = Vector3(sin(azimuth) * cos(elevation), sin(elevation), cos(azimuth) * cos(elevation))
	sun.look_at_from_position(Vector3.ZERO, -to_sun, Vector3.UP if absf(to_sun.y) < 0.99 else Vector3.FORWARD)
	var day_energy: float = float(Data3D.param("light.sun_energy", 1.25))
	var night_energy: float = float(Data3D.param("light.night_sun_energy", 0.06))
	sun.light_energy = lerpf(night_energy, day_energy, day)
	var warm: float = clampf(1.0 - absf(frac - 0.5) * 2.0, 0.0, 1.0)
	sun.light_color = Color(1.0, 0.78, 0.6).lerp(Color(1.0, 0.97, 0.9), warm).lerp(Color(0.6, 0.7, 1.0), _night)
	var amb_day: Color = Data3D.colour(Data3D.param("light.day_ambient", []), Color(0.8, 0.82, 0.86))
	var amb_night: Color = Data3D.colour(Data3D.param("light.night_ambient", []), Color(0.18, 0.22, 0.38))
	environment.ambient_light_color = amb_night.lerp(amb_day, day)
	environment.ambient_light_energy = lerpf(float(Data3D.param("light.night_ambient_energy", 0.5)), float(Data3D.param("light.ambient_energy", 0.32)), day)
	environment.background_energy_multiplier = lerpf(0.12, 1.0, day)
	environment.fog_light_color = Color(0.12, 0.14, 0.22).lerp(Color(0.7, 0.76, 0.82), day)
