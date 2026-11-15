# labels3d.gd (World3D/LabelLayer/Labels, made by world3d.gd): the map's text, bars and small marks, drawn in screen
#   space over the 3D view and under the HUD (M3D-SWITCH E2: ported from the deleted 2D OverlayLayer, RoomLayer,
#   AgentLayer, AnimalLayer, VisitorLayer, LivestockLayer and OutcropLayer, with their texts, keys and hover rules).
# Purpose: every anchor is a colony point (tiles; ground height from the terrain, plus a lift) projected through the
#   World3D camera; a label whose anchor is behind the camera or off screen is skipped. Text is `labels.font_px` screen
#   pixels (Galmuri11) on a dark backing box (`draw_label`), so it reads the same at every zoom.
#   - Blueprint sites (get_colony_overlay `site_*`, `site_rows`): a bar under the footprint (amber with the materials
#     delivered until all are there, then green with the work done) and above it "통나무 d/n · 작업 n%"
#     (UI_BLUEPRINT_LOGS / UI_BLUEPRINT_MATERIALS in UI_BLUEPRINT_PROGRESS_FMT); a space site (wall, floor, door,
#     bedroll) only a thin bar inside its tile once materials arrived or while hovered, and its label only while hovered.
#   - Under the mouse (the hovered tile, ViewProjection.screen_to_world): an item stack's "부싯돌 날 ×3 · 품질 62"
#     (+ `room_suffix` " · 방: 창고 2", " · 지붕 아래"), else a resource node's "바위 · 돌 12개 남음" (UI_NODE_LEFT_FMT), each
#     plus " · 주거 구역" on a housing designation (ZONE_HOUSING), else that word alone; a pen animal's
#     C1_LIVESTOCK_HOVER (the pen animal whose body is under the mouse, else on the hovered tile); a wild animal's "사슴 · 돌아다님 · 30" (M5_ANIMAL_HOVER_FMT + C1_WILD_AGE; the animal picked
#     under the mouse); a visitor group's "이주민 4명 · 붉은바위" / "교역 무리 · 붉은바위" (M55_HOVER_*_FMT).
#   - The room overlay shown (R, Main/RoomIndex): each room's label at its centre tile (RoomIndex.label_text), framed in
#     its role colour.
#   - Ore outcrops (get_outcrops): the catalogue glyph above each, its name while hovered.
#   - People (World3D/People bodies, get_agent_snapshot): a drafted person's ring above the head (render_params
#     `people.drafted_rgba`, `people.drafted_outline_px` wide), a sick person's green badge and a fleeing person's "!"
#     above the head; with the mood overlay (M, Main/PeopleView) a disc in the mood band's tint (StoryText.band_tint);
#     below 1× zoom, where people share a spot, their count on a label above them.
#   - Wild animals that may be hunted (get_animals `huntable`): a yellow ring over the head, full for its own mark,
#     dimmer when only a hunt zone covers it.
#   `drawn()` lists the labels and marks of the last frame ({kind, text, at}; people's and hunt marks also name whom)
#   for tests; `follow_mouse = false` lets a tool set
#   `hover_tile` and `hover_screen` itself.
# Bridge methods called: via SimReads get_colony_versions, get_colony_overlay, get_agent_snapshot, get_animals,
#   get_livestock, get_outcrops, get_neighbours (the per-tick reads again after any command, SimReads
#   snapshots_invalidated); signal world_reset.
extends Control

