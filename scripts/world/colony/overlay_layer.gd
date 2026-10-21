# overlay_layer.gd (ColonyView/OverlayLayer in scenes/main.tscn): what the player ordered and what lies on the
#   ground.
# Two draw passes:
#   - the child node "Ground" (z 1: above the terrain, below canopy, buildings and people): stockpile zones as a
#     tamped-earth floor with a 1-px darker border on each side without a zone neighbour and a stake where two such
#     sides meet; log stacks by count (logs_s 1–8, logs_m 9–16, logs_l 17+); hut blueprints as the blueprint-blue
#     ghost with the delivered logs piled on its south row as they arrive (4 logs per tile: s, s, m, l);
#   - this node (z 4: above the canopy, below people): chop marks (an axe glyph at the tile's top-right), each
#     blueprint's label "통나무 delivered/needed · 작업 n%" on a dark backing box above it and a 3-px bar under it
#     (amber with the logs delivered until all are there, then green with the work done), and the count of the log
#     stack under the mouse on a small backing box.
# Redrawn only when the colony overlay version or the hovered stack changes (`get_colony_versions` is polled every
#   frame).
# Bridge methods: via SimReads get_colony_versions, get_colony_overlay; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const FONT_PATH: String = "res://assets/fonts/galmuri/Galmuri11.ttf"
const BACK: Color = Color(0.094, 0.082, 0.075, 0.92)
const BACK_LINE: Color = Color(0.275, 0.243, 0.212, 1.0)
const TEXT: Color = Color(0.96, 0.92, 0.84)
const BAR_LOGS: Color = Color(0.85, 0.62, 0.3, 1.0)
const BAR_WORK: Color = Color(0.55, 0.85, 0.45, 1.0)
const BAR_HEIGHT: int = 3
const FONT_SIZE: int = 12
const LOGS_PER_PILE: int = 4

var _world_sim: Node = null
var _version: int = -1
var _overlay: Dictionary = {}
var _font: Font = null
var _ground: Node2D = null
var _stacks: Dictionary = {}
var _hover: Vector2i = Vector2i(-1, -1)


func _ready() -> void:
	# Above the trees (2) and buildings (3), below people (5): marks and labels must read over canopies.
	z_index = 4
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_font = load(FONT_PATH) as Font
	_ground = Node2D.new()
	_ground.name = "Ground"
	_ground.z_index = -3
	_ground.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_ground.draw.connect(_draw_ground)
	add_child(_ground)
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
	if version != _version:
		_version = version
		_overlay = SimReads.read(_world_sim, "get_colony_overlay")
		_index_stacks()
		_ground.queue_redraw()
		queue_redraw()
	var hover: Vector2i = View.world_to_tile(get_global_mouse_position())
	if hover != _hover:
		var redraw: bool = _stacks.has(hover) or _stacks.has(_hover)
		_hover = hover
		if redraw:
			queue_redraw()


## Number of log stacks currently drawn (for tests and tools).
func stack_count() -> int:
	return _stacks.size()


func _index_stacks() -> void:
	_stacks.clear()
	var ixs: PackedInt32Array = _overlay.get("item_xs", PackedInt32Array())
	var iys: PackedInt32Array = _overlay.get("item_ys", PackedInt32Array())
	var counts: PackedInt32Array = _overlay.get("item_counts", PackedInt32Array())
	for i: int in mini(ixs.size(), mini(iys.size(), counts.size())):
		_stacks[Vector2i(ixs[i], iys[i])] = counts[i]


## The pile sprite for a stack of `count` logs.
static func pile_name(count: int) -> String:
	return "logs_s" if count <= 8 else ("logs_m" if count <= 16 else "logs_l")


func _draw_ground() -> void:
	var texture: Texture2D = View.texture()
	if texture == null or _overlay.is_empty():
		return
	var tile: Vector2 = Vector2(View.TILE, View.TILE)
	# Zones: floor, a border on each open side, a stake where two open sides meet.
	var zone_ids: PackedInt32Array = _overlay.get("zone_ids", PackedInt32Array())
	var zxs: PackedInt32Array = _overlay.get("zone_xs", PackedInt32Array())
	var zys: PackedInt32Array = _overlay.get("zone_ys", PackedInt32Array())
	var zone_at: Dictionary = {}
	for i: int in mini(zone_ids.size(), mini(zxs.size(), zys.size())):
		zone_at[Vector2i(zxs[i], zys[i])] = zone_ids[i]
	var floor_src: Rect2 = View.rect("stock_floor")
	for cell: Vector2i in zone_at.keys():
		var dest: Rect2 = Rect2(View.tile_to_world(cell), tile)
		_ground.draw_texture_rect_region(texture, dest, floor_src)
		var id: int = zone_at[cell]
		var open_n: bool = zone_at.get(cell + Vector2i(0, -1), -1) != id
		var open_e: bool = zone_at.get(cell + Vector2i(1, 0), -1) != id
		var open_s: bool = zone_at.get(cell + Vector2i(0, 1), -1) != id
		var open_w: bool = zone_at.get(cell + Vector2i(-1, 0), -1) != id
		for part: Array in [[open_n, "stock_edge_n"], [open_e, "stock_edge_e"], [open_s, "stock_edge_s"], [open_w, "stock_edge_w"],
				[open_n and open_w, "stock_stake_nw"], [open_n and open_e, "stock_stake_ne"],
				[open_s and open_w, "stock_stake_sw"], [open_s and open_e, "stock_stake_se"]]:
			if bool(part[0]):
				_ground.draw_texture_rect_region(texture, dest, View.rect(str(part[1])))
	# Log stacks.
	for cell: Vector2i in _stacks.keys():
		_ground.draw_texture_rect_region(texture, Rect2(View.tile_to_world(cell), tile), View.rect(pile_name(int(_stacks[cell]))))
	# Blueprints: the ghost, then the delivered logs piled on its south row.
	var sites: Array[Dictionary] = _sites()
	var ghost: Rect2 = View.rect("hut_ghost")
	for site: Dictionary in sites:
		var rect: Rect2 = site["rect"]
		_ground.draw_texture_rect_region(texture, rect, ghost)
		var left: int = int(site["delivered"])
		var columns: int = int(rect.size.x / View.TILE)
		for col: int in columns:
			if left <= 0:
				break
			var here: int = mini(left, LOGS_PER_PILE)
			left -= here
			var name: String = "logs_s" if here <= 2 else ("logs_m" if here == 3 else "logs_l")
			var origin: Vector2 = rect.position + Vector2(col * View.TILE, rect.size.y - View.TILE)
			_ground.draw_texture_rect_region(texture, Rect2(origin, tile), View.rect(name))


