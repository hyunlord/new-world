extends Node

const _NATIVE_SINGLETON_CANDIDATES: Array[String] = [
	"WorldSimBridge",
	"SimBridgeNative",
	"RustBridge",
]
const _PATHFIND_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid",
	"pathfind",
]
const _PATHFIND_GPU_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_gpu",
]
const _PATHFIND_XY_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_xy",
]
const _PATHFIND_XY_GPU_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_gpu_xy",
]
const _PATHFIND_BATCH_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_batch",
]
const _PATHFIND_BATCH_GPU_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_gpu_batch",
]
const _PATHFIND_BATCH_XY_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_batch_xy",
]
const _PATHFIND_BATCH_XY_GPU_METHOD_CANDIDATES: Array[String] = [
	"pathfind_grid_gpu_batch_xy",
]
const _SET_PATHFIND_BACKEND_METHOD_CANDIDATES: Array[String] = [
	"set_pathfinding_backend",
]
const _GET_PATHFIND_BACKEND_METHOD_CANDIDATES: Array[String] = [
	"get_pathfinding_backend",
]
const _RESOLVE_PATHFIND_BACKEND_METHOD_CANDIDATES: Array[String] = [
	"resolve_pathfinding_backend",
]
const _RUNTIME_CLASS_CANDIDATES: Array[String] = [
	"WorldSimRuntime",
]
const _GDEXTENSION_PATH: String = "res://rust/worldsim.gdextension"
const _RUNTIME_REQUIRED_METHODS: Array[String] = [
	"runtime_bootstrap_world",
	"runtime_get_chronicle_feed",
	"runtime_get_chronicle_entry_detail",
	"runtime_get_story_threads",
	"runtime_get_history_slice",
	"runtime_get_recall_slice",
	"runtime_get_world_summary",
	"runtime_get_settlement_detail",
	"runtime_get_minimap_snapshot",
	"get_frame_snapshots",
	"get_prev_frame_snapshots",
	"get_render_alpha",
	"get_agent_count",
	"drain_notifications",
	"get_archetype_label",
	"get_thought_text",
	"get_narrative_display",
	"on_entity_narrative_click",
	"set_llm_quality",
	"get_llm_quality",
]

var _native_checked: bool = false
var _native_bridge: Object = null
var _native_runtime_checked: bool = false
var _native_runtime: Object = null
var _gdextension_checked: bool = false
var _pathfind_method_name: String = ""
var _pathfind_xy_method_name: String = ""
var _pathfind_batch_method_name: String = ""
var _pathfind_batch_xy_method_name: String = ""
var _set_pathfind_backend_method_name: String = ""
var _get_pathfind_backend_method_name: String = ""
var _resolve_pathfind_backend_method_name: String = ""
var _last_synced_pathfind_backend_mode: String = ""
var _resolved_pathfind_backend_cache: String = ""
var _resolved_pathfind_backend_cached: bool = false
var _gpu_pathfinding_capability_cached: bool = false
var _gpu_pathfinding_capability: bool = false
var _runtime_recovery_attempted: bool = false


## Initializes Rust runtime coordinator.
## Returns true when runtime instance is available and initialized.
func runtime_init(p_seed: int, config_json: String) -> bool:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return false
	if not runtime.has_method("runtime_init"):
		return false
	return bool(runtime.call("runtime_init", p_seed, config_json))


## Returns true when Rust runtime instance currently has initialized state.
func runtime_is_initialized() -> bool:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return false
	if not runtime.has_method("runtime_is_initialized"):
		return false
	return bool(runtime.call("runtime_is_initialized"))


## Ticks Rust runtime with frame delta.
## Returns state dictionary:
## { initialized, current_tick, ticks_processed, speed_index, paused, accumulator }.
func runtime_tick_frame(delta_sec: float, speed_index: int, paused: bool) -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {"initialized": false}
	if not runtime.has_method("runtime_tick_frame"):
		return {"initialized": false}
	var result: Variant = runtime.call("runtime_tick_frame", delta_sec, speed_index, paused)
	if result is Dictionary:
		return result
	return {"initialized": false}


## Returns current packed frame snapshots (36 bytes per agent).
func get_frame_snapshots() -> PackedByteArray:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return PackedByteArray()
	if not runtime.has_method("get_frame_snapshots"):
		return PackedByteArray()
	var result: Variant = runtime.call("get_frame_snapshots")
	if result is PackedByteArray:
		return result
	return PackedByteArray()


## Returns previous packed frame snapshots for interpolation.
func get_prev_frame_snapshots() -> PackedByteArray:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return PackedByteArray()
	if not runtime.has_method("get_prev_frame_snapshots"):
		return PackedByteArray()
	var result: Variant = runtime.call("get_prev_frame_snapshots")
	if result is PackedByteArray:
		return result
	return PackedByteArray()


## Returns interpolation alpha in the range 0.0..1.0.
func get_render_alpha() -> float:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return 0.0
	if not runtime.has_method("get_render_alpha"):
		return 0.0
	return float(runtime.call("get_render_alpha"))


## Returns the current number of alive agents in the binary snapshot buffer.
func get_agent_count() -> int:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return 0
	if not runtime.has_method("get_agent_count"):
		return 0
	return int(runtime.call("get_agent_count"))


## Returns runtime snapshot bytes.
func runtime_get_snapshot() -> PackedByteArray:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return PackedByteArray()
	if not runtime.has_method("runtime_get_snapshot"):
		return PackedByteArray()
	var result: Variant = runtime.call("runtime_get_snapshot")
	if result is PackedByteArray:
		return result
	return PackedByteArray()


## Applies runtime snapshot bytes.
func runtime_apply_snapshot(snapshot_bytes: PackedByteArray) -> bool:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return false
	if not runtime.has_method("runtime_apply_snapshot"):
		return false
	return bool(runtime.call("runtime_apply_snapshot", snapshot_bytes))


## Saves runtime snapshot in .ws2 binary format.
func runtime_save_ws2(path: String) -> bool:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return false
	if not runtime.has_method("runtime_save_ws2"):
		return false
	return bool(runtime.call("runtime_save_ws2", path))


## Loads runtime snapshot from .ws2 binary format.
func runtime_load_ws2(path: String) -> bool:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return false
	if not runtime.has_method("runtime_load_ws2"):
		return false
	return bool(runtime.call("runtime_load_ws2", path))


## Exports runtime events in Bus v2 payload format.
func runtime_export_events_v2() -> Array:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return []
	if not runtime.has_method("runtime_export_events_v2"):
		return []
	var result: Variant = runtime.call("runtime_export_events_v2")
	if result is Array:
		return result
	return []


## Drains sparse story notifications from the Rust runtime.
func drain_notifications() -> Array:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return []
	if not runtime.has_method("drain_notifications"):
		return []
	var result: Variant = runtime.call("drain_notifications")
	if result is Array:
		return result
	return []


## Returns the archetype locale key for an entity from Rust runtime.
func get_archetype_label(entity_id: int) -> String:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return ""
	if not runtime.has_method("get_archetype_label"):
		return ""
	return str(runtime.call("get_archetype_label", entity_id))


## Returns the formatted thought-stream text for an entity from Rust runtime.
func get_thought_text(entity_id: int) -> String:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return ""
	if not runtime.has_method("get_thought_text"):
		return ""
	return str(runtime.call("get_thought_text", entity_id))


## Returns pre-computed narrative panel display data for an entity from Rust runtime.
func get_narrative_display(entity_id: int) -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {}
	if not runtime.has_method("get_narrative_display"):
		return {}
	var result: Variant = runtime.call("get_narrative_display", entity_id)
	if result is Dictionary:
		return result
	return {}


## Drains queued LLM debug log lines from Rust runtime.
func drain_llm_debug_log() -> Array[String]:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return []
	if not runtime.has_method("drain_llm_debug_log"):
		return []
	var result: Variant = runtime.call("drain_llm_debug_log")
	if not (result is Array):
		return []
	var lines: Array[String] = []
	for entry in result:
		lines.append(str(entry))
	return lines


## Tells Rust that the player opened the narrative panel for an entity.
func on_entity_narrative_click(entity_id: int) -> int:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return 0
	if not runtime.has_method("on_entity_narrative_click"):
		return 0
	return int(runtime.call("on_entity_narrative_click", entity_id))


## Updates the runtime AI narration quality tier.
func set_llm_quality(quality: int) -> void:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return
	if not runtime.has_method("set_llm_quality"):
		return
	runtime.call("set_llm_quality", quality)


## Returns the runtime AI narration quality tier.
func get_llm_quality() -> int:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return 0
	if not runtime.has_method("get_llm_quality"):
		return 0
	return int(runtime.call("get_llm_quality"))


## Spawns agents into the Rust hecs world from a JSON string. Returns number spawned.
func runtime_spawn_agents(spawn_data_json: String) -> int:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return 0
	if not runtime.has_method("runtime_spawn_agents"):
		return 0
	return int(runtime.call("runtime_spawn_agents", spawn_data_json))


