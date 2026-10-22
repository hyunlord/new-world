# overlay_layer.gd (ColonyView/OverlayLayer in scenes/main.tscn): what the player ordered and what lies on the
#   ground.
# Two draw passes:
#   - the child node "Ground" (z 1: above the terrain, below canopy, buildings and people): stockpile zones as a
#     tamped-earth floor with a 1-px darker border on each side without a zone neighbour and a stake where two such
#     sides meet; M2 resource nodes (boulder, flint outcrop); item stacks by kind (logs by count: logs_s 1–8,
#     logs_m 9–16, logs_l 17+; the M2 kinds by their item sprite); hut blueprints as the blueprint-blue ghost with
#     the delivered logs piled on its south row as they arrive (4 logs per tile: s, s, m, l); M2 workshop
#     blueprints as the workshop's sprite tinted blueprint-blue;
#   - this node (z 4: above the canopy, below people): chop marks (an axe glyph) and M2 quarry marks (a pick glyph)
#     at the tile's top-right, each blueprint's label ("통나무 d/n" for a hut, "재료 d/n" for a workshop, then
#     "· 작업 n%") on a dark backing box above it and a 3-px bar under it (amber with the materials delivered until
#     all are there, then green with the work done), and, under the mouse, a stack's count (logs) or
#     "부싯돌 날 ×3 · 품질 62" (M2 kinds), or a node's "바위 · 돌 12개 남음", on a small backing box.
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
const EconomyText = preload("res://scripts/ui/economy_text.gd")
## Resource node sprites by node kind (0 boulder, 1 flint outcrop), their names and the item they yield.
const NODE_SPRITES: Array[String] = ["boulder", "flint_outcrop"]
const NODE_KEYS: Array[String] = ["NODE_BOULDER", "NODE_FLINT_OUTCROP"]
const NODE_ITEMS: Array[int] = [2, 3]
## Designation marks by designation kind (0 chop, 1 quarry).
const MARKS: Array[String] = ["mark_chop", "mark_quarry"]
const BLUEPRINT: Color = Color(0.55, 0.75, 1.0, 0.6)

var _world_sim: Node = null
var _version: int = -1
var _overlay: Dictionary = {}
var _font: Font = null
var _ground: Node2D = null
## Tile -> {kind, count, quality} of the item stack on it.
var _stacks: Dictionary = {}
## Tile -> {kind, left} of the resource node on it.
var _nodes: Dictionary = {}
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
		var redraw: bool = _stacks.has(hover) or _stacks.has(_hover) or _nodes.has(hover) or _nodes.has(_hover)
		_hover = hover
		if redraw:
			queue_redraw()


## Number of item stacks currently drawn (for tests and tools).
func stack_count() -> int:
	return _stacks.size()


## Number of resource nodes currently drawn (for tests and tools).
func node_count() -> int:
	return _nodes.size()


func _index_stacks() -> void:
	_stacks.clear()
	var ixs: PackedInt32Array = _overlay.get("item_xs", PackedInt32Array())
	var iys: PackedInt32Array = _overlay.get("item_ys", PackedInt32Array())
	var counts: PackedInt32Array = _overlay.get("item_counts", PackedInt32Array())
	var kinds: PackedInt32Array = _overlay.get("item_kinds", PackedInt32Array())
	var qualities: PackedInt32Array = _overlay.get("item_qualities", PackedInt32Array())
	for i: int in mini(ixs.size(), mini(iys.size(), counts.size())):
		var kind: int = kinds[i] if i < kinds.size() else 0
		var quality: int = qualities[i] if i < qualities.size() else 50
		_stacks[Vector2i(ixs[i], iys[i])] = {"kind": kind, "count": counts[i], "quality": quality}
	_nodes.clear()
	var nxs: PackedInt32Array = _overlay.get("node_xs", PackedInt32Array())
	var nys: PackedInt32Array = _overlay.get("node_ys", PackedInt32Array())
	var nkinds: PackedInt32Array = _overlay.get("node_kinds", PackedInt32Array())
	var left: PackedInt32Array = _overlay.get("node_left", PackedInt32Array())
	for i: int in mini(mini(nxs.size(), nys.size()), mini(nkinds.size(), left.size())):
		if nkinds[i] >= 0 and nkinds[i] < NODE_SPRITES.size():
			_nodes[Vector2i(nxs[i], nys[i])] = {"kind": nkinds[i], "left": left[i]}


