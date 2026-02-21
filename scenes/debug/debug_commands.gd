extends RefCounted

# NO class_name — headless compatibility
# Phase 4 debug command module loaded by debug_console.gd

var _entity_manager: Variant = null
var _coping_system: Variant = null
var _morale_system: Variant = null
var _contagion_system: Variant = null
var _sim_engine: Variant = null
var _console: Variant = null
var _settlement_manager: Variant = null
var _child_stress_processor: Variant = null
var _intergenerational_system: Variant = null
var _parenting_system: Variant = null
var _behavior_system: Variant = null
const LOG_PATH: String = "user://debug_phase4.log"
var _log_file: FileAccess = null


func init(entity_manager, coping_sys, morale_sys, contagion_sys, sim_engine, console, settlement_mgr) -> void:
	_entity_manager = entity_manager
	_coping_system = coping_sys
	_morale_system = morale_sys
	_contagion_system = contagion_sys
	_sim_engine = sim_engine
	_console = console
	_settlement_manager = settlement_mgr
	_log_file = FileAccess.open(LOG_PATH, FileAccess.WRITE)


func init_phase5(child_stress_processor, intergenerational_system, parenting_system) -> void:
	_child_stress_processor = child_stress_processor
	_intergenerational_system = intergenerational_system
	_parenting_system = parenting_system


func init_behavior(behavior_system) -> void:
	_behavior_system = behavior_system


func _print(text: String, color: Color = Color.WHITE) -> void:
	if _console != null:
		_console.print_output(text, color)
	else:
		print("[P4Debug] " + text)
	if _log_file != null:
		_log_file.store_line(text)
		_log_file.flush()


func _header(label: String) -> void:
	_print("── " + label + " ──", Color(0.6, 0.9, 1.0))


func _get_entity(id_str: String):
	if _entity_manager == null:
		_print("entity_manager not connected", Color(1.0, 0.4, 0.4))
		return null
	var id_int: int = id_str.to_int()
	var entity: Variant = _entity_manager.get_entity(id_int)
	if entity == null:
		_print("Entity not found: " + id_str, Color(1.0, 0.4, 0.4))
	return entity


func _pos_arg(args: Dictionary, index: int) -> String:
	var pos: Variant = args.get("_pos", [])
	if pos is Array and pos.size() > index:
		return str(pos[index])
	return ""


func cmd_debug_coping(args: Dictionary) -> void:
	var id_str: String = _pos_arg(args, 0)
	if id_str.is_empty():
		_print("Usage: debug_coping <agent_id>", Color(1.0, 0.4, 0.4))
		return
	var entity: Variant = _get_entity(id_str)
	if entity == null:
		return
	if _coping_system == null:
		_print("coping_system not connected", Color(1.0, 0.4, 0.4))
		return

	_header("debug_coping: %s (id:%d)" % [entity.entity_name, entity.id])

	var cs: Dictionary = _coping_system._entity_coping.get(entity.id, {})
	if cs.is_empty():
		_print("  (no coping state — entity has not experienced a break)")
		return

	var owned: Dictionary = cs.get("owned", {})
	_print("  Coping strategies owned: %d" % owned.size())
	if owned.is_empty():
		_print("    (none)")
	else:
		for cid in owned:
			var entry: Dictionary = owned.get(cid, {})
			var prof: float = float(entry.get("proficiency", 0.0))
			var cooldown: float = float(entry.get("cooldown", 0.0))
			var cooldown_color: Color = Color(1.0, 0.7, 0.3) if cooldown > 0.0 else Color(0.6, 1.0, 0.6)
			_print("    [%s] proficiency=%.2f  cooldown=%.0f ticks" % [cid, prof, cooldown], cooldown_color)

	_print("  break_count=%d  last_break_type=%s" % [
		int(cs.get("break_count", 0)),
		str(cs.get("last_break_type", ""))
	])
	_print("  denial_accumulator=%.3f  denial_timer=%d ticks" % [
		float(cs.get("denial_accumulator", 0.0)),
		int(cs.get("denial_timer", 0))
	])
	_print("  dependency_score=%.3f  helplessness_score=%.3f" % [
		float(cs.get("dependency_score", 0.0)),
		float(cs.get("helplessness_score", 0.0))
	])
	_print("  control_appraisal_cap=%.2f" % float(cs.get("control_appraisal_cap", 1.0)))
	var rebound_queue: Array = cs.get("rebound_queue", [])
	_print("  rebound_queue=%d pending" % rebound_queue.size())


func cmd_debug_morale(args: Dictionary) -> void:
	var id_str: String = _pos_arg(args, 0)
	if id_str.is_empty():
		_print("Usage: debug_morale <agent_id>", Color(1.0, 0.4, 0.4))
		return
	var entity: Variant = _get_entity(id_str)
	if entity == null:
		return
	if _morale_system == null:
		_print("morale_system not connected", Color(1.0, 0.4, 0.4))
		return

	_header("debug_morale: %s (id:%d)" % [entity.entity_name, entity.id])

	var morale_personal: float = float(_morale_system._personal_morale.get(entity.id, 0.5))
	var bwm: float = float(_morale_system.get_behavior_weight_multiplier(entity.id))
	var expectation_base: float = float(_morale_system._expectation_base.get(entity.id, 0.5))

	var maslow_state: String = "OK"
	if entity.hunger < 0.3 or entity.energy < 0.3:
		maslow_state = "CRITICAL (physiology blocked)"
	elif entity.hunger < 0.6 or entity.energy < 0.6:
		maslow_state = "LOW (physiology impaired)"
	elif entity.settlement_id < 0:
		maslow_state = "UNSAFE (no settlement)"
	elif entity.social < 0.3:
		maslow_state = "ISOLATED (belonging low)"

	var grievance: float = 0.0
	if entity.settlement_id >= 0:
		grievance = float(_morale_system._grievance.get(entity.settlement_id, 0.0))

	var morale_color: Color = Color(0.6, 1.0, 0.6)
	if morale_personal < 0.3:
		morale_color = Color(1.0, 0.4, 0.4)
	elif morale_personal < 0.6:
		morale_color = Color(1.0, 0.9, 0.3)

	_print("  morale_personal=%.3f" % morale_personal, morale_color)
	_print("  behavior_weight_multiplier=%.3f" % bwm)
	_print("  expectation_base=%.3f  (hedonic treadmill)" % expectation_base)
	_print("  hunger=%.3f  energy=%.3f  social=%.3f" % [entity.hunger, entity.energy, entity.social])
	_print("  Maslow state: %s" % maslow_state)
	_print("  settlement_id=%d  grievance=%.3f" % [entity.settlement_id, grievance])
	if grievance > 0.35:
		_print("  ⚠ Grievance above rebellion threshold (0.35)!", Color(1.0, 0.7, 0.3))