## Bootstraps the authoritative Rust world from setup JSON and returns a summary dictionary.
func runtime_bootstrap_world(setup_json: String) -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {}
	if not runtime.has_method("runtime_bootstrap_world"):
		return {}
	var result: Variant = runtime.call("runtime_bootstrap_world", setup_json)
	if result is Dictionary:
		return result
	return {}


## Returns registered runtime system metadata snapshot.
func runtime_get_registry_snapshot() -> Array:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return []
	if not runtime.has_method("runtime_get_registry_snapshot"):
		return []
	var result: Variant = runtime.call("runtime_get_registry_snapshot")
	if result is Array:
		return result
	return []


## Registers the authoritative default Rust runtime manifest and returns the entry count.
func runtime_register_default_systems() -> int:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return 0
	if not runtime.has_method("runtime_register_default_systems"):
		return 0
	return int(runtime.call("runtime_register_default_systems"))


## Returns entity detail dictionary from Rust runtime.
func runtime_get_entity_detail(entity_id: int) -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {}
	if not runtime.has_method("runtime_get_entity_detail"):
		return {}
	var result: Variant = runtime.call("runtime_get_entity_detail", entity_id)
	if result is Dictionary:
		return result
	return {}


## Returns the temporary legacy chronicle timeline adapter from Rust runtime.
func runtime_get_chronicle_timeline(limit: int) -> Array:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return []
	if not runtime.has_method("runtime_get_chronicle_timeline"):
		return []
	var result: Variant = runtime.call("runtime_get_chronicle_timeline", limit)
	if result is Array:
		return result
	return []


## Returns the runtime Chronicle feed snapshot family response.
func runtime_get_chronicle_feed(limit: int, snapshot_revision: int = -1) -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {"snapshot_revision": -1, "revision_unavailable": true, "items": []}
	if not runtime.has_method("runtime_get_chronicle_feed"):
		return {"snapshot_revision": -1, "revision_unavailable": true, "items": []}
	var result: Variant = runtime.call("runtime_get_chronicle_feed", limit, snapshot_revision)
	if result is Dictionary:
		return result
	return {"snapshot_revision": -1, "revision_unavailable": true, "items": []}


## Returns one Chronicle entry detail snapshot from Rust runtime.
func runtime_get_chronicle_entry_detail(entry_id: int, snapshot_revision: int = -1) -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {"snapshot_revision": -1, "revision_unavailable": true, "available": false}
	if not runtime.has_method("runtime_get_chronicle_entry_detail"):
		return {"snapshot_revision": -1, "revision_unavailable": true, "available": false}
	var result: Variant = runtime.call("runtime_get_chronicle_entry_detail", entry_id, snapshot_revision)
	if result is Dictionary:
		return result
	return {"snapshot_revision": -1, "revision_unavailable": true, "available": false}


## Returns the current story thread snapshot list from Rust runtime.
func runtime_get_story_threads(limit: int, snapshot_revision: int = -1) -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {"snapshot_revision": -1, "revision_unavailable": true, "items": []}
	if not runtime.has_method("runtime_get_story_threads"):
		return {"snapshot_revision": -1, "revision_unavailable": true, "items": []}
	var result: Variant = runtime.call("runtime_get_story_threads", limit, snapshot_revision)
	if result is Dictionary:
		return result
	return {"snapshot_revision": -1, "revision_unavailable": true, "items": []}


## Returns an archive/history slice snapshot from Rust runtime.
func runtime_get_history_slice(limit: int, cursor_before_tick: int = -1, cursor_before_entry_id: int = -1, snapshot_revision: int = -1) -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {"snapshot_revision": -1, "revision_unavailable": true, "items": [], "next_cursor_before_tick": -1, "next_cursor_before_entry_id": -1}
	if not runtime.has_method("runtime_get_history_slice"):
		return {"snapshot_revision": -1, "revision_unavailable": true, "items": [], "next_cursor_before_tick": -1, "next_cursor_before_entry_id": -1}
	var result: Variant = runtime.call("runtime_get_history_slice", limit, cursor_before_tick, cursor_before_entry_id, snapshot_revision)
	if result is Dictionary:
		return result
	return {"snapshot_revision": -1, "revision_unavailable": true, "items": [], "next_cursor_before_tick": -1, "next_cursor_before_entry_id": -1}


## Returns the current recall queue snapshot from Rust runtime.
func runtime_get_recall_slice(limit: int, snapshot_revision: int = -1) -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {"snapshot_revision": -1, "revision_unavailable": true, "items": []}
	if not runtime.has_method("runtime_get_recall_slice"):
		return {"snapshot_revision": -1, "revision_unavailable": true, "items": []}
	var result: Variant = runtime.call("runtime_get_recall_slice", limit, snapshot_revision)
	if result is Dictionary:
		return result
	return {"snapshot_revision": -1, "revision_unavailable": true, "items": []}


## Returns entity tab data from Rust runtime.
func runtime_get_entity_tab(entity_id: int, tab: String) -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {}
	if not runtime.has_method("runtime_get_entity_tab"):
		return {}
	var result: Variant = runtime.call("runtime_get_entity_tab", entity_id, tab)
	if result is Dictionary:
		return result
	return {}


## Returns entity list from Rust runtime.
func runtime_get_entity_list() -> Array:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return []
	if not runtime.has_method("runtime_get_entity_list"):
		return []
	var result: Variant = runtime.call("runtime_get_entity_list")
	if result is Array:
		return result
	return []


## Returns settlement detail from Rust runtime.
func runtime_get_settlement_detail(settlement_id: int) -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {}
	if not runtime.has_method("runtime_get_settlement_detail"):
		return {}
	var result: Variant = runtime.call("runtime_get_settlement_detail", settlement_id)
	if result is Dictionary:
		return result
	return {}


## Returns building detail from Rust runtime.
func runtime_get_building_detail(building_id: int) -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {}
	if not runtime.has_method("runtime_get_building_detail"):
		return {}
	var result: Variant = runtime.call("runtime_get_building_detail", building_id)
	if result is Dictionary:
		return result
	return {}


## Returns a world summary snapshot from Rust runtime.
func runtime_get_world_summary() -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {}
	if not runtime.has_method("runtime_get_world_summary"):
		return {}
	var result: Variant = runtime.call("runtime_get_world_summary")
	if result is Dictionary:
		return result
	return {}


## Returns a compact minimap snapshot from Rust runtime.
func runtime_get_minimap_snapshot() -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {}
	if not runtime.has_method("runtime_get_minimap_snapshot"):
		return {}
	var result: Variant = runtime.call("runtime_get_minimap_snapshot")
	if result is Dictionary:
		return result
	return {}


## Returns runtime compute-domain modes snapshot.
func runtime_get_compute_domain_modes() -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {}
	if not runtime.has_method("runtime_get_compute_domain_modes"):
		return {}
	var result: Variant = runtime.call("runtime_get_compute_domain_modes")
	if result is Dictionary:
		return result
	return {}


## Applies runtime commands in Bus v2 command format.
func runtime_apply_commands_v2(commands: Array) -> void:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return
	if not runtime.has_method("runtime_apply_commands_v2"):
		return
	runtime.call("runtime_apply_commands_v2", commands)


## Enables or disables Rust debug mode (activates PerfTracker).
func enable_debug_mode(enabled: bool) -> void:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return
	if not runtime.has_method("enable_debug_mode"):
		return
	runtime.call("enable_debug_mode", enabled)


## Returns a summary dictionary of current simulation state.
## Keys: tick, entity_count, population, season, ticks_per_second, paused, current_tick_us.
func get_debug_summary() -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {}
	if not runtime.has_method("get_debug_summary"):
		return {}
	var result: Variant = runtime.call("get_debug_summary")
	if result is Dictionary:
		return result
	return {}


## Returns per-system timing data (only populated when debug_mode is true).
func get_system_perf() -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {}
	if not runtime.has_method("get_system_perf"):
		return {}
	var result: Variant = runtime.call("get_system_perf")
	if result is Dictionary:
		return result
	return {}


## Returns the last 300 tick durations as PackedFloat32Array in milliseconds.
func get_tick_history() -> PackedFloat32Array:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return PackedFloat32Array()
	if not runtime.has_method("get_tick_history"):
		return PackedFloat32Array()
	var result: Variant = runtime.call("get_tick_history")
	if result is PackedFloat32Array:
		return result
	return PackedFloat32Array()


## Returns all SimConfig key-value pairs as a flat Dictionary.
func get_config_values() -> Dictionary:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return {}
	if not runtime.has_method("get_config_values"):
		return {}
	var result: Variant = runtime.call("get_config_values")
	if result is Dictionary:
		return result
	return {}


