# agent_layer.gd (ColonyView/AgentLayer in scenes/main.tscn): the people of the M1 colony view, one per tile.
# Each person is a pooled Node2D drawn in this order: a ground shadow; a light halo (a 1-px ring outside the
#   figure, shown on a tile a crown can cover — tree_layer.gd `under_canopy` — and at night away from a fire; never
#   on a sleeper); the logs carried
#   behind the head; the greyscale skin, hair and tunic layers coloured with `modulate` (tunic by household id
#   mod 6, hair by agent id mod 4, elder hair for elders; palettes from the sheet); the outline layer (outline,
#   eyes, belt; never modulated); the log carried across the chest; the job's tool at the right hand (axe for
#   chop, hammer for build) when not carrying; a selection outline for the inspector's person.
# Frames: 0 (stand) when idle, alternating with 1 (stride) on every move step; "sleep" (lying, head left, centred
#   on the tile, no shadow, carry or tool) while the person sleeps — snapshot `states` 3 (consuming) with
#   `target_kinds` 3 (sleep) (M1-ART DECISIONS D1). Children use the child_* layers (8×12, feet on the tile's
#   bottom row). A step to the west flips every layer (`flip_h`); a north/south step keeps the facing.
# Sleepers sharing a tile are spread out for drawing only: in a hut onto its four bedrolls, elsewhere fanned along
#   y (the simulation keeps them on the building's tile).
# At night (DaylightController.night_factor_at) a person within WARM_TILES of a hearth or inside a hut takes a
#   warm `modulate`, so the fire's light falls on them.
# Positions tween from the previous tile to the current one over `move_interval_ticks` at the current speed; the
#   drawn rect is what clicks hit (`entity_at`).
# Also feeds SettlementSelection.observe every frame (the inspector's village row; V8-0 DECISIONS D31).
# M5 (goal 3.5): a drafted person (snapshot `drafted`) carries a red outline (the selection frame tinted red, drawn
#   under the yellow selection one), a sick person (`sick`) a small green badge above the head (`badge_sick`); M
#   toggles the mood overlay, which tints each person by `mood_bands` (StoryText.band_tint: 우울 blue, 보통 none,
#   흡족 yellow, 무너짐 red; it replaces the night's warm light while shown).
# M5.5 (goal §9): a fleeing person (snapshot `fleeing`) shows a small code-drawn "!" above the head (yellow on a dark
#   backing, FleeMark); the drafted outline is unchanged.
# Bridge methods: get_agent_family (direct, per person on a slow refresh); via SimReads get_agent_snapshot,
#   get_world_info, get_society_snapshot, get_settlement_snapshot, get_time_snapshot, get_building_snapshot; signal
#   world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const Daylight = preload("res://scripts/world/daylight_controller.gd")
const SettlementSelection = preload("res://scripts/core/settlement_selection.gd")
const FAMILY_REFRESH_FRAMES: int = 120
# Snapshot job codes (sim-core JobKind::code): 0 none, 1 chop, 2 haul, 3 deliver, 4 build.
const TOOL_FOR_JOB: Dictionary = {1: "tool_axe", 4: "tool_hammer"}
const STATE_CONSUMING: int = 3
const TARGET_SLEEP: int = 3
const LAYERS: Array[String] = ["halo", "skin", "hair", "tunic", "outline"]
const HALO_ALPHA: float = 0.85
const NIGHT_HALO: float = 0.5
const WARM_TILES: int = 5
const WARM: Color = Color(2.0, 1.35, 0.72)
# Where a sleeper lies, as the top-left of its 16×16 cell: in a hut on one of the four bedrolls (offsets from the
#   hut's top-left: gen_tiles.py HUT_BEDROLLS minus (1, 4)), elsewhere fanned out when several share one tile.
const BEDROLLS: Array[Vector2] = [Vector2(8, 5), Vector2(24, 5), Vector2(8, 26), Vector2(24, 26)]
const FAN: Array[Vector2] = [Vector2(0, 0), Vector2(0, -6), Vector2(0, 6), Vector2(-2, -12), Vector2(2, 12)]
const StoryText = preload("res://scripts/ui/story_text.gd")
## M5: the mood overlay key, the drafted outline's tint (over the yellow selection frame) and the badge offset.
const MOOD_KEY: Key = KEY_M
const DRAFT_TINT: Color = Color(1.0, 0.22, 0.2)
const BADGE_OFFSET: Vector2 = Vector2(5, -6)
## M5.5: where the fleeing "!" stands (top-left of its 5×11 backing, relative to the person's cell).
const FLEE_OFFSET: Vector2 = Vector2(6, -14)

