# animal_layer.gd (ColonyView/AnimalLayer in scenes/main.tscn): the M5 animals and hunt zones (goal 3.2, 3.5) at
#   64-px tiles (M-ART2 §3.5).
# Each animal of get_animals is a pooled standing Sprite2D in the y-sorted ColonyView/World: the manifest frame
#   `<species>/<S|E>/<pose>` (species and pose from bindings.json `animals`: the pose per state code `poses`, the
#   row's `still` pose when a `moving` pose is shown on an animal that is not stepping), its anchor at the drawn base
#   point (tile_left + 32, tile_top + 43), easing from tile to tile like the people. Facing: the last step east → E,
#   west → E flipped, north or south → S (N = S, no back art; the 35° S rule is in the art).
# Drawn by this node above the world layer (z 3): a deer that may be hunted (`huntable`) carries a yellow ring over
#   its head (full for its own mark, dimmer when only a zone covers it); below 1× (`lod.people_below_zoom`) every
#   animal is a filled triangle in its bindings `lod_colour` with a 1-px ink ring, `lod.animal_marker_px` SCREEN
#   pixels for the nearest listed zoom ("0.5" → 10, "0.25" → 6), and the sprites are hidden. Under the mouse an
#   animal shows "사슴 · 돌아다님 · 30" (kind, state, hit points) on a dark backing box (labels follow the camera). The
#   hunt zones (`zones`, x0 y0 x1 y1 quadruples) are drawn flat by the child "Zones" (z −5, under the world layer)
#   as an amber frame with a faint fill. A carcass is not an animal: the kill's meat and hide lie as ground stacks
#   (OverlayLayer draws the dead frame).
# The drawn rect (the frame's rect around its anchor, and the tile cell) is the click target (`animal_at`): the order
#   tool's hunt click (F7-3) and the draft's right-click attack. `drawn_position` is the drawn tile cell's top-left.
# M5.5: this node creates the visitor layer (ColonyView/VisitorLayer, visitor_layer.gd: migrant waves and caravans)
#   beside itself.
# Bridge methods: via SimReads get_animals, get_world_info; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
const VisitorLayer = preload("res://scripts/world/colony/visitor_layer.gd")
const OverlayLayer = preload("res://scripts/world/colony/overlay_layer.gd")
const FONT_PATH: String = "res://assets/fonts/galmuri/Galmuri11.ttf"
const TILE: int = View.TILE
const ZONE_ALPHA: float = 0.12
const ZONE_MARK_ALPHA: float = 0.55
const MARK: Color = Color(0.98, 0.84, 0.3)

var _world_sim: Node = null
var _world: Node2D = null
var _font: Font = null
var _zones: Node2D = null
var _animals: Dictionary = {}
var _read: Dictionary = {}
var _zone_rects: Array[Rect2i] = []
var _zones_key: int = 0
var _hover: int = -1
var _frame: int = 0
var _lod: bool = false
var _zoom: float = 1.0
var _drawn: bool = false


class Animal:
	extends RefCounted
	var id: int = 0
	var kind: int = 0
	var state: int = 0
	var tile: Vector2i = Vector2i.ZERO
	## The drawn tile cell's top-left (eased).
	var position: Vector2 = Vector2.ZERO
	var from_pos: Vector2 = Vector2.ZERO
	var to_pos: Vector2 = Vector2.ZERO
	var elapsed: float = 1.0
	var duration: float = 0.25
	var seen: int = 0
	var facing: int = 0
	var flip: bool = false
	var huntable: bool = false
	var marked: bool = false
	var body: Sprite2D
	var frame_id: String = ""


func _ready() -> void:
	z_as_relative = false
	z_index = 3
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_font = load(FONT_PATH) as Font
	_world = get_node_or_null("../World")
	_zones = Node2D.new()
	_zones.name = "Zones"
	_zones.z_as_relative = false
	_zones.z_index = -5
	_zones.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_zones.draw.connect(_draw_zones)
	add_child(_zones)
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)
	if get_parent() != null and get_parent().get_node_or_null("VisitorLayer") == null:
		var visitors: Node2D = VisitorLayer.new()
		visitors.name = "VisitorLayer"
		get_parent().add_child.call_deferred(visitors)


