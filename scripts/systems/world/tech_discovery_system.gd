extends "res://scripts/core/simulation/simulation_system.gd"

## [Henrich 2004, Boyd & Richerson 1985]
## Annual discovery check per settlement for each undiscovered but unlockable tech.
## Discovery probability = base_chance + pop_bonus + knowledge/openness/intel bonuses
## priority=62 (after network_system=58, before stratification_monitor=90)
## tick_interval=TECH_DISCOVERY_INTERVAL_TICKS (annual)

const CivTechState = preload("res://scripts/core/tech/civ_tech_state.gd")
const TechState = preload("res://scripts/core/tech/tech_state.gd")
const KnowledgeType = preload("res://scripts/core/tech/knowledge_type.gd")
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_DISCOVERY_PROB_METHOD: String = "body_tech_discovery_prob"

var _entity_manager: RefCounted
var _settlement_manager: RefCounted
var _tech_tree_manager: RefCounted
var _chronicle
var _bridge_checked: bool = false
var _sim_bridge: Object = null


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
	if node != null and node.has_method(_SIM_BRIDGE_DISCOVERY_PROB_METHOD):
		_sim_bridge = node
	return _sim_bridge


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
	## V2 "discovery" block with V1 "discovery_conditions" fallback
	var disc: Dictionary = def.get("discovery", {})
	if disc.is_empty():
		disc = def.get("discovery_conditions", {})

	## V2 field names with V1 fallback
	var base: float = float(disc.get("base_chance_per_year",
		disc.get("base_discovery_chance_per_year", 0.20)))
	var pop_min: int = int(disc.get("required_population",
		disc.get("population_minimum", 2)))
	var pop: int = settlement.member_ids.size()

	if pop < pop_min:
		return 0.0

	var pop_bonus: float = float(pop - pop_min) * GameConfig.TECH_DISCOVERY_POP_SCALE

	## Required skills check
	var req_skills: Dictionary = disc.get("required_skills", {})
	for skill_id in req_skills:
		var min_level: int = int(req_skills[skill_id])
		if not _any_member_has_skill(settlement, skill_id, min_level):
			return 0.0

	## V2 modifiers block with V1 flat field fallback
	var mods: Dictionary = disc.get("modifiers", {})
	var knowledge_bonus: float = float(mods.get("knowledge_value",
		disc.get("knowledge_value_bonus", 0.0))) \
		* _settlement_avg_value(settlement, &"KNOWLEDGE")
	var openness_bonus: float = float(mods.get("openness",
		disc.get("openness_bonus", 0.0))) \
		* _settlement_avg_hexaco(settlement, "O")
	var logical_bonus: float = float(mods.get("logical_intel",
		disc.get("logical_intel_bonus", 0.0))) \
		* _settlement_avg_intel(settlement, "logical")
	var naturalistic_bonus: float = float(mods.get("naturalistic_intel", 0.0)) \
		* _settlement_avg_intel(settlement, "naturalistic")

	## Soft prereq bonus (V2 only)
	var prereq: Dictionary = def.get("prereq_logic", {})
	var soft_bonus: float = 0.0
	for soft_tech in prereq.get("soft", []):
		if settlement.has_tech(soft_tech):
			soft_bonus += GameConfig.TECH_SOFT_PREREQ_BONUS

	## Rediscovery bonus (if tech was previously forgotten)
	var rediscovery_bonus: float = 0.0
	if settlement.tech_states.has(tech_id):
		var cts: Dictionary = settlement.tech_states[tech_id]
		var state_enum: int = CivTechState.get_state_enum(cts)
		if TechState.is_forgotten(state_enum):
			var kt: int = KnowledgeType.resolve_from_def(def)
			var kt_config: Dictionary = KnowledgeType.CONFIG[kt]
			var memory: float = float(cts.get("cultural_memory", 0.0))
			rediscovery_bonus = float(kt_config["rediscovery_bonus"]) \
				* (0.5 + 0.5 * memory)

	var annual_total: float = base + pop_bonus + knowledge_bonus + openness_bonus \
		+ logical_bonus + naturalistic_bonus + soft_bonus + rediscovery_bonus
	annual_total = clampf(annual_total, 0.0,
		base + GameConfig.TECH_DISCOVERY_MAX_BONUS + rediscovery_bonus)

	## Convert annual probability → per-check probability
	## With monthly checks (interval=365), checks_per_year=12.
	## P_check = 1 - (1 - P_annual)^(1/checks_per_year)
	## This preserves the same expected annual discovery rate.
	var checks_per_year: float = 4380.0 / float(tick_interval)
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_DISCOVERY_PROB_METHOD,
			base,
			pop_bonus,
			knowledge_bonus,
			openness_bonus,
			logical_bonus,
			naturalistic_bonus,
			soft_bonus,
			rediscovery_bonus,
			float(GameConfig.TECH_DISCOVERY_MAX_BONUS),
			checks_per_year,
		)
		if rust_variant != null:
			return clampf(float(rust_variant), 0.0, 1.0)
	if checks_per_year <= 1.0 or annual_total >= 1.0:
		return clampf(annual_total, 0.0, 1.0)
	return 1.0 - pow(1.0 - annual_total, 1.0 / checks_per_year)


func _apply_discovery(settlement: RefCounted, tech_id: String, tick: int) -> void:
	## Find discoverer (highest relevant skill holder)
	var discoverer_id: int = _find_best_discoverer(settlement, tech_id)

	## Determine old state for signal
	var old_state: String = "unknown"
	if settlement.tech_states.has(tech_id):
		old_state = settlement.tech_states[tech_id].get("state", "unknown")

	## Create CivTechState
	var cts: Dictionary = CivTechState.create_discovered(tech_id, tick, discoverer_id)
	settlement.tech_states[tech_id] = cts

	## Update era
	_tech_tree_manager.update_era(settlement)

	## Chronicle
	if _chronicle != null:
		_chronicle.log_event("tech_discovered", discoverer_id,
			"[Settlement %d] discovered %s" % [settlement.id, tech_id],
			4, [], tick,
			{"key": "TECH_DISCOVERED_FMT", "params": {"tech": tech_id}})

	SimulationBus.tech_state_changed.emit(
		settlement.id, tech_id, old_state, "known_low", tick)
	SimulationBus.emit_event("tech_discovered", {
		"settlement_id": settlement.id,
		"tech_id": tech_id,
		"tick": tick,
		"discoverer_id": discoverer_id,
	})


func _find_best_discoverer(settlement: RefCounted, tech_id: String) -> int:
	var def: Dictionary = _tech_tree_manager.get_def(tech_id)
	var disc: Dictionary = def.get("discovery", {})
	if disc.is_empty():
		disc = def.get("discovery_conditions", {})
	var req_skills: Dictionary = disc.get("required_skills", {})
	var best_id: int = -1
	var best_score: float = -1.0
	for mid in settlement.member_ids:
		var e: RefCounted = _entity_manager.get_entity(mid)
		if e == null or not e.is_alive:
			continue
		if e.age_stage != "adult" and e.age_stage != "elder":
			continue
		var score: float = 0.0
		for skill_id in req_skills:
			score += float(e.skill_levels.get(StringName(skill_id), 0))
		score += float(e.intelligences.get("logical", 0.5))
		if e.personality != null:
			score += float(e.personality.axes.get("O", 0.5))
		else:
			score += 0.5
		if score > best_score:
			best_score = score
			best_id = e.id
	return best_id


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
