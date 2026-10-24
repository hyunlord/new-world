# overlay_layer.gd (ColonyView/OverlayLayer in scenes/main.tscn): what the player ordered and what lies on the
#   ground, at 64-px tiles (M-ART2 §3.4 zones/designations/sites, §3.5 things).
# Standing sprites in the y-sorted ColonyView/World (position = base point, offset = −anchor; things lying flat sit
#   one pixel above the base line so a person on the same tile draws over them):
#   - item stacks of get_colony_overlay at the tile's base point, by bindings.json `items` (count buckets 1 / 2–8 /
#     9+ by `items.bucket_limits`: a row with `assets` has one sprite per bucket; a row with `asset` is copied 1 / 3 /
#     9 times at the `items.stack_layout` offsets into one cached texture, and its `many` sprite replaces it at 9+);
#     an item whose row names `carcass_of` lying outside every stockpile zone draws that species' dead frame
#     (manifest `<species>/<first facing>/<the animals row's dead pose>`) instead;
#   - resource nodes by bindings `nodes` (`depleted` below `depleted_below_permille` of the most seen in the node);
#   - blueprint sites as their finished look through site_ghost.gd (pale outline + `sites.wash_alpha` wash): hut
#     rings, workshops, bedrolls and doors as one ghost each; wall sites of every style baked together (so a run
#     joins) and cut into per-tile ghosts; floor sites flat (below).
# Flat, drawn by child nodes: "Ground" (z −6, between the ground chunks and the world layer): zones as a
#   `zones.tint_alpha` tint with `zones.tick_px` corner ticks only on a zone's outer (convex) corners, no grid lines,
#   and floor-site ghosts; "Marks" (z 3, above the world layer): designations as ink marks (`marks.ink_rgb` on a
#   pale disc; the mark kind → glyph: 0 chop axe, 1 quarry pick, 2 dig and 4 clay pit spade, 3 deconstruct cross).
# This node (z 6): each blueprint's bar (amber with the materials delivered until all are there, then green with the
#   work done; inside the tile's bottom for a space site) and label ("통나무 d/n · 작업 n%"; a space site's only under
#   the mouse), and under the mouse a stack's "부싯돌 날 ×3 · 품질 62" (+ " · 방: 창고 2", " · 지붕 아래") or a node's
#   "바위 · 돌 12개 남음". Text follows the camera: `labels.font_px` screen pixels (Galmuri11, 24 = 2 × its 12-px
#   grid) at every zoom, the backing box scaled with it; redrawn when the zoom changes.
# The room overlay is the child "Rooms" (room_layer.gd).
# Sprites are rebuilt only when the colony overlay version moves (and on an F12 params reload), pooled by tile; item
#   stacks and nodes hold sprites only in the CULL × CULL-tile chunks in view plus a margin (a seed-0 map has ~1,300
#   nodes), re-synced when the view crosses a chunk.
# Bridge methods: via SimReads get_colony_versions, get_colony_overlay; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const SiteGhost = preload("res://scripts/world/art/site_ghost.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const RoomLayer = preload("res://scripts/world/colony/room_layer.gd")
const FONT_PATH: String = "res://assets/fonts/galmuri/Galmuri11.ttf"
const BACK: Color = Color(0.094, 0.082, 0.075, 0.92)
const BACK_LINE: Color = Color(0.275, 0.243, 0.212, 1.0)
const TEXT: Color = Color(0.96, 0.92, 0.84)
const BAR_LOGS: Color = Color(0.85, 0.62, 0.3, 1.0)
const BAR_WORK: Color = Color(0.55, 0.85, 0.45, 1.0)
const TILE: int = View.TILE
const BAR_HEIGHT: int = 6
const SPACE_BAR: int = 4
## Designation kind → glyph (drawing behaviour, not content): 0 chop, 1 quarry, 2 dig, 3 deconstruct, 4 clay pit.
const MARK_GLYPHS: Array[String] = ["axe", "pick", "spade", "cross", "spade"]
## Culling chunk size in tiles (items and nodes).
const CULL: int = 16