const ArtData = preload("res://scripts/world/art/art_data.gd")
const Data3D = preload("res://scripts/world3d/data3d.gd")
const EconomyText = preload("res://scripts/ui/economy_text.gd")
const StoryText = preload("res://scripts/ui/story_text.gd")
const NeighbourText = preload("res://scripts/ui/neighbour_text.gd")
const Content = preload("res://scripts/ui/content_context.gd")
const RoomIndex = preload("res://scripts/world/room_index.gd")
const ViewProjection = preload("res://scripts/world/view_projection.gd")
const View = preload("res://scripts/world/colony/view.gd")
const OrderTool = preload("res://scripts/world/colony/order_tool.gd")
const FONT_PATH: String = "res://assets/fonts/galmuri/Galmuri11.ttf"
const BACK: Color = Color(0.094, 0.082, 0.075, 0.92)
const BACK_LINE: Color = Color(0.275, 0.243, 0.212, 1.0)
const TEXT: Color = Color(0.96, 0.92, 0.84)
const BAR_LOGS: Color = Color(0.85, 0.62, 0.3, 1.0)
const BAR_WORK: Color = Color(0.55, 0.85, 0.45, 1.0)
const INK: Color = Color8(35, 29, 23)
const BADGE: Color = Color(0.45, 0.78, 0.36)
const FLEE_MARK: Color = Color(0.94, 0.75, 0.25)
const HUNT_MARK: Color = Color(0.98, 0.84, 0.3)
const ZONE_MARK_ALPHA: float = 0.55
## Bar heights in screen pixels (a footprint's bar; a space site's thin bar).
const BAR_HEIGHT: int = 6
const SPACE_BAR: int = 4
## How far above the ground a person's or an animal's marks float over its head (tiles), and an outcrop's glyph.
const HEAD_GAP: float = 0.25
const OUTCROP_LIFT: float = 0.6

## The hovered tile and screen point (from the mouse unless `follow_mouse` is off).
var follow_mouse: bool = true
var hover_tile: Vector2i = Vector2i(-1, -1)
var hover_screen: Vector2 = Vector2(-1, -1)
var _world3d: Node = null
var _world_sim: Node = null
var _font: Font = null
var _version: int = -1
var _overlay: Dictionary = {}
var _sites: Array[Dictionary] = []
var _stacks: Dictionary = {}
var _nodes: Dictionary = {}
var _housing: Dictionary = {}
var _tick: int = -1
var _animals: Dictionary = {}
var _livestock: Dictionary = {}
var _outcrops: Dictionary = {}
var _people: Dictionary = {}
var _drawn: Array[Dictionary] = []


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	_font = load(FONT_PATH) as Font
	_world3d = get_parent().get_parent()
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", func() -> void:
			_version = -1
			_tick = -1)
		# A command while paused (draft, hunt mark) changes the per-tick reads without a tick: read them again.
		var reader: Node = SimReads.resolve(_world_sim)
		if reader != null:
			reader.connect("snapshots_invalidated", func() -> void: _tick = -1)


func _process(_delta: float) -> void:
	if _world_sim == null:
		return
	var version: int = int(SimReads.read(_world_sim, "get_colony_versions").get("overlay_version", -1))
	if version != _version:
		_version = version
		_index_overlay(SimReads.read(_world_sim, "get_colony_overlay"))
	var tick: int = int(SimReads.read(_world_sim, "get_time_snapshot").get("tick", -1))
	if tick != _tick:
		_tick = tick
		_animals = SimReads.read(_world_sim, "get_animals")
		_livestock = SimReads.read(_world_sim, "get_livestock")
		_outcrops = SimReads.read(_world_sim, "get_outcrops")
		_people = SimReads.read(_world_sim, "get_agent_snapshot")
	if follow_mouse:
		hover_screen = get_viewport().get_mouse_position()
		hover_tile = View.world_to_tile(ViewProjection.screen_to_world(hover_screen))
	queue_redraw()


## The labels and marks drawn in the last frame: {kind, text, at (screen point of the anchor)[, bits (marks over a
## person: drafted, sick, flee, mood with its `tint`)][, id and `marked` (hunt rings over animals)]}.
func drawn() -> Array[Dictionary]:
	return _drawn


