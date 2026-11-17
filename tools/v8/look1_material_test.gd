# look1_material_test.gd — CPU checks for the L1 consumer material adapter and colour-grade cube.
# Purpose: preserve authored material paths and source meshes while sharing painterly shading; check a bounded,
# monotone 32-cube LUT. Run headless; LOOK1_MATERIAL_TEST_DONE reports all failures and exit status is nonzero on failure.
# Bridge methods called: none.
extends SceneTree

const Textures3D = preload("res://scripts/world3d/textures3d.gd")
const Sky3D = preload("res://scripts/world3d/sky3d.gd")
const MODEL_SHADER = preload("res://shaders/world3d/model_material.gdshader")
const TWOSIDED_SHADER = preload("res://shaders/world3d/model_material_twosided.gdshader")

var _failures: int = 0
var _checks: int = 0


func _init() -> void:
	call_deferred("_run")


func _check(name: String, ok: bool) -> void:
	_checks += 1
	print("LOOK1_MATERIAL %s %s" % [name, "PASS" if ok else "FAIL"])
	if not ok:
		_failures += 1


func _run() -> void:
	_materials()
	_special_paths()
	_meshes()
	_grade()
	print("LOOK1_MATERIAL_TEST_DONE failures=%d checks=%d" % [_failures, _checks])
	quit(1 if _failures > 0 else 0)


func _source(surface: String) -> StandardMaterial3D:
	var material: StandardMaterial3D = StandardMaterial3D.new()
	material.resource_name = surface
	material.albedo_color = Color(0.37, 0.53, 0.29, 1.0)
	material.roughness = 0.63
	material.metallic = 0.27
	material.metallic_specular = 0.41
	material.vertex_color_use_as_albedo = true
	material.vertex_color_is_srgb = true
	return material


func _materials() -> void:
	# Given one registered surface and one data-only surface, both with non-default authored properties.
	var surfaces: Dictionary = Textures3D.bindings().get("surfaces", {})
	_check("registered_surface_fixture", not surfaces.is_empty())
	if surfaces.is_empty():
		return
	for name: String in [str(surfaces.keys()[0]), "look1_unknown_surface"]:
		for cull: int in [BaseMaterial3D.CULL_BACK, BaseMaterial3D.CULL_DISABLED]:
			var source: StandardMaterial3D = _source(name)
			source.cull_mode = cull as BaseMaterial3D.CullMode
			# When the consumer adapts the material.
			var adapted: Material = Textures3D.imported(source)
			var label: String = "%s_cull%d" % [name, cull]
			_check(label + "_shared_shader", adapted is ShaderMaterial)
			if not adapted is ShaderMaterial:
				continue
			var shader_material: ShaderMaterial = adapted
			# Then colour, reflectance, tint and culling survive; an unknown surface needs no atlas tile.
			_check(label + "_properties", shader_material.get_shader_parameter("base_colour") == source.albedo_color
				and is_equal_approx(float(shader_material.get_shader_parameter("roughness")), source.roughness)
				and is_equal_approx(float(shader_material.get_shader_parameter("metallic")), source.metallic)
				and is_equal_approx(float(shader_material.get_shader_parameter("specular")), source.metallic_specular)
				and bool(shader_material.get_shader_parameter("vertex_tint"))
				and bool(shader_material.get_shader_parameter("vertex_srgb")))
			_check(label + "_culling", shader_material.shader == (TWOSIDED_SHADER if cull == BaseMaterial3D.CULL_DISABLED else MODEL_SHADER))
			_check(label + "_tile", int(shader_material.get_shader_parameter("texture_tile")) == int(surfaces.get(name, -1)))
			_check(label + "_source_unchanged", source.albedo_color == Color(0.37, 0.53, 0.29, 1.0)
				and is_equal_approx(source.roughness, 0.63) and is_equal_approx(source.metallic, 0.27)
				and source.cull_mode == cull and source.resource_name == name and source.albedo_texture == null)


