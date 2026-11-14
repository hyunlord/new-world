# overlay3d.gd (World3D/Overlay, made by world3d.gd): the 2D view's map overlays and the order feedback, projected
#   onto the 3D terrain as one texel per tile (M3D-1 scope 8).
# Purpose: an RGBA image of the map (terrain.gdshader `overlay_tex`, nearest) plus an outline image (`frame_tex`):
#   stockpile zones (render_params `zones.tint_rgb`), housing zones, hunt zones (`zones.hunt_rgb`), designations
#   (chop/quarry/dig/deconstruct/clay marks, framed), field tiles; with the room overlay shown (R, room_layer.gd) every
#   room tile in its role colour (RoomLayer.ROLE_COLORS); and the order tool's live state read from OrderTool — the
#   hovered tile framed in an area or build mode, the drag rectangle, and the placement preview (`preview()`: fitting
#   tiles green, refused tiles in `sites.refused_rgba`). O cycles the bridge's influence channels
#   (get_influence_overlay: warmth, light, noise, food smell, danger, social, spiritual, beauty) as a heat tint, read
#   once per sim tick while shown; a small label names the channel (Locale keys W3D_OVERLAY_*).
# Bridge methods called: get_influence_overlay (direct, once per tick while a channel is shown); via SimReads
#   get_colony_overlay, get_animals, get_rooms, get_field_tiles, get_time_snapshot.
extends Node3D

const ArtData = preload("res://scripts/world/art/art_data.gd")
const RoomLayer = preload("res://scripts/world/colony/room_layer.gd")
const Data3D = preload("res://scripts/world3d/data3d.gd")
const CHANNEL_KEYS: Array[String] = ["W3D_OVERLAY_WARMTH", "W3D_OVERLAY_LIGHT", "W3D_OVERLAY_NOISE", "W3D_OVERLAY_FOOD",
	"W3D_OVERLAY_DANGER", "W3D_OVERLAY_SOCIAL", "W3D_OVERLAY_SPIRITUAL", "W3D_OVERLAY_BEAUTY"]
const TOGGLE_KEY: Key = KEY_O

var channel: int = -1
var _world_sim: Node = null
var _terrain: Node = null
var _base: Image = null
var _frame_base: Image = null
var _img: Image = null
var _frame: Image = null
var _tex: ImageTexture = null
var _frame_tex: ImageTexture = null
var _base_key: String = ""
var _live_key: String = ""
var _influence: PackedByteArray = PackedByteArray()
var _influence_tick: int = -1
var _label: Label = null


func setup(world_sim: Node, terrain: Node) -> void:
	_world_sim = world_sim
	_terrain = terrain


func _ready() -> void:
	var layer: CanvasLayer = CanvasLayer.new()
	layer.layer = 2
	add_child(layer)
	_label = Label.new()
	_label.position = Vector2(24, 900)
	_label.visible = false
	_label.add_theme_color_override("font_color", Color(0.96, 0.92, 0.84))
	_label.add_theme_color_override("font_outline_color", Color(0.09, 0.08, 0.07))
	_label.add_theme_constant_override("outline_size", 6)
	layer.add_child(_label)


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and (event as InputEventKey).keycode == TOGGLE_KEY \
			and not (event as InputEventKey).ctrl_pressed and not (event as InputEventKey).meta_pressed:
		cycle_channel()
		get_viewport().set_input_as_handled()


## Show the next influence channel (after the last one: none).
func cycle_channel() -> void:
	channel = channel + 1 if channel + 1 < CHANNEL_KEYS.size() else -1
	_influence_tick = -1
	_base_key = ""
	var locale: Node = get_node_or_null("/root/Locale")
	_label.visible = channel >= 0
	if channel >= 0:
		_label.text = str(locale.call("ltr", CHANNEL_KEYS[channel])) if locale != null else CHANNEL_KEYS[channel]


