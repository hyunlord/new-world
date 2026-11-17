# Item icons: authored art first, then data-selected placeholder shapes/material tint.
# Also owns the shared raster helper for template roof/glyph previews and real sites.
# Bridge methods: none; immutable ContentContext metadata supplies presentation data.
extends RefCounted

const ArtData = preload("res://scripts/world/art/art_data.gd")
const Content = preload("res://scripts/ui/content_context.gd")
const MaterialResolver = preload("res://scripts/world/art/material_resolver.gd")
const Tone = preload("res://scripts/world/art/tone.gd")
const THEME = preload("res://assets/ui/worldsim_theme.tres")
const GLYPHS_PATH: String = "res://assets/ui/glyphs.json"
static var _textures: Dictionary = {}
static var _glyphs: Dictionary = {}
static var _appearance: Dictionary = {}
static var _resolver: MaterialResolver = null
static var _params_version: int = -1
static var _glyphs_loaded: bool = false


## Preserve the established manifest API, including unknown-family fallback.
static func asset_of(code: int) -> String:
	return _asset(ArtData.binding("items", code))


static func _asset(row: Dictionary) -> String:
	if row.get("asset") is String:
		return str(row["asset"])
	var list: Variant = row.get("assets", [])
	if list is Array and not list.is_empty():
		return str(list[0])
	return ""


static func _authored_asset(code: int) -> String:
	# binding() includes fallbacks; scan the authored rows to distinguish a real icon.
	for value: Variant in ArtData.bindings().get("items", {}).get("codes", []):
		if value is Dictionary and int(value.get("code", -1)) == code:
			return _asset(value)
	return ""


static func icon(code: int, size: int = 32) -> TextureRect:
	var rect: TextureRect = TextureRect.new()
	rect.name = "ItemIcon"
	rect.custom_minimum_size = Vector2(size, size)
	rect.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	rect.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	rect.mouse_filter = Control.MOUSE_FILTER_IGNORE
	rect.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR
	rect.texture = texture_of(code)
	if uses_placeholder(code): rect.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	rect.set_meta("item_kind", code)
	return rect


static func uses_placeholder(code: int) -> bool:
	return _authored_asset(code).is_empty() and Content.row("items", code).has("art_shape")


## Texture consumer seam shared by buttons, panel icons and loose-world piles.
static func texture_of(code: int) -> Texture2D:
	var authored: String = _authored_asset(code)
	if not authored.is_empty():
		var texture: Texture2D = ArtData.texture(authored)
		if texture != null: return texture
	if uses_placeholder(code): return item_texture(Content.row("items", code))
	return ArtData.texture(asset_of(code))


static func _load_appearance() -> void:
	if _glyphs_loaded: return
	_glyphs_loaded = true
	var value: Variant = JSON.parse_string(FileAccess.get_file_as_string(GLYPHS_PATH))
	if value is Dictionary:
		_appearance = value
		_glyphs = value.get("glyphs", {})


static func material_color(name: String) -> Color:
	_load_appearance()
	if _resolver == null or _params_version != ArtData.params_version():
		_params_version = ArtData.params_version()
		_resolver = MaterialResolver.new()
		_resolver.load_imported()
		_textures.clear()
	var resolved: Dictionary = _resolver.resolve("", name)
	var category: String = str(resolved.get("category", ""))
	var token: String = str(_appearance.get("material_category_tokens", {}).get(category, "Label"))
	var base: Color = THEME.get_color("font_color", token)
	var tint: Dictionary = resolved.get("tint", {})
	var lab: Vector3 = Tone.rgb_to_lab(Vector3(base.r, base.g, base.b))
	lab += Vector3(float(tint.get("dL", 0)), float(tint.get("da", 0)), float(tint.get("db", 0)))
	var rgb: Vector3 = Tone.lab_to_rgb(lab)
	return Color(clampf(rgb.x, 0.0, 1.0), clampf(rgb.y, 0.0, 1.0), clampf(rgb.z, 0.0, 1.0), 1.0)


## Prepare the selected catalogue through the same dimension guards and cache as actual consumers.
static func warmup_catalog(tile_px: int) -> void:
	if tile_px <= 0: return
	for row: Dictionary in Content.rows("items"):
		if uses_placeholder(int(row.get("code", -1))): item_texture(row)
	for row: Dictionary in Content.rows("outcrops"):
		item_texture(row)
	for row: Dictionary in Content.rows("buildings"):
		var pixels: Vector2i = Vector2i(int(row.get("width", 0)), int(row.get("height", 0))) * tile_px
		building_texture(row, pixels, true)
		if pixels.x != pixels.y: building_texture(row, Vector2i(pixels.y, pixels.x), true)
		building_texture(row, Vector2i(48, 40), true)


