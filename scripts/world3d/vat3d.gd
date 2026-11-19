# vat3d.gd — vertex-animation crowds for the 3D view (people and animals, M3D-1 scope 6).
# Purpose: loads a character's VAT built by tools/3d/blender/{people,animals}.py — assets/3d/vat/<id>_static.glb (rest
#   mesh, faces split, UV2.x = the unsplit vertex's column, COLOR_0 = tint region) and <id>_vat.bin/.json (float16
#   RGBA rows, one row of positions per frame; flat normals are derived in the shader) — into an RGBAH ImageTexture
#   read raw with FileAccess (no import), and rebuilds the mesh so the region code sits in UV2.y and the vertex colour is gone (the MultiMesh
#   instance colour then carries the first tint). A `textured` character (set B: json `textured`, `atlas`) keeps its atlas
#   UVs in UV; loaded with `tint_atlas` its regions come from the atlas colour under each vertex (dark → hair 2,
#   yellow/olive → clothing 3, else skin 1) and the shader rescales the painted colours toward the person's tints
#   (painted colours kept: skin value only, small clothing hue/value shift, hair blend; params3d `people.tint`), so painted detail stays and appearance still varies. `material()` is a ShaderMaterial on shaders/world3d/vat.gdshader;
#   per instance: COLOR.rgb = region-1 tint, INSTANCE_CUSTOM = (clip start × 256 + clip count, phase seconds,
#   region-2 colour, region-3 colour) with colours packed as r·65536 + g·256 + b (exact in a float).
# Bridge methods called: none.
extends RefCounted

const VAT_SHADER = preload("res://shaders/world3d/vat.gdshader")
const OUTLINE_SHADER = preload("res://shaders/world3d/vat_outline.gdshader")
const VISIBLE_MASK_SHADER = preload("res://shaders/world3d/vat_visible_mask.gdshader")
const XRAY_SHADER = preload("res://shaders/world3d/vat_xray.gdshader")
const Data3D = preload("res://scripts/world3d/data3d.gd")
const DIR: String = "res://assets/3d/vat"
static var _style2_shader: Shader

var id: String = ""
var meta: Dictionary = {}
var mesh: ArrayMesh = null
var texture: ImageTexture = null
var atlas: ImageTexture = null
## Mean sRGB atlas colour of regions 1–3 (textured, tinted characters).
var region_means: Array[Color] = [Color.WHITE, Color.WHITE, Color.WHITE]
var ok: bool = false


static func load_character(character: String, tint_atlas: bool = false) -> RefCounted:
	var v: RefCounted = (load("res://scripts/world3d/vat3d.gd") as GDScript).new()
	v.call("_load", character, tint_atlas)
	return v


func _load(character: String, tint_atlas: bool = false) -> void:
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
	var rows: int = int(meta.get("frames", 0))
	var bytes: PackedByteArray = FileAccess.get_file_as_bytes(bin_path)
	if width <= 0 or rows <= 0 or bytes.size() != width * rows * 8:
		push_error("vat3d: %s size mismatch" % character)
		return
	var image: Image = Image.create_from_data(width, rows, false, Image.FORMAT_RGBAH, bytes)
	texture = ImageTexture.create_from_image(image)
	var atlas_image: Image = null
	if bool(meta.get("textured", false)):
		atlas_image = Image.load_from_file(ProjectSettings.globalize_path(DIR.path_join(str(meta.get("atlas", "")))))
		if atlas_image == null or atlas_image.is_empty():
			push_error("vat3d: %s atlas missing" % character)
			return
		atlas_image.convert(Image.FORMAT_RGBA8)
		var gpu: Image = atlas_image.duplicate()
		gpu.generate_mipmaps()
		atlas = ImageTexture.create_from_image(gpu)
	var root: Node = (load(glb_path) as PackedScene).instantiate()
	mesh = _regions(root, atlas_image, tint_atlas, region_means)
	root.free()
	ok = mesh != null


