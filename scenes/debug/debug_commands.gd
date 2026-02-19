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
