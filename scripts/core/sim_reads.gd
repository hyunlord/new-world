# SimReads (node "SimReads" beside WorldSim in scenes/main.tscn): the read layer; every snapshot read (GETTERS) and
#   mutation (MUTATORS) goes through it, frozen and cached per frame (porter/relations and the M4 knowledge reads per
#   tick, terrain until reset; M5 get_story, get_animals, get_drafted per tick; M5.5 get_neighbours per frame).
# Bridge methods: get_agent_snapshot, get_settlement_snapshot, get_construction_snapshot, get_building_snapshot,
#   get_resource_snapshot, get_time_snapshot, get_tallies, get_recent_deaths, get_society_snapshot, get_world_info,
#   get_terrain_snapshot, get_activity_snapshot, get_porter_snapshot, get_settlement_relations, get_colony_versions,
#   get_trees, get_colony_overlay, get_workshops, get_work_priorities, get_recipes, get_space, get_rooms, get_tech,
#   get_teach_orders, get_knowledge_waits, get_story, get_animals, get_drafted, get_neighbours (GETTERS);
#   get_tile_causal_events;
#   advance_ticks, reset_world, reset_world_stress, on_building_placed, cmd_designate_chop, cmd_designate_chop_tiles,
#   cmd_cancel_area, cmd_add_stockpile, cmd_remove_zone_area, cmd_place_blueprint, cmd_designate_quarry,
#   cmd_add_bill, cmd_remove_bill, cmd_set_bill_mode, cmd_move_bill, cmd_set_bill_suspended,
#   cmd_set_work_priority, cmd_place_line, cmd_place_rect, cmd_designate_dig, cmd_designate_deconstruct,
#   cmd_set_room_role, cmd_designate_clay, cmd_order_teach, cmd_cancel_teach, cmd_draft, cmd_undraft_all,
#   cmd_draft_move, cmd_draft_attack, cmd_draft_attack_person, cmd_designate_hunt, cmd_add_hunt_zone, cmd_cancel_hunt,
#   cmd_set_dress_order, cmd_accept_migrants, cmd_decline_migrants, cmd_trade, save_world, load_world (MUTATORS);
#   signal world_reset.
class_name SimReads
extends Node

## Emitted only after the explicit new-game command succeeds, never after loading a save.
## Successful replacement receipt before the mutator returns; native world_reset follows deferred.
signal world_replaced
signal new_game_started

const GETTERS: Array[String] = [
	"get_content_catalog",
	"get_agent_snapshot", "get_settlement_snapshot", "get_construction_snapshot",
	"get_building_snapshot", "get_resource_snapshot", "get_time_snapshot",
	"get_tallies", "get_recent_deaths", "get_society_snapshot",
	"get_world_info", "get_terrain_snapshot",
	"get_activity_snapshot", "get_porter_snapshot", "get_settlement_relations",
	"get_colony_versions", "get_trees", "get_colony_overlay",
	# M2: workshops with bills and shelf, the adults' work priorities, the recipe table.
	"get_workshops", "get_work_priorities", "get_recipes",
	# M3: built space (walls, doors, floors, bedrolls, dug tiles) and rooms.
	"get_space", "get_rooms",
	# M4: the settlement's tech map and era, open apprenticeship orders, work waiting for a practitioner.
	"get_tech", "get_teach_orders", "get_knowledge_waits",
	# M5: the storyteller's pressures and banner numbers; the animals and hunt zones; the drafted people.
	"get_story", "get_animals", "get_drafted",
	# M5.5: the two neighbours, their migrant waves and caravans, the eureka clues and the tallies.
	"get_neighbours", "get_ui_facts", "get_steward",
]
## Read once per simulation tick (M4: the knowledge reads change only when a tick runs or a command is sent).
const TICK_GETTERS: Array[String] = ["get_porter_snapshot", "get_settlement_relations", "get_tech", "get_teach_orders", "get_knowledge_waits",
	"get_story", "get_animals", "get_drafted", "get_ui_facts", "get_steward"]
