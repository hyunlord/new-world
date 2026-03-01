extends "res://scripts/core/simulation/simulation_system.gd"

## [Lave & Wenger 1991, Vygotsky 1978, Rogers 2003]
## Within-settlement teacher-student mechanics and adoption curve tracking.
## Teachers with high skill pair with lower-skill students; knowledge spreads
## through an apprenticeship model with Vygotsky ZPD effectiveness curve.
## Rogers S-curve tracks settlement-level adoption phase.
##
## priority=62 (same tick band as tech_discovery; different cadence)
## tick_interval=TEACHING_TICK_INTERVAL (24 ticks = 1 game-day)

const CivTechState = preload("res://scripts/core/tech/civ_tech_state.gd")
const TechState = preload("res://scripts/core/tech/tech_state.gd")
const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_PROP_CULTURE_METHOD: String = "body_tech_propagation_culture_modifier"
const _SIM_BRIDGE_PROP_CARRIER_METHOD: String = "body_tech_propagation_carrier_bonus"
const _SIM_BRIDGE_PROP_FINAL_PROB_METHOD: String = "body_tech_propagation_final_prob"

var _entity_manager: RefCounted
var _settlement_manager: RefCounted
var _tech_tree_manager: RefCounted
var _chronicle
var _bridge_checked: bool = false
var _sim_bridge: Object = null

## Active teaching sessions.
## Each entry: {teacher_id: int, student_id: int, tech_id: String,
##   skill_id: String, progress: float, started_tick: int,
##   effectiveness: float, sessions_completed: int}
var _active_sessions: Array = []


func _init() -> void:
	system_name = "tech_propagation"
	priority = 62
	tick_interval = GameConfig.TEACHING_TICK_INTERVAL


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
	if node != null \
	and node.has_method(_SIM_BRIDGE_PROP_CULTURE_METHOD) \
	and node.has_method(_SIM_BRIDGE_PROP_CARRIER_METHOD) \
	and node.has_method(_SIM_BRIDGE_PROP_FINAL_PROB_METHOD):
		_sim_bridge = node
	return _sim_bridge


func execute_tick(tick: int) -> void:
	if _settlement_manager == null:
		return
	_update_teaching_sessions(tick)
	for settlement in _settlement_manager.get_all_settlements():
		_find_new_teaching_pairs(settlement, tick)
		_update_adoption_phase(settlement)


## ── Session Management ──────────────────────────────────────────────────────

func _update_teaching_sessions(tick: int) -> void:
	var to_remove: Array = []
	for i in range(_active_sessions.size()):
		var session: Dictionary = _active_sessions[i]
		var teacher: RefCounted = _entity_manager.get_entity(session["teacher_id"])
		var student: RefCounted = _entity_manager.get_entity(session["student_id"])

		## Validate session still valid
		if teacher == null or student == null \
				or not teacher.is_alive or not student.is_alive:
			_abandon_session(session, "entity_gone", tick)
			to_remove.append(i)
			continue
		if teacher.settlement_id != student.settlement_id:
			_abandon_session(session, "different_settlement", tick)
			to_remove.append(i)
			continue

		## Apply progress
		session["progress"] += session["effectiveness"]

		## Check completion of one learning cycle
		if session["progress"] >= 1.0:
			session["progress"] -= 1.0
			session["sessions_completed"] += 1

			## Student gains skill XP
			var skill_sn = StringName(session["skill_id"])
			var _old_level: int = int(student.skill_levels.get(skill_sn, 0))
			var xp_gain: float = _calculate_xp_gain(session, teacher)
			student.skill_xp[skill_sn] = float(student.skill_xp.get(skill_sn, 0.0)) + xp_gain

			## Emit completion when a learning cycle finishes
			## (actual level-up is reconciled by StatSync on its next tick)
			var current_level: int = int(student.skill_levels.get(skill_sn, 0))
			SimulationBus.teaching_session_completed.emit(
				session["teacher_id"], session["student_id"],
				session["tech_id"], session["skill_id"], current_level)

			## Update practitioner tracking
			_update_practitioner_count(student, session["tech_id"])

			## Record in apprenticeship history
			student.apprenticeship_history.append({
				"teacher_id": session["teacher_id"],
				"skill_id": session["skill_id"],
				"ticks_learned": tick - session["started_tick"],
				"final_level": current_level,
			})

			## Check for apprenticeship bond (3+ completions between same pair)
			_check_apprenticeship(session["teacher_id"], session["student_id"],
				session["tech_id"])

				if _chronicle != null:
					_chronicle.log_event("teaching_completed", session["student_id"],
						"Learned %s from teacher %d" % [session["skill_id"], session["teacher_id"]],
						3, [session["teacher_id"]], tick,
						{"key": "TOAST_TEACHING_COMPLETED",
						"params": {"student": session["student_id"],
							"teacher": session["teacher_id"],
							"skill": session["skill_id"],
							"level": current_level}})

		## Check abandonment (no progress for too long)
		var elapsed: int = tick - session["started_tick"]
		if elapsed > GameConfig.TEACHING_ABANDON_TICKS and session["sessions_completed"] == 0:
			_abandon_session(session, "no_progress", tick)
			to_remove.append(i)

	## Remove invalidated sessions (reverse order to preserve indices)
	to_remove.sort()
	for i in range(to_remove.size() - 1, -1, -1):
		var idx: int = to_remove[i]
		_clear_teaching_state(_active_sessions[idx])
		_active_sessions.remove_at(idx)


