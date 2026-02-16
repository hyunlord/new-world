extends RefCounted

## Cholesky-based HEXACO personality generator with parental inheritance,
## sex differences, and culture shifts.
## No class_name - use preload("res://scripts/systems/personality_generator.gd").

const PersonalityDataScript = preload("res://scripts/core/personality_data.gd")
const TraitSystem = preload("res://scripts/systems/trait_system.gd")

## Axis inter-correlation matrix (HEXACO-60, student sample, Ashton & Lee 2009 Table 3)
const CORRELATION_MATRIX: Array = [
	[1.00, 0.12, -0.11, 0.26, 0.18, 0.21],   # H
	[0.12, 1.00, -0.13, -0.08, 0.15, -0.10], # E
	[-0.11, -0.13, 1.00, 0.05, 0.10, 0.08],  # X
	[0.26, -0.08, 0.05, 1.00, 0.01, 0.03],   # A
	[0.18, 0.15, 0.10, 0.01, 1.00, 0.03],    # C
	[0.21, -0.10, 0.08, 0.03, 0.03, 1.00],   # O
]

## Heritability per axis (Vernon et al. 2008, extended twin-family model)
const HERITABILITY: Dictionary = {
	"H": 0.45, "E": 0.58, "X": 0.57,
	"A": 0.47, "C": 0.52, "O": 0.63,
}

## Sex differences Cohen's d (Ashton & Lee 2009, HEXACO-60 community sample)
## Positive = females score higher
const SEX_DIFF_D: Dictionary = {
	"H": 0.41, "E": 0.96, "X": 0.10,
	"A": 0.28, "C": 0.00, "O": -0.04,
}

var _cholesky_L: Array = []  # 6x6 lower triangular matrix (cached)
var _rng: RandomNumberGenerator


func init(rng: RandomNumberGenerator) -> void:
	_rng = rng
	_cholesky_L = _cholesky_decompose(CORRELATION_MATRIX)


## Box-Muller transform for normal distribution (Godot 4 has no randfn)
func _randfn(mean: float, std: float) -> float:
	var u1: float = _rng.randf()
	var u2: float = _rng.randf()
	# Avoid log(0)
	if u1 < 1e-10:
		u1 = 1e-10
	return mean + std * sqrt(-2.0 * log(u1)) * cos(2.0 * PI * u2)


## Cholesky decomposition: R = L * L^T
func _cholesky_decompose(R: Array) -> Array:
	var n: int = R.size()
	var L: Array = []
	for i in range(n):
		var row: Array = []
		for j in range(n):
			row.append(0.0)
		L.append(row)
	for i in range(n):
		for j in range(i + 1):
			var sum_val: float = 0.0
			for k in range(j):
				sum_val += L[i][k] * L[j][k]
			if i == j:
				L[i][j] = sqrt(R[i][i] - sum_val)
			else:
				L[i][j] = (R[i][j] - sum_val) / L[j][j]
	return L


## Sample 6 correlated axis z-scores using Cholesky
func _sample_correlated_axes() -> Array:
	var z_indep: Array = []
	for i in range(6):
		z_indep.append(_randfn(0.0, 1.0))
	var z_corr: Array = []
	for i in range(6):
		var val: float = 0.0
		for j in range(i + 1):
			val += _cholesky_L[i][j] * z_indep[j]
		z_corr.append(val)
	return z_corr


## Generate a new PersonalityData.
## sex: "male" or "female"
## culture_id: settlement culture (for future culture shifts, currently returns 0)
## parent_a, parent_b: PersonalityData of parents (null for 1st generation)
func generate_personality(sex: String, culture_id: String,
		parent_a: RefCounted, parent_b: RefCounted) -> RefCounted:
	var pd = PersonalityDataScript.new()
	var axis_ids: Array = PersonalityDataScript.AXIS_IDS

	# Step 1: Sample correlated axis z-scores
	var z_random: Array = _sample_correlated_axes()

	# Step 2: Per-axis: inheritance + environment + sex + culture
	var z_axes: Dictionary = {}
	for i in range(6):
		var aid: String = axis_ids[i]
		var h2: float = HERITABILITY[aid]

		# Mid-parent z-score
		var z_mid: float = 0.0
		if parent_a != null and parent_b != null:
			var z_pa: float = parent_a.to_zscore(parent_a.axes.get(aid, 0.5))
			var z_pb: float = parent_b.to_zscore(parent_b.axes.get(aid, 0.5))
			z_mid = 0.5 * (z_pa + z_pb)

		# Inheritance + environment noise
		var env_factor: float = sqrt(1.0 - 0.5 * h2 * h2)
		var z_child: float = h2 * z_mid + env_factor * z_random[i]

		# Sex difference shift
		var d: float = SEX_DIFF_D[aid]
		if sex == "female":
			z_child += d / 2.0
		else:
			z_child -= d / 2.0

		# Culture shift (stub - returns 0 for now)
		z_child += _get_culture_shift(culture_id, aid)

		z_axes[aid] = z_child

	# Step 3: Distribute axis z-score to 4 facets with intra-axis variation
	for i in range(axis_ids.size()):
		var aid: String = axis_ids[i]
		var z_axis: float = z_axes[aid]
		var fkeys: Array = PersonalityDataScript.FACET_KEYS[aid]
		for j in range(fkeys.size()):
			var facet_z: float = z_axis + _randfn(0.0, 0.25)
			pd.facets[fkeys[j]] = pd.from_zscore(facet_z)

	# Step 4: Recalculate axes from facet averages
	pd.recalculate_axes()

	# Step 5: Check trait emergence
	pd.active_traits = TraitSystem.check_traits(pd)

	return pd


## Culture z-shift stub (returns 0 for all; load from JSON in future)
func _get_culture_shift(_culture_id: String, _axis_id: String) -> float:
	return 0.0