const MUTATORS: Array[String] = [
	"advance_ticks", "reset_world", "reset_world_stress",
	"on_building_placed",
	# M1 orders (ADR 0012): each returns {accepted, count, reason}.
	"cmd_designate_chop", "cmd_designate_chop_tiles", "cmd_cancel_area",
	"cmd_add_stockpile", "cmd_remove_zone_area", "cmd_place_blueprint",
	# M2 orders (quarry, bills, work priorities): each returns {accepted, count, reason}.
	"cmd_designate_quarry", "cmd_add_bill", "cmd_remove_bill", "cmd_set_bill_mode", "cmd_move_bill",
	"cmd_set_bill_suspended", "cmd_set_work_priority",
	# M3 orders (walls/doors/floors/bedrolls by line or rectangle, dig, deconstruct, room roles).
	"cmd_place_line", "cmd_place_rect", "cmd_designate_dig", "cmd_designate_deconstruct", "cmd_set_room_role",
	# M4 orders (clay pits, apprenticeship).
	"cmd_designate_clay", "cmd_order_teach", "cmd_cancel_teach",
	# M5 orders (draft, hunt, the standing dress order).
	"cmd_draft", "cmd_undraft_all", "cmd_draft_move", "cmd_draft_attack", "cmd_draft_attack_person",
	"cmd_designate_hunt", "cmd_add_hunt_zone", "cmd_cancel_hunt", "cmd_set_dress_order",
	# M5.5 orders (migrant waves, barter with a caravan).
	"cmd_accept_migrants", "cmd_decline_migrants", "cmd_trade",
	# M1.5 save files: save_world(path) -> bool, load_world(path) -> bool (a load replaces the world).
	"save_world", "load_world", "cmd_set_steward", "cmd_set_steward_defence",
]
var _world: Node
var _cache: Dictionary = {}
var _tick_cache: Dictionary = {}
var _tick: int = -1
var _catalog: Dictionary = {}
var _terrain: Dictionary = {}
var _terrain_valid: bool = false
var _frame: int = -1
var _counts: Dictionary = {}
var _usec: Dictionary = {}
var _mux2_profile: bool = OS.get_environment("WORLDSIM_MUX2_PROFILE") == "1"
var _freeze_usec: Dictionary = {}

static func resolve(world: Node) -> Node:
	if not is_instance_valid(world) or world.get_parent() == null:
		return null
	var candidate: Node = world.get_parent().get_node_or_null("SimReads")
	if candidate is SimReads and candidate.owns_world(world):
		return candidate
	return null

static func read(world: Node, raw_getter: String) -> Dictionary:
	if raw_getter not in GETTERS:
		push_error("SimReads unsupported getter: " + raw_getter)
		return {}
	var reader: Node = resolve(world)
	if reader != null:
		return reader.read_raw(raw_getter)
	if not is_instance_valid(world) or (raw_getter in TICK_GETTERS and not world.has_method(raw_getter)):
		return {}
	var result: Variant = world.call(raw_getter)
	return result if result is Dictionary else {}

static func causal_events(world: Node, xs: PackedInt32Array, ys: PackedInt32Array, kinds: PackedStringArray) -> Array:
	var reader: Node = resolve(world)
	if reader != null:
		return reader.causal_events_raw(xs, ys, kinds)
	if not is_instance_valid(world):
		return []
	var result: Variant = world.call("get_tile_causal_events", xs, ys, kinds)
	return result if result is Array else []

static func mutate(world: Node, method: String, arguments: Array = []) -> Variant:
	if method not in MUTATORS:
		push_error("SimReads unsupported mutation: " + method)
		return null
	var reader: Node = resolve(world)
	if reader != null:
		return reader.mutate_raw(method, arguments)
	if not is_instance_valid(world):
		return null
	return world.callv(method, arguments)

func _ready() -> void:
	_sync_world()

func owns_world(world: Node) -> bool:
	_sync_world()
	return is_instance_valid(world) and world == _world

func _sync_world() -> void:
	var current: Node = null
	if get_parent() != null:
		current = get_parent().get_node_or_null("WorldSim")
	if is_instance_valid(_world) and current == _world:
		return
	if is_instance_valid(_world) and _world.has_signal("world_reset"):
		if _world.is_connected("world_reset", _on_world_reset):
			_world.disconnect("world_reset", _on_world_reset)
	_world = current
	_on_world_reset()
	if is_instance_valid(_world) and _world.has_signal("world_reset"):
		_world.connect("world_reset", _on_world_reset)

func _sync_frame() -> void:
	var current: int = Engine.get_process_frames()
	if current != _frame:
		_frame = current
		_cache.clear()
		_counts.clear()
		_usec.clear()
		_freeze_usec.clear()

func invalidate() -> void:
	_cache.clear()
	_tick_cache.clear()
	_tick = -1

func _on_world_reset() -> void:
	invalidate()
	_terrain = {}
	_terrain_valid = false

func read_raw(raw_getter: String) -> Dictionary:
	if raw_getter not in GETTERS:
		push_error("SimReads unsupported getter: " + raw_getter)
		return {}
	_sync_world()
	_sync_frame()
	if not is_instance_valid(_world) or (raw_getter in TICK_GETTERS and not _world.has_method(raw_getter)):
		return {}
	if not (_world is WorldSimNode):
		var fake_result: Variant = _world.call(raw_getter)
		return fake_result if fake_result is Dictionary else {}
	if raw_getter in TICK_GETTERS:
		var tick: int = int(read_raw("get_time_snapshot").get("tick", -1))
		if tick != _tick:
			_tick = tick
			_tick_cache.clear()
		if _tick_cache.has(raw_getter):
			return _tick_cache[raw_getter]
	if raw_getter == "get_content_catalog" and not _catalog.is_empty():
		return _catalog
	if raw_getter == "get_terrain_snapshot" and _terrain_valid:
		return _terrain
	if _cache.has(raw_getter):
		return _cache[raw_getter]
	var started: int = Time.get_ticks_usec()
	var result: Variant = _world.call(raw_getter)
	var elapsed: int = Time.get_ticks_usec() - started
	_counts[raw_getter] = int(_counts.get(raw_getter, 0)) + 1
	_usec[raw_getter] = int(_usec.get(raw_getter, 0)) + elapsed
	var snapshot: Dictionary = result if result is Dictionary else {}
	var freeze_started: int = Time.get_ticks_usec() if _mux2_profile else 0
	_freeze(snapshot)
	if _mux2_profile: _freeze_usec[raw_getter] = int(_freeze_usec.get(raw_getter, 0)) + Time.get_ticks_usec() - freeze_started
	if raw_getter == "get_content_catalog":
		_catalog = snapshot
	elif raw_getter == "get_terrain_snapshot":
		_terrain = snapshot
		_terrain_valid = true
	elif raw_getter in TICK_GETTERS:
		_tick_cache[raw_getter] = snapshot
	else:
		_cache[raw_getter] = snapshot
	return snapshot

