# vat3d.gd — vertex-animation crowds for the 3D view (people and animals, M3D-1 scope 6).
# Purpose: loads a character's VAT built by tools/3d/blender/{people,animals}.py — assets/3d/vat/<id>_static.glb (rest
#   mesh, faces split, UV2.x = vertex column, COLOR_0 = tint region) and <id>_vat.bin/.json (float32 RGBA rows: one
#   row of positions per frame, then the same rows of normals) — into an RGBAF ImageTexture read raw with FileAccess
#   (no import), and rebuilds the mesh so the region code sits in UV.x and the vertex colour is gone (the MultiMesh
#   instance colour then carries the first tint). `material()` is a ShaderMaterial on shaders/world3d/vat.gdshader;
#   per instance: COLOR.rgb = region-1 tint, INSTANCE_CUSTOM = (clip start × 256 + clip count, phase seconds,
#   region-2 colour, region-3 colour) with colours packed as r·65536 + g·256 + b (exact in a float).
# Bridge methods called: none.
extends RefCounted

const VAT_SHADER = preload("res://shaders/world3d/vat.gdshader")
const DIR: String = "res://assets/3d/vat"

var id: String = ""
var meta: Dictionary = {}
var mesh: ArrayMesh = null
var texture: ImageTexture = null
var ok: bool = false


static func load_character(character: String) -> RefCounted:
	var v: RefCounted = (load("res://scripts/world3d/vat3d.gd") as GDScript).new()
	v.call("_load", character)
	return v


func _load(character: String) -> void:
	id = character
	var json_path: String = DIR.path_join(character + "_vat.json")
	var bin_path: String = DIR.path_join(character + "_vat.bin")
	var glb_path: String = DIR.path_join(character + "_static.glb")
	if not FileAccess.file_exists(json_path) or not FileAccess.file_exists(bin_path) or not ResourceLoader.exists(glb_path):
		return
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(json_path))
	if parsed is not Dictionary:
		return
	meta = parsed
	var width: int = int(meta.get("width", 0))
	var rows: int = int(meta.get("frames", 0)) * 2
	var bytes: PackedByteArray = FileAccess.get_file_as_bytes(bin_path)
	if width <= 0 or rows <= 0 or bytes.size() != width * rows * 16:
		push_error("vat3d: %s size mismatch" % character)
		return
	var image: Image = Image.create_from_data(width, rows, false, Image.FORMAT_RGBAF, bytes)
	texture = ImageTexture.create_from_image(image)
	var root: Node = (load(glb_path) as PackedScene).instantiate()
	mesh = _regions(root)
	root.free()
	ok = mesh != null


## The first surface of the glb with its COLOR_0 region turned into UV.x (0 other, 1 r, 2 g, 3 b).
static func _regions(root: Node) -> ArrayMesh:
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var node: Node = stack.pop_back()
		if node is MeshInstance3D and (node as MeshInstance3D).mesh != null:
			var arrays: Array = (node as MeshInstance3D).mesh.surface_get_arrays(0)
			var cols: Variant = arrays[Mesh.ARRAY_COLOR]
			var verts: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
			var uv: PackedVector2Array = PackedVector2Array()
			uv.resize(verts.size())
			if cols is PackedColorArray:
				for i: int in verts.size():
					var c: Color = (cols as PackedColorArray)[i]
					var region: float = 0.0
					if c.r > 0.5 and c.r >= c.g and c.r >= c.b:
						region = 1.0
					elif c.g > 0.5 and c.g >= c.b:
						region = 2.0
					elif c.b > 0.5:
						region = 3.0
					uv[i] = Vector2(region, 0.0)
			arrays[Mesh.ARRAY_COLOR] = null
			arrays[Mesh.ARRAY_TEX_UV] = uv
			arrays[Mesh.ARRAY_BONES] = null
			arrays[Mesh.ARRAY_WEIGHTS] = null
			var out: ArrayMesh = ArrayMesh.new()
			out.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
			# The VAT moves vertices far from the rest pose: a generous AABB keeps culling honest.
			out.custom_aabb = AABB(Vector3(-1.5, -0.2, -1.5), Vector3(3.0, 3.0, 3.0))
			return out
		for child: Node in node.get_children():
			stack.append(child)
	return null


func material(other: Color) -> ShaderMaterial:
	var mat: ShaderMaterial = ShaderMaterial.new()
	mat.shader = VAT_SHADER
	mat.set_shader_parameter("vat", texture)
	mat.set_shader_parameter("width", float(meta.get("width", 1)))
	mat.set_shader_parameter("normals_offset", float(meta.get("normals_offset", meta.get("frames", 0))))
	mat.set_shader_parameter("fps", float(meta.get("fps", 15)))
	mat.set_shader_parameter("other_colour", other)
	return mat


## The packed instance custom value for clip `clip` (falls back to the first clip).
func clip_pack(clip: String) -> float:
	var clips: Dictionary = meta.get("clips", {})
	var row: Variant = clips.get(clip)
	if row is not Dictionary:
		if clips.is_empty():
			return 1.0
		row = clips[clips.keys()[0]]
	return float(int((row as Dictionary).get("start", 0)) * 256 + maxi(1, int((row as Dictionary).get("count", 1))))


func has_clip(clip: String) -> bool:
	return (meta.get("clips", {}) as Dictionary).has(clip)


static func pack(c: Color) -> float:
	return float(c.r8 * 65536 + c.g8 * 256 + c.b8)