func _index_overlay(overlay: Dictionary) -> void:
	_overlay = overlay
	_sites = site_rows(overlay)
	_stacks.clear()
	var ixs: PackedInt32Array = overlay.get("item_xs", PackedInt32Array())
	var iys: PackedInt32Array = overlay.get("item_ys", PackedInt32Array())
	var counts: PackedInt32Array = overlay.get("item_counts", PackedInt32Array())
	var kinds: PackedInt64Array = overlay.get("item_kinds", PackedInt64Array())
	var qualities: PackedInt32Array = overlay.get("item_qualities", PackedInt32Array())
	var covered: PackedInt32Array = overlay.get("item_covered", PackedInt32Array())
	for i: int in mini(ixs.size(), mini(iys.size(), counts.size())):
		_stacks[Vector2i(ixs[i], iys[i])] = {"kind": kinds[i] if i < kinds.size() else 0, "count": counts[i],
			"quality": qualities[i] if i < qualities.size() else 50, "covered": i < covered.size() and covered[i] != 0}
	_nodes.clear()
	var nxs: PackedInt32Array = overlay.get("node_xs", PackedInt32Array())
	var nys: PackedInt32Array = overlay.get("node_ys", PackedInt32Array())
	var nkinds: PackedInt32Array = overlay.get("node_kinds", PackedInt32Array())
	var left: PackedInt32Array = overlay.get("node_left", PackedInt32Array())
	for i: int in mini(mini(nxs.size(), nys.size()), mini(nkinds.size(), left.size())):
		_nodes[Vector2i(nxs[i], nys[i])] = {"kind": nkinds[i], "left": left[i]}
	_housing.clear()
	var hids: PackedInt64Array = overlay.get("housing_zone_ids", PackedInt64Array())
	var hxs: PackedInt32Array = overlay.get("housing_zone_xs", PackedInt32Array())
	var hys: PackedInt32Array = overlay.get("housing_zone_ys", PackedInt32Array())
	for i: int in mini(hids.size(), mini(hxs.size(), hys.size())):
		if hids[i] >= 0:
			_housing[Vector2i(hxs[i], hys[i])] = hids[i]