signal mood_toggled(on: bool)

var _world_sim: Node = null
var _trees: Node = null
var _persons: Dictionary = {}
var _households: Dictionary = {}
var _family_frame: int = 0
var _selected_bits: int = -1
var _frame: int = 0
var _tunics: Array[Color] = []
var _hairs: Array[Color] = []
var _skin: Color = Color.WHITE
var _elder_hair: Color = Color.WHITE
var _halo: Color = Color.WHITE
var _buildings_key: int = -1
var _hearths: Array[Vector2i] = []
var _huts: Array[Rect2i] = []
var _mood_shown: bool = false


class Person:
	extends Node2D
	var bits: int = 0
	var agent_id: int = 0
	var tile: Vector2i = Vector2i.ZERO
	var from_pos: Vector2 = Vector2.ZERO
	var to_pos: Vector2 = Vector2.ZERO
	var elapsed: float = 0.0
	var duration: float = 0.25
	var stage: int = -1
	var frame: String = ""
	var steps: int = 0
	var west: bool = false
	var seen: int = 0
	var bed: Vector2 = Vector2.ZERO
	var shadow: Sprite2D
	var carry_back: Sprite2D
	var carry_front: Sprite2D
	var tool: Sprite2D
	var select: Sprite2D
	var draft: Sprite2D
	var badge: Sprite2D
	var flee: FleeMark
	var layers: Dictionary = {}


## M5.5: the fleeing person's "!": a 5×11 dark box with a 3-px yellow bar and dot (code-drawn, whole pixels).
class FleeMark:
	extends Node2D
	const BACK: Color = Color(0.078, 0.071, 0.063, 0.95)
	const MARK: Color = Color(0.94, 0.75, 0.25)

	func _draw() -> void:
		draw_rect(Rect2(0, 0, 5, 11), BACK)
		draw_rect(Rect2(1, 1, 3, 6), MARK)
		draw_rect(Rect2(1, 8, 3, 2), MARK)


func _ready() -> void:
	z_index = 5
	y_sort_enabled = true
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_tunics = View.palette("tunics")
	_hairs = View.palette("hairs")
	_skin = View.colour("skin")
	_elder_hair = View.colour("elder_hair")
	_halo = View.colour("halo")
	_halo.a = HALO_ALPHA
	_trees = get_node_or_null("../TreeLayer")
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	for person: Person in _persons.values():
		person.queue_free()
	_persons.clear()
	_households.clear()
	_buildings_key = -1


func _sprite(parent: Node2D, name: String) -> Sprite2D:
	var sprite: Sprite2D = Sprite2D.new()
	sprite.texture = View.texture()
	sprite.centered = false
	sprite.region_enabled = true
	sprite.region_rect = View.rect(name)
	parent.add_child(sprite)
	return sprite


func _make_person(bits: int, agent_id: int, tile: Vector2i) -> Person:
	var person: Person = Person.new()
	person.bits = bits
	person.agent_id = agent_id
	person.tile = tile
	person.position = View.tile_to_world(tile)
	person.from_pos = person.position
	person.to_pos = person.position
	person.elapsed = 1.0
	person.shadow = _sprite(person, "shadow")
	person.shadow.position = Vector2(3, 13)
	person.layers["halo"] = _sprite(person, "person_halo_0")
	person.carry_back = _sprite(person, "carry_back")
	for layer: String in ["skin", "hair", "tunic", "outline"]:
		person.layers[layer] = _sprite(person, "person_%s_0" % layer)
	person.carry_front = _sprite(person, "carry_front")
	person.tool = _sprite(person, "tool_axe")
	person.draft = _sprite(person, "select")
	person.draft.modulate = DRAFT_TINT
	person.select = _sprite(person, "select")
	person.badge = _sprite(person, "badge_sick")
	person.badge.position = BADGE_OFFSET
	person.flee = FleeMark.new()
	person.flee.position = FLEE_OFFSET
	person.flee.visible = false
	person.add_child(person.flee)
	(person.layers["skin"] as Sprite2D).modulate = _skin
	(person.layers["halo"] as Sprite2D).modulate = _halo
	add_child(person)
	return person


