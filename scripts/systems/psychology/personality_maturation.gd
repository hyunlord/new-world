extends RefCounted

## Age-based personality maturation using OU (Ornstein-Uhlenbeck) process.
## Called once per game year for each entity.
## Ashton & Lee (2016): H increases ~+1 SD from 18→60, E/X mild increase.
## No class_name - use preload("res://scripts/systems/personality_maturation.gd").

const PersonalityDataScript = preload("res://scripts/core/entity/personality_data.gd")
const TraitSystem = preload("res://scripts/systems/psychology/trait_system.gd")
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_LINEAR_TARGET_METHOD: String = "body_personality_linear_target"

var _theta: float = 0.03
var _sigma: float = 0.03
var _maturation_targets: Dictionary = {}
var _rng: RandomNumberGenerator
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func init(rng: RandomNumberGenerator) -> void:
	_rng = rng
	_load_maturation_parameters()


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
	if node != null and node.has_method(_SIM_BRIDGE_LINEAR_TARGET_METHOD):
		_sim_bridge = node
	return _sim_bridge


func _load_maturation_parameters() -> void:
	var dist = SpeciesManager.personality_distribution
	var ou = dist.get("ou_parameters", {})
	_theta = float(ou.get("theta", 0.03))
	_sigma = float(ou.get("sigma", 0.03))
	_maturation_targets = dist.get("maturation", {})


## Box-Muller normal random
func _randfn(mean: float, std: float) -> float:
	var u1: float = _rng.randf()
	var u2: float = _rng.randf()
	if u1 < 1e-10:
		u1 = 1e-10
	return mean + std * sqrt(-2.0 * log(u1)) * cos(2.0 * PI * u2)


## Apply one year of maturation to a PersonalityData.
## age: entity's current age in years (integer).
func apply_maturation(pd: RefCounted, age: int) -> void:
	var axis_ids: Array = PersonalityDataScript.AXIS_IDS

	for i in range(axis_ids.size()):
		var aid: String = axis_ids[i]
		var target: float = _get_maturation_target(aid, age)
		var fkeys: Array = PersonalityDataScript.FACET_KEYS[aid]

		for j in range(fkeys.size()):
			var fkey: String = fkeys[j]
			var current_z: float = pd.to_zscore(pd.facets.get(fkey, 0.5))
			# OU drift toward target + random noise.
			var dz: float = _theta * (target - current_z) + _randfn(0.0, _sigma)
			pd.facets[fkey] = pd.from_zscore(current_z + dz)

	pd.recalculate_axes()
	pd.active_traits = TraitSystem.check_traits(pd)


## Get maturation target z-shift for axis at given age.
## Data-driven from SpeciesManager.personality_distribution.maturation.
func _get_maturation_target(axis_id: String, age: int) -> float:
	var cfg = _maturation_targets.get(axis_id, {})
	if cfg.is_empty():
		return 0.0
	var target_shift = float(cfg.get("target_shift", 0.0))
	if target_shift == 0.0:
		return 0.0
	var age_range = cfg.get("age_range", [18, 60])
	var start_age = int(age_range[0]) if age_range.size() > 0 else 18
	var end_age = int(age_range[1]) if age_range.size() > 1 else 60
	return _linear_target(age, target_shift, start_age, end_age)


## Linear maturation target: 0 at start_age, max_shift at end_age, clamped.
func _linear_target(age: int, max_shift: float, start_age: int = 18, end_age: int = 60) -> float:
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_LINEAR_TARGET_METHOD,
			age,
			max_shift,
			start_age,
			end_age,
		)
		if rust_variant != null:
			return float(rust_variant)
	if age < start_age:
		return 0.0
	var span: float = float(end_age - start_age)
	if span <= 0.0:
		return max_shift
	var t: float = clampf(float(age - start_age) / span, 0.0, 1.0)
	return max_shift * t