## Sets a single SimConfig value by key. Returns true if the key exists.
func set_config_value(key: String, value: float) -> bool:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return false
	if not runtime.has_method("set_config_value"):
		return false
	var result: Variant = runtime.call("set_config_value", key, value)
	if result is bool:
		return result
	return false


## Returns guardrail status array.
func get_guardrail_status() -> Array:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return []
	if not runtime.has_method("get_guardrail_status"):
		return []
	var result: Variant = runtime.call("get_guardrail_status")
	if result is Array:
		return result
	return []


## Queries entities matching a condition and returns their IDs.
## Supported conditions: "stress_gte", "health_lte", "hunger_lte".
func query_entities_by_condition(condition: String, threshold: float) -> PackedInt32Array:
	var runtime: Object = _get_native_runtime()
	if runtime == null:
		return PackedInt32Array()
	if not runtime.has_method("query_entities_by_condition"):
		return PackedInt32Array()
	var result: Variant = runtime.call("query_entities_by_condition", condition, threshold)
	if result is PackedInt32Array:
		return result
	return PackedInt32Array()


## Loads Fluent source text into Rust runtime localization cache.
func locale_load_fluent(locale: String, source: String) -> bool:
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return false
	if not bridge.has_method("locale_load_fluent"):
		return false
	return bool(bridge.call("locale_load_fluent", locale, source))


## Clears Fluent source text from Rust runtime localization cache.
func locale_clear_fluent(locale: String) -> void:
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return
	if not bridge.has_method("locale_clear_fluent"):
		return
	bridge.call("locale_clear_fluent", locale)


## Formats a Fluent message through Rust runtime localization cache.
func locale_format_fluent(locale: String, key: String, params: Dictionary = {}) -> String:
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return key
	if not bridge.has_method("locale_format_fluent"):
		return key
	var result: Variant = bridge.call("locale_format_fluent", locale, key, params)
	return str(result)


## Sets native pathfinding backend mode (`auto`, `cpu`, `gpu`) when supported.
## Returns true when mode was accepted by bridge.
func set_pathfinding_backend(mode: String) -> bool:
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return false
	var method_name: String = _set_pathfind_backend_method_name
	if method_name == "":
		method_name = _pick_method(_SET_PATHFIND_BACKEND_METHOD_CANDIDATES, "")
		_set_pathfind_backend_method_name = method_name
	if method_name == "":
		return false
	var applied: Variant = bridge.call(method_name, mode)
	if applied is bool and not bool(applied):
		return false
	_last_synced_pathfind_backend_mode = mode
	_resolved_pathfind_backend_cached = false
	_resolved_pathfind_backend_cache = ""
	return true


## Returns configured native pathfinding backend mode (`auto`, `cpu`, `gpu`) when supported.
func get_pathfinding_backend() -> String:
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return "auto"
	var method_name: String = _get_pathfind_backend_method_name
	if method_name == "":
		method_name = _pick_method(_GET_PATHFIND_BACKEND_METHOD_CANDIDATES, "")
		_get_pathfind_backend_method_name = method_name
	if method_name == "":
		return "auto"
	return str(bridge.call(method_name))


## Returns resolved runtime pathfinding backend (`cpu` or `gpu`) when supported.
func resolve_pathfinding_backend() -> String:
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return "cpu"
	var resolved: String = _resolve_pathfinding_backend_cached(bridge)
	if not resolved.is_empty():
		return resolved
	if Engine.has_singleton("ComputeBackend"):
		var backend: Object = Engine.get_singleton("ComputeBackend")
		if backend != null and backend.has_method("resolve_mode"):
			return str(backend.call("resolve_mode"))
	return "cpu"


## Returns true when native bridge reports GPU pathfinding capability.
func has_gpu_pathfinding() -> bool:
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return false
	return _resolve_gpu_pathfinding_capability(bridge)


## Delegates pathfinding to native bridge when available.
## Returns null when native bridge is unavailable, so caller can fallback.
func pathfind_grid(
	width: int,
	height: int,
	walkable: PackedByteArray,
	move_cost: PackedFloat32Array,
	from_x: int,
	from_y: int,
	to_x: int,
	to_y: int,
	max_steps: int
):
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return null
	var method_name: String = _pathfind_method_name
	if _prefer_gpu():
		method_name = _pick_method(_PATHFIND_GPU_METHOD_CANDIDATES, _pathfind_method_name)
	return bridge.call(
		method_name,
		width,
		height,
		walkable,
		move_cost,
		from_x,
		from_y,
		to_x,
		to_y,
		max_steps
	)


## Delegates pathfinding with PackedInt32Array(x,y,...) output when available.
## Returns null when native bridge is unavailable, so caller can fallback.
func pathfind_grid_xy(
	width: int,
	height: int,
	walkable: PackedByteArray,
	move_cost: PackedFloat32Array,
	from_x: int,
	from_y: int,
	to_x: int,
	to_y: int,
	max_steps: int
):
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return null
	if _pathfind_xy_method_name == "":
		return null
	var method_name: String = _pathfind_xy_method_name
	if _prefer_gpu():
		method_name = _pick_method(_PATHFIND_XY_GPU_METHOD_CANDIDATES, _pathfind_xy_method_name)
	return bridge.call(
		method_name,
		width,
		height,
		walkable,
		move_cost,
		from_x,
		from_y,
		to_x,
		to_y,
		max_steps
	)

## Delegates batch pathfinding to native bridge when available.
## Returns null when native bridge is unavailable, so caller can fallback.
func pathfind_grid_batch(
	width: int,
	height: int,
	walkable: PackedByteArray,
	move_cost: PackedFloat32Array,
	from_points: PackedVector2Array,
	to_points: PackedVector2Array,
	max_steps: int
):
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return null
	if _pathfind_batch_method_name == "":
		return null
	var method_name: String = _pathfind_batch_method_name
	if _prefer_gpu():
		method_name = _pick_method(_PATHFIND_BATCH_GPU_METHOD_CANDIDATES, _pathfind_batch_method_name)
	return bridge.call(
		method_name,
		width,
		height,
		walkable,
		move_cost,
		from_points,
		to_points,
		max_steps
	)


## Delegates batch pathfinding with PackedInt32Array(x,y,...) points.
## Returns null when native bridge is unavailable, so caller can fallback.
func pathfind_grid_batch_xy(
	width: int,
	height: int,
	walkable: PackedByteArray,
	move_cost: PackedFloat32Array,
	from_xy: PackedInt32Array,
	to_xy: PackedInt32Array,
	max_steps: int
):
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return null
	if _pathfind_batch_xy_method_name == "":
		return null
	var method_name: String = _pathfind_batch_xy_method_name
	if _prefer_gpu():
		method_name = _pick_method(_PATHFIND_BATCH_XY_GPU_METHOD_CANDIDATES, _pathfind_batch_xy_method_name)
	return bridge.call(
		method_name,
		width,
		height,
		walkable,
		move_cost,
		from_xy,
		to_xy,
		max_steps
	)


## Delegates LOG_DIMINISHING XP requirement curve to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_log_xp_required(
	level: int,
	base_xp: float,
	exponent: float,
	level_breakpoints: PackedInt32Array,
	breakpoint_multipliers: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_log_xp_required",
		[
			level,
			base_xp,
			exponent,
			level_breakpoints,
			breakpoint_multipliers
		]
	)


## Delegates cumulative XP->level conversion to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_xp_to_level(
	xp: float,
	base_xp: float,
	exponent: float,
	level_breakpoints: PackedInt32Array,
	breakpoint_multipliers: PackedFloat32Array,
	max_level: int
):
	return _call_native_if_exists(
		"stat_xp_to_level",
		[
			xp,
			base_xp,
			exponent,
			level_breakpoints,
			breakpoint_multipliers,
			max_level
		]
	)


## Delegates skill XP progress computation to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_skill_xp_progress(
	level: int,
	xp: float,
	base_xp: float,
	exponent: float,
	level_breakpoints: PackedInt32Array,
	breakpoint_multipliers: PackedFloat32Array,
	max_level: int
):
	return _call_native_if_exists(
		"stat_skill_xp_progress",
		[
			level,
			xp,
			base_xp,
			exponent,
			level_breakpoints,
			breakpoint_multipliers,
			max_level
		]
	)


## Delegates SCURVE phase speed lookup to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_scurve_speed(
	current_value: int,
	phase_breakpoints: PackedInt32Array,
	phase_speeds: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_scurve_speed",
		[
			current_value,
			phase_breakpoints,
			phase_speeds
		]
	)


## Delegates natural need decay curve to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_need_decay(
	current: int,
	decay_per_year: int,
	ticks_elapsed: int,
	metabolic_mult: float,
	ticks_per_year: int
):
	return _call_native_if_exists(
		"stat_need_decay",
		[
			current,
			decay_per_year,
			ticks_elapsed,
			metabolic_mult,
			ticks_per_year
		]
	)


