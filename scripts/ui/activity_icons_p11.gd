extends RefCounted
## Two-tone pictograms on shared pixel-grid badges; original Craft art stays immutable.
const CRAFT = preload("res://assets/art/production/U1/04.png")
const OUTLINE := Color("211C25")
const BADGE := Color(33.0 / 255.0, 28.0 / 255.0, 37.0 / 255.0, 0.85)
const MARGIN: int = 2
const HIGHLIGHT := Color("FFF2D4")
const COLORS: Dictionary = {0: Color("F5A076"), 1: Color("E0F576"), 2: Color("76A0F5"), 3: Color("B576F5"), 4: Color("76F576"), 5: Color("F576CA"), 6: Color("76F5E0")}
const MASKS: Dictionary = {
	0: ["............XX..", "...........XooX.", "..........XooX..", ".........XooX...", "........XooX....", "....X..XooX.....", "....XX.XXX......", "...XXXX.........", "..XXoXXX........", "..XXooXXX.......", ".XXXoooXXX......", ".XXooooXXX......", "..XXooXXX.......", "...XXXXX........", "..XXXXXXX.......", "................"],
	2: ["..XXXXXX........", ".XXooooXX.......", ".XXXXXXXX.......", "..XXXXXX........", ".....XX.........", ".....XX.........", "......XX........", "......XX........", ".......XX.......", ".......XX..o....", "........XX..o...", "........XX......", ".XXXXXXXXXXXXX..", ".XoooooooooooX..", ".XXXXXXXXXXXXX..", "................"],
	3: ["......XXX.......", ".....XoooX......", ".....XoooX......", "......XXX.......", "................", ".XXX.......XXX..", "XoooX.....XoooX.", "XoooX.....XoooX.", ".XXX...X...XXX..", "......XXX.......", ".....XXoXX......", ".....XoooX......", "......XXX.......", "....XXXXXXX.....", "................", "................"],
	4: ["..........XX....", ".........XooX...", ".........XooX...", ".........XooX...", "..........XX....", "..........XX....", ".......o........", "......o.........", "...XX...........", "..XooX..........", "..XooX..........", "..XooX..........", "...XX...........", "...XX..o........", "......o.........", "................"],
	5: ["...XXXXX........", "..XX............", ".XX...oooo......", ".X..oo..........", "XX.oo...........", "X..o.....XXXX...", "X.......XXooXX..", ".......XXoooXXX.", ".......XoooXXXX.", ".......XXXXXXXX.", ".......XXXXooXX.", ".......XXXoooXX.", "........XXooXX..", ".........XXXX...", "................", "................"],
	6: ["..XXX...........", ".XoooX..........", ".XoooX..........", "..XXX...........", "...XX...........", "...XX...........", "...XXXXXX.......", "...XX...XX...X..", "...XX.......XX..", "...XXXX....XXX..", "......XX...XoXX.", "......XX..XoooX.", "..XXXXXX...XXX..", "..X....XX.XXXXX.", "................", "................"],
}
static var _textures: Dictionary = {}

static func icon_id(kind: int) -> String:
	if kind < 0 or kind > 6:
		return ""
	return "U1_04" if kind == 1 else "p11_activity_%d" % kind

static func pictogram_image(kind: int) -> Image:
	if not COLORS.has(kind):
		return null
	if kind == 1:
		var original: Image = CRAFT.get_image()
		var recolored: Image = Image.create(original.get_width(), original.get_height(), false, Image.FORMAT_RGBA8)
		recolored.fill(Color.TRANSPARENT)
		for y: int in original.get_height():
			for x: int in original.get_width():
				var pixel: Color = original.get_pixel(x, y)
				if pixel.a > 0.0:
					recolored.set_pixel(x, y, HIGHLIGHT if pixel.get_luminance() > 0.5 else COLORS[kind])
		return recolored
	var image: Image = Image.create(16, 16, false, Image.FORMAT_RGBA8)
	image.fill(Color.TRANSPARENT)
	var rows: Array = MASKS[kind]
	for y: int in 16:
		for x: int in 16:
			var pixel: String = str(rows[y])[x]
			if pixel != ".":
				image.set_pixel(x, y, HIGHLIGHT if pixel == "o" else COLORS[kind])
	return image

static func texture(kind: int) -> Texture2D:
	if not COLORS.has(kind):
		return null
	if not _textures.has(kind):
		var pictogram: Image = pictogram_image(kind)
		var width: int = pictogram.get_width() + MARGIN * 2
		var height: int = pictogram.get_height() + MARGIN * 2
		var image: Image = Image.create(width, height, false, Image.FORMAT_RGBA8)
		image.fill(Color.TRANSPARENT)
		for y: int in height:
			for x: int in width:
				var edge_x: int = mini(x, width - 1 - x)
				var edge_y: int = mini(y, height - 1 - y)
				if edge_x + edge_y >= 2:
					image.set_pixel(x, y, BADGE)
		image.blend_rect(pictogram, Rect2i(Vector2i.ZERO, pictogram.get_size()), Vector2i(MARGIN, MARGIN))
		_textures[kind] = ImageTexture.create_from_image(image)
	return _textures[kind]
