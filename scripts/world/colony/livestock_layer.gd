# Real domestic animals in their physical pens, with data-selected placeholder art.
# Bridge methods: get_livestock through SimReads; world_reset invalidates presentation nodes.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const Content = preload("res://scripts/ui/content_context.gd")
const Icons = preload("res://scripts/ui/item_icons.gd")
const Overlay = preload("res://scripts/world/colony/overlay_layer.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const WildAnimals = preload("res://scripts/world/colony/animal_layer.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
var _world: Node
var _bodies: Dictionary = {}
var _rows: Dictionary = {}
var _font: Font
var _tick: int = -1
var _hover: int = -1
var _zoom: float = 1.0
var _lod: bool = false


func _ready() -> void:
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world = get_node("/root/Main/WorldSim")
	_world.connect("world_reset", _reset)
	_font = load("res://assets/fonts/galmuri/Galmuri11.ttf") as Font
	# Reuse the existing shape raster/cache at load, before a species first enters the view.
	for definition: Dictionary in Content.rows("animals"):
		if bool(definition.get("tameable", false)): Icons.item_texture(definition)


func _reset() -> void:
	for body: Sprite2D in _bodies.values():
		body.queue_free()
	_bodies.clear()
	_rows = {}
	_tick = -1
	_hover = -1
	queue_redraw()


func _process(_delta: float) -> void:
	var read: Dictionary = SimReads.read(_world, "get_livestock")
	var tick: int = int(read.get("tick", -1))
	if tick != _tick or _rows.is_empty():
		_tick = tick
		_rows = read
		_refresh()
	var zoom: float = get_viewport().get_canvas_transform().x.x
	var lod: bool = zoom < float(ArtData.param("lod.people_below_zoom", 1.0)) - 0.001
	if zoom != _zoom or lod != _lod:
		_zoom = zoom
		_lod = lod
		for body: Sprite2D in _bodies.values(): body.visible = not _lod
		queue_redraw()
	var hover: int = livestock_at(get_global_mouse_position())
	if hover != _hover:
		_hover = hover
		queue_redraw()


func _refresh() -> void:
	var ids: PackedInt64Array = _rows.get("ids", PackedInt64Array())
	var kinds: PackedInt64Array = _rows.get("kinds", PackedInt64Array())
	var xs: PackedInt64Array = _rows.get("xs", PackedInt64Array())
	var ys: PackedInt64Array = _rows.get("ys", PackedInt64Array())
	for id: int in _bodies.keys():
		if not ids.has(id):
			(_bodies[id] as Sprite2D).queue_free()
			_bodies.erase(id)
	var world_layer: Node2D = get_node("../World")
	for i: int in mini(ids.size(), mini(kinds.size(), mini(xs.size(), ys.size()))):
		var body: Sprite2D = _bodies.get(ids[i])
		if body == null:
			body = Sprite2D.new()
			body.name = "Livestock_%d" % ids[i]
			body.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
			world_layer.add_child(body)
			_bodies[ids[i]] = body
		body.visible = not _lod
		# The species' own still frame when bindings.json names it exactly; else the data-selected placeholder.
		var frame: String = WildAnimals.frame_id(kinds[i], -1, 0, false) if int(WildAnimals.row_of(kinds[i]).get("code", -1)) == kinds[i] else ""
		var art: Texture2D = ArtData.texture(frame) if not frame.is_empty() and ArtData.has(frame) else null
		body.texture = art if art != null else Icons.item_texture(Content.row("animals", kinds[i]))
		body.centered = art == null
		body.offset = -Vector2(ArtData.anchor(frame)) if art != null else Vector2(0, -body.texture.get_height() * 0.5)
		body.position = View.base_point(Vector2i(xs[i], ys[i]))
	queue_redraw()


func _draw() -> void:
	if _lod:
		var lod_ids: PackedInt64Array = _rows.get("ids", PackedInt64Array())
		var lod_kinds: PackedInt64Array = _rows.get("kinds", PackedInt64Array())
		var size: float = WildAnimals.marker_px(_zoom) / maxf(_zoom, 0.001)
		for i: int in mini(lod_ids.size(), lod_kinds.size()):
			var body: Sprite2D = _bodies.get(lod_ids[i])
			if body == null: continue
			var definition: Dictionary = Content.row("animals", lod_kinds[i])
			var tint: Color = Icons.material_color(str(definition.get("art_tint_material", "")))
			var centre: Vector2 = body.position - Vector2(0, size * 0.5)
			var tri := PackedVector2Array([centre + Vector2(0, -size * 0.5), centre + Vector2(size * 0.5, size * 0.5), centre + Vector2(-size * 0.5, size * 0.5)])
			draw_colored_polygon(tri, tint)
	if _hover < 0 or _font == null: return
	var ids: PackedInt64Array = _rows.get("ids", PackedInt64Array())
	var i: int = ids.find(_hover)
	if i < 0: return
	var kinds: PackedInt64Array = _rows.get("kinds", PackedInt64Array())
	var ages: PackedInt64Array = _rows.get("age_days", PackedInt64Array())
	var females: PackedInt64Array = _rows.get("female", PackedInt64Array())
	var hunger: PackedInt64Array = _rows.get("hunger_days", PackedInt64Array())
	if i >= mini(kinds.size(), mini(ages.size(), mini(females.size(), hunger.size()))): return
	var locale: Node = get_node_or_null("/root/Locale")
	var text: String = StoryText.trf(locale, "C1_LIVESTOCK_HOVER", {"animal": StoryText.animal(locale, kinds[i]), "id": _hover, "age": ages[i], "sex": StoryText.ltr(locale, "C1_SEX_FEMALE" if females[i] != 0 else "C1_SEX_MALE"), "hunger": hunger[i]})
	var camera: Camera2D = get_viewport().get_camera_2d()
	var zoom: float = camera.zoom.x if camera != null else 1.0
	var body: Sprite2D = _bodies[_hover]
	Overlay.draw_label(self, _font, text, body.position + Vector2(0, -40), zoom)


func livestock_at(point: Vector2) -> int:
	var best: int = -1
	for id: int in _bodies:
		var body: Sprite2D = _bodies[id]
		if body.get_rect().has_point(body.to_local(point)) and (best < 0 or id < best): best = id
	return best


func counts() -> Dictionary:
	return {"livestock": _bodies.size()}
