extends RefCounted

## HEXACO 24-facet personality data container.
## Use preload("res://scripts/core/personality_data.gd") to reference.

## 24 facet values (0.0~1.0)
var facets: Dictionary = {}  # "H_sincerity": 0.5, "H_fairness": 0.6, ...

## 6 axis values (auto-calculated from facet averages)
var axes: Dictionary = {}    # "H": 0.55, "E": 0.42, ...

## Active discrete traits (emerged from extreme values)
var active_traits: Array = []  # ["honest", "fearful", ...]

## Ordered axis IDs
const AXIS_IDS: Array = ["H", "E", "X", "A", "C", "O"]

## Axis -> facet key mapping
const FACET_KEYS: Dictionary = {
	"H": ["H_sincerity", "H_fairness", "H_greed_avoidance", "H_modesty"],
	"E": ["E_fearfulness", "E_anxiety", "E_dependence", "E_sentimentality"],
	"X": ["X_social_self_esteem", "X_social_boldness", "X_sociability", "X_liveliness"],
	"A": ["A_forgiveness", "A_gentleness", "A_flexibility", "A_patience"],
	"C": ["C_organization", "C_diligence", "C_perfectionism", "C_prudence"],
	"O": ["O_aesthetic", "O_inquisitiveness", "O_creativity", "O_unconventionality"],
}

## All 24 facet keys in fixed order (for serialization)
const ALL_FACET_KEYS: Array = [
	"H_sincerity", "H_fairness", "H_greed_avoidance", "H_modesty",
	"E_fearfulness", "E_anxiety", "E_dependence", "E_sentimentality",
	"X_social_self_esteem", "X_social_boldness", "X_sociability", "X_liveliness",
	"A_forgiveness", "A_gentleness", "A_flexibility", "A_patience",
	"C_organization", "C_diligence", "C_perfectionism", "C_prudence",
	"O_aesthetic", "O_inquisitiveness", "O_creativity", "O_unconventionality",
]


## Get axis value (average of 4 facets)
func get_axis(axis_id: String) -> float:
	var sum: float = 0.0
	var count: int = 0
	var keys = FACET_KEYS.get(axis_id, [])
	for i in range(keys.size()):
		sum += facets.get(keys[i], 0.5)
		count += 1
	return sum / maxf(count, 1)


## Recalculate all 6 axis values from facets
func recalculate_axes() -> void:
	for i in range(AXIS_IDS.size()):
		axes[AXIS_IDS[i]] = get_axis(AXIS_IDS[i])


## Convert 0.0~1.0 trait value to z-score (mean=0.5, sd=0.15)
func to_zscore(trait01: float) -> float:
	return (trait01 - 0.5) / 0.15


## Convert z-score back to 0.0~1.0 (clamped to [0.05, 0.95])
func from_zscore(z: float) -> float:
	return clampf(0.5 + 0.15 * z, 0.05, 0.95)


## Get facet keys for a given axis
func get_facet_keys_for_axis(axis_id: String) -> Array:
	return FACET_KEYS.get(axis_id, [])


## Initialize with default values (all facets = 0.5)
func init_defaults() -> void:
	for i in range(ALL_FACET_KEYS.size()):
		facets[ALL_FACET_KEYS[i]] = 0.5
	recalculate_axes()


## Create from old Big Five personality Dictionary (migration)
## Maps: openness->O, extraversion->X, agreeableness->A+H(0.5), diligence->C,
## emotional_stability -> invert to E
func migrate_from_big_five(old_personality: Dictionary) -> void:
	var o_val: float = old_personality.get("openness", 0.5)
	var a_val: float = old_personality.get("agreeableness", 0.5)
	var x_val: float = old_personality.get("extraversion", 0.5)
	var c_val: float = old_personality.get("diligence", 0.5)
	var e_stab: float = old_personality.get("emotional_stability", 0.5)
	# Map emotional_stability (high=stable) to Emotionality (high=emotional)
	var e_val: float = 1.0 - e_stab
	# H has no Big Five equivalent -> default 0.5
	var h_val: float = 0.5

	for fk in FACET_KEYS["H"]:
		facets[fk] = h_val
	for fk in FACET_KEYS["E"]:
		facets[fk] = e_val
	for fk in FACET_KEYS["X"]:
		facets[fk] = x_val
	for fk in FACET_KEYS["A"]:
		facets[fk] = a_val
	for fk in FACET_KEYS["C"]:
		facets[fk] = c_val
	for fk in FACET_KEYS["O"]:
		facets[fk] = o_val
	recalculate_axes()


## Serialize to Dictionary for save/load
func to_dict() -> Dictionary:
	return {
		"facets": facets.duplicate(),
		"active_traits": active_traits.duplicate(),
	}


## Deserialize from Dictionary
static func from_dict(data: Dictionary) -> RefCounted:
	var script = load("res://scripts/core/personality_data.gd")
	var pd = script.new()
	var f_data = data.get("facets", {})
	if f_data.is_empty():
		# Migration: old format might have axes directly
		pd.init_defaults()
	else:
		for i in range(pd.ALL_FACET_KEYS.size()):
			var key = pd.ALL_FACET_KEYS[i]
			pd.facets[key] = float(f_data.get(key, 0.5))
	pd.recalculate_axes()
	pd.active_traits = []
	var traits_data = data.get("active_traits", [])
	for i in range(traits_data.size()):
		pd.active_traits.append(str(traits_data[i]))
	return pd
