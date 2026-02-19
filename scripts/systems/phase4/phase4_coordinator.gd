extends Node

## Phase4Coordinator — Dependency Injection & Cross-System Signal Hub
## Glue Node that connects Phase 4 systems and routes cross-system events
## that individual systems cannot self-coordinate (mental break hooks).

class_name Phase4Coordinator

# ── System References ──────────────────────────────────────────────────
var _coping_system: RefCounted
var _morale_system: RefCounted
var _contagion_system: RefCounted
var _stress_system: RefCounted
var _entity_manager: RefCounted

var _initialized: bool = false


## Called by main.gd after all Phase 4 systems are instantiated.
## Injects mutual dependencies and connects SimulationBus signals.
func init_phase4(
	coping_s: RefCounted,
	morale_s: RefCounted,
	contagion_s: RefCounted,
	stress_s: RefCounted,
	entity_manager: RefCounted
) -> void:
	_coping_system = coping_s
	_morale_system = morale_s
	_contagion_system = contagion_s
	_stress_system = stress_s
	_entity_manager = entity_manager

	_connect_signals()
	_initialized = true


func _connect_signals() -> void:
	## Connect to SimulationBus for cross-system mental break events
	if not SimulationBus.has_signal("mental_break_started"):
		return
	if not SimulationBus.is_connected("mental_break_started", _on_mental_break_started):
		SimulationBus.connect("mental_break_started", _on_mental_break_started)
	if not SimulationBus.is_connected("mental_break_recovered", _on_mental_break_recovered):
		SimulationBus.connect("mental_break_recovered", _on_mental_break_recovered)


## Called by main.gd on every simulation tick (optional cross-system work).
func on_simulation_tick(tick: int) -> void:
	pass  # Phase 4 systems run via SimulationEngine; coordinator handles signal events only


## mental_break_started — notify coping system and apply morale penalty
func _on_mental_break_started(entity_id: int, break_type: String, tick: int) -> void:
	if _coping_system != null and _coping_system.has_method("on_mental_break_started"):
		_coping_system.on_mental_break_started(entity_id, break_type, tick)
	if _morale_system != null and _morale_system.has_method("on_mental_break_penalty"):
		_morale_system.on_mental_break_penalty(entity_id, break_type, tick)


## mental_break_recovered — notify coping and morale systems
func _on_mental_break_recovered(entity_id: int, tick: int) -> void:
	if _coping_system != null and _coping_system.has_method("on_mental_break_recovered"):
		_coping_system.on_mental_break_recovered(entity_id, tick)
	if _morale_system != null and _morale_system.has_method("on_mental_break_recovered"):
		_morale_system.on_mental_break_recovered(entity_id, tick)


func _exit_tree() -> void:
	if SimulationBus.has_signal("mental_break_started") and SimulationBus.is_connected("mental_break_started", _on_mental_break_started):
		SimulationBus.disconnect("mental_break_started", _on_mental_break_started)
	if SimulationBus.has_signal("mental_break_recovered") and SimulationBus.is_connected("mental_break_recovered", _on_mental_break_recovered):
		SimulationBus.disconnect("mental_break_recovered", _on_mental_break_recovered)
