# world3d_texture_test.gd — material asset import and presentation compatibility checks.
# Bridge methods called: none. Run headless after importing the generated assets.
extends SceneTree

const Textures = preload("res://scripts/world3d/textures3d.gd")
const Data3D = preload("res://scripts/world3d/data3d.gd")
const Plants = preload("res://scripts/world3d/plants3d.gd")
const Flora = preload("res://scripts/world3d/flora3d.gd")
const Props = preload("res://scripts/world3d/props3d.gd")
var _failures: int = 0


func _check(label: String, ok: bool) -> void:
	print("W3D_TEXTURE %s %s" % [label, "PASS" if ok else "FAIL"])
	_failures += 0 if ok else 1


func _init() -> void:
	call_deferred("_run")


func _run() -> void:
	# Given an imported material, when adding texture detail, its colour/name/PBR values survive.
	var original: StandardMaterial3D = StandardMaterial3D.new()
	original.resource_name = "stone"
	original.albedo_color = Color(0.42, 0.36, 0.29)
	original.metallic = 0.15
	original.roughness = 0.82
	var textured: Material = Textures.imported(original)
	_check("authored_tint_preserved", Flora._original_colour(textured) == original.albedo_color)
	_check("surface_name_preserved", textured.resource_name == original.resource_name)
	_check("pbr_preserved", textured is ShaderMaterial and is_equal_approx(float((textured as ShaderMaterial).get_shader_parameter("metallic")), 0.15) and is_equal_approx(float((textured as ShaderMaterial).get_shader_parameter("roughness")), 0.82))
	original.cull_mode = BaseMaterial3D.CULL_DISABLED
	var two_sided: Material = Textures.imported(original)
	_check("two_sided_material_textured", two_sided is ShaderMaterial and (two_sided as ShaderMaterial).shader == load("res://shaders/world3d/model_material_twosided.gdshader"))
	var station_id: String = str(Data3D.binding("stations.fallback", ""))
	var station: Mesh = Textures.mesh(Data3D.mesh(station_id), station_id)
	var textured_surfaces: int = 0
	for surface: int in station.get_surface_count():
		textured_surfaces += 1 if station.surface_get_material(surface) is ShaderMaterial else 0
	_check("real_station_surfaces_textured", textured_surfaces > 0)
	for sample: Array in [["item_091_wool_cloth_s", "cloth", 30], ["item_102_iron_bloom_s", "stone", 34]]:
		var mesh: Mesh = Textures.mesh(Data3D.mesh(str(sample[0])), str(sample[0]))
		var correct: bool = false
		for surface: int in mesh.get_surface_count():
			if Data3D.surface_key(mesh, surface) == str(sample[1]) and mesh.surface_get_material(surface) is ShaderMaterial:
				correct = is_equal_approx(float((mesh.surface_get_material(surface) as ShaderMaterial).get_shader_parameter("texture_tile")), float(sample[2]))
		_check("actual_texture_" + str(sample[0]), correct)
	original.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	_check("special_alpha_path_preserved", Textures.imported(original) == original)
	original.transparency = BaseMaterial3D.TRANSPARENCY_DISABLED
	original.resource_name = "unknown_future_material"
	var unknown: Material = Textures.imported(original)
	_check("unknown_material_fallback", unknown is ShaderMaterial and float((unknown as ShaderMaterial).get_shader_parameter("texture_tile")) == -1.0 and Flora._original_colour(unknown) == original.albedo_color)
	_check_prop_consumers()
	# Given production plant geometry, winter still desaturates textured materials without adding surfaces.
	var plants: Node3D = Plants.new()
	var sample: String = ""
	var rows: Dictionary = Data3D.binding("plants.wild", {})
	if not rows.is_empty():
		sample = str((rows[rows.keys()[0]] as Array)[1])
	var summer: Mesh = plants.call("_desaturated", sample, 1.0)
	var winter: Mesh = plants.call("_desaturated", sample, 0.35)
	var desaturated: bool = sample != "" and summer.get_surface_count() == winter.get_surface_count()
	var coloured: int = 0
	for surface: int in summer.get_surface_count():
		var a: Color = Flora._original_colour(summer.surface_get_material(surface))
		var b: Color = Flora._original_colour(winter.surface_get_material(surface))
		if a.s > 0.05:
			coloured += 1
			desaturated = desaturated and b.s < a.s * 0.5
	_check("textured_winter_desaturated", desaturated and coloured > 0)
	plants.free()
	# Given final assets, every original and atlas imports at the required size with mipmaps and VRAM compression.
	var names: Array = (Textures.bindings().get("tiles", []) as Array).duplicate()
	for tile: Variant in Textures.bindings().get("normal_tiles", []):
		names.append(str(names[int(tile)]) + "_normal")
	names.append_array(["albedo_atlas", "normal_atlas"])
	for name: Variant in names:
		var path: String = "res://assets/3d/textures/%s.%s" % [name, "webp" if str(name).ends_with("atlas") else "png"]
		var ok: bool = ResourceLoader.exists(path)
		if ok:
			var tex: Texture2D = load(path) as Texture2D
			var size: int = 8192 if str(name).ends_with("atlas") else 1024
			var config: ConfigFile = ConfigFile.new()
			ok = tex != null and tex.get_width() == size and tex.get_height() == size and config.load(path + ".import") == OK
			ok = ok and bool(config.get_value("params", "mipmaps/generate", false)) and int(config.get_value("params", "compress/mode", -1)) == 2
			if str(name).ends_with("_normal") or name == "normal_atlas":
				ok = ok and int(config.get_value("params", "compress/normal_map", -1)) == 1
		_check("asset_" + str(name), ok)
	print("W3D_TEXTURE_DONE failures=%d" % _failures)
	quit(1 if _failures > 0 else 0)


