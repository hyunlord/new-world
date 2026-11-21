# Style 2 house roof geometry and construction silhouette.
# Bridge methods called: none.
extends RefCounted

const Builder = preload("res://scripts/world3d/mesh_builder3d.gd")
const Data3D = preload("res://scripts/world3d/data3d.gd")
const Kit = preload("res://scripts/world3d/s2kit.gd")
const DIRS: Array[Vector2i] = [Vector2i.UP, Vector2i.RIGHT, Vector2i.DOWN, Vector2i.LEFT]


static func tiles(bounds: Rect2i) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	for y: int in range(bounds.position.y, bounds.end.y):
		for x: int in range(bounds.position.x, bounds.end.x):
			out.append(Vector2i(x, y))
	return out


const H2_STRAW: String = "res://assets/3d/anchor2/kit/h2.png"
const X1_STRAW: String = "res://assets/3d/anchor2/kit/x1.png"
const CLIP_EPSILON: float = 0.00001


static func _thatch(material: Material, eave: float, radial: bool, center: Vector2 = Vector2.ZERO, long_roof: bool = false) -> void:
	var shader: ShaderMaterial = material as ShaderMaterial
	if shader == null:
		return
	shader.set_shader_parameter("s2_thatch", true)
	shader.set_shader_parameter("thatch_eave", eave)
	shader.set_shader_parameter("radial_thatch", radial)
	shader.set_shader_parameter("cone_center", center)
	var tile_key: String = "s2kit.long_thatch_tile" if long_roof else "s2kit.house_thatch_tile"
	var tile_id: String = str(Data3D.binding(tile_key, ""))
	var albedo: Texture2D = Kit.texture(tile_id) if not radial and not tile_id.is_empty() else null
	if albedo != null:
		shader.set_shader_parameter("kit_straw", albedo)
		shader.set_shader_parameter("kit_patch", Vector4(0.006, 0.006, 0.988, 0.988))
		shader.set_shader_parameter("straw_normal", Kit.texture(tile_id, "_normal"))
		shader.set_shader_parameter("straw_rm", Kit.texture(tile_id, "_rm"))
		shader.set_shader_parameter("mapped_thatch", true)
		return
	shader.set_shader_parameter("kit_straw", load(X1_STRAW if radial else H2_STRAW) as Texture2D)
	# Interior straw islands, in atlas pixels. Exclude the seams of the baked kit UV unwrap.
	shader.set_shader_parameter("kit_patch", Vector4(113.0, 290.0, 15.0, 20.0) / 512.0 if radial
		else Vector4(473.0, 399.0, 27.0, 33.0) / 1024.0)


## A union of overlapping hip roofs. Each wing is the minimum of its planes;
## the visible roof is their upper envelope, clipped exactly at every valley.
static func hip(footprint: Array, ground: float, eave: float, pitch: float, rise_max: float, material: Material) -> ArrayMesh:
	_thatch(material, eave, false)
	var cells: Dictionary = {}
	for tile: Vector2i in footprint:
		cells[tile] = true
	var wings: Array[Rect2i] = _wings(cells)
	var b: Builder = Builder.new()
	b.base = ground
	for tile: Vector2i in cells:
		var active: Array = []
		for wing: Rect2i in wings:
			if wing.has_point(tile):
				active.append(_hip_planes(wing, rise_max / pitch))
		var start: Vector2 = Vector2(tile)
		var quad: PackedVector2Array = PackedVector2Array([start, start + Vector2.RIGHT, start + Vector2.ONE, start + Vector2.DOWN])
		for i: int in active.size():
			var roof: Array[Vector3] = active[i]
			for plane: Vector3 in roof:
				var face: PackedVector2Array = quad
				for neighbour: Vector3 in roof:
					if plane != neighbour:
						face = _clip(face, neighbour - plane)
				if face.size() < 3:
					continue
				var visible: Array[PackedVector2Array] = [face]
				for j: int in active.size():
					if i != j:
						visible = _above(visible, plane, active[j], i < j)
				for part: PackedVector2Array in visible:
					var a: Vector3 = _on_roof(part[0], plane, eave, pitch)
					for j: int in range(1, part.size() - 1):
						_slope(b, a, _on_roof(part[j], plane, eave, pitch), _on_roof(part[j + 1], plane, eave, pitch))
		for i: int in 4:
			if not cells.has(tile + DIRS[i]):
				var a: Vector2 = quad[i]
				var c: Vector2 = quad[(i + 1) % 4]
				var outward: Vector3 = Vector3(DIRS[i].x, 0.0, DIRS[i].y)
				_fringe(b, Vector3(a.x, eave, a.y), Vector3(c.x, eave, c.y), outward, 10, posmod(tile.x * 31 + tile.y * 7 + i, 23))
	return b.commit(material)