## The blueprint sites of a get_colony_overlay read: {tile, size, needed, delivered, progress, required, kind}.
static func site_rows(overlay: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var sxs: PackedInt32Array = overlay.get("site_xs", PackedInt32Array())
	var sys: PackedInt32Array = overlay.get("site_ys", PackedInt32Array())
	var sws: PackedInt32Array = overlay.get("site_ws", PackedInt32Array())
	var shs: PackedInt32Array = overlay.get("site_hs", PackedInt32Array())
	var needed: PackedInt32Array = overlay.get("site_needed", PackedInt32Array())
	var delivered: PackedInt32Array = overlay.get("site_delivered", PackedInt32Array())
	var progress: PackedInt32Array = overlay.get("site_progress", PackedInt32Array())
	var required: PackedInt32Array = overlay.get("site_required", PackedInt32Array())
	var kinds: PackedInt64Array = overlay.get("site_kinds", PackedInt64Array())
	var n: int = mini(mini(mini(sxs.size(), sys.size()), mini(sws.size(), shs.size())),
		mini(mini(needed.size(), delivered.size()), mini(progress.size(), required.size())))
	for i: int in n:
		out.append({"tile": Vector2i(sxs[i], sys[i]), "size": Vector2i(sws[i], shs[i]), "needed": needed[i],
			"delivered": delivered[i], "progress": progress[i], "required": required[i], "kind": kinds[i] if i < kinds.size() else 0})
	return out


## A site's progress label: "통나무 3/12 · 작업 40%".
static func site_text(site: Dictionary, locale: Node) -> String:
	var needed: int = int(site["needed"])
	var delivered: int = int(site["delivered"])
	var params: Dictionary = {"delivered": delivered, "needed": needed}
	var needs_key: String = "UI_BLUEPRINT_LOGS" if int(site["kind"]) == 0 else "UI_BLUEPRINT_MATERIALS"
	var logs_text: String = str(locale.call("trf", needs_key, params)) if locale != null else "%d/%d" % [delivered, needed]
	var percent: int = floori(site_work(site) * 100.0)
	return str(locale.call("trf", "UI_BLUEPRINT_PROGRESS_FMT", {"logs": logs_text, "percent": percent})) if locale != null else "%s %d%%" % [logs_text, percent]


static func site_work(site: Dictionary) -> float:
	return clampf(float(site["progress"]) / maxf(1.0, float(site["required"])), 0.0, 1.0)


## The bar's fill (materials until all are delivered, then the work) and whether the work has started.
static func site_fill(site: Dictionary) -> Array:
	var building: bool = int(site["delivered"]) >= int(site["needed"])
	var logs_done: float = clampf(float(site["delivered"]) / maxf(1.0, float(site["needed"])), 0.0, 1.0)
	return [site_work(site) if building else logs_done, building]


## The hovered tile's label from the indexed overlay ({} stacks, nodes, housing): a stack (+ room and roof suffix), else
## a node, each with " · 주거 구역" on a housing designation, else that word alone; "" when nothing is there.
static func tile_text(tile: Vector2i, stacks: Dictionary, nodes: Dictionary, housing: Dictionary, rooms: Node, locale: Node) -> String:
	var housing_label: String = EconomyText.ltr(locale, "ZONE_HOUSING") if housing.has(tile) else ""
	if stacks.has(tile):
		var stack: Dictionary = stacks[tile]
		var kind: int = int(stack["kind"])
		var text: String = str(int(stack["count"]))
		if kind > 0:
			text = EconomyText.stack(locale, kind, "", int(stack["count"]), int(stack["quality"]))
		text += room_suffix(rooms, tile, bool(stack.get("covered", false)), locale)
		return text + (" · " + housing_label if not housing_label.is_empty() else "")
	if nodes.has(tile):
		var node: Dictionary = nodes[tile]
		var row: Dictionary = ArtData.binding("nodes", int(node["kind"]))
		return EconomyText.trf(locale, "UI_NODE_LEFT_FMT", {"name": EconomyText.ltr(locale, str(row.get("name_key", "UI_UNKNOWN"))),
			"item": EconomyText.item(locale, int(row.get("yields", -1))), "left": int(node["left"])}) + (" · " + housing_label if not housing_label.is_empty() else "")
	return housing_label


## M3: " · 방: 창고 2" for a tile inside a room of `rooms` (Main/RoomIndex), plus " · 지붕 아래" when `covered`; "" outside.
static func room_suffix(rooms: Node, tile: Vector2i, covered: bool, locale: Node) -> String:
	var text: String = ""
	var id: int = int(rooms.call("room_at", tile)) if rooms != null else -1
	if id >= 0:
		var info: Dictionary = rooms.call("room", id)
		text += " · " + EconomyText.trf(locale, "UI_INSPECT_ROOM_FMT", {"role": EconomyText.role(locale, int(info.get("role", 0))), "id": id})
	if covered:
		text += " · " + EconomyText.ltr(locale, "UI_ITEM_COVERED")
	return text


## A wild animal's hover text: "사슴 · 돌아다님 · 30" (+ " · 3일, 암컷") from row `i` of get_animals `read`.
static func animal_text(read: Dictionary, i: int, locale: Node) -> String:
	var kinds: PackedInt64Array = read.get("kinds", PackedInt64Array())
	var states: PackedInt32Array = read.get("states", PackedInt32Array())
	var hps: PackedInt32Array = read.get("hps", PackedInt32Array())
	var text: String = StoryText.trf(locale, "M5_ANIMAL_HOVER_FMT", {"animal": StoryText.animal(locale, kinds[i] if i < kinds.size() else -1),
		"state": StoryText.animal_state(locale, states[i] if i < states.size() else -1), "hp": hps[i] if i < hps.size() else 0})
	var ages: PackedInt64Array = read.get("age_days", PackedInt64Array())
	var sexes: PackedInt64Array = read.get("female", PackedInt64Array())
	if i < ages.size() and i < sexes.size() and ages[i] >= 0:
		text += " · " + StoryText.trf(locale, "C1_WILD_AGE", {"age": ages[i], "sex": StoryText.ltr(locale, "C1_SEX_FEMALE" if sexes[i] != 0 else "C1_SEX_MALE")})
	return text


## A pen animal's hover text (C1_LIVESTOCK_HOVER) from row `i` of get_livestock `read`; "" for an incomplete row.
static func livestock_text(read: Dictionary, i: int, locale: Node) -> String:
	var ids: PackedInt64Array = read.get("ids", PackedInt64Array())
	var kinds: PackedInt64Array = read.get("kinds", PackedInt64Array())
	var ages: PackedInt64Array = read.get("age_days", PackedInt64Array())
	var females: PackedInt64Array = read.get("female", PackedInt64Array())
	var hunger: PackedInt64Array = read.get("hunger_days", PackedInt64Array())
	if i >= mini(ids.size(), mini(kinds.size(), mini(ages.size(), mini(females.size(), hunger.size())))):
		return ""
	return StoryText.trf(locale, "C1_LIVESTOCK_HOVER", {"animal": StoryText.ltr(locale, str(Content.row("animals", kinds[i]).get("domestic_name_key", "C2_ANIMAL_LIVESTOCK_GENERIC"))),
		"id": ids[i], "age": ages[i], "sex": StoryText.ltr(locale, "C1_SEX_FEMALE" if females[i] != 0 else "C1_SEX_MALE"), "hunger": hunger[i]})


## A visitor group's hover text from a NeighbourText wave or caravan row.
static func visitor_text(kind: String, row: Dictionary, locale: Node) -> String:
	var who: String = NeighbourText.name_of(locale, int(row.get("neighbour", -1)))
	if kind == "wave":
		return NeighbourText.trf(locale, "M55_HOVER_WAVE_FMT", {"count": int(row.get("size", 0)), "neighbour": who})
	return NeighbourText.trf(locale, "M55_HOVER_CARAVAN_FMT", {"neighbour": who})


## The screen point of colony point `at` (tiles) lifted `lift` tiles above the ground, and whether it is in front of
## the camera.
func _project(at: Vector2, lift: float) -> Array:
	var cam: Camera3D = _world3d.call("camera") as Camera3D if _world3d != null else null
	var terrain: Node = _world3d.get("terrain") if _world3d != null else null
	if cam == null or terrain == null:
		return [Vector2.ZERO, false]
	var p: Vector3 = Vector3(at.x, float(terrain.call("height_at", at.x, at.y)) + lift, at.y)
	if cam.is_position_behind(p):
		return [Vector2.ZERO, false]
	return [cam.unproject_position(p), true]


func _draw() -> void:
	_drawn.clear()
	if _font == null or _world3d == null or (_overlay.is_empty() and _people.is_empty()):
		return
	var locale: Node = get_node_or_null("/root/Locale")
	var rooms: Node = get_node_or_null("/root/Main/RoomIndex")
	var view: Rect2 = get_viewport_rect()
	_draw_sites(locale, view)
	if rooms != null and bool(rooms.call("is_shown")):
		_draw_rooms(rooms, locale, view)
	_draw_outcrops(locale, view)
	_draw_people(view)
	_draw_hunt_marks(view)
	_draw_hover(rooms, locale, view)


func _draw_sites(locale: Node, view: Rect2) -> void:
	for site: Dictionary in _sites:
		var t: Vector2i = site["tile"]
		var size: Vector2i = site["size"]
		var space: bool = EconomyText.is_space(int(site["kind"]))
		var hovered: bool = Rect2i(t, size).has_point(hover_tile)
		var fill: Array = site_fill(site)
		# The bar runs along the footprint's front (south) edge; a space site's sits inside its tile.
		var inset: float = 0.125 if space else 0.0625
		var bl: Array = _project(Vector2(t.x + inset, t.y + size.y - (inset if space else 0.0)), 0.0)
		var br: Array = _project(Vector2(t.x + size.x - inset, t.y + size.y - (inset if space else 0.0)), 0.0)
		if bool(bl[1]) and bool(br[1]) and (not space or int(site["delivered"]) > 0 or hovered):
			var a: Vector2 = bl[0]
			var b: Vector2 = br[0]
			var h: int = SPACE_BAR if space else BAR_HEIGHT
			var bar: Rect2 = Rect2(Vector2(minf(a.x, b.x), maxf(a.y, b.y) + (-float(h) - 2.0 if space else 2.0)), Vector2(maxf(4.0, absf(b.x - a.x)), h))
			if view.intersects(bar):
				draw_rect(bar, BACK)
				draw_rect(Rect2(bar.position + Vector2(1, 1), Vector2((bar.size.x - 2) * float(fill[0]), h - 2)), BAR_WORK if bool(fill[1]) else BAR_LOGS)
		if space and not hovered:
			continue
		var top: Array = _project(Vector2(t.x + size.x * 0.5, t.y), 0.0)
		if bool(top[1]):
			_label(site_text(site, locale), (top[0] as Vector2) - Vector2(0, 4), view, "site")


func _draw_rooms(rooms: Node, locale: Node, view: Rect2) -> void:
	var all: Dictionary = rooms.call("rooms")
	var px: int = int(ArtData.param("labels.font_px", 24))
	var pad: float = floorf(px / 4.0)
	for id: int in all:
		var info: Dictionary = all[id]
		var centre: Vector2i = info["centre"]
		if centre.x < 0:
			continue
		var at: Array = _project(Vector2(centre) + Vector2(0.5, 0.5), 0.0)
		if not bool(at[1]) or not view.has_point(at[0]):
			continue
		var text: String = RoomIndex.label_text(info, locale)
		var size: Vector2 = _font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, px)
		var box: Rect2 = Rect2((at[0] as Vector2).round() - Vector2(size.x * 0.5 + pad, (px + pad) * 0.5), Vector2(size.x + pad * 2.0, px + pad))
		box.position = box.position.round()
		draw_rect(box, BACK)
		draw_rect(box, Color(RoomIndex.ROLE_COLORS[int(info["role"])], 1.0), false, 2.0)
		draw_string(_font, box.position + Vector2(pad, px - 2.0), text, HORIZONTAL_ALIGNMENT_LEFT, -1, px, TEXT)
		_drawn.append({"kind": "room", "text": text, "at": at[0]})


