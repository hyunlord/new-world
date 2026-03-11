extends SceneTree

const SimBridgeScript = preload("res://scripts/core/simulation/sim_bridge.gd")
const SnapshotDecoderClass = preload("res://scripts/rendering/snapshot_decoder.gd")
const TEST_SEED: int = 20260307
const TEST_WORLD_SIZE: int = 64
const TEST_AGENT_COUNT: int = 20

var passed: int = 0
var failed: int = 0
var errors: Array[String] = []
var _decoder := SnapshotDecoderClass.new()


func _init() -> void:
	call_deferred("_run")


func _run() -> void:
	print("=== WorldSim Stage 1 Headless Test Suite ===")
	print("")
	var boot_ok: bool = await _boot_main_scene()
	if boot_ok:
		_run_all_tests()
	_print_results()
	quit(0 if failed == 0 else 1)


func _boot_main_scene() -> bool:
	var bridge: Node = _ensure_sim_bridge()
	if bridge == null:
		return _record_boot_failure("SimBridge node is missing")
	var initialized: bool = bool(bridge.call("runtime_init", TEST_SEED, "{}"))
	if not initialized:
		return _record_boot_failure("runtime_init returned false")
	if bridge.has_method("runtime_register_default_systems"):
		bridge.call("runtime_register_default_systems")
	var payload_json: String = JSON.stringify(_build_bootstrap_payload())
	var bootstrap_result: Dictionary = bridge.call("runtime_bootstrap_world", payload_json)
	if bootstrap_result.is_empty():
		return _record_boot_failure("runtime_bootstrap_world returned an empty result")
	var ready: bool = await _wait_for_runtime_ready(8)
	if not ready:
		return _record_boot_failure("Runtime did not report initialized state after bootstrap")
	return true


func _record_boot_failure(message: String) -> bool:
	failed += 1
	errors.append(message)
	push_error(message)
	print("  [FAIL] %s" % message)
	return false


func _run_all_tests() -> void:
	_run_test("SimBridge initialized", _test_sim_bridge_initialized)
	_run_test("Agents spawned (population > 0)", _test_agents_spawned)
	_run_test("FrameSnapshot returns PackedByteArray", _test_snapshot_returns_packed_bytes)
	_run_test("FrameSnapshot size = 36 × agent_count", _test_snapshot_size_matches_agent_count)
	_run_test("FrameSnapshot decode produces valid data", _test_snapshot_decode_is_valid)
	_run_test("Render alpha returns 0~1", _test_render_alpha_range)
	_run_test("Agents move after 60 ticks", _test_agents_move_after_sixty_ticks)
	_run_test("Velocity is non-zero for moving agents", _test_velocity_non_zero_for_some_agent)
	_run_test("drain_notifications returns Array", _test_notifications_returns_array)
	_run_test("Notification format has required keys", _test_notifications_have_required_keys)
	_run_test("get_archetype_label returns non-empty string", _test_archetype_label_non_empty)
	_run_test("get_thought_text returns non-empty string", _test_thought_text_non_empty)
	_run_test("60 ticks complete under 1 second", _test_tick_budget_under_one_second)


func _run_test(name: String, test_callable: Callable) -> void:
	print("  [RUN] %s" % name)
	var ok: bool = bool(test_callable.call())
	if ok:
		passed += 1
		print("  [PASS] %s" % name)
	else:
		failed += 1
		print("  [FAIL] %s" % name)


func _test_sim_bridge_initialized() -> bool:
	var bridge: Node = _get_sim_bridge()
	if bridge == null:
		return _fail("SimBridge node should exist")
	return _check(
		bool(bridge.call("runtime_is_initialized")),
		"Runtime should be initialized"
	)


func _test_agents_spawned() -> bool:
	var bridge: Node = _get_sim_bridge()
	if bridge == null:
		return _fail("SimBridge node should exist")
	var count: int = int(bridge.call("get_agent_count"))
	return _check(count > 0, "Agent count should be > 0, got %d" % count)


func _test_snapshot_returns_packed_bytes() -> bool:
	_tick_runtime(1)
	var bridge: Node = _get_sim_bridge()
	if bridge == null:
		return _fail("SimBridge node should exist")
	var bytes: PackedByteArray = bridge.call("get_frame_snapshots")
	if not _check(bytes is PackedByteArray, "Frame snapshot should be PackedByteArray"):
		return false
	return _check(bytes.size() > 0, "Frame snapshot should not be empty")


func _test_snapshot_size_matches_agent_count() -> bool:
	_tick_runtime(1)
	var bridge: Node = _get_sim_bridge()
	if bridge == null:
		return _fail("SimBridge node should exist")
	var bytes: PackedByteArray = bridge.call("get_frame_snapshots")
	var count: int = int(bridge.call("get_agent_count"))
	return _check(
		bytes.size() == count * SnapshotDecoderClass.AGENT_SIZE,
		"Snapshot size should be %d, got %d" % [count * SnapshotDecoderClass.AGENT_SIZE, bytes.size()]
	)


