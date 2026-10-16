extends RefCounted
## Immutable pixel silhouettes shared by agent icons and village summaries.
const CRAFT = preload("res://assets/art/production/U1/04.png")
const OUTLINE := Color("211C25")
const HIGHLIGHT := Color("FFF2D4")
const COLORS: Dictionary = {0: Color("F39A43"), 2: Color("91BED9"), 3: Color("D6A3E0"), 4: Color("98D47B"), 5: Color("F3D86B"), 6: Color("79D5CD")}
const MASKS: Dictionary = {
	0: ["............XX..", "...........XX...", "..........XX....", ".........XX.....", "......X.XX......", ".....XX.X.......", ".....XXXX.......", "...XXXXXXX......", "...XXoXXXX......", "..XXXooXXXX.....", "..XXoooXXXX.....", "...XoooXXX......", "....XXXXX.......", "..XXXXXXXXXX....", "................", "................"],
	2: ["................", "....XXXXXXXX....", "...XXXXXXXXXX...", "...XXXXXXXXXX...", "....XXXXXXXX....", ".......XX.......", ".......XX.......", ".......XX.......", ".......XX.......", ".......XX.......", ".......XX.......", ".......XX.......", ".......XX.......", "......XXXX......", "................", "................"],
	3: ["................", "..XXX......XXX..", ".XXXXX....XXXXX.", ".XXXXX....XXXXX.", "..XXX......XXX..", "...XX......XX...", "..XXX......XXX..", ".XXXX..X...XXXX.", ".XXXX.XXX..XXXX.", "..XX..XoX...XX..", "......XoX.......", ".....XXXXX......", "....XXXXXXX.....", "................", "................", "................"],
	4: ["................", ".......XXXXXXX..", ".........XXXXX..", ".........XXXXX..", "........XXXXXX..", ".......XXX..XX..", "......XXX...XX..", ".....XXX....XX..", "....XXX.........", "...XXX..........", "..XXX...........", ".XXX............", "..X.............", "................", "................", "................"],
	5: ["................", ".....XXXXXX.....", "...XXXXXXXXXX...", "..XXXXooXXXXXX..", "..XXXooooXXXXX..", ".XXXXooooXXXXXX.", ".XXXXXooXXXXXXX.", ".XXXXXXXXXXXXXX.", ".XXXXXXXXXXXXXX.", ".XXXXXXXooXXXXX.", "..XXXXXooooXXX..", "..XXXXXooooXXX..", "...XXXXXooXXX...", ".....XXXXXX.....", "................", "................"],
	6: ["................", "....o..o........", "...o..o.........", "....o..o........", "................", ".XXXXXXXX.......", ".XXXXXXXXXXX....", ".XXXXXXXX..XX...", ".XXXXXXXX..XX...", ".XXXXXXXXXXX....", "..XXXXXX........", "..XXXXXX....X...", ".XXXXXXXX..XXX..", "..........XXoXX.", "...........XXX..", "................"],
}
static var _textures: Dictionary = {}

static func icon_id(kind: int) -> String:
	if kind < 0 or kind > 6:
		return ""
	return "U1_04" if kind == 1 else "p11_activity_%d" % kind

static func texture(kind: int) -> Texture2D:
	if kind == 1:
		return CRAFT
	if not MASKS.has(kind):
		return null
	if not _textures.has(kind):
		var image: Image = Image.create(16, 16, false, Image.FORMAT_RGBA8)
		image.fill(Color.TRANSPARENT)
		var rows: Array = MASKS[kind]
		for y: int in 16:
			for x: int in 16:
				if str(rows[y])[x] == ".":
					continue
				for offset: Vector2i in [Vector2i.LEFT, Vector2i.RIGHT, Vector2i.UP, Vector2i.DOWN]:
					var edge: Vector2i = Vector2i(x, y) + offset
					if edge.x >= 0 and edge.x < 16 and edge.y >= 0 and edge.y < 16:
						image.set_pixelv(edge, OUTLINE)
		for y: int in 16:
			for x: int in 16:
				var pixel: String = str(rows[y])[x]
				if pixel != ".":
					image.set_pixel(x, y, HIGHLIGHT if pixel == "o" else COLORS[kind])
		_textures[kind] = ImageTexture.create_from_image(image)
	return _textures[kind]