var _world_sim: Node = null
var _world: Node2D = null
var _version: int = -1
var _params_version: int = -1
var _overlay: Dictionary = {}
var _font: Font = null
var _ground: Node2D = null
var _marks: Node2D = null
var _rooms: Node2D = null
## Tile -> {kind, count, quality, covered} of the item stack on it.
var _stacks: Dictionary = {}
## Tile -> {kind, left} of the resource node on it.
var _nodes: Dictionary = {}
## Tile -> the most items seen in that node (its "full" for the depleted look).
var _node_max: Dictionary = {}
var _hover: Vector2i = Vector2i(-1, -1)
var _zoom: float = 1.0
## Tiles of M3 space sites (their label shows under the mouse).
var _space_sites: Dictionary = {}
## Pools: key → Sprite2D (items "i", nodes "n", sites "s").
var _sprites: Dictionary = {}
var _sites_key: int = 0
## Floor-site ghosts: [world top-left, texture].
var _floor_ghosts: Array[Array] = []
var _stack_tex: Dictionary = {}
var _window: Rect2i = Rect2i()


func _ready() -> void:
	z_as_relative = false
	z_index = 6
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_font = load(FONT_PATH) as Font
	_world = get_node_or_null("../World")
	_ground = Node2D.new()
	_ground.name = "Ground"
	_ground.z_as_relative = false
	_ground.z_index = -6
	_ground.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_ground.draw.connect(_draw_ground)
	add_child(_ground)
	_marks = Node2D.new()
	_marks.name = "Marks"
	_marks.z_as_relative = false
	_marks.z_index = 3
	_marks.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_marks.draw.connect(_draw_marks)
	add_child(_marks)
	_rooms = RoomLayer.new()
	_rooms.name = "Rooms"
	add_child(_rooms)
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	_version = -1
	_node_max.clear()
	for s: Node in _sprites.values():
		s.queue_free()
	_sprites.clear()
	_sites_key = 0


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	if _params_version != ArtData.params_version():
		_params_version = ArtData.params_version()
		_version = -1
		_sites_key = 0
		_stack_tex.clear()
		for s: Node in _sprites.values():
			s.queue_free()
		_sprites.clear()
	var versions: Dictionary = SimReads.read(_world_sim, "get_colony_versions")
	var version: int = int(versions.get("overlay_version", -1))
	if version != _version:
		_version = version
		_overlay = SimReads.read(_world_sim, "get_colony_overlay")
		_index_stacks()
		_sync_things()
		_ground.queue_redraw()
		_marks.queue_redraw()
		queue_redraw()
	elif _update_window():
		_sync_things()
	var zoom: float = get_viewport().get_canvas_transform().x.x
	if not is_equal_approx(zoom, _zoom):
		_zoom = zoom
		queue_redraw()
	var hover: Vector2i = View.world_to_tile(get_global_mouse_position())
	if hover != _hover:
		var redraw: bool = _stacks.has(hover) or _stacks.has(_hover) or _nodes.has(hover) or _nodes.has(_hover) \
			or _space_sites.has(hover) or _space_sites.has(_hover)
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
	var covered: PackedInt32Array = _overlay.get("item_covered", PackedInt32Array())
	for i: int in mini(ixs.size(), mini(iys.size(), counts.size())):
		var kind: int = kinds[i] if i < kinds.size() else 0
		var quality: int = qualities[i] if i < qualities.size() else 50
		var under_roof: bool = i < covered.size() and covered[i] != 0
		_stacks[Vector2i(ixs[i], iys[i])] = {"kind": kind, "count": counts[i], "quality": quality, "covered": under_roof}
	_space_sites.clear()
	for site: Dictionary in _sites():
		if EconomyText.is_space(int(site["kind"])):
			_space_sites[site["tile"]] = true
	_nodes.clear()
	var nxs: PackedInt32Array = _overlay.get("node_xs", PackedInt32Array())
	var nys: PackedInt32Array = _overlay.get("node_ys", PackedInt32Array())
	var nkinds: PackedInt32Array = _overlay.get("node_kinds", PackedInt32Array())
	var left: PackedInt32Array = _overlay.get("node_left", PackedInt32Array())
	for i: int in mini(mini(nxs.size(), nys.size()), mini(nkinds.size(), left.size())):
		var t: Vector2i = Vector2i(nxs[i], nys[i])
		_nodes[t] = {"kind": nkinds[i], "left": left[i]}
		_node_max[t] = maxi(int(_node_max.get(t, 0)), left[i])