func cmd_debug_contagion(args: Dictionary) -> void:
	var sid_str: String = _pos_arg(args, 0)
	if sid_str.is_empty():
		_print("Usage: debug_contagion <settlement_id>", Color(1.0, 0.4, 0.4))
		return
	var sid: int = sid_str.to_int()

	if _entity_manager == null:
		_print("entity_manager not connected", Color(1.0, 0.4, 0.4))
		return

	_header("debug_contagion: settlement %d" % sid)

	var alive: Array = _entity_manager.get_alive_entities()
	var members: Array = []
	for i in range(alive.size()):
		if alive[i].settlement_id == sid:
			members.append(alive[i])

	if members.is_empty():
		_print("  No entities found in settlement %d" % sid, Color(1.0, 0.7, 0.3))
		return

	_print("  Total entities in settlement: %d" % members.size())

	var refractory_count: int = 0
	var high_anger_count: int = 0

	for entity in members:
		if entity.emotion_data == null:
			continue
		var ed: Variant = entity.emotion_data
		var refractory: int = int(ed.get_meta("contagion_refractory", 0))
		var anger: float = 0.0
		if ed.has_method("get_emotion"):
			anger = float(ed.get_emotion("anger"))
		else:
			anger = float(ed.fast.get("anger", 0.0))
		var stress: float = float(ed.stress)

		if refractory > 0:
			refractory_count += 1
		if anger > 50.0:
			high_anger_count += 1

		var line: String = "  %s anger=%.1f stress=%.1f refrac=%d" % [
			entity.entity_name, anger, stress, refractory
		]
		var line_color: Color = Color.WHITE
		if refractory > 0:
			line_color = Color(1.0, 0.9, 0.3)
		if anger > 70.0:
			line_color = Color(1.0, 0.5, 0.3)
		_print(line, line_color)

	_print("  ── Summary ──", Color(0.6, 0.9, 1.0))
	_print("  In refractory: %d / %d" % [refractory_count, members.size()])
	_print("  High anger (>50): %d / %d" % [high_anger_count, members.size()])
	_print("  (Contagion spiral events logged to ChronicleSystem autoload)")


func cmd_test_coping_acquire(args: Dictionary) -> void:
	var id_str: String = _pos_arg(args, 0)
	var break_type: String = _pos_arg(args, 1)
	if id_str.is_empty():
		_print("Usage: test_coping_acquire <agent_id> <break_type>", Color(1.0, 0.4, 0.4))
		return
	if break_type.is_empty():
		break_type = "rage"

	var entity: Variant = _get_entity(id_str)
	if entity == null:
		return
	if _coping_system == null:
		_print("coping_system not connected", Color(1.0, 0.4, 0.4))
		return

	_header("test_coping_acquire: %s break_type=%s (10x)" % [entity.entity_name, break_type])

	var tick: int = _sim_engine.current_tick if _sim_engine != null else 0

	if not _coping_system._entity_coping.has(entity.id):
		_coping_system._entity_coping[entity.id] = {
			"owned": {}, "denial_accumulator": 0.0, "denial_timer": 0,
			"dependency_score": 0.0, "helplessness_score": 0.0,
			"control_appraisal_cap": 1.0, "rebound_queue": [],
			"cooldowns": {}, "break_count": 0, "last_break_type": "",
			"substance_recent_timer": 0,
		}

	var state: Dictionary = _coping_system._entity_coping[entity.id]
	var before_owned: Array = state.get("owned", {}).keys().duplicate()
	_print("  Before: %d copings owned" % before_owned.size())

	var new_count: int = 0
	var upgrade_count: int = 0
	var none_count: int = 0

	for i in range(10):
		state["break_count"] = int(state.get("break_count", 0)) + 1
		var owned_before_size: int = state.get("owned", {}).size()
		var result: int = int(_coping_system.attempt_acquire_on_break_recovery(entity, break_type, tick + i))
		if result > 0:
			var owned_after_size: int = state.get("owned", {}).size()
			if owned_after_size > owned_before_size:
				new_count += 1
			else:
				upgrade_count += 1
		else:
			none_count += 1

	_print("  Results over 10 trials:")
	_print("    New acquisition: %d/10" % new_count, Color(0.6, 1.0, 0.6))
	_print("    Upgrade existing: %d/10" % upgrade_count, Color(0.8, 0.9, 1.0))
	_print("    No acquisition: %d/10" % none_count, Color(0.7, 0.7, 0.7))

	var final_owned: Dictionary = state.get("owned", {})
	_print("  Final owned (%d copings):" % final_owned.size())
	for cid in final_owned:
		var entry: Dictionary = final_owned.get(cid, {})
		var newly_acquired: bool = not before_owned.has(cid)
		var marker: String = " [NEW]" if newly_acquired else ""
		var entry_color: Color = Color(0.6, 1.0, 0.6) if newly_acquired else Color.WHITE
		_print(
			"    %s%s  proficiency=%.2f" % [cid, marker, float(entry.get("proficiency", 0.0))],
			entry_color
		)

	_print("  (Inflation check: %d new total — expected <3 in 10 trials without high stress)" % new_count)


func cmd_test_rage_spread(args: Dictionary) -> void:
	var id_str: String = _pos_arg(args, 0)
	if id_str.is_empty():
		_print("Usage: test_rage_spread <agent_id>", Color(1.0, 0.4, 0.4))
		return
	var entity: Variant = _get_entity(id_str)
	if entity == null:
		return
	if entity.emotion_data == null:
		_print("Entity has no emotion_data", Color(1.0, 0.4, 0.4))
		return

	_header("test_rage_spread: %s" % entity.entity_name)

	entity.emotion_data.fast["anger"] = 100.0
	entity.emotion_data.set_meta("contagion_refractory", 0)
	_print("  Set %s anger → 100.0, refractory reset" % entity.entity_name, Color(1.0, 0.5, 0.3))

	if _entity_manager == null:
		_print("entity_manager not connected", Color(1.0, 0.4, 0.4))
		return

	var alive: Array = _entity_manager.get_alive_entities()
	const AOE_RADIUS: int = 3
	const NETWORK_HOP_RADIUS: int = 15
	const BASE_MIMICRY_WEIGHT: float = 0.08
	const MAX_EMOTION_CONTAGION_DELTA: float = 8.0
	const REFRACTORY_SUSCEPTIBILITY: float = 0.25

	var aoe_targets: int = 0
	var net_targets: int = 0

	_print("  ── AoE spread (radius %d tiles) ──" % AOE_RADIUS, Color(0.6, 0.9, 1.0))

	for i in range(alive.size()):
		var recipient: Variant = alive[i]
		if recipient.id == entity.id:
			continue
		if recipient.emotion_data == null:
			continue

		var dist: int = int(abs(recipient.position.x - entity.position.x) + abs(recipient.position.y - entity.position.y))
		if dist <= AOE_RADIUS:
			aoe_targets += 1
			var r_ed: Variant = recipient.emotion_data
			var current_anger: float = 0.0
			if r_ed.fast.has("anger"):
				current_anger = float(r_ed.fast.get("anger", 0.0))
			var gap: float = 100.0 - current_anger

			var X_axis: float = 0.5
			var E_axis: float = 0.5
			if recipient.personality != null:
				X_axis = float(recipient.personality.axes.get("X", 0.5))
				E_axis = float(recipient.personality.axes.get("E", 0.5))

			var refractory: int = int(r_ed.get_meta("contagion_refractory", 0))
			var susceptibility: float = REFRACTORY_SUSCEPTIBILITY if refractory > 0 else 1.0
			var personality_susceptibility: float = 0.7 + 0.3 * X_axis + 0.2 * (E_axis - 0.5)
			var total_susceptibility: float = susceptibility * personality_susceptibility
			var delta: float = clampf(gap * BASE_MIMICRY_WEIGHT * total_susceptibility, -MAX_EMOTION_CONTAGION_DELTA, MAX_EMOTION_CONTAGION_DELTA)

			var intensity_color: Color = Color.WHITE
			if delta > 5.0:
				intensity_color = Color(1.0, 0.5, 0.3)
			elif delta > 2.0:
				intensity_color = Color(1.0, 0.9, 0.3)

			_print("  [AoE d=%d] %s  anger:%.1f→+%.1f  suscept=%.2f  refrac=%d" % [
				dist, recipient.entity_name, current_anger, delta, total_susceptibility, refractory
			], intensity_color)

	_print("  ── Network spread (radius %d, same settlement) ──" % NETWORK_HOP_RADIUS, Color(0.6, 0.9, 1.0))

	for i in range(alive.size()):
		var recipient: Variant = alive[i]
		if recipient.id == entity.id:
			continue
		if recipient.settlement_id != entity.settlement_id:
			continue
		var dist: int = int(abs(recipient.position.x - entity.position.x) + abs(recipient.position.y - entity.position.y))
		if dist > AOE_RADIUS and dist <= NETWORK_HOP_RADIUS:
			net_targets += 1
			var refractory: int = 0
			if recipient.emotion_data != null:
				refractory = int(recipient.emotion_data.get_meta("contagion_refractory", 0))
			_print("  [Net d=%d] %s  refrac=%d  settle=%d" % [
				dist, recipient.entity_name, refractory, recipient.settlement_id
			])

	_print("  ── Summary ──", Color(0.6, 0.9, 1.0))
	_print("  AoE targets (d≤%d): %d" % [AOE_RADIUS, aoe_targets])
	_print("  Network targets (d≤%d, same settle): %d" % [NETWORK_HOP_RADIUS, net_targets])
	_print("  (Actual spread occurs on next contagion tick — tick_interval=3)")