## The first surface of the glb with its tint region in UV2.y (0 other, 1 r, 2 g, 3 b of COLOR_0; textured: atlas
## classes when `tint_atlas`, else 0) and, untextured, UV cleared.
static func _regions(root: Node, atlas_image: Image, tint_atlas: bool, means: Array[Color]) -> ArrayMesh:
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var node: Node = stack.pop_back()
		if node is MeshInstance3D and (node as MeshInstance3D).mesh != null:
			var arrays: Array = (node as MeshInstance3D).mesh.surface_get_arrays(0)
			var cols: Variant = arrays[Mesh.ARRAY_COLOR]
			var verts: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
			var uv2: PackedVector2Array = arrays[Mesh.ARRAY_TEX_UV2]
			if atlas_image != null:
				var sums: Array[Color] = [Color(0, 0, 0, 0), Color(0, 0, 0, 0), Color(0, 0, 0, 0)]
				var atlas_uv: PackedVector2Array = arrays[Mesh.ARRAY_TEX_UV]
				var votes: PackedInt32Array = PackedInt32Array()
				votes.resize(verts.size() * 3)
				if tint_atlas:
					# Sample each triangle at its UV centroid: vertices sit on island borders next to the atlas padding.
					var index: PackedInt32Array = arrays[Mesh.ARRAY_INDEX] if arrays[Mesh.ARRAY_INDEX] is PackedInt32Array else PackedInt32Array(range(verts.size()))
					var size: Vector2 = Vector2(atlas_image.get_size())
					for t: int in range(0, index.size() - 2, 3):
						var uv: Vector2 = (atlas_uv[index[t]] + atlas_uv[index[t + 1]] + atlas_uv[index[t + 2]]) / 3.0
						var c: Color = atlas_image.get_pixelv(Vector2i((uv * size).floor()).clamp(Vector2i.ZERO, atlas_image.get_size() - Vector2i.ONE))
						var region: int = atlas_region(c)
						sums[region - 1] += Color(c.r, c.g, c.b, 1.0)
						for k: int in 3:
							votes[index[t + k] * 3 + region - 1] += 1
				for i: int in verts.size():
					var region: int = 0
					if tint_atlas:
						var best: int = 0
						for r: int in 3:
							if votes[i * 3 + r] > votes[i * 3 + best]:
								best = r
						region = best + 1
					uv2[i] = Vector2(uv2[i].x, float(region))
				for k: int in 3:
					if sums[k].a > 0.0:
						means[k] = Color(sums[k].r / sums[k].a, sums[k].g / sums[k].a, sums[k].b / sums[k].a)
				arrays[Mesh.ARRAY_TEX_UV2] = uv2
				arrays[Mesh.ARRAY_COLOR] = null
				arrays[Mesh.ARRAY_BONES] = null
				arrays[Mesh.ARRAY_WEIGHTS] = null
				var tex_out: ArrayMesh = ArrayMesh.new()
				tex_out.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
				tex_out.custom_aabb = AABB(Vector3(-1.5, -0.2, -1.5), Vector3(3.0, 3.0, 3.0))
				return tex_out
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
					uv2[i] = Vector2(uv2[i].x, region)
			arrays[Mesh.ARRAY_COLOR] = null
			arrays[Mesh.ARRAY_TEX_UV] = uv
			arrays[Mesh.ARRAY_TEX_UV2] = uv2
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


## Tint class of an atlas texel (sRGB): 2 hair (dark), 3 clothing (yellow or olive), 1 skin.
static func atlas_region(c: Color) -> int:
	if 0.3 * c.r + 0.59 * c.g + 0.11 * c.b < 0.3:
		return 2
	if c.b < 0.3 * c.r or c.g >= c.r:
		return 3
	return 1


func material(other: Color, style2: bool = false) -> ShaderMaterial:
	var mat: ShaderMaterial = ShaderMaterial.new()
	if style2 and _style2_shader == null:
		_style2_shader = Shader.new()
		_style2_shader.code = "#define S2_VAT\n" + VAT_SHADER.code
	mat.shader = _style2_shader if style2 else VAT_SHADER
	if atlas != null:
		mat.set_shader_parameter("atlas", atlas)
		mat.set_shader_parameter("textured", true)
		mat.set_shader_parameter("mean_skin", region_means[0])
		mat.set_shader_parameter("mean_hair", region_means[1])
		mat.set_shader_parameter("mean_cloth", region_means[2])
		var sv: Array = Data3D.param("people.tint.skin_value", [0.8, 1.2])
		var cv: Array = Data3D.param("people.tint.cloth_value", [0.8, 1.2])
		mat.set_shader_parameter("skin_value", Vector2(float(sv[0]), float(sv[1])))
		mat.set_shader_parameter("cloth_value", Vector2(float(cv[0]), float(cv[1])))
		mat.set_shader_parameter("hair_mix", float(Data3D.param("people.tint.hair_mix", 0.6)))
		mat.set_shader_parameter("cloth_hue_mix", float(Data3D.param("people.tint.cloth_hue_mix", 0.3)))
	_set_anim(mat)
	mat.set_shader_parameter("other_colour", other)
	return mat


