extends RefCounted

const RuntimeShadowReporter = preload("res://scripts/core/simulation/runtime_shadow_reporter.gd")

var current_tick: int = 0
var is_paused: bool = false
var speed_index: int = 0
var rng: RandomNumberGenerator = RandomNumberGenerator.new()

var _accumulator: float = 0.0
var _systems: Array = []
var _seed: int = 0
var _runtime_mode: String = GameConfig.SIM_RUNTIME_MODE_GDSCRIPT
var _rust_runtime_initialized: bool = false
var _rust_runtime_available: bool = false
var _shadow_mismatch_count: int = 0
var _last_gd_ticks_processed: int = 0
var _shadow_reporter: RefCounted = null
var _registered_system_count: int = 0
var _registered_system_payloads: Array[Dictionary] = []


## Initialize the engine with a deterministic seed
func init_with_seed(seed_value: int) -> void:
	_seed = seed_value
	rng.seed = seed_value
	current_tick = 0
	_accumulator = 0.0
	_registered_system_count = 0
	_registered_system_payloads.clear()
	_init_rust_runtime()


## Register a simulation system (sorted by priority)
func register_system(system: RefCounted) -> void:
	var system_payload: Dictionary = _build_runtime_system_payload(system)
	_registered_system_count += 1
	_registered_system_payloads.append(system_payload)
	if _rust_runtime_available:
		_queue_runtime_command(StringName("register_system"), system_payload)
	if _use_rust_primary():
		return
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
	if SimBridge == null:
		return result
	if not SimBridge.has_method("runtime_get_registry_snapshot"):
		return result
	_apply_runtime_commands_v2()
	var runtime_snapshot: Array = SimBridge.runtime_get_registry_snapshot()
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
	if _use_rust_primary():
		_update_rust_primary(delta, is_paused)
		return
	if is_paused:
		if _use_rust_shadow():
			_last_gd_ticks_processed = 0
			_run_shadow_runtime(0.0, true)
		return

	_update_gdscript(delta)
	if _use_rust_shadow():
		_run_shadow_runtime(delta, false)


func _update_gdscript(delta: float) -> void:
	_last_gd_ticks_processed = 0
	var tick_duration: float = 1.0 / GameConfig.TICKS_PER_SECOND
	var speed: int = GameConfig.SPEED_OPTIONS[speed_index]
	_accumulator += delta * speed
	var ticks_this_frame: int = 0
	while _accumulator >= tick_duration and ticks_this_frame < GameConfig.MAX_TICKS_PER_FRAME:
		_process_tick()
		_accumulator -= tick_duration
		ticks_this_frame += 1
	_last_gd_ticks_processed = ticks_this_frame
	# Prevent spiral of death
	if _accumulator > tick_duration * 3.0:
		_accumulator = 0.0


func _update_rust_primary(delta: float, paused: bool) -> void:
	_apply_runtime_commands_v2()
	var runtime_state: Dictionary = SimBridge.runtime_tick_frame(delta, speed_index, paused)
	current_tick = int(runtime_state.get("current_tick", current_tick))
	_accumulator = float(runtime_state.get("accumulator", _accumulator))
	_consume_runtime_events_v2()


func _run_shadow_runtime(delta: float, paused: bool = false) -> void:
	_apply_runtime_commands_v2()
	var runtime_state: Dictionary = SimBridge.runtime_tick_frame(delta, speed_index, paused)
	var shadow_tick: int = int(runtime_state.get("current_tick", current_tick))
	# Shadow mode: drain v2 events so runtime buffer does not grow,
	# but do not forward them to avoid duplicate v1/v2 emissions.
	var shadow_events: Array = SimBridge.runtime_export_events_v2()
	var shadow_event_count: int = shadow_events.size()
	if _shadow_reporter != null and _shadow_reporter.has_method("record_frame"):
		_shadow_reporter.call(
			"record_frame",
			current_tick,
			current_tick,
			shadow_tick,
			_last_gd_ticks_processed,
			shadow_event_count
		)
		_try_shadow_auto_cutover()
	if shadow_tick == current_tick:
		return
	_shadow_mismatch_count += 1
	if _shadow_mismatch_count <= 5 or _shadow_mismatch_count % 100 == 0:
		push_warning(
			"[SimulationEngine] Rust shadow mismatch gd_tick=%d rust_tick=%d gd_events=%d rust_events=%d (count=%d)" %
			[current_tick, shadow_tick, _last_gd_ticks_processed, shadow_event_count, _shadow_mismatch_count]
		)


func _consume_runtime_events_v2() -> void:
	var events: Array = SimBridge.runtime_export_events_v2()
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
	if not SimBridge.has_method("runtime_apply_commands_v2"):
		return
	var commands_raw: Variant = bus_v2.call("drain_runtime_commands")
	if not (commands_raw is Array):
		return
	var commands: Array = commands_raw
	if commands.is_empty():
		return
	SimBridge.runtime_apply_commands_v2(commands)


func _process_tick() -> void:
	current_tick += 1
	for i in range(_systems.size()):
		var system = _systems[i]
		if system.is_active and current_tick % system.tick_interval == 0:
			system.execute_tick(current_tick)
	SimulationBus.tick_completed.emit(current_tick)


## Toggle pause state
func toggle_pause() -> void:
	is_paused = not is_paused
	if _use_rust_primary():
		return
	SimulationBus.pause_changed.emit(is_paused)


