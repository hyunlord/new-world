extends Node

## Core simulation events
signal simulation_event(event: Dictionary)
signal ui_notification(message: String, type: String)

## Entity selection
signal entity_selected(entity_id: int)
signal entity_deselected()

## Building selection
signal building_selected(building_id: int)
signal building_deselected()

## Simulation state
signal tick_completed(tick: int)
signal speed_changed(speed_index: int)
signal pause_changed(paused: bool)

## Emit a structured simulation event via the bus
func emit_event(event_type: String, data: Dictionary = {}) -> void:
	var event := {
		"type": event_type,
		"tick": data.get("tick", -1),
		"timestamp": Time.get_ticks_msec(),
	}
	event.merge(data)
	simulation_event.emit(event)

## Send a UI notification
func notify(message: String, type: String = "info") -> void:
	ui_notification.emit(message, type)