## The count bucket of `count` (0: 1, 1: 2–8, 2: 9+ by bindings items.bucket_limits).
static func bucket(count: int) -> int:
	var limits: Array = (ArtData.bindings().get("items", {}) as Dictionary).get("bucket_limits", [1, 8])
	var b: int = 0
	for lim: Variant in limits:
		if count > int(lim):
			b += 1
	return b


## [manifest id, copies] of an item stack of `kind` and `count` (bindings `items`).
static func stack_look(kind: int, count: int) -> Array:
	var row: Dictionary = ArtData.binding("items", kind)
	var b: int = bucket(count)
	var assets: Variant = row.get("assets", null)
	if assets is Array and not (assets as Array).is_empty():
		return [str(assets[mini(b, (assets as Array).size() - 1)]), 1]
	if b >= 2 and row.get("many", null) is String:
		return [str(row["many"]), 1]
	return [str(row.get("asset", "")), [1, 3, 9][mini(b, 2)]]


## The texture and anchor of `copies` copies of item `id` at the stack layout offsets (cached).
func _stack_texture(id: String, copies: int) -> Array:
	var key: String = "%s|%d" % [id, copies]
	if _stack_tex.has(key):
		return _stack_tex[key]
	var out: Array = [null, Vector2i.ZERO]
	if copies <= 1:
		out = [ArtData.texture(id), ArtData.anchor(id)]
	else:
		var src: Image = ArtData.image(id)
		var layout: Array = (ArtData.bindings().get("items", {}) as Dictionary).get("stack_layout", [[0, 0]])
		if src != null:
			var offs: Array[Vector2i] = []
			for k: int in mini(copies, layout.size()):
				offs.append(Vector2i(int(layout[k][0]), int(layout[k][1])))
			offs.sort_custom(func(a: Vector2i, b: Vector2i) -> bool: return a.y < b.y or (a.y == b.y and a.x < b.x))
			var lo: Vector2i = Vector2i.ZERO
			var hi: Vector2i = Vector2i.ZERO
			for o: Vector2i in offs:
				lo = Vector2i(mini(lo.x, o.x), mini(lo.y, o.y))
				hi = Vector2i(maxi(hi.x, o.x), maxi(hi.y, o.y))
			var img: Image = Image.create(src.get_width() + hi.x - lo.x, src.get_height() + hi.y - lo.y, false, Image.FORMAT_RGBA8)
			for o: Vector2i in offs:
				img.blend_rect(src, Rect2i(Vector2i.ZERO, src.get_size()), o - lo)
			out = [ImageTexture.create_from_image(img), ArtData.anchor(id) - lo]
	_stack_tex[key] = out
	return out


## The dead-frame id for an item of `kind` lying outside every stockpile zone ("" when the row names no carcass).
static func carcass_id(kind: int) -> String:
	var species: Variant = ArtData.binding("items", kind).get("carcass_of", null)
	if not (species is String):
		return ""
	var facings: Array = (ArtData.manifest().get("animals", {}) as Dictionary).get("facings", [])
	for r: Variant in (ArtData.bindings().get("animals", {}) as Dictionary).get("codes", []):
		if r is Dictionary and str((r as Dictionary).get("species", "")) == str(species) and not facings.is_empty():
			return "%s/%s/%s" % [str(species), str(facings[0]), str((r as Dictionary).get("dead", ""))]
	return ""


## The chunk window in view plus one chunk (items and nodes outside hold no sprite); true when it moved.
func _update_window() -> bool:
	var size: float = float(CULL * TILE)
	var view: Rect2 = get_viewport().get_canvas_transform().affine_inverse() * get_viewport_rect()
	var c0: Vector2i = Vector2i(floori(view.position.x / size) - 1, floori(view.position.y / size) - 1)
	var c1: Vector2i = Vector2i(floori(view.end.x / size) + 1, floori(view.end.y / size) + 1)
	var window: Rect2i = Rect2i(c0, c1 - c0 + Vector2i.ONE)
	if window == _window:
		return false
	_window = window
	return true


func _in_window(t: Vector2i) -> bool:
	return _window.has_point(Vector2i(floori(float(t.x) / CULL), floori(float(t.y) / CULL)))


