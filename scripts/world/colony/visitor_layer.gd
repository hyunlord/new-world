# visitor_layer.gd (ColonyView/VisitorLayer, created by animal_layer.gd beside itself): the M5.5 visitors on the map
#   (goal §9) at 64-px tiles (M-ART2 §3.5). Visitors are not agents: a migrant wave or a caravan is one group on one
#   tile of get_neighbours (`wave_x/y`, `caravan_x/y`), drawn while it walks in, waits at the hearth or leaves (states
#   0–2; gone ones are not drawn). They are people, drawn as route I figures (standing Sprite2Ds in the y-sorted
#   ColonyView/World, anchor on the base point): a wave as up to six people in a fan west of its tile, a caravan as
#   two traders (the `carry` pose of bindings `people.pose_for_state`) with a pack (bindings `visitors.pack`, an item
#   sprite) between them east of its tile, so the two never cover each other at the hearth. Each person's look is a
#   keyed hash of (group number, person) among the manifest looks whose age is in bindings `visitors.look_ages` — no
#   names; no variant tint (visitors are strangers seen from afar). Facing: E while the group steps east, E flipped
#   west, else S. Below 1× (`lod.people_below_zoom`) the figures are hidden and this node (z 3, above the world layer)
#   draws each person as a filled circle in `visitors.lod_colour` with a 1-px ink ring (`lod.marker_px` screen
#   pixels). Groups ease from tile to tile like the animals (a jump of more than two tiles, as after advance_ticks, is
#   placed at once). Under the mouse a group shows "이주민 4명 · 붉은바위" / "교역 무리 · 붉은바위" on a dark backing box.
#   The drawn rects are the click targets (`wave_at`, `caravan_at`): order_tool.gd opens the migrant dialog or the
#   trade dialog on a click.
# Bridge methods: via SimReads get_neighbours, get_world_info; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const ArtData = preload("res://scripts/world/art/art_data.gd")
const NeighbourText = preload("res://scripts/ui/neighbour_text.gd")
const OverlayLayer = preload("res://scripts/world/colony/overlay_layer.gd")
const FONT_PATH: String = "res://assets/fonts/galmuri/Galmuri11.ttf"
const TILE: int = View.TILE
## Fan of a wave's people, base-point offsets from the group tile's base point (world px; west of the tile).
const WAVE_FAN: Array[Vector2] = [Vector2(-40, -6), Vector2(-64, 2), Vector2(-88, -6), Vector2(-52, 18), Vector2(-76, 22), Vector2(-100, 14)]
## A caravan: two traders and their pack east of the tile.
const CARAVAN_FAN: Array[Vector2] = [Vector2(36, 0), Vector2(84, 4)]
const PACK_AT: Vector2 = Vector2(60, 6)
const MAX_MARKERS: int = 6
## A group that moved further than this between two reads (the clock jumped) is placed, not eased.
const SNAP_TILES: int = 2
const SALT_LOOK: int = 31

var _world_sim: Node = null
var _world: Node2D = null
var _font: Font = null
## Groups by key ("w<id>" / "c<id>"): {kind, id, neighbour, size, tile, from, to, at, elapsed, duration, seen, flip,
## facing, sprites (Array[Sprite2D])}.
var _groups: Dictionary = {}
var _frame: int = 0
var _hover: String = ""
var _drawn: bool = false
var _lod: bool = false
var _zoom: float = 1.0


func _ready() -> void:
	z_as_relative = false
	z_index = 3
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_font = load(FONT_PATH) as Font
	_world = get_node_or_null("../World")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	for key: String in _groups.keys():
		_free_group(_groups[key])
	_groups.clear()
	_hover = ""
	queue_redraw()


func _free_group(group: Dictionary) -> void:
	for s: Node in group.get("sprites", []):
		s.queue_free()


