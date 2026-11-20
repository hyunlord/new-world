# Style 2 house roof geometry and construction silhouette.
# Bridge methods called: none.
extends RefCounted

const Builder = preload("res://scripts/world3d/mesh_builder3d.gd")
const Data3D = preload("res://scripts/world3d/data3d.gd")
const DIRS: Array[Vector2i] = [Vector2i.UP, Vector2i.RIGHT, Vector2i.DOWN, Vector2i.LEFT]


static func tiles(bounds: Rect2i) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	for y: int in range(bounds.position.y, bounds.end.y):
		for x: int in range(bounds.position.x, bounds.end.x):
			out.append(Vector2i(x, y))
	return out


## Offset fronts on a quarter-metre lattice meet at ridges and valleys. The entire
## footprint is one height field, including the re-entrant corners of L and U plans.
static func hip(footprint: Array, ground: float, eave: float, pitch: float, rise_max: float, material: Material) -> ArrayMesh:
	var cells: Dictionary = {}
	for tile: Vector2i in footprint:
		for y: int in 4:
			for x: int in 4:
				cells[tile * 4 + Vector2i(x, y)] = true
	var vertices: Dictionary = {}
	for cell: Vector2i in cells:
		for offset: Vector2i in [Vector2i.ZERO, Vector2i.RIGHT, Vector2i.DOWN, Vector2i.ONE]:
			vertices[cell + offset] = INF
	var frontier: Array[Vector2i] = []
	for v: Vector2i in vertices:
		if not (cells.has(v) and cells.has(v - Vector2i.ONE) and cells.has(v - Vector2i.RIGHT) and cells.has(v - Vector2i.DOWN)):
			frontier.append(v)
			vertices[v] = 0.0
	var cursor: int = 0
	while cursor < frontier.size():
		var v: Vector2i = frontier[cursor]
		cursor += 1
		var distance: float = float(vertices[v]) + 0.25
		for y: int in [-1, 0, 1]:
			for x: int in [-1, 0, 1]:
				var n: Vector2i = v + Vector2i(x, y)
				if vertices.has(n) and float(vertices[n]) > distance:
					vertices[n] = distance
					frontier.append(n)
	var point: Callable = func(v: Vector2i) -> Vector3:
		return Vector3(v.x * 0.25, eave + minf(rise_max, float(vertices[v]) * pitch), v.y * 0.25)
	var b: Builder = Builder.new()
	b.base = ground
	for c: Vector2i in cells:
		var corners: Array[Vector3] = [point.call(c), point.call(c + Vector2i.RIGHT), point.call(c + Vector2i.ONE), point.call(c + Vector2i.DOWN)]
		var diagonal: int = 0 if absf(corners[0].y - corners[2].y) <= absf(corners[1].y - corners[3].y) else 1
		_slope(b, corners[diagonal], corners[(diagonal + 1) % 4], corners[(diagonal + 2) % 4])
		_slope(b, corners[diagonal], corners[(diagonal + 2) % 4], corners[(diagonal + 3) % 4])
		for i: int in 4:
			if not cells.has(c + DIRS[i]):
				_fascia(b, corners[i], corners[(i + 1) % 4], Vector3(DIRS[i].x, 0.0, DIRS[i].y))
	return b.commit(material)


static func cone(center: Vector2, radius: Vector2, eave: float, rise: float, vent: bool, material: Material) -> ArrayMesh:
	var b: Builder = Builder.new()
	var hole: float = 0.075 if vent else 0.0
	var top: Vector3 = Vector3(center.x, eave + rise, center.y)
	var foot: Vector3 = Vector3(center.x, eave, center.y)
	var sides: int = 32
	for i: int in sides:
		var d: Vector2 = Vector2.from_angle(TAU * i / sides)
		var next: Vector2 = Vector2.from_angle(TAU * (i + 1) / sides)
		var a: Vector3 = foot + Vector3(d.x * radius.x, 0.0, d.y * radius.y)
		var c: Vector3 = foot + Vector3(next.x * radius.x, 0.0, next.y * radius.y)
		var inner_a: Vector3 = top.lerp(a, hole)
		var inner_c: Vector3 = top.lerp(c, hole)
		_slope(b, a, c, inner_c)
		if vent:
			_slope(b, a, inner_c, inner_a)
		_fascia(b, a, c, Vector3(d.x + next.x, 0.0, d.y + next.y))
	return b.commit(material)


static func _slope(b: Builder, a: Vector3, c: Vector3, d: Vector3) -> void:
	var n: Vector3 = (c - a).cross(d - a)
	b.profile = 0.0 if absf(n.z) >= absf(n.x) else 1.0
	b.tri_out(a, c, d, Color.WHITE, Vector3.UP)
	var drop: Vector3 = Vector3.DOWN * float(Data3D.param("structures.roof_thickness", 0.12))
	b.tri_out(a + drop, c + drop, d + drop, Color(0.64, 0.58, 0.46), Vector3.DOWN)