## Items, nodes and sites as standing sprites, synced by key with the pool.
func _sync_things() -> void:
	if _world == null:
		return
	_update_window()
	var zones: Dictionary = {}
	var zxs: PackedInt32Array = _overlay.get("zone_xs", PackedInt32Array())
	var zys: PackedInt32Array = _overlay.get("zone_ys", PackedInt32Array())
	for i: int in mini(zxs.size(), zys.size()):
		zones[Vector2i(zxs[i], zys[i])] = true
	# key → [texture, anchor, base point]
	var wanted: Dictionary = {}
	for t: Vector2i in _stacks.keys():
		if not _in_window(t):
			continue
		var stack: Dictionary = _stacks[t]
		var kind: int = int(stack["kind"])
		var dead: String = carcass_id(kind) if not zones.has(t) else ""
		if dead != "" and ArtData.has(dead):
			wanted["i%d_%d" % [t.x, t.y]] = [ArtData.texture(dead), ArtData.anchor(dead), View.base_point(t) - Vector2(0, 1), dead]
			continue
		var look: Array = stack_look(kind, int(stack["count"]))
		var tex: Array = _stack_texture(str(look[0]), int(look[1]))
		if tex[0] != null:
			wanted["i%d_%d" % [t.x, t.y]] = [tex[0], tex[1], View.base_point(t) - Vector2(0, 1), "%s|%d" % look]
	for t: Vector2i in _nodes.keys():
		if not _in_window(t):
			continue
		var node: Dictionary = _nodes[t]
		var row: Dictionary = ArtData.binding("nodes", int(node["kind"]))
		var id: String = str(row.get("structure", ""))
		var full: int = maxi(1, int(_node_max.get(t, 1)))
		if row.get("depleted", null) is String and floori(int(node["left"]) * 1000.0 / full) < int(row.get("depleted_below_permille", 0)):
			id = str(row["depleted"])
		if ArtData.has(id):
			wanted["n%d_%d" % [t.x, t.y]] = [ArtData.texture(id), ArtData.anchor(id), View.base_point(t), id]
	_sync(wanted, ["i", "n"])
	var sites: Array[Dictionary] = _sites()
	var key: int = hash([_overlay.get("site_xs"), _overlay.get("site_ys"), _overlay.get("site_kinds")])
	if key != _sites_key:
		_sites_key = key
		_build_site_ghosts(sites)


## Keep the pooled sprites of the key prefixes `prefixes` equal to `wanted` (key → [texture, anchor, base, ident]).
func _sync(wanted: Dictionary, prefixes: Array) -> void:
	for k: String in _sprites.keys():
		if not prefixes.has(k.left(1)):
			continue
		var w: Variant = wanted.get(k, null)
		var s: Sprite2D = _sprites[k]
		if w == null or str(s.get_meta("ident", "")) != str(w[3]):
			s.queue_free()
			_sprites.erase(k)
	for k: String in wanted.keys():
		if _sprites.has(k):
			continue
		var w: Array = wanted[k]
		_sprites[k] = _standing(w[0], w[1], w[2], str(w[3]))


func _standing(tex: Texture2D, anchor: Vector2i, base: Vector2, ident: String) -> Sprite2D:
	var s: Sprite2D = Sprite2D.new()
	s.texture = tex
	s.centered = false
	s.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	s.position = base.round()
	s.offset = -Vector2(anchor)
	s.set_meta("ident", ident)
	_world.add_child(s)
	return s