static func _wings(cells: Dictionary) -> Array[Rect2i]:
	var wings: Array[Rect2i] = []
	for start: Vector2i in cells:
		var height: int = 0
		var width: int = cells.size()
		while cells.has(start + Vector2i(0, height)):
			var length: int = 0
			while length < width and cells.has(start + Vector2i(length, height)):
				length += 1
			width = length
			height += 1
			var left: bool = _full_strip(cells, start + Vector2i.LEFT, Vector2i.DOWN, height)
			var top: bool = _full_strip(cells, start + Vector2i.UP, Vector2i.RIGHT, width)
			var bottom: bool = _full_strip(cells, start + Vector2i(0, height), Vector2i.RIGHT, width)
			if not (left or top or bottom):
				wings.append(Rect2i(start, Vector2i(width, height)))
	return wings


static func _full_strip(cells: Dictionary, start: Vector2i, step: Vector2i, count: int) -> bool:
	for i: int in count:
		if not cells.has(start + i * step):
			return false
	return true


static func _hip_planes(wing: Rect2i, cap: float) -> Array[Vector3]:
	return [Vector3(1, 0, -wing.position.x), Vector3(-1, 0, wing.end.x),
		Vector3(0, 1, -wing.position.y), Vector3(0, -1, wing.end.y), Vector3(0, 0, cap)]


## Subtract the convex region hidden by one wing. Identical planes belong to the earlier wing.
static func _above(faces: Array[PackedVector2Array], plane: Vector3, other: Array[Vector3], own_tie: bool) -> Array[PackedVector2Array]:
	if own_tie and other.has(plane):
		return faces
	var out: Array[PackedVector2Array] = []
	for part: PackedVector2Array in faces:
		var remainder: PackedVector2Array = part
		for neighbour: Vector3 in other:
			if plane == neighbour:
				continue
			var line: Vector3 = plane - neighbour
			var kept: PackedVector2Array = _clip(remainder, line)
			if kept.size() >= 3:
				out.append(kept)
			remainder = _clip(remainder, -line)
			if remainder.size() < 3:
				break
	return out


static func _clip(poly: PackedVector2Array, line: Vector3) -> PackedVector2Array:
	var out: PackedVector2Array = PackedVector2Array()
	if poly.is_empty():
		return out
	var a: Vector2 = poly[-1]
	var side_a: float = line.dot(Vector3(a.x, a.y, 1.0))
	for point: Vector2 in poly:
		var side_point: float = line.dot(Vector3(point.x, point.y, 1.0))
		if (side_a < -CLIP_EPSILON) != (side_point < -CLIP_EPSILON):
			out.append(a.lerp(point, clampf(side_a / (side_a - side_point), 0.0, 1.0)))
		if side_point >= -CLIP_EPSILON:
			out.append(point)
		a = point
		side_a = side_point
	return out


static func _on_roof(p: Vector2, plane: Vector3, eave: float, pitch: float) -> Vector3:
	return Vector3(p.x, eave + pitch * plane.dot(Vector3(p.x, p.y, 1.0)), p.y)