## Point the figure layers at `stage`'s sprites for `frame` ("0", "1" or "sleep").
func _set_look(person: Person, stage: int, frame: String) -> void:
	if person.stage == stage and person.frame == frame:
		return
	person.stage = stage
	person.frame = frame
	var prefix: String = "child_" if stage == 1 else "person_"
	var lift: Vector2 = Vector2(0, 4) if frame == "sleep" else Vector2.ZERO
	for layer: String in LAYERS:
		var sprite: Sprite2D = person.layers[layer]
		sprite.region_rect = View.rect("%s%s_%s" % [prefix, layer, frame])
		sprite.position = lift


func _set_west(person: Person, west: bool) -> void:
	person.west = west
	for layer: String in LAYERS:
		(person.layers[layer] as Sprite2D).flip_h = west
	person.carry_back.flip_h = west
	person.carry_front.flip_h = west
	person.tool.flip_h = west


func _process(delta: float) -> void:
	if _world_sim == null:
		return
	_frame += 1
	_observe_settlements()
	var snap: Dictionary = SimReads.read(_world_sim, "get_agent_snapshot")
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var stages: PackedByteArray = snap.get("life_stages", PackedByteArray())
	var jobs: PackedByteArray = snap.get("jobs", PackedByteArray())
	var carries: PackedInt32Array = snap.get("carry_counts", PackedInt32Array())
	var states: PackedByteArray = snap.get("states", PackedByteArray())
	var targets: PackedByteArray = snap.get("target_kinds", PackedByteArray())
	var drafted: PackedByteArray = snap.get("drafted", PackedByteArray())
	var sick: PackedByteArray = snap.get("sick", PackedByteArray())
	var bands: PackedByteArray = snap.get("mood_bands", PackedByteArray())
	var fleeing: PackedByteArray = snap.get("fleeing", PackedByteArray())
	var n: int = mini(mini(bits.size(), ids.size()), mini(xs.size(), ys.size()))
	_family_frame -= 1
	if _family_frame <= 0:
		_family_frame = FAMILY_REFRESH_FRAMES
		_refresh_households(bits, ids, n)
	var duration: float = _step_seconds()
	var hour: float = float((SimReads.read(_world_sim, "get_time_snapshot") as Dictionary).get("time_of_day", 12.0))
	var night: float = Daylight.night_factor_at(hour)
	if night > 0.0:
		_refresh_warm_sources()
	var sleepers: int = 0
	var inspector: Node = get_node_or_null("/root/Main/UI/AgentInspectorPanel")
	_selected_bits = int(inspector.call("get_selected_entity_bits")) if inspector != null and (inspector as Control).visible else -1
	for i: int in n:
		var key: int = bits[i]
		var tile: Vector2i = Vector2i(xs[i], ys[i])
		var person: Person = _persons.get(key)
		if person == null:
			person = _make_person(key, ids[i], tile)
			_persons[key] = person
		elif tile != person.tile:
			person.from_pos = person.position
			person.to_pos = View.tile_to_world(tile)
			if tile.x != person.tile.x:
				_set_west(person, tile.x < person.tile.x)
			person.tile = tile
			person.elapsed = 0.0
			person.duration = duration
			person.steps += 1
		person.seen = _frame
		var stage: int = int(stages[i]) if i < stages.size() else 0
		var asleep: bool = i < states.size() and i < targets.size() and states[i] == STATE_CONSUMING and targets[i] == TARGET_SLEEP
		var moving: bool = person.elapsed < person.duration
		var frame: String = "sleep" if asleep else ("1" if moving and person.steps % 2 == 1 else "0")
		if asleep:
			sleepers += 1
		_set_look(person, stage, frame)
		var household: int = int(_households.get(ids[i], ids[i]))
		(person.layers["tunic"] as Sprite2D).modulate = _tunics[posmod(household, _tunics.size())]
		(person.layers["hair"] as Sprite2D).modulate = _elder_hair if stage == 2 else _hairs[posmod(ids[i], _hairs.size())]
		var carrying: bool = not asleep and i < carries.size() and carries[i] > 0
		person.carry_back.visible = carrying
		person.carry_front.visible = carrying
		person.shadow.visible = not asleep
		var job: int = int(jobs[i]) if i < jobs.size() else 0
		person.tool.visible = not asleep and not carrying and TOOL_FOR_JOB.has(job)
		if person.tool.visible:
			person.tool.region_rect = View.rect(TOOL_FOR_JOB[job])
		person.select.visible = key == _selected_bits
		person.draft.visible = i < drafted.size() and drafted[i] != 0
		person.badge.visible = i < sick.size() and sick[i] != 0
		person.flee.visible = not asleep and i < fleeing.size() and fleeing[i] != 0
		var warm: bool = night > 0.0 and _is_warm(tile)
		person.modulate = Color.WHITE.lerp(WARM, night) if warm else Color.WHITE
		if _mood_shown:
			person.modulate = StoryText.band_tint(int(bands[i]) if i < bands.size() else 0)
		# The halo marks people that would be hard to find: under a crown, or in the dark away from a fire.
		var halo: bool = (night >= NIGHT_HALO and not warm) or (_trees != null and bool(_trees.call("under_canopy", tile)))
		(person.layers["halo"] as Sprite2D).visible = halo and not asleep
	for key: int in _persons.keys():
		var person: Person = _persons[key]
		if person.seen != _frame:
			person.queue_free()
			_persons.erase(key)
	if sleepers > 0:
		_place_sleepers()
	for person: Person in _persons.values():
		if person.frame == "sleep":
			person.position = person.to_pos + person.bed
		elif person.elapsed < person.duration:
			person.elapsed = minf(person.duration, person.elapsed + delta)
			person.position = person.from_pos.lerp(person.to_pos, person.elapsed / person.duration).round()
		else:
			person.position = person.to_pos


