# lod_markers.gd (child "LodMarkers" of ColonyView/AgentLayer, made in code): the people's LOD markers (M-ART2 §3.5).
# Below `lod.people_below_zoom` every person is one filled disc of `lod.marker_px` SCREEN pixels in their top's colour
#   with a 1-px ink ring (`lod.ring_rgba`), and at `lod.glyph_zoom` a small activity glyph beside it (axe working, log
#   carrying, zz sleeping, ! fleeing). Both are MultiMeshInstance2D (one draw each, the instance buffer written in one
#   assignment per frame): MART2_DECISIONS D4's fallback step 2, taken after `draw_circle` per person cost ~7 ms of CPU
#   and held the 1,000-person world under 30 FPS. The disc and glyph textures are code-drawn at the screen size (no
#   scaling blur): the disc's red channel marks the inside (tinted per instance), the rest is the ink ring; the glyphs
#   are pixel bitmaps in one 4-cell atlas picked by the instance's custom data.
# Bridge methods called: none (agent_layer.gd feeds it).
extends Node2D

const INK: Color = Color8(35, 29, 23)
const GLYPH_PX: int = 14
## Glyph bitmaps, GLYPH_PX × GLYPH_PX: '#' ink, 'a' light, 'b' mid, '.' empty. Order = glyph index: work, carry, sleep, flee.
const GLYPHS: Array = [
	["..........###.", ".........#aaa#", "........#aaaa#", ".......#aaaa#.", "......##bb##..", ".....#bb#.....", "....#bb#......", "...#bb#.......", "..#bb#........", ".#bb#.........", "#bb#..........", "#b#...........", ".#............", ".............."],
	["..............", "..............", "..............", ".#########.##.", "#bbbbbbbbb#aa#", "#bbbbbbbbb#aa#", "#bbbbbbbbb#aa#", ".#########.##.", "..............", "..............", "..............", "..............", "..............", ".............."],
	["..............", "#######.......", "#aaaaa#.......", "####aa#.......", "...#a#........", "..#a#####.....", ".#aaa#aaa#....", "#######a#.....", ".....#a#......", "....#aaa#.....", "....#####.....", "..............", "..............", ".............."],
	[".....####.....", ".....#aa#.....", ".....#aa#.....", ".....#aa#.....", ".....#aa#.....", ".....#aa#.....", ".....#aa#.....", ".....####.....", "..............", ".....####.....", ".....#aa#.....", ".....####.....", "..............", ".............."],
]
const GLYPH_LIGHT: Array[Color] = [Color8(205, 210, 194), Color8(215, 171, 109), Color8(232, 231, 205), Color8(240, 192, 64)]
const GLYPH_MID: Array[Color] = [Color8(187, 139, 78), Color8(157, 109, 66), Color8(232, 231, 205), Color8(240, 192, 64)]
const MARKER_SHADER: String = """
shader_type canvas_item;
uniform vec4 ink : source_color = vec4(0.137, 0.114, 0.09, 1.0);
varying vec4 v_colour;
varying float v_cell;
void vertex() {
	v_colour = COLOR;
	v_cell = INSTANCE_CUSTOM.x;
}
void fragment() {
	vec4 t = texture(TEXTURE, UV);
	COLOR = vec4(mix(ink.rgb, v_colour.rgb, step(0.5, t.r)), t.a);
}
"""
const GLYPH_SHADER: String = """
shader_type canvas_item;
uniform float cells = 4.0;
varying float v_cell;
void vertex() {
	v_cell = INSTANCE_CUSTOM.x;
}
void fragment() {
	COLOR = texture(TEXTURE, vec2((UV.x + floor(v_cell + 0.5)) / cells, UV.y));
}
"""

var _discs: MultiMeshInstance2D = null
var _glyphs: MultiMeshInstance2D = null
var _disc_px: int = -1
var _ink: Color = INK


func _ready() -> void:
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_discs = _instance(MARKER_SHADER)
	_glyphs = _instance(GLYPH_SHADER)
	_glyphs.texture = _glyph_atlas()


func _quad() -> ArrayMesh:
	var arrays: Array = []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = PackedVector2Array([Vector2(-0.5, -0.5), Vector2(0.5, -0.5), Vector2(0.5, 0.5), Vector2(-0.5, 0.5)])
	arrays[Mesh.ARRAY_TEX_UV] = PackedVector2Array([Vector2(0, 0), Vector2(1, 0), Vector2(1, 1), Vector2(0, 1)])
	arrays[Mesh.ARRAY_INDEX] = PackedInt32Array([0, 1, 2, 0, 2, 3])
	var mesh: ArrayMesh = ArrayMesh.new()
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
	return mesh


