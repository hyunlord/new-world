# agent_layer.gd (ColonyView/AgentLayer in scenes/main.tscn): the people of the M1 colony view, one per tile.
# Each person is a pooled Node2D: a ground shadow, then greyscale skin, hair and tunic layers coloured with
#   `modulate` (tunic by household id mod 6, hair by agent id mod 4, grey for elders; children use the 12×12
#   layers), a carried-log bundle (light, outlined, drawn 4 px above an adult's cell, 3 px above a child's, so it sits on the head), a tool
#   icon for the job (axe for chop, hammer for build), and a selection
#   outline for the inspector's person. Positions tween from the previous tile to the current one over
#   `move_interval_ticks` at the current speed; the drawn rect is what clicks hit (`entity_at`).
# Also feeds SettlementSelection.observe every frame (the inspector's village row; V8-0 DECISIONS D31).
# Bridge methods: get_agent_family (direct, per person on a slow refresh); via SimReads get_agent_snapshot,
#   get_world_info, get_society_snapshot, get_settlement_snapshot; signal world_reset.
extends Node2D

const View = preload("res://scripts/world/colony/view.gd")
const SettlementSelection = preload("res://scripts/core/settlement_selection.gd")
const FAMILY_REFRESH_FRAMES: int = 120
# Snapshot job codes (sim-core JobKind::code): 0 none, 1 chop, 2 haul, 3 deliver, 4 build.
const TOOL_FOR_JOB: Dictionary = {1: "tool_axe", 4: "tool_hammer"}
const CARRY_Y_ADULT: int = -4
const CARRY_Y_CHILD: int = -3

var _world_sim: Node = null
var _persons: Dictionary = {}
var _households: Dictionary = {}
var _family_frame: int = 0
var _selected_bits: int = -1


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
	var shadow: Sprite2D
	var skin: Sprite2D
	var hair: Sprite2D
	var tunic: Sprite2D
	var carry: Sprite2D
	var tool: Sprite2D
	var select: Sprite2D


func _ready() -> void:
	z_index = 5
	y_sort_enabled = true
	texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	_world_sim = get_node_or_null("/root/Main/WorldSim")
	if _world_sim != null:
		_world_sim.connect("world_reset", _on_world_reset)


func _on_world_reset() -> void:
	for person: Person in _persons.values():
		person.queue_free()
	_persons.clear()
	_households.clear()


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
	person.skin = _sprite(person, "person_skin")
	person.hair = _sprite(person, "person_hair")
	person.tunic = _sprite(person, "person_tunic")
	person.carry = _sprite(person, "carry_log")
	person.tool = _sprite(person, "tool_axe")
	person.select = _sprite(person, "select")
	add_child(person)
	return person


func _set_stage(person: Person, stage: int) -> void:
	if person.stage == stage:
		return
	person.stage = stage
	var prefix: String = "child_" if stage == 1 else "person_"
	person.skin.region_rect = View.rect(prefix + "skin")
	person.hair.region_rect = View.rect(prefix + "hair")
	person.tunic.region_rect = View.rect(prefix + "tunic")
	person.shadow.position = Vector2(0, 10 if stage == 1 else 11)
	# The carried bundle (rows 0..6 of its tile) rides on the head: it covers the hair, the eyes stay visible
	# (adult: head top row 0, eyes row 3; child: head top row 2, eyes row 4).
	person.carry.position = Vector2(0, CARRY_Y_CHILD if stage == 1 else CARRY_Y_ADULT)


func _process(delta: float) -> void:
	if _world_sim == null:
		return
	_observe_settlements()
	var snap: Dictionary = SimReads.read(_world_sim, "get_agent_snapshot")
	var bits: PackedInt64Array = snap.get("ids", PackedInt64Array())
	var ids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	var xs: PackedInt32Array = snap.get("xs", PackedInt32Array())
	var ys: PackedInt32Array = snap.get("ys", PackedInt32Array())
	var stages: PackedByteArray = snap.get("life_stages", PackedByteArray())
	var jobs: PackedByteArray = snap.get("jobs", PackedByteArray())
	var carries: PackedInt32Array = snap.get("carry_counts", PackedInt32Array())
	var n: int = mini(mini(bits.size(), ids.size()), mini(xs.size(), ys.size()))
	_family_frame -= 1
	if _family_frame <= 0:
		_family_frame = FAMILY_REFRESH_FRAMES
		_refresh_households(bits, ids, n)
	var duration: float = _step_seconds()
	var seen: Dictionary = {}
	var inspector: Node = get_node_or_null("/root/Main/UI/AgentInspectorPanel")
	_selected_bits = int(inspector.call("get_selected_entity_bits")) if inspector != null and (inspector as Control).visible else -1
	for i: int in n:
		var key: int = bits[i]
		seen[key] = true
		var tile: Vector2i = Vector2i(xs[i], ys[i])
		var person: Person = _persons.get(key)
		if person == null:
			person = _make_person(key, ids[i], tile)
			_persons[key] = person
		elif tile != person.tile:
			person.from_pos = person.position
			person.to_pos = View.tile_to_world(tile)
			person.tile = tile
			person.elapsed = 0.0
			person.duration = duration
		var stage: int = int(stages[i]) if i < stages.size() else 0
		_set_stage(person, stage)
		var household: int = int(_households.get(ids[i], ids[i]))
		person.tunic.modulate = View.TUNICS[posmod(household, View.TUNICS.size())]
		person.hair.modulate = View.ELDER_HAIR if stage == 2 else View.HAIRS[posmod(ids[i], View.HAIRS.size())]
		person.skin.modulate = View.SKIN
		var job: int = int(jobs[i]) if i < jobs.size() else 0
		person.carry.visible = i < carries.size() and carries[i] > 0
		person.tool.visible = TOOL_FOR_JOB.has(job) and not person.carry.visible
		if person.tool.visible:
			person.tool.region_rect = View.rect(TOOL_FOR_JOB[job])
		person.select.visible = key == _selected_bits
	for key: int in _persons.keys():
		if not seen.has(key):
			(_persons[key] as Person).queue_free()
			_persons.erase(key)
	for person: Person in _persons.values():
		if person.elapsed < person.duration:
			person.elapsed = minf(person.duration, person.elapsed + delta)
			person.position = person.from_pos.lerp(person.to_pos, person.elapsed / person.duration).round()
		else:
			person.position = person.to_pos


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


## How many people are drawn.
func person_count() -> int:
	return _persons.size()
