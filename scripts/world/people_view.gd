# people_view.gd (Main/PeopleView in scenes/main.tscn): what the views show about people beyond their bodies, shared by
#   every view (M3D-SWITCH E1).
# Purpose: the mood overlay flag (M toggles it; `mood_toggled` for the story banner's legend) and the chronicle's
#   highlighted people (AgentIds; `highlights_changed` — the 3D People layer rings them in HIGHLIGHT_RGBA). Static data
#   helpers read straight from get_agent_snapshot rows: a person's look ({look, variant, frames}, the inspector's
#   portrait) and the household keys and their ring colour (the minimap's dots).
# Bridge methods: none directly (static helpers take a get_agent_snapshot dictionary).
extends Node

const ArtData = preload("res://scripts/world/art/art_data.gd")
const View = preload("res://scripts/world/colony/view.gd")
const MOOD_KEY: Key = KEY_M
## Snapshot `life_stages` code of a child (bridge contract); the salt of the household colour hash.
const LIFE_CHILD: int = 1
const HOUSEHOLD_SALT: int = 11
## The chronicle highlight ring.
const HIGHLIGHT_RGBA: Color = Color(0.4, 0.85, 0.95, 0.8)

signal mood_toggled(on: bool)
signal highlights_changed

var _mood_shown: bool = false
var _highlight_ids: PackedInt64Array = PackedInt64Array()


func _ready() -> void:
	var world_sim: Node = get_node_or_null("/root/Main/WorldSim")
	if world_sim != null:
		world_sim.connect("world_reset", clear_highlights)


func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and (event as InputEventKey).keycode == MOOD_KEY:
		set_mood_shown(not _mood_shown)
		get_viewport().set_input_as_handled()


## M5: show or hide the mood overlay (M).
func set_mood_shown(on: bool) -> void:
	if on == _mood_shown:
		return
	_mood_shown = on
	mood_toggled.emit(on)


func mood_shown() -> bool:
	return _mood_shown


## M-READ: highlight people by AgentId (chronicle group click).
func set_highlights(ids: PackedInt64Array) -> void:
	_highlight_ids = ids
	highlights_changed.emit()


## M-READ: clear all highlights.
func clear_highlights() -> void:
	_highlight_ids = PackedInt64Array()
	highlights_changed.emit()


func highlight_ids() -> PackedInt64Array:
	return _highlight_ids


## The look of the person with entity bits `bits` in agent snapshot `snap`: {look, variant, frames}, or {} when absent.
## Rows without appearance columns fall back to a look and variant keyed by AgentId.
static func person_look(snap: Dictionary, bits: int) -> Dictionary:
	var i: int = (snap.get("ids", PackedInt64Array()) as PackedInt64Array).find(bits)
	var aids: PackedInt64Array = snap.get("agent_ids", PackedInt64Array())
	if i < 0 or i >= aids.size():
		return {}
	var looks: PackedInt32Array = snap.get("look_ids", PackedInt32Array())
	var variants: PackedByteArray = snap.get("variants", PackedByteArray())
	var look: int = looks[i] if i < looks.size() else posmod(aids[i], maxi(1, ArtData.look_count()))
	var variant: int = int(variants[i]) if i < variants.size() else posmod(aids[i], 6)
	return {"look": look, "variant": variant, "frames": str(ArtData.look(look).get("frames", ""))}


## The ring colour of household key `key` from `palette` (RGB 0–255 rows); alpha 0 (the ink ring) without either.
static func ring_colour(key: int, palette: Array) -> Color:
	if key < 0 or palette.is_empty():
		return Color(0, 0, 0, 0)
	var row: Variant = palette[View.key_hash(key, 0, 0, HOUSEHOLD_SALT) % palette.size()]
	if not (row is Array and (row as Array).size() == 3):
		return Color(0, 0, 0, 0)
	return Color8(int(row[0]), int(row[1]), int(row[2]))


## The household key of every person in a snapshot (AgentId → key): a household is a couple and the children of its
## members — an adult's key is the lower AgentId of themselves and a living partner, a child's is their living
## mother's household, else their living father's, else their own. Pure (tests).
static func household_keys(ids: PackedInt64Array, stages: PackedByteArray, mothers: PackedInt64Array, fathers: PackedInt64Array, partners: PackedInt64Array) -> Dictionary:
	var alive: Dictionary = {}
	for id: int in ids:
		alive[id] = true
	var adult: Dictionary = {}
	for i: int in ids.size():
		var p: int = partners[i] if i < partners.size() else -1
		adult[ids[i]] = mini(ids[i], p) if p >= 0 and alive.has(p) else ids[i]
	var out: Dictionary = {}
	for i: int in ids.size():
		var key: int = adult[ids[i]]
		if i < stages.size() and stages[i] == LIFE_CHILD:
			var m: int = mothers[i] if i < mothers.size() else -1
			var f: int = fathers[i] if i < fathers.size() else -1
			key = adult[m] if alive.has(m) else (adult[f] if alive.has(f) else ids[i])
		out[ids[i]] = key
	return out
