extends "res://scripts/core/simulation/simulation_system.gd"

## [Kahneman & Tversky 1979, Modigliani 1966, Engel 2011, Piff 2010]
## Computes 4 economic tendencies per entity: saving, risk, generosity, materialism.

var _entity_manager: RefCounted
var _settlement_manager: RefCounted
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_ECON_METHOD: String = "body_economic_tendencies_step"
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func _init() -> void:
	system_name = "economic_tendency"
	priority = 39
	tick_interval = GameConfig.ECON_TICK_INTERVAL


func init(entity_manager: RefCounted, settlement_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_settlement_manager = settlement_manager


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
	if node != null and node.has_method(_SIM_BRIDGE_ECON_METHOD):
		_sim_bridge = node
	return _sim_bridge

func _compute_tendencies(entity: RefCounted) -> void:
	if entity.personality == null:
		return
	var H = float(entity.personality.axes.get("H", 0.5))
	var E_val = float(entity.personality.axes.get("E", 0.5))
	var X = float(entity.personality.axes.get("X", 0.5))
	var A = float(entity.personality.axes.get("A", 0.5))
	var C = float(entity.personality.axes.get("C", 0.5))
	var O = float(entity.personality.axes.get("O", 0.5))

	var age_years: float = GameConfig.get_age_years(entity.age)
	# [Modigliani 1966] age_factor: sigmoid (0 at youth, 1 at mature)
	var age_factor: float = 1.0 / (1.0 + exp(-(age_years - 22.0) / 10.0))

	# Values are already bipolar [-1, +1]
	var values: Dictionary = entity.values

	# --- SAVING [Frederick 2002, Ashton & Lee 2007] ---
	entity.economic_tendencies["saving"] = clampf(
		_bipolar(C) * 0.40
		+ float(values.get("SELF_CONTROL", 0.0)) * 0.20
		+ _bipolar(E_val) * 0.15
		+ _bipolar(age_factor) * 0.10
		+ float(values.get("LAW", 0.0)) * 0.10
		+ (-float(values.get("COMMERCE", 0.0))) * 0.05,
		-1.0, 1.0)

	# --- RISK [Kahneman & Tversky 1979, Dohmen 2011] ---
	var risk_val: float = clampf(
		-_bipolar(E_val) * 0.25
		+ _bipolar(X) * 0.20
		+ -_bipolar(C) * 0.20
		+ _bipolar(O) * 0.15
		+ float(values.get("COMPETITION", 0.0)) * 0.10
		+ float(values.get("MARTIAL_PROWESS", 0.0)) * 0.05
		+ -_bipolar(age_factor) * 0.05,
		-1.0, 1.0)
	# Sex difference: male +0.06 [Byrnes 1999]
	if entity.gender == "male":
		risk_val = clampf(risk_val + 0.06, -1.0, 1.0)
	entity.economic_tendencies["risk"] = risk_val

	# --- GENEROSITY [Engel 2011, Piff 2010] ---
	var culture_gen: float = 0.0
	var settlement = _get_settlement(entity)
	if settlement != null and settlement.shared_values.has("SACRIFICE"):
		culture_gen = float(settlement.shared_values["SACRIFICE"])

	var gen_val: float = clampf(
		_bipolar(H) * 0.25
		+ _bipolar(A) * 0.20
		+ float(values.get("SACRIFICE", 0.0)) * 0.20
		+ float(values.get("COOPERATION", 0.0)) * 0.15
		+ _bipolar(entity.belonging) * 0.10
		+ float(values.get("FAMILY", 0.0)) * 0.05
		+ culture_gen * 0.05,
		-1.0, 1.0)
	# [Piff 2010] Wealth→generosity feedback
	if entity.wealth_norm > 0.80:
		gen_val *= GameConfig.ECON_WEALTH_GENEROSITY_PENALTY
	entity.economic_tendencies["generosity"] = gen_val

	# --- MATERIALISM [Kasser & Ryan 1993, Dittmar 2014] ---
	var culture_mat: float = 0.0
	if settlement != null and settlement.shared_values.has("COMMERCE"):
		culture_mat = float(settlement.shared_values["COMMERCE"])

	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var scalar_inputs: PackedFloat32Array = PackedFloat32Array([
			H,
			E_val,
			X,
			A,
			C,
			O,
			age_years,
			float(values.get("SELF_CONTROL", 0.0)),
			float(values.get("LAW", 0.0)),
			float(values.get("COMMERCE", 0.0)),
			float(values.get("COMPETITION", 0.0)),
			float(values.get("MARTIAL_PROWESS", 0.0)),
			float(values.get("SACRIFICE", 0.0)),
			float(values.get("COOPERATION", 0.0)),
			float(values.get("FAMILY", 0.0)),
			float(values.get("POWER", 0.0)),
			float(values.get("FAIRNESS", 0.0)),
			float(entity.belonging),
			float(entity.wealth_norm),
			culture_gen,
			culture_mat,
		])
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_ECON_METHOD,
			scalar_inputs,
			entity.gender == "male",
			float(GameConfig.ECON_WEALTH_GENEROSITY_PENALTY)
		)
		if rust_variant is PackedFloat32Array:
			var out: PackedFloat32Array = rust_variant
			if out.size() >= 4:
				entity.economic_tendencies["saving"] = float(out[0])
				entity.economic_tendencies["risk"] = float(out[1])
				entity.economic_tendencies["generosity"] = float(out[2])
				entity.economic_tendencies["materialism"] = float(out[3])
				return

	entity.economic_tendencies["materialism"] = clampf(
		-_bipolar(H) * 0.30
		+ float(values.get("COMMERCE", 0.0)) * 0.20
		+ float(values.get("POWER", 0.0)) * 0.15
		+ -float(values.get("FAIRNESS", 0.0)) * 0.10
		+ _bipolar(entity.wealth_norm) * 0.10
		+ float(values.get("COMPETITION", 0.0)) * 0.10
		+ culture_mat * 0.05,
		-1.0, 1.0)


## Convert unipolar [0.0, 1.0] to bipolar [-1.0, +1.0]
## Osgood et al. (1957) Semantic Differential — bipolar scales are standard for attitudes/tendencies
func _bipolar(val: float) -> float:
	return (val - 0.5) * 2.0


## Get settlement for entity
func _get_settlement(entity: RefCounted) -> RefCounted:
	if _settlement_manager == null or entity.settlement_id <= 0:
		return null
	return _settlement_manager.get_settlement(entity.settlement_id)
