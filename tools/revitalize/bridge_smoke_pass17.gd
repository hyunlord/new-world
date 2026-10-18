extends SceneTree

const MAX_DAYS: int = 10
const ACTIVITY_KEYS: Array[String] = ["ids", "kinds", "place_x", "place_y", "until_tick"]
const DETAIL_KEYS: Array[String] = [
	"found", "agent_id", "x", "y", "state_tag", "hunger", "thirst", "sleep", "target_kind",
]
const FAMILY_KEYS: Array[String] = ["found", "female", "mother", "father", "partner", "children"]
const SNAPSHOT_KEYS: Array[String] = [
	"ids", "xs", "ys", "states", "agent_ids", "seek_kinds", "target_xs", "target_ys",
	"hungers", "thirsts", "sleeps", "carried_foods", "carried_woods", "target_kinds",
	"life_stages", "ages_days", "colds",
]

var _world: Node = null
var _require_routine: bool = false
var _require_family: bool = false
var _family_rows: int = 0
var _initial_agent_ids: Dictionary[int, bool] = {}
var _pair_sample: Dictionary = {}
var _birth_sample: Dictionary = {}
var _passed: int = 0
var _failures: Array[String] = []
var _tick: int = 0
var _samples: int = 0
var _active_rows: int = 0
var _stale_rows: int = 0
var _stale_rechecks: int = 0
var _first_active_tick: int = -1


func _init() -> void:
	for argument: String in OS.get_cmdline_user_args():
		if argument == "--routine":
			_require_routine = true
		elif argument == "--family":
			_require_family = true
		else:
			_check(false, "unsupported argument: " + argument)
			_finish(2)
			return
	call_deferred("_run")


func _run() -> void:
	if not _check(ClassDB.class_exists("WorldSimNode"), "WorldSimNode registered"):
		_finish(1)
		return
	_world = ClassDB.instantiate("WorldSimNode") as Node
	if not _check(_world != null, "WorldSimNode instantiates as Node"):
		_finish(1)
		return
	_world.process_mode = Node.PROCESS_MODE_DISABLED
	root.add_child(_world)
	for method: String in [
		"get_activity_snapshot", "get_agent_snapshot", "get_agent_detail",
		"get_world_info", "get_time_snapshot", "advance_ticks",
	]:
		if not _check(_world.has_method(method), "method exists: " + method):
			_finish(1)
			return
	if _require_family:
		if not _check(_world.has_method("get_agent_family"), "family getter exists"):
			_finish(1)
			return
		if _read_family(0, false).is_empty():
			_finish(1)
			return
	var info_value: Variant = _world.call("get_world_info")
	var time_value: Variant = _world.call("get_time_snapshot")
	if not _check(info_value is Dictionary and time_value is Dictionary, "world and time dictionaries"):
		_finish(1)
		return
	var info: Dictionary = info_value
	var time: Dictionary = time_value
	var ticks_per_day: int = int(info.get("ticks_per_day", 0))
	_tick = int(time.get("tick", -1))
	if not _check(ticks_per_day > 0 and ticks_per_day % 24 == 0 and _tick == 0, "hourly clock at initial tick zero"):
		_finish(1)
		return
	if not _check_detail(0, false, false) or not _sample_and_recheck_stale():
		_finish(1)
		return
	if not _required_observations_found():
		var ticks_per_hour: int = int(float(ticks_per_day) / 24.0)
		for hour: int in range(1, MAX_DAYS * 24 + 1):
			if not _advance(hour * ticks_per_hour - _tick) or not _sample_and_recheck_stale():
				_finish(1)
				return
			if _required_observations_found():
				break
	if _require_routine:
		_check(_active_rows > 0, "nonempty activity joins state 4 and target 8 within ten days")
	if _require_family:
		_check(_family_rows > 0, "family rows observed")
		_check(not _pair_sample.is_empty(), "mutual pair observed within ten days")
		_check(not _birth_sample.is_empty(), "newborn parent links observed within ten days")
	_finish(0 if _failures.is_empty() else 1)


func _sample_and_recheck_stale() -> bool:
	var stale: Array[int] = _sample()
	if not _failures.is_empty():
		return false
	if not stale.is_empty():
		# Combat may leave Activity until the next decision tick removes it.
		_stale_rechecks += 1
		if not _advance(1):
			return false
		var after: Array[int] = _sample()
		for entity_bits: int in stale:
			_check(not after.has(entity_bits), "stale activity resolved next tick: %d" % entity_bits)
	return _failures.is_empty()