func _process(delta: float) -> void:
	if _world_sim == null or not _world_sim.has_method("get_neighbours"):
		return
	_frame += 1
	_zoom = get_viewport().get_canvas_transform().x.x
	_lod = _zoom < float(ArtData.param("lod.people_below_zoom", 1.0)) - 0.001
	var read: Dictionary = SimReads.read(_world_sim, "get_neighbours")
	NeighbourText.observe(read)
	var duration: float = _step_seconds()
	for wave: Dictionary in NeighbourText.waves(read):
		if int(wave["state"]) <= NeighbourText.LEAVING:
			_track("w%d" % int(wave["id"]), "wave", wave, int(wave["size"]), duration)
	for caravan: Dictionary in NeighbourText.caravans(read):
		if int(caravan["state"]) <= NeighbourText.LEAVING:
			_track("c%d" % int(caravan["id"]), "caravan", caravan, 2, duration)
	for key: String in _groups.keys():
		var group: Dictionary = _groups[key]
		if int(group["seen"]) != _frame:
			_free_group(group)
			_groups.erase(key)
			continue
		var elapsed: float = float(group["elapsed"])
		var span: float = float(group["duration"])
		if elapsed < span:
			elapsed = minf(span, elapsed + delta)
			group["elapsed"] = elapsed
			group["at"] = (group["from"] as Vector2).lerp(group["to"] as Vector2, elapsed / span).round()
		else:
			group["at"] = group["to"]
			group["facing"] = 0
		_place(group)
	_hover = group_at(get_global_mouse_position())
	var needs: bool = _hover != "" or (_lod and not _groups.is_empty())
	if needs or _drawn:
		queue_redraw()
	_drawn = needs


func _track(key: String, kind: String, row: Dictionary, count: int, duration: float) -> void:
	var tile: Vector2i = row["tile"]
	var target: Vector2 = View.tile_to_world(tile)
	var group: Dictionary = _groups.get(key, {})
	if group.is_empty():
		group = {"kind": kind, "id": int(row["id"]), "tile": tile, "from": target, "to": target, "at": target, "elapsed": 1.0,
			"duration": 1.0, "flip": false, "facing": 0, "sprites": []}
		_groups[key] = group
	elif tile != group["tile"]:
		var jump: Vector2i = tile - (group["tile"] as Vector2i)
		group["from"] = group["at"] if maxi(absi(jump.x), absi(jump.y)) <= SNAP_TILES else target
		group["to"] = target
		group["tile"] = tile
		group["elapsed"] = 0.0
		group["duration"] = duration
		group["facing"] = 1 if jump.x != 0 else 0
		group["flip"] = jump.x < 0
	group["neighbour"] = int(row["neighbour"])
	group["size"] = count
	group["state"] = int(row["state"])
	group["seen"] = _frame


## The base-point offsets of a group's people (and, for a caravan, its pack last).
func _offsets(group: Dictionary) -> Array[Vector2]:
	var out: Array[Vector2] = []
	if str(group["kind"]) == "wave":
		for k: int in clampi(int(group["size"]), 1, MAX_MARKERS):
			out.append(WAVE_FAN[k])
	else:
		out.append_array(CARAVAN_FAN)
	return out


## The look index of person `k` of group number `id` (`wave` or caravan), among the looks of `visitors.look_ages`.
static func look_index(id: int, k: int, wave: bool) -> int:
	var ages: Array = (ArtData.bindings().get("visitors", {}) as Dictionary).get("look_ages", [])
	var pool: Array[int] = []
	for i: int in ArtData.look_count():
		if ages.is_empty() or ages.has(str(ArtData.look(i).get("age", ""))):
			pool.append(i)
	if pool.is_empty():
		return -1
	return pool[View.key_hash(id, k, 1 if wave else 2, SALT_LOOK) % pool.size()]


## The manifest frame path of person `k` of `group` facing index `facing`.
static func frame_path_of(group: Dictionary, k: int, facing: int) -> String:
	var look: Dictionary = ArtData.look(look_index(int(group["id"]), k, str(group["kind"]) == "wave"))
	if look.is_empty():
		return ""
	var people: Dictionary = ArtData.manifest().get("people", {})
	var facings: Array = people.get("facings", [])
	var poses: Dictionary = (ArtData.bindings().get("people", {}) as Dictionary).get("pose_for_state", {})
	var pose: String = str(poses.get("idle" if str(group["kind"]) == "wave" else "carry", ""))
	if facings.is_empty():
		return ""
	return ArtData.frame_path(str(look.get("frames", "")), str(facings[clampi(facing, 0, facings.size() - 1)]), pose)


