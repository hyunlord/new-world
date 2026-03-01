extends RefCounted

## [Salthouse 2009, 2012] Age-dependent intelligence modifiers.
## Fluid peaks early (~22), crystallized peaks late (~65), physical peaks ~28.
## Reference: const IntelligenceCurves = preload("res://scripts/systems/cognition/intelligence_curves.gd")
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_INTEL_ACTIVITY_METHOD: String = "body_cognition_activity_modifier"
const _SIM_BRIDGE_INTEL_ACE_DECLINE_METHOD: String = "body_cognition_ace_fluid_decline_mult"
static var _bridge_checked_static: bool = false
static var _sim_bridge_static: Object = null


## Get age modifier for a specific intelligence key.
## Returns 0.0~1.0 where 1.0 = peak performance.
static func get_age_modifier(intel_key: String, age_years: float) -> float:
	var curve: Array
	if intel_key in GameConfig.INTEL_GROUP_FLUID:
		curve = GameConfig.INTEL_CURVE_FLUID
	elif intel_key in GameConfig.INTEL_GROUP_PHYSICAL:
		curve = GameConfig.INTEL_CURVE_PHYSICAL
	else:
		curve = GameConfig.INTEL_CURVE_CRYSTALLIZED
	return _interpolate_curve(curve, age_years)


## Piecewise linear interpolation over [age, modifier] breakpoints.
static func _interpolate_curve(curve: Array, age: float) -> float:
	if curve.is_empty():
		return 1.0
	if age <= curve[0][0]:
		return curve[0][1]
	if age >= curve[curve.size() - 1][0]:
		return curve[curve.size() - 1][1]
	for i in range(1, curve.size()):
		if age <= curve[i][0]:
			var a0: float = curve[i - 1][0]
			var m0: float = curve[i - 1][1]
			var a1: float = curve[i][0]
			var m1: float = curve[i][1]
			var t: float = (age - a0) / maxf(a1 - a0, 0.001)
			return lerpf(m0, m1, t)
	return curve[curve.size() - 1][1]


## [Hertzog 2009] "Use it or lose it"
## active_skill_count: number of skills at level >= INTEL_ACTIVITY_SKILL_THRESHOLD
static func get_activity_modifier(active_skill_count: int) -> float:
	var bridge: Object = _get_sim_bridge_static()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_INTEL_ACTIVITY_METHOD,
			active_skill_count,
			float(GameConfig.INTEL_ACTIVITY_BUFFER),
			float(GameConfig.INTEL_INACTIVITY_ACCEL),
		)
		if rust_variant != null:
			return float(rust_variant)
	if active_skill_count >= 1:
		return GameConfig.INTEL_ACTIVITY_BUFFER
	else:
		return GameConfig.INTEL_INACTIVITY_ACCEL


## [Lupien 2009] High ACE → 1.5x fluid decline acceleration
static func get_ace_fluid_decline_mult(ace_penalty: float) -> float:
	var bridge: Object = _get_sim_bridge_static()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_INTEL_ACE_DECLINE_METHOD,
			ace_penalty,
			float(GameConfig.INTEL_ACE_PENALTY_MINOR),
			float(GameConfig.INTEL_ACE_FLUID_DECLINE_MULT),
		)
		if rust_variant != null:
			return float(rust_variant)
	if ace_penalty >= GameConfig.INTEL_ACE_PENALTY_MINOR:
		return GameConfig.INTEL_ACE_FLUID_DECLINE_MULT
	return 1.0


static func _get_sim_bridge_static() -> Object:
	if _bridge_checked_static:
		return _sim_bridge_static
	_bridge_checked_static = true
	var tree: SceneTree = Engine.get_main_loop() as SceneTree
	if tree == null:
		return null
	var root: Node = tree.get_root()
	if root == null:
		return null
	var node: Node = root.get_node_or_null(_SIM_BRIDGE_NODE_NAME)
	if node != null \
	and node.has_method(_SIM_BRIDGE_INTEL_ACTIVITY_METHOD) \
	and node.has_method(_SIM_BRIDGE_INTEL_ACE_DECLINE_METHOD):
		_sim_bridge_static = node
	return _sim_bridge_static
