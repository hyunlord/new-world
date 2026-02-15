extends RefCounted

var system_name: String = "base"
var priority: int = 0
var tick_interval: int = 1
var is_active: bool = true

## Override in subclasses to implement per-tick logic
func execute_tick(_tick: int) -> void:
	pass

## Helper to emit events through the global SimulationBus
func emit_event(event_type: String, data: Dictionary = {}) -> void:
	SimulationBus.emit_event(event_type, data)
