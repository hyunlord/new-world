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

## Entity lifecycle (for ChronicleSystem)
signal entity_born(entity_id: int, entity_name: String, parent_ids: Array, tick: int)
signal entity_died(entity_id: int, entity_name: String, cause: String, age_years: float, tick: int)
signal couple_formed(entity_a_id: int, entity_a_name: String, entity_b_id: int, entity_b_name: String, tick: int)

## Camera follow
signal follow_entity_requested(entity_id: int)
signal follow_entity_stopped()

## Trauma Scar events (Phase 3A)
signal scar_acquired(data: Dictionary)
signal scar_reactivated(data: Dictionary)

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
