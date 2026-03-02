extends RefCounted

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


## Initialize the engine with a deterministic seed
func init_with_seed(seed_value: int) -> void:
	_seed = seed_value
	rng.seed = seed_value
	current_tick = 0
	_accumulator = 0.0
	_init_rust_runtime()


## Register a simulation system (sorted by priority)
func register_system(system: RefCounted) -> void:
	if _use_rust_primary():
		return
	_systems.append(system)
	_systems.sort_custom(func(a, b): return a.priority < b.priority)


## Called every frame from Main._process(delta)
func update(delta: float) -> void:
	if is_paused:
		return
	if _use_rust_primary():
		_update_rust_primary(delta)
		return

	_update_gdscript(delta)
	if _use_rust_shadow():
		_run_shadow_runtime(delta)


func _update_gdscript(delta: float) -> void:
	var tick_duration: float = 1.0 / GameConfig.TICKS_PER_SECOND
	var speed: int = GameConfig.SPEED_OPTIONS[speed_index]
	_accumulator += delta * speed
	var ticks_this_frame: int = 0
	while _accumulator >= tick_duration and ticks_this_frame < GameConfig.MAX_TICKS_PER_FRAME:
		_process_tick()
		_accumulator -= tick_duration
		ticks_this_frame += 1
	# Prevent spiral of death
	if _accumulator > tick_duration * 3.0:
		_accumulator = 0.0


func _update_rust_primary(delta: float) -> void:
	var runtime_state: Dictionary = SimBridge.runtime_tick_frame(delta, speed_index, false)
	current_tick = int(runtime_state.get("current_tick", current_tick))
	_accumulator = float(runtime_state.get("accumulator", _accumulator))
	_consume_runtime_events_v2()


func _run_shadow_runtime(delta: float) -> void:
	var runtime_state: Dictionary = SimBridge.runtime_tick_frame(delta, speed_index, false)
	var shadow_tick: int = int(runtime_state.get("current_tick", current_tick))
	# Shadow mode: drain v2 events so runtime buffer does not grow,
	# but do not forward them to avoid duplicate v1/v2 emissions.
	SimBridge.runtime_export_events_v2()
	if shadow_tick == current_tick:
		return
	_shadow_mismatch_count += 1
	if _shadow_mismatch_count <= 5 or _shadow_mismatch_count % 100 == 0:
		push_warning(
			"[SimulationEngine] Rust shadow tick mismatch gd=%d rust=%d (count=%d)" %
			[current_tick, shadow_tick, _shadow_mismatch_count]
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
	SimulationBus.pause_changed.emit(is_paused)


## Set speed by index
func set_speed(index: int) -> void:
	speed_index = clampi(index, 0, GameConfig.SPEED_OPTIONS.size() - 1)
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
			_update_rust_primary(tick_duration)
		return
	for i in range(n):
		_process_tick()


func _init_rust_runtime() -> void:
	_runtime_mode = str(GameConfig.SIM_RUNTIME_MODE)
	_rust_runtime_initialized = false
	_rust_runtime_available = false
	_shadow_mismatch_count = 0
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
