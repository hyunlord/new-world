extends "res://scripts/core/simulation/simulation_system.gd"

const BodyAttributes = preload("res://scripts/core/entity/body_attributes.gd")
const _BASE_DECAY_SCALAR_COUNT: int = 14
const _BASE_DECAY_FLAG_COUNT: int = 2
const _CRITICAL_SEVERITY_SCALAR_COUNT: int = 6
const _ERG_FRUSTRATION_SCALAR_COUNT: int = 10
const _ERG_FRUSTRATION_FLAG_COUNT: int = 2

var _entity_manager: RefCounted
var _building_manager: RefCounted
var _world_data: RefCounted = null
var _stress_system = null
var _last_tick: int = 0
var _erg_frustration_scalar_inputs: PackedFloat32Array = PackedFloat32Array()
var _erg_frustration_flag_inputs: PackedByteArray = PackedByteArray()


func _init() -> void:
	system_name = "needs"
	priority = 10
	tick_interval = GameConfig.NEEDS_TICK_INTERVAL


## Initialize with entity/building manager references
func init(entity_manager: RefCounted, building_manager: RefCounted, world_data: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_building_manager = building_manager
	_world_data = world_data


## Get total food in stockpiles belonging to a settlement
func _get_settlement_food(settlement_id: int) -> float:
	if _building_manager == null:
		return 0.0
	var total_food: float = 0.0
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	for i in range(stockpiles.size()):
		var stockpile: RefCounted = stockpiles[i]
		if stockpile.settlement_id != settlement_id or not stockpile.is_built:
			continue
		total_food += float(stockpile.storage.get("food", 0.0))
	return total_food


## Withdraw food from stockpiles belonging to a settlement
func _withdraw_food(settlement_id: int, amount: float) -> float:
	if _building_manager == null or amount <= 0.0:
		return 0.0
	var remaining: float = amount
	var withdrawn: float = 0.0
	var stockpiles: Array = _building_manager.get_buildings_by_type("stockpile")
	for i in range(stockpiles.size()):
		if remaining <= 0.0:
			break
		var stockpile: RefCounted = stockpiles[i]
		if stockpile.settlement_id != settlement_id or not stockpile.is_built:
			continue
		var available: float = float(stockpile.storage.get("food", 0.0))
		if available <= 0.0:
			continue
		var take: float = minf(available, remaining)
		stockpile.storage["food"] = available - take
		remaining -= take
		withdrawn += take
	return withdrawn


## Setter for external injection of StressSystem reference
func set_stress_system(ss) -> void:
	_stress_system = ss


## [Alderfer 1969 ERG Theory] Track sustained need frustration.
## If growth needs are chronically unmet → regression to existence-level obsession.
## If relatedness needs are chronically unmet → regression to existence-level obsession.
func _update_erg_frustration(entity: RefCounted) -> void:
	if not entity.is_alive:
		return
	var started_growth_regression: bool = false
	var started_relatedness_regression: bool = false
	var rust_applied: bool = false

	_erg_frustration_scalar_inputs[0] = entity.competence
	_erg_frustration_scalar_inputs[1] = entity.autonomy
	_erg_frustration_scalar_inputs[2] = entity.self_actualization
	_erg_frustration_scalar_inputs[3] = entity.belonging
	_erg_frustration_scalar_inputs[4] = entity.intimacy
	_erg_frustration_scalar_inputs[5] = GameConfig.ERG_GROWTH_FRUSTRATION_THRESHOLD
	_erg_frustration_scalar_inputs[6] = GameConfig.ERG_RELATEDNESS_FRUSTRATION_THRESHOLD
	_erg_frustration_scalar_inputs[7] = float(GameConfig.ERG_FRUSTRATION_WINDOW)
	_erg_frustration_scalar_inputs[8] = float(entity.erg_growth_frustration_ticks)
	_erg_frustration_scalar_inputs[9] = float(entity.erg_relatedness_frustration_ticks)
	_erg_frustration_flag_inputs[0] = 1 if entity.erg_regressing_to_existence else 0
	_erg_frustration_flag_inputs[1] = 1 if entity.erg_regressing_to_relatedness else 0

	var erg_step_variant: Variant = SimBridge.body_erg_frustration_step_packed(
		_erg_frustration_scalar_inputs,
		_erg_frustration_flag_inputs
	)
	if erg_step_variant is PackedInt32Array:
		var packed_erg_step: PackedInt32Array = erg_step_variant
		if packed_erg_step.size() >= 6:
			entity.erg_growth_frustration_ticks = int(packed_erg_step[0])
			entity.erg_relatedness_frustration_ticks = int(packed_erg_step[1])
			entity.erg_regressing_to_existence = int(packed_erg_step[2]) != 0
			started_growth_regression = int(packed_erg_step[3]) != 0
			entity.erg_regressing_to_relatedness = int(packed_erg_step[4]) != 0
			started_relatedness_regression = int(packed_erg_step[5]) != 0
			rust_applied = true

	if not rust_applied:
		# --- Growth frustration check ---
		var growth_frustrated: bool = (
			entity.competence < GameConfig.ERG_GROWTH_FRUSTRATION_THRESHOLD and
			entity.autonomy < GameConfig.ERG_GROWTH_FRUSTRATION_THRESHOLD and
			entity.self_actualization < GameConfig.ERG_GROWTH_FRUSTRATION_THRESHOLD
		)
		if growth_frustrated:
			entity.erg_growth_frustration_ticks += 1
		else:
			entity.erg_growth_frustration_ticks = maxi(0, entity.erg_growth_frustration_ticks - 10)

		var was_regressing_growth: bool = entity.erg_regressing_to_existence
		entity.erg_regressing_to_existence = (
			entity.erg_growth_frustration_ticks >= GameConfig.ERG_FRUSTRATION_WINDOW
		)
		started_growth_regression = entity.erg_regressing_to_existence and not was_regressing_growth

		# --- Relatedness frustration check ---
		var relatedness_frustrated: bool = (
			entity.belonging < GameConfig.ERG_RELATEDNESS_FRUSTRATION_THRESHOLD and
			entity.intimacy < GameConfig.ERG_RELATEDNESS_FRUSTRATION_THRESHOLD
		)
		if relatedness_frustrated:
			entity.erg_relatedness_frustration_ticks += 1
		else:
			entity.erg_relatedness_frustration_ticks = maxi(0, entity.erg_relatedness_frustration_ticks - 10)

		var was_regressing_rel: bool = entity.erg_regressing_to_relatedness
		entity.erg_regressing_to_relatedness = (
			entity.erg_relatedness_frustration_ticks >= GameConfig.ERG_FRUSTRATION_WINDOW
		)
		started_relatedness_regression = entity.erg_regressing_to_relatedness and not was_regressing_rel

	if started_growth_regression:
		if entity.emotion_data != null and _stress_system != null:
			_stress_system.inject_stress_event(entity.emotion_data, "erg_growth_regression", 5.0)
		emit_event("erg_regression_started", {
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"regression_type": "growth_to_existence",
			"tick": _last_tick,
		})

	if started_relatedness_regression:
		if entity.emotion_data != null and _stress_system != null:
			_stress_system.inject_stress_event(entity.emotion_data, "erg_relatedness_regression", 4.0)
		emit_event("erg_regression_started", {
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"regression_type": "relatedness_to_existence",
			"tick": _last_tick,
		})