static func cone(center: Vector2, radius: Vector2, eave: float, rise: float, vent: bool, material: Material) -> ArrayMesh:
	_thatch(material, eave, true, center)
	var b: Builder = Builder.new()
	var hole: float = 0.082 if vent else 0.035
	var top: Vector3 = Vector3(center.x, eave + rise, center.y)
	var foot: Vector3 = Vector3(center.x, eave, center.y)
	var sides: int = 56
	var courses: int = maxi(4, roundi(rise / 0.29))
	for i: int in sides:
		var angle: float = TAU * i / sides
		var next_angle: float = TAU * (i + 1) / sides
		var d: Vector2 = Vector2.from_angle(angle)
		var next: Vector2 = Vector2.from_angle(next_angle)
		var a: Vector3 = foot + Vector3(d.x * radius.x, 0.0, d.y * radius.y) * _eave_spread(angle)
		var c: Vector3 = foot + Vector3(next.x * radius.x, 0.0, next.y * radius.y) * _eave_spread(next_angle)
		var out: Vector3 = Vector3(d.x + next.x, 0.0, d.y + next.y).normalized()
		var lo_a: Vector3 = a
		var lo_c: Vector3 = c
		for row: int in range(1, courses + 1):
			var hi_a: Vector3 = _round_course_point(top, a, angle, row, courses, hole)
			var hi_c: Vector3 = _round_course_point(top, c, next_angle, row, courses, hole)
			var lip: Vector3 = Vector3.UP * (0.011 + float(posmod(i * 9 + row, 5)) * 0.003)
			_course(b, lo_a, lo_c, hi_c, hi_a, lip, out, Color(0.87, 0.80, 0.65))
			lo_a = hi_a
			lo_c = hi_c
		_fringe(b, a, c, out, 2, i * 7)
	if vent:
		# A bound straw crown hugs the stone rim and leaves the opening clear.
		for i: int in 12:
			var a: Vector2 = Vector2.from_angle(TAU * i / 12) * (radius * hole + Vector2.ONE * 0.066)
			var c: Vector2 = Vector2.from_angle(TAU * (i + 1) / 12) * (radius * hole + Vector2.ONE * 0.066)
			var crown: Vector3 = top - Vector3.UP * (rise * hole + 0.018)
			b.log_prism(crown + Vector3(a.x, 0, a.y), crown + Vector3(c.x, 0, c.y), 0.047, Color(0.7, 0.63, 0.46))
	else:
		b.log_prism(top - Vector3.UP * 0.11, top + Vector3.UP * 0.19, 0.127, Color(0.75, 0.69, 0.53), 9)
		b.log_prism(top + Vector3.UP * 0.012, top + Vector3.UP * 0.048, 0.138, Color(0.48, 0.41, 0.28), 9)
	return b.commit(material)


static func _eave_spread(angle: float) -> float:
	return 1.0 + 0.020 * sin(angle * 7.0) + 0.016 * cos(angle * 15.0)


static func _round_course_point(top: Vector3, foot: Vector3, angle: float, row: int, courses: int, hole: float) -> Vector3:
	var ratio: float = float(row) / courses
	var spread: float = lerpf(1.0, hole, ratio)
	if row == courses:
		return top.lerp(top + (foot - top) / _eave_spread(angle), hole)
	var drift: float = sin(angle * 11.0 + row * 0.84) * 0.018 + cos(angle * 17.0 - row * 1.2) * 0.017
	return top.lerp(foot, spread + drift)


static func _course(b: Builder, a: Vector3, c: Vector3, hi_c: Vector3, hi_a: Vector3, lift: Vector3, out: Vector3, edge_colour: Color = Color(0.69, 0.61, 0.47)) -> void:
	_slope(b, a + lift, c + lift, hi_c)
	_slope(b, a + lift, hi_c, hi_a)
	b.quad_out(a, c, c + lift, a + lift, edge_colour, out)


static func _slope(b: Builder, a: Vector3, c: Vector3, d: Vector3) -> void:
	var n: Vector3 = (c - a).cross(d - a)
	b.profile = 0.0 if absf(n.z) >= absf(n.x) else 1.0
	b.tri_out(a, c, d, Color.WHITE, Vector3.UP)
	var drop: Vector3 = Vector3.DOWN * float(Data3D.param("structures.roof_thickness", 0.12))
	b.tri_out(a + drop, c + drop, d + drop, Color(0.64, 0.58, 0.46), Vector3.DOWN)


static func _fringe(b: Builder, a: Vector3, c: Vector3, outward: Vector3, count: int, seed: int) -> void:
	var thick: float = float(Data3D.param("structures.roof_thickness", 0.12))
	b.profile = 0.0 if absf(outward.z) >= absf(outward.x) else 1.0
	b.quad_out(a, c, c - Vector3.UP * thick, a - Vector3.UP * thick, Color(0.76, 0.68, 0.50), outward)
	for i: int in count:
		var n: int = posmod(seed * 19 + i * 73, 37)
		var width: float = 1.0 / count
		var start: Vector3 = a.lerp(c, i * width)
		var end: Vector3 = a.lerp(c, (i + 1) * width)
		var point: Vector3 = start.lerp(end, 0.25 + 0.01 * n) + outward * (0.06 + 0.002 * n) - Vector3.UP * (thick + 0.003 * n)
		var tone: Color = Color.WHITE.darkened(0.11 + 0.007 * n)
		# Bent tabs break up both the outer silhouette and the shadow under the eave.
		b.tri_out(start - outward * 0.08 + Vector3.UP * 0.08, end - outward * 0.08 + Vector3.UP * 0.08, point, tone, Vector3.UP + outward)


