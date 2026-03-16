extends RefCounted

const GameConfig = preload("res://scripts/core/simulation/game_config.gd")

var current_tick: int = 0
var is_paused: bool = false
var speed_index: int = 0
var rng: RandomNumberGenerator = RandomNumberGenerator.new()

var _accumulator: float = 0.0
var _seed: int = 0
var _rust_runtime_initialized: bool = false
var _rust_runtime_available: bool = false
var _registered_system_count: int = 0
var _last_agent_snapshots: Array = []
var _entity_detail_cache: Dictionary = {}
var _entity_detail_cache_tick: int = -1


## Initialize the engine with a deterministic seed
func init_with_seed(seed_value: int) -> void:
	_seed = seed_value
	rng.seed = seed_value
	current_tick = 0
	_accumulator = 0.0
	_registered_system_count = 0
	_init_rust_runtime()

## Validates that the runtime registry is populated and fully Rust-backed.
func validate_runtime_registry() -> Dictionary:
	var result: Dictionary = {
		"runtime_available": _rust_runtime_available,
		"expected_count": _registered_system_count,
		"runtime_count": 0,
		"count_match": false,
		"all_rust": false,
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
	var all_rust: bool = true
	for row_raw in runtime_snapshot:
		if not (row_raw is Dictionary):
			all_rust = false
			break
		var row: Dictionary = row_raw
		if str(row.get("exec_backend", "")) != "rust" or not bool(row.get("rust_registered", false)):
			all_rust = false
			break
	result["all_rust"] = all_rust
	if bool(result["count_match"]) and bool(result["all_rust"]):
		return result
	push_warning(
		"[SimulationEngine] Runtime registry mismatch expected=%d runtime=%d all_rust=%s" %
		[_registered_system_count, runtime_count, str(result["all_rust"])]
	)
	return result


## Called every frame from Main._process(delta)
func update(delta: float) -> void:
	if not _rust_runtime_available:
		return
	_update_rust_primary(delta, is_paused)
	_flush_llm_debug_log()


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
	_last_agent_snapshots = runtime_state.get("agent_snapshots", [])
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


func get_agent_snapshots() -> Array:
	return _last_agent_snapshots


## Spawns agents into the Rust hecs world at the given positions.
## spawn_list: Array of Dicts with keys: x, y, age_ticks
## settlement_id: ID for the settlement to assign agents to (creates it if missing)
## settlement_x/y: Center of the settlement (used if creating new)
## Returns number of agents spawned.
func spawn_agents(spawn_list: Array, settlement_id: int, settlement_x: int, settlement_y: int) -> int:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("runtime_spawn_agents"):
		return 0
	var data: Array = []
	for sp in spawn_list:
		data.append({
			"x": sp.get("x", 0),
			"y": sp.get("y", 0),
			"age_ticks": sp.get("age_ticks", 0),
			"settlement_id": settlement_id,
			"settlement_x": settlement_x,
			"settlement_y": settlement_y,
		})
	var json_str: String = JSON.stringify(data)
	return int(sim_bridge.call("runtime_spawn_agents", json_str))


## Bootstraps the authoritative Rust world from a setup dictionary.
func bootstrap_world(setup: Dictionary) -> Dictionary:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("runtime_bootstrap_world"):
		return {}
	return sim_bridge.call("runtime_bootstrap_world", JSON.stringify(setup))


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
		if sim_bridge.has_method("runtime_register_default_systems"):
			_registered_system_count = int(sim_bridge.call("runtime_register_default_systems"))
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


## Returns cached entity detail dictionary from SimBridge, invalidating cache each tick.
func get_entity_detail(entity_id: int) -> Dictionary:
	if current_tick != _entity_detail_cache_tick:
		_entity_detail_cache.clear()
		_entity_detail_cache_tick = current_tick
	if _entity_detail_cache.has(entity_id):
		return _entity_detail_cache[entity_id]
	var sim_bridge: Object = _get_sim_bridge()
	var detail: Dictionary = {}
	if sim_bridge != null and sim_bridge.has_method("runtime_get_entity_detail"):
		var raw: Variant = sim_bridge.call("runtime_get_entity_detail", entity_id)
		if raw is Dictionary:
			detail = raw
	_entity_detail_cache[entity_id] = detail
	return detail


## Returns entity tab data for the given tab name from SimBridge (no caching — tab data is large).
func get_entity_tab(entity_id: int, tab: String) -> Dictionary:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("runtime_get_entity_tab"):
		return {}
	var raw: Variant = sim_bridge.call("runtime_get_entity_tab", entity_id, tab)
	if raw is Dictionary:
		return raw
	return {}


## Returns the archetype locale key for the given entity from SimBridge.
func get_archetype_label(entity_id: int) -> String:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("get_archetype_label"):
		return ""
	return str(sim_bridge.call("get_archetype_label", entity_id))


## Returns formatted thought-stream text for the given entity from SimBridge.
func get_thought_text(entity_id: int) -> String:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("get_thought_text"):
		return ""
	return str(sim_bridge.call("get_thought_text", entity_id))


## Returns pre-computed narrative panel display data for the given entity.
func get_narrative_display(entity_id: int) -> Dictionary:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("get_narrative_display"):
		return {}
	var raw: Variant = sim_bridge.call("get_narrative_display", entity_id)
	if raw is Dictionary:
		return raw
	return {}


## Drains queued LLM debug log lines for debug output.
func drain_llm_debug_log() -> Array[String]:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("drain_llm_debug_log"):
		return []
	var raw: Variant = sim_bridge.call("drain_llm_debug_log")
	if not (raw is Array):
		return []
	var lines: Array[String] = []
	for entry in raw:
		lines.append(str(entry))
	return lines


## Notifies Rust that the player opened the narrative panel for the given entity.
func on_entity_narrative_click(entity_id: int) -> int:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("on_entity_narrative_click"):
		return 0
	return int(sim_bridge.call("on_entity_narrative_click", entity_id))


## Sets the AI narration quality tier handled by the Rust runtime.
func set_llm_quality(quality: int) -> void:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("set_llm_quality"):
		return
	sim_bridge.call("set_llm_quality", quality)


## Returns the AI narration quality tier from the Rust runtime.
func get_llm_quality() -> int:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("get_llm_quality"):
		return 0
	return int(sim_bridge.call("get_llm_quality"))


## Returns settlement detail from Rust runtime.
func get_settlement_detail(settlement_id: int) -> Dictionary:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("runtime_get_settlement_detail"):
		return {}
	var raw: Variant = sim_bridge.call("runtime_get_settlement_detail", settlement_id)
	if raw is Dictionary:
		return raw
	return {}


## Returns building detail from Rust runtime.
func get_building_detail(building_id: int) -> Dictionary:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("runtime_get_building_detail"):
		return {}
	var raw: Variant = sim_bridge.call("runtime_get_building_detail", building_id)
	if raw is Dictionary:
		return raw
	return {}


## Returns world summary from Rust runtime.
func get_world_summary() -> Dictionary:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("runtime_get_world_summary"):
		return {}
	var raw: Variant = sim_bridge.call("runtime_get_world_summary")
	if raw is Dictionary:
		return raw
	return {}


func get_band_list() -> Array:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("runtime_get_band_list"):
		return []
	var raw: Variant = sim_bridge.call("runtime_get_band_list")
	if raw is Array:
		return raw
	return []


func get_band_detail(band_id: int) -> Dictionary:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("runtime_get_band_detail"):
		return {}
	var raw: Variant = sim_bridge.call("runtime_get_band_detail", band_id)
	if raw is Dictionary:
		return raw
	return {}


func get_influence_texture(channel: String) -> PackedByteArray:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("runtime_get_influence_texture"):
		return PackedByteArray()
	var raw: Variant = sim_bridge.call("runtime_get_influence_texture", channel)
	if raw is PackedByteArray:
		return raw
	return PackedByteArray()


func get_influence_grid_size() -> Vector2i:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("runtime_get_influence_grid_size"):
		return Vector2i.ZERO
	var raw: Variant = sim_bridge.call("runtime_get_influence_grid_size")
	if raw is Vector2i:
		return raw
	return Vector2i.ZERO


## Returns the number of runtime systems registered in the authoritative Rust manifest.
func get_registered_system_count() -> int:
	return _registered_system_count


## Returns minimap snapshot from Rust runtime.
func get_minimap_snapshot() -> Dictionary:
	var sim_bridge: Object = _get_sim_bridge()
	if sim_bridge == null or not sim_bridge.has_method("runtime_get_minimap_snapshot"):
		return {}
	var raw: Variant = sim_bridge.call("runtime_get_minimap_snapshot")
	if raw is Dictionary:
		return raw
	return {}


func _flush_llm_debug_log() -> void:
	# Keep draining the bridge-side debug ring buffer so it does not retain stale
	# lines between frames, but stop mirroring `[LLM-DEBUG]` traffic to stdout.
	drain_llm_debug_log()


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
