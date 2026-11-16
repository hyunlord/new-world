# Strict M-UX2 per-person framebuffer contrast and coverage oracle.
# Bridge methods: none. Inputs are actual GPU images and an independently enumerated ID set. M3D-SWITCH E2: a 3D
#   person's bounds are `box_bounds`, the screen rect of a box around their figure.
extends RefCounted

const CONTRAST: float = 0.20
const OWNERSHIP_EPSILON: float = 1.0 / 255.0


static func luminance(colour: Color) -> float:
	return colour.r * 0.2126 + colour.g * 0.7152 + colour.b * 0.0722


static func measure(full: Image, removed: Image, ground: Image, bounds: Rect2i, minimum: int, restored: Image = null) -> Dictionary:
	if full == null or removed == null or ground == null or full.get_size() != removed.get_size() or full.get_size() != ground.get_size():
		return {"valid": false, "pass": false, "reason": "missing_or_mismatched_image"}
	if restored != null and restored.get_size() != full.get_size():
		return {"valid": false, "pass": false, "reason": "mismatched_stability_image"}
	var visible: Rect2i = bounds.intersection(Rect2i(Vector2i.ZERO, full.get_size()))
	var owned: int = 0
	var qualifying: int = 0
	var unstable: int = 0
	var mask: Image = Image.create(maxi(1, visible.size.x), maxi(1, visible.size.y), false, Image.FORMAT_RGBA8)
	for y: int in range(visible.position.y, visible.end.y):
		for x: int in range(visible.position.x, visible.end.x):
			var colour: Color = full.get_pixel(x, y)
			if restored != null and colour != restored.get_pixel(x, y):
				unstable += 1
				continue
			var absent: Color = removed.get_pixel(x, y)
			var delta: float = maxf(absf(colour.r - absent.r), maxf(absf(colour.g - absent.g), absf(colour.b - absent.b)))
			if delta <= OWNERSHIP_EPSILON:
				continue
			owned += 1
			var passes: bool = absf(luminance(colour) - luminance(ground.get_pixel(x, y))) >= CONTRAST
			if passes:
				qualifying += 1
			mask.set_pixel(x - visible.position.x, y - visible.position.y, Color.WHITE if passes else Color.RED)
	return {"valid": true, "pass": qualifying >= minimum, "qualifying_pixels": qualifying,
		"owned_pixels": owned, "unstable_pixels": unstable, "required_pixels": minimum, "clipped": bounds != visible,
		"bounds": [bounds.position.x, bounds.position.y, bounds.size.x, bounds.size.y],
		"visible_bounds": [visible.position.x, visible.position.y, visible.size.x, visible.size.y], "mask": mask}


# R2 (M6A R2 brief 2026-10-30): only a person whose whole body bounds lie on screen is judged;
# a partly clipped person cannot show its full figure and is recorded as partial instead.
## The screen bounds (whole pixels, grown by `margin`) of the box `feet` ± `half_width` (x, z), from the feet up to
## `height`, through `camera`; an empty Rect2i when any corner is behind the camera (never judged as on screen).
static func box_bounds(camera: Camera3D, feet: Vector3, height: float, half_width: float, margin: float) -> Rect2i:
	if camera == null:
		return Rect2i()
	var out: Rect2 = Rect2()
	var first: bool = true
	for dy: float in [0.0, height]:
		for dx: float in [-half_width, half_width]:
			for dz: float in [-half_width, half_width]:
				var p: Vector3 = feet + Vector3(dx, dy, dz)
				if camera.is_position_behind(p):
					return Rect2i()
				var s: Vector2 = camera.unproject_position(p)
				out = Rect2(s, Vector2.ZERO) if first else out.expand(s)
				first = false
	out = out.grow(margin)
	return Rect2i(Vector2i(floori(out.position.x), floori(out.position.y)), Vector2i(ceili(out.end.x) - floori(out.position.x), ceili(out.end.y) - floori(out.position.y)))


static func judged(bounds: Rect2i, viewport: Rect2i) -> bool:
	return viewport.encloses(bounds)


static func coverage(expected: Array[int], rows: Array[Dictionary]) -> bool:
	if expected.is_empty() or rows.size() != expected.size():
		return false
	var seen: Dictionary = {}
	for row: Dictionary in rows:
		if not row.has("entity_bits") or not bool(row.get("valid", false)) or not bool(row.get("pass", false)):
			return false
		var bits: int = int(row["entity_bits"])
		if bits not in expected or seen.has(bits):
			return false
		seen[bits] = true
	return seen.size() == expected.size()
