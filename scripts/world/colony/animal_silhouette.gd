# Cached data-driven quadruped silhouettes for wild and penned animals.
# Bridge methods: none; immutable ContentContext supplies normalized geometry.
extends RefCounted

const View = preload("res://scripts/world/colony/view.gd")
const Content = preload("res://scripts/ui/content_context.gd")
const Icons = preload("res://scripts/ui/item_icons.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
static var _textures: Dictionary = {}
static var _params_version: int = -1


static func is_juvenile(row: Dictionary, age: int) -> bool:
	return age >= 0 and age < int(row.get("kid_age_days", 0))


## Populate both lifecycle sizes and ownership markers before their first visible use.
static func warmup() -> void:
	for row: Dictionary in Content.rows("animals"):
		for juvenile: bool in [false, true]:
			for penned: bool in [false, true]:
				texture(row, juvenile, penned)


static func texture(row: Dictionary, juvenile: bool = false, penned: bool = false) -> Texture2D:
	if _params_version != ArtData.params_version():
		_params_version = ArtData.params_version()
		_textures.clear()
	var key: String = "%s:%s:%s:%s" % [Content.revision(), hash(row), juvenile, penned]
	if _textures.has(key): return _textures[key]
	var scale_permille: int = int(row.get("art_juvenile_scale_permille", 600)) if juvenile else 1000
	var side: int = maxi(1, roundi(float(View.TILE * scale_permille) / 1000.0))
	var image: Image = Image.create(side, side, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)
	var tint: Color = Icons.material_color(str(row.get("art_tint_material", "")))
	var ink: Color = tint.darkened(0.72)
	var length: int = _dimension(row, "art_body_length_permille", 600, side)
	var height: int = _dimension(row, "art_body_height_permille", 280, side)
	var leg: int = _dimension(row, "art_leg_height_permille", 250, side)
	var head: int = maxi(4, roundi(side * 0.17))
	var left: int = maxi(1, floori((side - length - head) * 0.5))
	var foot: int = side - 2
	var top: int = foot - leg - height
	var leg_width: int = maxi(2, roundi(side * 0.055))
	for index: int in int(row.get("art_leg_count", 4)):
		var x: int = left + roundi(float(length - leg_width) * float([0.08, 0.28, 0.70, 0.91][index]))
		_box(image, Rect2i(x, top + height - 1, leg_width, leg + 1), tint.darkened(0.18 if index % 2 == 0 else 0.32), ink)
	var tail: int = _dimension(row, "art_tail_length_permille", 100, side)
	Icons._line(image, Vector2(left, top + height * 0.3), Vector2(maxi(0, left - tail), top + height * 0.65), ink)
	var body := PackedVector2Array([Vector2(left + 2, top), Vector2(left + length - 3, top), Vector2(left + length, top + 3), Vector2(left + length - 1, top + height - 2), Vector2(left + 3, top + height), Vector2(left, top + height - 3), Vector2(left, top + 3)])
	Icons._fill(image, body, tint)
	Icons._outline(image, body, ink)
	Icons._line(image, Vector2(left + 3, top + 2), Vector2(left + length - 4, top + 2), tint.lightened(0.18))
	var head_x: int = left + length - 2
	var head_y: int = top - maxi(1, roundi(height * 0.12))
	_box(image, Rect2i(head_x, head_y, head, maxi(4, roundi(height * 0.65))), tint.darkened(0.08), ink)
	Icons._line(image, Vector2(head_x + head - 3, head_y + 3), Vector2(head_x + head - 3, head_y + 3), ink)
	var ear := PackedVector2Array([Vector2(head_x + 1, head_y), Vector2(head_x - 2, head_y - 4), Vector2(head_x + 5, head_y - 1)])
	Icons._fill(image, ear, tint)
	Icons._outline(image, ear, ink)
	if bool(row.get("art_horns", false)):
		Icons._line(image, Vector2(head_x + 3, head_y), Vector2(head_x, head_y - side * 0.10), ink)
		Icons._line(image, Vector2(head_x, head_y - side * 0.10), Vector2(head_x - side * 0.06, head_y - side * 0.07), ink)
		Icons._line(image, Vector2(head_x + 6, head_y), Vector2(head_x + 9, head_y - side * 0.12), ink)
	if bool(row.get("art_antlers", false)):
		for spread: int in [-1, 1]:
			var base: Vector2 = Vector2(head_x + 4, head_y)
			var tip: Vector2 = base + Vector2(spread * side * 0.11, -side * 0.23)
			Icons._line(image, base, tip, ink)
			for branch: float in [0.45, 0.75]:
				var fork: Vector2 = base.lerp(tip, branch)
				Icons._line(image, fork, fork + Vector2(spread * side * 0.08, -side * 0.04), ink)
	if penned:
		var rope: Color = tint.lightened(0.48)
		var neck: int = head_x + 1
		Icons._line(image, Vector2(neck, head_y + 1), Vector2(neck, top + height - 1), ink)
		Icons._line(image, Vector2(neck + 1, head_y + 1), Vector2(neck + 1, top + height - 1), rope)
		_box(image, Rect2i(neck - 1, top + height - 1, 4, 4), rope, ink)
	if int(row.get("art_head_direction", 1)) < 0: image.flip_x()
	var result: ImageTexture = ImageTexture.create_from_image(image)
	_textures[key] = result
	return result


static func _dimension(row: Dictionary, key: String, fallback: int, side: int) -> int:
	return maxi(1, roundi(float(int(row.get(key, fallback)) * side) / 1000.0))


static func _box(image: Image, rect: Rect2i, fill: Color, ink: Color) -> void:
	var polygon := PackedVector2Array([Vector2(rect.position), Vector2(rect.end.x - 1, rect.position.y), Vector2(rect.end - Vector2i.ONE), Vector2(rect.position.x, rect.end.y - 1)])
	Icons._fill(image, polygon, fill)
	Icons._outline(image, polygon, ink)
