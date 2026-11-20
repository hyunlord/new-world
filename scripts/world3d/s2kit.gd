# Load and fit the opt-in s2 static mesh tiers. Bridge methods called: none.
extends RefCounted

const Data3D = preload("res://scripts/world3d/data3d.gd")
const SHADER = preload("res://shaders/world3d/s2kit.gdshader")
const ROOT: String = "res://assets/3d/anchor2/kit/"
static var _meshes: Dictionary = {}
static var _atlases: Dictionary = {}


static func _atlas_id(id: String) -> String:
	return str(Data3D.binding("s2kit.part_atlases", {}).get(id, id))


static func texture(id: String, suffix: String = "") -> Texture2D:
	var key: String = _atlas_id(id) + suffix
	if not _atlases.has(key):
		var path: String = ROOT + key + ".png"
		var atlas: Texture2D = null
		if FileAccess.file_exists(path):
			var image: Image = Image.load_from_file(ProjectSettings.globalize_path(path))
			if image != null and not image.is_empty():
				image.generate_mipmaps()
				atlas = ImageTexture.create_from_image(image)
		_atlases[key] = atlas
	return _atlases[key]


static func mesh(id: String, lod: bool = false) -> ArrayMesh:
	if id.is_empty():
		return null
	var key: String = id + ("_lod" if lod else "")
	if not _meshes.has(key):
		var path: String = ROOT + key + ".glb"
		var packed: PackedScene = load(path) as PackedScene if ResourceLoader.exists(path, "PackedScene") else null
		var model: ArrayMesh = null
		if packed != null:
			var scene: Node = packed.instantiate()
			model = Data3D.merge_meshes(scene)
			scene.free()
			var material: BaseMaterial3D = model.surface_get_material(0) as BaseMaterial3D if model != null else null
			var atlas: Texture2D = texture(id)
			if material != null and atlas != null:
				material.albedo_texture = atlas
			else:
				model = null
		_meshes[key] = model
	return _meshes[key]


static func distance() -> float:
	return maxf(1.0, float(Data3D.binding("s2kit.lod_distance", 28.0)))


static func material_for(model: Mesh, wall: bool, id: String = "") -> ShaderMaterial:
	var source: BaseMaterial3D = model.surface_get_material(0) as BaseMaterial3D
	var material: ShaderMaterial = ShaderMaterial.new()
	material.shader = SHADER
	material.set_shader_parameter("wall", wall)
	if source != null:
		material.set_shader_parameter("albedo_tex", source.albedo_texture)
	if not id.is_empty():
		var normal: Texture2D = texture(id, "_normal")
		var roughness: Texture2D = texture(id, "_rm")
		if normal != null and roughness != null:
			material.set_shader_parameter("mapped_surface", true)
			material.set_shader_parameter("normal_tex", normal)
			material.set_shader_parameter("rm_tex", roughness)
	return material


## Fit to physical dimensions about the point on the ground that the mesh stands on.
static func fit(model: Mesh, size: Vector3, yaw: float, ground: Vector3) -> Transform3D:
	var box: AABB = model.get_aabb()
	var base: Vector3 = Vector3(box.get_center().x, box.position.y, box.get_center().z)
	var basis: Basis = Basis(Vector3.UP, yaw).scaled_local(size / box.size.max(Vector3.ONE * 0.0001))
	return Transform3D(basis, ground - basis * base)


## Uniform fit for an authored shell. Split parts use the same whole-mesh transform.
static func fit_proportional(model: Mesh, span: Vector2, yaw: float, ground: Vector3) -> Transform3D:
	var size: Vector3 = model.get_aabb().size
	var oriented: Basis = Basis(Vector3.UP, yaw)
	var width: float = absf(oriented.x.x) * size.x + absf(oriented.z.x) * size.z
	var depth: float = absf(oriented.x.z) * size.x + absf(oriented.z.z) * size.z
	var scale: float = minf(span.x / maxf(width, 0.001), span.y / maxf(depth, 0.001))
	return fit(model, size * scale, yaw, ground)


static func fitted_part(id: String, model: ArrayMesh, size: Vector3, yaw: float, ground: Vector3) -> MeshInstance3D:
	var part: MeshInstance3D = MeshInstance3D.new()
	part.mesh = model
	part.transform = fit(model, size, yaw, ground)
	part.material_override = material_for(model, false, id)
	add_lod(part, id)
	return part


## The child shares its parent's model-space frame. Its own distance range keeps the active tiers exclusive.
static func add_lod(near: MeshInstance3D, id: String) -> void:
	var model: ArrayMesh = mesh(id, true)
	if model == null:
		return
	var far: MeshInstance3D = MeshInstance3D.new()
	far.name = "KitLod"
	far.mesh = model
	far.visibility_range_begin = distance()
	far.material_override = near.material_override
	far.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	near.visibility_range_end = distance()
	near.add_child(far)