func cmd_test_migration(args: Dictionary) -> void:
	var sid_str: String = _pos_arg(args, 0)
	if sid_str.is_empty():
		_print("Usage: test_migration <settlement_id>", Color(1.0, 0.4, 0.4))
		return
	var sid: int = sid_str.to_int()

	if _entity_manager == null or _morale_system == null:
		_print("entity_manager or morale_system not connected", Color(1.0, 0.4, 0.4))
		return

	_header("test_migration: settlement %d" % sid)

	var alive: Array = _entity_manager.get_alive_entities()
	var members: Array = []
	for i in range(alive.size()):
		if alive[i].settlement_id == sid:
			members.append(alive[i])

	if members.is_empty():
		_print("  No entities in settlement %d" % sid, Color(1.0, 0.7, 0.3))
		return

	var settlement_morale_true: float = float(_morale_system.get_settlement_morale_true(sid))
	_print("  Settlement morale_true: %.3f" % settlement_morale_true)
	_print("  Member count: %d" % members.size())

	var total_prob: float = 0.0
	var likely_migrants: int = 0

	for entity in members:
		var prob: float = float(_morale_system.get_migration_probability(entity.id))
		total_prob += prob
		var prob_color: Color = Color.WHITE
		if prob > 0.5:
			prob_color = Color(1.0, 0.4, 0.4)
			likely_migrants += 1
		elif prob > 0.2:
			prob_color = Color(1.0, 0.9, 0.3)
		_print("  %s (id:%d): p_migrate=%.3f" % [entity.entity_name, entity.id, prob], prob_color)

	var avg_prob: float = total_prob / float(members.size()) if members.size() > 0 else 0.0
	_print("  Average migration probability: %.3f" % avg_prob)
	_print("  Likely migrants (p>0.5): %d" % likely_migrants)

	var mcfg: Dictionary = _morale_system._cfg.get("migration", {})
	var k: float = float(mcfg.get("k", 10.0))
	var m0: float = float(mcfg.get("threshold_morale", 0.35))
	_print("  ── Simulated (k=%.1f, threshold=%.2f) ──" % [k, m0], Color(0.6, 0.9, 1.0))
	for test_morale in [0.2, 0.5, 0.8]:
		var test_value: float = float(test_morale)
		var p: float = 1.0 / (1.0 + exp(-k * (m0 - test_value)))
		var sim_color: Color = Color.WHITE
		if test_value == 0.2:
			sim_color = Color(1.0, 0.4, 0.4)
		elif test_value == 0.5:
			sim_color = Color(1.0, 0.9, 0.3)
		else:
			sim_color = Color(0.6, 1.0, 0.6)
		_print("  [Sim] morale=%.1f → p_migrate=%.3f" % [test_value, p], sim_color)


func cmd_test_rebellion(args: Dictionary) -> void:
	var sid_str: String = _pos_arg(args, 0)
	if sid_str.is_empty():
		_print("Usage: test_rebellion <settlement_id>", Color(1.0, 0.4, 0.4))
		return
	var sid: int = sid_str.to_int()

	if _morale_system == null:
		_print("morale_system not connected", Color(1.0, 0.4, 0.4))
		return

	_header("test_rebellion: settlement %d" % sid)

	var tick: int = _sim_engine.current_tick if _sim_engine != null else 0
	var grievance: float = float(_morale_system._grievance.get(sid, 0.0))
	var morale_true: float = float(_morale_system.get_settlement_morale_true(sid))
	var rebellion_p: float = float(_morale_system.check_rebellion_probability(sid, tick))

	var g_color: Color = Color.WHITE
	if grievance > 0.5:
		g_color = Color(1.0, 0.4, 0.4)
	elif grievance > 0.35:
		g_color = Color(1.0, 0.7, 0.3)

	var r_color: Color = Color(0.6, 1.0, 0.6)
	if rebellion_p > 0.05:
		r_color = Color(1.0, 0.4, 0.4)
	elif rebellion_p > 0.02:
		r_color = Color(1.0, 0.7, 0.3)

	_print("  morale_true: %.3f" % morale_true)
	_print("  grievance: %.3f" % grievance, g_color)
	_print("  rebellion_probability: %.4f (%.2f%%)" % [rebellion_p, rebellion_p * 100.0], r_color)

	if grievance > 0.35:
		_print("  ⚠ Grievance above rebellion trigger threshold!", Color(1.0, 0.7, 0.3))
	if rebellion_p > 0.05:
		_print("  🔴 HIGH REBELLION RISK!", Color(1.0, 0.4, 0.4))
	elif rebellion_p < 0.01:
		_print("  ✓ Stable (rebellion probability < 1%)", Color(0.6, 1.0, 0.6))


