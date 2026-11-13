# Revealed finite surface deposits, rendered from bound art or procedural placeholder.
# Bridge methods: get_outcrops via SimReads; world_reset clears presentation nodes.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const Content = preload("res://scripts/ui/content_context.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const Icons = preload("res://scripts/ui/item_icons.gd")
const Overlay = preload("res://scripts/world/colony/overlay_layer.gd")
const Text = preload("res://scripts/ui/story_text.gd")
var _world: Node
var _bodies: Dictionary = {}
var _rows: Dictionary = {}
var _version: int = -1
var _art_version: int = -1
var _hover := Vector2i(-1, -1)
var _font: Font
var _zoom: float = -1.0


func _ready() -> void:
	_world = get_node("/root/Main/WorldSim")
	_world.connect("world_reset", _reset)
	_font = load("res://assets/fonts/galmuri/Galmuri11.ttf") as Font
	warmup()


static func warmup() -> void:
	for definition: Dictionary in Content.rows("outcrops"):
		Icons.item_texture(definition)


func _reset() -> void:
	for body: Sprite2D in _bodies.values(): body.queue_free()
	_bodies.clear()
	_rows = {}
	_version = -1
	_hover = Vector2i(-1, -1)
	queue_redraw()


func _process(_delta: float) -> void:
	var read: Dictionary = SimReads.read(_world, "get_outcrops")
	var version: int = int(read.get("version", -1))
	if version != _version or _art_version != ArtData.params_version():
		_art_version = ArtData.params_version()
		_version = version
		_rows = read
		_refresh()
	var visible_rect: Rect2 = get_viewport().get_visible_rect().grow(float(View.TILE))
	var zoom: float = get_viewport().get_canvas_transform().x.x
	var redraw: bool = zoom != _zoom
	_zoom = zoom
	for body: Sprite2D in _bodies.values():
		var shown: bool = visible_rect.has_point(body.get_global_transform_with_canvas().origin)
		redraw = redraw or shown != body.visible
		body.visible = shown
	var hover: Vector2i = outcrop_at(get_global_mouse_position())
	if hover != _hover or redraw:
		_hover = hover
		queue_redraw()


func _refresh() -> void:
	var xs: PackedInt64Array = _rows.get("xs", PackedInt64Array())
	var ys: PackedInt64Array = _rows.get("ys", PackedInt64Array())
	var kinds: PackedInt64Array = _rows.get("kinds", PackedInt64Array())
	var remaining: PackedInt64Array = _rows.get("remaining", PackedInt64Array())
	var wanted: Dictionary = {}
	for i: int in mini(xs.size(), mini(ys.size(), kinds.size())):
		var tile := Vector2i(xs[i], ys[i])
		wanted[tile] = true
		var body: Sprite2D = _bodies.get(tile)
		if body == null:
			body = Sprite2D.new()
			body.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
			get_node("../World").add_child(body)
			_bodies[tile] = body
		var definition: Dictionary = Content.row("outcrops", kinds[i])
		var tex: Texture2D = _bound_texture(kinds[i], remaining[i] if i < remaining.size() else -1, definition)
		body.texture = tex
		body.scale = Vector2.ONE
		body.position = View.base_point(tile) - Vector2(0, floorf(body.texture.get_height() * 0.5))
		body.set_meta("definition", definition)
		body.set_meta("bound", tex != Icons.item_texture(definition))
	for tile: Vector2i in _bodies.keys():
		if not wanted.has(tile):
			(_bodies[tile] as Sprite2D).queue_free()
			_bodies.erase(tile)
	queue_redraw()


## Authored art from bindings when available, with depleted state; procedural placeholder otherwise.
static func _bound_texture(kind: int, units: int, definition: Dictionary) -> Texture2D:
	var row: Dictionary = ArtData.binding("outcrops", kind)
	var id: String = str(row.get("structure", ""))
	if id.is_empty() or not ArtData.has(id):
		return Icons.item_texture(definition)
	var full: int = maxi(1, int(definition.get("units", 1)))
	if row.get("depleted", null) is String and units >= 0 and floori(float(units) * 1000.0 / full) < int(row.get("depleted_below_permille", 0)):
		var dep: String = str(row["depleted"])
		if ArtData.has(dep):
			return ArtData.texture(dep)
	return ArtData.texture(id)


func _draw() -> void:
	if _font == null: return
	var camera: Camera2D = get_viewport().get_camera_2d()
	var zoom: float = camera.zoom.x if camera != null else 1.0
	for tile: Vector2i in _bodies:
		var body: Sprite2D = _bodies[tile]
		if not body.visible: continue
		var definition: Dictionary = body.get_meta("definition", {})
		var bound: bool = bool(body.get_meta("bound", false))
		var label: String = "" if bound else str(definition.get("glyph", ""))
		if tile == _hover:
			label = Text.ltr(get_node_or_null("/root/Locale"), str(definition.get("name_key", "")))
		if not label.is_empty():
			Overlay.draw_label(self, _font, label, View.base_point(tile) - Vector2(0, float(View.TILE) * 0.6), zoom)


func outcrop_at(point: Vector2) -> Vector2i:
	for tile: Vector2i in _bodies:
		var body: Sprite2D = _bodies[tile]
		if body.visible and body.get_rect().has_point(body.to_local(point)): return tile
	return Vector2i(-1, -1)


func counts() -> Dictionary:
	return {"outcrops": _bodies.size()}
