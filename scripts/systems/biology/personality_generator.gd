extends RefCounted

## Cholesky-based HEXACO personality generator with parental inheritance,
## sex differences, and culture shifts.
## No class_name - use preload("res://scripts/systems/biology/personality_generator.gd").

const PersonalityDataScript = preload("res://scripts/core/entity/personality_data.gd")
const TraitSystem = preload("res://scripts/systems/psychology/trait_system.gd")
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_CHILD_AXIS_METHOD: String = "body_personality_child_axis_z"

var _correlation_matrix: Array = []
var _heritability: Dictionary = {}
var _sex_diff_d: Dictionary = {}
var _facet_spread: float = 0.75  # Intra-axis facet variance (z-score units)
var _cholesky_L: Array = []  # 6x6 lower triangular matrix (cached)
var _rng: RandomNumberGenerator
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func init(rng: RandomNumberGenerator) -> void:
	_rng = rng
	var dist = SpeciesManager.personality_distribution
	var cm = dist.get("correlation_matrix", {})
	_correlation_matrix = cm.get("matrix", [
		[1.00, 0.12, -0.11, 0.26, 0.18, 0.21],
		[0.12, 1.00, -0.13, -0.08, 0.15, -0.10],
		[-0.11, -0.13, 1.00, 0.05, 0.10, 0.08],
		[0.26, -0.08, 0.05, 1.00, 0.01, 0.03],
		[0.18, 0.15, 0.10, 0.01, 1.00, 0.03],
		[0.21, -0.10, 0.08, 0.03, 0.03, 1.00],
	])
	_heritability = dist.get("heritability", {
		"H": 0.45, "E": 0.58, "X": 0.57,
		"A": 0.47, "C": 0.52, "O": 0.63,
	})
	_sex_diff_d = dist.get("sex_difference_d", {
		"H": 0.41, "E": 0.96, "X": 0.10,
		"A": 0.28, "C": 0.00, "O": -0.04,
	})
	_facet_spread = float(dist.get("facet_spread", 0.75))
	_cholesky_L = _cholesky_decompose(_correlation_matrix)


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
	if node != null and node.has_method(_SIM_BRIDGE_CHILD_AXIS_METHOD):
		_sim_bridge = node
	return _sim_bridge


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
	var has_parents: bool = parent_a != null and parent_b != null
	var is_female: bool = sex == "female"
	var bridge: Object = _get_sim_bridge()

	# Step 1: Sample correlated axis z-scores
	var z_random: Array = _sample_correlated_axes()

	# Step 2: Per-axis: inheritance + environment + sex + culture
	var z_axes: Dictionary = {}
	for i in range(6):
		var aid: String = axis_ids[i]
		var h2: float = _heritability.get(aid, 0.5)
		var z_pa: float = 0.0
		var z_pb: float = 0.0
		if has_parents:
			z_pa = parent_a.to_zscore(parent_a.axes.get(aid, 0.5))
			z_pb = parent_b.to_zscore(parent_b.axes.get(aid, 0.5))
		var d: float = _sex_diff_d.get(aid, 0.0)
		var culture_shift: float = _get_culture_shift(culture_id, aid)
		var z_child: float = 0.0
		var used_rust: bool = false
		if bridge != null:
			var rust_variant: Variant = bridge.call(
				_SIM_BRIDGE_CHILD_AXIS_METHOD,
				has_parents,
				z_pa,
				z_pb,
				h2,
				float(z_random[i]),
				is_female,
				d,
				culture_shift,
			)
			if rust_variant != null:
				z_child = float(rust_variant)
				used_rust = true
		if not used_rust:
			var z_mid: float = 0.0
			if has_parents:
				z_mid = 0.5 * (z_pa + z_pb)
			var env_factor: float = sqrt(1.0 - 0.5 * h2 * h2)
			z_child = h2 * z_mid + env_factor * z_random[i]
			if is_female:
				z_child += d / 2.0
			else:
				z_child -= d / 2.0
			z_child += culture_shift

		z_axes[aid] = z_child

	# Step 3: Distribute axis z-score to 4 facets with intra-axis variation
	for i in range(axis_ids.size()):
		var aid: String = axis_ids[i]
		var z_axis: float = z_axes[aid]
		var fkeys: Array = PersonalityDataScript.FACET_KEYS[aid]
		for j in range(fkeys.size()):
			# Intra-axis facet variance from SpeciesManager (0.75 enables contradictory facet combos)
			var facet_z: float = z_axis + _randfn(0.0, _facet_spread)
			pd.facets[fkeys[j]] = pd.from_zscore(facet_z)

	# Step 4: Recalculate axes from facet averages
	pd.recalculate_axes()

	# Step 5: Check trait emergence
	pd.active_traits = TraitSystem.check_traits(pd)

	return pd


## Culture z-shift via SpeciesManager
func _get_culture_shift(culture_id: String, axis_id: String) -> float:
	return SpeciesManager.get_culture_shift(culture_id, axis_id)
