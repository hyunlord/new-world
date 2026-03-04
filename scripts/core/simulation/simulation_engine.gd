extends RefCounted

const GameConfig = preload("res://scripts/core/simulation/game_config.gd")

var current_tick: int = 0
var is_paused: bool = false
var speed_index: int = 0
var rng: RandomNumberGenerator = RandomNumberGenerator.new()

var _accumulator: float = 0.0
var _systems: Array = []
var _seed: int = 0
var _rust_runtime_initialized: bool = false
var _rust_runtime_available: bool = false
var _registered_system_count: int = 0
var _registered_system_payloads: Array[Dictionary] = []
var _system_key_by_instance_id: Dictionary = {}


## Initialize the engine with a deterministic seed
func init_with_seed(seed_value: int) -> void:
	_seed = seed_value
	rng.seed = seed_value
	current_tick = 0
	_accumulator = 0.0
	_registered_system_count = 0
	_registered_system_payloads.clear()
	_system_key_by_instance_id.clear()
	_init_rust_runtime()


## Register a simulation system (sorted by priority)
func register_system(system: RefCounted) -> void:
	var system_payload: Dictionary = _build_runtime_system_payload(system, _registered_system_count)
	_registered_system_count += 1
	_registered_system_payloads.append(system_payload)
	var key: String = _runtime_system_key_from_name(str(system_payload.get("name", "")))
	if not key.is_empty():
		_system_key_by_instance_id[system.get_instance_id()] = key
	if _rust_runtime_available:
		_queue_runtime_command(StringName("register_system"), system_payload)
	_systems.append(system)
	_systems.sort_custom(func(a, b): return a.priority < b.priority)


## Validates Rust runtime registry snapshot against GDScript registration metadata.
func validate_runtime_registry() -> Dictionary:
	var result: Dictionary = {
		"runtime_available": _rust_runtime_available,
		"expected_count": _registered_system_count,
		"runtime_count": 0,
		"count_match": false,
		"order_match": false,
	}
	if not _rust_runtime_available:
		return result
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null:
		return result
	if not sim_bridge.has_method("runtime_get_registry_snapshot"):
		return result
	_apply_runtime_commands_v2()
	var runtime_snapshot: Array = sim_bridge.call("runtime_get_registry_snapshot")
	var runtime_count: int = runtime_snapshot.size()
	result["runtime_count"] = runtime_count
	result["count_match"] = runtime_count == _registered_system_count
	var expected_names: PackedStringArray = _expected_runtime_registry_names()
	var runtime_names: PackedStringArray = _runtime_registry_names(runtime_snapshot)
	result["order_match"] = expected_names == runtime_names
	if bool(result["count_match"]) and bool(result["order_match"]):
		return result
	push_warning(
		"[SimulationEngine] Runtime registry mismatch expected=%d runtime=%d order_match=%s" %
		[_registered_system_count, runtime_count, str(result["order_match"])]
	)
	return result


## Called every frame from Main._process(delta)
func update(delta: float) -> void:
	if not _rust_runtime_available:
		return
	_update_rust_primary(delta, is_paused)


func _update_rust_primary(delta: float, paused: bool) -> void:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null:
		return
	_apply_runtime_commands_v2()
	if not sim_bridge.has_method("runtime_tick_frame"):
		return
	var runtime_state_raw: Variant = sim_bridge.call("runtime_tick_frame", delta, speed_index, paused)
	if not (runtime_state_raw is Dictionary):
		return
	var runtime_state: Dictionary = runtime_state_raw
	current_tick = int(runtime_state.get("current_tick", current_tick))
	_accumulator = float(runtime_state.get("accumulator", _accumulator))
	_consume_runtime_events_v2()