func cmd_debug_childhood(args: Dictionary) -> void:
	var id_str: String = _pos_arg(args, 0)
	if id_str.is_empty():
		_print("Usage: debug_childhood <agent_id>", Color(1.0, 0.4, 0.4))
		return
	var entity: Variant = _get_entity(id_str)
	if entity == null:
		return

	_header("debug_childhood: %s (id:%d)" % [entity.entity_name, entity.id])

	var age_years: float = float(entity.age) / 8760.0
	var dev_stage: String = "N/A"
	if _child_stress_processor != null:
		dev_stage = str(_child_stress_processor.get_current_stage(int(entity.age)))

	var ace_tracker: Variant = entity.get_meta("ace_tracker", null)
	var ace_score_total: float = float(entity.get_meta("ace_score_total", 0.0))
	var threat_score: float = 0.0
	var deprivation_score: float = 0.0
	if ace_tracker != null:
		ace_score_total = float(ace_tracker.ace_score_total)
		if ace_tracker.has_method("get_threat_deprivation_scores"):
			var td_scores: Dictionary = ace_tracker.get_threat_deprivation_scores()
			threat_score = float(td_scores.get("threat", 0.0))
			deprivation_score = float(td_scores.get("deprivation", 0.0))

	var attachment_type: String = str(entity.get_meta("attachment_type", "secure"))
	var attachment_color: Color = Color.WHITE
	if attachment_type == "disorganized":
		attachment_color = Color(1.0, 0.4, 0.4)
	elif attachment_type == "anxious" or attachment_type == "avoidant":
		attachment_color = Color(1.0, 0.9, 0.3)
	elif attachment_type == "secure":
		attachment_color = Color(0.6, 1.0, 0.6)

	var epigenetic_load: float = float(entity.get_meta("epigenetic_load_effective", 0.05))
	var hpa_sensitivity: float = 1.0 + epigenetic_load * 0.6
	var developmental_damage: float = 0.0
	var shrp_breached: bool = false
	if entity.emotion_data != null:
		developmental_damage = float(entity.emotion_data.get_meta("developmental_damage", 0.0))
		shrp_breached = bool(entity.emotion_data.get_meta("shrp_breached", false))

	var parenting_quality: float = float(entity.get_meta("parenting_quality", 0.5))
	var adulthood_applied: bool = bool(entity.get_meta("adulthood_applied", false))
	var ace_stress_gain_mult: float = float(entity.get_meta("ace_stress_gain_mult", 1.0))
	var ace_break_threshold_mult: float = float(entity.get_meta("ace_break_threshold_mult", 1.0))

	var stress_mult_color: Color = Color(0.6, 1.0, 0.6)
	if ace_stress_gain_mult > 1.5:
		stress_mult_color = Color(1.0, 0.4, 0.4)
	elif ace_stress_gain_mult > 1.2:
		stress_mult_color = Color(1.0, 0.9, 0.3)

	_print("  age_stage: %s      age: %.2f yrs (%d ticks)" % [str(entity.age_stage), age_years, int(entity.age)])
	_print("  dev_stage: %s      (from child_stress_processor)" % dev_stage)
	_print("  ace_score_total: %.2f" % ace_score_total)
	_print("  threat_score: %.2f   deprivation_score: %.2f" % [threat_score, deprivation_score])
	_print("  attachment_type: %s" % attachment_type, attachment_color)
	_print("  epigenetic_load: %.2f   hpa_sensitivity: %.2f" % [epigenetic_load, hpa_sensitivity])
	_print("  developmental_damage: %.2f" % developmental_damage)
	_print("  shrp_breached: %s" % str(shrp_breached).to_lower())
	_print("  parenting_quality: %.2f" % parenting_quality)
	_print("  adulthood_applied: %s" % str(adulthood_applied).to_lower())
	_print(
		"  ace_stress_gain_mult: %.2f   ace_break_threshold_mult: %.2f" % [
			ace_stress_gain_mult, ace_break_threshold_mult
		],
		stress_mult_color
	)


func cmd_debug_ace(args: Dictionary) -> void:
	var id_str: String = _pos_arg(args, 0)
	if id_str.is_empty():
		_print("Usage: debug_ace <agent_id>", Color(1.0, 0.4, 0.4))
		return
	var entity: Variant = _get_entity(id_str)
	if entity == null:
		return

	_header("debug_ace: %s (id:%d)" % [entity.entity_name, entity.id])

	var ace_tracker: Variant = entity.get_meta("ace_tracker", null)
	var ace_score_total: float = float(entity.get_meta("ace_score_total", 0.0))
	var backfilled: bool = false
	if ace_tracker != null:
		ace_score_total = float(ace_tracker.ace_score_total)
		backfilled = bool(ace_tracker.is_backfilled)

	var ace_score_color: Color = Color(0.6, 1.0, 0.6)
	if ace_score_total >= 4.0:
		ace_score_color = Color(1.0, 0.4, 0.4)
	elif ace_score_total >= 2.0:
		ace_score_color = Color(1.0, 0.9, 0.3)

	_print(
		"  ace_score_total: %.2f   backfilled: %s" % [ace_score_total, str(backfilled).to_lower()],
		ace_score_color
	)
	if ace_tracker == null:
		_print("  (no ace_tracker — using ace_score_total from meta only)", Color(1.0, 0.9, 0.3))

	var ace_item_ids: Array = [
		"physical_abuse",
		"emotional_abuse",
		"sexual_abuse",
		"physical_neglect",
		"emotional_neglect",
		"domestic_violence",
		"substance_household",
		"mental_illness_household",
		"parental_separation",
		"incarceration",
	]
	var item_scores: Dictionary = {}

	_print("  ── ACE Items ──", Color(0.6, 0.9, 1.0))
	for item_id in ace_item_ids:
		var item_score: float = 0.0
		if ace_tracker != null:
			var raw_item: Variant = ace_tracker.ace_items.get(item_id, 0.0)
			if raw_item is Dictionary:
				item_score = float(raw_item.get("partial_score", 0.0))
			else:
				item_score = float(raw_item)
		item_scores[item_id] = item_score
		_print("  %-25s %.2f" % [item_id + ":", item_score])

	var threat_score: float = 0.0
	var deprivation_score: float = 0.0
	if ace_tracker != null and ace_tracker.has_method("get_threat_deprivation_scores"):
		var td_scores: Dictionary = ace_tracker.get_threat_deprivation_scores()
		threat_score = float(td_scores.get("threat", 0.0))
		deprivation_score = float(td_scores.get("deprivation", 0.0))
	else:
		threat_score = float(item_scores.get("physical_abuse", 0.0))
		threat_score += float(item_scores.get("sexual_abuse", 0.0))
		threat_score += float(item_scores.get("domestic_violence", 0.0))
		deprivation_score = maxf(0.0, ace_score_total - threat_score)

	_print("  ── Threat/Deprivation ──", Color(0.6, 0.9, 1.0))
	_print("  threat_score:      %.2f  (physical_abuse, domestic_violence, sexual_abuse)" % threat_score)
	_print("  deprivation_score: %.2f  (others)" % deprivation_score)

	var mods: Dictionary = {}
	if ace_tracker != null and ace_tracker.has_method("calculate_adult_modifiers"):
		mods = ace_tracker.calculate_adult_modifiers()
	else:
		var ace_int: int = int(clampf(ace_score_total, 0.0, 10.0))
		var table: Array = [
			{"stress_gain_mult": 1.00, "break_threshold_mult": 1.00, "allostatic_base": 0.0},
			{"stress_gain_mult": 1.06, "break_threshold_mult": 0.98, "allostatic_base": 3.0},
			{"stress_gain_mult": 1.12, "break_threshold_mult": 0.95, "allostatic_base": 6.0},
			{"stress_gain_mult": 1.18, "break_threshold_mult": 0.92, "allostatic_base": 9.0},
			{"stress_gain_mult": 1.24, "break_threshold_mult": 0.90, "allostatic_base": 12.0},
			{"stress_gain_mult": 1.40, "break_threshold_mult": 0.84, "allostatic_base": 19.0},
			{"stress_gain_mult": 1.56, "break_threshold_mult": 0.79, "allostatic_base": 26.0},
			{"stress_gain_mult": 1.72, "break_threshold_mult": 0.74, "allostatic_base": 33.0},
			{"stress_gain_mult": 1.94, "break_threshold_mult": 0.66, "allostatic_base": 43.0},
			{"stress_gain_mult": 2.16, "break_threshold_mult": 0.58, "allostatic_base": 53.0},
			{"stress_gain_mult": 2.38, "break_threshold_mult": 0.51, "allostatic_base": 63.0},
		]
		if ace_int >= 0 and ace_int < table.size():
			mods = table[ace_int]

	_print("  ── Adult Modifiers (preview) ──", Color(0.6, 0.9, 1.0))
	_print("  stress_gain_mult:       %.2f" % float(mods.get("stress_gain_mult", 1.0)))
	_print("  break_threshold_mult:   %.2f" % float(mods.get("break_threshold_mult", 1.0)))
	_print("  allostatic_base:       %.2f" % float(mods.get("allostatic_base", 0.0)))