func _abandon_session(session: Dictionary, reason: String, tick: int) -> void:
	SimulationBus.teaching_session_abandoned.emit(
		session["teacher_id"], session["student_id"],
		session["tech_id"], reason)
	if _chronicle != null:
		_chronicle.log_event("teaching_abandoned", session["student_id"],
			"Teaching of %s abandoned: %s" % [session["skill_id"], reason],
			2, [session["teacher_id"]], tick,
			{"key": "TOAST_TEACHING_ABANDONED",
			"params": {"student": session["student_id"],
				"teacher": session["teacher_id"],
				"skill": session["skill_id"]}})


func _clear_teaching_state(session: Dictionary) -> void:
	var student: RefCounted = _entity_manager.get_entity(session["student_id"])
	if student != null:
		student.learning_from_id = -1
	## Only clear teacher's flag if no other sessions remain
	var teacher: RefCounted = _entity_manager.get_entity(session["teacher_id"])
	if teacher != null:
		var remaining: int = _count_students(session["teacher_id"]) - 1
		if remaining <= 0:
			teacher.teaching_target_id = -1


## ── Teacher-Student Matching ────────────────────────────────────────────────

func _find_new_teaching_pairs(settlement: RefCounted, tick: int) -> void:
	var civ_techs: Dictionary = settlement.tech_states
	for tech_id in civ_techs:
		var cts: Dictionary = civ_techs[tech_id]
		var state_enum: int = CivTechState.get_state_enum(cts)
		if not TechState.is_known(state_enum):
			continue

		var def: Dictionary = _tech_tree_manager.get_def(tech_id)
		var disc: Dictionary = def.get("discovery", {})
		if disc.is_empty():
			disc = def.get("discovery_conditions", {})
		var req_skills: Dictionary = disc.get("required_skills", {})
		if req_skills.is_empty():
			continue

		for skill_id in req_skills:
			_match_pairs_for_skill(settlement, tech_id, skill_id, tick)