## Delegates continuous stress inputs from unmet needs to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_continuous_inputs(
	hunger: float,
	energy: float,
	social: float,
	appraisal_scale: float
):
	return _call_native_if_exists(
		"stat_stress_continuous_inputs",
		[
			hunger,
			energy,
			social,
			appraisal_scale
		]
	)


## Delegates Lazarus appraisal stress scale computation to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_appraisal_scale(
	hunger: float,
	energy: float,
	social: float,
	threat: float,
	conflict: float,
	support_score: float,
	extroversion: float,
	fear_value: float,
	trust_value: float,
	conscientiousness: float,
	openness: float,
	reserve_ratio: float
):
	return _call_native_if_exists(
		"stat_stress_appraisal_scale",
		[
			hunger,
			energy,
			social,
			threat,
			conflict,
			support_score,
			extroversion,
			fear_value,
			trust_value,
			conscientiousness,
			openness,
			reserve_ratio
		]
	)


## Delegates combined appraisal + continuous stress primary step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_primary_step(
	hunger: float,
	energy: float,
	social: float,
	threat: float,
	conflict: float,
	support_score: float,
	extroversion: float,
	fear_value: float,
	trust_value: float,
	conscientiousness: float,
	openness: float,
	reserve_ratio: float
):
	return _call_native_if_exists(
		"stat_stress_primary_step",
		[
			hunger,
			energy,
			social,
			threat,
			conflict,
			support_score,
			extroversion,
			fear_value,
			trust_value,
			conscientiousness,
			openness,
			reserve_ratio
		]
	)


## Delegates emotion-to-stress contribution computation to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_emotion_contribution(
	fear: float,
	anger: float,
	sadness: float,
	disgust: float,
	surprise: float,
	joy: float,
	trust: float,
	anticipation: float,
	valence: float,
	arousal: float
):
	return _call_native_if_exists(
		"stat_stress_emotion_contribution",
		[
			fear,
			anger,
			sadness,
			disgust,
			surprise,
			joy,
			trust,
			anticipation,
			valence,
			arousal
		]
	)


## Delegates stress recovery decay computation to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_recovery_value(
	stress: float,
	support_score: float,
	resilience: float,
	reserve: float,
	is_sleeping: bool,
	is_safe: bool
):
	return _call_native_if_exists(
		"stat_stress_recovery_value",
		[
			stress,
			support_score,
			resilience,
			reserve,
			is_sleeping,
			is_safe
		]
	)


## Delegates emotion+recovery+delta(denial) combined step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_emotion_recovery_delta_step(
	fear: float,
	anger: float,
	sadness: float,
	disgust: float,
	surprise: float,
	joy: float,
	trust: float,
	anticipation: float,
	valence: float,
	arousal: float,
	stress: float,
	support_score: float,
	resilience: float,
	reserve: float,
	is_sleeping: bool,
	is_safe: bool,
	continuous_input: float,
	trace_input: float,
	ace_stress_mult: float,
	trait_accum_mult: float,
	epsilon: float,
	denial_active: bool,
	denial_redirect_fraction: float,
	hidden_threat_accumulator: float,
	denial_max_accumulator: float
):
	var emotion_inputs: PackedFloat32Array = PackedFloat32Array([
		fear,
		anger,
		sadness,
		disgust,
		surprise,
		joy,
		trust,
		anticipation,
		valence,
		arousal,
	])
	var scalar_inputs: PackedFloat32Array = PackedFloat32Array([
		stress,
		support_score,
		resilience,
		reserve,
		continuous_input,
		trace_input,
		ace_stress_mult,
		trait_accum_mult,
		epsilon,
		denial_redirect_fraction,
		hidden_threat_accumulator,
		denial_max_accumulator,
	])
	var flags: PackedByteArray = PackedByteArray([
		1 if is_sleeping else 0,
		1 if is_safe else 0,
		1 if denial_active else 0,
	])
	return _call_native_if_exists(
		"stat_stress_emotion_recovery_delta_step",
		[
			emotion_inputs,
			scalar_inputs,
			flags
		]
	)


## Delegates combined trace+emotion+recovery+delta step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_trace_emotion_recovery_delta_step(
	per_tick: PackedFloat32Array,
	decay_rate: PackedFloat32Array,
	min_keep: float,
	fear: float,
	anger: float,
	sadness: float,
	disgust: float,
	surprise: float,
	joy: float,
	trust: float,
	anticipation: float,
	valence: float,
	arousal: float,
	stress: float,
	support_score: float,
	resilience: float,
	reserve: float,
	is_sleeping: bool,
	is_safe: bool,
	continuous_input: float,
	ace_stress_mult: float,
	trait_accum_mult: float,
	epsilon: float,
	denial_active: bool,
	denial_redirect_fraction: float,
	hidden_threat_accumulator: float,
	denial_max_accumulator: float
):
	var emotion_inputs: PackedFloat32Array = PackedFloat32Array([
		fear,
		anger,
		sadness,
		disgust,
		surprise,
		joy,
		trust,
		anticipation,
		valence,
		arousal,
	])
	var scalar_inputs: PackedFloat32Array = PackedFloat32Array([
		stress,
		support_score,
		resilience,
		reserve,
		continuous_input,
		ace_stress_mult,
		trait_accum_mult,
		epsilon,
		denial_redirect_fraction,
		hidden_threat_accumulator,
		denial_max_accumulator,
	])
	var flags: PackedByteArray = PackedByteArray([
		1 if is_sleeping else 0,
		1 if is_safe else 0,
		1 if denial_active else 0,
	])
	return _call_native_if_exists(
		"stat_stress_trace_emotion_recovery_delta_step",
		[
			per_tick,
			decay_rate,
			min_keep,
			emotion_inputs,
			scalar_inputs,
			flags
		]
	)


## Delegates full stress tick step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_tick_step(
	per_tick: PackedFloat32Array,
	decay_rate: PackedFloat32Array,
	min_keep: float,
	scalar_inputs: PackedFloat32Array,
	flags: PackedByteArray
):
	return _call_native_if_exists(
		"stat_stress_tick_step",
		[
			per_tick,
			decay_rate,
			min_keep,
			scalar_inputs,
			flags
		]
	)


## Delegates full stress tick step (packed output) to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_tick_step_packed(
	per_tick: PackedFloat32Array,
	decay_rate: PackedFloat32Array,
	min_keep: float,
	scalar_inputs: PackedFloat32Array,
	flags: PackedByteArray
):
	return _call_native_if_exists(
		"stat_stress_tick_step_packed",
		[
			per_tick,
			decay_rate,
			min_keep,
			scalar_inputs,
			flags
		]
	)


## Delegates final stress delta step (including denial redirect) to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_delta_step(
	continuous_input: float,
	trace_input: float,
	emotion_input: float,
	ace_stress_mult: float,
	trait_accum_mult: float,
	recovery: float,
	epsilon: float,
	denial_active: bool,
	denial_redirect_fraction: float,
	hidden_threat_accumulator: float,
	denial_max_accumulator: float
):
	return _call_native_if_exists(
		"stat_stress_delta_step",
		[
			continuous_input,
			trace_input,
			emotion_input,
			ace_stress_mult,
			trait_accum_mult,
			recovery,
			epsilon,
			denial_active,
			denial_redirect_fraction,
			hidden_threat_accumulator,
			denial_max_accumulator
		]
	)


## Delegates post-stress reserve/allostatic/state combined step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_post_update_step(
	reserve: float,
	stress: float,
	resilience: float,
	stress_delta_last: float,
	gas_stage: int,
	is_sleeping: bool,
	allostatic: float,
	avoidant_allostatic_mult: float
):
	return _call_native_if_exists(
		"stat_stress_post_update_step",
		[
			reserve,
			stress,
			resilience,
			stress_delta_last,
			gas_stage,
			is_sleeping,
			allostatic,
			avoidant_allostatic_mult
		]
	)


## Delegates post-update + resilience combined step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_post_update_resilience_step(
	reserve: float,
	stress: float,
	resilience: float,
	stress_delta_last: float,
	gas_stage: int,
	is_sleeping: bool,
	allostatic: float,
	avoidant_allostatic_mult: float,
	e_axis: float,
	c_axis: float,
	x_axis: float,
	o_axis: float,
	a_axis: float,
	h_axis: float,
	support_score: float,
	hunger: float,
	energy: float,
	scar_resilience_mod: float
):
	var scalar_inputs: PackedFloat32Array = PackedFloat32Array([
		reserve,
		stress,
		resilience,
		stress_delta_last,
		float(gas_stage),
		allostatic,
		avoidant_allostatic_mult,
		e_axis,
		c_axis,
		x_axis,
		o_axis,
		a_axis,
		h_axis,
		support_score,
		hunger,
		energy,
		scar_resilience_mod,
	])
	var flags: PackedByteArray = PackedByteArray([
		1 if is_sleeping else 0,
	])
	return _call_native_if_exists(
		"stat_stress_post_update_resilience_step",
		[
			scalar_inputs,
			flags
		]
	)