## Set speed by index
func set_speed(index: int) -> void:
	speed_index = clampi(index, 0, GameConfig.SPEED_OPTIONS.size() - 1)
	_queue_runtime_command(StringName("set_speed_index"), {"speed_index": speed_index})
	if _use_rust_primary():
		return
	SimulationBus.speed_changed.emit(speed_index)


## Increase speed to next level
func increase_speed() -> void:
	set_speed(speed_index + 1)


## Decrease speed to previous level
func decrease_speed() -> void:
	set_speed(speed_index - 1)


## Convert current tick to game time
func get_game_time() -> Dictionary:
	return GameConfig.tick_to_date(current_tick)


## Debug: N 틱 즉시 일괄 처리 (debug build 전용)
## 시뮬레이션을 N tick 빠르게 진행. 화면 갱신 없음.
func advance_ticks(n: int) -> void:
	if _use_rust_primary():
		var tick_duration: float = 1.0 / float(GameConfig.TICKS_PER_SECOND)
		for i in range(n):
			_update_rust_primary(tick_duration, false)
		return
	for i in range(n):
		_process_tick()


func _init_rust_runtime() -> void:
	_runtime_mode = str(GameConfig.SIM_RUNTIME_MODE)
	_rust_runtime_initialized = false
	_rust_runtime_available = false
	_shadow_mismatch_count = 0
	_last_gd_ticks_processed = 0
	_shadow_reporter = null
	if _runtime_mode == GameConfig.SIM_RUNTIME_MODE_GDSCRIPT:
		return
	if SimBridge == null:
		push_warning("[SimulationEngine] SimBridge autoload missing. Falling back to GDScript runtime.")
		_runtime_mode = GameConfig.SIM_RUNTIME_MODE_GDSCRIPT
		return
	if not SimBridge.has_method("runtime_init"):
		push_warning("[SimulationEngine] runtime_init not found. Falling back to GDScript runtime.")
		_runtime_mode = GameConfig.SIM_RUNTIME_MODE_GDSCRIPT
		return

	var config_json: String = _build_runtime_config_json()
	_rust_runtime_initialized = bool(SimBridge.runtime_init(_seed, config_json))
	_rust_runtime_available = _rust_runtime_initialized
	if _rust_runtime_available:
		if SimBridge.has_method("runtime_clear_registry"):
			SimBridge.runtime_clear_registry()
		if _use_rust_shadow():
			_shadow_reporter = RuntimeShadowReporter.new()
			_shadow_reporter.call(
				"setup",
				GameConfig.RUST_SHADOW_REPORT_PATH,
				GameConfig.RUST_SHADOW_REPORT_INTERVAL_TICKS,
				GameConfig.RUST_SHADOW_ALLOWED_MAX_TICK_DELTA,
				GameConfig.RUST_SHADOW_ALLOWED_MAX_EVENT_DELTA,
				GameConfig.RUST_SHADOW_ALLOWED_MISMATCH_RATIO
			)
		return
	push_warning("[SimulationEngine] Rust runtime init failed. Falling back to GDScript runtime.")
	_runtime_mode = GameConfig.SIM_RUNTIME_MODE_GDSCRIPT


func _build_runtime_config_json() -> String:
	var config: Dictionary = {
		"world_width": GameConfig.WORLD_SIZE.x,
		"world_height": GameConfig.WORLD_SIZE.y,
		"ticks_per_second": GameConfig.TICKS_PER_SECOND,
		"max_ticks_per_frame": GameConfig.MAX_TICKS_PER_FRAME,
	}
	return JSON.stringify(config)


func _use_rust_primary() -> bool:
	return _rust_runtime_available and _runtime_mode == GameConfig.SIM_RUNTIME_MODE_RUST_PRIMARY


func _use_rust_shadow() -> bool:
	return _rust_runtime_available and _runtime_mode == GameConfig.SIM_RUNTIME_MODE_RUST_SHADOW


func _queue_runtime_command(command_id: StringName, payload: Dictionary) -> void:
	if not _rust_runtime_available:
		return
	var bus_v2: Object = _get_simulation_bus_v2()
	if bus_v2 == null:
		return
	if not bus_v2.has_method("queue_runtime_command"):
		return
	bus_v2.call("queue_runtime_command", command_id, payload)


func _build_runtime_system_payload(system: RefCounted) -> Dictionary:
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
	return payload


func _try_shadow_auto_cutover() -> void:
	if not GameConfig.RUST_SHADOW_AUTO_CUTOVER_ENABLED:
		return
	if _runtime_mode != GameConfig.SIM_RUNTIME_MODE_RUST_SHADOW:
		return
	if _shadow_reporter == null:
		return
	if not _shadow_reporter.has_method("is_approved_for_cutover"):
		return
	var approved: bool = bool(_shadow_reporter.call("is_approved_for_cutover"))
	if not approved:
		return
	_runtime_mode = GameConfig.SIM_RUNTIME_MODE_RUST_PRIMARY
	push_warning("[SimulationEngine] Shadow cutover approved. Switching runtime mode to rust_primary.")


func _expected_runtime_registry_names() -> PackedStringArray:
	var sorted_payloads: Array = _registered_system_payloads.duplicate(true)
	sorted_payloads.sort_custom(func(a, b): return int(a.get("priority", 100)) < int(b.get("priority", 100)))
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
