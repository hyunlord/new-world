class_name DebugDataProvider
extends RefCounted

## Shared data core for debug panels. Wraps SimBridge debug API with caching.
## Both in-game overlay and EditorPlugin consume from this provider.

var _sim_bridge: Object  # WorldSimRuntime autoload reference
var _cache: Dictionary = {}
var _cache_ticks: Dictionary = {}

const CACHE_INTERVALS := {
	"system_perf": 60,
	"world_stats": 100,
	"guardrails": 100,
	"config": -1,   # only on change (manual invalidation)
	"events": 1,    # every tick
	"debug_summary": 10,
}

func _init(sim_bridge) -> void:
	_sim_bridge = sim_bridge

func get_system_perf() -> Dictionary:
	return _get_cached("system_perf", func(): return _sim_bridge.get_system_perf())

func get_debug_summary() -> Dictionary:
	return _get_cached("debug_summary", func(): return _sim_bridge.get_debug_summary())

func get_config_all() -> Dictionary:
	return _get_cached("config", func(): return _sim_bridge.get_config_values())

func set_config_value(key: String, value: float) -> bool:
	var result: bool = _sim_bridge.set_config_value(key, value)
	if result:
		_cache.erase("config")  # invalidate cache on change
	return result

func get_guardrails() -> Array:
	return _get_cached("guardrails", func(): return _sim_bridge.get_guardrail_status())

func get_tick_history() -> PackedFloat32Array:
	return _sim_bridge.get_tick_history()

func query_entities(condition: String, threshold: float) -> PackedInt32Array:
	return _sim_bridge.query_entities_by_condition(condition, threshold)

func get_entity_detail(entity_id: int) -> Dictionary:
	return _sim_bridge.runtime_get_entity_detail(entity_id)

func enable_debug(enabled: bool) -> void:
	_sim_bridge.enable_debug_mode(enabled)

func get_current_tick() -> int:
	var summary := get_debug_summary()
	return summary.get("tick", 0) as int

# --- internal ---

func _get_cached(key: String, fetcher: Callable) -> Variant:
	var current_tick: int = get_current_tick()
	var interval: int = CACHE_INTERVALS.get(key, 60)
	if interval == -1:
		if not _cache.has(key):
			_cache[key] = fetcher.call()
		return _cache[key]
	if not _cache.has(key) or (current_tick - _cache_ticks.get(key, 0)) >= interval:
		_cache[key] = fetcher.call()
		_cache_ticks[key] = current_tick
	return _cache[key]