## Delegates reserve + GAS stage transition step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_reserve_step(
	reserve: float,
	stress: float,
	resilience: float,
	stress_delta_last: float,
	gas_stage: int,
	is_sleeping: bool
):
	return _call_native_if_exists(
		"stat_stress_reserve_step",
		[
			reserve,
			stress,
			resilience,
			stress_delta_last,
			gas_stage,
			is_sleeping
		]
	)


## Delegates allostatic load update step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_allostatic_step(
	allostatic: float,
	stress: float,
	avoidant_allostatic_mult: float
):
	return _call_native_if_exists(
		"stat_stress_allostatic_step",
		[
			allostatic,
			stress,
			avoidant_allostatic_mult
		]
	)


## Delegates stress state + emotion-meta snapshot to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_state_snapshot(
	stress: float,
	allostatic: float
):
	return _call_native_if_exists(
		"stat_stress_state_snapshot",
		[
			stress,
			allostatic
		]
	)


## Delegates batch stress-trace decay/keep step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_trace_batch_step(
	per_tick: PackedFloat32Array,
	decay_rate: PackedFloat32Array,
	min_keep: float
):
	return _call_native_if_exists(
		"stat_stress_trace_batch_step",
		[
			per_tick,
			decay_rate,
			min_keep
		]
	)


## Delegates stress resilience value computation to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_resilience_value(
	e_axis: float,
	c_axis: float,
	x_axis: float,
	o_axis: float,
	a_axis: float,
	h_axis: float,
	support_score: float,
	allostatic: float,
	hunger: float,
	energy: float,
	scar_resilience_mod: float
):
	return _call_native_if_exists(
		"stat_stress_resilience_value",
		[
			e_axis,
			c_axis,
			x_axis,
			o_axis,
			a_axis,
			h_axis,
			support_score,
			allostatic,
			hunger,
			energy,
			scar_resilience_mod
		]
	)


## Delegates stress work efficiency curve to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_work_efficiency(
	stress: float,
	shaken_penalty: float
):
	return _call_native_if_exists(
		"stat_stress_work_efficiency",
		[
			stress,
			shaken_penalty
		]
	)


## Delegates stress personality scaling curve to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_personality_scale(
	values: PackedFloat32Array,
	weights: PackedFloat32Array,
	high_amplifies: PackedByteArray,
	trait_multipliers: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_stress_personality_scale",
		[
			values,
			weights,
			high_amplifies,
			trait_multipliers
		]
	)


## Delegates stress relationship scaling to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_relationship_scale(
	method: String,
	bond_strength: float,
	min_mult: float,
	max_mult: float
):
	return _call_native_if_exists(
		"stat_stress_relationship_scale",
		[
			method,
			bond_strength,
			min_mult,
			max_mult
		]
	)


## Delegates stress context scaling to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_context_scale(active_multipliers: PackedFloat32Array):
	return _call_native_if_exists(
		"stat_stress_context_scale",
		[
			active_multipliers
		]
	)


## Delegates stress emotion injection step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_emotion_inject_step(
	fast_current: PackedFloat32Array,
	slow_current: PackedFloat32Array,
	fast_inject: PackedFloat32Array,
	slow_inject: PackedFloat32Array,
	scale: float
):
	return _call_native_if_exists(
		"stat_stress_emotion_inject_step",
		[
			fast_current,
			slow_current,
			fast_inject,
			slow_inject,
			scale
		]
	)


## Delegates stress rebound queue step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_rebound_queue_step(
	amounts: PackedFloat32Array,
	delays: PackedInt32Array,
	decay_per_tick: float
):
	return _call_native_if_exists(
		"stat_stress_rebound_queue_step",
		[
			amounts,
			delays,
			decay_per_tick
		]
	)


## Delegates combined stress event scale step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_event_scale_step(
	base_instant: float,
	base_per_tick: float,
	is_loss: bool,
	personality_scale: float,
	appraisal_scale: float,
	relationship_method: String,
	bond_strength: float,
	relationship_min_mult: float,
	relationship_max_mult: float,
	context_active_multipliers: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_stress_event_scale_step",
		[
			base_instant,
			base_per_tick,
			is_loss,
			personality_scale,
			appraisal_scale,
			relationship_method,
			bond_strength,
			relationship_min_mult,
			relationship_max_mult,
			context_active_multipliers
		]
	)


## Delegates combined stress event scale step (method code) to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_event_scale_step_code(
	base_instant: float,
	base_per_tick: float,
	is_loss: bool,
	personality_scale: float,
	appraisal_scale: float,
	relationship_method_code: int,
	bond_strength: float,
	relationship_min_mult: float,
	relationship_max_mult: float,
	context_active_multipliers: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_stress_event_scale_step_code",
		[
			base_instant,
			base_per_tick,
			is_loss,
			personality_scale,
			appraisal_scale,
			relationship_method_code,
			bond_strength,
			relationship_min_mult,
			relationship_max_mult,
			context_active_multipliers
		]
	)


## Delegates combined stress event scale + emotion inject step to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_event_inject_step(
	base_instant: float,
	base_per_tick: float,
	is_loss: bool,
	personality_scale: float,
	appraisal_scale: float,
	relationship_method: String,
	bond_strength: float,
	relationship_min_mult: float,
	relationship_max_mult: float,
	context_active_multipliers: PackedFloat32Array,
	fast_current: PackedFloat32Array,
	slow_current: PackedFloat32Array,
	fast_inject: PackedFloat32Array,
	slow_inject: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_stress_event_inject_step",
		[
			base_instant,
			base_per_tick,
			is_loss,
			personality_scale,
			appraisal_scale,
			relationship_method,
			bond_strength,
			relationship_min_mult,
			relationship_max_mult,
			context_active_multipliers,
			fast_current,
			slow_current,
			fast_inject,
			slow_inject
		]
	)


## Delegates combined stress event scale + emotion inject step (method code) to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_event_inject_step_code(
	base_instant: float,
	base_per_tick: float,
	is_loss: bool,
	personality_scale: float,
	appraisal_scale: float,
	relationship_method_code: int,
	bond_strength: float,
	relationship_min_mult: float,
	relationship_max_mult: float,
	context_active_multipliers: PackedFloat32Array,
	fast_current: PackedFloat32Array,
	slow_current: PackedFloat32Array,
	fast_inject: PackedFloat32Array,
	slow_inject: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_stress_event_inject_step_code",
		[
			base_instant,
			base_per_tick,
			is_loss,
			personality_scale,
			appraisal_scale,
			relationship_method_code,
			bond_strength,
			relationship_min_mult,
			relationship_max_mult,
			context_active_multipliers,
			fast_current,
			slow_current,
			fast_inject,
			slow_inject
		]
	)


## Delegates stress event final scaling to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_stress_event_scaled(
	base_instant: float,
	base_per_tick: float,
	is_loss: bool,
	personality_scale: float,
	relationship_scale: float,
	context_scale: float,
	appraisal_scale: float
):
	return _call_native_if_exists(
		"stat_stress_event_scaled",
		[
			base_instant,
			base_per_tick,
			is_loss,
			personality_scale,
			relationship_scale,
			context_scale,
			appraisal_scale
		]
	)


## Delegates SIGMOID_EXTREME influence to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_sigmoid_extreme(
	value: int,
	flat_zone_lo: int,
	flat_zone_hi: int,
	pole_multiplier: float
):
	return _call_native_if_exists(
		"stat_sigmoid_extreme",
		[
			value,
			flat_zone_lo,
			flat_zone_hi,
			pole_multiplier
		]
	)


## Delegates POWER influence to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_power_influence(value: int, exponent: float):
	return _call_native_if_exists("stat_power_influence", [value, exponent])


## Delegates THRESHOLD_POWER influence to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_threshold_power(
	value: int,
	threshold: int,
	exponent: float,
	max_output: float
):
	return _call_native_if_exists(
		"stat_threshold_power",
		[
			value,
			threshold,
			exponent,
			max_output
		]
	)


## Delegates LINEAR influence to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_linear_influence(value: int):
	return _call_native_if_exists("stat_linear_influence", [value])


## Delegates STEP influence to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_step_influence(
	value: int,
	threshold: int,
	above_value: float,
	below_value: float
):
	return _call_native_if_exists(
		"stat_step_influence",
		[
			value,
			threshold,
			above_value,
			below_value
		]
	)


## Delegates STEP_LINEAR influence to native bridge.
## Returns null when native bridge/method is unavailable.
func stat_step_linear(
	value: int,
	below_thresholds: PackedInt32Array,
	multipliers: PackedFloat32Array
):
	return _call_native_if_exists(
		"stat_step_linear",
		[
			value,
			below_thresholds,
			multipliers
		]
	)