func _draw_outcrops(locale: Node, view: Rect2) -> void:
	var xs: PackedInt64Array = _outcrops.get("xs", PackedInt64Array())
	var ys: PackedInt64Array = _outcrops.get("ys", PackedInt64Array())
	var kinds: PackedInt64Array = _outcrops.get("kinds", PackedInt64Array())
	for i: int in mini(xs.size(), mini(ys.size(), kinds.size())):
		var tile: Vector2i = Vector2i(xs[i], ys[i])
		var at: Array = _project(Vector2(tile) + Vector2(0.5, 0.5), OUTCROP_LIFT)
		if not bool(at[1]):
			continue
		var definition: Dictionary = Content.row("outcrops", kinds[i])
		var text: String = StoryText.ltr(locale, str(definition.get("name_key", ""))) if tile == hover_tile else str(definition.get("glyph", ""))
		_label(text, at[0], view, "outcrop")


func _draw_people(view: Rect2) -> void:
	var people: Node = _world3d.get("people")
	var cam: Camera3D = _world3d.call("camera") as Camera3D
	if people == null or cam == null:
		return
	var ids: PackedInt64Array = _people.get("ids", PackedInt64Array())
	var sick: PackedByteArray = _people.get("sick", PackedByteArray())
	var fleeing: PackedByteArray = _people.get("fleeing", PackedByteArray())
	var states: PackedByteArray = _people.get("states", PackedByteArray())
	var targets: PackedByteArray = _people.get("target_kinds", PackedByteArray())
	var bands: PackedByteArray = _people.get("mood_bands", PackedByteArray())
	var drafted: PackedByteArray = _people.get("drafted", PackedByteArray())
	var drafted_v: Variant = ArtData.param("people.drafted_rgba", [196, 52, 40, 255])
	var drafted_rgba: Color = Color8(int(drafted_v[0]), int(drafted_v[1]), int(drafted_v[2]), int(drafted_v[3])) if drafted_v is Array and (drafted_v as Array).size() == 4 else Color(0.77, 0.2, 0.16)
	var drafted_px: float = float(ArtData.param("people.drafted_outline_px", 2))
	var people_view: Node = get_node_or_null("/root/Main/PeopleView")
	var mood: bool = people_view != null and bool(people_view.call("mood_shown"))
	var camera2d: Camera2D = get_node_or_null("/root/Main/Camera2D") as Camera2D
	var counting: bool = camera2d != null and camera2d.zoom.x < 1.0
	var spots: Dictionary = {}
	for i: int in ids.size():
		# Most people carry no mark: skip the body lookup and projection for them.
		if not (mood or counting or (i < drafted.size() and drafted[i] != 0) or (i < sick.size() and sick[i] != 0)
				or (i < fleeing.size() and fleeing[i] != 0)):
			continue
		var body: Dictionary = people.call("body_of", ids[i])
		if body.is_empty():
			continue
		var feet: Vector3 = body["feet"]
		var top: Vector3 = feet + Vector3(0, float(body["h"]) + HEAD_GAP, 0)
		if cam.is_position_behind(top) or cam.is_position_behind(feet):
			continue
		var p: Vector2 = cam.unproject_position(top)
		if not view.grow(16.0).has_point(p):
			continue
		var asleep: bool = i < states.size() and i < targets.size() and states[i] == 3 and targets[i] == 3
		if i < drafted.size() and drafted[i] != 0:
			draw_arc(p, 8.0, 0.0, TAU, 20, INK, drafted_px + 2.0)
			draw_arc(p, 8.0, 0.0, TAU, 20, drafted_rgba, drafted_px)
			_drawn.append({"kind": "drafted", "text": "", "at": p, "bits": ids[i]})
		if i < sick.size() and sick[i] != 0:
			draw_circle(p + Vector2(10, 0), 4.0, INK)
			draw_circle(p + Vector2(10, 0), 3.0, BADGE)
			_drawn.append({"kind": "sick", "text": "", "at": p, "bits": ids[i]})
		if not asleep and i < fleeing.size() and fleeing[i] != 0:
			draw_rect(Rect2(p + Vector2(-3, -14), Vector2(7, 15)), Color(INK, 0.95))
			draw_rect(Rect2(p + Vector2(-1, -12), Vector2(3, 8)), FLEE_MARK)
			draw_rect(Rect2(p + Vector2(-1, -2), Vector2(3, 2)), FLEE_MARK)
			_drawn.append({"kind": "flee", "text": "", "at": p, "bits": ids[i]})
		if mood:
			var tint: Color = StoryText.band_tint(int(bands[i]) if i < bands.size() else 0)
			draw_circle(p + Vector2(-10, 0), 5.0, INK)
			draw_circle(p + Vector2(-10, 0), 4.0, tint)
			if view.has_point(p):
				_drawn.append({"kind": "mood", "text": "", "at": p, "bits": ids[i], "tint": tint})
		if counting:
			var spot: Vector2i = Vector2i((Vector2(feet.x, feet.z) * 100.0).round())
			var row: Array = spots.get(spot, [p, 0])
			row[1] = int(row[1]) + 1
			row[0] = (row[0] as Vector2) if (row[0] as Vector2).y <= p.y else p
			spots[spot] = row
	for row: Array in spots.values():
		if int(row[1]) > 1:
			_label(str(int(row[1])), (row[0] as Vector2) - Vector2(0, 16), view, "count")


