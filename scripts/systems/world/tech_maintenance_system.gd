extends "res://scripts/core/simulation/simulation_system.gd"

## [Henrich 2004, Boyd & Richerson 1985, Tainter 1988]
## Annual maintenance check: can each settlement sustain its known technologies?
## Technologies that lose practitioners, infrastructure, or critical population
## begin atrophying and can eventually be forgotten.
##
## State machine:
##   UNKNOWN → KNOWN_LOW → KNOWN_STABLE → (atrophy) → FORGOTTEN_RECENT → FORGOTTEN_LONG
##
## priority=63 (immediately after tech_discovery=62)
## tick_interval=TECH_DISCOVERY_INTERVAL_TICKS (annual, same as discovery)

const CivTechState = preload("res://scripts/core/tech/civ_tech_state.gd")
const TechState = preload("res://scripts/core/tech/tech_state.gd")
const KnowledgeType = preload("res://scripts/core/tech/knowledge_type.gd")
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_TECH_MEMORY_DECAY_METHOD: String = "body_tech_cultural_memory_decay"

var _entity_manager: RefCounted
var _settlement_manager: RefCounted
var _tech_tree_manager: RefCounted
var _chronicle
var _bridge_checked: bool = false
var _sim_bridge: Object = null


func _init() -> void:
	system_name = "tech_maintenance"
	priority = 63
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
	if node != null and node.has_method(_SIM_BRIDGE_TECH_MEMORY_DECAY_METHOD):
		_sim_bridge = node
	return _sim_bridge


func execute_tick(tick: int) -> void:
	if _settlement_manager == null:
		return
	for settlement in _settlement_manager.get_all_settlements():
		_maintain_techs(settlement, tick)


func _maintain_techs(settlement: RefCounted, tick: int) -> void:
	var tech_ids: Array = settlement.tech_states.keys().duplicate()
	for tech_id in tech_ids:
		var cts: Dictionary = settlement.tech_states[tech_id]
		var state_enum: int = CivTechState.get_state_enum(cts)

		match state_enum:
			TechState.State.KNOWN_LOW:
				_check_stabilization(settlement, tech_id, cts, tick)
				_check_atrophy(settlement, tech_id, cts, tick)
			TechState.State.KNOWN_STABLE:
				_check_atrophy(settlement, tech_id, cts, tick)
			TechState.State.FORGOTTEN_RECENT:
				## Continue incrementing atrophy_years to track time in forgotten state
				cts["atrophy_years"] = float(cts.get("atrophy_years", 0.0)) + 1.0
				_decay_cultural_memory(settlement, tech_id, cts, tick)
				_check_forgotten_long(settlement, tech_id, cts, tick)
			TechState.State.FORGOTTEN_LONG:
				cts["atrophy_years"] = float(cts.get("atrophy_years", 0.0)) + 1.0
				_decay_cultural_memory(settlement, tech_id, cts, tick)


## ── Practitioner Counting ────────────────────────────────────────────────────

## Count living adult/elder members who qualify as practitioners for this tech.
func _count_practitioners(settlement: RefCounted, tech_id: String) -> int:
	var def: Dictionary = _tech_tree_manager.get_def(tech_id)
	var maint: Dictionary = def.get("maintenance", {})
	var carrier_roles: Array = maint.get("carrier_roles", [])
	var disc: Dictionary = def.get("discovery", {})
	var req_skills: Dictionary = disc.get("required_skills", {})

	var has_roles: bool = not carrier_roles.is_empty()
	var has_skills: bool = not req_skills.is_empty()

	var count: int = 0
	for mid in settlement.member_ids:
		var e: RefCounted = _entity_manager.get_entity(mid)
		if e == null or not e.is_alive:
			continue
		if e.age_stage != "adult" and e.age_stage != "elder":
			continue

		if has_roles:
			## Option A: carrier_roles match (occupation-based, exclusive gate)
			if carrier_roles.has(e.occupation):
				count += 1
		elif has_skills:
			## Option B: skill-level match (only if no carrier_roles defined)
			var meets_all: bool = true
			for skill_id in req_skills:
				if int(e.skill_levels.get(StringName(skill_id), 0)) < int(req_skills[skill_id]):
					meets_all = false
					break
			if meets_all:
				count += 1
		else:
			## Option C: no requirements defined — all adults count
			count += 1

	return count


## Count effective carriers: practitioners + artifact/institution bonuses.
func _count_effective_carriers(settlement: RefCounted, tech_id: String,
		practitioner_count: int) -> int:
	var def: Dictionary = _tech_tree_manager.get_def(tech_id)
	var maint: Dictionary = def.get("maintenance", {})
	var artifact_carriers: Array = maint.get("artifact_carriers", [])
	var institution_help: Array = maint.get("institution_tags_help", [])

	var effective: int = practitioner_count

	for artifact_tag in artifact_carriers:
		if _settlement_has_building_tag(settlement, artifact_tag):
			effective += GameConfig.TECH_ARTIFACT_CARRIER_VALUE

	for inst_tag in institution_help:
		if _settlement_has_institution_tag(settlement, inst_tag):
			effective += GameConfig.TECH_INSTITUTION_CARRIER_BONUS

	return effective


