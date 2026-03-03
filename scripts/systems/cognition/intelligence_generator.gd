extends RefCounted

## [Visser 2006, CHC, Plomin & Deary 2015]
## Hybrid intelligence generation: g-factor + Cholesky-correlated residuals.
## Reference: const IntelligenceGenerator = preload("res://scripts/systems/cognition/intelligence_generator.gd")

const INTEL_KEYS: Array = [
	"linguistic", "logical", "spatial", "musical",
	"kinesthetic", "naturalistic", "interpersonal", "intrapersonal",
]
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_INTEL_G_METHOD: String = "body_intelligence_g_value"

## Precomputed lower-triangular Cholesky factor of INTEL_RESIDUAL_CORR.
var _cholesky_L: Array = []
var _rng: RandomNumberGenerator = RandomNumberGenerator.new()
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func init(rng: RandomNumberGenerator = null) -> void:
	if rng != null:
		_rng = rng
	else:
		_rng.randomize()
	_cholesky_L = _cholesky_decompose(GameConfig.INTEL_RESIDUAL_CORR)


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
	if node != null and node.has_method(_SIM_BRIDGE_INTEL_G_METHOD):
		_sim_bridge = node
	return _sim_bridge


## Main entry point: generate all intelligence data for a new entity.
## parent_a, parent_b: parent entity_data (null for initial spawns).
## gender: "male" / "female"
## hexaco_facets: Dictionary of facet values
## Returns: { "g": float, "potentials": Dictionary, "effective": Dictionary }
func generate(gender: String, hexaco_facets: Dictionary,
		parent_a = null, parent_b = null) -> Dictionary:
	var g: float = _generate_g(parent_a, parent_b, hexaco_facets)

	var residuals: Dictionary = _generate_correlated_residuals()

	var potentials: Dictionary = {}
	for i in range(INTEL_KEYS.size()):
		var key: String = INTEL_KEYS[i]
		var loading: float = GameConfig.INTEL_G_LOADING.get(key, 0.5)
		var raw: float = g * loading + residuals[key] * (1.0 - loading)
		potentials[key] = raw

	potentials = _apply_parental_inheritance(potentials, parent_a, parent_b)

	var sex_shifts: Dictionary = GameConfig.INTEL_SEX_DIFF_MALE if gender == "male" else GameConfig.INTEL_SEX_DIFF_FEMALE
	for key in sex_shifts:
		potentials[key] = potentials.get(key, 0.5) + sex_shifts[key]

	for key in INTEL_KEYS:
		potentials[key] = clampf(potentials.get(key, 0.5), 0.05, 0.95)

	var effective: Dictionary = potentials.duplicate()
	return {"g": clampf(g, 0.05, 0.95), "potentials": potentials, "effective": effective}


func _generate_g(parent_a, parent_b, hexaco_facets: Dictionary) -> float:
	var has_parents: bool = parent_a != null and parent_b != null
	var pa_g: float = parent_a.general_intelligence if parent_a != null else 0.5
	var pb_g: float = parent_b.general_intelligence if parent_b != null else 0.5
	var h2: float = GameConfig.INTEL_HERITABILITY_G
	var O_mean: float = _get_openness_mean(hexaco_facets)
	var noise: float = _randfn(0.0, GameConfig.INTEL_G_SD * 0.6)
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_INTEL_G_METHOD,
			has_parents,
			pa_g,
			pb_g,
			h2,
			float(GameConfig.INTEL_G_MEAN),
			O_mean,
			float(GameConfig.INTEL_OPENNESS_G_WEIGHT),
			noise,
		)
		if rust_variant != null:
			return float(rust_variant)

	var base: float = GameConfig.INTEL_G_MEAN
	if has_parents:
		base = ((pa_g + pb_g) / 2.0) * h2 + GameConfig.INTEL_G_MEAN * (1.0 - h2)
	var openness_shift: float = GameConfig.INTEL_OPENNESS_G_WEIGHT * (O_mean - 0.5)
	return base + openness_shift + noise


func _generate_correlated_residuals() -> Dictionary:
	var z: Array = []
	for i in range(8):
		z.append(_randfn(0.0, 1.0))

	var correlated: Array = []
	for i in range(8):
		var val: float = 0.0
		for j in range(i + 1):
			val += _cholesky_L[i][j] * z[j]
		correlated.append(val)

	var result: Dictionary = {}
	for i in range(INTEL_KEYS.size()):
		var key: String = INTEL_KEYS[i]
		result[key] = clampf(0.5 + correlated[i] * GameConfig.INTEL_RESIDUAL_SD, 0.05, 0.95)
	return result


func _apply_parental_inheritance(potentials: Dictionary, parent_a, parent_b) -> Dictionary:
	if parent_a == null or parent_b == null:
		return potentials
	if parent_a.intelligence_potentials.is_empty() or parent_b.intelligence_potentials.is_empty():
		return potentials

	for key in INTEL_KEYS:
		var h2: float
		if key in GameConfig.INTEL_GROUP_FLUID:
			h2 = GameConfig.INTEL_HERITABILITY_FLUID
		elif key in GameConfig.INTEL_GROUP_PHYSICAL:
			h2 = GameConfig.INTEL_HERITABILITY_PHYSICAL
		else:
			h2 = GameConfig.INTEL_HERITABILITY_CRYSTALLIZED

		var pa_val: float = parent_a.intelligence_potentials.get(key, 0.5)
		var pb_val: float = parent_b.intelligence_potentials.get(key, 0.5)
		var parental_mid: float = (pa_val + pb_val) / 2.0
		potentials[key] = potentials[key] * (1.0 - h2) + parental_mid * h2
		potentials[key] += _randfn(0.0, 0.04)
	return potentials


static func _cholesky_decompose(matrix: Array) -> Array:
	var n: int = matrix.size()
	var L: Array = []
	for i in range(n):
		var row: Array = []
		row.resize(n)
		row.fill(0.0)
		L.append(row)

	for i in range(n):
		for j in range(i + 1):
			var s: float = 0.0
			for k in range(j):
				s += L[i][k] * L[j][k]
			if i == j:
				var diag: float = matrix[i][i] - s
				L[i][j] = sqrt(maxf(diag, 0.0001))
			else:
				L[i][j] = (matrix[i][j] - s) / maxf(L[j][j], 0.0001)
	return L


func _get_openness_mean(facets: Dictionary) -> float:
	var sum_val: float = 0.0
	var count: int = 0
	for key in ["O_aesthetic", "O_inquisitiveness", "O_creativity", "O_unconventionality"]:
		if facets.has(key):
			sum_val += float(facets[key])
			count += 1
	if count > 0:
		return sum_val / float(count)
	return 0.5


func _randfn(mean: float, sd: float) -> float:
	var u1: float = maxf(_rng.randf(), 0.0001)
	var u2: float = _rng.randf()
	var z: float = sqrt(-2.0 * log(u1)) * cos(2.0 * PI * u2)
	return mean + sd * z