## Create or move the group's sprites (people, and a caravan's pack); hidden under LOD.
func _place(group: Dictionary) -> void:
	if _world == null:
		return
	var sprites: Array = group["sprites"]
	var offs: Array[Vector2] = _offsets(group)
	var wanted: int = offs.size() + (1 if str(group["kind"]) == "caravan" else 0)
	var facing: int = int(group.get("facing", 0))
	var stamp: String = "%d|%d" % [facing, int(bool(group.get("flip", false)))]
	if sprites.size() != wanted or str(group.get("stamp", "")) != stamp:
		for s: Node in sprites:
			s.queue_free()
		sprites.clear()
		for k: int in offs.size():
			var path: String = frame_path_of(group, k, facing)
			var tex: Texture2D = ArtData.texture_at(path)
			var s: Sprite2D = _sprite(tex)
			s.flip_h = bool(group.get("flip", false)) and facing == 1
			var anchor: Vector2 = _anchor_of(tex)
			s.offset = -anchor if not s.flip_h or tex == null else -Vector2(tex.get_width() - anchor.x, anchor.y)
			sprites.append(s)
		if str(group["kind"]) == "caravan":
			var pack: String = str((ArtData.bindings().get("visitors", {}) as Dictionary).get("pack", ""))
			var ps: Sprite2D = _sprite(ArtData.texture(pack))
			ps.offset = -Vector2(ArtData.anchor(pack))
			sprites.append(ps)
		group["stamp"] = stamp
	var base: Vector2 = (group["at"] as Vector2) + Vector2(View.BASE)
	for k: int in sprites.size():
		var s: Sprite2D = sprites[k]
		var off: Vector2 = offs[k] if k < offs.size() else PACK_AT
		s.position = (base + off).round()
		s.visible = not _lod


func _sprite(tex: Texture2D) -> Sprite2D:
	var s: Sprite2D = Sprite2D.new()
	s.texture = tex
	s.centered = false
	s.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world.add_child(s)
	return s


## A person frame's anchor (the manifest's upright anchor; the frame's bottom centre when it is unknown).
func _anchor_of(tex: Texture2D) -> Vector2:
	var upright: Dictionary = (ArtData.manifest().get("people", {}) as Dictionary).get("upright", {})
	var a: Variant = upright.get("anchor", null)
	if a is Array and (a as Array).size() == 2:
		return Vector2(float(a[0]), float(a[1]))
	return Vector2(tex.get_width() * 0.5, tex.get_height()) if tex != null else Vector2.ZERO


func _draw() -> void:
	var inv: float = 1.0 / maxf(0.05, _zoom)
	if _lod:
		var lc: Variant = (ArtData.bindings().get("visitors", {}) as Dictionary).get("lod_colour", [196, 182, 150])
		var fill: Color = Color8(int(lc[0]), int(lc[1]), int(lc[2])) if lc is Array and (lc as Array).size() >= 3 else Color.WHITE
		var rv: Variant = ArtData.param("lod.ring_rgba", [35, 29, 23, 255])
		var ring: Color = Color8(int(rv[0]), int(rv[1]), int(rv[2]), int(rv[3])) if rv is Array and (rv as Array).size() == 4 else Color8(35, 29, 23)
		var r: float = _marker_px() * 0.5 * inv
		for key: String in _groups.keys():
			var group: Dictionary = _groups[key]
			var base: Vector2 = (group["at"] as Vector2) + Vector2(View.BASE)
			for off: Vector2 in _offsets(group):
				var c: Vector2 = base + off - Vector2(0, r)
				draw_circle(c, r + inv, ring)
				draw_circle(c, r, fill)
	if _hover != "" and _groups.has(_hover) and _font != null:
		_hover_box(_groups[_hover])