## The ground sprite for an item stack of `kind` and `count` (logs by count, else the kind's sprite).
static func stack_sprite(kind: int, count: int) -> String:
	if kind <= 0 or kind >= EconomyText.ITEM_SPRITES.size():
		return pile_name(count)
	return EconomyText.ITEM_SPRITES[kind]


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
	# Resource nodes (M2).
	for cell: Vector2i in _nodes.keys():
		var node: Dictionary = _nodes[cell]
		_ground.draw_texture_rect_region(texture, Rect2(View.tile_to_world(cell), tile), View.rect(NODE_SPRITES[int(node["kind"])]))
	# Item stacks.
	for cell: Vector2i in _stacks.keys():
		var stack: Dictionary = _stacks[cell]
		_ground.draw_texture_rect_region(texture, Rect2(View.tile_to_world(cell), tile), View.rect(stack_sprite(int(stack["kind"]), int(stack["count"]))))
	# Blueprints: the hut ghost with the delivered logs piled on its south row; a workshop's sprite tinted blue.
	var sites: Array[Dictionary] = _sites()
	var ghost: Rect2 = View.rect("hut_ghost")
	for site: Dictionary in sites:
		var rect: Rect2 = site["rect"]
		var kind: int = int(site["kind"])
		if kind > 0 and kind < EconomyText.BUILD_KINDS.size():
			_ground.draw_rect(rect, Color(BLUEPRINT, 0.15))
			_ground.draw_texture_rect_region(texture, rect, View.rect(str(EconomyText.BUILD_KINDS[kind]["sprite"])), BLUEPRINT)
			_ground.draw_rect(rect, BLUEPRINT, false, 1.0)
			continue
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
	var kinds: PackedInt32Array = _overlay.get("site_kinds", PackedInt32Array())
	var n: int = mini(mini(mini(sxs.size(), sys.size()), mini(sws.size(), shs.size())),
		mini(mini(needed.size(), delivered.size()), mini(progress.size(), required.size())))
	for i: int in n:
		out.append({
			"rect": Rect2(View.tile_to_world(Vector2i(sxs[i], sys[i])), Vector2(sws[i], shs[i]) * View.TILE),
			"needed": needed[i], "delivered": delivered[i], "progress": progress[i], "required": required[i],
			"kind": kinds[i] if i < kinds.size() else 0,
		})
	return out


func _draw() -> void:
	var texture: Texture2D = View.texture()
	if texture == null or _overlay.is_empty():
		return
	var tile: Vector2 = Vector2(View.TILE, View.TILE)
	# Chop and quarry marks (this node sits over the canopy).
	var dxs: PackedInt32Array = _overlay.get("designation_xs", PackedInt32Array())
	var dys: PackedInt32Array = _overlay.get("designation_ys", PackedInt32Array())
	var dkinds: PackedInt32Array = _overlay.get("designation_kinds", PackedInt32Array())
	for i: int in mini(dxs.size(), dys.size()):
		var mark_kind: int = clampi(dkinds[i], 0, MARKS.size() - 1) if i < dkinds.size() else 0
		draw_texture_rect_region(texture, Rect2(View.tile_to_world(Vector2i(dxs[i], dys[i])), tile), View.rect(MARKS[mark_kind]))
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
		var needs_key: String = "UI_BLUEPRINT_LOGS" if int(site["kind"]) == 0 else "UI_BLUEPRINT_MATERIALS"
		var logs_text: String = str(locale.call("trf", needs_key, params)) if locale != null else "%d/%d" % [delivered, needed]
		var percent: int = floori(work_done * 100.0)
		var text: String = str(locale.call("trf", "UI_BLUEPRINT_PROGRESS_FMT", {"logs": logs_text, "percent": percent})) if locale != null else "%s %d%%" % [logs_text, percent]
		_label(text, Vector2(rect.position.x + rect.size.x * 0.5, rect.position.y - 2), true)
	# The hovered stack's count (logs) or name, count and quality (M2 kinds); else the hovered node.
	var at: Vector2 = View.tile_to_world(_hover) + Vector2(View.TILE * 0.5, -1)
	if _stacks.has(_hover) and _font != null:
		var stack: Dictionary = _stacks[_hover]
		var kind: int = int(stack["kind"])
		var text: String = str(int(stack["count"]))
		if kind > 0:
			text = EconomyText.stack(locale, kind, "", int(stack["count"]), int(stack["quality"]))
		_label(text, at, true)
	elif _nodes.has(_hover) and _font != null:
		var node: Dictionary = _nodes[_hover]
		var node_kind: int = int(node["kind"])
		_label(EconomyText.trf(locale, "UI_NODE_LEFT_FMT", {"name": EconomyText.ltr(locale, NODE_KEYS[node_kind]),
			"item": EconomyText.item(locale, NODE_ITEMS[node_kind]), "left": int(node["left"])}), at, true)


## Text centred on `anchor.x` with its box's bottom at `anchor.y`, on a dark backing box (art_ui panel colours).
func _label(text: String, anchor: Vector2, centred: bool) -> void:
	var size: Vector2 = _font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE)
	var box: Rect2 = Rect2(Vector2(anchor.x - (size.x * 0.5 if centred else 0.0) - 3, anchor.y - FONT_SIZE - 3), Vector2(size.x + 6, FONT_SIZE + 3))
	box.position = box.position.round()
	draw_rect(box, BACK)
	draw_rect(box, BACK_LINE, false, 1.0)
	draw_string(_font, box.position + Vector2(3, FONT_SIZE - 1), text, HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE, TEXT)