## Spread the sleepers that share a tile: in a hut onto its bedrolls (by agent id), elsewhere fanned along y.
func _place_sleepers() -> void:
	var by_tile: Dictionary = {}
	for person: Person in _persons.values():
		person.bed = Vector2.ZERO
		if person.frame == "sleep":
			if not by_tile.has(person.tile):
				by_tile[person.tile] = []
			(by_tile[person.tile] as Array).append(person)
	if by_tile.is_empty():
		return
	_refresh_warm_sources()
	for tile: Vector2i in by_tile.keys():
		var group: Array = by_tile[tile]
		group.sort_custom(func(a: Person, b: Person) -> bool: return a.agent_id < b.agent_id)
		var hut: Rect2i = Rect2i()
		for r: Rect2i in _huts:
			if r.has_point(tile):
				hut = r
		for k: int in group.size():
			var person: Person = group[k]
			if hut.size.x > 0:
				var corner: Vector2 = View.tile_to_world(hut.position) + BEDROLLS[k % BEDROLLS.size()]
				person.bed = corner - View.tile_to_world(tile) + Vector2(0, 6 * floori(float(k) / BEDROLLS.size()))
			elif group.size() > 1:
				person.bed = FAN[k % FAN.size()]


## Hearth tiles and hut footprints from the building snapshot (re-read only when it changes).
func _refresh_warm_sources() -> void:
	var b: Dictionary = SimReads.read(_world_sim, "get_building_snapshot")
	var xs: PackedInt32Array = b.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = b.get("ys", PackedInt32Array())
	var kinds: PackedByteArray = b.get("kinds", PackedByteArray())
	var key: int = hash([xs, ys, kinds])
	if key == _buildings_key:
		return
	_buildings_key = key
	_hearths.clear()
	_huts.clear()
	for i: int in mini(kinds.size(), mini(xs.size(), ys.size())):
		if kinds[i] == 0:
			_hearths.append(Vector2i(xs[i], ys[i]))
		elif kinds[i] == 3:
			_huts.append(Rect2i(xs[i] - 1, ys[i] - 1, 3, 3))


