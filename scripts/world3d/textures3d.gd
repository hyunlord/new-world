# textures3d.gd — presentation-only material texture bindings and shared atlas setup.
# Bridge methods called: none. Unknown materials retain their existing flat colour.
extends RefCounted

static var _bindings: Dictionary = {}


static func bindings() -> Dictionary:
	if _bindings.is_empty():
		_bindings = JSON.parse_string(FileAccess.get_file_as_string("res://assets/3d/textures3d.json"))
	return _bindings


static func index(family: String, key: String, fallback: int = -1) -> int:
	return int((bindings().get(family, {}) as Dictionary).get(key, fallback))


static func setup(mat: ShaderMaterial) -> void:
	var data: GDScript = load("res://scripts/world3d/data3d.gd") as GDScript
	var look: Dictionary = data.mood("detail.surface", {})
	mat.set_shader_parameter("material_scale", float(look.get("texture_scale", 1.0)))
	mat.set_shader_parameter("material_normal_strength", float(look.get("normal_strength", 0.15)))
	mat.set_shader_parameter("material_grain_height", float(look.get("grain_height", 0.036)))
	var path: String = "res://assets/3d/textures/albedo_atlas.webp"
	if ResourceLoader.exists(path):
		mat.set_shader_parameter("material_atlas", load(path) as Texture2D)
		mat.set_shader_parameter("texture_strength", float(bindings().get("strength", 0.5)))
	var normal_path: String = "res://assets/3d/textures/normal_atlas.webp"
	if ResourceLoader.exists(normal_path):
		mat.set_shader_parameter("material_normal_atlas", load(normal_path) as Texture2D)


static func imported(original: Material, model_id: String = "") -> Material:
	if not original is BaseMaterial3D:
		return original
	var key: String = original.resource_name.get_slice(".", 0)
	var overrides: Dictionary = (bindings().get("model_surfaces", {}) as Dictionary).get(model_id, {})
	var tile: int = int(overrides.get(key, index("surfaces", key)))
	if tile < 0:
		return original
	var source: BaseMaterial3D = original
	# Preserve special imported rendering paths (glow, alpha, front-face culling, existing maps).
	if source.transparency != BaseMaterial3D.TRANSPARENCY_DISABLED or source.emission_enabled or source.cull_mode == BaseMaterial3D.CULL_FRONT or source.albedo_texture != null or source.shading_mode != BaseMaterial3D.SHADING_MODE_PER_PIXEL:
		return original
	var mat: ShaderMaterial = ShaderMaterial.new()
	mat.shader = preload("res://shaders/world3d/model_material_twosided.gdshader") if source.cull_mode == BaseMaterial3D.CULL_DISABLED else preload("res://shaders/world3d/model_material.gdshader")
	mat.resource_name = original.resource_name
	mat.set_shader_parameter("base_colour", source.albedo_color)
	mat.set_shader_parameter("roughness", source.roughness)
	mat.set_shader_parameter("metallic", source.metallic)
	mat.set_shader_parameter("specular", source.metallic_specular)
	mat.set_shader_parameter("vertex_tint", source.vertex_color_use_as_albedo)
	mat.set_shader_parameter("texture_tile", float(tile))
	setup(mat)
	return mat