func cmd_debug_generation(args: Dictionary) -> void:
	var sid_str: String = _pos_arg(args, 0)
	if sid_str.is_empty():
		_print("Usage: debug_generation <settlement_id>", Color(1.0, 0.4, 0.4))
		return
	if _entity_manager == null:
		_print("entity_manager not connected", Color(1.0, 0.4, 0.4))
		return
	var sid: int = sid_str.to_int()

	_header("debug_generation: settlement %d" % sid)

	var alive: Array = _entity_manager.get_alive_entities()
	var members: Array = []
	for i in range(alive.size()):
		if alive[i].settlement_id == sid:
			members.append(alive[i])

	if members.is_empty():
		_print("  No entities found in settlement %d" % sid, Color(1.0, 0.7, 0.3))
		return

	var epigenetic_sum: float = 0.0
	var parenting_sum: float = 0.0
	var ace_high_count: int = 0
	var stress_sum: float = 0.0

	for member in members:
		var epi_load: float = float(member.get_meta("epigenetic_load_effective", 0.05))
		var parenting_quality: float = float(member.get_meta("parenting_quality", 0.5))
		var ace_score: float = float(member.get_meta("ace_score_total", 0.0))
		epigenetic_sum += epi_load
		parenting_sum += parenting_quality
		if ace_score > 4.0:
			ace_high_count += 1
		if member.emotion_data != null:
			stress_sum += float(member.emotion_data.stress)

	var member_count: int = members.size()
	var avg_epigenetic_load: float = epigenetic_sum / float(member_count)
	var avg_parenting_quality: float = parenting_sum / float(member_count)
	var ace_high_ratio: float = float(ace_high_count) / float(member_count)
	var adversity: float = clampf((stress_sum / float(member_count)) / 2000.0, 0.0, 1.0)

	var current_t: float = 0.30
	if _intergenerational_system != null:
		current_t = float(_intergenerational_system._current_T)

	var denominator: float = maxf(0.01, 1.0 - current_t)
	var e_star: float = (0.05 + 0.10) / denominator

	var status: String = "SAFE"
	var status_detail: String = "(E* < 0.30)"
	if e_star >= 0.50:
		status = "HIGH-LOAD"
		status_detail = "(E* >= 0.50)"
	elif e_star >= 0.30:
		status = "ELEVATED"
		status_detail = "(0.30 <= E* < 0.50)"

	var e_color: Color = Color(0.6, 1.0, 0.6)
	if e_star >= 0.50:
		e_color = Color(1.0, 0.4, 0.4)
	elif e_star >= 0.30:
		e_color = Color(1.0, 0.9, 0.3)

	var collapse_high: bool = adversity > 0.85 and avg_parenting_quality < 0.30 and ace_high_ratio > 0.35
	var collapse_text: String = "HIGH" if collapse_high else "LOW"
	var collapse_color: Color = Color(1.0, 0.4, 0.4) if collapse_high else Color(0.6, 1.0, 0.6)

	_print("  members: %d" % member_count)
	_print(
		"  avg_epigenetic_load: %.2f   avg_parenting_quality: %.2f" % [
			avg_epigenetic_load, avg_parenting_quality
		]
	)
	_print("  ace_high_ratio (score>4): %.2f  (%d/%d)" % [ace_high_ratio, ace_high_count, member_count])
	_print("  current_T: %.2f" % current_t)
	_print("  E* fixed point: %.2f  (baseline=0.05 + prenatal=0.10) / (1 - %.2f)" % [e_star, current_t], e_color)
	_print("  E* status: %s %s" % [status, status_detail], e_color)
	_print("  collapse_risk: %s" % collapse_text, collapse_color)


func cmd_test_shrp(args: Dictionary) -> void:
	var id_str: String = _pos_arg(args, 0)
	if id_str.is_empty():
		_print("Usage: test_shrp <agent_id>", Color(1.0, 0.4, 0.4))
		return
	if _child_stress_processor == null:
		_print("child_stress_processor not connected", Color(1.0, 0.4, 0.4))
		return
	var entity: Variant = _get_entity(id_str)
	if entity == null:
		return
	if entity.emotion_data == null:
		_print("Entity has no emotion_data", Color(1.0, 0.4, 0.4))
		return

	_header("test_shrp: %s (id:%d)" % [entity.entity_name, entity.id])

	var tick: int = _sim_engine.current_tick if _sim_engine != null else 0
	var dev_stage: String = str(_child_stress_processor.get_current_stage(int(entity.age)))
	var shrp_note: String = "SHRP active" if dev_stage == "infant" else "SHRP may be inactive"
	_print("  age_stage: %s   dev_stage: %s   (%s)" % [str(entity.age_stage), dev_stage, shrp_note])
	_print("  NOTE: SHRP only active for infants (shrp_active=true in developmental_stages.json)")
	if dev_stage != "infant":
		_print("  ⚠ Entity is %s (not infant) — SHRP may not be active" % dev_stage, Color(1.0, 0.9, 0.3))

	var stress_before_1: float = float(entity.emotion_data.stress)
	var damage_before_1: float = float(entity.emotion_data.get_meta("developmental_damage", 0.0))

	_print("  ── Test 1: threat intensity 0.50 (below SHRP threshold 0.85) ──", Color(0.6, 0.9, 1.0))
	_print("  stress before: %.2f   developmental_damage before: %.2f" % [stress_before_1, damage_before_1])
	_print("  → process_stressor({type:threat, intensity:0.50})")
	_child_stress_processor.process_stressor(entity, {"type": "threat", "intensity": 0.5}, tick)
	var stress_after_1: float = float(entity.emotion_data.stress)
	var damage_after_1: float = float(entity.emotion_data.get_meta("developmental_damage", 0.0))
	_print("  stress after:  %.2f   developmental_damage after:  %.2f" % [stress_after_1, damage_after_1])
	var pass_test_1: bool = (stress_after_1 - stress_before_1) < 1.0
	_print(
		"  SHRP suppressed cortisol response [%s if stress delta < 1.0]" % ("PASS" if pass_test_1 else "FAIL"),
		Color(0.6, 1.0, 0.6) if pass_test_1 else Color(1.0, 0.4, 0.4)
	)

	var damage_before_2: float = float(entity.emotion_data.get_meta("developmental_damage", 0.0))
	_print("  ── Test 2: deprivation intensity 0.50 (bypasses SHRP) ──", Color(0.6, 0.9, 1.0))
	_print("  developmental_damage before: %.2f" % damage_before_2)
	_print("  → process_stressor({type:deprivation, intensity:0.50})")
	_child_stress_processor.process_stressor(entity, {"type": "deprivation", "intensity": 0.5}, tick)
	var damage_after_2: float = float(entity.emotion_data.get_meta("developmental_damage", 0.0))
	var damage_delta_2: float = damage_after_2 - damage_before_2
	_print("  developmental_damage after:  %.2f  (%+.2f)" % [damage_after_2, damage_delta_2])
	var pass_test_2: bool = damage_delta_2 > 0.0
	_print(
		"  Deprivation channel active [%s if developmental_damage increased]" % ("PASS" if pass_test_2 else "FAIL"),
		Color(0.6, 1.0, 0.6) if pass_test_2 else Color(1.0, 0.4, 0.4)
	)

	var shrp_before_3: bool = bool(entity.emotion_data.get_meta("shrp_breached", false))
	_print("  ── Test 3: threat intensity 0.90 (above SHRP threshold → breach) ──", Color(0.6, 0.9, 1.0))
	_print("  shrp_breached before: %s" % str(shrp_before_3).to_lower())
	_print("  → process_stressor({type:threat, intensity:0.90})")
	_child_stress_processor.process_stressor(entity, {"type": "threat", "intensity": 0.9}, tick)
	var shrp_after_3: bool = bool(entity.emotion_data.get_meta("shrp_breached", false))
	var stress_after_3: float = float(entity.emotion_data.stress)
	_print("  shrp_breached after:  %s" % str(shrp_after_3).to_lower())
	_print("  stress after:  %.2f" % stress_after_3)
	var pass_test_3: bool = shrp_after_3
	_print(
		"  SHRP breach logged [%s if shrp_breached=true]" % ("PASS" if pass_test_3 else "FAIL"),
		Color(0.6, 1.0, 0.6) if pass_test_3 else Color(1.0, 0.4, 0.4)
	)