## ── Atrophy Check (Core Regression Logic) ────────────────────────────────────

func _check_atrophy(settlement: RefCounted, tech_id: String,
		cts: Dictionary, tick: int) -> void:
	var def: Dictionary = _tech_tree_manager.get_def(tech_id)
	var maint: Dictionary = def.get("maintenance", {})
	var kt: int = KnowledgeType.resolve_from_def(def)
	var kt_config: Dictionary = KnowledgeType.CONFIG[kt]

	var min_practitioners: int = int(maint.get("min_practitioners",
		kt_config["min_practitioners"]))
	var grace_years: float = float(maint.get("regression_grace_years",
		kt_config["regression_grace_years"]))

	## Count current practitioners and effective carriers
	var practitioners: int = _count_practitioners(settlement, tech_id)
	var effective: int = _count_effective_carriers(settlement, tech_id, practitioners)

	cts["practitioner_count"] = practitioners
	cts["effective_carriers"] = effective

	## Artifact grace extension
	var artifact_bonus: float = 0.0
	var artifact_carriers: Array = maint.get("artifact_carriers", [])
	for ac in artifact_carriers:
		if _settlement_has_building_tag(settlement, ac):
			artifact_bonus += GameConfig.TECH_ARTIFACT_GRACE_BONUS
	var effective_grace: float = grace_years * (1.0 + artifact_bonus)

	## --- ATROPHY CALCULATION ---
	if effective < min_practitioners:
		## UNDERMAINTAINED — atrophy increases
		var base_atrophy: float = GameConfig.TECH_ATROPHY_BASE_RATE

		## Population factor [Henrich 2004]: excess pop above min slows atrophy
		var pop_above_min: int = maxi(settlement.member_ids.size() - min_practitioners, 0)
		var pop_reduction: float = minf(
			float(pop_above_min) * GameConfig.TECH_POP_MAINTENANCE_BONUS,
			GameConfig.TECH_POP_MAINTENANCE_CAP
		)

		## Active use factor
		var use_reduction: float = 0.0
		if maint.get("requires_active_use", false):
			var last_use: int = int(cts.get("last_active_use_tick", 0))
			var years_since_use: float = float(tick - last_use) \
				/ float(GameConfig.TECH_DISCOVERY_INTERVAL_TICKS)
			if years_since_use < 1.0:
				use_reduction = GameConfig.TECH_ACTIVE_USE_ATROPHY_REDUCTION

		var final_atrophy: float = maxf(
			base_atrophy * (1.0 - pop_reduction - use_reduction), 0.1)

		var prev_atrophy: float = float(cts.get("atrophy_years", 0.0))
		cts["atrophy_years"] = prev_atrophy + final_atrophy

		## Emit warning when atrophy first starts
		if prev_atrophy <= 0.0:
			emit_event("tech_atrophy_warning", {
				"settlement_id": settlement.id,
				"tech_id": tech_id,
				"practitioners": practitioners,
				"min_required": min_practitioners,
				"tick": tick,
			})

		## CHECK: exceeded grace period → REGRESSION
		if float(cts["atrophy_years"]) > effective_grace:
			cts["effective_grace_at_loss"] = effective_grace
			_transition_to_forgotten(settlement, tech_id, cts, def, tick)
	else:
		## WELL-MAINTAINED — atrophy decreases (recovery)
		cts["atrophy_years"] = maxf(
			float(cts.get("atrophy_years", 0.0)) - GameConfig.TECH_ATROPHY_RECOVERY_RATE,
			0.0)


## ── State Transitions ────────────────────────────────────────────────────────

## KNOWN_LOW → KNOWN_STABLE: when atrophy stays at 0 for N years after discovery.
func _check_stabilization(settlement: RefCounted, tech_id: String,
		cts: Dictionary, tick: int) -> void:
	var years_since_discovery: float = float(tick - int(cts.get("discovered_tick", 0))) \
		/ float(GameConfig.TECH_DISCOVERY_INTERVAL_TICKS)
	if float(cts.get("atrophy_years", 0.0)) <= 0.0 \
			and years_since_discovery >= GameConfig.TECH_KNOWN_STABLE_THRESHOLD_YEARS:
		var old_state: String = cts.get("state", "known_low")
		cts["state"] = "known_stable"

		SimulationBus.tech_state_changed.emit(
			settlement.id, tech_id, old_state, "known_stable", tick)

			if _chronicle != null:
				_chronicle.log_event("tech_stabilized", -1,
					"[Settlement %d] %s knowledge is now stable" % [settlement.id, tech_id],
					3, [], tick,
					{"key": "TECH_STABILIZED_FMT",
					"params": {"tech": tech_id, "settlement": settlement.id}})


