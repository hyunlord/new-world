# Strict M-UX2 per-person framebuffer contrast and coverage oracle.
# Bridge methods: none. Inputs are actual GPU images and an independently enumerated ID set.
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