func _on_world_reset() -> void:
	for animal: Animal in _animals.values():
		if animal.body != null:
			animal.body.queue_free()
	_animals.clear()
	_zone_rects.clear()
	_zones_key = 0
	_zones.queue_redraw()


## The bindings `animals` row of kind `kind` (its family fallback when unknown).
static func row_of(kind: int) -> Dictionary:
	return ArtData.binding("animals", kind)


## The manifest frame id of an animal of `kind` in `state`, facing index `facing` (0 = first facing, 1 = second),
## `moving` when it is stepping.
static func frame_id(kind: int, state: int, facing: int, moving: bool) -> String:
	var row: Dictionary = row_of(kind)
	var poses: Dictionary = row.get("poses", {})
	var pose: String = str(poses.get(str(state), row.get("still", "")))
	if not moving and (row.get("moving", []) as Array).has(pose):
		pose = str(row.get("still", pose))
	var facings: Array = (ArtData.manifest().get("animals", {}) as Dictionary).get("facings", [])
	if facings.is_empty():
		return ""
	return "%s/%s/%s" % [str(row.get("species", "")), str(facings[clampi(facing, 0, facings.size() - 1)]), pose]


func _process(delta: float) -> void:
	if _world_sim == null:
		return
	_frame += 1
	_read = SimReads.read(_world_sim, "get_animals")
	var ids: PackedInt64Array = _read.get("ids", PackedInt64Array())
	var kinds: PackedInt64Array = _read.get("kinds", PackedInt64Array())
	var xs: PackedInt32Array = _read.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = _read.get("ys", PackedInt32Array())
	var states: PackedInt32Array = _read.get("states", PackedInt32Array())
	var huntable: PackedByteArray = _read.get("huntable", PackedByteArray())
	var marked: PackedByteArray = _read.get("marked", PackedByteArray())
	var n: int = mini(mini(ids.size(), kinds.size()), mini(xs.size(), ys.size()))
	var duration: float = _step_seconds()
	_zoom = get_viewport().get_canvas_transform().x.x
	var lod: bool = _zoom < float(ArtData.param("lod.people_below_zoom", 1.0)) - 0.001
	for i: int in n:
		var id: int = ids[i]
		var tile: Vector2i = Vector2i(xs[i], ys[i])
		var animal: Animal = _animals.get(id)
		if animal == null:
			animal = Animal.new()
			animal.id = id
			animal.tile = tile
			animal.position = View.tile_to_world(tile)
			animal.from_pos = animal.position
			animal.to_pos = animal.position
			_animals[id] = animal
		elif tile != animal.tile:
			animal.from_pos = animal.position
			animal.to_pos = View.tile_to_world(tile)
			if tile.x != animal.tile.x:
				animal.facing = 1
				animal.flip = tile.x < animal.tile.x
			else:
				animal.facing = 0
				animal.flip = false
			animal.tile = tile
			animal.elapsed = 0.0
			animal.duration = duration
		animal.kind = kinds[i]
		animal.state = states[i] if i < states.size() else 0
		animal.seen = _frame
		animal.huntable = i < huntable.size() and huntable[i] != 0
		animal.marked = i < marked.size() and marked[i] != 0
	for key: int in _animals.keys():
		var animal: Animal = _animals[key]
		if animal.seen != _frame:
			if animal.body != null:
				animal.body.queue_free()
			_animals.erase(key)
			continue
		if animal.elapsed < animal.duration:
			animal.elapsed = minf(animal.duration, animal.elapsed + delta)
			animal.position = animal.from_pos.lerp(animal.to_pos, animal.elapsed / animal.duration).round()
		else:
			animal.position = animal.to_pos
		_place(animal, lod)
	_lod = lod
	_refresh_zones()
	_hover = animal_at(get_global_mouse_position())
	var any_mark: bool = false
	for animal: Animal in _animals.values():
		any_mark = any_mark or animal.huntable
	var needs: bool = _hover >= 0 or any_mark or (_lod and not _animals.is_empty())
	if needs or _drawn:
		queue_redraw()
	_drawn = needs