func _match_pairs_for_skill(settlement: RefCounted, tech_id: String,
		skill_id: String, tick: int) -> void:
	var skill_sn = StringName(skill_id)
	var teachers: Array = []
	var students: Array = []

	for mid in settlement.member_ids:
		var e: RefCounted = _entity_manager.get_entity(mid)
		if e == null or not e.is_alive:
			continue
		if e.age_stage != "adult" and e.age_stage != "elder":
			continue

		var level: int = int(e.skill_levels.get(skill_sn, 0))

		## Potential teacher: skill >= SKILL_GAP_MIN and not at max students
		if level >= GameConfig.TEACHING_SKILL_GAP_MIN \
				and _count_students(e.id) < GameConfig.TEACHING_MAX_STUDENTS:
			teachers.append({"id": e.id, "level": level,
				"teaching_skill": _compute_teaching_skill(e)})

		## Potential student: not already learning from someone
		if e.learning_from_id == -1:
			var willingness: float = _calculate_adoption_willingness(e)
			if willingness > 0.2:
				students.append({"id": e.id, "level": level,
					"willingness": willingness})

	## Sort: best teachers first, most eager students first
	teachers.sort_custom(func(a, b): return a["teaching_skill"] > b["teaching_skill"])
	students.sort_custom(func(a, b): return a["willingness"] > b["willingness"])

	## Greedy matching
	for t_data in teachers:
		if _count_students(t_data["id"]) >= GameConfig.TEACHING_MAX_STUDENTS:
			continue
		for s_data in students:
			if _is_already_learning(s_data["id"]):
				continue
			var gap: int = t_data["level"] - s_data["level"]
			if gap < GameConfig.TEACHING_SKILL_GAP_MIN:
				continue
			## Skip if session already exists for this exact combination
			if _has_session(t_data["id"], s_data["id"], tech_id, skill_id):
				continue

			var effectiveness: float = _calculate_effectiveness(
				t_data, s_data, gap)
			var session: Dictionary = {
				"teacher_id": t_data["id"],
				"student_id": s_data["id"],
				"tech_id": tech_id,
				"skill_id": skill_id,
				"progress": 0.0,
				"started_tick": tick,
				"effectiveness": effectiveness,
				"sessions_completed": 0,
			}
			_active_sessions.append(session)

			## Set entity teaching state
			var teacher: RefCounted = _entity_manager.get_entity(t_data["id"])
			var student: RefCounted = _entity_manager.get_entity(s_data["id"])
			if teacher != null:
				teacher.teaching_target_id = s_data["id"]
				teacher.teaching_skill = t_data["teaching_skill"]
			if student != null:
				student.learning_from_id = t_data["id"]

			SimulationBus.teaching_session_started.emit(
				t_data["id"], s_data["id"], tech_id, skill_id)

				if _chronicle != null:
					_chronicle.log_event("teaching_started", s_data["id"],
						"Learning %s from teacher %d" % [skill_id, t_data["id"]],
						2, [t_data["id"]], tick,
						{"key": "TOAST_TEACHING_STARTED",
						"params": {"teacher": t_data["id"],
							"student": s_data["id"],
							"skill": skill_id}})

			if _count_students(t_data["id"]) >= GameConfig.TEACHING_MAX_STUDENTS:
				break


## ── Teaching Effectiveness [Vygotsky 1978 ZPD] ─────────────────────────────

func _calculate_effectiveness(teacher_data: Dictionary, student_data: Dictionary,
		skill_gap: int) -> float:
	var teacher: RefCounted = _entity_manager.get_entity(teacher_data["id"])
	var student: RefCounted = _entity_manager.get_entity(student_data["id"])
	if teacher == null or student == null:
		return 0.001

	## Component 1: Teacher's teaching ability (0.0-1.0)
	var teaching_ability: float = teacher_data.get("teaching_skill",
		_compute_teaching_skill(teacher))

	## Component 2: Student's learning receptivity (0.0-1.0)
	## [Gardner 1983] Logical + Intrapersonal intelligence blend
	var learning_receptivity: float = (
		float(student.intelligences.get("logical", 0.5)) * 0.5
		+ float(student.intelligences.get("intrapersonal", 0.5)) * 0.5
	)

	## Component 3: Skill gap modifier (Vygotsky ZPD curve)
	## Optimal gap = TEACHING_SKILL_GAP_OPTIMAL
	## Below: teacher barely knows more → low effectiveness
	## At optimal: sweet spot → maximum effectiveness
	## Above: teacher too advanced, can't relate → declining effectiveness
	var gap_modifier: float
	if skill_gap <= GameConfig.TEACHING_SKILL_GAP_MIN:
		gap_modifier = 0.3
	elif skill_gap <= GameConfig.TEACHING_SKILL_GAP_OPTIMAL:
		gap_modifier = 0.6 + 0.4 * (
			float(skill_gap - GameConfig.TEACHING_SKILL_GAP_MIN)
			/ maxf(float(GameConfig.TEACHING_SKILL_GAP_OPTIMAL
				- GameConfig.TEACHING_SKILL_GAP_MIN), 1.0))
	elif skill_gap <= GameConfig.TEACHING_SKILL_GAP_MAX:
		gap_modifier = 1.0 - 0.3 * (
			float(skill_gap - GameConfig.TEACHING_SKILL_GAP_OPTIMAL)
			/ maxf(float(GameConfig.TEACHING_SKILL_GAP_MAX
				- GameConfig.TEACHING_SKILL_GAP_OPTIMAL), 1.0))
	else:
		gap_modifier = 0.5

	## Final effectiveness
	var effectiveness: float = (
		GameConfig.TEACHING_BASE_EFFECTIVENESS
		* teaching_ability
		* learning_receptivity
		* gap_modifier
	)

	return clampf(effectiveness, 0.001, 0.1)


