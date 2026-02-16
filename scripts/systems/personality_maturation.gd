extends RefCounted

## Age-based personality maturation using OU (Ornstein-Uhlenbeck) process.
## Called once per game year for each entity.
## Ashton & Lee (2016): H increases ~+1 SD from 18→60, E/X mild increase.
## No class_name - use preload("res://scripts/systems/personality_maturation.gd").

const PersonalityDataScript = preload("res://scripts/core/personality_data.gd")
const TraitSystem = preload("res://scripts/systems/trait_system.gd")

## OU process parameters (annual):
## THETA from annual convergence assumption (~3%), SIGMA as annual random drift SD.
## Target shifts based on Ashton & Lee (2016) age trends.
const THETA: float = 0.03
const SIGMA: float = 0.03

var _rng: RandomNumberGenerator


func init(rng: RandomNumberGenerator) -> void:
	_rng = rng


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
			var dz: float = THETA * (target - current_z) + _randfn(0.0, SIGMA)
			pd.facets[fkey] = pd.from_zscore(current_z + dz)

	pd.recalculate_axes()
	pd.active_traits = TraitSystem.check_traits(pd)


## Get maturation target z-shift for axis at given age.
## H: +1.0 SD from 18→60 (linear), E/X: +0.3 SD, A/C/O: stable.
## Ashton & Lee (2016).
func _get_maturation_target(axis_id: String, age: int) -> float:
	match axis_id:
		"H":
			return _linear_target(age, 1.0)
		"E":
			return _linear_target(age, 0.3)
		"X":
			return _linear_target(age, 0.3)
	return 0.0


## Linear maturation target: 0 at age 18, max_shift at age 60, clamped.
func _linear_target(age: int, max_shift: float) -> float:
	if age < 18:
		return 0.0
	var t: float = clampf(float(age - 18) / 42.0, 0.0, 1.0)
	return max_shift * t
