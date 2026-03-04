extends "res://scripts/core/simulation/simulation_system.gd"

## [Boehm 1999, Kohler 2017, Scheidel 2017]
## Settlement-level stratification monitoring: wealth, Gini, status, leveling.

var _entity_manager: RefCounted
var _settlement_manager: RefCounted
var _reputation_manager: RefCounted
var _current_tick: int = 0
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_WEALTH_METHOD: String = "body_stratification_wealth_score"
const _SIM_BRIDGE_GINI_METHOD: String = "body_stratification_gini"
const _SIM_BRIDGE_STATUS_METHOD: String = "body_stratification_status_score"
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func _init() -> void:
	system_name = "stratification_monitor"
	priority = 90
	tick_interval = GameConfig.STRAT_TICK_INTERVAL


func init(entity_manager: RefCounted, settlement_manager: RefCounted, reputation_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_settlement_manager = settlement_manager
	_reputation_manager = reputation_manager


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
	if node != null \
	and node.has_method(_SIM_BRIDGE_WEALTH_METHOD) \
	and node.has_method(_SIM_BRIDGE_GINI_METHOD) \
	and node.has_method(_SIM_BRIDGE_STATUS_METHOD):
		_sim_bridge = node
	return _sim_bridge

## ── Wealth Scores ─────────────────────────────────────────

func _compute_wealth_scores(settlement: RefCounted) -> void:
	var avg_wood: float = _get_settlement_avg(settlement, "wood")
	var avg_stone: float = _get_settlement_avg(settlement, "stone")
	var scores: Array = []
	var bridge: Object = _get_sim_bridge()

	for m_idx in range(settlement.member_ids.size()):
		var mid = settlement.member_ids[m_idx]
		var entity = _entity_manager.get_entity(mid)
		if entity == null or not entity.is_alive:
			continue

		var food_amount = entity.inventory.get("food", 0.0)
		var wood_amount = entity.inventory.get("wood", 0.0)
		var stone_amount = entity.inventory.get("stone", 0.0)
		var food_days: float = food_amount / maxf(
			GameConfig.HUNGER_DECAY_RATE * float(GameConfig.TICKS_PER_DAY), 0.001)
		var wood_norm: float = wood_amount / maxf(avg_wood, 1.0)
		var stone_norm: float = stone_amount / maxf(avg_stone, 1.0)

		var fallback_wealth_score: float = (
			GameConfig.WEALTH_W_FOOD * log(1.0 + food_days)
			+ GameConfig.WEALTH_W_WOOD * log(1.0 + 10.0 * wood_norm)
			+ GameConfig.WEALTH_W_STONE * log(1.0 + 10.0 * stone_norm)
		)
		entity.wealth_score = fallback_wealth_score
		if bridge != null:
			var rust_variant: Variant = bridge.call(
				_SIM_BRIDGE_WEALTH_METHOD,
				food_days,
				wood_norm,
				stone_norm,
				float(GameConfig.WEALTH_W_FOOD),
				float(GameConfig.WEALTH_W_WOOD),
				float(GameConfig.WEALTH_W_STONE),
			)
			if rust_variant != null:
				entity.wealth_score = float(rust_variant)
		scores.append(entity.wealth_score)

	scores.sort()
	var p90_idx: int = int(float(scores.size()) * 0.90)
	if scores.size() > 0:
		settlement.wealth_p90 = scores[mini(p90_idx, scores.size() - 1)]
	else:
		settlement.wealth_p90 = 1.0
	settlement.wealth_p90 = maxf(settlement.wealth_p90, 0.01)

	for m_idx in range(settlement.member_ids.size()):
		var mid = settlement.member_ids[m_idx]
		var entity = _entity_manager.get_entity(mid)
		if entity != null and entity.is_alive:
			entity.wealth_norm = clampf(entity.wealth_score / settlement.wealth_p90, 0.0, 1.0)


func _get_settlement_avg(settlement: RefCounted, resource: String) -> float:
	var total: float = 0.0
	var count: int = 0
	for m_idx in range(settlement.member_ids.size()):
		var mid = settlement.member_ids[m_idx]
		var entity = _entity_manager.get_entity(mid)
		if entity != null and entity.is_alive:
			total += entity.inventory.get(resource, 0.0)
			count += 1
	if count == 0:
		return 1.0
	return total / float(count)


## ── Gini Coefficient ──────────────────────────────────────

func _compute_gini(settlement: RefCounted) -> void:
	var values: Array = []
	for m_idx in range(settlement.member_ids.size()):
		var mid = settlement.member_ids[m_idx]
		var entity = _entity_manager.get_entity(mid)
		if entity != null and entity.is_alive:
			values.append(entity.wealth_score)
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var packed_values: PackedFloat32Array = PackedFloat32Array()
		packed_values.resize(values.size())
		for i in range(values.size()):
			packed_values[i] = float(values[i])
		var rust_variant: Variant = bridge.call(_SIM_BRIDGE_GINI_METHOD, packed_values)
		if rust_variant != null:
			settlement.gini_coefficient = clampf(float(rust_variant), 0.0, 1.0)
			return
	settlement.gini_coefficient = _calculate_gini(values)


func _calculate_gini(values: Array) -> float:
	var n: int = values.size()
	if n < 2:
		return 0.0
	var sorted_vals: Array = values.duplicate()
	sorted_vals.sort()
	var sum_diff: float = 0.0
	var total: float = 0.0
	for i in range(n):
		var value = sorted_vals[i]
		total += value
		sum_diff += (2.0 * float(i) - float(n) + 1.0) * value
	if total < 0.001:
		return 0.0
	return clampf(sum_diff / (float(n) * total), 0.0, 1.0)


## ── Status Scores ─────────────────────────────────────────

func _compute_status_scores(settlement: RefCounted) -> void:
	for m_idx in range(settlement.member_ids.size()):
		var mid = settlement.member_ids[m_idx]
		var entity = _entity_manager.get_entity(mid)
		if entity == null or not entity.is_alive:
			continue
		_compute_entity_status(entity, settlement)


func _compute_entity_status(entity: RefCounted, settlement: RefCounted) -> void:
	var avg_rep: Dictionary = {"overall": 0.0, "competence": 0.0}
	if _reputation_manager != null:
		avg_rep = _reputation_manager.get_settlement_average(entity.id, settlement.member_ids)

	var leader_bonus: float = 0.0
	if settlement.leader_id == entity.id:
		leader_bonus = GameConfig.STATUS_LEADER_CURRENT

	var age_years: float = GameConfig.get_age_years(entity.age)
	var age_respect: float = clampf((age_years - 30.0) / 40.0, 0.0, 1.0)
	var rep_overall: float = float(avg_rep.get("overall", 0.0))
	var rep_competence: float = float(avg_rep.get("competence", 0.0))
	var fallback_status_score: float = clampf(
		rep_overall * GameConfig.STATUS_W_REPUTATION
		+ entity.wealth_norm * GameConfig.STATUS_W_WEALTH
		+ leader_bonus * GameConfig.STATUS_W_LEADER
		+ age_respect * GameConfig.STATUS_W_AGE
		+ rep_competence * GameConfig.STATUS_W_COMPETENCE,
		-1.0, 1.0)
	entity.status_score = fallback_status_score

	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var scalar_inputs: PackedFloat32Array = PackedFloat32Array([
			rep_overall,
			entity.wealth_norm,
			leader_bonus,
			age_years,
			rep_competence,
			float(GameConfig.STATUS_W_REPUTATION),
			float(GameConfig.STATUS_W_WEALTH),
			float(GameConfig.STATUS_W_LEADER),
			float(GameConfig.STATUS_W_AGE),
			float(GameConfig.STATUS_W_COMPETENCE),
		])
		var rust_variant: Variant = bridge.call(_SIM_BRIDGE_STATUS_METHOD, scalar_inputs)
		if rust_variant != null:
			entity.status_score = clampf(float(rust_variant), -1.0, 1.0)

	var old_tier: String = entity.status_tier
	if entity.status_score > GameConfig.STATUS_TIER_ELITE:
		entity.status_tier = "elite"
	elif entity.status_score > GameConfig.STATUS_TIER_RESPECTED:
		entity.status_tier = "respected"
	elif entity.status_score > GameConfig.STATUS_TIER_MARGINAL:
		entity.status_tier = "common"
	elif entity.status_score > GameConfig.STATUS_TIER_OUTCAST:
		entity.status_tier = "marginal"
	else:
		entity.status_tier = "outcast"

	if old_tier != entity.status_tier:
		SimulationBus.status_tier_changed.emit(
			entity.id, old_tier, entity.status_tier, _current_tick)


## ── Leveling Effectiveness & Phase ────────────────────────

func _compute_leveling(settlement: RefCounted) -> void:
	var pop_count: int = 0
	var stockpile_food: float = 0.0
	for m_idx in range(settlement.member_ids.size()):
		var mid = settlement.member_ids[m_idx]
		var entity = _entity_manager.get_entity(mid)
		if entity != null and entity.is_alive:
			pop_count += 1
			stockpile_food += entity.inventory.get("food", 0.0)

	var pop: float = float(pop_count)
	var dunbar_factor: float = minf(1.0, GameConfig.LEVELING_DUNBAR_N / maxf(pop, 1.0))
	var mobility_factor: float = 1.0 - GameConfig.LEVELING_SEDENTISM_DEFAULT
	var need_30days: float = pop * GameConfig.HUNGER_DECAY_RATE * float(GameConfig.TICKS_PER_DAY) * 30.0
	var surplus_ratio: float = stockpile_food / maxf(need_30days, 1.0)
	settlement.leveling_effectiveness = dunbar_factor * mobility_factor * (1.0 / (1.0 + surplus_ratio))

	var gini: float = settlement.gini_coefficient
	var leveling: float = settlement.leveling_effectiveness
	var old_phase: String = settlement.stratification_phase

	if gini < GameConfig.GINI_UNREST_THRESHOLD and leveling > 0.5:
		settlement.stratification_phase = "egalitarian"
	elif gini < GameConfig.GINI_ENTRENCHED_THRESHOLD and leveling > 0.2:
		settlement.stratification_phase = "transitional"
	else:
		settlement.stratification_phase = "stratified"

	if old_phase != settlement.stratification_phase:
		SimulationBus.stratification_phase_changed.emit(
			settlement.id, old_phase, settlement.stratification_phase, _current_tick)