func _draw_hunt_marks(view: Rect2) -> void:
	var animals: Node = _world3d.get("animals")
	var cam: Camera3D = _world3d.call("camera") as Camera3D
	if animals == null or cam == null:
		return
	var ids: PackedInt64Array = _animals.get("ids", PackedInt64Array())
	var huntable: PackedByteArray = _animals.get("huntable", PackedByteArray())
	var marked: PackedByteArray = _animals.get("marked", PackedByteArray())
	for body: Dictionary in animals.call("bodies", OrderTool.Mode.HUNT):
		var i: int = ids.find(int(body["id"]))
		if i < 0 or i >= huntable.size() or huntable[i] == 0:
			continue
		var top: Vector3 = (body["feet"] as Vector3) + Vector3(0, float(body["h"]) + HEAD_GAP, 0)
		if cam.is_position_behind(top):
			continue
		var p: Vector2 = cam.unproject_position(top)
		if not view.grow(16.0).has_point(p):
			continue
		var col: Color = Color(HUNT_MARK, 1.0 if i < marked.size() and marked[i] != 0 else ZONE_MARK_ALPHA)
		draw_arc(p, 7.0, 0.0, TAU, 20, Color(0.137, 0.114, 0.09, col.a), 4.0)
		draw_arc(p, 7.0, 0.0, TAU, 20, col, 2.0)
		if view.has_point(p):
			_drawn.append({"kind": "hunt", "text": "", "at": p, "id": int(body["id"]), "marked": i < marked.size() and marked[i] != 0})