static func item_texture(row: Dictionary) -> Texture2D:
	var shape: String = str(row.get("art_shape", "lump")).to_lower()
	var tint: Color = material_color(str(row.get("art_tint_material", "")))
	var key: String = "item/%s/%s" % [shape, tint.to_html()]
	if _textures.has(key):
		return _textures[key]
	var image: Image = Image.create(32, 32, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)
	var polygon: PackedVector2Array
	# These are eight rendering selectors, never item identities.
	match shape:
		"rod": polygon = _points([[5, 25], [8, 28], [28, 7], [25, 4]])
		"blade": polygon = _points([[6, 28], [3, 25], [12, 17], [14, 6], [28, 3], [25, 17], [16, 20]])
		"vessel": polygon = _points([[7, 7], [25, 7], [23, 12], [26, 23], [22, 28], [10, 28], [6, 23], [9, 12]])
		"cloth": polygon = _points([[5, 6], [25, 4], [28, 25], [7, 28], [9, 19]])
		"grain": polygon = _points([[10, 25], [7, 18], [10, 8], [17, 4], [24, 10], [25, 20], [20, 27]])
		"ore": polygon = _points([[4, 21], [8, 8], [20, 3], [28, 13], [25, 27], [12, 29]])
		"ingot": polygon = _points([[3, 22], [8, 8], [23, 8], [29, 21], [25, 27], [7, 27]])
		_: polygon = _points([[5, 21], [6, 12], [13, 6], [23, 7], [28, 17], [23, 26], [12, 28]])
	var ink: Color = THEME.get_color("font_color", "Label")
	_fill(image, polygon, tint)
	_outline(image, polygon, ink)
	if shape == "vessel": _line(image, Vector2(9, 10), Vector2(23, 10), ink)
	if shape == "cloth": _line(image, Vector2(11, 9), Vector2(13, 23), ink)
	if shape == "grain": _line(image, Vector2(16, 9), Vector2(16, 24), ink)
	if shape == "ore": _line(image, Vector2(9, 12), Vector2(18, 20), ink)
	var texture: ImageTexture = ImageTexture.create_from_image(image)
	_textures[key] = texture
	return texture


## Complete and active sites use actual rotated footprint pixels. Inert is preview-only.
static func building_texture(row: Dictionary, pixels: Vector2i, preview: bool = false) -> Texture2D:
	if not bool(row.get("connected", false)) and not preview:
		return null
	if pixels.x <= 0 or pixels.y <= 0 or pixels.x > 4096 or pixels.y > 4096:
		return null
	var roof: String = str(row.get("art_roof", "round")).to_lower()
	var glyph: String = str(row.get("art_icon", ""))
	var tint: Color = material_color(str(row.get("art_tint_material", "")))
	_load_appearance()
	var era_hex: String = str(_appearance.get("era_outlines", {}).get(str(row.get("era_id", "")), ""))
	var ink: Color = Color.from_string(era_hex, THEME.get_color("font_color", "Label"))
	var inert: bool = not bool(row.get("connected", false))
	var key: String = "roof/%s/%s/%s/%s/%s/%s" % [roof, glyph, tint.to_html(), ink.to_html(), pixels, inert]
	if _textures.has(key): return _textures[key]
	var image: Image = Image.create(pixels.x, pixels.y, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)
	var polygon: PackedVector2Array = PackedVector2Array()
	if roof == "angular":
		polygon = _points([[2, pixels.y - 3], [2, pixels.y / 3.0], [pixels.x / 2.0, 2], [pixels.x - 3, pixels.y / 3.0], [pixels.x - 3, pixels.y - 3]])
	else:
		var centre: Vector2 = Vector2(pixels) * 0.5
		var radius: Vector2 = centre - Vector2(3, 3)
		for i: int in 32:
			var angle: float = TAU * i / 32.0
			polygon.append(centre + Vector2(cos(angle), sin(angle)) * radius)
	_fill(image, polygon, tint)
	_outline(image, polygon, ink)
	_draw_glyph(image, glyph, ink)
	if inert:
		# Only pixels already belonging to this silhouette are hatched/halved.
		for y: int in pixels.y:
			for x: int in pixels.x:
				var color: Color = image.get_pixel(x, y)
				if color.a == 0.0: continue
				if (x + y) % 8 < 2: color = ink
				color.a *= 0.5
				image.set_pixel(x, y, color)
	var texture: ImageTexture = ImageTexture.create_from_image(image)
	_textures[key] = texture
	return texture


static func _draw_glyph(image: Image, glyph: String, ink: Color) -> void:
	_load_appearance()
	var scale: float = minf(image.get_width(), image.get_height()) / 16.0
	var origin: Vector2 = Vector2(image.get_size()) * 0.5 - Vector2(4, 4) * scale
	for path: Array in _glyphs.get(glyph, []):
		for i: int in range(1, path.size()):
			var a: Array = path[i - 1]
			var b: Array = path[i]
			_line(image, origin + Vector2(a[0], a[1]) * scale, origin + Vector2(b[0], b[1]) * scale, ink)


static func _points(values: Array) -> PackedVector2Array:
	var out: PackedVector2Array = PackedVector2Array()
	for value: Array in values: out.append(Vector2(float(value[0]), float(value[1])))
	return out


static func _fill(image: Image, polygon: PackedVector2Array, color: Color) -> void:
	for y: int in image.get_height():
		for x: int in image.get_width():
			if Geometry2D.is_point_in_polygon(Vector2(x + 0.5, y + 0.5), polygon):
				image.set_pixel(x, y, color)


static func _outline(image: Image, polygon: PackedVector2Array, color: Color) -> void:
	for i: int in polygon.size(): _line(image, polygon[i], polygon[(i + 1) % polygon.size()], color)


static func _line(image: Image, a: Vector2, b: Vector2, color: Color) -> void:
	var steps: int = maxi(1, ceili(maxf(absf(b.x - a.x), absf(b.y - a.y))))
	for i: int in steps + 1:
		var point: Vector2i = Vector2i(a.lerp(b, float(i) / steps).round())
		if point.x >= 0 and point.y >= 0 and point.x < image.get_width() and point.y < image.get_height():
			image.set_pixelv(point, color)