## Compute teaching skill for an entity.
## Interpersonal × 0.30 + Linguistic × 0.20 + Agreeableness × 0.20 + Conscientiousness × 0.30
func _compute_teaching_skill(e: RefCounted) -> float:
	var interpersonal: float = float(e.intelligences.get("interpersonal", 0.5))
	var linguistic: float = float(e.intelligences.get("linguistic", 0.5))
	var agreeableness: float = 0.5
	var conscientiousness: float = 0.5
	if e.personality != null:
		agreeableness = float(e.personality.axes.get("A", 0.5))
		conscientiousness = float(e.personality.axes.get("C", 0.5))
	return (interpersonal * 0.30 + linguistic * 0.20
		+ agreeableness * 0.20 + conscientiousness * 0.30)


## ── Adoption Willingness [Rogers 2003] ──────────────────────────────────────

## Calculate how willing an agent is to learn new technology.
## High Openness + high KNOWLEDGE value → eager adopter.
## High TRADITION → resistant.
func _calculate_adoption_willingness(e: RefCounted) -> float:
	var openness: float = 0.5
	var conscientiousness: float = 0.5
	if e.personality != null:
		openness = float(e.personality.axes.get("O", 0.5))
		conscientiousness = float(e.personality.axes.get("C", 0.5))

	var knowledge_value: float = float(e.values.get(&"KNOWLEDGE", 0.0))
	var knowledge_norm: float = (knowledge_value + 1.0) / 2.0

	var score: float = (
		openness * GameConfig.ADOPTION_OPENNESS_WEIGHT
		+ openness * GameConfig.ADOPTION_CURIOSITY_WEIGHT
		+ conscientiousness * GameConfig.ADOPTION_CONSCIENTIOUSNESS_WEIGHT
		+ knowledge_norm * GameConfig.ADOPTION_KNOWLEDGE_VALUE_WEIGHT
	)

	## TRADITION modifier: high tradition → slower adoption
	var tradition: float = float(e.values.get(&"TRADITION", 0.0))
	if tradition > 0.3:
		score *= (1.0 - tradition * 0.3)

	## Stress modifier: high stress → not learning new things
	var stress: float = float(e.emotions.get("stress", 0.0))
	if stress > 0.6:
		score *= 0.5

	## Age modifier (age_stage based)
	match e.age_stage:
		"child":
			score *= 0.5
		"teen":
			score *= 0.9
		"adult":
			pass  ## 1.0x — peak adoption
		"elder":
			score *= 0.6

	return clampf(score, 0.0, 1.0)


## ── Adoption Phase Tracking [Rogers S-curve] ────────────────────────────────

## Update the adoption curve phase for each known tech in a settlement.
func _update_adoption_phase(settlement: RefCounted) -> void:
	var pop: int = settlement.member_ids.size()
	if pop == 0:
		return

	for tech_id in settlement.tech_states:
		var cts: Dictionary = settlement.tech_states[tech_id]
		var state_enum: int = CivTechState.get_state_enum(cts)
		if not TechState.is_known(state_enum):
			continue

		var practitioners: int = int(cts.get("practitioner_count", 0))
		var ratio: float = float(practitioners) / float(pop)
		var old_phase: String = cts.get("adoption_curve_phase", "innovator")
		var new_phase: String

		if ratio >= GameConfig.ADOPTION_LATE_MAJORITY_PCT:
			new_phase = "laggard"
		elif ratio >= GameConfig.ADOPTION_EARLY_MAJORITY_PCT:
			new_phase = "late_majority"
		elif ratio >= GameConfig.ADOPTION_EARLY_ADOPTER_PCT:
			new_phase = "early_majority"
		elif ratio >= GameConfig.ADOPTION_INNOVATOR_PCT:
			new_phase = "early_adopter"
		else:
			new_phase = "innovator"

		if new_phase != old_phase:
			cts["adoption_curve_phase"] = new_phase
			SimulationBus.tech_adoption_phase_changed.emit(
				settlement.id, tech_id, old_phase, new_phase)

		## Update propagation rate metric
		cts["propagation_rate"] = ratio


