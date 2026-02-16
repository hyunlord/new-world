extends RefCounted

## Personality utility functions: compatibility and affinity scaling.
## No class_name - use preload("res://scripts/core/personality_system.gd").

## Compatibility weights: trust/fairness (H) and conflict handling (A) matter most.
## H: 3.0, A: 2.0, C: 1.5, E: 1.0, X: 1.0, O: 0.8
const COMPAT_WEIGHTS: Dictionary = {
	"H": 3.0,
	"A": 2.0,
	"C": 1.5,
	"E": 1.0,
	"X": 1.0,
	"O": 0.8,
}


## Calculate personality compatibility between two PersonalityData instances.
## Returns float in range [-1.0, +1.0].
## +1.0 = maximally compatible (identical on weighted axes)
## -1.0 = maximally incompatible (opposite extremes on weighted axes)
##  0.0 = average compatibility
static func personality_compatibility(pd_a: RefCounted, pd_b: RefCounted) -> float:
	var weighted_dist: float = 0.0
	var total_weight: float = 0.0
	var keys: Array = COMPAT_WEIGHTS.keys()
	for i in range(keys.size()):
		var axis_id: String = keys[i]
		var w = COMPAT_WEIGHTS.get(axis_id, 0.0)
		var val_a = pd_a.axes.get(axis_id, 0.5)
		var val_b = pd_b.axes.get(axis_id, 0.5)
		var diff: float = absf(float(val_a) - float(val_b))
		weighted_dist += float(w) * diff
		total_weight += float(w)
	var similarity: float = 1.0 - weighted_dist / maxf(total_weight, 0.00001)
	return 2.0 * similarity - 1.0  # map [0,1] -> [-1,+1]


## Apply compatibility to affinity change speed.
## compat +1.0 -> change * 1.5 (fast bonding)
## compat -1.0 -> change * 0.5 (slow bonding)
## compat  0.0 -> change * 1.0 (baseline)
static func apply_compatibility_modifier(base_change: float, compatibility: float) -> float:
	return base_change * (1.0 + 0.5 * compatibility)
