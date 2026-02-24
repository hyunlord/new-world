extends "res://scripts/core/simulation/simulation_system.gd"

## [Weber 1922, French & Raven 1959, Axelrod 1997]
## Elects the highest-charisma adult as settlement leader every LEADER_TICK_INTERVAL ticks.
## Leader entity's values are passed to SettlementCulture.compute_shared_values() with
## LEADER_INFLUENCE = 0.20 weight, biasing the settlement's cultural profile.
##
## Election logic (charismatic authority):
##   candidates = adults + elders in settlement
##   score = DERIVED_CHARISMA (0~1000 in stat_cache, normalized to 0.0~1.0)
##   tiebreak = DERIVED_POPULARITY if |score_a - score_b| < LEADER_CHARISMA_TIE_MARGIN
##   winner = argmax(score)
##   minimum candidates = LEADER_MIN_POPULATION

var _entity_manager: RefCounted
var _settlement_manager: RefCounted


func init(entity_manager: RefCounted, settlement_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_settlement_manager = settlement_manager
	system_name = "leader"
	priority = 52           ## runs before value_system (priority 55)
	tick_interval = GameConfig.LEADER_TICK_INTERVAL


func execute_tick(tick: int) -> void:
	if _settlement_manager == null or _entity_manager == null:
		return
	var all_settlements: Array = _settlement_manager.get_all_settlements()
	for i in range(all_settlements.size()):
		var settlement: RefCounted = all_settlements[i]
		_update_settlement_leader(settlement, tick)


## [Weber 1922] Reassess leadership every LEADER_TICK_INTERVAL ticks.
## Incumbent participates as a candidate — a higher-charisma challenger can unseat them.
## leader_elected signal fires only when the winner changes.
func _update_settlement_leader(settlement: RefCounted, tick: int) -> void:
	## Step 1: validate existing leader — emit loss if dead or departed
	if settlement.leader_id > -1:
		var current: RefCounted = _entity_manager.get_entity(settlement.leader_id)
		var still_valid: bool = current != null and current.is_alive \
				and current.settlement_id == settlement.id
		if not still_valid:
			SimulationBus.leader_lost.emit(settlement.id, tick)
			settlement.leader_id = -1
	## Do NOT return here — always run full election scoring.
	## Incumbent participates as a candidate; challenger wins only if score is higher.

	## Step 2: gather candidates (adult + elder in this settlement)
	var candidates: Array = []
	for mid in settlement.member_ids:
		var entity: RefCounted = _entity_manager.get_entity(mid)
		if entity == null or not entity.is_alive:
			continue
		if entity.age_stage == "adult" or entity.age_stage == "elder":
			candidates.append(entity)

	if candidates.size() < GameConfig.LEADER_MIN_POPULATION:
		return

	## Step 3: score by DERIVED_CHARISMA, tiebreak by DERIVED_POPULARITY
	var best_entity: RefCounted = null
	var best_score: float = -1.0

	for i in range(candidates.size()):
		var c: RefCounted = candidates[i]
		var charisma_norm: float = StatQuery.get_normalized(c, &"DERIVED_CHARISMA")
		if best_entity == null:
			best_entity = c
			best_score = charisma_norm
			continue
		var diff: float = charisma_norm - best_score
		if diff > GameConfig.LEADER_CHARISMA_TIE_MARGIN:
			best_entity = c
			best_score = charisma_norm
		elif absf(diff) <= GameConfig.LEADER_CHARISMA_TIE_MARGIN:
			## Tiebreak: use DERIVED_POPULARITY
			var pop_c: float = StatQuery.get_normalized(c, &"DERIVED_POPULARITY")
			var pop_best: float = StatQuery.get_normalized(best_entity, &"DERIVED_POPULARITY")
			if pop_c > pop_best:
				best_entity = c
				best_score = charisma_norm

	if best_entity == null:
		return

	## Step 4: emit only on leadership change (silent re-affirmation if unchanged)
	if best_entity.id != settlement.leader_id:
		settlement.leader_id = best_entity.id
		SimulationBus.leader_elected.emit(
			settlement.id,
			best_entity.id,
			best_entity.entity_name,
			best_score,
			tick,
		)