func _draw_hover(rooms: Node, locale: Node, view: Rect2) -> void:
	if hover_tile.x < 0:
		return
	var at: Array = _project(Vector2(hover_tile) + Vector2(0.5, 0.0), 0.0)
	var text: String = tile_text(hover_tile, _stacks, _nodes, _housing, rooms, locale)
	if not text.is_empty() and bool(at[1]):
		_label(text, (at[0] as Vector2) - Vector2(0, 2), view, "tile")
	var cam: Camera3D = _world3d.call("camera") as Camera3D
	var lxs: PackedInt64Array = _livestock.get("xs", PackedInt64Array())
	var lys: PackedInt64Array = _livestock.get("ys", PackedInt64Array())
	# The pen animal drawn under the mouse (its body, as the 2D sprite hit-test; the lowest id where bodies overlap),
	# else the first one standing on the hovered tile.
	var pen: int = (_livestock.get("ids", PackedInt64Array()) as PackedInt64Array).find(_pen_animal_at(cam, hover_screen))
	for i: int in mini(lxs.size(), lys.size()):
		if pen < 0 and Vector2i(lxs[i], lys[i]) == hover_tile:
			pen = i
	if pen >= 0 and pen < mini(lxs.size(), lys.size()):
		var lat: Array = _project(Vector2(lxs[pen], lys[pen]) + Vector2(0.5, 0.5), 1.0)
		if bool(lat[1]):
			_label(livestock_text(_livestock, pen, locale), lat[0], view, "livestock")
	if cam == null or hover_screen.x < 0:
		return
	var animal: Dictionary = ViewProjection.pick(hover_screen, OrderTool.Mode.HUNT)
	if animal.get("kind", "") == "animal":
		var i: int = (_animals.get("ids", PackedInt64Array()) as PackedInt64Array).find(int(animal["id"]))
		if i >= 0:
			_label(animal_text(_animals, i, locale), hover_screen - Vector2(0, 24), view, "animal")
	var visitor: Dictionary = ViewProjection.pick(hover_screen, 0)
	var kind: String = str(visitor.get("kind", ""))
	if kind == "wave" or kind == "caravan":
		var read: Dictionary = SimReads.read(_world_sim, "get_neighbours")
		for row: Dictionary in (NeighbourText.waves(read) if kind == "wave" else NeighbourText.caravans(read)):
			if int(row["id"]) == int(visitor["id"]):
				_label(visitor_text(kind, row, locale), hover_screen - Vector2(0, 24), view, kind)
				break