func _process(_delta: float) -> void:
	if _world_sim == null or _terrain == null:
		return
	var w: int = int(_terrain.get("width"))
	var h: int = int(_terrain.get("height"))
	if w <= 0 or h <= 0:
		return
	if _img == null or _img.get_width() != w or _img.get_height() != h:
		_base = Image.create_empty(w, h, false, Image.FORMAT_RGBA8)
		_frame_base = Image.create_empty(w, h, false, Image.FORMAT_R8)
		_img = _base.duplicate()
		_frame = _frame_base.duplicate()
		_tex = ImageTexture.create_from_image(_img)
		_frame_tex = ImageTexture.create_from_image(_frame)
		var mat: ShaderMaterial = _terrain.get("material")
		mat.set_shader_parameter("overlay_tex", _tex)
		mat.set_shader_parameter("frame_tex", _frame_tex)
		mat.set_shader_parameter("overlay_on", 1.0)
		# Plants standing on an overlaid tile (a chop mark, the drag rectangle) tint too (flora.gdshader).
		RenderingServer.global_shader_parameter_set("w3d_overlay", _tex)
		RenderingServer.global_shader_parameter_set("w3d_frame", _frame_tex)
		RenderingServer.global_shader_parameter_set("w3d_map_size", Vector2(w, h))
		_base_key = ""
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", 0))
	if channel >= 0 and tick != _influence_tick:
		_influence_tick = tick
		_influence = _world_sim.call("get_influence_overlay", channel)
		_base_key = ""
	var overlay: Dictionary = SimReads.read(_world_sim, "get_colony_overlay")
	var rooms: Dictionary = SimReads.read(_world_sim, "get_rooms")
	var animals: Dictionary = SimReads.read(_world_sim, "get_animals")
	var fields: Dictionary = SimReads.read(_world_sim, "get_field_tiles")
	var room_layer: Node = get_node_or_null("/root/Main/ColonyView/OverlayLayer/Rooms")
	var rooms_shown: bool = room_layer != null and bool(room_layer.call("is_shown"))
	var key: String = "%d|%d|%d|%d|%s|%d|%d" % [int(overlay.get("version", 0)), int(rooms.get("version", 0)), int(animals.get("version", 0)),
		hash(fields.get("phases", PackedInt64Array())), rooms_shown, channel, _influence_tick]
	if key != _base_key:
		_base_key = key
		_build_base(overlay, rooms, animals, fields, rooms_shown)
		_live_key = ""
	_apply_live()


func _put(img: Image, frame: Image, t: Vector2i, c: Color, framed: bool) -> void:
	if t.x < 0 or t.y < 0 or t.x >= img.get_width() or t.y >= img.get_height():
		return
	var prev: Color = img.get_pixelv(t)
	img.set_pixelv(t, prev.blend(c) if prev.a > 0.0 else c)
	if framed:
		frame.set_pixelv(t, Color(1, 0, 0))


func _rgb(path: String, fallback: Color, alpha: float) -> Color:
	var v: Variant = ArtData.param(path, null)
	var c: Color = Color8(int(v[0]), int(v[1]), int(v[2])) if v is Array and (v as Array).size() >= 3 else fallback
	c.a = alpha
	return c


func _build_base(overlay: Dictionary, rooms: Dictionary, animals: Dictionary, fields: Dictionary, rooms_shown: bool) -> void:
	_base.fill(Color(0, 0, 0, 0))
	_frame_base.fill(Color(0, 0, 0, 0))
	var w: int = _base.get_width()
	if channel >= 0 and _influence.size() == w * _base.get_height():
		for i: int in _influence.size():
			var v: float = float(_influence[i]) / 255.0
			if v > 0.02:
				_base.set_pixel(i % w, int(float(i) / float(w)), Color(1.0, 1.0 - v * 0.85, 0.25 * (1.0 - v), clampf(v * 0.8, 0.1, 0.7)))
	var alpha: float = clampf(float(ArtData.param("zones.tint_alpha", 0.08)) * 3.5, 0.15, 0.5)
	var zone: Color = _rgb("zones.tint_rgb", Color(0.89, 0.8, 0.59), alpha)
	var zx: PackedInt32Array = overlay.get("zone_xs", PackedInt32Array())
	var zy: PackedInt32Array = overlay.get("zone_ys", PackedInt32Array())
	for i: int in mini(zx.size(), zy.size()):
		_put(_base, _frame_base, Vector2i(zx[i], zy[i]), zone, false)
	var housing: Color = Color(0.55, 0.75, 0.95, alpha)
	var hx: PackedInt32Array = overlay.get("housing_zone_xs", PackedInt32Array())
	var hy: PackedInt32Array = overlay.get("housing_zone_ys", PackedInt32Array())
	for i: int in mini(hx.size(), hy.size()):
		_put(_base, _frame_base, Vector2i(hx[i], hy[i]), housing, false)
	var hunt: Color = _rgb("zones.hunt_rgb", Color(0.95, 0.72, 0.3), alpha)
	var hz: PackedInt32Array = animals.get("zones", PackedInt32Array())
	for q: int in range(0, hz.size() - 3, 4):
		for y: int in range(mini(hz[q + 1], hz[q + 3]), maxi(hz[q + 1], hz[q + 3]) + 1):
			for x: int in range(mini(hz[q], hz[q + 2]), maxi(hz[q], hz[q + 2]) + 1):
				_put(_base, _frame_base, Vector2i(x, y), hunt, false)
	var field_c: Color = Color(0.85, 0.75, 0.35, 0.18)
	var fx: PackedInt64Array = fields.get("xs", PackedInt64Array())
	var fy: PackedInt64Array = fields.get("ys", PackedInt64Array())
	var factive: PackedByteArray = fields.get("active", PackedByteArray())
	for i: int in mini(fx.size(), fy.size()):
		if i >= factive.size() or factive[i] == 1:
			_put(_base, _frame_base, Vector2i(int(fx[i]), int(fy[i])), field_c, false)
	var mark_colours: Array = Data3D.binding("overlays.mark_colours", [[0.95, 0.85, 0.3, 0.45]])
	var dx: PackedInt32Array = overlay.get("designation_xs", PackedInt32Array())
	var dy: PackedInt32Array = overlay.get("designation_ys", PackedInt32Array())
	var dk: PackedInt32Array = overlay.get("designation_kinds", PackedInt32Array())
	for i: int in mini(dx.size(), dy.size()):
		var k: int = clampi(dk[i] if i < dk.size() else 0, 0, mark_colours.size() - 1)
		_put(_base, _frame_base, Vector2i(dx[i], dy[i]), Data3D.colour(mark_colours[k], Color(0.95, 0.85, 0.3, 0.45)), true)
	if rooms_shown:
		var tx: PackedInt32Array = rooms.get("tile_xs", PackedInt32Array())
		var ty: PackedInt32Array = rooms.get("tile_ys", PackedInt32Array())
		var tr: PackedInt32Array = rooms.get("tile_rooms", PackedInt32Array())
		var ids: PackedInt32Array = rooms.get("ids", PackedInt32Array())
		var roles: PackedInt32Array = rooms.get("roles", PackedInt32Array())
		var role_of: Dictionary = {}
		for i: int in mini(ids.size(), roles.size()):
			role_of[ids[i]] = roles[i]
		for i: int in mini(tr.size(), mini(tx.size(), ty.size())):
			var c: Color = RoomLayer.ROLE_COLORS[clampi(int(role_of.get(tr[i], 0)), 0, RoomLayer.ROLE_COLORS.size() - 1)]
			c.a = minf(0.6, c.a * 1.8)
			_put(_base, _frame_base, Vector2i(tx[i], ty[i]), c, false)