## The animal's sprite: frame by state and facing, anchored at the drawn base point; hidden under LOD.
func _place(animal: Animal, lod: bool) -> void:
	if _world == null:
		return
	if animal.body == null:
		animal.body = Sprite2D.new()
		animal.body.centered = false
		animal.body.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
		_world.add_child(animal.body)
	var fid: String = frame_id(animal.kind, animal.state, animal.facing, animal.elapsed < animal.duration)
	if fid != animal.frame_id:
		animal.frame_id = fid
		animal.body.texture = ArtData.texture(fid)
		animal.body.offset = -Vector2(ArtData.anchor(fid))
	animal.body.flip_h = animal.flip
	if animal.flip and animal.body.texture != null:
		# Flipping mirrors the canvas: keep the anchor on the base point.
		animal.body.offset.x = -float(animal.body.texture.get_width() - ArtData.anchor(fid).x)
	animal.body.position = animal.position + Vector2(View.BASE)
	animal.body.visible = not lod


## Hunt zones from the read (redrawn only when they change).
func _refresh_zones() -> void:
	var zones: PackedInt32Array = _read.get("zones", PackedInt32Array())
	var key: int = hash(zones)
	if key == _zones_key:
		return
	_zones_key = key
	_zone_rects.clear()
	for k: int in range(0, zones.size() - 3, 4):
		var a: Vector2i = Vector2i(mini(zones[k], zones[k + 2]), mini(zones[k + 1], zones[k + 3]))
		var b: Vector2i = Vector2i(maxi(zones[k], zones[k + 2]), maxi(zones[k + 1], zones[k + 3]))
		_zone_rects.append(Rect2i(a, b - a + Vector2i.ONE))
	_zones.queue_redraw()


func _zone_colour() -> Color:
	var v: Variant = ArtData.param("zones.hunt_rgb", [242, 184, 77])
	return Color8(int(v[0]), int(v[1]), int(v[2])) if v is Array and (v as Array).size() >= 3 else Color(0.95, 0.72, 0.3)


func _draw_zones() -> void:
	var zone: Color = _zone_colour()
	for r: Rect2i in _zone_rects:
		var rect: Rect2 = Rect2(View.tile_to_world(r.position), Vector2(r.size) * TILE)
		_zones.draw_rect(rect, Color(zone, ZONE_ALPHA))
		_zones.draw_rect(rect, Color(zone, 0.9), false, 2.0)


## The LOD marker size in screen pixels for camera zoom `zoom` (the nearest listed zoom in `lod.animal_marker_px`).
static func marker_px(zoom: float) -> float:
	var table: Variant = ArtData.param("lod.animal_marker_px", {})
	var best: float = 8.0
	var best_d: float = INF
	if table is Dictionary:
		for k: Variant in (table as Dictionary).keys():
			var d: float = absf(float(str(k)) - zoom)
			if d < best_d:
				best_d = d
				best = float((table as Dictionary)[k])
	return best