## The bounding rectangle uses the outer tile edges; members run on the wall centre line.
static func frame(b: Builder, bounds: Rect2i, ground: float, height: float, stage: int, progress: float, walls: Dictionary, doors: Dictionary) -> void:
	var wood: Color = Data3D.colour(Data3D.param("s2_house.timber", []), Color(0.47, 0.30, 0.16))
	var cord: Color = Color(0.68, 0.54, 0.34)
	var start: Vector2 = Vector2(bounds.position) + Vector2(0.5, 0.5)
	var end: Vector2 = Vector2(bounds.end) - Vector2(0.5, 0.5)
	var corners: Array[Vector2] = [start, Vector2(end.x, start.y), end, Vector2(start.x, end.y)]
	var top: float = ground + (0.25 if stage == 0 else height)
	b.base = ground
	for i: int in corners.size():
		var a: Vector2 = corners[i]
		var c: Vector2 = corners[(i + 1) % corners.size()]
		b.stake(a.x, a.y, ground - 0.05, top + 0.1, 0.045 if stage == 0 else 0.105, wood)
		b.log_prism(Vector3(a.x, top, a.y), Vector3(c.x, top, c.y), 0.013 if stage == 0 else 0.085, cord if stage == 0 else wood)
		if stage == 0:
			continue
		b.log_prism(Vector3(a.x, top, a.y), Vector3(lerpf(a.x, c.x, 0.22), top - 0.50, lerpf(a.y, c.y, 0.22)), 0.05, wood)
	if stage >= 1:
		_rafters(b, bounds, top, wood)
	if stage < 2:
		return
	var courses: int = clampi(ceili(7.0 * progress), 1, 7)
	b.profile = 1.0
	for tile: Vector2i in walls:
		if bool(walls[tile]) or doors.has(tile):
			continue
		var p: Vector3 = Vector3(tile.x + 0.5, ground, tile.y + 0.5)
		var along_x: bool = walls.has(tile + Vector2i.LEFT) or walls.has(tile + Vector2i.RIGHT)
		var along_z: bool = walls.has(tile + Vector2i.UP) or walls.has(tile + Vector2i.DOWN)
		for course: int in courses:
			p.y = ground + (course + 0.5) * height / 7.0
			if along_x:
				b.log_prism(p - Vector3.RIGHT * 0.52, p + Vector3.RIGHT * 0.52, height / 14.0, wood)
			if along_z:
				b.log_prism(p - Vector3.BACK * 0.52, p + Vector3.BACK * 0.52, height / 14.0, wood)
	b.profile = 3.0


static func _rafters(b: Builder, bounds: Rect2i, wall_top: float, wood: Color) -> void:
	var radius: Vector2 = Vector2(bounds.size) * 0.51
	var along: Vector3 = Vector3.RIGHT if radius.x > radius.y else Vector3.BACK
	var across: Vector3 = Vector3.BACK if radius.x > radius.y else Vector3.RIGHT
	var half_width: float = minf(bounds.size.x, bounds.size.y) * 0.5 - 0.5
	var half_length: float = maxf(bounds.size.x, bounds.size.y) * 0.5 - 0.5
	var pitch: float = tan(deg_to_rad(float(Data3D.param("structures.roof_pitch_deg", 38.0))))
	var rise: float = minf(float(Data3D.param("structures.roof_max_rise", 2.4)), minf(radius.x, radius.y) * pitch)
	var centre: Vector2 = Rect2(bounds).get_center()
	var foot: Vector3 = Vector3(centre.x, wall_top + 0.03, centre.y)
	var ridge: Vector3 = foot + Vector3.UP * (rise - 0.12)
	var previous: float = b.texture_tile
	b.profile = 1.0
	b.texture_tile = preload("res://scripts/world3d/textures3d.gd").index("styles", "log")
	for offset: float in [-half_length, 0.0, half_length]:
		var seat: Vector3 = foot + along * offset
		var peak: Vector3 = ridge + along * offset
		for side: float in [-1.0, 1.0]:
			b.log_prism(seat + across * side * half_width, peak, 0.071, wood.lightened(0.12))
	b.log_prism(ridge - along * (half_length + 0.07), ridge + along * (half_length + 0.07), 0.083, wood.lightened(0.09))
	b.texture_tile = previous
	b.profile = 3.0