func _test_snapshot_decode_is_valid() -> bool:
	_tick_runtime(1)
	if not _refresh_decoder():
		return false
	if not _check(_decoder.has_data(), "Snapshot decoder should have data"):
		return false
	var entity_id: int = _decoder.get_entity_id(0)
	var position: Vector2 = _decoder.get_interpolated_position(0, 0.5)
	var mood: int = _decoder.get_mood_color(0)
	var stress: int = _decoder.get_stress_phase(0)
	if not _check(entity_id >= 0, "entity_id should be non-negative, got %d" % entity_id):
		return false
	if not _check(position.x >= 0.0 and position.x <= 10000.0, "x should be in range, got %f" % position.x):
		return false
	if not _check(position.y >= 0.0 and position.y <= 10000.0, "y should be in range, got %f" % position.y):
		return false
	if not _check(mood >= 0 and mood <= 4, "mood_color should be 0~4, got %d" % mood):
		return false
	return _check(stress >= 0 and stress <= 4, "stress_phase should be 0~4, got %d" % stress)


func _test_render_alpha_range() -> bool:
	var bridge: Node = _get_sim_bridge()
	if bridge == null:
		return _fail("SimBridge node should exist")
	var alpha: float = float(bridge.call("get_render_alpha"))
	return _check(alpha >= 0.0 and alpha <= 1.0, "Render alpha should be 0~1, got %f" % alpha)


func _test_agents_move_after_sixty_ticks() -> bool:
	_tick_runtime(1)
	var bridge: Node = _get_sim_bridge()
	if bridge == null:
		return _fail("SimBridge node should exist")
	var before: PackedByteArray = bridge.call("get_frame_snapshots")
	if before.size() < SnapshotDecoderClass.AGENT_SIZE:
		return _fail("Need at least one snapshot record before movement test")
	var before_x: float = before.decode_float(SnapshotDecoderClass.OFF_X)
	var before_y: float = before.decode_float(SnapshotDecoderClass.OFF_Y)
	_tick_runtime(60)
	var after: PackedByteArray = bridge.call("get_frame_snapshots")
	if after.size() < SnapshotDecoderClass.AGENT_SIZE:
		return _fail("Need at least one snapshot record after movement test")
	var after_x: float = after.decode_float(SnapshotDecoderClass.OFF_X)
	var after_y: float = after.decode_float(SnapshotDecoderClass.OFF_Y)
	var delta: float = absf(after_x - before_x) + absf(after_y - before_y)
	return _check(delta > 0.05, "Agent should move after 60 ticks, delta=%f" % delta)


func _test_velocity_non_zero_for_some_agent() -> bool:
	_tick_runtime(10)
	if not _refresh_decoder():
		return false
	for index: int in range(_decoder.agent_count):
		var velocity: Vector2 = _decoder.get_velocity(index)
		if absf(velocity.x) + absf(velocity.y) > 0.01:
			return true
	return _fail("At least one agent should have non-zero velocity after 10 ticks")


func _test_notifications_returns_array() -> bool:
	_tick_runtime(200)
	var bridge: Node = _get_sim_bridge()
	if bridge == null:
		return _fail("SimBridge node should exist")
	var notifications: Array = bridge.call("drain_notifications")
	return _check(notifications is Array, "drain_notifications should return Array")


func _test_notifications_have_required_keys() -> bool:
	_tick_runtime(500)
	var bridge: Node = _get_sim_bridge()
	if bridge == null:
		return _fail("SimBridge node should exist")
	var notifications: Array = bridge.call("drain_notifications")
	if notifications.is_empty():
		print("  [SKIP] No notifications generated in 500 ticks (may be normal)")
		return true
	var notification_data: Dictionary = notifications[0]
	var required_keys: Array[String] = [
		"tick",
		"tier",
		"kind",
		"message",
		"primary_entity",
		"position_x",
	]
	for key: String in required_keys:
		if not _check(notification_data.has(key), "Notification missing key '%s'" % key):
			return false
	var tier: int = int(notification_data.get("tier", -1))
	return _check(tier >= 0 and tier <= 3, "Notification tier should be 0~3, got %d" % tier)


func _test_archetype_label_non_empty() -> bool:
	_tick_runtime(1)
	var bridge: Node = _get_sim_bridge()
	if bridge == null:
		return _fail("SimBridge node should exist")
	var bytes: PackedByteArray = bridge.call("get_frame_snapshots")
	if bytes.size() < SnapshotDecoderClass.AGENT_SIZE:
		return _fail("Need at least one snapshot to fetch archetype label")
	var entity_id: int = bytes.decode_u32(SnapshotDecoderClass.OFF_ENTITY_ID)
	var label: String = str(bridge.call("get_archetype_label", entity_id))
	return _check(label.length() > 0, "Archetype label should not be empty")