func cmd_test_ace_adulthood(args: Dictionary) -> void:
	var id_str: String = _pos_arg(args, 0)
	if id_str.is_empty():
		_print("Usage: test_ace_adulthood <agent_id>", Color(1.0, 0.4, 0.4))
		return
	if _parenting_system == null:
		_print("parenting_system not connected", Color(1.0, 0.4, 0.4))
		return
	var entity: Variant = _get_entity(id_str)
	if entity == null:
		return

	_header("test_ace_adulthood: %s (id:%d)" % [entity.entity_name, entity.id])

	var ace_score_total: float = float(entity.get_meta("ace_score_total", 0.0))
	var attachment_type: String = str(entity.get_meta("attachment_type", "secure"))
	var before_adult: bool = bool(entity.get_meta("adulthood_applied", false))
	var before_stress_mult: float = float(entity.get_meta("ace_stress_gain_mult", 1.0))
	var before_break_mult: float = float(entity.get_meta("ace_break_threshold_mult", 1.0))
	var before_epi_load: float = float(entity.get_meta("epigenetic_load_effective", 0.05))
	var before_allostatic: float = 0.0
	if entity.emotion_data != null:
		before_allostatic = float(entity.emotion_data.allostatic)

	_print("  ace_score_total: %.2f   attachment_type: %s" % [ace_score_total, attachment_type])
	_print("  ── Before ──", Color(0.6, 0.9, 1.0))
	_print("  adulthood_applied:        %s" % str(before_adult).to_lower())
	_print("  ace_stress_gain_mult:     %.2f" % before_stress_mult)
	_print("  ace_break_threshold_mult: %.2f" % before_break_mult)
	_print("  epigenetic_load:          %.2f" % before_epi_load)
	_print("  allostatic_load:          %.2f" % before_allostatic)
	if before_adult:
		_print("  ⚠ adulthood_applied was already true — call is idempotent, no changes expected", Color(1.0, 0.9, 0.3))

	_print("  ── Calling on_agent_reaches_adulthood... ──", Color(0.6, 0.9, 1.0))
	var tick: int = _sim_engine.current_tick if _sim_engine != null else 0
	_parenting_system.on_agent_reaches_adulthood(entity, tick)

	var after_adult: bool = bool(entity.get_meta("adulthood_applied", false))
	var after_stress_mult: float = float(entity.get_meta("ace_stress_gain_mult", 1.0))
	var after_break_mult: float = float(entity.get_meta("ace_break_threshold_mult", 1.0))
	var after_epi_load: float = float(entity.get_meta("epigenetic_load_effective", 0.05))
	var after_allostatic: float = 0.0
	if entity.emotion_data != null:
		after_allostatic = float(entity.emotion_data.allostatic)

	var stress_mult_color: Color = Color(0.6, 1.0, 0.6)
	if after_stress_mult > 1.5:
		stress_mult_color = Color(1.0, 0.4, 0.4)
	elif after_stress_mult > 1.2:
		stress_mult_color = Color(1.0, 0.9, 0.3)

	var break_mult_color: Color = Color(0.6, 1.0, 0.6)
	if after_break_mult < 0.75:
		break_mult_color = Color(1.0, 0.4, 0.4)
	elif after_break_mult < 0.9:
		break_mult_color = Color(1.0, 0.9, 0.3)

	_print("  ── After ──", Color(0.6, 0.9, 1.0))
	_print("  adulthood_applied:        %s" % str(after_adult).to_lower())
	_print(
		"  ace_stress_gain_mult:     %.2f  → %+.2f" % [
			after_stress_mult, after_stress_mult - before_stress_mult
		],
		stress_mult_color
	)
	_print(
		"  ace_break_threshold_mult: %.2f  → %+.2f" % [
			after_break_mult, after_break_mult - before_break_mult
		],
		break_mult_color
	)
	_print("  epigenetic_load:          %.2f  (unchanged)" % after_epi_load)
	_print(
		"  allostatic_load:          %.2f  → %+.2f" % [
			after_allostatic, after_allostatic - before_allostatic
		]
	)


func cmd_test_epigenetic(args: Dictionary) -> void:
	var mother_id_str: String = _pos_arg(args, 0)
	var father_id_str: String = _pos_arg(args, 1)
	if mother_id_str.is_empty() or father_id_str.is_empty():
		_print("Usage: test_epigenetic <mother_id> <father_id>", Color(1.0, 0.4, 0.4))
		return
	if _intergenerational_system == null:
		_print("intergenerational_system not connected", Color(1.0, 0.4, 0.4))
		return

	var mother_entity: Variant = _get_entity(mother_id_str)
	if mother_entity == null:
		return
	var father_entity: Variant = _get_entity(father_id_str)
	if father_entity == null:
		return

	_header(
		"test_epigenetic: mother=%s (id:%d) father=%s (id:%d)" % [
			mother_entity.entity_name, mother_entity.id, father_entity.entity_name, father_entity.id
		]
	)

	var mother_epi: float = float(mother_entity.get_meta("epigenetic_load_effective", 0.05))
	var mother_allostatic: float = 0.0
	if mother_entity.emotion_data != null:
		mother_allostatic = float(mother_entity.emotion_data.allostatic)
	var mother_scars: int = mother_entity.trauma_scars.size()

	var father_epi: float = float(father_entity.get_meta("epigenetic_load_effective", 0.05))
	var father_allostatic: float = 0.0
	if father_entity.emotion_data != null:
		father_allostatic = float(father_entity.emotion_data.allostatic)
	var father_scars: int = father_entity.trauma_scars.size()

	var adversity_index: float = 0.30
	if _settlement_manager != null and _entity_manager != null:
		var alive: Array = _entity_manager.get_alive_entities()
		var member_count: int = 0
		var stress_sum: float = 0.0
		for i in range(alive.size()):
			var member: Variant = alive[i]
			if member.settlement_id != mother_entity.settlement_id:
				continue
			member_count += 1
			if member.emotion_data != null:
				stress_sum += float(member.emotion_data.stress)
		if member_count > 0:
			adversity_index = clampf((stress_sum / float(member_count)) / 2000.0, 0.0, 1.0)

	var current_t: float = float(_intergenerational_system._current_T)
	var child_load: float = float(
		_intergenerational_system.calculate_child_epigenetic_load(
			mother_entity, father_entity, adversity_index
		)
	)
	var e_star: float = (0.05 + 0.10) / maxf(0.01, 1.0 - current_t)

	var risk_text: String = "LOW"
	var risk_detail: String = "(load < 0.10)"
	var risk_color: Color = Color(0.6, 1.0, 0.6)
	if child_load > 0.30:
		risk_text = "HIGH"
		risk_detail = "(load > 0.30)"
		risk_color = Color(1.0, 0.4, 0.4)
	elif child_load >= 0.10:
		risk_text = "MODERATE"
		risk_detail = "(0.10 <= load <= 0.30)"
		risk_color = Color(1.0, 0.9, 0.3)

	_print("  mother: epigenetic_load=%.2f  allostatic=%.2f  trauma_scars=%d" % [mother_epi, mother_allostatic, mother_scars])
	_print("  father: epigenetic_load=%.2f  allostatic=%.2f  trauma_scars=%d" % [father_epi, father_allostatic, father_scars])
	_print("  adversity_index: %.2f  (from settlement avg stress)" % adversity_index)
	_print("  current_T: %.2f" % current_t)
	_print("  ── Result ──", Color(0.6, 0.9, 1.0))
	_print("  child_epigenetic_load: %.2f" % child_load, risk_color)
	_print("  E* fixed point (T=%.2f): %.2f" % [current_t, e_star])
	_print("  Risk: %s  %s" % [risk_text, risk_detail], risk_color)


