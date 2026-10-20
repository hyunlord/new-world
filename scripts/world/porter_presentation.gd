extends Node2D
## Presentation only: packed entity identities and authoritative carried loads.
const FOOD := Color("e9a34f")
const WOOD := Color("98663f")
static var _textures: Dictionary = {}
var routes: Array[Dictionary] = []
var zoom: float = 1.0

static func rows(snapshot: Dictionary) -> Dictionary:
	var result: Dictionary = {}
	var ids: PackedInt64Array = snapshot.get("ids", PackedInt64Array())
	for i: int in ids.size():
		result[int(ids[i])] = {"from": int(snapshot["from"][i]), "to": int(snapshot["to"][i]), "kind": int(snapshot["kind"][i]), "amount": int(snapshot["amount"][i]), "stage": int(snapshot["stage"][i])}
	return result

static func activity(_row: Dictionary, food: float, wood: int) -> String:
	return "carry_wood" if wood > 0 else ("carry" if food > 0 else "walk")

static func destination(row: Dictionary, settlements: Dictionary) -> Vector2i:
	var target: int = int(row["to"] if int(row["stage"]) == 0 else row["from"])
	var ids: PackedInt32Array = settlements.get("settlement_ids", PackedInt32Array())
	var index: int = ids.find(target)
	if index < 0:
		return Vector2i(-1, -1)
	return Vector2i(settlements["formation_xs"][index], settlements["formation_ys"][index])

static func texture(kind: int) -> ImageTexture:
	if _textures.has(kind):
		return _textures[kind]
	var image := Image.create(16, 16, false, Image.FORMAT_RGBA8)
	var hue: Color = WOOD if kind == 1 else FOOD
	# Pack, raised handle and diagonal shoulder strap remain legible at native size.
	for y: int in range(3, 15):
		for x: int in range(3, 13):
			image.set_pixel(x, y, Color("33281f") if x in [3,12] or y in [3,14] else hue)
	for x: int in range(6,10):
		image.set_pixel(x,1,Color("33281f"))
	image.set_pixel(5,2,Color("33281f"))
	image.set_pixel(10,2,Color("33281f"))
	for y: int in range(4,14):
		var x: int = 4 + ((y - 4) >> 1)
		image.set_pixel(x,y,Color("f5d7a1"))
		image.set_pixel(x+1,y,Color("543e29"))
	for x: int in range(4,12):
		image.set_pixel(x,5,hue.lightened(0.25))
	_textures[kind] = ImageTexture.create_from_image(image)
	return _textures[kind]

static func apply_icon(body: Sprite2D, icon: Sprite2D, row: Dictionary, camera_zoom: float) -> void:
	icon.texture = texture(int(row["kind"]))
	icon.region_enabled = false
	icon.centered = false
	icon.offset = Vector2(-8,-16)
	icon.scale = Vector2.ONE * (2.0 if camera_zoom < 2.0 else 1.0)
	icon.position = Vector2(0,body.offset.y-2)
	icon.modulate = Color(1,1,1,0.6 if int(row["stage"]) == 1 else 1.0)
	icon.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	icon.visible = body.visible
	icon.set_meta("sprite_id", "p12_porter_wood" if int(row["kind"]) == 1 else "p12_porter_food")
	icon.set_meta("frame_id", icon.get_meta("sprite_id"))
	icon.set_meta("frame_size", Vector2i(16,16))
	icon.set_meta("anchor", Vector2(8,16))
	icon.set_meta("drawable", true)
	icon.set_meta("art_kind", "activity_icon")

func _draw() -> void:
	if zoom >= 2.0:
		return
	for route: Dictionary in routes:
		var body: Sprite2D = route["body"]
		if not is_instance_valid(body) or not body.visible:
			continue
		var start: Vector2 = to_local(body.global_position)
		var end: Vector2 = to_local(route["end"])
		var color: Color = WOOD if int(route["kind"]) == 1 else FOOD
		color.a = 0.6
		var length: float = start.distance_to(end)
		var direction: Vector2 = start.direction_to(end)
		var step: float = 4.0 / maxf(zoom,0.01)
		for index: int in ceili(length / step):
			var distance: float = float(index) * step
			draw_line(start + direction * distance, start + direction * minf(length,distance + 1.0 / maxf(zoom,0.01)),color,1.0 / maxf(zoom,0.01),false)
