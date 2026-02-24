extends "res://scripts/core/simulation/simulation_system.gd"

## [Kahneman & Tversky 1979, Modigliani 1966, Engel 2011, Piff 2010]
## Computes 4 economic tendencies per entity: saving, risk, generosity, materialism.

var _entity_manager: RefCounted
var _settlement_manager: RefCounted


func _init() -> void:
	system_name = "economic_tendency"
	priority = 39
	tick_interval = GameConfig.ECON_TICK_INTERVAL


func init(entity_manager: RefCounted, settlement_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_settlement_manager = settlement_manager


func execute_tick(_tick: int) -> void:
	if _entity_manager == null:
		return
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity = alive[i]
		# Skip children — keep default 0.5
		if entity.age_stage == "child" or entity.age_stage == "infant":
			continue
		_compute_tendencies(entity)


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

	# Value helper: convert -1..+1 to 0..1
	var values: Dictionary = entity.values

	# --- SAVING [Frederick 2002, Ashton & Lee 2007] ---
	entity.economic_tendencies["saving"] = clampf(
		C * 0.40
		+ _v01(values, "SELF_CONTROL") * 0.20
		+ E_val * 0.15
		+ age_factor * 0.10
		+ _v01(values, "LAW") * 0.10
		+ (1.0 - _v01(values, "COMMERCE")) * 0.05,
		0.0, 1.0)

	# --- RISK [Kahneman & Tversky 1979, Dohmen 2011] ---
	var risk_val: float = clampf(
		(1.0 - E_val) * 0.25
		+ X * 0.20
		+ (1.0 - C) * 0.20
		+ O * 0.15
		+ _v01(values, "COMPETITION") * 0.10
		+ _v01(values, "MARTIAL_PROWESS") * 0.05
		+ (1.0 - age_factor) * 0.05,
		0.0, 1.0)
	# Sex difference: male +0.03 [Byrnes 1999]
	if entity.gender == "male":
		risk_val = clampf(risk_val + 0.03, 0.0, 1.0)
	entity.economic_tendencies["risk"] = risk_val

	# --- GENEROSITY [Engel 2011, Piff 2010] ---
	var belonging_sat: float = entity.belonging
	var culture_gen: float = 0.5
	var settlement = _get_settlement(entity)
	if settlement != null and settlement.shared_values.has("SACRIFICE"):
		culture_gen = (float(settlement.shared_values["SACRIFICE"]) + 1.0) / 2.0

	var gen_val: float = clampf(
		H * 0.25
		+ A * 0.20
		+ _v01(values, "SACRIFICE") * 0.20
		+ _v01(values, "COOPERATION") * 0.15
		+ belonging_sat * 0.10
		+ _v01(values, "FAMILY") * 0.05
		+ culture_gen * 0.05,
		0.0, 1.0)
	# [Piff 2010] Wealth→generosity feedback
	if entity.wealth_norm > 0.80:
		gen_val *= GameConfig.ECON_WEALTH_GENEROSITY_PENALTY
	entity.economic_tendencies["generosity"] = gen_val

	# --- MATERIALISM [Kasser & Ryan 1993, Dittmar 2014] ---
	var culture_mat: float = 0.5
	if settlement != null and settlement.shared_values.has("COMMERCE"):
		culture_mat = (float(settlement.shared_values["COMMERCE"]) + 1.0) / 2.0

	entity.economic_tendencies["materialism"] = clampf(
		(1.0 - H) * 0.30
		+ _v01(values, "COMMERCE") * 0.20
		+ _v01(values, "POWER") * 0.15
		+ (1.0 - _v01(values, "FAIRNESS")) * 0.10
		+ entity.wealth_norm * 0.10
		+ _v01(values, "COMPETITION") * 0.10
		+ culture_mat * 0.05,
		0.0, 1.0)


## Convert value from [-1, +1] to [0, 1]
func _v01(values: Dictionary, key: String) -> float:
	return (float(values.get(key, 0.0)) + 1.0) / 2.0


## Get settlement for entity
func _get_settlement(entity: RefCounted) -> RefCounted:
	if _settlement_manager == null or entity.settlement_id <= 0:
		return null
	return _settlement_manager.get_settlement(entity.settlement_id)