## Blueprint sites as ghosts of their finished look (rebuilt when the set of sites changes).
func _build_site_ghosts(sites: Array[Dictionary]) -> void:
	for k: String in _sprites.keys():
		if k.begins_with("s"):
			(_sprites[k] as Node).queue_free()
			_sprites.erase(k)
	_floor_ghosts.clear()
	var walls: Dictionary = {}
	var doors: Dictionary = {}
	for site: Dictionary in sites:
		var code: int = int(site["kind"])
		var row: Dictionary = SiteGhost.blueprint(code)
		var tile: Vector2i = site["tile"]
		var size: Vector2i = site["size"]
		match str(row.get("kind", "")):
			"wall":
				walls[tile] = str(row.get("style", ""))
				continue
			"door":
				doors[tile] = str(row.get("style", ""))
			"floor":
				var fl: Image = SiteGhost.floor_tile(str(row.get("style", "")), tile)
				if fl != null:
					_floor_ghosts.append([View.tile_to_world(tile), SiteGhost.ghost("", fl)])
				continue
		var look: Dictionary = SiteGhost.finished(code)
		if look.is_empty():
			continue
		var fp: Vector2i = look.get("footprint", size) if bool(row.get("bed", false)) else size
		var base: Vector2 = SiteGhost.footprint_base(tile, fp)
		if bool(look.get("flat", false)):
			base.y -= 1
		_sprites["s%d_%d" % [tile.x, tile.y]] = _standing(SiteGhost.ghost("site:%d" % code, look["image"]), look["base"], base, "site")
	if not walls.is_empty():
		var rows: Dictionary = SiteGhost.wall_rows(walls, doors)
		for t: Vector2i in walls.keys():
			var img: Image = SiteGhost.wall_tile(rows, t)
			if img == null:
				continue
			var s: Sprite2D = _standing(SiteGhost.ghost("", img), Vector2i(0, SiteGhost.CAP + View.BASE.y), Vector2(t.x * TILE, t.y * TILE + View.BASE.y), "site")
			_sprites["s%d_%d" % [t.x, t.y]] = s


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
			"tile": Vector2i(sxs[i], sys[i]), "size": Vector2i(sws[i], shs[i]),
			"rect": Rect2(View.tile_to_world(Vector2i(sxs[i], sys[i])), Vector2(sws[i], shs[i]) * TILE),
			"needed": needed[i], "delivered": delivered[i], "progress": progress[i], "required": required[i],
			"kind": kinds[i] if i < kinds.size() else 0,
		})
	return out


func _colour(path: String, fallback: Array) -> Color:
	var v: Variant = ArtData.param(path, fallback)
	var a: Array = v if v is Array and ((v as Array).size() == 3 or (v as Array).size() == 4) else fallback
	return Color8(int(a[0]), int(a[1]), int(a[2]), int(a[3]) if a.size() == 4 else 255)


## Zones: a faint tint and ticks on the outer corners; floor-site ghosts.
func _draw_ground() -> void:
	if _overlay.is_empty():
		return
	var zone_ids: PackedInt32Array = _overlay.get("zone_ids", PackedInt32Array())
	var zxs: PackedInt32Array = _overlay.get("zone_xs", PackedInt32Array())
	var zys: PackedInt32Array = _overlay.get("zone_ys", PackedInt32Array())
	var zone_at: Dictionary = {}
	for i: int in mini(zone_ids.size(), mini(zxs.size(), zys.size())):
		zone_at[Vector2i(zxs[i], zys[i])] = zone_ids[i]
	var tint: Color = _colour("zones.tint_rgb", [226, 204, 150])
	tint.a = float(ArtData.param("zones.tint_alpha", 0.08))
	var tick: Color = _colour("zones.tick_rgb", [236, 228, 204])
	tick.a = float(ArtData.param("zones.tick_alpha", 0.45))
	var tick_px: float = float(ArtData.param("zones.tick_px", 5))
	var w: float = 2.0
	for cell: Vector2i in zone_at.keys():
		var o: Vector2 = View.tile_to_world(cell)
		_ground.draw_rect(Rect2(o, Vector2(TILE, TILE)), tint)
		var id: int = zone_at[cell]
		var open_n: bool = zone_at.get(cell + Vector2i(0, -1), -1) != id
		var open_e: bool = zone_at.get(cell + Vector2i(1, 0), -1) != id
		var open_s: bool = zone_at.get(cell + Vector2i(0, 1), -1) != id
		var open_w: bool = zone_at.get(cell + Vector2i(-1, 0), -1) != id
		var t: float = float(TILE)
		if open_n and open_w:
			_ground.draw_rect(Rect2(o, Vector2(tick_px, w)), tick)
			_ground.draw_rect(Rect2(o, Vector2(w, tick_px)), tick)
		if open_n and open_e:
			_ground.draw_rect(Rect2(o + Vector2(t - tick_px, 0), Vector2(tick_px, w)), tick)
			_ground.draw_rect(Rect2(o + Vector2(t - w, 0), Vector2(w, tick_px)), tick)
		if open_s and open_w:
			_ground.draw_rect(Rect2(o + Vector2(0, t - w), Vector2(tick_px, w)), tick)
			_ground.draw_rect(Rect2(o + Vector2(0, t - tick_px), Vector2(w, tick_px)), tick)
		if open_s and open_e:
			_ground.draw_rect(Rect2(o + Vector2(t - tick_px, t - w), Vector2(tick_px, w)), tick)
			_ground.draw_rect(Rect2(o + Vector2(t - w, t - tick_px), Vector2(w, tick_px)), tick)
	for g: Array in _floor_ghosts:
		if g[1] != null:
			_ground.draw_texture(g[1], g[0])