## ── XP Calculation ──────────────────────────────────────────────────────────

## Calculate skill XP gained per completed teaching cycle.
## Higher teacher mastery → more XP transferred.
func _calculate_xp_gain(session: Dictionary, teacher: RefCounted) -> float:
	var skill_sn = StringName(session["skill_id"])
	var teacher_level: int = int(teacher.skill_levels.get(skill_sn, 0))
	## Base 10 XP + 0.5 per teacher level
	return 10.0 + float(teacher_level) * 0.5


## ── Practitioner Tracking ───────────────────────────────────────────────────

## Update total_ever_learned when a student potentially becomes a practitioner.
func _update_practitioner_count(student: RefCounted, tech_id: String) -> void:
	var settlement: RefCounted = _settlement_manager.get_settlement(student.settlement_id)
	if settlement == null:
		return
	if not settlement.tech_states.has(tech_id):
		return

	var def: Dictionary = _tech_tree_manager.get_def(tech_id)
	var disc: Dictionary = def.get("discovery", {})
	if disc.is_empty():
		disc = def.get("discovery_conditions", {})
	var req_skills: Dictionary = disc.get("required_skills", {})

	## Check if student now meets all required skill levels
	var meets_all: bool = true
	for skill_id in req_skills:
		if int(student.skill_levels.get(StringName(skill_id), 0)) < int(req_skills[skill_id]):
			meets_all = false
			break

	if meets_all:
		var cts: Dictionary = settlement.tech_states[tech_id]
		cts["total_ever_learned"] = int(cts.get("total_ever_learned", 0)) + 1

		## Check if tech has enough practitioners for stability
		var maint: Dictionary = def.get("maintenance", {})
		var min_practitioners: int = int(maint.get("min_practitioners", 2))
		var practitioners: int = int(cts.get("practitioner_count", 0))
		if practitioners >= min_practitioners:
			SimulationBus.tech_reached_stable.emit(
				settlement.id, tech_id, practitioners)


## ── Helpers ─────────────────────────────────────────────────────────────────

## Count how many students a teacher currently has in active sessions.
func _count_students(teacher_id: int) -> int:
	var count: int = 0
	for session in _active_sessions:
		if session["teacher_id"] == teacher_id:
			count += 1
	return count


## Check if a student is already in an active learning session.
func _is_already_learning(student_id: int) -> bool:
	for session in _active_sessions:
		if session["student_id"] == student_id:
			return true
	return false


## Check if a specific teacher-student-tech-skill session already exists.
func _has_session(teacher_id: int, student_id: int, tech_id: String,
		skill_id: String) -> bool:
	for session in _active_sessions:
		if session["teacher_id"] == teacher_id \
				and session["student_id"] == student_id \
				and session["tech_id"] == tech_id \
				and session["skill_id"] == skill_id:
			return true
	return false


## Check if an apprenticeship bond should form (3+ completed sessions).
func _check_apprenticeship(teacher_id: int, student_id: int,
		tech_id: String) -> void:
	var student: RefCounted = _entity_manager.get_entity(student_id)
	if student == null:
		return
	var count: int = 0
	for record in student.apprenticeship_history:
		if int(record.get("teacher_id", -1)) == teacher_id:
			count += 1
	if count >= 3:
		SimulationBus.apprenticeship_formed.emit(teacher_id, student_id, tech_id)


## ═══════════════════════════════════════════════════════════════════════════
## Cross-Settlement Propagation [Diamond 1997, Boyd & Richerson 2005]
## ═══════════════════════════════════════════════════════════════════════════
## Public API: called by trade, migration, war, and diplomacy systems
## when inter-settlement contact occurs. Those systems do not exist yet;
## these methods are ready for future integration.