static func gable(center: Vector2, radius: Vector2, eave: float, rise: float, material: Material, vents: Array[Vector3] = [],
		authored_ridge: bool = false, aperture_half: float = 0.12) -> ArrayMesh:
	_thatch(material, eave, false, Vector2.ZERO, maxf(radius.x, radius.y) >= 3.0)
	if material is ShaderMaterial:
		(material as ShaderMaterial).set_shader_parameter("gable_straw_cards", true)
	var b: Builder = Builder.new()
	var along: Vector3 = Vector3.RIGHT if radius.x > radius.y else Vector3.BACK
	var across: Vector3 = Vector3.BACK if radius.x > radius.y else Vector3.RIGHT
	var length: float = maxf(radius.x, radius.y)
	var width: float = maxf(0.2, minf(radius.x, radius.y))
	var base: Vector3 = Vector3(center.x, eave, center.y)
	var cuts: Array[float] = [-length, length]
	var holes: Array[Vector2] = []
	for vent_point: Vector3 in vents:
		var x: float = (vent_point - base).dot(along)
		var gap: Vector2 = Vector2(maxf(-length, x - aperture_half), minf(length, x + aperture_half))
		if gap.x < gap.y:
			holes.append(gap)
			cuts.append(gap.x)
			cuts.append(gap.y)
	cuts.sort()
	var courses: int = maxi(4, roundi(rise / 0.29))
	for side: float in [-1.0, 1.0]:
		var out: Vector3 = across * side
		var eaves: Vector3 = base + out * width
		_fringe(b, eaves - along * length, eaves + along * length, out, ceili(length * 21.0), 7 if side < 0.0 else 23)
		for i: int in range(1, cuts.size()):
			if is_equal_approx(cuts[i], cuts[i - 1]):
				continue
			var a: Vector3 = eaves + along * cuts[i - 1]
			var c: Vector3 = eaves + along * cuts[i]
			var centre: float = (cuts[i - 1] + cuts[i]) * 0.5
			var aperture: bool = holes.any(func(gap: Vector2) -> bool: return centre > gap.x and centre < gap.y)
			var limit: float = (1.0 - aperture_half / width) if aperture else 1.0
			var up: Vector3 = Vector3.UP * rise - out * width
			_slope(b, a, c, c + up * limit)
			_slope(b, a, c + up * limit, a + up * limit)
			_gable_straw_cards(b, a, c, up, out, limit, courses, i * 53 + (0 if side < 0.0 else 101))
			if i == 1:
				_fringe(b, a, a + up * limit, -along, ceili(up.length() * 14.0), 19)
			if i == cuts.size() - 1:
				_fringe(b, c, c + up * limit, along, ceili(up.length() * 14.0), 31)
			if side < 0.0 and not aperture and not authored_ridge:
				_ridge(b, base + along * cuts[i - 1] + Vector3.UP * rise, base + along * cuts[i] + Vector3.UP * rise)
	return b.commit(material)