## Designations as ink marks on a pale disc at the tile's top-right.
func _draw_marks() -> void:
	var dxs: PackedInt32Array = _overlay.get("designation_xs", PackedInt32Array())
	var dys: PackedInt32Array = _overlay.get("designation_ys", PackedInt32Array())
	var dkinds: PackedInt32Array = _overlay.get("designation_kinds", PackedInt32Array())
	var ink: Color = _colour("marks.ink_rgb", [35, 29, 23])
	var back: Color = _colour("marks.back_rgba", [236, 228, 204, 215])
	var size: float = float(ArtData.param("marks.size_px", 18))
	for i: int in mini(dxs.size(), dys.size()):
		var kind: int = clampi(dkinds[i], 0, MARK_GLYPHS.size() - 1) if i < dkinds.size() else 0
		var c: Vector2 = View.tile_to_world(Vector2i(dxs[i], dys[i])) + Vector2(TILE - size * 0.5 - 4, size * 0.5 + 4)
		draw_mark(_marks, MARK_GLYPHS[kind], c, size, ink, back)


## One ink mark `glyph` centred on `c` (`size` px) on a pale disc with an ink ring.
static func draw_mark(ci: CanvasItem, glyph: String, c: Vector2, size: float, ink: Color, back: Color) -> void:
	var r: float = size * 0.5
	ci.draw_circle(c, r + 1.0, ink)
	ci.draw_circle(c, r - 1.0, back)
	var s: float = r * 0.62
	match glyph:
		"axe":
			ci.draw_line(c + Vector2(-s, s), c + Vector2(s * 0.6, -s * 0.6), ink, 2.0)
			ci.draw_rect(Rect2(c + Vector2(s * 0.1, -s), Vector2(s * 0.9, s * 0.8)), ink)
		"pick":
			ci.draw_line(c + Vector2(-s * 0.7, s), c + Vector2(s * 0.5, -s * 0.5), ink, 2.0)
			ci.draw_line(c + Vector2(-s * 0.6, -s * 0.9), c + Vector2(s, s * 0.2), ink, 2.0)
		"spade":
			ci.draw_line(c + Vector2(0, -s), c + Vector2(0, s * 0.1), ink, 2.0)
			ci.draw_colored_polygon(PackedVector2Array([c + Vector2(-s * 0.55, 0), c + Vector2(s * 0.55, 0), c + Vector2(0, s)]), ink)
		_:
			ci.draw_line(c + Vector2(-s, -s), c + Vector2(s, s), ink, 2.0)
			ci.draw_line(c + Vector2(-s, s), c + Vector2(s, -s), ink, 2.0)


