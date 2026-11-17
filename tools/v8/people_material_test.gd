# people_material_test.gd — wardrobe VAT material coverage, including elder hair and both LODs.
# Bridge methods called: none. Headless resource checks; no pixels are rendered.
extends SceneTree

const Vat = preload("res://scripts/world3d/vat3d.gd")
var failures: int = 0

func _init() -> void:
	call_deferred("_run")

func _check(label: String, ok: bool) -> void:
	print("W3D_PEOPLE_MATERIAL %s %s" % [label, "PASS" if ok else "FAIL"])
	if not ok:
		failures += 1

func _run() -> void:
	var bindings: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://assets/3d/bindings3d.json"))
	var models: Dictionary = {}
	for era: Dictionary in (bindings["people"]["by_era"] as Dictionary).values():
		for model: String in era.values():
			models[model] = true
	for model: String in models:
		for suffix: String in ["", "_lod"]:
			var id: String = model + suffix
			var vat: RefCounted = Vat.load_character(id, true)
			_check(id + "_loads", bool(vat.get("ok")))
			if not bool(vat.get("ok")):
				continue
			var mat: ShaderMaterial = vat.call("with_findability", vat.call("material", Color(0.28, 0.2, 0.14)))
			_check(id + "_atlas", mat.get_shader_parameter("textured") == true and mat.get_shader_parameter("atlas") != null)
			var mesh: ArrayMesh = vat.get("mesh")
			_check(id + "_shared_surface", mesh.get_surface_count() == 1)
			var arrays: Array = mesh.surface_get_arrays(0)
			var uv: PackedVector2Array = arrays[Mesh.ARRAY_TEX_UV]
			var index: PackedInt32Array = arrays[Mesh.ARRAY_INDEX]
			var atlas: Image = (vat.get("atlas") as ImageTexture).get_image()
			atlas.convert(Image.FORMAT_RGBA8)
			var dark: int = 0
			for t: int in range(0, index.size(), 3):
				var centre: Vector2 = (uv[index[t]] + uv[index[t + 1]] + uv[index[t + 2]]) / 3.0
				var pixel: Vector2i = Vector2i((centre * Vector2(atlas.get_size())).floor()).clamp(Vector2i.ZERO, atlas.get_size() - Vector2i.ONE)
				var c: Color = atlas.get_pixelv(pixel)
				if maxf(c.r, maxf(c.g, c.b)) < 0.05:
					dark += 1
			# Most painted triangles must sample a real island rather than the atlas's black padding.
			_check(id + "_painted_uv", dark * 6 < index.size())
			# Wardrobe and body share the same animated surface and every findability pass.
			var outline: ShaderMaterial = mat.next_pass as ShaderMaterial
			if outline == null:
				_check(id + "_outline", false)
				continue
			var xray: ShaderMaterial = outline.next_pass as ShaderMaterial
			if xray == null:
				_check(id + "_xray", false)
				continue
			var day: Color = xray.get_shader_parameter("day_colour")
			var night: Color = xray.get_shader_parameter("night_colour")
			_check(id + "_xray_silhouette", maxf(day.r, maxf(day.g, day.b)) < 0.3
				and minf(night.r, minf(night.g, night.b)) > 0.7 and day.a >= 0.5 and night.a >= 0.5)
			_check(id + "_passes", outline != null and xray != null and
				outline.get_shader_parameter("vat") == vat.get("texture") and
				xray.get_shader_parameter("vat") == vat.get("texture"))
	print("W3D_PEOPLE_MATERIAL_DONE failures=%d models=%d" % [failures, models.size() * 2])
	quit(1 if failures else 0)