## Hover, drag and placement preview from OrderTool, on a copy of the base.
func _apply_live() -> void:
	var tool: Node = get_node_or_null("/root/Main/ColonyView/OrderTool")
	var mode: int = int(tool.get("mode")) if tool != null else 0
	var hover: Vector2i = tool.get("_hover") if tool != null else Vector2i(-1, -1)
	var dragging: bool = bool(tool.get("_dragging")) if tool != null else false
	var a: Vector2i = tool.get("_drag_from") if tool != null else Vector2i.ZERO
	var b: Vector2i = tool.get("_drag_to") if tool != null else Vector2i.ZERO
	var preview: Dictionary = tool.call("preview") if tool != null and mode != 0 else {}
	var key: String = "%d|%s|%s|%s|%s|%d" % [mode, hover, dragging, a, b, hash(preview)]
	if key == _live_key:
		return
	_live_key = key
	_img.copy_from(_base)
	_frame.copy_from(_frame_base)
	if mode != 0:
		var white: Color = Color(1, 1, 1, 0.22)
		if dragging:
			for y: int in range(mini(a.y, b.y), maxi(a.y, b.y) + 1):
				for x: int in range(mini(a.x, b.x), maxi(a.x, b.x) + 1):
					_put(_img, _frame, Vector2i(x, y), white, x == mini(a.x, b.x) or x == maxi(a.x, b.x) or y == mini(a.y, b.y) or y == maxi(a.y, b.y))
		var xs: Variant = preview.get("xs")
		var ys: Variant = preview.get("ys")
		var ok: Variant = preview.get("ok")
		if xs is PackedInt32Array and ys is PackedInt32Array:
			var refused: Variant = ArtData.param("sites.refused_rgba", [235, 80, 64, 90])
			var bad: Color = Color8(int(refused[0]), int(refused[1]), int(refused[2]), 150) if refused is Array else Color(0.9, 0.3, 0.25, 0.6)
			for i: int in mini((xs as PackedInt32Array).size(), (ys as PackedInt32Array).size()):
				var fits: bool = ok is PackedByteArray and i < (ok as PackedByteArray).size() and (ok as PackedByteArray)[i] == 1
				_put(_img, _frame, Vector2i(xs[i], ys[i]), Color(0.45, 0.9, 0.45, 0.45) if fits else bad, true)
		if (xs is not PackedInt32Array or (xs as PackedInt32Array).is_empty()) and tool.has_method("build_size") and tool.has_method("build_origin") \
				and bool(tool.call("template_active")) == false and mode == preload("res://scripts/world/colony/order_tool.gd").Mode.BUILD:
			var size: Vector2i = tool.call("build_size")
			var origin: Vector2i = tool.call("build_origin", hover)
			for y: int in size.y:
				for x: int in size.x:
					_put(_img, _frame, origin + Vector2i(x, y), Color(0.85, 0.9, 0.8, 0.3), true)
		_put(_img, _frame, hover, Color(1, 1, 1, 0.12), true)
	_tex.update(_img)
	_frame_tex.update(_frame)