## The LivestockId of the pen animal whose drawn body (feet to head) passes within `people.pick_radius_px` of screen
## point `screen` (the lowest id when several do), or -1.
func _pen_animal_at(cam: Camera3D, screen: Vector2) -> int:
	var animals: Node = _world3d.get("animals")
	if cam == null or animals == null or screen.x < 0:
		return -1
	var radius: float = float(Data3D.param("people.pick_radius_px", 18))
	var best: int = -1
	for body: Dictionary in animals.call("pen_bodies"):
		var feet: Vector3 = body["feet"]
		var head: Vector3 = feet + Vector3(0, float(body["h"]), 0)
		if cam.is_position_behind(feet) or cam.is_position_behind(head):
			continue
		var a: Vector2 = cam.unproject_position(feet)
		var b: Vector2 = cam.unproject_position(head)
		if Geometry2D.get_closest_point_to_segment(screen, a, b).distance_to(screen) <= maxf(radius, (a - b).length() * 0.5):
			if best < 0 or int(body["id"]) < best:
				best = int(body["id"])
	return best


## A label whose box's bottom centre is at screen point `anchor`, skipped when that point is off screen.
func _label(text: String, anchor: Vector2, view: Rect2, kind: String) -> void:
	if text.is_empty() or not view.has_point(anchor):
		return
	draw_label(self, _font, text, anchor)
	_drawn.append({"kind": kind, "text": text, "at": anchor})


## Text centred on `anchor.x` with its box's bottom at `anchor.y` (screen), `labels.font_px` pixels on a dark box.
static func draw_label(ci: CanvasItem, font: Font, text: String, anchor: Vector2) -> void:
	var px: int = int(ArtData.param("labels.font_px", 24))
	var size: Vector2 = font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, px)
	var pad: float = floorf(px / 4.0)
	var box: Rect2 = Rect2(anchor.round() + Vector2(-size.x * 0.5 - pad, -px - pad), Vector2(size.x + pad * 2.0, px + pad)).abs()
	box.position = box.position.round()
	ci.draw_rect(box, BACK)
	ci.draw_rect(box, BACK_LINE, false, 2.0)
	ci.draw_string(font, box.position + Vector2(pad, px - 2.0), text, HORIZONTAL_ALIGNMENT_LEFT, -1, px, TEXT)
