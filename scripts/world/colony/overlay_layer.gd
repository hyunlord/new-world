# overlay_layer.gd (ColonyView/OverlayLayer in scenes/main.tscn): what the player ordered and what lies on the
#   ground — stockpile zone tint and edges, chop marks, log stacks (three sizes) and hut blueprints (a ghost
#   with "통나무 delivered/needed" and a work bar).
# Redrawn only when the colony overlay version changes (`get_colony_versions` is polled every frame).
# Bridge methods: via SimReads get_colony_versions, get_colony_overlay; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const FONT_PATH: String = "res://assets/fonts/galmuri/Galmuri11.ttf"
const ZONE_COLOR: Color = Color(0.85, 0.72, 0.38, 0.45)
const GHOST_COLOR: Color = Color(0.82, 0.9, 1.0, 0.9)
const BAR_BACK: Color = Color(0.1, 0.08, 0.06, 0.85)
const BAR_LOGS: Color = Color(0.85, 0.62, 0.3, 1.0)
const BAR_WORK: Color = Color(0.55, 0.85, 0.45, 1.0)

var _world_sim: Node = null
var _version: int = -1
var _overlay: Dictionary = {}
var _font: Font = null


func _ready() -> void:
	# Above the trees (2) and buildings (3), below people (5): marks must read over canopies.
	z_index = 4
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_font = load(FONT_PATH) as Font
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	_version = -1


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	var versions: Dictionary = SimReads.read(_world_sim, "get_colony_versions")
	var version: int = int(versions.get("overlay_version", -1))
	if version == _version:
		return
	_version = version
	_overlay = SimReads.read(_world_sim, "get_colony_overlay")
	queue_redraw()


## Number of log stacks currently drawn (for tests and tools).
func stack_count() -> int:
	return (_overlay.get("item_xs", PackedInt32Array()) as PackedInt32Array).size()


func _draw() -> void:
	var texture: Texture2D = View.texture()
	if texture == null or _overlay.is_empty():
		return
	var tile: Vector2 = Vector2(View.TILE, View.TILE)
	# Zones: a tint on every cell, an edge where the neighbour is not the same zone.
	var zone_ids: PackedInt32Array = _overlay.get("zone_ids", PackedInt32Array())
	var zxs: PackedInt32Array = _overlay.get("zone_xs", PackedInt32Array())
	var zys: PackedInt32Array = _overlay.get("zone_ys", PackedInt32Array())
	var zone_at: Dictionary = {}
	for i: int in mini(zone_ids.size(), mini(zxs.size(), zys.size())):
		zone_at[Vector2i(zxs[i], zys[i])] = zone_ids[i]
	for cell: Vector2i in zone_at.keys():
		var origin: Vector2 = View.tile_to_world(cell)
		draw_texture_rect_region(texture, Rect2(origin, tile), View.rect("zone_tint"), ZONE_COLOR)
		var id: int = zone_at[cell]
		var edge: Color = ZONE_COLOR.lightened(0.2)
		if zone_at.get(cell + Vector2i(0, -1), -1) != id:
			draw_line(origin, origin + Vector2(View.TILE, 0), edge, 1.0)
		if zone_at.get(cell + Vector2i(0, 1), -1) != id:
			draw_line(origin + Vector2(0, View.TILE - 1), origin + Vector2(View.TILE, View.TILE - 1), edge, 1.0)
		if zone_at.get(cell + Vector2i(-1, 0), -1) != id:
			draw_line(origin, origin + Vector2(0, View.TILE), edge, 1.0)
		if zone_at.get(cell + Vector2i(1, 0), -1) != id:
			draw_line(origin + Vector2(View.TILE - 1, 0), origin + Vector2(View.TILE - 1, View.TILE), edge, 1.0)
	# Log stacks.
	var ixs: PackedInt32Array = _overlay.get("item_xs", PackedInt32Array())
	var iys: PackedInt32Array = _overlay.get("item_ys", PackedInt32Array())
	var counts: PackedInt32Array = _overlay.get("item_counts", PackedInt32Array())
	for i: int in mini(ixs.size(), mini(iys.size(), counts.size())):
		var size_name: String = "logs_s" if counts[i] <= 8 else ("logs_m" if counts[i] <= 16 else "logs_l")
		draw_texture_rect_region(texture, Rect2(View.tile_to_world(Vector2i(ixs[i], iys[i])), tile), View.rect(size_name))
	# Chop marks (this node sits over the tree layer).
	var dxs: PackedInt32Array = _overlay.get("designation_xs", PackedInt32Array())
	var dys: PackedInt32Array = _overlay.get("designation_ys", PackedInt32Array())
	for i: int in mini(dxs.size(), dys.size()):
		draw_texture_rect_region(texture, Rect2(View.tile_to_world(Vector2i(dxs[i], dys[i])), tile), View.rect("mark_chop"))
	# Blueprint sites: ghost, logs delivered/needed, then the work bar once building.
	var sxs: PackedInt32Array = _overlay.get("site_xs", PackedInt32Array())
	var sys: PackedInt32Array = _overlay.get("site_ys", PackedInt32Array())
	var sws: PackedInt32Array = _overlay.get("site_ws", PackedInt32Array())
	var shs: PackedInt32Array = _overlay.get("site_hs", PackedInt32Array())
	var needed: PackedInt32Array = _overlay.get("site_needed", PackedInt32Array())
	var delivered: PackedInt32Array = _overlay.get("site_delivered", PackedInt32Array())
	var progress: PackedInt32Array = _overlay.get("site_progress", PackedInt32Array())
	var required: PackedInt32Array = _overlay.get("site_required", PackedInt32Array())
	for i: int in sxs.size():
		if i >= sys.size() or i >= sws.size() or i >= shs.size() or i >= needed.size() or i >= delivered.size() or i >= progress.size() or i >= required.size():
			break
		var rect: Rect2 = Rect2(View.tile_to_world(Vector2i(sxs[i], sys[i])), Vector2(sws[i], shs[i]) * View.TILE)
		draw_texture_rect_region(texture, rect, View.rect("hut_ghost"), GHOST_COLOR)
		var bar: Rect2 = Rect2(rect.position + Vector2(2, rect.size.y - 5), Vector2(rect.size.x - 4, 3))
		draw_rect(bar, BAR_BACK)
		var logs_done: float = float(delivered[i]) / maxf(1.0, float(needed[i]))
		var work_done: float = float(progress[i]) / maxf(1.0, float(required[i]))
		draw_rect(Rect2(bar.position, Vector2(bar.size.x * logs_done, 1)), BAR_LOGS)
		draw_rect(Rect2(bar.position + Vector2(0, 2), Vector2(bar.size.x * work_done, 1)), BAR_WORK)
		if _font != null:
			var locale: Node = get_node_or_null("/root/Locale")
			var params: Dictionary = {"delivered": delivered[i], "needed": needed[i]}
			var text: String = str(locale.call("trf", "UI_BLUEPRINT_LOGS", params)) if locale != null else "%d/%d" % [delivered[i], needed[i]]
			draw_string(_font, rect.position + Vector2(1, -2), text, HORIZONTAL_ALIGNMENT_LEFT, -1, 12, Color(0.96, 0.92, 0.84))
