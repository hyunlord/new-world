extends "res://scripts/core/simulation/simulation_system.gd"

## [Henrich 2004, Boyd & Richerson 1985]
## Annual discovery check per settlement for each undiscovered but unlockable tech.
## Discovery probability = base_chance + pop_bonus + knowledge/openness/intel bonuses
## priority=62 (after network_system=58, before stratification_monitor=90)
## tick_interval=TECH_DISCOVERY_INTERVAL_TICKS (annual)

var _entity_manager: RefCounted
var _settlement_manager: RefCounted
var _tech_tree_manager: RefCounted
var _chronicle


func _init() -> void:
	system_name = "tech_discovery"
	priority = 62
	tick_interval = GameConfig.TECH_DISCOVERY_INTERVAL_TICKS


func init(p_entity_manager: RefCounted, p_settlement_manager: RefCounted,
		p_tech_tree_manager: RefCounted, p_chronicle) -> void:
	_entity_manager = p_entity_manager
	_settlement_manager = p_settlement_manager
	_tech_tree_manager = p_tech_tree_manager
	_chronicle = p_chronicle


func execute_tick(tick: int) -> void:
	if _settlement_manager == null:
		return
	for settlement in _settlement_manager.get_all_settlements():
		_check_discoveries(settlement, tick)


func _check_discoveries(settlement: RefCounted, tick: int) -> void:
	var all_ids: Array = _tech_tree_manager.get_all_ids()
	for tech_id in all_ids:
		if not _tech_tree_manager.can_discover(settlement, tech_id):
			continue
		var prob: float = _compute_discovery_prob(settlement, tech_id)
		if randf() < prob:
			_apply_discovery(settlement, tech_id, tick)


## [Henrich 2004] discovery_prob = base + pop_bonus + innovator_bonuses
func _compute_discovery_prob(settlement: RefCounted, tech_id: String) -> float:
	var def: Dictionary = _tech_tree_manager.get_def(tech_id)
	var cond: Dictionary = def.get("discovery_conditions", {})

	var base: float = float(cond.get("base_discovery_chance_per_year", 0.20))
	var pop_min: int = int(cond.get("population_minimum", 2))
	var pop: int = settlement.member_ids.size()

	if pop < pop_min:
		return 0.0  ## population gate not met

	## Population bonus: +TECH_DISCOVERY_POP_SCALE per person above minimum
	var pop_bonus: float = float(pop - pop_min) * GameConfig.TECH_DISCOVERY_POP_SCALE

	## Required skills check (any alive adult/elder member must meet the minimum)
	var req_skills: Dictionary = cond.get("required_skills", {})
	for skill_id in req_skills:
		var min_level: int = int(req_skills[skill_id])
		if not _any_member_has_skill(settlement, skill_id, min_level):
			return 0.0

	## Innovator bonus: knowledge value × openness × logical intel
	var knowledge_bonus: float = float(cond.get("knowledge_value_bonus", 0.0)) \
		* _settlement_avg_value(settlement, &"KNOWLEDGE")
	var openness_bonus: float = float(cond.get("openness_bonus", 0.0)) \
		* _settlement_avg_hexaco(settlement, "O")
	var logical_bonus: float = float(cond.get("logical_intel_bonus", 0.0)) \
		* _settlement_avg_intel(settlement, "logical")

	var total: float = base + pop_bonus + knowledge_bonus + openness_bonus + logical_bonus
	return clampf(total, 0.0, base + GameConfig.TECH_DISCOVERY_MAX_BONUS)


func _apply_discovery(settlement: RefCounted, tech_id: String, tick: int) -> void:
	settlement.discovered_techs.append(tech_id)

	## Update era
	_tech_tree_manager.update_era(settlement)

	## Chronicle
	if _chronicle != null:
		_chronicle.log_event("tech_discovered", -1,
			"[Settlement %d] discovered %s" % [settlement.id, tech_id],
			4, [], tick,
			{"key": "TECH_DISCOVERED_FMT", "params": {"tech": tech_id}})

	SimulationBus.emit_event("tech_discovered", {
		"settlement_id": settlement.id,
		"tech_id": tech_id,
		"tick": tick,
	})


## Helper: does any alive adult/elder member have skill >= min_level?
func _any_member_has_skill(settlement: RefCounted, skill_id: String, min_level: int) -> bool:
	var sname = StringName(skill_id)
	for mid in settlement.member_ids:
		var e: RefCounted = _entity_manager.get_entity(mid)
		if e == null or not e.is_alive:
			continue
		if e.age_stage != "adult" and e.age_stage != "elder":
			continue
		if int(e.skill_levels.get(sname, 0)) >= min_level:
			return true
	return false


func _settlement_avg_value(settlement: RefCounted, value_key: StringName) -> float:
	var total: float = 0.0
	var count: int = 0
	for mid in settlement.member_ids:
		var e: RefCounted = _entity_manager.get_entity(mid)
		if e == null or not e.is_alive:
			continue
		total += float(e.values.get(value_key, 0.0))
		count += 1
	return total / maxf(float(count), 1.0)


func _settlement_avg_hexaco(settlement: RefCounted, axis: String) -> float:
	var total: float = 0.0
	var count: int = 0
	for mid in settlement.member_ids:
		var e: RefCounted = _entity_manager.get_entity(mid)
		if e == null or not e.is_alive or e.personality == null:
			continue
		total += float(e.personality.axes.get(axis, 0.5))
		count += 1
	return total / maxf(float(count), 1.0)


func _settlement_avg_intel(settlement: RefCounted, intel_key: String) -> float:
	var total: float = 0.0
	var count: int = 0
	for mid in settlement.member_ids:
		var e: RefCounted = _entity_manager.get_entity(mid)
		if e == null or not e.is_alive:
			continue
		total += float(e.intelligences.get(intel_key, 0.5))
		count += 1
	return total / maxf(float(count), 1.0)