## Delegates body age curve computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_compute_age_curve(axis: String, age_years: float):
	return _call_native_if_exists(
		"body_compute_age_curve",
		[
			axis,
			age_years
		]
	)


## Delegates batched body age curve computation to native bridge.
## Axis order: [str, agi, end, tou, rec, dr].
## Returns null when native bridge/method is unavailable.
func body_compute_age_curves(age_years: float):
	return _call_native_if_exists(
		"body_compute_age_curves",
		[
			age_years
		]
	)


## Delegates body training gain computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_calc_training_gain(
	potential: int,
	trainability: int,
	xp: float,
	training_ceiling: float,
	xp_for_full_progress: float
):
	return _call_native_if_exists(
		"body_calc_training_gain",
		[
			potential,
			trainability,
			xp,
			training_ceiling,
			xp_for_full_progress
		]
	)


## Delegates batched body training gain computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_calc_training_gains(
	potentials: PackedInt32Array,
	trainabilities: PackedInt32Array,
	xps: PackedFloat32Array,
	training_ceilings: PackedFloat32Array,
	xp_for_full_progress: float
):
	return _call_native_if_exists(
		"body_calc_training_gains",
		[
			potentials,
			trainabilities,
			xps,
			training_ceilings,
			xp_for_full_progress
		]
	)


## Delegates batched body realized value computation to native bridge.
## Axis order in output: [str, agi, end, tou, rec, dr].
## Returns null when native bridge/method is unavailable.
func body_calc_realized_values(
	potentials: PackedInt32Array,
	trainabilities: PackedInt32Array,
	xps: PackedFloat32Array,
	training_ceilings: PackedFloat32Array,
	age_years: float,
	xp_for_full_progress: float
):
	return _call_native_if_exists(
		"body_calc_realized_values",
		[
			potentials,
			trainabilities,
			xps,
			training_ceilings,
			age_years,
			xp_for_full_progress
		]
	)


## Delegates age-based body trainability modifier to native bridge.
## Returns null when native bridge/method is unavailable.
func body_age_trainability_modifier(axis: String, age_years: float):
	return _call_native_if_exists(
		"body_age_trainability_modifier",
		[
			axis,
			age_years
		]
	)


## Delegates age-based REC trainability modifier to native bridge.
## Returns null when native bridge/method is unavailable.
func body_age_trainability_modifier_rec(age_years: float):
	return _call_native_if_exists(
		"body_age_trainability_modifier_rec",
		[
			age_years
		]
	)


## Delegates batched age-based trainability modifiers to native bridge.
## Axis order: [str, agi, end, tou, rec].
## Returns null when native bridge/method is unavailable.
func body_age_trainability_modifiers(age_years: float):
	return _call_native_if_exists(
		"body_age_trainability_modifiers",
		[
			age_years
		]
	)


## Delegates body action energy cost computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_action_energy_cost(
	base_cost: float,
	end_norm: float,
	end_cost_reduction: float
):
	return _call_native_if_exists(
		"body_action_energy_cost",
		[
			base_cost,
			end_norm,
			end_cost_reduction
		]
	)


## Delegates body rest energy recovery computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_rest_energy_recovery(
	base_recovery: float,
	rec_norm: float,
	rec_recovery_bonus: float
):
	return _call_native_if_exists(
		"body_rest_energy_recovery",
		[
			base_recovery,
			rec_norm,
			rec_recovery_bonus
		]
	)


## Delegates thirst decay computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_thirst_decay(
	base_decay: float,
	tile_temp: float,
	temp_neutral: float
):
	return _call_native_if_exists(
		"body_thirst_decay",
		[
			base_decay,
			tile_temp,
			temp_neutral
		]
	)


## Delegates warmth decay computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_warmth_decay(
	base_decay: float,
	tile_temp: float,
	has_tile_temp: bool,
	temp_neutral: float,
	temp_freezing: float,
	temp_cold: float
):
	return _call_native_if_exists(
		"body_warmth_decay",
		[
			base_decay,
			tile_temp,
			has_tile_temp,
			temp_neutral,
			temp_freezing,
			temp_cold
		]
	)


## Delegates combined baseline need decay step to native bridge.
## Returns null when native bridge/method is unavailable.
func body_needs_base_decay_step_packed(
	scalar_inputs: PackedFloat32Array,
	flag_inputs: PackedByteArray
):
	return _call_native_if_exists(
		"body_needs_base_decay_step",
		[
			scalar_inputs,
			flag_inputs
		]
	)


## Delegates combined baseline need decay step to native bridge.
## Returns null when native bridge/method is unavailable.
func body_needs_base_decay_step(
	hunger_value: float,
	hunger_decay_rate: float,
	hunger_stage_mult: float,
	hunger_metabolic_min: float,
	hunger_metabolic_range: float,
	energy_decay_rate: float,
	social_decay_rate: float,
	safety_decay_rate: float,
	thirst_base_decay: float,
	warmth_base_decay: float,
	tile_temp: float,
	has_tile_temp: bool,
	temp_neutral: float,
	temp_freezing: float,
	temp_cold: float,
	needs_expansion_enabled: bool
):
	var scalar_inputs: PackedFloat32Array = PackedFloat32Array()
	scalar_inputs.append(hunger_value)
	scalar_inputs.append(hunger_decay_rate)
	scalar_inputs.append(hunger_stage_mult)
	scalar_inputs.append(hunger_metabolic_min)
	scalar_inputs.append(hunger_metabolic_range)
	scalar_inputs.append(energy_decay_rate)
	scalar_inputs.append(social_decay_rate)
	scalar_inputs.append(safety_decay_rate)
	scalar_inputs.append(thirst_base_decay)
	scalar_inputs.append(warmth_base_decay)
	scalar_inputs.append(tile_temp)
	scalar_inputs.append(temp_neutral)
	scalar_inputs.append(temp_freezing)
	scalar_inputs.append(temp_cold)
	var flag_inputs: PackedByteArray = PackedByteArray()
	flag_inputs.append(1 if has_tile_temp else 0)
	flag_inputs.append(1 if needs_expansion_enabled else 0)
	return body_needs_base_decay_step_packed(scalar_inputs, flag_inputs)


## Delegates critical severity step for thirst/warmth/safety to native bridge.
## Returns null when native bridge/method is unavailable.
func body_needs_critical_severity_step_packed(
	scalar_inputs: PackedFloat32Array
):
	return _call_native_if_exists(
		"body_needs_critical_severity_step_packed",
		[
			scalar_inputs
		]
	)


## Delegates critical severity step for thirst/warmth/safety to native bridge.
## Returns null when native bridge/method is unavailable.
func body_needs_critical_severity_step(
	thirst: float,
	warmth: float,
	safety: float,
	thirst_critical: float,
	warmth_critical: float,
	safety_critical: float
):
	var scalar_inputs: PackedFloat32Array = PackedFloat32Array()
	scalar_inputs.append(thirst)
	scalar_inputs.append(warmth)
	scalar_inputs.append(safety)
	scalar_inputs.append(thirst_critical)
	scalar_inputs.append(warmth_critical)
	scalar_inputs.append(safety_critical)
	return body_needs_critical_severity_step_packed(scalar_inputs)


## Delegates ERG frustration tick-step to native bridge.
## Returns null when native bridge/method is unavailable.
func body_erg_frustration_step_packed(
	scalar_inputs: PackedFloat32Array,
	flag_inputs: PackedByteArray
):
	return _call_native_if_exists(
		"body_erg_frustration_step_packed",
		[
			scalar_inputs,
			flag_inputs
		]
	)


## Delegates anxious-attachment stress delta computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_anxious_attachment_stress_delta(
	social: float,
	social_threshold: float,
	stress_rate: float
):
	return _call_native_if_exists(
		"body_anxious_attachment_stress_delta",
		[
			social,
			social_threshold,
			stress_rate
		]
	)


## Delegates upper-needs best-skill normalization to native bridge.
## Returns null when native bridge/method is unavailable.
func body_upper_needs_best_skill_normalized(
	skill_levels: PackedInt32Array,
	max_level: int
):
	return _call_native_if_exists(
		"body_upper_needs_best_skill_normalized",
		[
			skill_levels,
			max_level
		]
	)


## Delegates upper-needs job alignment to native bridge.
## Returns null when native bridge/method is unavailable.
func body_upper_needs_job_alignment(
	job_code: int,
	craftsmanship: float,
	skill: float,
	hard_work: float,
	nature: float,
	independence: float
):
	return _call_native_if_exists(
		"body_upper_needs_job_alignment",
		[
			job_code,
			craftsmanship,
			skill,
			hard_work,
			nature,
			independence
		]
	)


## Delegates upper-needs combined step to native bridge.
## Returns null when native bridge/method is unavailable.
func body_upper_needs_step_packed(
	scalar_inputs: PackedFloat32Array,
	flag_inputs: PackedByteArray
):
	return _call_native_if_exists(
		"body_upper_needs_step_packed",
		[
			scalar_inputs,
			flag_inputs
		]
	)


