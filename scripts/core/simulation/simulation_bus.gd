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

## Mental Break events (Phase 4)
signal mental_break_started(entity_id: int, break_type: String, tick: int)
signal mental_break_recovered(entity_id: int, tick: int)

## Stat threshold crossing (Phase 3 — StatThresholdSystem)
## direction: "entered" (조건 진입) or "exited" (조건 해제)
signal stat_threshold_crossed(entity_id: int, stat_id: String, effect: String, direction: String)

## Emitted when an entity's skill level increases
## skill_id: StringName e.g. &"SKILL_FORAGING"
## old_level: int previous level (0–100)
## new_level: int new level (0–100)
signal skill_leveled_up(entity_id: int, entity_name: String, skill_id: StringName, old_level: int, new_level: int, tick: int)

## [Anderson 1982 ACT*] Emitted when a skill threshold grants a new action to an entity.
## action_id: StringName e.g. &"UNLOCK_ACTION_HERB_GATHER"
## skill_id:  StringName e.g. &"SKILL_FORAGING"
## at_level:  int — the skill level that triggered the unlock
signal skill_action_unlocked(entity_id: int, entity_name: String, action_id: StringName, skill_id: StringName, at_level: int, tick: int)

## [Weber 1922] Emitted when a settlement elects a new leader.
signal leader_elected(settlement_id: int, leader_id: int, leader_name: String, charisma: float, tick: int)

## Emitted when a settlement's leader dies or leaves with no replacement yet.
signal leader_lost(settlement_id: int, tick: int)

## [Fiske 2007] Reputation event (direct observation or interaction outcome)
signal reputation_event(data: Dictionary)

## [Dunbar 1997] Gossip propagation between agents
signal gossip_spread(data: Dictionary)

## [Boehm 1999] Status tier transition
signal status_tier_changed(entity_id: int, old_tier: String, new_tier: String, tick: int)

## [Kohler 2017] Settlement stratification phase transition
signal stratification_phase_changed(settlement_id: int, old_phase: String, new_phase: String, tick: int)

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