func _consume_runtime_events_v2() -> void:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null:
		return
	if not sim_bridge.has_method("runtime_export_events_v2"):
		return
	var events_raw: Variant = sim_bridge.call("runtime_export_events_v2")
	if not (events_raw is Array):
		return
	var events: Array = events_raw
	if events.is_empty():
		return
	var bus_v2: Object = _get_simulation_bus_v2()
	if bus_v2 == null:
		return
	if not bus_v2.has_method("emit_runtime_event"):
		return
	for i in range(events.size()):
		var event_raw: Variant = events[i]
		if not (event_raw is Dictionary):
			continue
		var event: Dictionary = event_raw
		var event_type_id: int = int(event.get("event_type_id", -1))
		var tick: int = int(event.get("tick", -1))
		var payload_raw: Variant = event.get("payload", {})
		var payload: Dictionary = {}
		if payload_raw is Dictionary:
			payload = payload_raw
		bus_v2.call("emit_runtime_event", event_type_id, payload, tick)


func _apply_runtime_commands_v2() -> void:
	var bus_v2: Object = _get_simulation_bus_v2()
	if bus_v2 == null:
		return
	if not bus_v2.has_method("drain_runtime_commands"):
		return
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null:
		return
	if not sim_bridge.has_method("runtime_apply_commands_v2"):
		return
	var commands_raw: Variant = bus_v2.call("drain_runtime_commands")
	if not (commands_raw is Array):
		return
	var commands: Array = commands_raw
	if commands.is_empty():
		return
	sim_bridge.call("runtime_apply_commands_v2", commands)


## Toggle pause state
func toggle_pause() -> void:
	is_paused = not is_paused
	var simulation_bus: Object = _get_simulation_bus()
	if simulation_bus != null:
		simulation_bus.emit_signal("pause_changed", is_paused)


## Set speed by index
func set_speed(index: int) -> void:
	speed_index = clampi(index, 0, GameConfig.SPEED_OPTIONS.size() - 1)
	_queue_runtime_command(StringName("set_speed_index"), {"speed_index": speed_index})
	var simulation_bus: Object = _get_simulation_bus()
	if simulation_bus != null:
		simulation_bus.emit_signal("speed_changed", speed_index)


## Increase speed to next level
func increase_speed() -> void:
	set_speed(speed_index + 1)


## Decrease speed to previous level
func decrease_speed() -> void:
	set_speed(speed_index - 1)


## Convert current tick to game time
func get_game_time() -> Dictionary:
	var game_config_singleton: Object = _get_game_config_singleton()
	if game_config_singleton != null and game_config_singleton.has_method("tick_to_date"):
		var result_raw: Variant = game_config_singleton.call("tick_to_date", current_tick)
		if result_raw is Dictionary:
			return result_raw
	return {"year": 1, "month": 1, "day": 1, "hour": 0, "minute": 0, "day_of_year": 0, "tick": 0}


## Debug: N 틱 즉시 일괄 처리 (debug build 전용)
## 시뮬레이션을 N tick 빠르게 진행. 화면 갱신 없음.
func advance_ticks(n: int) -> void:
	var tick_duration: float = 1.0 / float(GameConfig.TICKS_PER_SECOND)
	for i in range(n):
		_update_rust_primary(tick_duration, false)


func _init_rust_runtime() -> void:
	_rust_runtime_initialized = false
	_rust_runtime_available = false
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null:
		push_warning("[SimulationEngine] SimBridge autoload missing.")
		return
	if not sim_bridge.has_method("runtime_init"):
		push_warning("[SimulationEngine] runtime_init not found.")
		return
	var config_json: String = _build_runtime_config_json()
	_rust_runtime_initialized = bool(sim_bridge.call("runtime_init", _seed, config_json))
	_rust_runtime_available = _rust_runtime_initialized
	if _rust_runtime_available:
		if sim_bridge.has_method("runtime_clear_registry"):
			sim_bridge.call("runtime_clear_registry")
		return
	push_warning("[SimulationEngine] Rust runtime init failed.")


func _build_runtime_config_json() -> String:
	var config: Dictionary = {
		"world_width": GameConfig.WORLD_SIZE.x,
		"world_height": GameConfig.WORLD_SIZE.y,
		"ticks_per_second": GameConfig.TICKS_PER_SECOND,
		"max_ticks_per_frame": GameConfig.MAX_TICKS_PER_FRAME,
	}
	return JSON.stringify(config)


