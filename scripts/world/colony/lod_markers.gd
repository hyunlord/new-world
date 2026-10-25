# lod_markers.gd (child "LodMarkers" of ColonyView/AgentLayer, made in code): the people's LOD markers (M-ART2 §3.5).
# At or below `lod.marker_zoom_max` every person is a filled disc of `lod.marker_fill_px` SCREEN pixels,
#   with a bright ring, household-colour outer ring, and dark outer outline, all unshaded for night readability.
#   `lod.household_ring_rgb` supplies household colours; at `lod.glyph_zoom` a small activity glyph sits beside it (axe working, log
#   carrying, zz sleeping, ! fleeing). Both are MultiMeshInstance2D (one draw each, the instance buffer written in one
#   assignment per frame): MART2_DECISIONS D4's fallback step 2, taken after `draw_circle` per person cost ~7 ms of CPU
#   and held the 1,000-person world under 30 FPS. The disc and glyph textures are code-drawn at the screen size (no
#   scaling blur): the disc's red channel marks the inside (tinted per instance), the rest is the ink ring; the glyphs
#   are pixel bitmaps in one 4-cell atlas picked by the instance's custom data.
# Bridge methods called: none (agent_layer.gd feeds it).
extends Node2D

const Layout = preload("res://scripts/world/colony/marker_layout.gd")
const INK: Color = Color8(35, 29, 23)
const GLYPH_PX: int = 14
## Floats per MultiMesh instance in the buffer: a 2D transform (8), a colour (4), custom data (4).
const FLOATS_PER_INSTANCE: int = 16
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
render_mode unshaded;
uniform vec4 ink : source_color = vec4(0.137, 0.114, 0.09, 1.0);
uniform vec4 light_ink : source_color = vec4(0.949, 0.933, 0.875, 1.0);
varying vec4 v_colour;
varying vec4 v_ring;
void vertex() {
	v_colour = COLOR;
	v_ring = INSTANCE_CUSTOM;
}
void fragment() {
	vec4 t = texture(TEXTURE, UV);
	vec3 household = v_ring.a > 0.5 ? v_ring.rgb : light_ink.rgb;
	vec3 colour = t.r > 0.5 ? v_colour.rgb : (t.g > 0.5 ? light_ink.rgb : (t.b > 0.5 ? household : ink.rgb));
	COLOR = vec4(colour, t.a);
}
"""
const GLYPH_SHADER: String = """
shader_type canvas_item;
render_mode unshaded;
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
var _profile: Array = []
var _total_px: int = 0
var _anchors: PackedVector2Array = PackedVector2Array()
var _points: PackedVector2Array = PackedVector2Array()
var _leader_width: float = 1.0


func _ready() -> void:
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	var leader_material: CanvasItemMaterial = CanvasItemMaterial.new()
	leader_material.light_mode = CanvasItemMaterial.LIGHT_MODE_UNSHADED
	material = leader_material
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


func _disc_texture(fill_px: int, bright_px: int, household_px: int, outline_px: int) -> Texture2D:
	var px: int = fill_px + 2 * (bright_px + household_px + outline_px)
	var img: Image = Image.create(px, px, false, Image.FORMAT_RGBA8)
	var r: float = px * 0.5
	var fill_radius: float = fill_px * 0.5
	for y: int in px:
		for x: int in px:
			var d: float = Vector2(x + 0.5 - r, y + 0.5 - r).length()
			if d <= r:
				var colour: Color = Color(0, 0, 0, 1)
				if d <= fill_radius:
					colour.r = 1.0
				elif d <= fill_radius + bright_px:
					colour.g = 1.0
				elif d <= fill_radius + bright_px + household_px:
					colour.b = 1.0
				img.set_pixel(x, y, colour)
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