func _draw() -> void:
	if _overlay.is_empty():
		return
	var locale: Node = get_node_or_null("/root/Locale")
	for site: Dictionary in _sites():
		var rect: Rect2 = site["rect"]
		var needed: int = int(site["needed"])
		var delivered: int = int(site["delivered"])
		var logs_done: float = clampf(float(delivered) / maxf(1.0, float(needed)), 0.0, 1.0)
		var work_done: float = clampf(float(site["progress"]) / maxf(1.0, float(site["required"])), 0.0, 1.0)
		var building: bool = delivered >= needed
		var fill: float = work_done if building else logs_done
		var space: bool = EconomyText.is_space(int(site["kind"]))
		if space:
			# A thin bar inside the tile's bottom once materials arrived (a fresh drag of 30 walls shows none); the
			# label on hover only.
			var hovered: bool = rect.has_point(View.tile_centre(_hover))
			if delivered > 0 or hovered:
				var inner: Rect2 = Rect2(rect.position + Vector2(8, rect.size.y - 8), Vector2(rect.size.x - TILE * 0.25, SPACE_BAR))
				draw_rect(inner, BACK)
				draw_rect(Rect2(inner.position + Vector2(1, 1), Vector2((inner.size.x - 2) * fill, SPACE_BAR - 2)), BAR_WORK if building else BAR_LOGS)
			if not hovered:
				continue
		else:
			var bar: Rect2 = Rect2(rect.position + Vector2(4, rect.size.y + 2), Vector2(rect.size.x - 8, BAR_HEIGHT))
			draw_rect(bar, BACK)
			draw_rect(Rect2(bar.position + Vector2(1, 1), Vector2((bar.size.x - 2) * fill, BAR_HEIGHT - 2)), BAR_WORK if building else BAR_LOGS)
		if _font == null:
			continue
		var params: Dictionary = {"delivered": delivered, "needed": needed}
		var needs_key: String = "UI_BLUEPRINT_LOGS" if int(site["kind"]) == 0 else "UI_BLUEPRINT_MATERIALS"
		var logs_text: String = str(locale.call("trf", needs_key, params)) if locale != null else "%d/%d" % [delivered, needed]
		var percent: int = floori(work_done * 100.0)
		var text: String = str(locale.call("trf", "UI_BLUEPRINT_PROGRESS_FMT", {"logs": logs_text, "percent": percent})) if locale != null else "%s %d%%" % [logs_text, percent]
		draw_label(self, _font, text, Vector2(rect.position.x + rect.size.x * 0.5, rect.position.y - 4), _zoom)
	# The hovered stack's name, count and quality; else the hovered node.
	var at: Vector2 = View.tile_to_world(_hover) + Vector2(TILE * 0.5, -2)
	if _stacks.has(_hover) and _font != null:
		var stack: Dictionary = _stacks[_hover]
		var kind: int = int(stack["kind"])
		var text: String = str(int(stack["count"]))
		if kind > 0:
			text = EconomyText.stack(locale, kind, "", int(stack["count"]), int(stack["quality"]))
		text += room_suffix(_rooms, _hover, bool(stack.get("covered", false)), locale)
		draw_label(self, _font, text, at, _zoom)
	elif _nodes.has(_hover) and _font != null:
		var node: Dictionary = _nodes[_hover]
		var row: Dictionary = ArtData.binding("nodes", int(node["kind"]))
		draw_label(self, _font, EconomyText.trf(locale, "UI_NODE_LEFT_FMT", {"name": EconomyText.ltr(locale, str(row.get("name_key", "UI_UNKNOWN"))),
			"item": EconomyText.item(locale, int(row.get("yields", -1))), "left": int(node["left"])}), at, _zoom)


## M3: " · 방: 창고 2" for a tile inside a room of `rooms` (room_layer.gd), plus " · 지붕 아래" when `covered`; "" outside.
static func room_suffix(rooms: Node, tile: Vector2i, covered: bool, locale: Node) -> String:
	var text: String = ""
	var id: int = int(rooms.call("room_at", tile)) if rooms != null else -1
	if id >= 0:
		var info: Dictionary = rooms.call("room", id)
		text += " · " + EconomyText.trf(locale, "UI_INSPECT_ROOM_FMT", {"role": EconomyText.role(locale, int(info.get("role", 0))), "id": id})
	if covered:
		text += " · " + EconomyText.ltr(locale, "UI_ITEM_COVERED")
	return text


## The room overlay node (room_layer.gd; for the order tool, the order bar, the inspector and tools).
func rooms() -> Node2D:
	return _rooms


## Text centred on `anchor.x` with its box's bottom at `anchor.y` (world), on a dark backing box; the text is
## `labels.font_px` screen pixels at camera zoom `zoom` (the box is scaled about the anchor by 1 / zoom).
static func draw_label(ci: CanvasItem, font: Font, text: String, anchor: Vector2, zoom: float) -> void:
	var px: int = int(ArtData.param("labels.font_px", 24))
	var follow: bool = bool(ArtData.param("labels.follow_camera", true))
	var inv: float = 1.0 / maxf(0.05, zoom) if follow else 1.0
	var size: Vector2 = font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, px)
	var pad: float = floorf(px / 4.0)
	ci.draw_set_transform(anchor.round(), 0.0, Vector2(inv, inv))
	var box: Rect2 = Rect2(Vector2(-size.x * 0.5 - pad, -px - pad), Vector2(size.x + pad * 2.0, px + pad)).abs()
	box.position = box.position.round()
	ci.draw_rect(box, BACK)
	ci.draw_rect(box, BACK_LINE, false, 2.0)
	ci.draw_string(font, box.position + Vector2(pad, px - 2.0), text, HORIZONTAL_ALIGNMENT_LEFT, -1, px, TEXT)
	ci.draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