## KNOWN → FORGOTTEN_RECENT: atrophy exceeded grace period.
func _transition_to_forgotten(settlement: RefCounted, tech_id: String,
		cts: Dictionary, def: Dictionary, tick: int) -> void:
	var old_state: String = cts.get("state", "known_low")
	cts["state"] = "forgotten_recent"
	cts["cultural_memory"] = 1.0

	SimulationBus.tech_state_changed.emit(
		settlement.id, tech_id, old_state, "forgotten_recent", tick)

	var maint: Dictionary = def.get("maintenance", {})
	var kt: int = KnowledgeType.resolve_from_def(def)
	var kt_config: Dictionary = KnowledgeType.CONFIG[kt]
	var grace: int = int(maint.get("regression_grace_years",
		kt_config["regression_grace_years"]))
	SimulationBus.tech_regression_started.emit(
		settlement.id, tech_id, int(cts.get("atrophy_years", 0)), grace, tick)

	## Handle regression_fallback: grant simpler tech if lost
	var fallback: String = def.get("regression_fallback", "")
	if fallback != "" and not settlement.has_tech(fallback):
		var fallback_cts: Dictionary = CivTechState.create_discovered(fallback, tick, -1)
		fallback_cts["state"] = "known_stable"
		fallback_cts["acquisition_method"] = "fallback"
		settlement.tech_states[fallback] = fallback_cts

		emit_event("tech_fallback", {
			"settlement_id": settlement.id,
			"lost_tech": tech_id,
			"fallback_tech": fallback,
			"tick": tick,
		})

			if _chronicle != null:
				_chronicle.log_event("tech_fallback", -1,
					"[Settlement %d] fell back to %s after losing %s" \
						% [settlement.id, fallback, tech_id],
					4, [], tick,
					{"key": "TECH_FALLBACK_FMT",
					"params": {"tech": tech_id, "fallback": fallback,
						"settlement": settlement.id}})

	## Emit tech_lost signal
	SimulationBus.tech_lost.emit(
		settlement.id, tech_id, float(cts.get("cultural_memory", 1.0)), tick)

		if _chronicle != null:
			_chronicle.log_event("tech_lost", -1,
				"[Settlement %d] lost %s" % [settlement.id, tech_id],
				5, [], tick,
				{"key": "TECH_LOST_FMT",
				"params": {"tech": tech_id, "settlement": settlement.id}})


## FORGOTTEN_RECENT → FORGOTTEN_LONG after N years.
func _check_forgotten_long(settlement: RefCounted, tech_id: String,
		cts: Dictionary, tick: int) -> void:
	## Use effective grace stored at the moment of loss (includes artifact bonus)
	var grace: float
	if cts.has("effective_grace_at_loss"):
		grace = float(cts["effective_grace_at_loss"])
	else:
		var def: Dictionary = _tech_tree_manager.get_def(tech_id)
		var kt: int = KnowledgeType.resolve_from_def(def)
		var kt_config: Dictionary = KnowledgeType.CONFIG[kt]
		grace = float(def.get("maintenance", {}).get(
			"regression_grace_years", kt_config["regression_grace_years"]))

	## Time in forgotten state = atrophy_years - grace_period
	var forgotten_years: float = float(cts.get("atrophy_years", 0.0)) - grace
	if forgotten_years > GameConfig.TECH_FORGOTTEN_RECENT_YEARS:
		var old_state: String = cts.get("state", "forgotten_recent")
		cts["state"] = "forgotten_long"
		SimulationBus.tech_state_changed.emit(
			settlement.id, tech_id, old_state, "forgotten_long", tick)


## Decay cultural_memory using per-KnowledgeType decay rate from CONFIG.
func _decay_cultural_memory(_settlement: RefCounted, tech_id: String,
		cts: Dictionary, _tick: int) -> void:
	var state_str: String = cts.get("state", "")
	var def: Dictionary = _tech_tree_manager.get_def(tech_id)
	var kt: int = KnowledgeType.resolve_from_def(def)
	var kt_config: Dictionary = KnowledgeType.CONFIG[kt]
	var base_decay: float = float(kt_config.get("memory_decay_rate", 0.05))

	var decay_rate: float
	if state_str == "forgotten_recent":
		decay_rate = base_decay
	else:
		## forgotten_long decays slower — oral legends fade slowly
		decay_rate = base_decay * GameConfig.TECH_FORGOTTEN_LONG_DECAY_MULTIPLIER

	var current_memory: float = float(cts.get("cultural_memory", 1.0))
	var next_memory: float = maxf(
		current_memory - decay_rate,
		GameConfig.TECH_CULTURAL_MEMORY_FLOOR
	)
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_TECH_MEMORY_DECAY_METHOD,
			current_memory,
			base_decay,
			float(GameConfig.TECH_FORGOTTEN_LONG_DECAY_MULTIPLIER),
			float(GameConfig.TECH_CULTURAL_MEMORY_FLOOR),
			state_str == "forgotten_recent",
		)
		if rust_variant != null:
			next_memory = float(rust_variant)
	cts["cultural_memory"] = next_memory


## ── Building/Institution Helpers ─────────────────────────────────────────────
## Buildings and institutions not yet fully implemented — return false for now.
## Future: check settlement.buildings for matching tags.

func _settlement_has_building_tag(_settlement: RefCounted, _tag: String) -> bool:
	return false


func _settlement_has_institution_tag(_settlement: RefCounted, _tag: String) -> bool:
	return false