static func _gable_straw_cards(b: Builder, a: Vector3, c: Vector3, up: Vector3, out: Vector3,
		limit: float, courses: int, seed: int) -> void:
	b.profile = 0.0 if absf(out.z) >= absf(out.x) else 1.0
	var span: Vector3 = c - a
	var columns: int = maxi(1, ceili(span.length() * 7.0))
	var course: float = 1.0 / courses
	for row: int in courses:
		var start: float = float(row) * course
		if start + course * 0.16 >= limit:
			break
		for col: int in columns:
			var n: int = posmod(seed + row * 37 + col * 53 + row * col * 13 + col * col * 7, 97)
			var m: int = posmod(n * 43 + row * 19, 89)
			var col_start: float = float(col)
			var col_end: float = float(col + 1)
			if row % 2 == 1:
				col_start -= 0.42
				col_end -= 0.42
				if col == columns - 1:
					col_end = float(columns)
			var left: Vector3 = a + span * (maxf(0.0, col_start - 0.07) / columns)
			var right: Vector3 = a + span * (minf(columns, col_end + 0.09) / columns)
			var tip: float = start - course * (0.13 + float(n) * 0.0041)
			var crest: float = minf(limit, start + course * 0.97)
			var shoulder: float = minf(crest - course * 0.025, start + course * (0.03 + float(m) * 0.0025))
			var depth: Vector3 = Vector3.UP * (0.026 + float(m) * 0.00032)
			var root: Vector3 = up * crest + Vector3.UP * 0.004
			var taper: Vector3 = (right - left) * (0.08 + float(n) * 0.0013)
			var ragged: Vector3 = left.lerp(right, 0.28 + float(m) * 0.005) + up * tip + depth
			var edge_a: Vector3 = left + taper + up * shoulder + depth * 0.68
			var edge_c: Vector3 = right - taper * 0.55 + up * maxf(tip, shoulder - course * 0.13) + depth * 0.84
			var shade: float = 0.85 + float(n) * 0.00155
			var straw: Color = Color(shade, shade * 0.98, shade * (0.92 + float(m) * 0.00065))
			b.tri_out(edge_a, ragged, edge_c, straw, Vector3.UP + out)
			b.quad_out(edge_a, edge_c, right + root, left + root, straw, Vector3.UP + out)


static func _ridge(b: Builder, a: Vector3, c: Vector3) -> void:
	b.log_prism(a, c, 0.115, Color(0.81, 0.73, 0.57), 8)
	var ties: int = maxi(1, roundi(a.distance_to(c) / 0.34))
	for i: int in ties:
		var middle: Vector3 = a.lerp(c, (float(i) + 0.5) / ties)
		var half: Vector3 = (c - a).normalized() * 0.018
		b.log_prism(middle - half, middle + half, 0.123, Color(0.53, 0.46, 0.32), 8)


static func gable_ends(b: Builder, center: Vector2, radius: Vector2, ground: float, eave: float, rise: float) -> void:
	var along: Vector3 = Vector3.RIGHT if radius.x > radius.y else Vector3.BACK
	var across: Vector3 = Vector3.BACK if radius.x > radius.y else Vector3.RIGHT
	var half_width: float = minf(radius.x, radius.y)
	var half_length: float = maxf(radius.x, radius.y)
	var wood: Color = Data3D.colour(Data3D.param("s2_house.timber", []), Color(0.47, 0.30, 0.16))
	var base: Vector3 = Vector3(center.x, eave, center.y)
	var run: float = maxf(0.1, half_width - 0.42)
	var wall_top: float = rise * 0.98
	var board_count: int = maxi(5, ceili(run / 0.15))
	for side: float in [-1.0, 1.0]:
		var end: Vector3 = base + along * side * (half_length - 0.45)
		for i: int in board_count:
			var left: float = lerpf(-run, run, float(i) / board_count)
			var right: float = lerpf(-run, run, float(i + 1) / board_count) - 0.01
			var lower_left: Vector3 = end + across * left - Vector3.UP * 0.02
			var lower_right: Vector3 = end + across * right - Vector3.UP * 0.02
			var upper_left: Vector3 = lower_left + Vector3.UP * (wall_top * (1.0 - absf(left) / half_width))
			var upper_right: Vector3 = lower_right + Vector3.UP * (wall_top * (1.0 - absf(right) / half_width))
			var tone: Color = Color(wood.lightened(0.04 * posmod(i * 7, 4)), 0.0)
			b.quad_out(lower_left, lower_right, upper_right, upper_left, tone, along * side)
		var bar: Vector3 = base + along * side * (half_length + 0.09)
		b.base = ground
		for cross_side: float in [-1.0, 1.0]:
			var toe: Vector3 = bar + across * cross_side * (half_width + 0.06)
			var tip: Vector3 = bar - across * cross_side * 0.31 + Vector3.UP * (rise + 0.27)
			b.log_prism(toe, tip, 0.043, Color(wood.lightened(0.10), 0.0))