func cmd_test_attachment(args: Dictionary) -> void:
	var id_str: String = _pos_arg(args, 0)
	if id_str.is_empty():
		_print("Usage: test_attachment <agent_id>", Color(1.0, 0.4, 0.4))
		return
	var entity: Variant = _get_entity(id_str)
	if entity == null:
		return

	_header("test_attachment: %s (id:%d)" % [entity.entity_name, entity.id])

	var attachment_script: Variant = load("res://scripts/systems/phase5/attachment_system.gd")
	if attachment_script == null:
		_print("attachment_system.gd not found", Color(1.0, 0.4, 0.4))
		return
	var attachment_sys: Variant = attachment_script.new()

	var childhood_data_var: Variant = entity.get_meta("childhood_data", null)
	var childhood_data: Dictionary = {}
	if childhood_data_var is Dictionary:
		childhood_data = childhood_data_var
	else:
		_print("  (no childhood_data meta — entity may be adult or uninitialized)", Color(1.0, 0.9, 0.3))

	var samples_var: Variant = childhood_data.get("caregiver_sensitivity_samples", [])
	var samples: Array = samples_var if samples_var is Array else []
	var samples_sum: float = 0.0
	for sample in samples:
		samples_sum += float(sample)
	var samples_avg: float = samples_sum / float(samples.size()) if samples.size() > 0 else 0.0

	var caregiver_consistency: float = float(childhood_data.get("caregiver_consistency", 0.5))
	var ace_score: float = float(childhood_data.get("ace_score", entity.get_meta("ace_score_total", 0.0)))
	var abuser_is_caregiver: bool = bool(childhood_data.get("abuser_is_caregiver", false))

	var child_data: Dictionary = {
		"caregiver_sensitivity_samples": samples,
		"caregiver_consistency": caregiver_consistency,
		"ace_score": ace_score,
		"abuser_is_caregiver": abuser_is_caregiver,
	}
	var result: String = str(attachment_sys.determine_attachment_type(child_data))

	var result_color: Color = Color.WHITE
	if result == "disorganized":
		result_color = Color(1.0, 0.4, 0.4)
	elif result == "anxious" or result == "avoidant":
		result_color = Color(1.0, 0.9, 0.3)
	elif result == "secure":
		result_color = Color(0.6, 1.0, 0.6)

	_print("  ── Caregiver Data (from childhood_data meta) ──", Color(0.6, 0.9, 1.0))
	_print(
		"  caregiver_sensitivity_samples: %s  avg: %.2f" % [
			str(samples), samples_avg
		]
	)
	_print("  caregiver_consistency: %.2f" % caregiver_consistency)
	_print("  ace_score (childhood): %.2f" % ace_score)
	_print("  abuser_is_caregiver: %s" % str(abuser_is_caregiver).to_lower())
	_print("  ── Attachment Classification ──", Color(0.6, 0.9, 1.0))
	_print("  → determine_attachment_type result: %s" % result, result_color)


func cmd_test_simultaneous_ace(args: Dictionary) -> void:
	var id_str: String = _pos_arg(args, 0)
	if id_str.is_empty():
		_print("Usage: test_simultaneous_ace <agent_id>", Color(1.0, 0.4, 0.4))
		return
	var entity: Variant = _get_entity(id_str)
	if entity == null:
		return

	_header("test_simultaneous_ace: %s (id:%d)" % [entity.entity_name, entity.id])

	var ace_tracker: Variant = entity.get_meta("ace_tracker", null)
	if ace_tracker == null:
		var ace_tracker_script: Variant = load("res://scripts/systems/phase5/ace_tracker.gd")
		if ace_tracker_script == null:
			_print("ace_tracker.gd not found", Color(1.0, 0.4, 0.4))
			return
		ace_tracker = ace_tracker_script.new()
		entity.set_meta("ace_tracker", ace_tracker)
		_print("  (no ace_tracker found — created new instance)", Color(1.0, 0.9, 0.3))

	var tick: int = _sim_engine.current_tick if _sim_engine != null else 0
	var ace_score_before: float = float(ace_tracker.ace_score_total)
	var scars_before: int = entity.trauma_scars.size()
	var childhood_data_var: Variant = entity.get_meta("childhood_data", {})
	var ace_residual_before: float = 0.0
	if childhood_data_var is Dictionary:
		ace_residual_before = float(childhood_data_var.get("ace_residual_arousal", 0.0))

	_print("  ── Before ──", Color(0.6, 0.9, 1.0))
	_print("  ace_score_total: %.2f" % ace_score_before)
	_print("  trauma_scars: %d" % scars_before)
	_print("  ace_residual_arousal: %.2f  (from childhood_data)" % ace_residual_before)
	_print("  ── Injecting: physical_abuse (severity=0.70) + emotional_neglect (severity=0.60) ──", Color(0.6, 0.9, 1.0))
	_print("  → record_ace_event(\"physical_abuse\", 0.70, tick, name)")
	ace_tracker.record_ace_event("physical_abuse", 0.7, tick, entity.entity_name)
	_print("  → record_ace_event(\"emotional_neglect\", 0.60, tick, name)")
	ace_tracker.record_ace_event("emotional_neglect", 0.6, tick, entity.entity_name)

	var ace_score_after: float = float(ace_tracker.ace_score_total)
	var scars_after: int = entity.trauma_scars.size()
	var kindling_bonus: int = 0
	var scar_candidate: String = ""
	if entity.emotion_data != null:
		kindling_bonus = int(entity.emotion_data.get_meta("ace_kindling_bonus", 0))
		scar_candidate = str(entity.emotion_data.get_meta("ace_scar_candidate", ""))

	var saturation_color: Color = Color(0.6, 1.0, 0.6)
	var saturation_line: String = "score < 4: subclinical  (green)"
	if ace_score_after >= 8.0:
		saturation_color = Color(1.0, 0.4, 0.4)
		saturation_line = "score >= 8: high-load  (red)"
	elif ace_score_after >= 4.0:
		saturation_color = Color(1.0, 0.9, 0.3)
		saturation_line = "score 4-7: elevated  (yellow)"

	_print("  ── After ──", Color(0.6, 0.9, 1.0))
	_print("  ace_score_total: %.2f  (%+.2f)" % [ace_score_after, ace_score_after - ace_score_before])
	_print("  trauma_scars: %d  (unchanged — trauma_scar not auto-acquired here)" % scars_after)
	_print("  kindling_bonus: %d  (from ace_kindling_bonus meta)" % kindling_bonus)
	_print("  scar_candidate: %s  (from ace_scar_candidate meta)" % scar_candidate)
	_print("  ── Saturation ──", Color(0.6, 0.9, 1.0))
	_print("  %s" % saturation_line, saturation_color)


