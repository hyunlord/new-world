extends Node

## Phase4Coordinator — Dependency Injection & Cross-System Signal Hub
## Glue Node that connects Phase 4 systems and routes cross-system events
## that individual systems cannot self-coordinate (mental break hooks).

const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_BREAK_TYPE_CODE_METHOD: String = "body_psychology_break_type_code"
const _SIM_BRIDGE_BREAK_TYPE_LABEL_METHOD: String = "body_psychology_break_type_label"

# ── System References ──────────────────────────────────────────────────
var _coping_system: RefCounted
var _morale_system: RefCounted
var _contagion_system: RefCounted
var _stress_system: RefCounted
var _entity_manager: RefCounted

var _initialized: bool = false
var _active_break_types: Dictionary = {}  # entity_id → break code (or string fallback)
var _bridge_checked: bool = false
var _sim_bridge: Object = null


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


func _get_sim_bridge() -> Object:
	if _bridge_checked:
		return _sim_bridge
	_bridge_checked = true
	var tree: SceneTree = Engine.get_main_loop() as SceneTree
	if tree == null:
		return null
	var root: Node = tree.get_root()
	if root == null:
		return null
	var node: Node = root.get_node_or_null(_SIM_BRIDGE_NODE_NAME)
	if node != null \
	and node.has_method(_SIM_BRIDGE_BREAK_TYPE_CODE_METHOD) \
	and node.has_method(_SIM_BRIDGE_BREAK_TYPE_LABEL_METHOD):
		_sim_bridge = node
	return _sim_bridge


func _connect_signals() -> void:
	## Connect to SimulationBus for cross-system mental break events
	if not SimulationBus.has_signal("mental_break_started"):
		return
	if not SimulationBus.is_connected("mental_break_started", _on_mental_break_started):
		SimulationBus.connect("mental_break_started", _on_mental_break_started)
	if not SimulationBus.is_connected("mental_break_recovered", _on_mental_break_recovered):
		SimulationBus.connect("mental_break_recovered", _on_mental_break_recovered)


## Called by main.gd on every simulation tick (optional cross-system work).
func on_simulation_tick(_tick: int) -> void:
	pass  # Phase 4 systems run via SimulationEngine; coordinator handles signal events only


## mental_break_started — notify coping system and apply morale penalty
func _on_mental_break_started(entity_id: int, break_type: String, tick: int) -> void:
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var code_variant: Variant = bridge.call(_SIM_BRIDGE_BREAK_TYPE_CODE_METHOD, break_type)
		if code_variant is int:
			_active_break_types[entity_id] = int(code_variant)
		else:
			_active_break_types[entity_id] = break_type
	else:
		_active_break_types[entity_id] = break_type
	var entity = _entity_manager.get_entity(entity_id) if _entity_manager != null else null
	if _coping_system != null and _coping_system.has_method("on_mental_break_started"):
		_coping_system.on_mental_break_started(entity, break_type)
	if _morale_system != null and _morale_system.has_method("on_mental_break_penalty"):
		_morale_system.on_mental_break_penalty(entity_id, break_type, tick)


## mental_break_recovered — notify coping and morale systems
func _on_mental_break_recovered(entity_id: int, tick: int) -> void:
	var cached_value: Variant = _active_break_types.get(entity_id, "")
	var cached_break_type: String = ""
	if cached_value is int:
		var bridge: Object = _get_sim_bridge()
		if bridge != null:
			var label_variant: Variant = bridge.call(_SIM_BRIDGE_BREAK_TYPE_LABEL_METHOD, int(cached_value))
			if label_variant is String:
				cached_break_type = str(label_variant)
		if cached_break_type.is_empty():
			cached_break_type = ""
	elif cached_value is String:
		cached_break_type = str(cached_value)
	_active_break_types.erase(entity_id)
	var entity = _entity_manager.get_entity(entity_id) if _entity_manager != null else null
	if _coping_system != null and _coping_system.has_method("on_mental_break_recovered"):
		_coping_system.on_mental_break_recovered(entity, cached_break_type, tick)
	if _morale_system != null and _morale_system.has_method("on_mental_break_recovered"):
		_morale_system.on_mental_break_recovered(entity_id, tick)


func _exit_tree() -> void:
	if SimulationBus.has_signal("mental_break_started") and SimulationBus.is_connected("mental_break_started", _on_mental_break_started):
		SimulationBus.disconnect("mental_break_started", _on_mental_break_started)
	if SimulationBus.has_signal("mental_break_recovered") and SimulationBus.is_connected("mental_break_recovered", _on_mental_break_recovered):
		SimulationBus.disconnect("mental_break_recovered", _on_mental_break_recovered)