func _sites() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var sxs: PackedInt32Array = _overlay.get("site_xs", PackedInt32Array())
	var sys: PackedInt32Array = _overlay.get("site_ys", PackedInt32Array())
	var sws: PackedInt32Array = _overlay.get("site_ws", PackedInt32Array())
	var shs: PackedInt32Array = _overlay.get("site_hs", PackedInt32Array())
	var needed: PackedInt32Array = _overlay.get("site_needed", PackedInt32Array())
	var delivered: PackedInt32Array = _overlay.get("site_delivered", PackedInt32Array())
	var progress: PackedInt32Array = _overlay.get("site_progress", PackedInt32Array())
	var required: PackedInt32Array = _overlay.get("site_required", PackedInt32Array())
	var n: int = mini(mini(mini(sxs.size(), sys.size()), mini(sws.size(), shs.size())),
		mini(mini(needed.size(), delivered.size()), mini(progress.size(), required.size())))
	for i: int in n:
		out.append({
			"rect": Rect2(View.tile_to_world(Vector2i(sxs[i], sys[i])), Vector2(sws[i], shs[i]) * View.TILE),
			"needed": needed[i], "delivered": delivered[i], "progress": progress[i], "required": required[i],
		})
	return out


func _draw() -> void:
	var texture: Texture2D = View.texture()
	if texture == null or _overlay.is_empty():
		return
	var tile: Vector2 = Vector2(View.TILE, View.TILE)
	# Chop marks (this node sits over the canopy).
	var dxs: PackedInt32Array = _overlay.get("designation_xs", PackedInt32Array())
	var dys: PackedInt32Array = _overlay.get("designation_ys", PackedInt32Array())
	var mark: Rect2 = View.rect("mark_chop")
	for i: int in mini(dxs.size(), dys.size()):
		draw_texture_rect_region(texture, Rect2(View.tile_to_world(Vector2i(dxs[i], dys[i])), tile), mark)
	# Blueprint labels and bars.
	var locale: Node = get_node_or_null("/root/Locale")
	for site: Dictionary in _sites():
		var rect: Rect2 = site["rect"]
		var needed: int = int(site["needed"])
		var delivered: int = int(site["delivered"])
		var logs_done: float = clampf(float(delivered) / maxf(1.0, float(needed)), 0.0, 1.0)
		var work_done: float = clampf(float(site["progress"]) / maxf(1.0, float(site["required"])), 0.0, 1.0)
		var bar: Rect2 = Rect2(rect.position + Vector2(0, rect.size.y + 1), Vector2(rect.size.x, BAR_HEIGHT + 2))
		draw_rect(bar, BACK)
		var building: bool = delivered >= needed
		var fill: float = work_done if building else logs_done
		draw_rect(Rect2(bar.position + Vector2(1, 1), Vector2((bar.size.x - 2) * fill, BAR_HEIGHT)), BAR_WORK if building else BAR_LOGS)
		if _font == null:
			continue
		var params: Dictionary = {"delivered": delivered, "needed": needed}
		var logs_text: String = str(locale.call("trf", "UI_BLUEPRINT_LOGS", params)) if locale != null else "%d/%d" % [delivered, needed]
		var percent: int = floori(work_done * 100.0)
		var text: String = str(locale.call("trf", "UI_BLUEPRINT_PROGRESS_FMT", {"logs": logs_text, "percent": percent})) if locale != null else "%s %d%%" % [logs_text, percent]
		_label(text, Vector2(rect.position.x + rect.size.x * 0.5, rect.position.y - 2), true)
	# The hovered stack's count.
	if _stacks.has(_hover) and _font != null:
		var at: Vector2 = View.tile_to_world(_hover) + Vector2(View.TILE * 0.5, -1)
		_label(str(int(_stacks[_hover])), at, true)


## Text centred on `anchor.x` with its box's bottom at `anchor.y`, on a dark backing box (art_ui panel colours).
func _label(text: String, anchor: Vector2, centred: bool) -> void:
	var size: Vector2 = _font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE)
	var box: Rect2 = Rect2(Vector2(anchor.x - (size.x * 0.5 if centred else 0.0) - 3, anchor.y - FONT_SIZE - 3), Vector2(size.x + 6, FONT_SIZE + 3))
	box.position = box.position.round()
	draw_rect(box, BACK)
	draw_rect(box, BACK_LINE, false, 1.0)
	draw_string(_font, box.position + Vector2(3, FONT_SIZE - 1), text, HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE, TEXT)
