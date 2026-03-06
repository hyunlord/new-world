class_name DebugDataProvider
extends RefCounted

## Shared data core for debug panels.
## Wraps SimBridge debug API with has_method() guards.
## Panels NEVER call SimBridge directly — always go through this provider.

var _bridge: Node  # The SimBridge autoload Node


func _init(bridge: Node) -> void:
	_bridge = bridge


func enable_debug(enabled: bool) -> void:
	if _bridge and _bridge.has_method("enable_debug_mode"):
		_bridge.enable_debug_mode(enabled)


func get_system_perf() -> Dictionary:
	if _bridge and _bridge.has_method("get_system_perf"):
		var result = _bridge.get_system_perf()
		if result is Dictionary:
			return result
	return {}


func get_tick_history() -> PackedFloat32Array:
	if _bridge and _bridge.has_method("get_tick_history"):
		var result = _bridge.get_tick_history()
		if result is PackedFloat32Array:
			return result
	return PackedFloat32Array()


func get_config_all() -> Dictionary:
	if _bridge and _bridge.has_method("get_config_values"):
		var result = _bridge.get_config_values()
		if result is Dictionary:
			return result
	return {}


func set_config_value(key: String, value: float) -> bool:
	if _bridge and _bridge.has_method("set_config_value"):
		return bool(_bridge.set_config_value(key, value))
	return false


func get_tick() -> int:
	if _bridge and _bridge.has_method("get_tick"):
		return int(_bridge.get_tick())
	return 0


func get_entity_count() -> int:
	if _bridge and _bridge.has_method("runtime_get_entity_list"):
		var list = _bridge.runtime_get_entity_list()
		if list is Array:
			return list.size()
	return 0


func get_debug_summary() -> Dictionary:
	if _bridge and _bridge.has_method("get_debug_summary"):
		var result = _bridge.get_debug_summary()
		if result is Dictionary:
			return result
	return {}


func get_guardrail_status() -> Array:
	if _bridge and _bridge.has_method("get_guardrail_status"):
		var result = _bridge.get_guardrail_status()
		if result is Array:
			return result
	return []


func get_entity_detail(entity_id: int) -> Dictionary:
	if _bridge and _bridge.has_method("runtime_get_entity_detail"):
		var result = _bridge.runtime_get_entity_detail(entity_id)
		if result is Dictionary:
			return result
	return {}
