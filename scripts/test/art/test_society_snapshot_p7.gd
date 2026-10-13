extends SceneTree

const SEED: int = 20261013
var _world: Node
var _failures: int = 0

func _initialize() -> void:
	call_deferred("_run")

func _check(condition: bool, message: String) -> void:
	if not condition:
		_failures += 1
		push_error(message)

func _run() -> void:
	_world = ClassDB.instantiate("WorldSimNode")
	root.add_child(_world)
	_world.set_process(false)
	if not _world.has_method("get_society_snapshot"):
		push_error("Missing real get_society_snapshot FFI")
		_world.free()
		quit(1)
		return
	_world.reset_world(SEED)
	await process_frame
	await process_frame
	_world.advance_ticks(1440)
	_verify_snapshot()
	var snapshot: Dictionary = _world.get_society_snapshot()
	_check(snapshot.settlement_ids.size() == 1, "Built v17 world forms one original settlement")
	_check(snapshot.home_agent_ids.size() > 0, "Built world has joined roster members")
	_check(snapshot.parent_ids[0] == -1, "Original has no parent")
	_check(snapshot == _world.get_society_snapshot(), "Read-only getter leaves state unchanged")
	if OS.get_environment("SOCIETY_LONG_TEST") == "1":
		_world.advance_ticks(40230 - int(snapshot.current_tick))
		_verify_snapshot()
		snapshot = _world.get_society_snapshot()
		_check(snapshot.founder_agent_ids.size() == 13, "Natural departure contains 13 founders")
		_check(snapshot.fission_ids == PackedInt32Array([1]), "First natural fission recorded")
		_check(snapshot.fission_depart_ticks == PackedInt64Array([40200]), "Measured departure tick")
		_check(snapshot.fission_sizes == PackedInt32Array([13]), "Natural group size in log")
		_check(snapshot.fission_daughters == PackedInt32Array([-1]), "Walking group has no daughter yet")
		_check(snapshot.fission_founded_ticks == PackedInt64Array([-1]), "Walking group has no founding tick")
		_check(snapshot.founder_leader.count(1) == 1, "One leader in the walking group")
		for agent_id: int in snapshot.founder_agent_ids:
			var index: int = snapshot.home_agent_ids.find(agent_id)
			_check(index >= 0 and snapshot.home_settlement_ids[index] == 1, "Founder belongs to parent while walking")
		_world.advance_ticks(100)
		_verify_snapshot()
		snapshot = _world.get_society_snapshot()
		_check(snapshot.settlement_ids == PackedInt32Array([1, 2]), "Natural founding adds daughter")
		_check(snapshot.parent_ids == PackedInt32Array([-1, 1]), "Daughter identifies original parent")
		_check(snapshot.founder_agent_ids.is_empty(), "Founding removes walking markers")
		_check(snapshot.fission_founded_ticks == PackedInt64Array([40329]), "Measured founding tick")
		_check(snapshot.fission_daughters == PackedInt32Array([2]), "Log retains daughter")
		_check(snapshot.fission_cancelled_ticks == PackedInt64Array([-1]), "Natural fission not cancelled")
		print("SOCIETY NATURAL FISSION: seed=20261013 depart=40200 found=40329 size=13")
	_world.reset_world(SEED)
	await process_frame
	await process_frame
	snapshot = _world.get_society_snapshot()
	_check(snapshot.current_tick == 0, "Reset clears time")
	_check(snapshot.fission_ids.is_empty() and snapshot.founder_agent_ids.is_empty(), "Reset clears fission state")
	_check(snapshot.retired_hearth_xs.is_empty(), "Reset clears retired hearths")
	_verify_snapshot()
	_world.free()
	print("SOCIETY SNAPSHOT P7: failures=%d long=%s" % [_failures, OS.get_environment("SOCIETY_LONG_TEST")])
	quit(0 if _failures == 0 else 1)

func _verify_snapshot() -> void:
	var society: Dictionary = _world.get_society_snapshot()
	var settlements: Dictionary = _world.get_settlement_snapshot()
	var agents: Dictionary = _world.get_agent_snapshot()
	var groups: Array = [
		["settlement_ids", "parent_ids", "founded_ticks", "member_counts"],
		["home_agent_ids", "home_settlement_ids"],
		["founder_agent_ids", "founder_fission_ids", "founder_site_xs", "founder_site_ys", "founder_leader", "founder_since_ticks"],
		["fission_ids", "fission_parents", "fission_site_xs", "fission_site_ys", "fission_sizes", "fission_depart_ticks", "fission_founded_ticks", "fission_cancelled_ticks", "fission_daughters"],
		["retired_hearth_xs", "retired_hearth_ys"],
	]
	var wide_keys: Array[String] = ["founded_ticks", "home_agent_ids", "founder_agent_ids", "founder_since_ticks", "fission_depart_ticks", "fission_founded_ticks", "fission_cancelled_ticks"]
	for group: Array in groups:
		for key: String in group:
			var expected_type: int = TYPE_PACKED_INT64_ARRAY if key in wide_keys else TYPE_PACKED_INT32_ARRAY
			if key == "founder_leader":
				expected_type = TYPE_PACKED_BYTE_ARRAY
			_check(typeof(society[key]) == expected_type, "Packed type: " + key)
			_check(society[key].size() == society[group[0]].size(), "Parallel length: " + key)
	for key: String in ["settlement_ids", "home_agent_ids", "founder_agent_ids", "fission_ids"]:
		for index: int in range(1, society[key].size()):
			_check(society[key][index - 1] < society[key][index], "Strictly sorted identities: " + key)
	_check(society.settlement_ids == settlements.settlement_ids, "Filter and order match settlement getter")
	for index: int in society.home_agent_ids.size():
		_check(agents.agent_ids.has(society.home_agent_ids[index]), "Home identity joins agent_ids")
	for index: int in society.settlement_ids.size():
		_check(society.member_counts[index] == society.home_settlement_ids.count(society.settlement_ids[index]), "Member count includes walking founders")
	_check(typeof(society.current_tick) == TYPE_INT, "Current tick is integer")
