# visitor_layer.gd (ColonyView/VisitorLayer, created by animal_layer.gd beside itself): the M5.5 visitors on the map
#   (goal §9). Visitors are not agents: a migrant wave or a caravan is one group on one tile of get_neighbours
#   (`wave_x/y`, `caravan_x/y`), drawn in code as placeholders while it walks in, waits at the hearth or leaves (states
#   0–2; gone ones are not drawn): a wave as up to six small people-markers (head, tunic, 1-px dark outline) in a
#   fan left of its tile, a caravan as two traders with a brown pack between them right of its tile, so the two never
#   cover each other at the hearth. Groups ease from tile to tile like the animals (a jump of more than
#   two tiles, as after advance_ticks, is placed at once). Under the mouse a group shows
#   "이주민 4명 · 붉은바위" / "교역 무리 · 붉은바위" on a dark backing box. The drawn rects are the click targets
#   (`wave_at`, `caravan_at`): order_tool.gd opens the migrant dialog or the trade dialog on a click.
# Bridge methods: via SimReads get_neighbours, get_world_info; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const NeighbourText = preload("res://scripts/ui/neighbour_text.gd")
const FONT_PATH: String = "res://assets/fonts/galmuri/Galmuri11.ttf"
const FONT_SIZE: int = 12
const OUTLINE: Color = Color8(20, 18, 16)
const SKIN: Color = Color("d8a878")
const HAIR: Color = Color("3a2a1e")
## Tunic colours of the migrants (by person index) and of the traders.
const TUNICS: Array[Color] = [Color("8a6a48"), Color("6a7a50"), Color("9a5a40"), Color("5a6a7a"), Color("8a7a50"), Color("7a5a6a")]
const TRADER_TUNIC: Color = Color("b08840")
const PACK: Color = Color("7a5530")
const PACK_LIGHT: Color = Color("9a7040")
const BACK: Color = Color(0.094, 0.082, 0.075, 0.92)
const BACK_LINE: Color = Color(0.275, 0.243, 0.212, 1.0)
const TEXT: Color = Color(0.96, 0.92, 0.84)
## A wave's markers fan out left of its tile; a caravan stands right of its tile (world pixels from the tile's top-left).
const WAVE_ORIGIN: Vector2 = Vector2(-26, 0)
const WAVE_SIZE: Vector2 = Vector2(32, 16)
const CARAVAN_ORIGIN: Vector2 = Vector2(14, 0)
const CARAVAN_SIZE: Vector2 = Vector2(28, 16)
const MAX_MARKERS: int = 6
## A group that moved further than this between two reads (the clock jumped) is placed, not eased.
const SNAP_TILES: int = 2

var _world_sim: Node = null
var _font: Font = null
## Groups by key ("w<id>" / "c<id>"): {kind, id, neighbour, size, tile, from, to, at, elapsed, duration, seen}.
var _groups: Dictionary = {}
var _frame: int = 0
var _hover: String = ""
var _drawn: bool = false


func _ready() -> void:
	z_index = 5
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_font = load(FONT_PATH) as Font
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	_groups.clear()
	_hover = ""
	queue_redraw()


func _process(delta: float) -> void:
	if _world_sim == null or not _world_sim.has_method("get_neighbours"):
		return
	_frame += 1
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
	_hover = group_at(get_global_mouse_position())
	if not _groups.is_empty() or _drawn:
		queue_redraw()
	_drawn = not _groups.is_empty()