func _instance(code: String) -> MultiMeshInstance2D:
	var mm: MultiMesh = MultiMesh.new()
	mm.transform_format = MultiMesh.TRANSFORM_2D
	mm.use_colors = true
	mm.use_custom_data = true
	mm.mesh = _quad()
	var node: MultiMeshInstance2D = MultiMeshInstance2D.new()
	node.multimesh = mm
	var shader: Shader = Shader.new()
	shader.code = code
	var mat: ShaderMaterial = ShaderMaterial.new()
	mat.shader = shader
	node.material = mat
	node.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	add_child(node)
	return node


func _disc_texture(px: int) -> Texture2D:
	var img: Image = Image.create(px, px, false, Image.FORMAT_RGBA8)
	var r: float = px * 0.5
	for y: int in px:
		for x: int in px:
			var d: float = Vector2(x + 0.5 - r, y + 0.5 - r).length()
			if d <= r:
				img.set_pixel(x, y, Color(1.0 if d < r - 1.0 else 0.0, 0, 0, 1))
	return ImageTexture.create_from_image(img)


func _glyph_atlas() -> Texture2D:
	var img: Image = Image.create(GLYPH_PX * GLYPHS.size(), GLYPH_PX, false, Image.FORMAT_RGBA8)
	for g: int in GLYPHS.size():
		var rows: Array = GLYPHS[g]
		for y: int in GLYPH_PX:
			var row: String = str(rows[y])
			for x: int in mini(GLYPH_PX, row.length()):
				var c: String = row[x]
				if c == "#":
					img.set_pixel(g * GLYPH_PX + x, y, INK)
				elif c == "a":
					img.set_pixel(g * GLYPH_PX + x, y, GLYPH_LIGHT[g])
				elif c == "b":
					img.set_pixel(g * GLYPH_PX + x, y, GLYPH_MID[g])
	return ImageTexture.create_from_image(img)


## Show one disc per point (world positions, colours) of `marker_px` screen pixels at camera zoom `zoom`, and a
## glyph (index into GLYPHS, −1 none) beside each when `glyphs` is on.
func show_markers(points: PackedVector2Array, colours: PackedColorArray, glyph_ids: PackedInt32Array, marker_px: int, zoom: float, glyphs: bool, ink: Color) -> void:
	visible = true
	if marker_px != _disc_px or ink != _ink:
		_disc_px = marker_px
		_ink = ink
		_discs.texture = _disc_texture(maxi(2, marker_px))
		(_discs.material as ShaderMaterial).set_shader_parameter("ink", ink)
	var scale: float = float(marker_px) / maxf(zoom, 0.01)
	var n: int = mini(points.size(), colours.size())
	var buf: PackedFloat32Array = PackedFloat32Array()
	buf.resize(n * 16)
	for i: int in n:
		var p: Vector2 = points[i]
		var c: Color = colours[i]
		var o: int = i * 16
		buf[o] = scale
		buf[o + 3] = p.x
		buf[o + 5] = scale
		buf[o + 7] = p.y
		buf[o + 8] = c.r
		buf[o + 9] = c.g
		buf[o + 10] = c.b
		buf[o + 11] = 1.0
	var mm: MultiMesh = _discs.multimesh
	if mm.instance_count != n:
		mm.instance_count = n
	if n > 0:
		mm.buffer = buf
	var gm: MultiMesh = _glyphs.multimesh
	if not glyphs:
		gm.instance_count = 0
		return
	var g_scale: float = float(GLYPH_PX) / maxf(zoom, 0.01)
	var offset: Vector2 = Vector2(scale * 0.5 + g_scale * 0.5, -scale * 0.5 - g_scale * 0.3)
	var gbuf: PackedFloat32Array = PackedFloat32Array()
	var count: int = 0
	for i: int in mini(n, glyph_ids.size()):
		if glyph_ids[i] < 0:
			continue
		var p: Vector2 = points[i] + offset
		gbuf.append_array([g_scale, 0.0, 0.0, p.x, 0.0, g_scale, 0.0, p.y, 1.0, 1.0, 1.0, 1.0, float(glyph_ids[i]), 0.0, 0.0, 0.0])
		count += 1
	if gm.instance_count != count:
		gm.instance_count = count
	if count > 0:
		gm.buffer = gbuf


## Hide both.
func hide_markers() -> void:
	visible = false


## Instances shown (tests): [discs, glyphs].
func counts() -> Array[int]:
	return [_discs.multimesh.instance_count if visible else 0, _glyphs.multimesh.instance_count if visible else 0]