func _special_paths() -> void:
	# Given each authored rendering path that the adapter must leave alone.
	for path: String in ["transparent", "emissive", "unshaded", "vertex_lit", "front_cull", "albedo_map"]:
		var source: StandardMaterial3D = _source("look1_special")
		match path:
			"transparent": source.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
			"emissive": source.emission_enabled = true
			"unshaded": source.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
			"vertex_lit": source.shading_mode = BaseMaterial3D.SHADING_MODE_PER_VERTEX
			"front_cull": source.cull_mode = BaseMaterial3D.CULL_FRONT
			"albedo_map": source.albedo_texture = GradientTexture2D.new()
		# When adapted, then the exact original resource survives.
		_check(path + "_retained", Textures3D.imported(source) == source)
	var custom: ShaderMaterial = ShaderMaterial.new()
	_check("custom_shader_retained", Textures3D.imported(custom) == custom)
	_check("null_retained", Textures3D.imported(null) == null)


func _meshes() -> void:
	# Given a mesh shared by consumers, including a special surface.
	var primitive: BoxMesh = BoxMesh.new()
	var original: StandardMaterial3D = _source("look1_unknown_surface")
	primitive.material = original
	var source: ArrayMesh = ArrayMesh.new()
	source.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, primitive.get_mesh_arrays())
	source.surface_set_material(0, original)
	var glow: StandardMaterial3D = _source("look1_glow")
	glow.emission_enabled = true
	source.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, primitive.get_mesh_arrays())
	source.surface_set_material(1, glow)
	var vertices: PackedVector3Array = source.surface_get_arrays(0)[Mesh.ARRAY_VERTEX]
	# When both ArrayMesh and PrimitiveMesh consumers are adapted.
	var consumer: Mesh = Textures3D.mesh(source, "look1_fixture")
	var primitive_consumer: Mesh = Textures3D.mesh(primitive, "look1_fixture")
	# Then the sources and geometry survive, special materials remain, and repeat consumers reuse the mesh.
	_check("array_consumer_isolated", consumer != source and source.surface_get_material(0) == original
		and source.surface_get_material(1) == glow and original is StandardMaterial3D)
	_check("array_geometry_retained", consumer.get_surface_count() == source.get_surface_count()
		and consumer.surface_get_arrays(0)[Mesh.ARRAY_VERTEX] == vertices
		and source.surface_get_arrays(0)[Mesh.ARRAY_VERTEX] == vertices)
	_check("array_materials_adapted", consumer.surface_get_material(0) is ShaderMaterial and consumer.surface_get_material(1) == glow)
	_check("array_consumer_cached", Textures3D.mesh(source, "look1_fixture") == consumer)
	_check("primitive_consumer_isolated", primitive_consumer != primitive and primitive.material == original
		and primitive_consumer.surface_get_material(0) is ShaderMaterial)
	_check("primitive_consumer_cached", Textures3D.mesh(primitive, "look1_fixture") == primitive_consumer)
	_check("model_binding_cache_separate", Textures3D.mesh(source, "look1_other_model") != consumer)