func _track(key: String, kind: String, row: Dictionary, count: int, duration: float) -> void:
	var tile: Vector2i = row["tile"]
	var target: Vector2 = View.tile_to_world(tile)
	var group: Dictionary = _groups.get(key, {})
	if group.is_empty():
		group = {"kind": kind, "id": int(row["id"]), "tile": tile, "from": target, "to": target, "at": target, "elapsed": 1.0, "duration": 1.0}
		_groups[key] = group
	elif tile != group["tile"]:
		var jump: Vector2i = tile - (group["tile"] as Vector2i)
		group["from"] = group["at"] if maxi(absi(jump.x), absi(jump.y)) <= SNAP_TILES else target
		group["to"] = target
		group["tile"] = tile
		group["elapsed"] = 0.0
		group["duration"] = duration
	group["neighbour"] = int(row["neighbour"])
	group["size"] = count
	group["state"] = int(row["state"])
	group["seen"] = _frame


func _draw() -> void:
	for key: String in _groups.keys():
		var group: Dictionary = _groups[key]
		var at: Vector2 = group["at"]
		if str(group["kind"]) == "wave":
			var n: int = clampi(int(group["size"]), 1, MAX_MARKERS)
			for k: int in n:
				var col: int = k % 3
				var row: int = floori(k / 3.0)
				_person(at + WAVE_ORIGIN + Vector2(2 + col * 10 + row * 4, 1 + row * 4), TUNICS[(int(group["id"]) + k) % TUNICS.size()])
		else:
			var origin: Vector2 = at + CARAVAN_ORIGIN
			_person(origin + Vector2(1, 1), TRADER_TUNIC)
			_pack(origin + Vector2(9, 6))
			_person(origin + Vector2(19, 1), TRADER_TUNIC)
	if _hover != "" and _groups.has(_hover) and _font != null:
		_hover_box(_groups[_hover])


## A small person-marker with its top-left at `p`: a 4×4 head, a 6×7 tunic, 1-px outline (8×14 overall).
func _person(p: Vector2, tunic: Color) -> void:
	draw_rect(Rect2(p + Vector2(2, 0), Vector2(6, 6)), OUTLINE)
	draw_rect(Rect2(p + Vector2(3, 1), Vector2(4, 4)), SKIN)
	draw_rect(Rect2(p + Vector2(3, 1), Vector2(4, 1)), HAIR)
	draw_rect(Rect2(p + Vector2(1, 5), Vector2(8, 9)), OUTLINE)
	draw_rect(Rect2(p + Vector2(2, 6), Vector2(6, 7)), tunic)


## The traders' pack: an 8×7 bundle with a light strap.
func _pack(p: Vector2) -> void:
	draw_rect(Rect2(p, Vector2(10, 9)), OUTLINE)
	draw_rect(Rect2(p + Vector2(1, 1), Vector2(8, 7)), PACK)
	draw_rect(Rect2(p + Vector2(1, 3), Vector2(8, 1)), PACK_LIGHT)


func _hover_box(group: Dictionary) -> void:
	var locale: Node = get_node_or_null("/root/Locale")
	var who: String = NeighbourText.name_of(locale, int(group["neighbour"]))
	var text: String = NeighbourText.trf(locale, "M55_HOVER_WAVE_FMT", {"count": int(group["size"]), "neighbour": who}) if str(group["kind"]) == "wave" \
		else NeighbourText.trf(locale, "M55_HOVER_CARAVAN_FMT", {"neighbour": who})
	var rect: Rect2 = _rect_of(group)
	var text_size: Vector2 = _font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE)
	var box: Rect2 = Rect2(Vector2(rect.get_center().x - text_size.x * 0.5 - 3, rect.position.y - FONT_SIZE - 4), Vector2(text_size.x + 6, FONT_SIZE + 3))
	box.position = box.position.round()
	draw_rect(box, BACK)
	draw_rect(box, BACK_LINE, false, 1.0)
	draw_string(_font, box.position + Vector2(3, FONT_SIZE - 1), text, HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE, TEXT)


func _rect_of(group: Dictionary) -> Rect2:
	var at: Vector2 = group["at"]
	if str(group["kind"]) == "wave":
		return Rect2(at + WAVE_ORIGIN, WAVE_SIZE)
	return Rect2(at + CARAVAN_ORIGIN, CARAVAN_SIZE)


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