func _sample() -> Array[int]:
	_samples += 1
	var stale: Array[int] = []
	var value: Variant = _world.call("get_activity_snapshot")
	var agents_value: Variant = _world.call("get_agent_snapshot")
	if not _check(value is Dictionary and agents_value is Dictionary, "snapshot dictionaries"):
		return stale
	var snapshot: Dictionary = value
	var agents: Dictionary = agents_value
	if not _exact_keys(agents, SNAPSHOT_KEYS, "agent snapshot"):
		return stale
	if not _exact_keys(snapshot, ACTIVITY_KEYS, "activity"):
		return stale
	if not _check(
		snapshot["ids"] is PackedInt64Array and snapshot["kinds"] is PackedByteArray
		and snapshot["place_x"] is PackedInt32Array and snapshot["place_y"] is PackedInt32Array
		and snapshot["until_tick"] is PackedInt64Array, "activity packed array types"
	):
		return stale
	var ids: PackedInt64Array = snapshot["ids"]
	var kinds: PackedByteArray = snapshot["kinds"]
	var place_x: PackedInt32Array = snapshot["place_x"]
	var place_y: PackedInt32Array = snapshot["place_y"]
	var until_tick: PackedInt64Array = snapshot["until_tick"]
	if not _check(
		ids.size() == kinds.size() and ids.size() == place_x.size()
		and ids.size() == place_y.size() and ids.size() == until_tick.size(), "activity parallel lengths"
	):
		return stale
	if not _check(
		agents.get("ids") is PackedInt64Array and agents.get("states") is PackedByteArray
		and agents.get("target_kinds") is PackedByteArray, "agent snapshot join array types"
	):
		return stale
	var agent_ids: PackedInt64Array = agents["ids"]
	var states: PackedByteArray = agents["states"]
	var targets: PackedByteArray = agents["target_kinds"]
	if not _check(agent_ids.size() == states.size() and agent_ids.size() == targets.size(), "agent join lengths"):
		return stale
	if not _check(not agent_ids.is_empty(), "live agents available for detail contract"):
		return stale
	if not _check_detail(agent_ids[0], true, false):
		return stale
	if _require_family and not _check_families(agents):
		return stale
	var agent_index: Dictionary[int, int] = {}
	for index: int in range(agent_ids.size()):
		agent_index[agent_ids[index]] = index
	var seen: Dictionary[int, bool] = {}
	for index: int in range(ids.size()):
		var entity_bits: int = ids[index]
		if not _check(agent_index.has(entity_bits) and not seen.has(entity_bits), "unique activity id joins live snapshot"):
			return stale
		seen[entity_bits] = true
		if not _check(kinds[index] <= 6 and place_x[index] >= 0 and place_y[index] >= 0 and until_tick[index] >= -1, "activity value domains"):
			return stale
		var agent_row: int = agent_index[entity_bits]
		if states[agent_row] == 4:
			if not _check(targets[agent_row] == 8, "activity state 4 joins target 8"):
				return stale
			if not _check_detail(entity_bits, true, true):
				return stale
			_active_rows += 1
			if _first_active_tick < 0:
				_first_active_tick = _tick
		else:
			stale.append(entity_bits)
			_stale_rows += 1
	return stale


func _check_detail(entity_bits: int, expected_found: bool, activity: bool) -> bool:
	var value: Variant = _world.call("get_agent_detail", entity_bits)
	if not _check(value is Dictionary, "detail dictionary"):
		return false
	var detail: Dictionary = value
	if not _exact_keys(detail, DETAIL_KEYS, "detail"):
		return false
	if not _check(detail["found"] is bool and bool(detail["found"]) == expected_found, "detail found sentinel"):
		return false
	if activity:
		return _check(int(detail["state_tag"]) == 4 and int(detail["target_kind"]) == 8, "activity detail state 4 target 8")
	return true


func _required_observations_found() -> bool:
	return (not _require_routine or _active_rows > 0) and (
		not _require_family or (not _pair_sample.is_empty() and not _birth_sample.is_empty())
	)


func _read_family(entity_bits: int, expected_found: bool) -> Dictionary:
	var value: Variant = _world.call("get_agent_family", entity_bits)
	if not _check(value is Dictionary, "family dictionary"):
		return {}
	var family: Dictionary = value
	if not _exact_keys(family, FAMILY_KEYS, "family"):
		return {}
	if not _check(
		family["found"] is bool and family["female"] is bool
		and family["mother"] is int and family["father"] is int and family["partner"] is int
		and family["children"] is PackedInt64Array, "family value types"
	):
		return {}
	if not _check(bool(family["found"]) == expected_found, "family found sentinel"):
		return {}
	if not _check(
		int(family["mother"]) >= -1 and int(family["father"]) >= -1
		and int(family["partner"]) >= -1, "family link sentinel domains"
	):
		return {}
	if not expected_found:
		var children: PackedInt64Array = family["children"]
		if not _check(
			not bool(family["female"]) and int(family["mother"]) == -1
			and int(family["father"]) == -1 and int(family["partner"]) == -1
			and children.is_empty(), "missing family sentinel values"
		):
			return {}
	return family


