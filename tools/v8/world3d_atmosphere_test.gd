# world3d_atmosphere_test.gd — real sky/water resources, clock blends and missing-art fallback.
# Bridge methods called: none. Run headless after importing the T2 atmosphere assets.
extends SceneTree

const SkyView = preload("res://scripts/world3d/sky3d.gd")
const Terrain = preload("res://scripts/world3d/terrain3d.gd")
const Data3D = preload("res://scripts/world3d/data3d.gd")
var _failures: int = 0


func _check(label: String, ok: bool) -> void:
	print("W3D_ATMOSPHERE %s %s" % [label, "PASS" if ok else "FAIL"])
	_failures += 0 if ok else 1


func _init() -> void:
	call_deferred("_run")


func _run() -> void:
	var config: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://assets/3d/atmosphere_t2.json"))
	var sky: Node3D = SkyView.new()
	root.add_child(sky)
	var environment: Environment = sky.get("environment")
	var material: ShaderMaterial = sky.get("_painted_sky")
	_check("generated_sky_loaded", material != null and environment.sky.sky_material == material)
	_check("ambient_source_preserved", environment.ambient_light_source == Environment.AMBIENT_SOURCE_COLOR)
	_check("missing_sky_fallback", sky.call("_create_painted_sky", {}) == null)
	if material != null:
		var sky_code: String = material.shader.code
		_check("sky_overlap_boundary_contract", sky_code.contains("uv.x *= 0.85;") and sky_code.contains("uv.x + 0.85") and sky_code.contains("smoothstep(0.0, 0.15, uv.x)") and not sky_code.contains("abs(SKY_COORDS.x"))
		_check("sky_overlap_premultiplied", sky_code.contains("primary = premultiply(texture(tex, uv))") and sky_code.contains("wrapped = premultiply(texture(tex, vec2(uv.x + 0.85, uv.y)))"))
		for key: String in (config["sky"]["textures"] as Dictionary):
			_check("sky_sampler_" + key, material.get_shader_parameter(key) is Texture2D)
		sky.call("_apply_hour", 12.0)
		_check("day_weights", is_zero_approx((material.get_shader_parameter("blend_weights") as Vector3).x))
		sky.call("_apply_hour", 0.0)
		_check("night_weights", is_equal_approx((material.get_shader_parameter("blend_weights") as Vector3).x, 1.0))
		var dusk: Array = Data3D.param("light.dusk_hours", [18.0, 20.0])
		sky.call("_apply_hour", (float(dusk[0]) + float(dusk[1])) * 0.5)
		_check("dusk_weights", is_equal_approx((material.get_shader_parameter("blend_weights") as Vector3).y, 1.0))
		sky.set("_season", 3)
		sky.call("_apply_hour", 12.0)
		var expected: float = float((Data3D.param("terrain.snow_cover", {}) as Dictionary).get("3", 0.0))
		_check("winter_follows_existing_snow", is_equal_approx((material.get_shader_parameter("blend_weights") as Vector3).z, expected))
	var terrain: Node3D = Terrain.new()
	root.add_child(terrain)
	var water: ShaderMaterial = terrain.get("water_material")
	_check("generated_water_enabled", bool(water.get_shader_parameter("painted_water")))
	_check("water_samplers_loaded", water.get_shader_parameter("water_normal") is Texture2D and water.get_shader_parameter("water_flow") is Texture2D)
	var colour: Color = water.get_shader_parameter("water_colour")
	_check("water_colour_preserved", colour == Data3D.colour(Data3D.param("terrain.water_colour", []), Color(0.16, 0.36, 0.45, 0.82)))
	terrain.call("_setup_water_textures", {})
	_check("missing_water_fallback", not bool(water.get_shader_parameter("painted_water")))
	var incomplete: Dictionary = (config.get("water", {}) as Dictionary).duplicate()
	incomplete.erase("flow")
	terrain.call("_setup_water_textures", incomplete)
	_check("missing_flow_fallback", not bool(water.get_shader_parameter("painted_water")))
	terrain.call("_setup_water_textures", config.get("water", {}))
	_check("water_recovers_with_assets", bool(water.get_shader_parameter("painted_water")))
	terrain.free()
	sky.free()
	print("W3D_ATMOSPHERE_DONE failures=%d" % _failures)
	quit(1 if _failures > 0 else 0)
