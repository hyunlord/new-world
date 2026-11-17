# world3d_texture_test.gd — material asset import and presentation compatibility checks.
# Bridge methods called: none. Run headless after importing the generated assets.
extends SceneTree

const Textures = preload("res://scripts/world3d/textures3d.gd")
const Data3D = preload("res://scripts/world3d/data3d.gd")
const Plants = preload("res://scripts/world3d/plants3d.gd")
const Flora = preload("res://scripts/world3d/flora3d.gd")
const Structures = preload("res://scripts/world3d/structures3d.gd")
const Terrain = preload("res://scripts/world3d/terrain3d.gd")
var _failures: int = 0


class FieldWorld extends Node:
	var phase: int = 1
	func get_field_tiles() -> Dictionary:
		return {"xs": PackedInt64Array([0]), "ys": PackedInt64Array([0]), "phases": PackedInt64Array([phase])}
	func get_space() -> Dictionary:
		return {}
	func get_building_snapshot() -> Dictionary:
		return {}
	func get_world_info() -> Dictionary:
		return {"seed": 0}


class FieldTerrain extends "res://scripts/world3d/terrain3d.gd":
	var full_recomputes: int = 0
	func _compute_tiles() -> void:
		full_recomputes += 1
		super._compute_tiles()


func _check(label: String, ok: bool) -> void:
	print("W3D_TEXTURE %s %s" % [label, "PASS" if ok else "FAIL"])
	_failures += 0 if ok else 1


func _init() -> void:
	call_deferred("_run")


func _run() -> void:
	_check("dedicated_log_wall", Textures.index("styles", "log") == 41)
	_check("tilled_field_binding", Textures.index("terrain_special", "field") == 53)
	_check("plank_roof_keeps_plank_material", Textures.index("roofs", "plank") == 17)
	var structures: Node3D = Structures.new()
	root.add_child(structures)
	var floors: Dictionary = structures.call("_floor_tiles", {"floor_xs": PackedInt32Array([0]), "floor_ys": PackedInt32Array([0]), "floor_styles": PackedStringArray(["PackedFloor"])})
	_check("real_floor_resolver_selects_earth", int((floors[Vector2i.ZERO] as Dictionary)["texture"]) == 46)
	structures.free()
	# A floor's explicit texture must not be overwritten by the outdoor clearing blend.
	var terrain: Node3D = Terrain.new()
	terrain.set("width", 1)
	terrain.set("height", 1)
	terrain.set("_terrain", PackedByteArray([0]))
	terrain.set("_floor", {Vector2i.ZERO: true})
	terrain.set("_raise", PackedFloat32Array([0.0]))
	terrain.set("_colour", PackedColorArray([Color.WHITE]))
	terrain.set("_texture_tiles", PackedByteArray([0]))
	terrain.set("_texture_weights", PackedByteArray([0]))
	terrain.call("_compute_tile", 0, {}, [], {}, Color.WHITE, Color.WHITE, Color.WHITE, Color.WHITE, 4, 0.0, 1)
	_check("floor_ground_texture_not_overridden", (terrain.get("_texture_tiles") as PackedByteArray)[0] == 46 and (terrain.get("_texture_weights") as PackedByteArray)[0] == 0)
	terrain.free()
	# Changing only the authoritative phase must refresh the same cell without rebuilding map heights.
	var field_world: FieldWorld = FieldWorld.new()
	var field_terrain: FieldTerrain = FieldTerrain.new()
	root.add_child(field_terrain)
	field_terrain.set_process(false)
	field_terrain.width = 1
	field_terrain.height = 1
	field_terrain.set("_terrain", PackedByteArray([0]))
	field_terrain.set("_world_sim", field_world)
	for phase: int in [1, 2, 3, 0, 4, 5, 6]:
		field_world.phase = phase
		field_terrain.call("_poll_space")
		var expected: int = 52 + phase if phase >= 1 and phase <= 3 else (6 if phase == 0 else 53)
		_check("same_field_tile_phase_%d" % phase, (field_terrain.get("_texture_tiles") as PackedByteArray)[0] == expected)
	_check("field_phases_keep_incremental_height_path", field_terrain.full_recomputes == 1)
	field_terrain.free()
	field_world.free()
	# Given an imported material, when adding texture detail, its colour/name/PBR values survive.
	var original: StandardMaterial3D = StandardMaterial3D.new()
	original.resource_name = "stone"
	original.albedo_color = Color(0.42, 0.36, 0.29)
	original.metallic = 0.15
	original.roughness = 0.82
	var textured: Material = Textures.imported(original)
	# Given atlas statistics, setup uploads one linear mean per cell without extra texture reads.
	var means: PackedFloat32Array = (textured as ShaderMaterial).get_shader_parameter("material_luminance")
	var configured: Array = Textures.bindings().get("linear_luminance_means", [])
	_check("mean_count_matches_tiles", configured.size() == (Textures.bindings().get("tiles", []) as Array).size() and configured.size() <= 64)
	var means_match: bool = means.size() == 64
	for tile: int in configured.size():
		means_match = means_match and float(configured[tile]) > 0.0 and float(configured[tile]) < 1.0 and is_equal_approx(means[tile], float(configured[tile]))
	_check("linear_means_uploaded", means_match)
	_check("macro_repeat_uploaded", is_equal_approx(float((textured as ShaderMaterial).get_shader_parameter("texture_repeat_scale")), float(Textures.bindings().get("repeat_scale", 1.0))))
	_check("authored_tint_preserved", Flora._original_colour(textured) == original.albedo_color)
	_check("surface_name_preserved", textured.resource_name == original.resource_name)
	_check("pbr_preserved", textured is ShaderMaterial and is_equal_approx(float((textured as ShaderMaterial).get_shader_parameter("metallic")), 0.15) and is_equal_approx(float((textured as ShaderMaterial).get_shader_parameter("roughness")), 0.82))
	original.cull_mode = BaseMaterial3D.CULL_DISABLED
	var two_sided: Material = Textures.imported(original)
	_check("two_sided_material_textured", two_sided is ShaderMaterial and (two_sided as ShaderMaterial).shader == load("res://shaders/world3d/model_material_twosided.gdshader"))
	var station: Mesh = Data3D.mesh(str(Data3D.binding("stations.fallback", "")))
	var textured_surfaces: int = 0
	for surface: int in station.get_surface_count():
		textured_surfaces += 1 if station.surface_get_material(surface) is ShaderMaterial else 0
	_check("real_station_surfaces_textured", textured_surfaces > 0)
	for sample: Array in [["item_091_wool_cloth_s", "cloth", 30], ["item_102_iron_bloom_s", "stone", 34]]:
		var mesh: Mesh = Data3D.mesh(str(sample[0]))
		var correct: bool = false
		for surface: int in mesh.get_surface_count():
			if Data3D.surface_key(mesh, surface) == str(sample[1]) and mesh.surface_get_material(surface) is ShaderMaterial:
				correct = is_equal_approx(float((mesh.surface_get_material(surface) as ShaderMaterial).get_shader_parameter("texture_tile")), float(sample[2]))
		_check("actual_texture_" + str(sample[0]), correct)
	original.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	_check("special_alpha_path_preserved", Textures.imported(original) == original)
	original.transparency = BaseMaterial3D.TRANSPARENCY_DISABLED
	original.resource_name = "unknown_future_material"
	_check("unknown_material_fallback", Textures.imported(original) == original)
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
