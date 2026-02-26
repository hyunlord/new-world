extends "res://scripts/core/simulation/simulation_system.gd"

## [Granovetter 1973, Burt 2004, Weber 1922, Milgram 1963, Tilly 1978]
## Three responsibilities:
##   1. Social capital computation: strong×3 + weak×1 + bridge×5 + rep×10
##   2. Weber authority type update for settlements
##   3. Revolution risk evaluation + uprising event
## priority=58 (after value_system=55, before stratification_monitor=90)
## tick_interval=REVOLUTION_TICK_INTERVAL (annual)

var _entity_manager: RefCounted
var _settlement_manager: RefCounted
var _relationship_manager: RefCounted
var _reputation_manager: RefCounted


func _init() -> void:
	system_name = "network"
	priority = 58
	tick_interval = GameConfig.REVOLUTION_TICK_INTERVAL


func init(entity_manager: RefCounted, settlement_manager: RefCounted,
		relationship_manager: RefCounted, reputation_manager: RefCounted = null) -> void:
	_entity_manager = entity_manager
	_settlement_manager = settlement_manager
	_relationship_manager = relationship_manager
	_reputation_manager = reputation_manager


func execute_tick(tick: int) -> void:
	if _settlement_manager == null or _entity_manager == null:
		return
	var settlements: Array = _settlement_manager.get_all_settlements()
	for settlement in settlements:
		_update_authority_type(settlement)
		_compute_settlement_social_capital(settlement)
		_evaluate_revolution_risk(settlement, tick)


## ── 1. Social Capital ────────────────────────────────────────────────────────

## Compute social capital for all alive members of a settlement.
## Writes result to entity.stat_cache[&"DERIVED_SOCIAL_CAPITAL"] (0~1000 scale).
func _compute_settlement_social_capital(settlement: RefCounted) -> void:
	if _relationship_manager == null:
		return
	var member_set: Dictionary = {}
	for mid in settlement.member_ids:
		member_set[mid] = true

	for mid in settlement.member_ids:
		var entity: RefCounted = _entity_manager.get_entity(mid)
		if entity == null or not entity.is_alive:
			continue
		var sc: float = _compute_entity_social_capital(entity, member_set, settlement)
		entity.stat_cache[&"DERIVED_SOCIAL_CAPITAL"] = int(clampf(sc, 0.0, 1.0) * 1000.0)


## [Burt 2004] social_capital = strong×3 + weak×1 + bridge×5 + reputation×10, normalized.
func _compute_entity_social_capital(
		entity: RefCounted,
		member_set: Dictionary,
		settlement: RefCounted) -> float:

	var strong_count: float = 0.0
	var weak_count: float = 0.0
	var bridge_count: float = 0.0

	var rels: Array = _relationship_manager.get_relationships_for(entity.id)
	for item in rels:
		var rel: RefCounted = item["rel"]
		var other_id: int = item["other_id"]
		var is_bridge: bool = not member_set.has(other_id)
		match rel.tie_type:
			"strong", "intimate":
				if is_bridge:
					bridge_count += 1.0
				else:
					strong_count += 1.0
			"moderate", "weak":
				if is_bridge:
					bridge_count += 0.5
				else:
					weak_count += 1.0

	var rep_score: float = 0.0
	if _reputation_manager != null:
		var avg_rep: Dictionary = _reputation_manager.get_settlement_average(entity.id, settlement.member_ids)
		rep_score = (avg_rep.get("overall", 0.0) + 1.0) / 2.0  ## remap -1~1 to 0~1

	var raw_sc: float = (
		strong_count * GameConfig.NETWORK_SOCIAL_CAP_STRONG_W
		+ weak_count * GameConfig.NETWORK_SOCIAL_CAP_WEAK_W
		+ bridge_count * GameConfig.NETWORK_SOCIAL_CAP_BRIDGE_W
		+ rep_score * GameConfig.NETWORK_SOCIAL_CAP_REP_W
	)
	return raw_sc / GameConfig.NETWORK_SOCIAL_CAP_NORM_DIV


## ── 2. Weber Authority Type ──────────────────────────────────────────────────

## [Weber 1922] Determine authority type from settlement's shared_values.
## traditional: TRADITION>0.3 AND LAW<0.1
## rational_legal: LAW>0.3
## charismatic: default
func _update_authority_type(settlement: RefCounted) -> void:
	var sv: Dictionary = settlement.shared_values
	var tradition: float = float(sv.get(&"TRADITION", 0.0))
	var law: float = float(sv.get(&"LAW", 0.0))

	var old_type: String = settlement.authority_type
	var new_type: String
	if tradition > GameConfig.AUTHORITY_TRADITIONAL_TRADITION_MIN \
			and law < GameConfig.AUTHORITY_TRADITIONAL_LAW_MAX:
		new_type = "traditional"
	elif law > GameConfig.AUTHORITY_RATIONAL_LAW_MIN:
		new_type = "rational_legal"
	else:
		new_type = "charismatic"

	if new_type != old_type:
		settlement.authority_type = new_type
		SimulationBus.emit_event("authority_type_changed", {
			"settlement_id": settlement.id,
			"old_type": old_type,
			"new_type": new_type,
		})