## Delegates child parent-stress transfer computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_child_parent_stress_transfer(
	parent_stress: float,
	parent_dependency: float,
	attachment_code: int,
	caregiver_support_active: bool,
	buffer_power: float,
	contagion_input: float
):
	return _call_native_if_exists(
		"body_child_parent_stress_transfer",
		[
			parent_stress,
			parent_dependency,
			attachment_code,
			caregiver_support_active,
			buffer_power,
			contagion_input
		]
	)


## Delegates simultaneous ACE burst step to native bridge.
## Returns null when native bridge/method is unavailable.
func body_child_simultaneous_ace_step(
	prev_residual: float,
	severities: PackedFloat32Array
):
	return _call_native_if_exists(
		"body_child_simultaneous_ace_step",
		[
			prev_residual,
			severities
		]
	)


## Delegates child social-buffer attenuation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_child_social_buffered_intensity(
	intensity: float,
	attachment_quality: float,
	caregiver_present: bool,
	buffer_power: float
):
	return _call_native_if_exists(
		"body_child_social_buffered_intensity",
		[
			intensity,
			attachment_quality,
			caregiver_present,
			buffer_power
		]
	)


## Delegates child SHRP step to native bridge.
## Returns null when native bridge/method is unavailable.
func body_child_shrp_step(
	intensity: float,
	shrp_active: bool,
	shrp_override_threshold: float,
	vulnerability_mult: float
):
	return _call_native_if_exists(
		"body_child_shrp_step",
		[
			intensity,
			shrp_active,
			shrp_override_threshold,
			vulnerability_mult
		]
	)


## Delegates child stress type classification to native bridge.
## Returns null when native bridge/method is unavailable.
func body_child_stress_type_code(
	intensity: float,
	attachment_present: bool,
	attachment_quality: float
):
	return _call_native_if_exists(
		"body_child_stress_type_code",
		[
			intensity,
			attachment_present,
			attachment_quality
		]
	)


## Delegates child stress state apply step to native bridge.
## Returns null when native bridge/method is unavailable.
func body_child_stress_apply_step(
	resilience: float,
	reserve: float,
	stress: float,
	allostatic: float,
	intensity: float,
	spike_mult: float,
	vulnerability_mult: float,
	break_threshold_mult: float,
	stress_type_code: int
):
	return _call_native_if_exists(
		"body_child_stress_apply_step",
		[
			resilience,
			reserve,
			stress,
			allostatic,
			intensity,
			spike_mult,
			vulnerability_mult,
			break_threshold_mult,
			stress_type_code
		]
	)


## Delegates child parent-transfer stress apply step to native bridge.
## Returns null when native bridge/method is unavailable.
func body_child_parent_transfer_apply_step(
	current_stress: float,
	transferred: float,
	transfer_threshold: float,
	transfer_scale: float,
	stress_clamp_max: float
):
	return _call_native_if_exists(
		"body_child_parent_transfer_apply_step",
		[
			current_stress,
			transferred,
			transfer_threshold,
			transfer_scale,
			stress_clamp_max
		]
	)


## Delegates child deprivation-damage accumulation step to native bridge.
## Returns null when native bridge/method is unavailable.
func body_child_deprivation_damage_step(current_damage: float, damage_rate: float):
	return _call_native_if_exists(
		"body_child_deprivation_damage_step",
		[
			current_damage,
			damage_rate
		]
	)


## Delegates child stage code classification from age ticks to native bridge.
## Returns null when native bridge/method is unavailable.
func body_child_stage_code_from_age_ticks(
	age_ticks: int,
	infant_max_years: float,
	toddler_max_years: float,
	child_max_years: float,
	teen_max_years: float
):
	return _call_native_if_exists(
		"body_child_stage_code_from_age_ticks",
		[
			age_ticks,
			infant_max_years,
			toddler_max_years,
			child_max_years,
			teen_max_years
		]
	)


## Delegates stress rebound apply step to native bridge.
## Returns null when native bridge/method is unavailable.
func body_stress_rebound_apply_step(
	stress: float,
	hidden_threat_accumulator: float,
	total_rebound: float,
	stress_clamp_max: float
):
	return _call_native_if_exists(
		"body_stress_rebound_apply_step",
		[
			stress,
			hidden_threat_accumulator,
			total_rebound,
			stress_clamp_max
		]
	)


## Delegates stress event injection apply step to native bridge.
## Returns null when native bridge/method is unavailable.
func body_stress_injection_apply_step(
	stress: float,
	final_instant: float,
	final_per_tick: float,
	trace_threshold: float,
	stress_clamp_max: float
):
	return _call_native_if_exists(
		"body_stress_injection_apply_step",
		[
			stress,
			final_instant,
			final_per_tick,
			trace_threshold,
			stress_clamp_max
		]
	)


## Delegates shaken countdown step to native bridge.
## Returns null when native bridge/method is unavailable.
func body_stress_shaken_countdown_step(shaken_remaining: int):
	return _call_native_if_exists(
		"body_stress_shaken_countdown_step",
		[
			shaken_remaining
		]
	)


## Delegates stress support score computation to native bridge.
## Returns null when native bridge/method is unavailable.
func body_stress_support_score(strengths: PackedFloat32Array):
	return _call_native_if_exists(
		"body_stress_support_score",
		[
			strengths
		]
	)


func _get_native_bridge() -> Object:
	_ensure_gdextension_loaded()
	if _native_checked and _native_bridge != null:
		return _native_bridge
	_native_checked = true

	for i in range(_NATIVE_SINGLETON_CANDIDATES.size()):
		var singleton_name: String = _NATIVE_SINGLETON_CANDIDATES[i]
		if not Engine.has_singleton(singleton_name):
			continue
		var singleton_obj: Object = Engine.get_singleton(singleton_name)
		if singleton_obj == null:
			continue
		for j in range(_PATHFIND_METHOD_CANDIDATES.size()):
			var method_name: String = _PATHFIND_METHOD_CANDIDATES[j]
			if singleton_obj.has_method(method_name):
				_native_bridge = singleton_obj
				_resolved_pathfind_backend_cached = false
				_resolved_pathfind_backend_cache = ""
				_gpu_pathfinding_capability_cached = false
				_gpu_pathfinding_capability = false
				_pathfind_method_name = method_name
				_pathfind_xy_method_name = _pick_method(
					_PATHFIND_XY_METHOD_CANDIDATES, ""
				)
				_pathfind_batch_method_name = _pick_method(
					_PATHFIND_BATCH_METHOD_CANDIDATES, ""
				)
				_pathfind_batch_xy_method_name = _pick_method(
					_PATHFIND_BATCH_XY_METHOD_CANDIDATES, ""
				)
				_set_pathfind_backend_method_name = _pick_method(
					_SET_PATHFIND_BACKEND_METHOD_CANDIDATES, ""
				)
				_get_pathfind_backend_method_name = _pick_method(
					_GET_PATHFIND_BACKEND_METHOD_CANDIDATES, ""
				)
				_resolve_pathfind_backend_method_name = _pick_method(
					_RESOLVE_PATHFIND_BACKEND_METHOD_CANDIDATES, ""
				)
				return _native_bridge

	for i in range(_NATIVE_SINGLETON_CANDIDATES.size()):
		var native_class_name: String = _NATIVE_SINGLETON_CANDIDATES[i]
		if not ClassDB.class_exists(native_class_name):
			continue
		var instance: Object = ClassDB.instantiate(native_class_name)
		if instance == null:
			continue
		for j in range(_PATHFIND_METHOD_CANDIDATES.size()):
			var method_name: String = _PATHFIND_METHOD_CANDIDATES[j]
			if instance.has_method(method_name):
				_native_bridge = instance
				_resolved_pathfind_backend_cached = false
				_resolved_pathfind_backend_cache = ""
				_gpu_pathfinding_capability_cached = false
				_gpu_pathfinding_capability = false
				_pathfind_method_name = method_name
				_pathfind_xy_method_name = _pick_method(
					_PATHFIND_XY_METHOD_CANDIDATES, ""
				)
				_pathfind_batch_method_name = _pick_method(
					_PATHFIND_BATCH_METHOD_CANDIDATES, ""
				)
				_pathfind_batch_xy_method_name = _pick_method(
					_PATHFIND_BATCH_XY_METHOD_CANDIDATES, ""
				)
				_set_pathfind_backend_method_name = _pick_method(
					_SET_PATHFIND_BACKEND_METHOD_CANDIDATES, ""
				)
				_get_pathfind_backend_method_name = _pick_method(
					_GET_PATHFIND_BACKEND_METHOD_CANDIDATES, ""
				)
				_resolve_pathfind_backend_method_name = _pick_method(
					_RESOLVE_PATHFIND_BACKEND_METHOD_CANDIDATES, ""
				)
				return _native_bridge

	return null


