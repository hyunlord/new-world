# data3d.gd — the 3D view's data: tunables (assets/3d/params3d.json), identity → model bindings
#   (assets/3d/bindings3d.json) and the model cache (assets/3d/models/<id>.glb, built by tools/3d/blender/*.py).
# Purpose: every number and content name the 3D renderer uses comes from here (ADR 0033). `mesh(id)` merges a glb's
#   mesh instances into one ArrayMesh (surface materials keep their glb names: `canopy`, `trunk`, `glow`, … are the
#   recolouring API) and caches it; a missing or broken model returns a small placeholder box, never null.
# Bridge methods called: none.
extends RefCounted

const PARAMS_PATH: String = "res://assets/3d/params3d.json"
const BINDINGS_PATH: String = "res://assets/3d/bindings3d.json"

static var _params: Dictionary = {}
static var _bindings: Dictionary = {}
static var _meshes: Dictionary = {}
static var _scenes: Dictionary = {}
static var _manifest: Dictionary = {}
static var _loaded: bool = false


static func _read(path: String) -> Dictionary:
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(path))
	if parsed is Dictionary:
		return parsed
	push_error("data3d: cannot read " + path)
	return {}


static func _load() -> void:
	if _loaded:
		return
	_loaded = true
	_params = _read(PARAMS_PATH)
	_bindings = _read(BINDINGS_PATH)
	var manifest_path: String = str(_bindings.get("models_dir", "res://assets/3d/models")).path_join("manifest.json")
	if FileAccess.file_exists(manifest_path):
		_manifest = _read(manifest_path)


## The value at dotted `path` in params3d.json, or `fallback`.
static func param(path: String, fallback: Variant) -> Variant:
	_load()
	return _dig(_params, path, fallback)


## The value at dotted `path` in bindings3d.json, or `fallback`.
static func binding(path: String, fallback: Variant) -> Variant:
	_load()
	return _dig(_bindings, path, fallback)


static func _dig(root: Dictionary, path: String, fallback: Variant) -> Variant:
	var node: Variant = root
	for part: String in path.split("."):
		if node is Dictionary and (node as Dictionary).has(part):
			node = (node as Dictionary)[part]
		else:
			return fallback
	return node


static func colour(value: Variant, fallback: Color = Color(0.6, 0.6, 0.6)) -> Color:
	if value is Array and (value as Array).size() >= 3:
		var a: Array = value
		return Color(float(a[0]), float(a[1]), float(a[2]), float(a[3]) if a.size() > 3 else 1.0)
	return fallback


## Albedo of a material id (or its category) from bindings3d `materials`.
static func material_colour(material_id: String, category: String = "") -> Color:
	var by_id: Dictionary = binding("materials.by_id", {})
	if by_id.has(material_id):
		return colour(by_id[material_id])
	var by_cat: Dictionary = binding("materials.by_category", {})
	if by_cat.has(category):
		return colour(by_cat[category])
	return colour(binding("materials.fallback", []))


## Manifest row of a model id ({} when the manifest or row is missing).
static func model_row(id: String) -> Dictionary:
	_load()
	var models: Variant = _manifest.get("models", _manifest)
	if models is Dictionary and (models as Dictionary).get(id) is Dictionary:
		return (models as Dictionary)[id]
	return {}


static func model_path(id: String) -> String:
	return str(binding("models_dir", "res://assets/3d/models")).path_join(id + ".glb")


static func has_model(id: String) -> bool:
	return id != "" and ResourceLoader.exists(model_path(id))


## The glb scene of model `id` (null when missing).
static func scene(id: String) -> PackedScene:
	if _scenes.has(id):
		return _scenes[id]
	var packed: PackedScene = null
	if has_model(id):
		packed = load(model_path(id)) as PackedScene
	_scenes[id] = packed
	return packed


## One ArrayMesh with every surface of model `id` in model space (cached). Placeholder box when missing.
static func mesh(id: String) -> Mesh:
	if _meshes.has(id):
		return _meshes[id]
	var out: Mesh = null
	var packed: PackedScene = scene(id)
	if packed != null:
		var root: Node = packed.instantiate()
		out = merge_meshes(root)
		root.free()
	if out == null:
		out = placeholder()
	_meshes[id] = out
	return out


## Merge every MeshInstance3D under `root` (with its transform relative to root) into one ArrayMesh.
static func merge_meshes(root: Node) -> ArrayMesh:
	var out: ArrayMesh = ArrayMesh.new()
	var stack: Array = [[root, Transform3D.IDENTITY]]
	while not stack.is_empty():
		var entry: Array = stack.pop_back()
		var node: Node = entry[0]
		var xf: Transform3D = entry[1]
		if node is Node3D and node != root:
			xf = xf * (node as Node3D).transform
		if node is MeshInstance3D and (node as MeshInstance3D).mesh != null:
			var m: Mesh = (node as MeshInstance3D).mesh
			for s: int in m.get_surface_count():
				var arrays: Array = m.surface_get_arrays(s)
				var verts: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
				for i: int in verts.size():
					verts[i] = xf * verts[i]
				arrays[Mesh.ARRAY_VERTEX] = verts
				if arrays[Mesh.ARRAY_NORMAL] is PackedVector3Array:
					var normals: PackedVector3Array = arrays[Mesh.ARRAY_NORMAL]
					for i: int in normals.size():
						normals[i] = (xf.basis * normals[i]).normalized()
					arrays[Mesh.ARRAY_NORMAL] = normals
				# Skinned data does not apply to a static merge.
				arrays[Mesh.ARRAY_BONES] = null
				arrays[Mesh.ARRAY_WEIGHTS] = null
				var material: Material = (node as MeshInstance3D).get_active_material(s)
				out.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
				var idx: int = out.get_surface_count() - 1
				out.surface_set_material(idx, material)
				out.surface_set_name(idx, material.resource_name if material != null else "")
		for child: Node in node.get_children():
			stack.append([child, xf])
	return out if out.get_surface_count() > 0 else null


## The surface material name of surface `s` (the recolouring key).
static func surface_key(m: Mesh, s: int) -> String:
	var mat: Material = m.surface_get_material(s)
	var key: String = mat.resource_name if mat != null else ""
	if key == "" and m is ArrayMesh:
		key = (m as ArrayMesh).surface_get_name(s)
	# glTF importers may suffix duplicates ("canopy.001").
	return key.get_slice(".", 0)


static func placeholder() -> Mesh:
	var box: BoxMesh = BoxMesh.new()
	box.size = Vector3(0.6, 0.6, 0.6)
	var mat: StandardMaterial3D = StandardMaterial3D.new()
	mat.albedo_color = Color(0.85, 0.2, 0.75)
	box.material = mat
	return box


static func clear() -> void:
	_loaded = false
	_meshes.clear()
	_scenes.clear()