func _is_warm(tile: Vector2i) -> bool:
	for h: Vector2i in _hearths:
		if maxi(absi(tile.x - h.x), absi(tile.y - h.y)) <= WARM_TILES:
			return true
	for r: Rect2i in _huts:
		if r.has_point(tile):
			return true
	return false


## Seconds one step lasts at the current speed: move_interval_ticks / (12 × speed).
func _step_seconds() -> float:
	var info: Dictionary = SimReads.read(_world_sim, "get_world_info")
	var interval: float = float(info.get("move_interval_ticks", 3))
	var camera: Node = get_node_or_null("/root/Main/Camera2D")
	var speed: float = float(camera.call("get_speed")) if camera != null and camera.has_method("get_speed") else 1.0
	return maxf(0.02, interval / (12.0 * maxf(1.0, speed)))


## Household id per AgentId: an adult's is min(self, partner); a child's is its mother's household.
func _refresh_households(bits: PackedInt64Array, ids: PackedInt64Array, n: int) -> void:
	var mothers: Dictionary = {}
	for i: int in n:
		var family: Dictionary = _world_sim.call("get_agent_family", bits[i])
		var partner: int = int(family.get("partner", -1))
		_households[ids[i]] = mini(ids[i], partner) if partner >= 0 else ids[i]
		mothers[ids[i]] = int(family.get("mother", -1))
	for id: int in mothers.keys():
		var mother: int = mothers[id]
		if mother >= 0 and _households.has(mother):
			_households[id] = _households[mother]


func _observe_settlements() -> void:
	var society: Dictionary = SimReads.read(_world_sim, "get_society_snapshot")
	var settle_snap: Variant = SimReads.read(_world_sim, "get_settlement_snapshot")
	var settlements: Dictionary = settle_snap.duplicate() if settle_snap is Dictionary else {}
	if not settlements.has("settlement_ids"):
		settlements["settlement_ids"] = PackedInt32Array(Array(settlements.get("ids", PackedInt64Array())))
	SettlementSelection.observe(society, settlements)


## Entity bits of the person drawn at world position `p` (the lowest on screen wins), or -1.
func entity_at(p: Vector2) -> int:
	var best: int = -1
	var best_y: float = -INF
	for person: Person in _persons.values():
		if Rect2(person.position, Vector2(View.TILE, View.TILE)).has_point(p) and person.position.y > best_y:
			best = person.bits
			best_y = person.position.y
	return best


## The drawn top-left of the person with entity bits `bits` (for tools), or (-1, -1).
func drawn_position(bits: int) -> Vector2:
	var person: Person = _persons.get(bits)
	return person.position if person != null else Vector2(-1, -1)


## The frame ("0", "1", "sleep") and halo visibility of the person with entity bits `bits` (for tools), or {}.
func person_look(bits: int) -> Dictionary:
	var person: Person = _persons.get(bits)
	if person == null:
		return {}
	return {"frame": person.frame, "west": person.west, "halo": (person.layers["halo"] as Sprite2D).visible,
		"carry": person.carry_front.visible, "tool": person.tool.visible, "warm": person.modulate != Color.WHITE,
		"drafted": person.draft.visible, "sick": person.badge.visible}


## M5: show or hide the mood overlay (M).
func set_mood_shown(on: bool) -> void:
	if on == _mood_shown:
		return
	_mood_shown = on
	mood_toggled.emit(on)


func mood_shown() -> bool:
	return _mood_shown


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and (event as InputEventKey).keycode == MOOD_KEY:
		set_mood_shown(not _mood_shown)
		get_viewport().set_input_as_handled()


## M5: the drafted outline, sick badge, M5.5 fleeing "!" and tint of the person with entity bits `bits` (for tools), or {}.
func person_marks(bits: int) -> Dictionary:
	var person: Person = _persons.get(bits)
	if person == null:
		return {}
	return {"drafted": person.draft.visible, "sick": person.badge.visible, "fleeing": person.flee.visible, "tint": person.modulate}


## How many people are drawn.
func person_count() -> int:
	return _persons.size()