func _use_rust_primary() -> bool:
	return _rust_runtime_available


func _queue_runtime_command(command_id: StringName, payload: Dictionary) -> void:
	if not _rust_runtime_available:
		return
	var bus_v2: Object = _get_simulation_bus_v2()
	if bus_v2 == null:
		return
	if not bus_v2.has_method("queue_runtime_command"):
		return
	bus_v2.call("queue_runtime_command", command_id, payload)


func _build_runtime_system_payload(system: RefCounted, registration_index: int) -> Dictionary:
	var payload: Dictionary = {}
	var script_name: String = ""
	var script_ref: Variant = system.get_script()
	if script_ref is GDScript:
		script_name = str(script_ref.resource_path)
	if script_name.is_empty():
		script_name = system.get_class()
	payload["name"] = script_name
	payload["priority"] = int(system.get("priority"))
	payload["tick_interval"] = int(system.get("tick_interval"))
	payload["active"] = bool(system.get("is_active"))
	payload["registration_index"] = registration_index
	return payload


func _runtime_system_key_from_name(name: String) -> String:
	var trimmed: String = name.strip_edges()
	if trimmed.is_empty():
		return ""
	var normalized: String = trimmed.replace("\\", "/").to_lower()
	var tail: String = normalized.get_file()
	if tail.ends_with(".gd"):
		tail = tail.left(tail.length() - 3)
	return tail


func _expected_runtime_registry_names() -> PackedStringArray:
	var sorted_payloads: Array = _registered_system_payloads.duplicate(true)
	sorted_payloads.sort_custom(func(a, b):
		var a_priority: int = int(a.get("priority", 100))
		var b_priority: int = int(b.get("priority", 100))
		if a_priority == b_priority:
			return int(a.get("registration_index", 0)) < int(b.get("registration_index", 0))
		return a_priority < b_priority
	)
	var names: PackedStringArray = PackedStringArray()
	for i in range(sorted_payloads.size()):
		var payload_raw: Variant = sorted_payloads[i]
		if not (payload_raw is Dictionary):
			continue
		var payload: Dictionary = payload_raw
		names.append(str(payload.get("name", "")))
	return names


func _runtime_registry_names(runtime_snapshot: Array) -> PackedStringArray:
	var names: PackedStringArray = PackedStringArray()
	for i in range(runtime_snapshot.size()):
		var row_raw: Variant = runtime_snapshot[i]
		if not (row_raw is Dictionary):
			continue
		var row: Dictionary = row_raw
		names.append(str(row.get("name", "")))
	return names


func _get_sim_bridge() -> Object:
	if Engine.has_singleton("SimBridge"):
		var bridge_singleton: Object = Engine.get_singleton("SimBridge")
		if bridge_singleton != null:
			return bridge_singleton
	var main_loop: MainLoop = Engine.get_main_loop()
	if main_loop == null:
		return null
	if not (main_loop is SceneTree):
		return null
	var tree: SceneTree = main_loop
	if tree.root == null:
		return null
	return tree.root.get_node_or_null("SimBridge")


func _get_game_config_singleton() -> Object:
	if Engine.has_singleton("GameConfig"):
		var config_singleton: Object = Engine.get_singleton("GameConfig")
		if config_singleton != null:
			return config_singleton
	return null


func _get_simulation_bus() -> Object:
	if Engine.has_singleton("SimulationBus"):
		var bus_singleton: Object = Engine.get_singleton("SimulationBus")
		if bus_singleton != null:
			return bus_singleton
	var main_loop: MainLoop = Engine.get_main_loop()
	if main_loop == null:
		return null
	if not (main_loop is SceneTree):
		return null
	var tree: SceneTree = main_loop
	if tree.root == null:
		return null
	return tree.root.get_node_or_null("SimulationBus")


func _get_simulation_bus_v2() -> Object:
	var main_loop: MainLoop = Engine.get_main_loop()
	if main_loop == null:
		return null
	if not (main_loop is SceneTree):
		return null
	var tree: SceneTree = main_loop
	if tree.root == null:
		return null
	return tree.root.get_node_or_null("SimulationBusV2")