static func _fascia(b: Builder, a: Vector3, c: Vector3, outward: Vector3) -> void:
	var drop: Vector3 = Vector3.DOWN * float(Data3D.param("structures.roof_thickness", 0.12))
	b.quad_out(a, c, c + drop, a + drop, Color(0.82, 0.75, 0.61), outward)


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


static func gable(center: Vector2, radius: Vector2, eave: float, rise: float, material: Material) -> ArrayMesh:
	var b: Builder = Builder.new()
	var along: Vector3 = Vector3.RIGHT * radius.x if radius.x > radius.y else Vector3.BACK * radius.y
	var across: Vector3 = Vector3.BACK * radius.y if radius.x > radius.y else Vector3.RIGHT * radius.x
	var base: Vector3 = Vector3(center.x, eave, center.y)
	for side: float in [-1.0, 1.0]:
		var a: Vector3 = base + across * side - along
		var c: Vector3 = base + across * side + along
		var upper: Vector3 = base + Vector3.UP * rise
		_slope(b, a, c, upper + along)
		_slope(b, a, upper + along, upper - along)
		_fascia(b, a, c, across * side)
	return b.commit(material)


static func gable_ends(b: Builder, center: Vector2, radius: Vector2, ground: float, eave: float, rise: float) -> void:
	var along: Vector3 = Vector3.RIGHT if radius.x > radius.y else Vector3.BACK
	var across: Vector3 = Vector3.BACK if radius.x > radius.y else Vector3.RIGHT
	var half_width: float = minf(radius.x, radius.y)
	var half_length: float = maxf(radius.x, radius.y)
	var wood: Color = Data3D.colour(Data3D.param("s2_house.timber", []), Color(0.47, 0.30, 0.16))
	var base: Vector3 = Vector3(center.x, eave, center.y)
	var run: float = maxf(0.1, half_width - 0.42)
	var wall_top: float = rise * 0.98
	var wall_y: Vector3 = Vector3.UP * (wall_top * (1.0 - run / half_width))
	for side: float in [-1.0, 1.0]:
		var end: Vector3 = base + along * side * (half_length - 0.45)
		b.tri_out(end - across * run + wall_y, end + across * run + wall_y, end + Vector3.UP * wall_top, Color(wood, 0.0), along * side)
		b.quad_out(end - across * run, end + across * run, end + across * run + wall_y, end - across * run + wall_y, Color(wood, 0.0), along * side)
		var bar: Vector3 = base + along * side * (half_length + 0.09)
		b.base = ground
		for cross_side: float in [-1.0, 1.0]:
			var toe: Vector3 = bar + across * cross_side * (half_width + 0.06)
			var tip: Vector3 = bar - across * cross_side * 0.31 + Vector3.UP * (rise + 0.27)
			b.log_prism(toe, tip, 0.043, Color(wood.lightened(0.10), 0.0))


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
		var gap: float = 0.49 / maxf(1.0, minf(radius.x, radius.y) - 0.3)
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


static func angular_roof(footprint: Array, bounds: Rect2i, ground: float, eave: float, pitch: float, rise_max: float, material: Material) -> ArrayMesh:
	if footprint.size() != bounds.get_area():
		return hip(footprint, ground, eave, pitch, rise_max, material)
	var radius: Vector2 = Vector2(bounds.size) * 0.51
	return gable(Rect2(bounds).get_center(), radius, eave, minf(rise_max, minf(radius.x, radius.y) * pitch), material)


static func vent(b: Builder, base: Vector3) -> void:
	var rim: Color = Color(0.29, 0.25, 0.20, 0.0)
	var pupil: Color = Color(0.08, 0.065, 0.055, 0.0)
	for i: int in 10:
		var a: Vector2 = Vector2.from_angle(TAU * i / 10.0)
		var c: Vector2 = Vector2.from_angle(TAU * (i + 1) / 10.0)
		var x: Vector3 = Vector3(a.x, 0.0, a.y)
		var z: Vector3 = Vector3(c.x, 0.0, c.y)
		var bottom: Vector3 = base - Vector3.UP * 0.12
		var top: Vector3 = base + Vector3.UP * 0.16
		b.quad_out(bottom + x * 0.18, bottom + z * 0.18, top + z * 0.15, top + x * 0.15, rim, x + z)
		b.quad_out(top + x * 0.15, top + z * 0.15, top + z * 0.1, top + x * 0.1, rim.lightened(0.13), Vector3.UP)
		b.tri_out(base + Vector3.UP * 0.12, top + z * 0.1, top + x * 0.1, pupil, Vector3.UP)