## Construction uses the finished berm's inner line and the same opening angle.
static func round_frame(b: Builder, center: Vector2, radius: Vector2, ground: float, height: float, stage: int, progress: float, facing: Vector2) -> void:
	var wood: Color = Data3D.colour(Data3D.param("s2_house.timber", []), Color(0.47, 0.30, 0.16))
	var ring: Vector2 = radius - Vector2.ONE * 0.41
	var post_height: float = 0.28 if stage == 0 else height
	var top: Vector3 = Vector3(center.x, ground + post_height, center.y)
	var beam: float = 0.014 if stage == 0 else 0.052
	var prior_texture: float = b.texture_tile
	b.base = ground
	b.profile = 3.0
	for i: int in 16:
		var angle: float = TAU * i / 16
		var next_angle: float = TAU * (i + 1) / 16
		var here: Vector2 = Vector2.from_angle(angle) * ring
		var next: Vector2 = Vector2.from_angle(next_angle) * ring
		var a: Vector3 = top + Vector3(here.x, 0.0, here.y)
		var c: Vector3 = top + Vector3(next.x, 0.0, next.y)
		if absf(angle_difference(facing.angle(), angle)) >= _door_gap(radius):
			b.stake(a.x, a.z, ground - 0.03, a.y + 0.09, 0.04 if stage == 0 else 0.08, wood)
		if absf(angle_difference(facing.angle(), (angle + next_angle) * 0.5)) >= _door_gap(radius):
			b.log_prism(a, c, beam, wood.lightened(0.22) if stage == 0 else wood)
	if stage >= 2:
		earth_ring(b, center, radius, ground, height * clampf(progress / 0.7, 0.36, 1.0), facing)
	b.texture_tile = prior_texture
	b.profile = 3.0


static func _door_gap(radius: Vector2) -> float:
	return 0.49 / maxf(1.0, minf(radius.x, radius.y) - 0.3)


## A packed earth berm on the roof's elliptical plan, with the gap on the real door side.
static func earth_ring(b: Builder, center: Vector2, radius: Vector2, ground: float, height: float, facing: Vector2) -> void:
	var sides: int = 32
	var earth: Color = Data3D.colour(Data3D.param("s2_house.earth_ring", []), Color(0.44, 0.32, 0.21))
	var foot: Vector3 = Vector3(center.x, ground, center.y)
	b.base = ground
	b.profile = 3.0
	b.texture_tile = preload("res://scripts/world3d/textures3d.gd").index("surfaces", "earth")
	for i: int in sides:
		var theta: float = TAU * i / sides
		var a: Vector2 = Vector2.from_angle(theta)
		var c: Vector2 = Vector2.from_angle(TAU * (i + 1) / sides)
		var gap: float = _door_gap(radius)
		if absf(angle_difference(facing.angle(), theta + PI / sides)) < gap:
			continue
		var out: Vector3 = Vector3(a.x + c.x, 0.0, a.y + c.y)
		var floor_a: Vector3 = foot + Vector3(a.x * (radius.x - 0.11), -0.05, a.y * (radius.y - 0.11))
		var floor_c: Vector3 = foot + Vector3(c.x * (radius.x - 0.11), -0.05, c.y * (radius.y - 0.11))
		var top_a: Vector3 = foot + Vector3(a.x * (radius.x - 0.41), height, a.y * (radius.y - 0.41))
		var top_c: Vector3 = foot + Vector3(c.x * (radius.x - 0.41), height, c.y * (radius.y - 0.41))
		var inner_a: Vector3 = top_a - Vector3(a.x, 0.0, a.y) * 0.13
		var inner_c: Vector3 = top_c - Vector3(c.x, 0.0, c.y) * 0.13
		var col: Color = earth.lightened(float(posmod(i * 13, 7)) * 0.012)
		b.quad_out(floor_a, floor_c, top_c, top_a, col, out)
		b.quad_out(top_a, top_c, inner_c, inner_a, col.lightened(0.12), Vector3.UP)
		b.quad_out(inner_a, inner_c, Vector3(inner_c.x, ground, inner_c.z), Vector3(inner_a.x, ground, inner_a.z), col.darkened(0.07), -out)
		if i % 2 == 0:
			_berm_stone(b, floor_a, floor_c, i)