func _grade() -> void:
	# Given the production LUT; when sampled on every authored lattice point, its range and ordering stay valid.
	var lut: Texture3D = Sky3D.make_grade_lut()
	_check("grade_dimensions", lut.get_width() == 32 and lut.get_height() == 32 and lut.get_depth() == 32)
	# The dummy renderer cannot read GPU storage; use the exact production upload slices headlessly.
	var slices: Array[Image] = Sky3D.make_grade_slices() if DisplayServer.get_name() == "headless" else lut.get_data()
	_check("grade_slices", slices.size() == 32)
	if slices.size() != 32:
		return
	_sampled_grade(slices)
	var finite: bool = true
	var monotone: bool = true
	var coloured: bool = false
	for b: int in 32:
		for g: int in 32:
			for r: int in 32:
				var value: Color = slices[b].get_pixel(r, g)
				for channel: int in 3:
					finite = finite and is_finite(value[channel]) and value[channel] >= 0.0 and value[channel] <= 1.0
				if r > 0:
					monotone = monotone and value.r + 0.00001 >= slices[b].get_pixel(r - 1, g).r
				if g > 0:
					monotone = monotone and value.g + 0.00001 >= slices[b].get_pixel(r, g - 1).g
				if b > 0:
					monotone = monotone and value.b + 0.00001 >= slices[b - 1].get_pixel(r, g).b
				if r >= 8 and r <= 23 and g >= 8 and g <= 23 and b >= 8 and b <= 23 and r != g:
					coloured = coloured or absf(value.r - (float(r) + 0.5) / 32.0) + absf(value.g - (float(g) + 0.5) / 32.0) + absf(value.b - (float(b) + 0.5) / 32.0) > 0.01
	var black: Color = slices[0].get_pixel(0, 0)
	var white: Color = slices[31].get_pixel(31, 31)
	_check("grade_finite_range", finite)
	_check("grade_monotone_channels", monotone)
	_check("grade_restrained_endpoints", maxf(black.r, maxf(black.g, black.b)) <= 0.08
		and minf(white.r, minf(white.g, white.b)) >= 0.85)
	_check("grade_coloured_midtones", coloured)


## GPU normalized linear sampling: texel centres are (i + 0.5) / size, with edge clamping.
func _sample_grade(slices: Array[Image], input: Color) -> Color:
	var size: int = slices.size()
	var position: Vector3 = Vector3(input.r, input.g, input.b) * size - Vector3.ONE * 0.5
	var lower: Vector3i = Vector3i(position.floor())
	var blend: Vector3 = position - position.floor()
	var result: Color = Color(0.0, 0.0, 0.0, 0.0)
	for b: int in 2:
		for g: int in 2:
			for r: int in 2:
				var weight: float = (blend.x if r == 1 else 1.0 - blend.x) * (blend.y if g == 1 else 1.0 - blend.y) * (blend.z if b == 1 else 1.0 - blend.z)
				result += slices[clampi(lower.z + b, 0, size - 1)].get_pixel(clampi(lower.x + r, 0, size - 1), clampi(lower.y + g, 0, size - 1)) * weight
	return result


func _sampled_grade(slices: Array[Image]) -> void:
	# Given the restrained grade's neutral-ramp target colours, including visible night values.
	var inputs: Array[float] = [0.1, 0.2, 0.8, 0.9]
	var expected: Array[Color] = [Color(0.10144, 0.10298, 0.10767), Color(0.19936, 0.20032, 0.20348),
		Color(0.79696, 0.79192, 0.78128), Color(0.89824, 0.89178, 0.87807)]
	var ramp_ok: bool = true
	var tolerance: float = 2.0 / 255.0
	# When the renderer samples normalized input directly, then the LUT must not expand the contrast range.
	for i: int in inputs.size():
		var sampled: Color = _sample_grade(slices, Color(inputs[i], inputs[i], inputs[i]))
		for channel: int in 3:
			ramp_ok = ramp_ok and absf(sampled[channel] - expected[i][channel]) <= tolerance
		print("LOOK1_LUT_SAMPLE input=%.2f sampled=%s expected=%s" % [inputs[i], sampled, expected[i]])
	_check("grade_normalized_midtone_ramp", ramp_ok)
	var night_dark: Color = _sample_grade(slices, Color(0.02, 0.02, 0.02))
	var night_light: Color = _sample_grade(slices, Color(0.04, 0.04, 0.04))
	var signal_delta: Color = night_light - night_dark
	var night_ok: bool = true
	for channel: int in 3:
		night_ok = night_ok and absf(signal_delta[channel] - 0.02) <= tolerance
	print("LOOK1_LUT_NIGHT dark=%s light=%s signal=%s" % [night_dark, night_light, signal_delta])
	_check("grade_normalized_night_signal", night_ok)
