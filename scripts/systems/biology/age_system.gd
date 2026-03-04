extends "res://scripts/core/simulation/simulation_system.gd"

## Checks age stage transitions, emits growth notifications,
## and applies yearly personality maturation.
## Runs every 50 ticks (~4 days).

const PersonalityMaturation = preload("res://scripts/systems/psychology/personality_maturation.gd")
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_AGE_SPEED_METHOD: String = "body_age_body_speed"
const _SIM_BRIDGE_AGE_STRENGTH_METHOD: String = "body_age_body_strength"
var _entity_manager: RefCounted
var _personality_maturation: RefCounted
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func _init() -> void:
	system_name = "aging"
	priority = 48
	tick_interval = 50


func init(entity_manager: RefCounted, rng: RandomNumberGenerator = null) -> void:
	_entity_manager = entity_manager
	if rng != null:
		_personality_maturation = PersonalityMaturation.new()
		_personality_maturation.init(rng)


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
	and node.has_method(_SIM_BRIDGE_AGE_SPEED_METHOD) \
	and node.has_method(_SIM_BRIDGE_AGE_STRENGTH_METHOD):
		_sim_bridge = node
	return _sim_bridge

func _on_stage_changed(entity: RefCounted, old_stage: String, new_stage: String, tick: int) -> void:
	var age_years: float = GameConfig.get_age_years(entity.age)
	emit_event("age_stage_changed", {
		"entity_id": entity.id,
		"entity_name": entity.entity_name,
		"from_stage": old_stage,
		"to_stage": new_stage,
		"age_years": age_years,
		"tick": tick,
	})
	match new_stage:
		"teen":
			SimulationBus.emit_signal("ui_notification",
				"%s grew up (teen, %.0fy)" % [entity.entity_name, age_years], "growth")
		"adult":
			SimulationBus.emit_signal("ui_notification",
				"%s is now adult (%.0fy)" % [entity.entity_name, age_years], "growth")
		"elder":
			SimulationBus.emit_signal("ui_notification",
				"%s became elder (%.0fy)" % [entity.entity_name, age_years], "growth")
			# Elders can't be builders — clear for reassignment
			if entity.job == "builder":
				entity.job = "none"