func _check_families(agents: Dictionary) -> bool:
	if not _check(agents["agent_ids"] is PackedInt64Array, "stable AgentId array type"):
		return false
	var entity_bits: PackedInt64Array = agents["ids"]
	var identities: PackedInt64Array = agents["agent_ids"]
	if not _check(identities.size() == entity_bits.size(), "family identity join lengths"):
		return false
	var families: Dictionary[int, Dictionary] = {}
	var entities: Dictionary[int, int] = {}
	var initial_sample: bool = _initial_agent_ids.is_empty()
	for index: int in range(identities.size()):
		var identity: int = identities[index]
		if not _check(identity >= 0 and not families.has(identity), "unique stable AgentId"):
			return false
		var family: Dictionary = _read_family(entity_bits[index], true)
		if family.is_empty():
			return false
		families[identity] = family
		entities[identity] = entity_bits[index]
		_family_rows += 1
		if initial_sample:
			_initial_agent_ids[identity] = true
	for identity: int in families:
		var family: Dictionary = families[identity]
		var children: PackedInt64Array = family["children"]
		var previous: int = -1
		for child_id: int in children:
			if not _check(child_id > previous and families.has(child_id), "children sorted unique living AgentIds"):
				return false
			var child: Dictionary = families[child_id]
			if not _check(int(child["mother"]) == identity or int(child["father"]) == identity, "child references this parent AgentId"):
				return false
			previous = child_id
		if not _check_family_links(identity, families, entities):
			return false
	return true


func _check_family_links(identity: int, families: Dictionary[int, Dictionary], entities: Dictionary[int, int]) -> bool:
	var family: Dictionary = families[identity]
	var partner_id: int = int(family["partner"])
	if families.has(partner_id):
		var partner: Dictionary = families[partner_id]
		if not _check(
			partner_id != identity and int(partner["partner"]) == identity
			and bool(family["female"]) != bool(partner["female"]), "living partners are mutual female-male AgentId links"
		):
			return false
		if _pair_sample.is_empty():
			_pair_sample = {"tick": _tick, "people": [
				_family_evidence(identity, family, entities),
				_family_evidence(partner_id, partner, entities),
			]}
	var living_parents: Array[Dictionary] = []
	for parent_key: String in ["mother", "father"]:
		var parent_id: int = int(family[parent_key])
		if families.has(parent_id):
			var parent: Dictionary = families[parent_id]
			var children: PackedInt64Array = parent["children"]
			if not _check(children.has(identity), "living parent lists this child's AgentId"):
				return false
			living_parents.append(_family_evidence(parent_id, parent, entities))
	if _birth_sample.is_empty() and not _initial_agent_ids.has(identity) and living_parents.size() == 2:
		_birth_sample = {"tick": _tick, "newborn": _family_evidence(identity, family, entities), "parents": living_parents}
	return true


func _family_evidence(identity: int, family: Dictionary, entities: Dictionary[int, int]) -> Dictionary:
	return {
		"entity_bits": entities[identity], "agent_id": identity, "female": family["female"],
		"mother": family["mother"], "father": family["father"], "partner": family["partner"],
		"children": family["children"],
	}


func _advance(count: int) -> bool:
	var value: Variant = _world.call("advance_ticks", count)
	if not _check(value is int and int(value) == _tick + count, "advance_ticks exact clock"):
		return false
	_tick = int(value)
	return true


func _exact_keys(value: Dictionary, expected: Array[String], label: String) -> bool:
	var exact: bool = value.size() == expected.size()
	for key: String in expected:
		exact = exact and value.has(key)
	return _check(exact, "%s exact %d keys" % [label, expected.size()])


func _check(condition: bool, label: String) -> bool:
	if condition:
		_passed += 1
	else:
		_failures.append("tick %d: %s" % [_tick, label])
	return condition


func _finish(exit_code: int) -> void:
	print(JSON.stringify({
		"suite": "pass17_bridge", "ok": exit_code == 0, "routine_required": _require_routine,
		"family_required": _require_family, "family_rows": _family_rows,
		"family_pair_sample": _pair_sample, "family_birth_sample": _birth_sample,
		"family_id_contract": "input entity_bits joins ids; links are AgentId and join agent_ids",
		"passed": _passed, "failures": _failures, "tick": _tick, "samples": _samples,
		"active_joined_rows": _active_rows, "first_active_tick": _first_active_tick,
		"stale_rows": _stale_rows, "stale_rechecks": _stale_rechecks,
		"stale_policy": "removed or active again after one tick",
	}))
	if is_instance_valid(_world):
		_world.free()
	quit(exit_code)