static func _berm_stone(b: Builder, a: Vector3, c: Vector3, index: int) -> void:
	var detail: float = b.texture_tile
	b.texture_tile = preload("res://scripts/world3d/textures3d.gd").index("surfaces", "stone")
	var tones: Color = Data3D.colour(Data3D.param("s2_house.footing", []), Color(0.48, 0.46, 0.41))
	var size: int = posmod(index * 11, 7)
	var edge: Vector3 = c - a
	var middle: Vector3 = (a + c) * 0.5 + Vector3.UP * (0.13 + size * 0.011)
	b.log_prism(middle - edge * 0.53, middle + edge * 0.63, 0.16 + 0.008 * size, tones.lightened(0.026 * size), 5)
	b.texture_tile = detail


static func smoke(origin: Vector3) -> CPUParticles3D:
	var p: CPUParticles3D = CPUParticles3D.new()
	p.position = origin
	p.amount = 7
	p.lifetime = 2.3
	p.preprocess = 1.2
	p.direction = Vector3(0.32, 1.0, 0.12)
	p.spread = 12.0
	p.initial_velocity_min = 0.48
	p.initial_velocity_max = 0.74
	p.gravity = Vector3.ZERO
	p.scale_amount_min = 0.65
	p.scale_amount_max = 1.3
	p.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	var fade: Gradient = Gradient.new()
	fade.colors = PackedColorArray([Color(0.53, 0.48, 0.39, 0.48), Color(0.76, 0.72, 0.64, 0.0)])
	p.color_ramp = fade
	var puff: SphereMesh = SphereMesh.new()
	puff.radius = 0.16
	puff.height = 0.32
	puff.radial_segments = 8
	puff.rings = 4
	var material: StandardMaterial3D = StandardMaterial3D.new()
	material.vertex_color_use_as_albedo = true
	material.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	puff.material = material
	p.mesh = puff
	return p


## Recover the enclosed tile mask from the site's rotated wall and door pieces.
## Anchor-only templates and a plan without an enclosure use the authored bounds.
static func enclosed_tiles(bounds: Rect2i, walls: Dictionary) -> Array[Vector2i]:
	if walls.is_empty():
		return tiles(bounds)
	var expanded: Rect2i = bounds.grow(1)
	var fringe: Array[Vector2i] = [expanded.position]
	var outside: Dictionary = {expanded.position: true}
	var cursor: int = 0
	while cursor < fringe.size():
		var cell: Vector2i = fringe[cursor]
		cursor += 1
		for direction: Vector2i in DIRS:
			var next: Vector2i = cell + direction
			if expanded.has_point(next) and not walls.has(next) and not outside.has(next):
				outside[next] = true
				fringe.append(next)
	var inside: Array[Vector2i] = []
	for tile: Vector2i in tiles(bounds):
		if not outside.has(tile):
			inside.append(tile)
	return inside if inside.size() > walls.size() else tiles(bounds)


static func angular_roof(footprint: Array, bounds: Rect2i, ground: float, eave: float, pitch: float, rise_max: float, material: Material,
		vents: Array[Vector3] = [], authored_ridge: bool = false, aperture_half: float = 0.12) -> ArrayMesh:
	if footprint.size() != bounds.get_area():
		return hip(footprint, ground, eave, pitch, rise_max, material)
	var radius: Vector2 = Vector2(bounds.size) * 0.51
	return gable(Rect2(bounds).get_center(), radius, eave, minf(rise_max, minf(radius.x, radius.y) * pitch), material, vents, authored_ridge, aperture_half)


static func vent(b: Builder, base: Vector3) -> void:
	var rim: Color = Color(0.29, 0.25, 0.20, 0.0)
	var pupil: Color = Color(0.08, 0.065, 0.055, 0.0)
	for i: int in 10:
		var a: Vector2 = Vector2.from_angle(TAU * i / 10.0)
		var c: Vector2 = Vector2.from_angle(TAU * (i + 1) / 10.0)
		var x: Vector3 = Vector3(a.x, 0.0, a.y)
		var z: Vector3 = Vector3(c.x, 0.0, c.y)
		var bottom: Vector3 = base - Vector3.UP * 0.12
		var top: Vector3 = base + Vector3.UP * 0.082
		b.quad_out(bottom + x * 0.18, bottom + z * 0.18, top + z * 0.15, top + x * 0.15, rim, x + z)
		b.quad_out(top + x * 0.15, top + z * 0.15, top + z * 0.1, top + x * 0.1, rim.lightened(0.13), Vector3.UP)
		b.tri_out(base + Vector3.UP * 0.045, top + z * 0.1, top + x * 0.1, pupil, Vector3.UP)