## People only: far figures enlarged (params3d `people.far_scale` [from metres, max]), a screen-constant outline pass
## (`people.outline`: px, far_px, day and night colours) and a see-through silhouette where something stands in front
## (`people.xray`) that skips pixels occupied by a visible body.
func with_findability(mat: ShaderMaterial, style2: bool = false) -> ShaderMaterial:
	var far: Array = Data3D.param("people.style2_far_scale" if style2 else "people.far_scale", [60.0, 1.6])
	mat.set_shader_parameter("far_scale", Vector2(float(far[0]), float(far[1])))
	var fill: Color = Data3D.colour(Data3D.param("people.far_fill", []), Color(0.16, 0.13, 0.11))
	mat.set_shader_parameter("far_fill", Color(fill, float(Data3D.param("people.far_fill_strength", 1.0))))
	mat.set_shader_parameter("far_fill_night", Data3D.colour(Data3D.param("people.far_fill_night", []), Color(0.91, 0.89, 0.81)))
	if style2:
		mat.set_shader_parameter("far_fill_range", Vector2(25.0, 48.0))
		mat.set_shader_parameter("far_fill", Vector4(0.012, 0.010, 0.009, 1.0))
	var line: ShaderMaterial = ShaderMaterial.new()
	line.shader = OUTLINE_SHADER
	_set_anim(line)
	line.set_shader_parameter("far_scale", Vector2(float(far[0]), float(far[1])))
	line.set_shader_parameter("body_height", float(meta.get("height", 1.7)))
	line.set_shader_parameter("outline_px", float(Data3D.param("people.outline.px", 1.5)))
	line.set_shader_parameter("day_colour", Data3D.colour(Data3D.param("people.outline.day", []), Color(0.14, 0.11, 0.09)))
	line.set_shader_parameter("night_colour", Data3D.colour(Data3D.param("people.outline.night", []), Color(0.91, 0.89, 0.81)))
	line.set_shader_parameter("outline_far_px", float(Data3D.param("people.outline.far_px", 4.0)))
	var mask: ShaderMaterial = ShaderMaterial.new()
	mask.shader = VISIBLE_MASK_SHADER
	_set_anim(mask)
	mask.set_shader_parameter("far_scale", Vector2(float(far[0]), float(far[1])))
	# Stamp every visible body before any transparent silhouette draws.
	mask.render_priority = -1
	var xray: ShaderMaterial = ShaderMaterial.new()
	xray.shader = XRAY_SHADER
	_set_anim(xray)
	xray.set_shader_parameter("far_scale", Vector2(float(far[0]), float(far[1])))
	var dc: Color = Data3D.colour(Data3D.param("people.xray.day", []), Color(0.06, 0.055, 0.05))
	var nc: Color = Data3D.colour(Data3D.param("people.xray.night", []), Color(0.57, 0.65, 0.66))
	xray.set_shader_parameter("day_colour", Color(dc, float(Data3D.param("people.xray.day_alpha", 0.9))))
	xray.set_shader_parameter("night_colour", Color(nc, float(Data3D.param("people.xray.night_alpha", 0.54))))
	if style2:
		xray.set_shader_parameter("night_colour", Color(nc, 0.78))
		xray.set_shader_parameter("distant_only", true)
		var sleep_start: float = floorf(clip_pack("sleep") / 256.0)
		for pass_material: ShaderMaterial in [mat, line, mask, xray]:
			pass_material.set_shader_parameter("distant_sleep_clip", sleep_start)
	mask.next_pass = xray
	line.next_pass = mask
	mat.next_pass = line
	return mat


func _set_anim(mat: ShaderMaterial) -> void:
	mat.set_shader_parameter("vat", texture)
	mat.set_shader_parameter("width", float(meta.get("width", 1)))
	mat.set_shader_parameter("fps", float(meta.get("fps", 15)))


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