## Attempt to spread a technology from one settlement to another.
## event: {source_settlement_id: int, target_settlement_id: int,
##         tech_id: String, channel: String, carrier_id: int}
## channel: "trade" | "migration" | "war" | "diplomacy"
## Returns true if the tech was successfully imported.
func attempt_cross_propagation(event: Dictionary) -> bool:
	var source_id: int = int(event.get("source_settlement_id", -1))
	var target_id: int = int(event.get("target_settlement_id", -1))
	var tech_id: String = str(event.get("tech_id", ""))
	var channel: String = str(event.get("channel", ""))
	var carrier_id: int = int(event.get("carrier_id", -1))

	if source_id < 0 or target_id < 0 or tech_id.is_empty():
		return false

	var source: RefCounted = _settlement_manager.get_settlement(source_id)
	var target: RefCounted = _settlement_manager.get_settlement(target_id)
	if source == null or target == null:
		return false

	## Source must actually have the tech (known_low or known_stable)
	if not source.tech_states.has(tech_id):
		return false
	var source_cts: Dictionary = source.tech_states[tech_id]
	var source_state: int = CivTechState.get_state_enum(source_cts)
	if not TechState.is_known(source_state):
		return false

	## Target must NOT already have this tech in a known state
	if target.tech_states.has(tech_id):
		var target_cts: Dictionary = target.tech_states[tech_id]
		var target_state: int = CivTechState.get_state_enum(target_cts)
		if TechState.is_known(target_state):
			return false

	## Check prerequisites: target must have all hard prereqs
	var def: Dictionary = _tech_tree_manager.get_def(tech_id)
	var prereq: Dictionary = def.get("prereq_logic", {})
	for hard_prereq in prereq.get("hard", []):
		if not target.has_tech(hard_prereq):
			SimulationBus.tech_spread_blocked.emit(
				source_id, target_id, tech_id,
				"missing_prerequisite_" + str(hard_prereq))
			return false

	## Calculate base probability by channel
	var base_prob: float
	match channel:
		"trade":
			base_prob = GameConfig.CROSS_PROP_TRADE_BASE
		"migration":
			base_prob = GameConfig.CROSS_PROP_MIGRATION_BASE
		"war":
			base_prob = GameConfig.CROSS_PROP_WAR_CAPTURE_BASE
		"diplomacy":
			base_prob = GameConfig.CROSS_PROP_DIPLOMACY_BASE
		_:
			base_prob = 0.01

	## Language penalty (placeholder — language_divergence system not yet built)
	## When CultureManager exists, replace with actual divergence calculation.
	## For now, same-settlement-origin entities have 0 divergence.
	var lang_penalty: float = 1.0

	## Cultural modifier from target settlement's values
	var culture_mod: float = _calculate_culture_modifier(target)

	## Carrier skill bonus
	var carrier_bonus: float = 1.0
	if carrier_id >= 0:
		var carrier: RefCounted = _entity_manager.get_entity(carrier_id)
		if carrier != null:
			carrier_bonus = _calculate_carrier_bonus(carrier, tech_id)

	## Source tech stability bonus
	var stability_bonus: float = 1.0
	var source_state_str: String = source_cts.get("state", "known_low")
	if source_state_str == "known_stable":
		stability_bonus = 1.3
	elif source_state_str == "known_low":
		stability_bonus = 0.7

	## Final probability
	var bridge: Object = _get_sim_bridge()
	var final_prob: float = base_prob * lang_penalty * culture_mod \
		* carrier_bonus * stability_bonus
	if bridge != null:
		var final_variant: Variant = bridge.call(
			_SIM_BRIDGE_PROP_FINAL_PROB_METHOD,
			base_prob,
			lang_penalty,
			culture_mod,
			carrier_bonus,
			stability_bonus,
			0.95,
		)
		if final_variant != null:
			final_prob = float(final_variant)
	final_prob = clampf(final_prob, 0.0, 0.95)

	## Roll
	var success: bool = randf() < final_prob

	SimulationBus.tech_spread_attempt.emit(
		source_id, target_id, tech_id, channel, success)

	if success:
		_import_tech(target, tech_id, source_id, carrier_id, channel)
		return true

	return false