func peek_uncounted(raw_getter: String) -> Dictionary:
	if raw_getter not in GETTERS:
		push_error("SimReads unsupported getter: " + raw_getter)
		return {}
	_sync_world()
	if not is_instance_valid(_world) or (raw_getter in TICK_GETTERS and not _world.has_method(raw_getter)):
		return {}
	var result: Variant = _world.call(raw_getter)
	var snapshot: Dictionary = result if result is Dictionary else {}
	if _world is WorldSimNode:
		_freeze(snapshot)
	return snapshot

func causal_events_raw(xs: PackedInt32Array, ys: PackedInt32Array, kinds: PackedStringArray) -> Array:
	_sync_world()
	_sync_frame()
	if not is_instance_valid(_world):
		return []
	var started: int = Time.get_ticks_usec()
	var result: Variant = _world.call("get_tile_causal_events", xs, ys, kinds)
	var elapsed: int = Time.get_ticks_usec() - started
	if _world is WorldSimNode:
		_counts["get_tile_causal_events"] = int(_counts.get("get_tile_causal_events", 0)) + 1
		_usec["get_tile_causal_events"] = int(_usec.get("get_tile_causal_events", 0)) + elapsed
	return result if result is Array else []

func mutate_raw(method: String, arguments: Array = []) -> Variant:
	if method not in MUTATORS:
		push_error("SimReads unsupported mutation: " + method)
		return null
	_sync_world()
	if not is_instance_valid(_world):
		return null
	var result: Variant = _world.callv(method, arguments)
	invalidate()
	# Rust schedules these signals deferred; same-frame reads need fresh terrain now.
	if method in ["reset_world", "reset_world_stress", "load_world"] and bool(result):
		_on_world_reset()
		world_replaced.emit()
	if method == "reset_world" and bool(result):
		new_game_started.emit()
	return result

func frame_call_counts() -> Dictionary:
	_sync_frame()
	return _counts.duplicate()

func frame_call_usec() -> Dictionary:
	_sync_frame()
	return _usec.duplicate()

func mux2_freeze_timings() -> Dictionary:
	if not _mux2_profile: return {"status": "DISABLED"}
	_sync_frame()
	return {"status": "MEASURED", "frame": _frame, "usec": _freeze_usec.duplicate()}

func frame_id() -> int:
	_sync_frame()
	return _frame

static func _freeze(value: Variant) -> void:
	if value is Dictionary:
		for key: Variant in value:
			_freeze(value[key])
		value.make_read_only()
	elif value is Array:
		for child: Variant in value:
			_freeze(child)
		value.make_read_only()

func agent_snapshot() -> Dictionary:
	return read_raw("get_agent_snapshot")

func settlement_snapshot() -> Dictionary:
	return read_raw("get_settlement_snapshot")

func construction_snapshot() -> Dictionary:
	return read_raw("get_construction_snapshot")

func building_snapshot() -> Dictionary:
	return read_raw("get_building_snapshot")

func resource_snapshot() -> Dictionary:
	return read_raw("get_resource_snapshot")

func time_snapshot() -> Dictionary:
	return read_raw("get_time_snapshot")

func tallies() -> Dictionary:
	return read_raw("get_tallies")

func recent_deaths() -> Dictionary:
	return read_raw("get_recent_deaths")

func society_snapshot() -> Dictionary:
	return read_raw("get_society_snapshot")

func world_info() -> Dictionary:
	return read_raw("get_world_info")

func terrain_snapshot() -> Dictionary:
	return read_raw("get_terrain_snapshot")

func advance_ticks(n: int) -> int:
	return int(mutate_raw("advance_ticks", [n]))

func reset_world(seed: int) -> bool:
	return bool(mutate_raw("reset_world", [seed]))

func reset_world_stress(seed: int, agents: int) -> bool:
	return bool(mutate_raw("reset_world_stress", [seed, agents]))

func on_building_placed(x: int, y: int, radius: int) -> bool:
	return bool(mutate_raw("on_building_placed", [x, y, radius]))