func _marker_px() -> float:
	var table: Variant = ArtData.param("lod.marker_px", {})
	var best: float = 8.0
	var best_d: float = INF
	if table is Dictionary:
		for k: Variant in (table as Dictionary).keys():
			var d: float = absf(float(str(k)) - _zoom)
			if d < best_d:
				best_d = d
				best = float((table as Dictionary)[k])
	return best


func _hover_box(group: Dictionary) -> void:
	var locale: Node = get_node_or_null("/root/Locale")
	var who: String = NeighbourText.name_of(locale, int(group["neighbour"]))
	var text: String = NeighbourText.trf(locale, "M55_HOVER_WAVE_FMT", {"count": int(group["size"]), "neighbour": who}) if str(group["kind"]) == "wave" \
		else NeighbourText.trf(locale, "M55_HOVER_CARAVAN_FMT", {"neighbour": who})
	var rect: Rect2 = _rect_of(group)
	OverlayLayer.draw_label(self, _font, text, Vector2(rect.get_center().x, rect.position.y - 2), _zoom)


## The drawn rect of a group: the union of its people's frames (or LOD markers) and the pack.
func _rect_of(group: Dictionary) -> Rect2:
	var base: Vector2 = (group["at"] as Vector2) + Vector2(View.BASE)
	var upright: Dictionary = (ArtData.manifest().get("people", {}) as Dictionary).get("upright", {})
	var canvas_v: Variant = upright.get("canvas", [80, 112])
	var canvas: Vector2 = Vector2(float(canvas_v[0]), float(canvas_v[1])) if canvas_v is Array and (canvas_v as Array).size() == 2 else Vector2(TILE, TILE)
	var anchor: Vector2 = _anchor_of(null)
	var out: Rect2 = Rect2()
	var first: bool = true
	for off: Vector2 in _offsets(group):
		var r: Rect2 = Rect2(base + off - anchor, canvas)
		if _lod:
			var m: float = _marker_px() / maxf(0.05, _zoom)
			r = Rect2(base + off - Vector2(m * 0.5, m), Vector2(m, m))
		out = r if first else out.merge(r)
		first = false
	return out


## Seconds one step lasts at the current speed (as AgentLayer): move_interval_ticks / (12 × speed).
func _step_seconds() -> float:
	var info: Dictionary = SimReads.read(_world_sim, "get_world_info")
	var interval: float = float(info.get("move_interval_ticks", 3))
	var camera: Node = get_node_or_null("/root/Main/Camera2D")
	var speed: float = float(camera.call("get_speed")) if camera != null and camera.has_method("get_speed") else 1.0
	return maxf(0.02, interval / (12.0 * maxf(1.0, speed)))


## The key ("w<id>" / "c<id>") of the group drawn at world position `p` (a caravan wins over a wave), or "".
func group_at(p: Vector2) -> String:
	var best: String = ""
	for key: String in _groups.keys():
		if _rect_of(_groups[key]).has_point(p) and (best == "" or key.begins_with("c")):
			best = key
	return best


## The wave number drawn at world position `p`, or -1.
func wave_at(p: Vector2) -> int:
	var key: String = group_at(p)
	return int(_groups[key]["id"]) if key.begins_with("w") else -1


## The caravan number drawn at world position `p`, or -1.
func caravan_at(p: Vector2) -> int:
	var key: String = group_at(p)
	return int(_groups[key]["id"]) if key.begins_with("c") else -1


## The drawn rect of wave or caravan `id` (`kind` "wave" / "caravan") in world pixels (for tools), or an empty rect.
func drawn_rect(kind: String, id: int) -> Rect2:
	var key: String = ("w%d" if kind == "wave" else "c%d") % id
	return _rect_of(_groups[key]) if _groups.has(key) else Rect2()


## How many waves and caravans are drawn (for tests and tools).
func counts() -> Dictionary:
	var waves: int = 0
	for key: String in _groups.keys():
		if key.begins_with("w"):
			waves += 1
	return {"waves": waves, "caravans": _groups.size() - waves}
