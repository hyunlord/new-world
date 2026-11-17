# textures3d.gd — presentation-only material texture bindings and shared atlas setup.
# Bridge methods called: none. Unknown materials keep their authored colour without an atlas tile.
extends RefCounted

static var _bindings: Dictionary = {}
static var _meshes: Dictionary = {}


static func bindings() -> Dictionary:
	if _bindings.is_empty():
		_bindings = JSON.parse_string(FileAccess.get_file_as_string("res://assets/3d/textures3d.json"))
	return _bindings


static func index(family: String, key: String, fallback: int = -1) -> int:
	return int((bindings().get(family, {}) as Dictionary).get(key, fallback))


static func setup(mat: ShaderMaterial) -> void:
	var means: PackedFloat32Array = PackedFloat32Array()
	means.resize(64)
	means.fill(0.5)
	var configured: Array = bindings().get("linear_luminance_means", [])
	for i: int in mini(configured.size(), means.size()):
		means[i] = clampf(float(configured[i]), 0.0, 1.0)
	mat.set_shader_parameter("material_luminance", means)
	var path: String = "res://assets/3d/textures/albedo_atlas.webp"
	if ResourceLoader.exists(path):
		mat.set_shader_parameter("material_atlas", load(path) as Texture2D)
		mat.set_shader_parameter("texture_strength", float(bindings().get("strength", 0.5)))
		mat.set_shader_parameter("texture_repeat_scale", float(bindings().get("repeat_scale", 1.0)))
	var normal_path: String = "res://assets/3d/textures/normal_atlas.webp"
	if ResourceLoader.exists(normal_path):
		mat.set_shader_parameter("material_normal_atlas", load(normal_path) as Texture2D)


static func imported(original: Material, model_id: String = "") -> Material:
	if not original is BaseMaterial3D:
		return original
	var key: String = original.resource_name.get_slice(".", 0)
	var overrides: Dictionary = (bindings().get("model_surfaces", {}) as Dictionary).get(model_id, {})
	var tile: int = int(overrides.get(key, index("surfaces", key)))
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
	mat.set_shader_parameter("vertex_srgb", source.vertex_color_is_srgb)
	mat.set_shader_parameter("texture_tile", float(tile))
	setup(mat)
	return mat


## Adapt only the consumer's mesh; Flora/Plants keep their authored source material recolouring contracts.
static func mesh(source: Mesh, model_id: String) -> Mesh:
	var key: String = str(source.get_instance_id()) + "|" + model_id
	if _meshes.has(key):
		return _meshes[key]
	var result: Mesh = source.duplicate() as Mesh
	if result is ArrayMesh:
		for s: int in result.get_surface_count():
			(result as ArrayMesh).surface_set_material(s, imported(source.surface_get_material(s), model_id))
	elif result is PrimitiveMesh:
		(result as PrimitiveMesh).material = imported((source as PrimitiveMesh).material, model_id)
	_meshes[key] = result
	return result
