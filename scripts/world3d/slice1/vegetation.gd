# Slice 1 Poly Haven vegetation. Replace imported surface materials and preserve mesh LODs.
# Bridge methods called: none.
extends RefCounted

const ROOT: String = "res://assets/3d/slice1/vegetation/"
const WIND = preload("res://shaders/world3d/slice1_vegetation.gdshader")
const SlicePbr = preload("res://scripts/world3d/slice_pbr.gd")
const MODELS: Dictionary = {
	"slice_broadleaf": "tree_small_02",
	"slice_conifer": "fir_sapling_medium",
	"slice_fern": "fern_02",
}
const TEXTURE_STEMS: Dictionary = {
	"tree_small_02_trunk": "tree_small_02",
	"tree_small_02_branches": "tree_small_02_branch",
	"fir_sapling_medium_branches_dead": "fir_sapling_medium_branches",
}

static var _meshes: Dictionary = {}
static var _textures: Dictionary = {}
static var _cards: Dictionary = {}


static func mesh(key: String) -> ArrayMesh:
	if _meshes.has(key):
		return _meshes[key]
	var id: String = MODELS[key]
	var scene: PackedScene = load(ROOT + id + ".glb") as PackedScene
	var plant: Node = scene.instantiate()
	var instance: MeshInstance3D = plant as MeshInstance3D
	if instance == null:
		instance = plant.get_child(0) as MeshInstance3D
	var model: ArrayMesh = instance.mesh.duplicate() as ArrayMesh
	plant.free()
	# The opaque import's fused shadow mesh has no UVs for the cutouts.
	model.shadow_mesh = null
	var height: float = model.get_aabb().size.y
	var styled: bool = SlicePbr.style2()
	for surface: int in model.get_surface_count():
		var name: String = model.surface_get_material(surface).resource_name.trim_suffix("_slice")
		var stem: String = TEXTURE_STEMS.get(name, name)
		var material: ShaderMaterial = ShaderMaterial.new()
		material.shader = WIND
		material.set_shader_parameter("plant_height", height)
		material.set_shader_parameter("gust_amplitude", 0.065 if key == "slice_fern" else 0.17)
		material.set_shader_parameter("colour_map", _styled_texture(stem) if styled else _texture(stem, "diff"))
		material.set_shader_parameter("normal_map", _texture(stem, "nor_gl"))
		material.set_shader_parameter("rough_map", _texture(stem, "rough"))
		if styled:
			material.set_shader_parameter("detail", 0.22)
			material.set_shader_parameter("xray", true)
		var cutout: Texture2D = _texture(stem, "alpha")
		if cutout != null:
			material.set_shader_parameter("leaf_card", true)
			material.set_shader_parameter("cutout_map", cutout)
		model.surface_set_material(surface, material)
	_meshes[key] = model
	return model


## The two crossed quads use the same ground pivot, size and wind as the near tree.
static func card(key: String) -> ArrayMesh:
	if _cards.has(key):
		return _cards[key]
	var bounds: AABB = mesh(key).get_aabb()
	var half_width: float = maxf(bounds.size.x, bounds.size.z) * 0.5
	var height: float = bounds.size.y
	var material: ShaderMaterial = ShaderMaterial.new()
	material.shader = WIND
	material.set_shader_parameter("colour_map", _styled_texture(key.trim_prefix("slice_") + "_card"))
	material.set_shader_parameter("rgba_card", true)
	material.set_shader_parameter("plant_height", height)
	material.set_shader_parameter("gust_amplitude", 0.17)
	material.set_shader_parameter("xray", true)
	var surface: SurfaceTool = SurfaceTool.new()
	surface.begin(Mesh.PRIMITIVE_TRIANGLES)
	surface.set_material(material)
	const CORNERS: Array[Vector2] = [Vector2(0, 1), Vector2(1, 1), Vector2(1, 0), Vector2(0, 0)]
	for right: Vector3 in [Vector3.RIGHT, Vector3.BACK]:
		for index: int in [0, 1, 2, 0, 2, 3]:
			var uv: Vector2 = CORNERS[index]
			surface.set_uv(uv)
			surface.set_normal((right.cross(Vector3.UP) + Vector3.UP).normalized())
			surface.add_vertex(right * ((uv.x * 2.0 - 1.0) * half_width) + Vector3.UP * ((1.0 - uv.y) * height))
	var out: ArrayMesh = surface.commit()
	_cards[key] = out
	return out


static func _styled_texture(stem: String) -> Texture2D:
	var path: String = SlicePbr.S2_ROOT + stem + ".png"
	if _textures.has(path):
		return _textures[path]
	var source: Texture2D = load(path) as Texture2D
	var image: Image = source.get_image()
	if not image.has_mipmaps():
		image.generate_mipmaps()
	var texture: ImageTexture = ImageTexture.create_from_image(image)
	_textures[path] = texture
	return texture


static func _texture(stem: String, kind: String) -> Texture2D:
	var base: String = ROOT + "textures/%s_%s_1k." % [stem, kind]
	if _textures.has(base):
		return _textures[base]
	for extension: String in ["png", "jpg"]:
		if not ResourceLoader.exists(base + extension):
			continue
		var source: Texture2D = load(base + extension) as Texture2D
		var image: Image = source.get_image()
		if not image.has_mipmaps():
			image.generate_mipmaps(kind == "nor_gl")
		var texture: ImageTexture = ImageTexture.create_from_image(image)
		_textures[base] = texture
		return texture
	return null
