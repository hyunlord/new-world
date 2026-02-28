extends Node

## Core simulation events
@warning_ignore("unused_signal")
signal simulation_event(event: Dictionary)
@warning_ignore("unused_signal")
signal ui_notification(message: String, type: String)

## Entity selection
@warning_ignore("unused_signal")
signal entity_selected(entity_id: int)
@warning_ignore("unused_signal")
signal entity_deselected()

## Building selection
@warning_ignore("unused_signal")
signal building_selected(building_id: int)
@warning_ignore("unused_signal")
signal building_deselected()

## Simulation state
@warning_ignore("unused_signal")
signal tick_completed(tick: int)
@warning_ignore("unused_signal")
signal speed_changed(speed_index: int)
@warning_ignore("unused_signal")
signal pause_changed(paused: bool)

## Entity lifecycle (for ChronicleSystem)
@warning_ignore("unused_signal")
signal entity_born(entity_id: int, entity_name: String, parent_ids: Array, tick: int)
@warning_ignore("unused_signal")
signal entity_died(entity_id: int, entity_name: String, cause: String, age_years: float, tick: int)
@warning_ignore("unused_signal")
signal couple_formed(entity_a_id: int, entity_a_name: String, entity_b_id: int, entity_b_name: String, tick: int)

## Camera follow
@warning_ignore("unused_signal")
signal follow_entity_requested(entity_id: int)
@warning_ignore("unused_signal")
signal follow_entity_stopped()

## Trauma Scar events (Phase 3A)
@warning_ignore("unused_signal")
signal scar_acquired(data: Dictionary)
@warning_ignore("unused_signal")
signal scar_reactivated(data: Dictionary)

## Mental Break events (Phase 4)
@warning_ignore("unused_signal")
signal mental_break_started(entity_id: int, break_type: String, tick: int)
@warning_ignore("unused_signal")
signal mental_break_recovered(entity_id: int, tick: int)

## Stat threshold crossing (Phase 3 — StatThresholdSystem)
## direction: "entered" (조건 진입) or "exited" (조건 해제)
@warning_ignore("unused_signal")
signal stat_threshold_crossed(entity_id: int, stat_id: String, effect: String, direction: String)

## Emitted when an entity's skill level increases
## skill_id: StringName e.g. &"SKILL_FORAGING"
## old_level: int previous level (0–100)
## new_level: int new level (0–100)
@warning_ignore("unused_signal")
signal skill_leveled_up(entity_id: int, entity_name: String, skill_id: StringName, old_level: int, new_level: int, tick: int)

## [Anderson 1982 ACT*] Emitted when a skill threshold grants a new action to an entity.
## action_id: StringName e.g. &"UNLOCK_ACTION_HERB_GATHER"
## skill_id:  StringName e.g. &"SKILL_FORAGING"
## at_level:  int — the skill level that triggered the unlock
@warning_ignore("unused_signal")
signal skill_action_unlocked(entity_id: int, entity_name: String, action_id: StringName, skill_id: StringName, at_level: int, tick: int)

## [Weber 1922] Emitted when a settlement elects a new leader.
@warning_ignore("unused_signal")
signal leader_elected(settlement_id: int, leader_id: int, leader_name: String, charisma: float, tick: int)

## Emitted when a settlement's leader dies or leaves with no replacement yet.
@warning_ignore("unused_signal")
signal leader_lost(settlement_id: int, tick: int)

## [Fiske 2007] Reputation event (direct observation or interaction outcome)
@warning_ignore("unused_signal")
signal reputation_event(data: Dictionary)

## [Dunbar 1997] Gossip propagation between agents
@warning_ignore("unused_signal")
signal gossip_spread(data: Dictionary)

## [Boehm 1999] Status tier transition
@warning_ignore("unused_signal")
signal status_tier_changed(entity_id: int, old_tier: String, new_tier: String, tick: int)

## [Kohler 2017] Settlement stratification phase transition
@warning_ignore("unused_signal")
signal stratification_phase_changed(settlement_id: int, old_phase: String, new_phase: String, tick: int)

## [Holland 1959] Occupation changed due to skill progression
@warning_ignore("unused_signal")
signal occupation_changed(entity_id: int, entity_name: String, old_occupation: String, new_occupation: String, tick: int)

## [Barth 1969] Title granted to entity
@warning_ignore("unused_signal")
signal title_granted(entity_id: int, entity_name: String, title_id: StringName, tick: int)

## [Barth 1969] Title revoked from entity
@warning_ignore("unused_signal")
signal title_revoked(entity_id: int, entity_name: String, title_id: StringName, tick: int)

## [Henrich 2004, Tainter 1988] Tech V2 state machine transitions
@warning_ignore("unused_signal")
signal tech_state_changed(settlement_id: int, tech_id: String, old_state: String, new_state: String, tick: int)
@warning_ignore("unused_signal")
signal tech_regression_started(settlement_id: int, tech_id: String, atrophy_years: int, grace_years: int, tick: int)
@warning_ignore("unused_signal")
signal tech_lost(settlement_id: int, tech_id: String, cultural_memory: float, tick: int)
@warning_ignore("unused_signal")
signal tech_rediscovered(settlement_id: int, tech_id: String, rediscovered_count: int, tick: int)

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
