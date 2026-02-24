extends "res://scripts/core/simulation/simulation_system.gd"

## [Weber 1922, French & Raven 1959, Boehm 1999, Henrich & Gil-White 2001]
## Composite leadership election with immediate election on vacant leader
## and per-settlement re-election cycles.
##
## Scoring: 6-factor composite blending multiple authority bases:
##   charisma (0.25), wisdom (0.15), trustworthiness (0.15),
##   intimidation (0.15), social capital (0.15), age respect (0.15)
##
## Election triggers:
##   - leader_id == -1 → immediate election (Bug Fix 1)
##   - tick - last_election_tick >= LEADER_REELECTION_INTERVAL → scheduled re-election (Bug Fix 2)
##
## Notifications emitted via SimulationBus.emit_event for HUD display (Bug Fix 3).

var _entity_manager: RefCounted
var _settlement_manager: RefCounted
var _relationship_manager: RefCounted


func init(entity_manager: RefCounted, settlement_manager: RefCounted, relationship_manager: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_settlement_manager = settlement_manager
	_relationship_manager = relationship_manager
	system_name = "leader"
	priority = 52           ## runs before value_system (priority 55)
	tick_interval = GameConfig.LEADER_CHECK_INTERVAL


func execute_tick(tick: int) -> void:
	if _settlement_manager == null or _entity_manager == null:
		return
	var all_settlements: Array = _settlement_manager.get_all_settlements()
	for i in range(all_settlements.size()):
		var settlement: RefCounted = all_settlements[i]
		_check_and_elect(settlement, tick)


## Determine if election should run for this settlement, then execute if needed.
func _check_and_elect(settlement: RefCounted, tick: int) -> void:
	## Step 1: Validate existing leader — emit loss if dead or departed
	if settlement.leader_id > -1:
		var current: RefCounted = _entity_manager.get_entity(settlement.leader_id)
		var still_valid: bool = current != null and current.is_alive \
				and current.settlement_id == settlement.id
		if not still_valid:
			SimulationBus.leader_lost.emit(settlement.id, tick)
			_emit_leader_lost_notification(settlement, tick)
			settlement.leader_id = -1
			## Fall through — will attempt immediate re-election below

	## Step 2: Determine if election should run
	var needs_election: bool = false

	if settlement.leader_id == -1:
		## No leader — immediate election (Bug Fix 1)
		needs_election = true
	elif settlement.last_election_tick >= 0:
		## Has leader — check if re-election cycle is due (per-settlement, Bug Fix 2)
		var since_last: int = tick - settlement.last_election_tick
		if since_last >= GameConfig.LEADER_REELECTION_INTERVAL:
			needs_election = true
	else:
		## Has leader but last_election_tick is -1 (legacy save migration)
		## Set it to current tick and skip — next cycle will trigger normally
		settlement.last_election_tick = tick

	if not needs_election:
		return

	## Step 3: Run election
	_run_election(settlement, tick)


## [Weber 1922, Boehm 1999] Run full election with composite scoring.
## Incumbent participates as a candidate — a higher-scoring challenger can unseat them.
func _run_election(settlement: RefCounted, tick: int) -> void:
	## Gather candidates: adults + elders in this settlement
	var candidates: Array = []
	for mid in settlement.member_ids:
		var entity: RefCounted = _entity_manager.get_entity(mid)
		if entity == null or not entity.is_alive:
			continue
		if entity.age_stage == "adult" or entity.age_stage == "elder":
			candidates.append(entity)

	if candidates.size() < GameConfig.LEADER_MIN_POPULATION:
		return

	## Score each candidate using composite formula
	var best_entity: RefCounted = null
	var best_score: float = -1.0

	for i in range(candidates.size()):
		var c: RefCounted = candidates[i]
		var score: float = _compute_leader_score(c, settlement)

		if best_entity == null or score > best_score + GameConfig.LEADER_CHARISMA_TIE_MARGIN:
			best_entity = c
			best_score = score
		elif absf(score - best_score) <= GameConfig.LEADER_CHARISMA_TIE_MARGIN:
			## Tiebreak: DERIVED_POPULARITY
			var pop_c: float = StatQuery.get_normalized(c, &"DERIVED_POPULARITY")
			var pop_best: float = StatQuery.get_normalized(best_entity, &"DERIVED_POPULARITY")
			if pop_c > pop_best:
				best_entity = c
				best_score = score

	if best_entity == null:
		return

	## Record election tick (per-settlement cycle)
	settlement.last_election_tick = tick

	## Emit only on leadership change (silent re-affirmation if unchanged)
	if best_entity.id != settlement.leader_id:
		settlement.leader_id = best_entity.id
		SimulationBus.leader_elected.emit(
			settlement.id,
			best_entity.id,
			best_entity.entity_name,
			best_score,
			tick,
		)
		_emit_leader_elected_notification(settlement, best_entity, tick)


## [French & Raven 1959, Boehm 1999, Henrich & Gil-White 2001]
## Composite leadership score blending multiple authority bases.
## Primitive era: no single factor dominates — balanced across charisma,
## wisdom, physical authority, trust, social connections, and age.
func _compute_leader_score(entity: RefCounted, settlement: RefCounted) -> float:
	var charisma: float = StatQuery.get_normalized(entity, &"DERIVED_CHARISMA")
	var wisdom: float = StatQuery.get_normalized(entity, &"DERIVED_WISDOM")
	var trustworthiness: float = StatQuery.get_normalized(entity, &"DERIVED_TRUSTWORTHINESS")
	var intimidation: float = StatQuery.get_normalized(entity, &"DERIVED_INTIMIDATION")
	var social_cap: float = _compute_social_capital_norm(entity, settlement)
	var age_respect: float = _compute_age_respect(entity)

	return (
		charisma * GameConfig.LEADER_W_CHARISMA +
		wisdom * GameConfig.LEADER_W_WISDOM +
		trustworthiness * GameConfig.LEADER_W_TRUSTWORTHINESS +
		intimidation * GameConfig.LEADER_W_INTIMIDATION +
		social_cap * GameConfig.LEADER_W_SOCIAL_CAPITAL +
		age_respect * GameConfig.LEADER_W_AGE_RESPECT
	)


## Social capital proxy: count of meaningful relationships (affinity > 30)
## with other members of the SAME settlement, normalized.
## Full social_capital formula (strong×3 + weak×1 + bridge×5) deferred to Phase 3+.
func _compute_social_capital_norm(entity: RefCounted, settlement: RefCounted) -> float:
	if _relationship_manager == null:
		return 0.0
	var count: int = 0
	var total_members: int = settlement.member_ids.size()
	for mid in settlement.member_ids:
		if mid == entity.id:
			continue
		var rel: RefCounted = _relationship_manager.get_relationship(entity.id, mid)
		if rel != null and rel.affinity > 30.0:
			count += 1
	## Normalize: having relationships with ~50% of settlement = 1.0
	var denom: float = maxf(float(total_members - 1) * 0.5, 1.0)
	return clampf(float(count) / denom, 0.0, 1.0)


## [Simmons 1945, The Role of the Aged in Primitive Society]
## Age respect grows linearly from adulthood (18) to peak at ~58,
## then stays at max. Reflects traditional societies' elder deference.
func _compute_age_respect(entity: RefCounted) -> float:
	var age_years: float = GameConfig.get_age_years(entity.age)
	return clampf((age_years - 18.0) / 40.0, 0.0, 1.0)


func _emit_leader_elected_notification(settlement: RefCounted, leader: RefCounted, tick: int) -> void:
	SimulationBus.emit_event("leader_elected", {
		"settlement_id": settlement.id,
		"leader_id": leader.id,
		"leader_name": leader.entity_name,
		"tick": tick,
	})


func _emit_leader_lost_notification(settlement: RefCounted, tick: int) -> void:
	SimulationBus.emit_event("leader_lost", {
		"settlement_id": settlement.id,
		"tick": tick,
	})