func _test_thought_text_non_empty() -> bool:
	_tick_runtime(10)
	var bridge: Node = _get_sim_bridge()
	if bridge == null:
		return _fail("SimBridge node should exist")
	var bytes: PackedByteArray = bridge.call("get_frame_snapshots")
	if bytes.size() < SnapshotDecoderClass.AGENT_SIZE:
		return _fail("Need at least one snapshot to fetch thought text")
	var entity_id: int = bytes.decode_u32(SnapshotDecoderClass.OFF_ENTITY_ID)
	var text: String = str(bridge.call("get_thought_text", entity_id))
	return _check(text.length() > 0, "Thought text should not be empty")


func _test_tick_budget_under_one_second() -> bool:
	var bridge: Node = _get_sim_bridge()
	if bridge == null:
		return _fail("SimBridge node should exist")
	var started_usec: int = Time.get_ticks_usec()
	_tick_runtime(60)
	var elapsed_ms: float = float(Time.get_ticks_usec() - started_usec) / 1000.0
	print("  [PERF] 60 ticks in %.1fms (%.2fms/tick)" % [elapsed_ms, elapsed_ms / 60.0])
	return _check(elapsed_ms < 1000.0, "60 ticks should take < 1s, took %fms" % elapsed_ms)


func _tick_runtime(steps: int, delta_sec: float = 0.1, speed_index: int = 0, paused: bool = false) -> void:
	var bridge: Node = _get_sim_bridge()
	if bridge == null:
		return
	for _step: int in range(steps):
		bridge.call("runtime_tick_frame", delta_sec, speed_index, paused)


func _refresh_decoder() -> bool:
	var bridge: Node = _get_sim_bridge()
	if bridge == null:
		return _fail("SimBridge node should exist")
	var curr_bytes: PackedByteArray = bridge.call("get_frame_snapshots")
	var prev_bytes: PackedByteArray = bridge.call("get_prev_frame_snapshots")
	var agent_count: int = int(bridge.call("get_agent_count"))
	_decoder.update(curr_bytes, prev_bytes, agent_count)
	return true


func _wait_for_runtime_ready(max_frames: int) -> bool:
	for _frame_index: int in range(max_frames):
		var bridge: Node = _get_sim_bridge()
		if bridge != null:
			var initialized: bool = bool(bridge.call("runtime_is_initialized"))
			var count: int = int(bridge.call("get_agent_count"))
			if initialized and count > 0:
				return true
		await process_frame
	return false


func _get_sim_bridge() -> Node:
	return root.get_node_or_null("SimBridge")


func _ensure_sim_bridge() -> Node:
	var bridge: Node = _get_sim_bridge()
	if bridge != null:
		return bridge
	var sim_bridge_node: Node = SimBridgeScript.new()
	sim_bridge_node.name = "SimBridge"
	root.add_child(sim_bridge_node)
	return sim_bridge_node


func _build_bootstrap_payload() -> Dictionary:
	var tile_count: int = TEST_WORLD_SIZE * TEST_WORLD_SIZE
	var biomes: Array = []
	var elevation: Array = []
	var moisture: Array = []
	var temperature: Array = []
	var food: Array = []
	var wood: Array = []
	var stone: Array = []
	biomes.resize(tile_count)
	elevation.resize(tile_count)
	moisture.resize(tile_count)
	temperature.resize(tile_count)
	food.resize(tile_count)
	wood.resize(tile_count)
	stone.resize(tile_count)
	for index: int in range(tile_count):
		biomes[index] = 3
		elevation[index] = 0.5
		moisture[index] = 0.5
		temperature[index] = 0.5
		food[index] = 18.0
		wood[index] = 12.0
		stone[index] = 10.0
	var agents: Array = []
	var center: int = TEST_WORLD_SIZE / 2
	for agent_index: int in range(TEST_AGENT_COUNT):
		var offset_x: int = (agent_index % 5) - 2
		var offset_y: int = int(agent_index / 5) - 2
		agents.append({
			"x": center + offset_x,
			"y": center + offset_y,
			"age_ticks": int((15 + (agent_index % 10)) * 4380),
		})
	return {
		"world": {
			"width": TEST_WORLD_SIZE,
			"height": TEST_WORLD_SIZE,
			"biomes": biomes,
			"elevation": elevation,
			"moisture": moisture,
			"temperature": temperature,
			"food": food,
			"wood": wood,
			"stone": stone,
		},
		"founding_settlement": {
			"id": 1,
			"name": "Test Hold",
			"x": center,
			"y": center,
			"stockpile_food": 20.0,
			"stockpile_wood": 10.0,
			"stockpile_stone": 8.0,
		},
		"agents": agents,
	}

func _check(condition: bool, message: String) -> bool:
	if condition:
		return true
	errors.append(message)
	push_error(message)
	return false


func _fail(message: String) -> bool:
	return _check(false, message)


func _print_results() -> void:
	print("")
	print("=== RESULTS ===")
	print("Passed: %d" % passed)
	print("Failed: %d" % failed)
	if not errors.is_empty():
		print("")
		print("Failures:")
		for message: String in errors:
			print(message)
	print("===============")
	print("")