## debug_behavior [agent_id]
## Shows current action, timer, scores, hysteresis status, social value.
func cmd_debug_behavior(args: Dictionary) -> void:
	var id_str: String = _pos_arg(args, 0)
	if id_str.is_empty():
		_print("Usage: debug_behavior <agent_id>", Color(1.0, 0.4, 0.4))
		return
	var entity: Variant = _get_entity(id_str)
	if entity == null:
		return
	if _behavior_system == null:
		_print("behavior_system not connected", Color(1.0, 0.4, 0.4))
		return

	_header("debug_behavior entity=" + id_str)
	_print("current_action : " + str(entity.current_action))
	_print("action_timer   : " + str(entity.action_timer))
	_print("social         : " + "%.3f" % float(entity.social))

	var scores: Dictionary = _behavior_system._evaluate_actions(entity)
	var best_action: String = ""
	var best_score: float = -1.0
	for action in scores:
		var score: float = float(scores[action])
		if score > best_score:
			best_score = score
			best_action = str(action)
	var current_score: float = float(scores.get(entity.current_action, -1.0))
	_print("best_action    : " + best_action + " (" + "%.4f" % best_score + ")")
	_print("current_score  : " + "%.4f" % current_score)

	var threshold: float = float(_behavior_system._debug_hysteresis_threshold_override)
	var eff_threshold: float = threshold if threshold >= 0.0 else float(_behavior_system.HYSTERESIS_THRESHOLD)
	var hysteresis_fired: bool = entity.current_action != "" \
		and entity.current_action in scores \
		and current_score >= best_score * eff_threshold
	_print(
		"hysteresis     : " + ("ACTIVE (kept " + str(entity.current_action) + ")" if hysteresis_fired else "NOT fired"),
		Color(0.4, 1.0, 0.4) if hysteresis_fired else Color.WHITE
	)
	_print("── scores ──", Color(0.6, 0.9, 1.0))
	for action in scores:
		_print("  " + str(action) + " : " + "%.4f" % float(scores[action]))


## test_hysteresis [agent_id] [--off]
## Shows action switch count from history. --off disables hysteresis threshold.
func cmd_test_hysteresis(args: Dictionary) -> void:
	var id_str: String = _pos_arg(args, 0)
	if id_str.is_empty():
		_print("Usage: test_hysteresis <agent_id> [--off]", Color(1.0, 0.4, 0.4))
		return
	var pos_raw: Variant = args.get("_pos", [])
	var arg_list: Array = pos_raw if pos_raw is Array else []
	var disable_mode: bool = arg_list.has("--off")
	var entity: Variant = _get_entity(id_str)
	if entity == null:
		return
	if _behavior_system == null:
		_print("behavior_system not connected", Color(1.0, 0.4, 0.4))
		return

	_header("test_hysteresis entity=" + id_str + (" [THRESHOLD=0.0]" if disable_mode else ""))
	if disable_mode:
		_behavior_system.debug_set_hysteresis_threshold(0.0)
		_print("Hysteresis DISABLED (threshold=0.0). Run without --off to restore.", Color(1.0, 0.8, 0.2))
	else:
		_behavior_system.debug_set_hysteresis_threshold(-1.0)
		_print("Hysteresis RESTORED (threshold=" + "%.2f" % float(_behavior_system.HYSTERESIS_THRESHOLD) + ")", Color(0.4, 1.0, 0.4))

	var history: Array = entity.action_history
	_print("action_history (" + str(history.size()) + " entries):", Color(0.6, 0.9, 1.0))
	var switch_count: int = 0
	var prev_action: String = ""
	for i in range(history.size()):
		var entry: Variant = history[i]
		var act: String = str(entry.get("action", "?"))
		var tick: int = int(entry.get("tick", 0))
		if prev_action != "" and act != prev_action:
			switch_count += 1
		_print("  tick=" + str(tick) + " " + act + (" ← SWITCH" if (prev_action != "" and act != prev_action) else ""))
		prev_action = act
	_print(
		"total switches: " + str(switch_count) + " / " + str(maxi(history.size() - 1, 1)) + " transitions",
		Color(1.0, 0.8, 0.2)
	)


## test_social [agent_id]
## Shows social value, forces socialize action, shows change.
func cmd_test_social(args: Dictionary) -> void:
	var id_str: String = _pos_arg(args, 0)
	if id_str.is_empty():
		_print("Usage: test_social <agent_id>", Color(1.0, 0.4, 0.4))
		return
	var entity: Variant = _get_entity(id_str)
	if entity == null:
		return
	if _behavior_system == null:
		_print("behavior_system not connected", Color(1.0, 0.4, 0.4))
		return

	_header("test_social entity=" + id_str)
	var social_before: float = float(entity.social)
	_print("social BEFORE  : " + "%.4f" % social_before)
	_print("forcing socialize action...")
	_behavior_system._assign_action(entity, "socialize", 0)
	var social_after: float = float(entity.social)
	_print("social AFTER   : " + "%.4f" % social_after)
	var delta: float = social_after - social_before
	if delta > 0.0:
		_print("delta          : +" + "%.4f" % delta + " (P2 loop working)", Color(0.4, 1.0, 0.4))
	else:
		_print("delta          : " + "%.4f" % delta + " (no change - P2 loop missing?)", Color(1.0, 0.4, 0.4))


## test_boredom [agent_id]
## Shows action_history, boredom penalties, O_inquisitiveness.
func cmd_test_boredom(args: Dictionary) -> void:
	var id_str: String = _pos_arg(args, 0)
	if id_str.is_empty():
		_print("Usage: test_boredom <agent_id>", Color(1.0, 0.4, 0.4))
		return
	var entity: Variant = _get_entity(id_str)
	if entity == null:
		return
	if _behavior_system == null:
		_print("behavior_system not connected", Color(1.0, 0.4, 0.4))
		return

	_header("test_boredom entity=" + id_str)
	var inq: float = 0.5
	if entity.personality != null:
		inq = float(entity.personality.facets.get("O_inquisitiveness", 0.5))
	_print("O_inquisitiveness: " + "%.3f" % inq)

	var history: Array = entity.action_history
	var show_count: int = mini(10, history.size())
	_print("action_history (last " + str(show_count) + "):", Color(0.6, 0.9, 1.0))
	for i in range(history.size() - show_count, history.size()):
		var entry: Variant = history[i]
		_print("  tick=" + str(entry.get("tick", 0)) + " " + str(entry.get("action", "?")))

	var scores: Dictionary = _behavior_system._evaluate_actions(entity)
	_print("boredom penalties:", Color(0.6, 0.9, 1.0))
	for action in scores:
		var penalty: float = float(_behavior_system._calc_boredom_penalty(entity, str(action)))
		var marker: String = " <- PENALIZED" if penalty < 1.0 else ""
		_print(
			"  " + str(action) + " : " + "%.3f" % penalty + marker,
			Color(1.0, 0.6, 0.2) if penalty < 1.0 else Color.WHITE
		)