## ── 3. Revolution Risk ───────────────────────────────────────────────────────

## [Tilly 1978, Human Definition v3 §17] Check and potentially trigger revolution.
func _evaluate_revolution_risk(settlement: RefCounted, tick: int) -> void:
	if settlement.revolution_cooldown_tick > 0:
		if tick - settlement.revolution_cooldown_tick < GameConfig.REVOLUTION_COOLDOWN_TICKS:
			return

	var risk: float = _compute_revolution_risk(settlement)

	var rebel_leader: RefCounted = _find_rebel_leader(settlement)
	if rebel_leader != null:
		risk = minf(risk * GameConfig.REVOLUTION_CHARISMA_MULTIPLIER, 1.0)

	if risk > GameConfig.REVOLUTION_RISK_THRESHOLD:
		_trigger_uprising(settlement, rebel_leader, tick, risk)


## [Tilly 1978] Composite risk from 5 equal-weight components (each 0~1).
func _compute_revolution_risk(settlement: RefCounted) -> float:
	var alive_members: Array = []
	for mid in settlement.member_ids:
		var e: RefCounted = _entity_manager.get_entity(mid)
		if e != null and e.is_alive:
			alive_members.append(e)
	if alive_members.is_empty():
		return 0.0

	## Component 1: Unhappiness = 1 - avg valence (normalized 0~1)
	var total_valence: float = 0.0
	for e in alive_members:
		if e.emotion_data != null:
			total_valence += (e.emotion_data.valence + 100.0) / 200.0
		else:
			total_valence += 0.5
	var unhappiness: float = 1.0 - (total_valence / float(alive_members.size()))

	## Component 2: Need frustration = avg unmet basic needs
	var total_frustration: float = 0.0
	for e in alive_members:
		var hunf: float = maxf(0.0, 1.0 - e.hunger)
		var enf: float  = maxf(0.0, 1.0 - e.energy)
		var saf: float  = maxf(0.0, 1.0 - e.safety)
		total_frustration += (hunf + enf + saf) / 3.0
	var frustration: float = total_frustration / float(alive_members.size())

	## Component 3: Inequality = Gini coefficient
	var inequality: float = settlement.gini_coefficient

	## Component 4: Leader unpopularity = 1 - leader_popularity_norm
	var leader_unpopularity: float = 0.5
	if settlement.leader_id > -1:
		var leader: RefCounted = _entity_manager.get_entity(settlement.leader_id)
		if leader != null and leader.is_alive:
			var pop: float = float(leader.stat_cache.get(&"DERIVED_POPULARITY", 500)) / 1000.0
			leader_unpopularity = 1.0 - pop

	## Component 5: Independence-seekers ratio
	var independence_count: int = 0
	for e in alive_members:
		if float(e.values.get(&"INDEPENDENCE", 0.0)) > 0.3:
			independence_count += 1
	var independence_ratio: float = float(independence_count) / float(alive_members.size())

	return (unhappiness + frustration + inequality + leader_unpopularity + independence_ratio) / 5.0


## Find highest-charisma non-leader adult/elder with INDEPENDENCE > 0.3.
func _find_rebel_leader(settlement: RefCounted) -> RefCounted:
	var best: RefCounted = null
	var best_score: float = -1.0
	for mid in settlement.member_ids:
		var e: RefCounted = _entity_manager.get_entity(mid)
		if e == null or not e.is_alive or e.id == settlement.leader_id:
			continue
		if e.age_stage != "adult" and e.age_stage != "elder":
			continue
		if float(e.values.get(&"INDEPENDENCE", 0.0)) < 0.3:
			continue
		var charisma: float = float(e.stat_cache.get(&"DERIVED_CHARISMA", 0)) / 1000.0
		if charisma > best_score:
			best_score = charisma
			best = e
	return best


## Depose current leader, install rebel (or chaos), emit revolution event.
func _trigger_uprising(
		settlement: RefCounted,
		rebel: RefCounted,
		tick: int,
		risk: float) -> void:

	settlement.revolution_cooldown_tick = tick

	var old_leader_name: String = "none"
	if settlement.leader_id > -1:
		var old_leader: RefCounted = _entity_manager.get_entity(settlement.leader_id)
		if old_leader != null:
			old_leader_name = old_leader.entity_name

	settlement.leader_id = -1

	var new_leader_name: String = "none"
	if rebel != null:
		settlement.leader_id = rebel.id
		new_leader_name = rebel.entity_name

	SimulationBus.emit_event("revolution", {
		"settlement_id":   settlement.id,
		"old_leader_name": old_leader_name,
		"new_leader_name": new_leader_name,
		"risk_score":      risk,
		"tick":            tick,
	})