func _check_prop_consumers() -> void:
	var props: Node3D = Props.new()
	for id: String in ["yard_cart", "yard_grain_baskets", "yard_hay", "yard_tool_rack", "yard_woodpile"]:
		var source: Mesh = Data3D.mesh(id)
		var materials: Array[Material] = []
		var colours: Array[Color] = []
		for s: int in source.get_surface_count():
			materials.append(source.surface_get_material(s))
			colours.append(Flora._original_colour(materials[s]))
		var rendered: Mesh = props.call("_tinted_mesh", id, "")
		var source_ok: bool = Data3D.has_model(id) and rendered != source
		var tiles_ok: bool = rendered.get_surface_count() == source.get_surface_count() and rendered.get_surface_count() > 0
		var overrides: Dictionary = (Textures.bindings().get("model_surfaces", {}) as Dictionary).get(id, {})
		for s: int in source.get_surface_count():
			var key: String = Data3D.surface_key(source, s)
			var expected: int = int(overrides.get(key, Textures.index("surfaces", key)))
			var mat: Material = rendered.surface_get_material(s)
			source_ok = source_ok and materials[s] is BaseMaterial3D and source.surface_get_material(s) == materials[s] and Flora._original_colour(materials[s]) == colours[s]
			tiles_ok = tiles_ok and expected >= 0 and mat is ShaderMaterial
			if mat is ShaderMaterial:
				tiles_ok = tiles_ok and int((mat as ShaderMaterial).get_shader_parameter("texture_tile")) == expected and Flora._original_colour(mat) == colours[s]
		_check(id + "_source_preserved", source_ok)
		_check(id + "_props_atlas_tiles", tiles_ok)
	var tinted: Mesh = props.call("_tinted_mesh", "item_076_copper_ingot_s", "bronze")
	var tint_ok: bool = false
	for s: int in tinted.get_surface_count():
		var mat: Material = tinted.surface_get_material(s)
		if Data3D.surface_key(tinted, s) == "metal" and mat is ShaderMaterial:
			tint_ok = int((mat as ShaderMaterial).get_shader_parameter("texture_tile")) == Textures.index("materials", "bronze") and Flora._original_colour(mat) == Data3D.material_colour("bronze", "Mineral")
	_check("props_tint_overrides_model_tile", tint_ok)
	props.free()