func _draw() -> void:
	var inv: float = 1.0 / maxf(0.05, _zoom)
	if _lod:
		var ring_v: Variant = ArtData.param("lod.ring_rgba", [35, 29, 23, 255])
		var ring: Color = Color8(int(ring_v[0]), int(ring_v[1]), int(ring_v[2]), int(ring_v[3])) if ring_v is Array and (ring_v as Array).size() == 4 else Color8(35, 29, 23)
		var size: float = marker_px(_zoom) * inv
		for animal: Animal in _animals.values():
			var c: Vector2 = animal.position + Vector2(View.BASE) - Vector2(0, size * 0.5)
			var lc: Variant = row_of(animal.kind).get("lod_colour", [200, 200, 200])
			var fill: Color = Color8(int(lc[0]), int(lc[1]), int(lc[2])) if lc is Array and (lc as Array).size() >= 3 else Color.WHITE
			var h: float = size * 0.5
			var tri: PackedVector2Array = PackedVector2Array([c + Vector2(0, -h), c + Vector2(h, h), c + Vector2(-h, h)])
			draw_colored_polygon(tri, fill)
			var outline: PackedVector2Array = tri.duplicate()
			outline.append(tri[0])
			draw_polyline(outline, ring, inv)
	for animal: Animal in _animals.values():
		if not animal.huntable:
			continue
		var head: Vector2 = animal.position + Vector2(View.BASE) - Vector2(0, TILE * 0.9 if not _lod else marker_px(_zoom) * inv * 1.6)
		var col: Color = Color(MARK, 1.0 if animal.marked else ZONE_MARK_ALPHA)
		draw_arc(head, 7.0 * (inv if _lod else 1.0), 0.0, TAU, 20, Color(0.137, 0.114, 0.09, col.a), 4.0 * (inv if _lod else 1.0))
		draw_arc(head, 7.0 * (inv if _lod else 1.0), 0.0, TAU, 20, col, 2.0 * (inv if _lod else 1.0))
	var hovered: Animal = _animals.get(_hover)
	if hovered == null or _font == null:
		return
	var i: int = (_read.get("ids", PackedInt64Array()) as PackedInt64Array).find(_hover)
	if i < 0:
		return
	var locale: Node = get_node_or_null("/root/Locale")
	var states: PackedInt32Array = _read.get("states", PackedInt32Array())
	var hps: PackedInt32Array = _read.get("hps", PackedInt32Array())
	var text: String = StoryText.trf(locale, "M5_ANIMAL_HOVER_FMT", {"animal": StoryText.animal(locale, hovered.kind),
		"state": StoryText.animal_state(locale, states[i] if i < states.size() else -1), "hp": hps[i] if i < hps.size() else 0})
	var top: float = _drawn_rect(hovered).position.y
	OverlayLayer.draw_label(self, _font, text, Vector2(hovered.position.x + TILE * 0.5, top - 2), _zoom)


## Seconds one step lasts at the current speed (as AgentLayer): move_interval_ticks / (12 × speed).
func _step_seconds() -> float:
	var info: Dictionary = SimReads.read(_world_sim, "get_world_info")
	var interval: float = float(info.get("move_interval_ticks", 3))
	var camera: Node = get_node_or_null("/root/Main/Camera2D")
	var speed: float = float(camera.call("get_speed")) if camera != null and camera.has_method("get_speed") else 1.0
	return maxf(0.02, interval / (12.0 * maxf(1.0, speed)))


## The drawn rect of `animal`: its frame around the anchor (or the LOD marker), merged with its tile cell.
func _drawn_rect(animal: Animal) -> Rect2:
	var cell: Rect2 = Rect2(animal.position, Vector2(TILE, TILE))
	if animal.body == null or animal.body.texture == null or _lod:
		return cell
	var size: Vector2 = animal.body.texture.get_size()
	return cell.merge(Rect2(animal.body.position + animal.body.offset, size))


## The AnimalId of the animal drawn at world position `p` (the lowest on screen wins), or -1.
func animal_at(p: Vector2) -> int:
	var best: int = -1
	var best_y: float = -INF
	for animal: Animal in _animals.values():
		if _drawn_rect(animal).has_point(p) and animal.position.y > best_y:
			best = animal.id
			best_y = animal.position.y
	return best


## The kind of drawn animal `id` (0 wolf, 1 deer), -1 when not drawn.
func kind_of(id: int) -> int:
	var animal: Animal = _animals.get(id)
	return animal.kind if animal != null else -1


## The drawn tile cell's top-left of animal `id` (for tools), or (-1, -1).
func drawn_position(id: int) -> Vector2:
	var animal: Animal = _animals.get(id)
	return animal.position if animal != null else Vector2(-1, -1)


## Whether animal `id` shows its hunt mark (for tools).
func marked_shown(id: int) -> bool:
	var animal: Animal = _animals.get(id)
	return animal != null and animal.huntable


## The manifest frame drawn for animal `id` ("" when not drawn; for tests and tools).
func frame_of(id: int) -> String:
	var animal: Animal = _animals.get(id)
	return animal.frame_id if animal != null else ""


## How many animals are drawn and how many hunt zones (for tests and tools).
func counts() -> Dictionary:
	return {"animals": _animals.size(), "zones": _zone_rects.size()}