## Show one disc per point with `marker_px` filled screen pixels plus the configured ring bands.
## Household alpha0 uses neutral light ink. Optional glyph IDs refer to GLYPHS; −1 means none.
func show_markers(points: PackedVector2Array, colours: PackedColorArray, glyph_ids: PackedInt32Array, marker_px: int, zoom: float, glyphs: bool, ink: Color, rings: PackedColorArray = PackedColorArray(), style: Dictionary = {}, stable_ids: PackedInt64Array = PackedInt64Array()) -> void:
	visible = true
	var fill_px: int = clampi(marker_px, 2, 64)
	var bright_px: int = clampi(int(style.get("marker_light_ring_px", 2)), 1, 8)
	var household_px: int = clampi(int(style.get("marker_household_ring_px", 2)), 1, 8)
	var outline_px: int = clampi(int(style.get("marker_outline_px", 1)), 1, 4)
	var light_value: float = clampf(float(style.get("marker_light_value", 0.95)), 0.85, 1.0)
	var light_ink: Color = Color(light_value, light_value, light_value)
	var profile: Array = [fill_px, bright_px, household_px, outline_px, light_ink, ink]
	if profile != _profile:
		_profile = profile
		_total_px = fill_px + 2 * (bright_px + household_px + outline_px)
		_discs.texture = _disc_texture(fill_px, bright_px, household_px, outline_px)
		(_discs.material as ShaderMaterial).set_shader_parameter("ink", ink)
		(_discs.material as ShaderMaterial).set_shader_parameter("light_ink", light_ink)
	var scale: float = float(_total_px) / maxf(zoom, 0.01)
	var n: int = mini(points.size(), colours.size())
	var screen: PackedVector2Array = PackedVector2Array()
	var canvas: Transform2D = get_global_transform_with_canvas()
	for point: Vector2 in points:
		screen.append(canvas * point)
	if stable_ids.size() != points.size():
		stable_ids.resize(points.size())
		for i: int in stable_ids.size():
			stable_ids[i] = i
	var blocked: Array[Rect2] = []
	var ui: Node = get_node_or_null("/root/Main/UI")
	if ui != null:
		Layout.ui_rects(ui, blocked)
	var placed: PackedVector2Array = Layout.place(screen, stable_ids, float(_total_px), glyphs, get_viewport_rect(), blocked, float(style.get("marker_gap_px", 4)))
	_anchors = points.duplicate()
	_points.resize(points.size())
	var inverse: Transform2D = canvas.affine_inverse()
	for i: int in placed.size():
		_points[i] = inverse * placed[i]
	_leader_width = 1.0 / maxf(zoom, 0.01)
	queue_redraw()
	var buf: PackedFloat32Array = PackedFloat32Array()
	buf.resize(n * FLOATS_PER_INSTANCE)
	for i: int in n:
		var p: Vector2 = _points[i]
		var c: Color = colours[i]
		var minimum_luminance: float = clampf(float(style.get("marker_fill_luminance", 0.85)), 0.0, 0.95)
		var luminance: float = c.r * 0.2126 + c.g * 0.7152 + c.b * 0.0722
		if luminance < minimum_luminance:
			c = c.lerp(Color.WHITE, (minimum_luminance - luminance) / maxf(0.001, 1.0 - luminance))
		var o: int = i * FLOATS_PER_INSTANCE
		buf[o] = scale
		buf[o + 3] = p.x
		buf[o + 5] = scale
		buf[o + 7] = p.y
		buf[o + 8] = c.r
		buf[o + 9] = c.g
		buf[o + 10] = c.b
		buf[o + 11] = 1.0
		if i < rings.size():
			var ring: Color = rings[i]
			buf[o + 12] = ring.r
			buf[o + 13] = ring.g
			buf[o + 14] = ring.b
			buf[o + 15] = ring.a
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
		var p: Vector2 = _points[i] + offset
		gbuf.append_array([g_scale, 0.0, 0.0, p.x, 0.0, g_scale, 0.0, p.y, 1.0, 1.0, 1.0, 1.0, float(glyph_ids[i]), 0.0, 0.0, 0.0])
		count += 1
	if gm.instance_count != count:
		gm.instance_count = count
	if count > 0:
		gm.buffer = gbuf


## Hide both.
func hide_markers() -> void:
	visible = false


## Full screen-space extent, including the fill and all three concentric ring bands.
func total_px() -> int:
	return _total_px


func point_at(index: int) -> Vector2:
	return _points[index] if index >= 0 and index < _points.size() else Vector2.ZERO


func _draw() -> void:
	for i: int in mini(_anchors.size(), _points.size()):
		if _anchors[i].distance_squared_to(_points[i]) > 1.0:
			draw_line(_anchors[i], _points[i], INK, 3.0 * _leader_width)
			draw_line(_anchors[i], _points[i], Color(0.9, 0.9, 0.85), _leader_width)


## The ring colour of disc instance `i` (alpha 0 = the ink ring; tests).
func ring_of(i: int) -> Color:
	var mm: MultiMesh = _discs.multimesh
	return mm.get_instance_custom_data(i) if i >= 0 and i < mm.instance_count else Color(0, 0, 0, 0)


## Instances shown (tests): [discs, glyphs].
func counts() -> Array[int]:
	return [_discs.multimesh.instance_count if visible else 0, _glyphs.multimesh.instance_count if visible else 0]
