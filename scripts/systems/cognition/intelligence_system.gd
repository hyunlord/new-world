extends "res://scripts/core/simulation/simulation_system.gd"

## [Gardner 1983 + CHC + Salthouse 2009 + Georgieff 2007 + Lupien 2009]
## Tick-based system: recomputes effective intelligences from potentials × age × environment.
## Reference: const IntelligenceSystem = preload("res://scripts/systems/cognition/intelligence_system.gd")

const IntelligenceCurves = preload("res://scripts/systems/cognition/intelligence_curves.gd")
const IntelligenceGeneratorScript = preload("res://scripts/systems/cognition/intelligence_generator.gd")
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_INTEL_EFFECTIVE_METHOD: String = "body_intelligence_effective_value"

var _entity_manager = null
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func init(entity_manager) -> void:
	system_name = "intelligence_system"
	priority = 18
	tick_interval = 50
	_entity_manager = entity_manager


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
	if node != null and node.has_method(_SIM_BRIDGE_INTEL_EFFECTIVE_METHOD):
		_sim_bridge = node
	return _sim_bridge

## [Georgieff 2007] Monitor hunger in critical window (0~2 years)
func _check_nutrition_damage(entity: RefCounted) -> void:
	if entity.age > GameConfig.INTEL_NUTRITION_CRIT_AGE_TICKS:
		return
	if entity.intel_nutrition_penalty >= GameConfig.INTEL_NUTRITION_MAX_PENALTY:
		return
	var hunger: float = entity.hunger
	if hunger < GameConfig.INTEL_NUTRITION_HUNGER_THRESHOLD:
		var severity: float = 1.0 - (hunger / GameConfig.INTEL_NUTRITION_HUNGER_THRESHOLD)
		var delta: float = GameConfig.INTEL_NUTRITION_PENALTY_PER_TICK * severity
		entity.intel_nutrition_penalty = minf(
			entity.intel_nutrition_penalty + delta,
			GameConfig.INTEL_NUTRITION_MAX_PENALTY
		)


## [Lupien 2009] Check trauma scars accumulated before age 12
func _check_ace_damage(entity: RefCounted, age_years: float) -> void:
	if age_years < GameConfig.INTEL_ACE_CRIT_AGE_YEARS:
		return
	if entity.intel_ace_penalty > 0.0:
		return
	var early_scars: int = 0
	var crit_age_ticks: int = int(GameConfig.INTEL_ACE_CRIT_AGE_YEARS * 365.0)
	for scar in entity.trauma_scars:
		var scar_tick: int = scar.get("acquired_tick", 0)
		var scar_age_ticks: int = scar_tick - entity.birth_tick
		if scar_age_ticks >= 0 and scar_age_ticks < crit_age_ticks:
			early_scars += 1
	if early_scars >= GameConfig.INTEL_ACE_SCARS_THRESHOLD_MAJOR:
		entity.intel_ace_penalty = GameConfig.INTEL_ACE_PENALTY_MAJOR
	elif early_scars >= GameConfig.INTEL_ACE_SCARS_THRESHOLD_MINOR:
		entity.intel_ace_penalty = GameConfig.INTEL_ACE_PENALTY_MINOR


## Recompute effective intelligences from potentials × age_curve × environment
func _update_effective_intelligence(entity: RefCounted, age_years: float) -> void:
	var active_count: int = 0
	for skill_id in entity.skill_levels:
		if int(entity.skill_levels[skill_id]) >= GameConfig.INTEL_ACTIVITY_SKILL_THRESHOLD:
			active_count += 1

	var activity_mod: float = IntelligenceCurves.get_activity_modifier(active_count)
	var ace_fluid_mult: float = IntelligenceCurves.get_ace_fluid_decline_mult(entity.intel_ace_penalty)
	var env_penalty: float = entity.intel_nutrition_penalty + entity.intel_ace_penalty
	var bridge: Object = _get_sim_bridge()

	for key in IntelligenceGeneratorScript.INTEL_KEYS:
		var potential: float = entity.intelligence_potentials.get(key, 0.5)
		var base_mod: float = IntelligenceCurves.get_age_modifier(key, age_years)
		var effective: float = 0.0
		var used_rust: bool = false
		if bridge != null:
			var rust_variant: Variant = bridge.call(
				_SIM_BRIDGE_INTEL_EFFECTIVE_METHOD,
				potential,
				base_mod,
				age_years,
				key in GameConfig.INTEL_GROUP_FLUID,
				activity_mod,
				ace_fluid_mult,
				env_penalty,
				0.02,
				0.98,
			)
			if rust_variant != null:
				effective = float(rust_variant)
				used_rust = true

		if not used_rust:
			var effective_mod: float = base_mod
			if base_mod < 1.0 and age_years > 25.0:
				var decline_amount: float = 1.0 - base_mod
				if key in GameConfig.INTEL_GROUP_FLUID:
					decline_amount *= activity_mod * ace_fluid_mult
				else:
					decline_amount *= activity_mod
				effective_mod = 1.0 - decline_amount
			effective = clampf((potential - env_penalty) * effective_mod, 0.02, 0.98)
		entity.intelligences[key] = effective