## Import a technology into a target settlement.
func _import_tech(target: RefCounted, tech_id: String,
		source_id: int, carrier_id: int, channel: String) -> void:
	var tick: int = 0
	if _chronicle != null:
		tick = _chronicle.get_current_tick() if _chronicle.has_method("get_current_tick") else 0

	## Create new CivTechState as known_low (fragile — needs practitioners)
	var cts: Dictionary = CivTechState.create_discovered(tech_id, tick, -1)
	cts["acquisition_method"] = channel
	cts["source_settlement_id"] = source_id
	cts["propagation_rate"] = 0.0
	cts["adoption_curve_phase"] = "innovator"
	cts["total_ever_learned"] = 1
	cts["cross_settlement_sources"] = [{
		"source_settlement_id": source_id,
		"channel": channel,
		"tick": tick,
	}]

	target.tech_states[tech_id] = cts

	## Update era
	_tech_tree_manager.update_era(target)

	SimulationBus.tech_imported.emit(
		target.id, tech_id, source_id, carrier_id, channel)

	if _chronicle != null:
		var toast_key: String
		match channel:
			"trade":
				toast_key = "TOAST_TECH_IMPORTED_TRADE"
			"migration":
				toast_key = "TOAST_TECH_IMPORTED_MIGRATION"
			"war":
				toast_key = "TOAST_TECH_IMPORTED_WAR"
			"diplomacy":
				toast_key = "TOAST_TECH_IMPORTED_DIPLOMACY"
			_:
				toast_key = "TOAST_TECH_IMPORTED_TRADE"

			_chronicle.log_event("tech_imported", carrier_id,
				"[Settlement %d] imported %s via %s from settlement %d" \
					% [target.id, tech_id, channel, source_id],
				4, [], tick,
				{"key": toast_key,
				"params": {"settlement": target.id,
					"tech": tech_id,
					"source": source_id,
					"carrier": carrier_id}})


## Calculate cultural modifier for the target settlement.
## High KNOWLEDGE/COMMERCE → receptive. High TRADITION → resistant.
func _calculate_culture_modifier(target: RefCounted) -> float:
	var knowledge_avg: float = _settlement_avg_value(target, &"KNOWLEDGE")
	var tradition_avg: float = _settlement_avg_value(target, &"TRADITION")
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_PROP_CULTURE_METHOD,
			knowledge_avg,
			tradition_avg,
			0.3,
			0.4,
			0.1,
			2.0,
		)
		if rust_variant != null:
			return float(rust_variant)

	var culture_mod: float = 1.0
	culture_mod += (knowledge_avg + 1.0) / 2.0 * 0.3
	culture_mod -= (tradition_avg + 1.0) / 2.0 * 0.4
	return clampf(culture_mod, 0.1, 2.0)


## Calculate carrier skill bonus for cross-settlement propagation.
func _calculate_carrier_bonus(carrier: RefCounted, tech_id: String) -> float:
	var def: Dictionary = _tech_tree_manager.get_def(tech_id)
	var disc: Dictionary = def.get("discovery", {})
	if disc.is_empty():
		disc = def.get("discovery_conditions", {})
	var req_skills: Dictionary = disc.get("required_skills", {})

	var max_skill: int = 0
	for skill_id in req_skills:
		var level: int = int(carrier.skill_levels.get(StringName(skill_id), 0))
		max_skill = maxi(max_skill, level)
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var rust_variant: Variant = bridge.call(
			_SIM_BRIDGE_PROP_CARRIER_METHOD,
			max_skill,
			20.0,
			0.5,
		)
		if rust_variant != null:
			return float(rust_variant)
	return 1.0 + float(max_skill) / 20.0 * 0.5


## Average value across all alive members of a settlement.
func _settlement_avg_value(settlement: RefCounted,
		value_key: StringName) -> float:
	var total: float = 0.0
	var count: int = 0
	for mid in settlement.member_ids:
		var e: RefCounted = _entity_manager.get_entity(mid)
		if e == null or not e.is_alive:
			continue
		total += float(e.values.get(value_key, 0.0))
		count += 1
	return total / maxf(float(count), 1.0)