func _prefer_gpu() -> bool:
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return false

	var resolved_backend: String = _resolve_pathfinding_backend_cached(bridge)
	if resolved_backend != "":
		return resolved_backend == "gpu"

	if not Engine.has_singleton("ComputeBackend"):
		return false
	var backend: Object = Engine.get_singleton("ComputeBackend")
	if backend == null:
		return false
	var resolved_mode: String = ""
	if backend.has_method("resolve_mode_for_domain"):
		resolved_mode = str(backend.call("resolve_mode_for_domain", "pathfinding"))
	elif backend.has_method("resolve_mode"):
		resolved_mode = str(backend.call("resolve_mode"))
	if resolved_mode != "gpu":
		return false

	return _resolve_gpu_pathfinding_capability(bridge)


func _resolve_desired_pathfinding_backend_mode() -> String:
	if not Engine.has_singleton("ComputeBackend"):
		return "auto"
	var backend: Object = Engine.get_singleton("ComputeBackend")
	if backend == null:
		return "auto"
	var mode: String = "auto"
	if backend.has_method("get_mode_for_domain"):
		mode = str(backend.call("get_mode_for_domain", "pathfinding"))
	elif backend.has_method("get_mode"):
		mode = str(backend.call("get_mode"))
	if mode == "cpu":
		return "cpu"
	if mode == "gpu_force":
		return "gpu"
	return "auto"


func _sync_pathfinding_backend_mode(bridge: Object) -> void:
	var method_name: String = _set_pathfind_backend_method_name
	if method_name == "":
		method_name = _pick_method(_SET_PATHFIND_BACKEND_METHOD_CANDIDATES, "")
		_set_pathfind_backend_method_name = method_name
	if method_name == "":
		return
	var desired_mode: String = _resolve_desired_pathfinding_backend_mode()
	if desired_mode == _last_synced_pathfind_backend_mode:
		return

	var applied: Variant = bridge.call(method_name, desired_mode)
	if applied is bool and not bool(applied):
		return
	_last_synced_pathfind_backend_mode = desired_mode
	_resolved_pathfind_backend_cached = false
	_resolved_pathfind_backend_cache = ""


func _resolve_pathfinding_backend_cached(bridge: Object) -> String:
	if _resolved_pathfind_backend_cached:
		return _resolved_pathfind_backend_cache
	_sync_pathfinding_backend_mode(bridge)
	if _resolved_pathfind_backend_cached:
		return _resolved_pathfind_backend_cache
	var resolved: String = _resolve_pathfinding_backend_mode(bridge)
	if resolved == "":
		return ""
	_resolved_pathfind_backend_cache = resolved
	_resolved_pathfind_backend_cached = true
	return resolved


func _resolve_gpu_pathfinding_capability(bridge: Object) -> bool:
	if _gpu_pathfinding_capability_cached:
		return _gpu_pathfinding_capability
	var capability: bool = false
	if bridge.has_method("has_gpu_pathfinding"):
		capability = bool(bridge.call("has_gpu_pathfinding"))
	else:
		capability = _pick_method(_PATHFIND_GPU_METHOD_CANDIDATES, "").length() > 0
	_gpu_pathfinding_capability = capability
	_gpu_pathfinding_capability_cached = true
	return capability


func _resolve_pathfinding_backend_mode(bridge: Object) -> String:
	var method_name: String = _resolve_pathfind_backend_method_name
	if method_name == "":
		method_name = _pick_method(_RESOLVE_PATHFIND_BACKEND_METHOD_CANDIDATES, "")
		_resolve_pathfind_backend_method_name = method_name
	if method_name == "":
		return ""
	return str(bridge.call(method_name))


func _pick_method(candidates: Array[String], fallback: String) -> String:
	if _native_bridge == null:
		return fallback
	for i in range(candidates.size()):
		var candidate: String = candidates[i]
		if _native_bridge.has_method(candidate):
			return candidate
	return fallback


func _call_native_if_exists(method_name: String, args: Array):
	var bridge: Object = _get_native_bridge()
	if bridge == null:
		return null
	if not bridge.has_method(method_name):
		return null
	return bridge.callv(method_name, args)


func _get_native_runtime() -> Object:
	_ensure_gdextension_loaded()
	if _native_runtime_checked and _native_runtime != null and (
		_native_runtime_has_required_surface(_native_runtime)
		or _runtime_recovery_attempted
	):
		return _native_runtime
	_native_runtime_checked = true

	for i in range(_RUNTIME_CLASS_CANDIDATES.size()):
		var runtime_class_name: String = _RUNTIME_CLASS_CANDIDATES[i]
		if not ClassDB.class_exists(runtime_class_name):
			continue
		var instance: Object = ClassDB.instantiate(runtime_class_name)
		if instance == null:
			continue
		if instance.has_method("runtime_init") and instance.has_method("runtime_tick_frame"):
			if not _native_runtime_has_required_surface(instance) and _recover_runtime_extension():
				_native_runtime_checked = false
				return _get_native_runtime()
			_native_runtime = instance
		return _native_runtime
	return _native_runtime


func _ensure_gdextension_loaded() -> void:
	if _gdextension_checked:
		return
	_gdextension_checked = true
	if not ClassDB.class_exists("GDExtensionManager"):
		return
	var manager: Object = Engine.get_singleton("GDExtensionManager")
	if manager == null:
		return
	if not manager.has_method("is_extension_loaded"):
		return
	var loaded: bool = bool(manager.call("is_extension_loaded", _GDEXTENSION_PATH))
	if loaded:
		return
	if not manager.has_method("load_extension"):
		return
	var load_result: int = int(manager.call("load_extension", _GDEXTENSION_PATH))
	if load_result != OK:
		push_warning("[SimBridge] Failed to load GDExtension: %s (error=%d)" % [_GDEXTENSION_PATH, load_result])


func _native_runtime_has_required_surface(runtime: Object) -> bool:
	if runtime == null:
		return false
	for i in range(_RUNTIME_REQUIRED_METHODS.size()):
		var method_name: String = _RUNTIME_REQUIRED_METHODS[i]
		if not runtime.has_method(method_name):
			return false
	return true


func _recover_runtime_extension() -> bool:
	if _runtime_recovery_attempted:
		return false
	_runtime_recovery_attempted = true
	if not OS.is_debug_build():
		push_warning("[SimBridge] Native runtime is stale, but automatic sim-bridge rebuild is only available in debug builds.")
		return false
	var build_ok: bool = _rebuild_sim_bridge_extension()
	if not build_ok:
		push_warning("[SimBridge] Native runtime is stale and sim-bridge rebuild failed.")
		return false
	var reload_ok: bool = _reload_gdextension()
	if not reload_ok:
		push_warning("[SimBridge] sim-bridge rebuild completed but GDExtension reload failed.")
		return false
	_reset_native_extension_cache()
	return true


func _rebuild_sim_bridge_extension() -> bool:
	var rust_dir: String = ProjectSettings.globalize_path("res://rust")
	var escaped_rust_dir: String = rust_dir.replace("\"", "\\\"")
	var output: Array = []
	var exit_code: int = 0
	if OS.has_feature("windows"):
		exit_code = OS.execute(
			"cmd",
			["/c", "cd /d \"%s\" && cargo build -p sim-bridge" % escaped_rust_dir],
			output,
			true
		)
	else:
		exit_code = OS.execute(
			"/bin/zsh",
			["-lc", "cd \"%s\" && cargo build -p sim-bridge" % escaped_rust_dir],
			output,
			true
		)
	if exit_code == OK:
		return true
	push_warning("[SimBridge] cargo build -p sim-bridge failed: %s" % "\n".join(output))
	return false


func _reload_gdextension() -> bool:
	if not ClassDB.class_exists("GDExtensionManager"):
		return false
	var manager: Object = Engine.get_singleton("GDExtensionManager")
	if manager == null:
		return false
	var unloaded: bool = true
	if manager.has_method("unload_extension"):
		unloaded = int(manager.call("unload_extension", _GDEXTENSION_PATH)) == OK
	if not unloaded:
		return false
	if not manager.has_method("load_extension"):
		return false
	return int(manager.call("load_extension", _GDEXTENSION_PATH)) == OK


func _reset_native_extension_cache() -> void:
	_native_checked = false
	_native_bridge = null
	_native_runtime_checked = false
	_native_runtime = null
	_pathfind_method_name = ""
	_pathfind_xy_method_name = ""
	_pathfind_batch_method_name = ""
	_pathfind_batch_xy_method_name = ""
	_set_pathfind_backend_method_name = ""
	_get_pathfind_backend_method_name = ""
	_resolve_pathfind_backend_method_name = ""
	_last_synced_pathfind_backend_mode = ""
	_resolved_pathfind_backend_cache = ""
	_resolved_pathfind_backend_cached = false
	_gpu_pathfinding_capability_cached = false
	_gpu_pathfinding_capability = false
