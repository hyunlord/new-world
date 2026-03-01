extends "res://scripts/core/simulation/simulation_system.gd"

## Stress System — Phase 1 Pipeline
## Lazarus & Folkman (1984) Transactional Model
## Selye (1956) GAS reserve
## McEwen (1998) Allostatic Load
## Hobfoll (1989) COR loss aversion
## Yerkes & Dodson (1908) Eustress efficiency

var _entity_manager: RefCounted
var _stressor_defs: Dictionary = {}
var _trauma_scar_system = null  # TraumaScarSystem (RefCounted), set by main.gd

# ── Constants ─────────────────────────────────────────────────────────
const _TraitEffectCache = preload("res://scripts/systems/psychology/trait_effect_cache.gd")
const StatCurveScript = preload("res://scripts/core/stats/stat_curve.gd")
const STRESS_CLAMP_MAX: float = 2000.0
const STRESS_EPSILON: float = 0.05

const EUSTRESS_OPTIMAL: float = 150.0

# ── Phase 4 Extension: C05 Denial + Rebound Queue ─────────────────────
## Gross (1998) Emotion Regulation — cognitive reappraisal and suppression
## Folkman & Lazarus (1988) — denial as maladaptive avoidant coping
const DENIAL_REDIRECT_FRACTION: float = 0.60   # fraction of stress redirected to hidden accumulator
const DENIAL_MAX_ACCUMULATOR: float = 800.0    # cap on hidden threat accumulator
const REBOUND_DECAY_PER_TICK: float = 0.0      # rebounds don't decay (full delayed payment)


func _init() -> void:
	system_name = "stress"
	priority = 34        # after emotion(32), before social(37)
	tick_interval = GameConfig.STRESS_SYSTEM_TICK_INTERVAL


## Initializes the stress system with the entity manager and loads stressor definitions from JSON.
func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_load_stressor_defs()


## Sets the TraumaScarSystem reference used to apply scar-based resilience modifiers.
func set_trauma_scar_system(tss) -> void:
	_trauma_scar_system = tss


## Schedule a delayed stress rebound — called by CopingSystem when C05 Denial expires.
## Gross (1998): suppressed stress is stored, not eliminated; it rebounds when defenses drop.
## rebound_queue meta: Array of {amount: float, delay: int}
func schedule_rebound(entity_id: int, amount: float, delay_ticks: int) -> void:
	var entity = _entity_manager.get_entity(entity_id)
	if entity == null or entity.emotion_data == null:
		return
	var ed = entity.emotion_data
	var queue = ed.get_meta("rebound_queue", [])
	queue.append({"amount": amount, "delay": delay_ticks})
	ed.set_meta("rebound_queue", queue)


## Updates stress for all alive entities each tick: processes rebound queues, continuous stressors, emotion contributions, recovery, allostatic load, and reserve.
func execute_tick(_tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity = alive[i]
		if entity.emotion_data == null:
			continue
		var is_sleeping: bool = entity.current_action == "sleep"
		var is_safe: bool = entity.settlement_id >= 0
		# Phase 4: process any scheduled stress rebounds (C05 Denial expiry)
		_process_rebound_queue(entity.emotion_data)
		_update_entity_stress(entity, is_sleeping, is_safe)


func _update_entity_stress(entity: RefCounted, is_sleeping: bool, is_safe: bool) -> void:
	var ed = entity.emotion_data
	var pd = entity.personality

	var breakdown: Dictionary = {}
	var hunger: float = StatQuery.get_normalized(entity, &"NEED_HUNGER")
	var energy: float = StatQuery.get_normalized(entity, &"NEED_ENERGY")
	var social: float = StatQuery.get_normalized(entity, &"NEED_SOCIAL")

	# 1) Lazarus appraisal + unmet-needs stress (combined native step)
	var primary_input: Dictionary = _calc_primary_inputs(entity, pd, ed, hunger, energy, social, breakdown)
	var continuous_input: float = float(primary_input.get("total", 0.0))

	# 3) 스트레스 후유증 traces
	var trace_input: float = _process_stress_traces(ed, breakdown)

	# 4) 감정 + 회복 + 최종 delta(denial 포함) 통합 step
	var ace_stress_mult: float = float(entity.get_meta("ace_stress_gain_mult", 1.0))
	# v3 trait effect: stress/mult target=accumulation_rate
	var trait_accum_mult: float = _TraitEffectCache.get_stress_accum_mult(entity)

	# Phase 4: C05 Denial — redirect fraction of incoming stress to hidden accumulator
	## Folkman & Lazarus (1988): denial suppresses immediate distress at cost of later rebound
	var denial_active: bool = ed.get_meta("denial_active", false)
	var hidden: float = ed.get_meta("hidden_threat_accumulator", 0.0)
	var support_score: float = _calc_support_score(entity)
	var delta_step: Dictionary = _calc_emotion_recovery_delta(
		ed,
		support_score,
		is_sleeping,
		is_safe,
		continuous_input,
		trace_input,
		ace_stress_mult,
		trait_accum_mult,
		denial_active,
		hidden,
		breakdown
	)
	var delta: float = float(delta_step.get("delta", 0.0))
	if denial_active:
		ed.set_meta("hidden_threat_accumulator", float(delta_step.get("hidden_threat_accumulator", hidden)))

	ed.stress = clampf(ed.stress + delta, 0.0, STRESS_CLAMP_MAX)
	ed.stress_delta_last = delta
	ed.stress_breakdown = breakdown

	# Shaken 상태 카운트다운
	var shaken_remaining: int = ed.get_meta("shaken_remaining", 0)
	if shaken_remaining > 0:
		shaken_remaining -= 1
		ed.set_meta("shaken_remaining", shaken_remaining)
		if shaken_remaining <= 0:
			ed.set_meta("shaken_work_penalty", 0.0)

	# 7) Reserve + Allostatic + State snapshot (combined native step)
	var avoidant_mult: float = (
		GameConfig.ATTACHMENT_AVOIDANT_ALLO_MULT
		if str(entity.get_meta("attachment_type", "secure")) == "avoidant"
		else 1.0
	)
	var post_step: Dictionary = StatCurveScript.stress_post_update_step(
		ed.reserve,
		ed.stress,
		ed.resilience,
		ed.stress_delta_last,
		ed.gas_stage,
		is_sleeping,
		ed.allostatic,
		avoidant_mult
	)
	ed.reserve = float(post_step.get("reserve", ed.reserve))
	ed.gas_stage = int(post_step.get("gas_stage", ed.gas_stage))
	ed.allostatic = float(post_step.get("allostatic", ed.allostatic))
	var state_snapshot: Dictionary = post_step

	# 8) 스트레스 상태
	_update_stress_state(ed, state_snapshot)

	# 9) Resilience
	_update_resilience(entity, ed, pd)

	# 10) 스트레스 → 감정 역방향
	_apply_stress_to_emotions(ed, state_snapshot)

	# 11) 디버그 로그
	_debug_log(entity, ed, delta)


# ── 1) Lazarus appraisal + unmet-needs stress (combined) ─────────────
func _calc_primary_inputs(
	entity: RefCounted,
	_pd,
	ed,
	hunger: float,
	energy: float,
	social: float,
	breakdown: Dictionary
) -> Dictionary:
	var threat: float = 0.0
	var conflict: float = 0.0
	var support_score: float = _calc_support_score(entity)

	var E_axis: float = StatQuery.get_normalized(entity, &"HEXACO_E")
	var fear_val: float = ed.get_emotion("fear") if ed != null else 0.0
	var trust_val: float = ed.get_emotion("trust") if ed != null else 0.0

	var C_axis: float = StatQuery.get_normalized(entity, &"HEXACO_C")
	var O_axis: float = StatQuery.get_normalized(entity, &"HEXACO_O")
	var reserve_ratio: float = ed.reserve / 100.0 if ed != null else 0.5
	var inputs: Dictionary = StatCurveScript.stress_primary_step(
		hunger,
		energy,
		social,
		threat,
		conflict,
		support_score,
		E_axis,
		fear_val,
		trust_val,
		C_axis,
		O_axis,
		reserve_ratio
	)

	var s_hunger: float = float(inputs.get("hunger", 0.0))
	if s_hunger > STRESS_EPSILON:
		breakdown["hunger"] = s_hunger

	var s_energy: float = float(inputs.get("energy_deficit", 0.0))
	if s_energy > STRESS_EPSILON:
		breakdown["energy_deficit"] = s_energy

	var s_social: float = float(inputs.get("social_isolation", 0.0))
	if s_social > STRESS_EPSILON:
		breakdown["social_isolation"] = s_social

	return inputs


# ── 3) 스트레스 후유증 traces ─────────────────────────────────────────
func _process_stress_traces(ed, breakdown: Dictionary) -> float:
	var trace_count: int = ed.stress_traces.size()
	if trace_count <= 0:
		return 0.0

	var per_tick: PackedFloat32Array = PackedFloat32Array()
	var decay_rate: PackedFloat32Array = PackedFloat32Array()
	per_tick.resize(trace_count)
	decay_rate.resize(trace_count)
	for i in range(trace_count):
		var trace_data: Dictionary = ed.stress_traces[i]
		per_tick[i] = float(trace_data.get("per_tick", 0.0))
		decay_rate[i] = float(trace_data.get("decay_rate", 0.05))

	var result: Dictionary = StatCurveScript.stress_trace_batch_step(per_tick, decay_rate, 0.01)
	var total: float = float(result.get("total_contribution", 0.0))
	var updated: PackedFloat32Array = result.get("updated_per_tick", PackedFloat32Array())
	var active_mask: PackedByteArray = result.get("active_mask", PackedByteArray())
	var usable_len: int = mini(trace_count, mini(updated.size(), active_mask.size()))
	var next_traces: Array = []
	for i in range(usable_len):
		var trace: Dictionary = ed.stress_traces[i]
		var contribution: float = float(per_tick[i])
		trace["per_tick"] = float(updated[i])
		if int(active_mask[i]) != 0:
			var key: String = "trace_%s" % str(trace.get("source_id", "unknown"))
			breakdown[key] = contribution
			next_traces.append(trace)
	ed.stress_traces = next_traces
	return total


# ── 4) 감정 + 회복 + delta 통합 ───────────────────────────────────────
func _calc_emotion_recovery_delta(
	ed,
	support_score: float,
	is_sleeping: bool,
	is_safe: bool,
	continuous_input: float,
	trace_input: float,
	ace_stress_mult: float,
	trait_accum_mult: float,
	denial_active: bool,
	hidden_threat_accumulator: float,
	breakdown: Dictionary
) -> Dictionary:
	if ed == null:
		return {
			"delta": 0.0,
			"hidden_threat_accumulator": hidden_threat_accumulator,
		}

	var fear_val: float = ed.get_emotion("fear")
	var anger_val: float = ed.get_emotion("anger")
	var sadness_val: float = ed.get_emotion("sadness")
	var disgust_val: float = ed.get_emotion("disgust")
	var surprise_val: float = ed.get_emotion("surprise")
	var joy_val: float = ed.get_emotion("joy")
	var trust_val: float = ed.get_emotion("trust")
	var anticipation_val: float = ed.get_emotion("anticipation")
	var out: Dictionary = StatCurveScript.stress_emotion_recovery_delta_step(
		fear_val,
		anger_val,
		sadness_val,
		disgust_val,
		surprise_val,
		joy_val,
		trust_val,
		anticipation_val,
		ed.valence,
		ed.arousal,
		ed.stress,
		support_score,
		ed.resilience,
		ed.reserve,
		is_sleeping,
		is_safe,
		continuous_input,
		trace_input,
		ace_stress_mult,
		trait_accum_mult,
		STRESS_EPSILON,
		denial_active,
		DENIAL_REDIRECT_FRACTION,
		hidden_threat_accumulator,
		DENIAL_MAX_ACCUMULATOR
	)

	var emotion_keys: Array[String] = ["fear", "anger", "sadness", "disgust", "surprise", "joy", "trust", "anticipation"]
	for i in range(emotion_keys.size()):
		var emotion_name: String = emotion_keys[i]
		var contrib: float = float(out.get(emotion_name, 0.0))
		if absf(contrib) > STRESS_EPSILON:
			breakdown["emo_%s" % emotion_name] = contrib

	var va_contrib: float = float(out.get("va_composite", 0.0))
	if va_contrib > STRESS_EPSILON:
		breakdown["va_composite"] = va_contrib

	var recovery: float = float(out.get("recovery", 0.0))
	breakdown["recovery"] = -recovery
	return out


# ── 8) 스트레스 상태 ──────────────────────────────────────────────────
func _update_stress_state(ed, snapshot: Dictionary) -> void:
	ed.stress_state = int(snapshot.get("stress_state", 0))


# ── 9) Resilience ────────────────────────────────────────────────────
func _update_resilience(entity: RefCounted, ed, pd) -> void:
	if pd == null:
		ed.resilience = 0.5
		return

	var E: float = StatQuery.get_normalized(entity, &"HEXACO_E")
	var C: float = StatQuery.get_normalized(entity, &"HEXACO_C")
	var X: float = StatQuery.get_normalized(entity, &"HEXACO_X")
	var O: float = StatQuery.get_normalized(entity, &"HEXACO_O")
	var A: float = StatQuery.get_normalized(entity, &"HEXACO_A")
	var H: float = StatQuery.get_normalized(entity, &"HEXACO_H")

	var support: float = _calc_support_score(entity)
	var hunger: float = StatQuery.get_normalized(entity, &"NEED_HUNGER")
	var energy: float = StatQuery.get_normalized(entity, &"NEED_ENERGY")

	# 트라우마 흉터 회복력 모디파이어 (음수 = 회복 더 느림)
	var scar_resilience_mod: float = 0.0
	if _trauma_scar_system != null:
		scar_resilience_mod = _trauma_scar_system.get_scar_resilience_mod(entity)

	ed.resilience = StatCurveScript.stress_resilience_value(
		E,
		C,
		X,
		O,
		A,
		H,
		support,
		ed.allostatic,
		hunger,
		energy,
		scar_resilience_mod
	)


# ── 11) 스트레스 → 감정 역방향 ───────────────────────────────────────
func _apply_stress_to_emotions(ed, snapshot: Dictionary) -> void:
	ed.set_meta("stress_mu_sadness", float(snapshot.get("stress_mu_sadness", 0.0)))
	ed.set_meta("stress_mu_anger", float(snapshot.get("stress_mu_anger", 0.0)))
	ed.set_meta("stress_mu_fear", float(snapshot.get("stress_mu_fear", 0.0)))
	ed.set_meta("stress_mu_joy", float(snapshot.get("stress_mu_joy", 0.0)))
	ed.set_meta("stress_mu_trust", float(snapshot.get("stress_mu_trust", 0.0)))
	ed.set_meta("stress_neg_gain_mult", float(snapshot.get("stress_neg_gain_mult", 1.0)))
	ed.set_meta("stress_pos_gain_mult", float(snapshot.get("stress_pos_gain_mult", 1.0)))
	ed.set_meta("stress_blunt_mult", float(snapshot.get("stress_blunt_mult", 1.0)))


# ── Support score ─────────────────────────────────────────────────────
func _calc_support_score(_entity: RefCounted) -> float:
	var relationships = []
	if relationships.is_empty():
		return 0.3

	var strong: float = 0.0
	var weak_sum: float = 0.0

	for rel in relationships:
		var strength: float = rel.get("strength", 0.0)
		if strength > strong:
			weak_sum += strong
			strong = strength
		else:
			weak_sum += strength

	return clampf(0.65 * strong + 0.35 * (1.0 - exp(-weak_sum / 1.5)), 0.0, 1.0)


# ── Yerkes-Dodson work efficiency ─────────────────────────────────────
## Returns a Yerkes-Dodson work efficiency multiplier (0.35–1.10) based on the entity's current stress level.
func get_work_efficiency(ed) -> float:
	var s: float = ed.stress
	var perf: float
	if s < 150.0:
		perf = 1.0 + 0.0006 * s
	elif s < 350.0:
		perf = 1.09 - 0.0004 * (s - 150.0)
	else:
		perf = 1.01 - 0.0012 * (s - 350.0)
	# Shaken 후유증 패널티
	var shaken_penalty: float = ed.get_meta("shaken_work_penalty", 0.0)
	perf += shaken_penalty
	return clampf(perf, 0.35, 1.10)


# ── Event stress injection (COR loss aversion x2.5) ──────────────────
## Injects an immediate stress event into an entity's emotion_data, with optional per-tick trace and COR loss-aversion multiplier.
func inject_stress_event(ed, source_id: String, instant: float,
		per_tick: float = 0.0, decay_rate: float = 0.05,
		is_loss: bool = false, appraisal_scale: float = 1.0) -> void:
	var loss_mult: float = 2.5 if is_loss else 1.0
	var final_instant: float = instant * loss_mult * appraisal_scale

	ed.stress = clampf(ed.stress + final_instant, 0.0, STRESS_CLAMP_MAX)

	if per_tick > 0.01:
		ed.stress_traces.append({
			"source_id": source_id,
			"per_tick": per_tick * loss_mult * appraisal_scale,
			"decay_rate": decay_rate,
		})


# ── 스트레서 이벤트 데이터 로드 ────────────────────────────────────────
func _load_stressor_defs() -> void:
	var path: String = "res://data/stressor_events.json"
	if not FileAccess.file_exists(path):
		push_warning("[StressSystem] stressor_events.json not found")
		return
	var f = FileAccess.open(path, FileAccess.READ)
	if f == null:
		push_warning("[StressSystem] Cannot open stressor_events.json")
		return
	var text: String = f.get_as_text()
	f.close()
	var json = JSON.new()
	var err: int = json.parse(text)
	if err != OK:
		push_error("[StressSystem] stressor_events.json parse error: " + json.get_error_message())
		return
	var raw = json.get_data()
	# _comment 키 제거
	for key in raw.keys():
		if not key.begins_with("_comment"):
			_stressor_defs[key] = raw[key]


# ── Personality-aware 이벤트 스트레스 주입 ───────────────────────────
## 학술: Lazarus (1984) 개인별 appraisal
## HEXACO: 같은 사건도 성격에 따라 stress 강도가 다름
## COR (Hobfoll 1989): is_loss=true → 2.5배
func inject_event(entity, event_id: String, context: Dictionary = {}) -> void:
	if entity == null:
		return
	var ed = entity.emotion_data
	if ed == null:
		return
	if not _stressor_defs.has(event_id):
		push_warning("[StressSystem] Unknown stressor event: %s" % event_id)
		return

	var sdef = _stressor_defs[event_id]

	# 1) Base 값
	var instant = float(sdef.get("base_instant", 0.0))
	var per_tick = float(sdef.get("base_per_tick", 0.0))
	var decay_rate = float(sdef.get("base_decay_rate", 0.05))
	var is_loss = sdef.get("is_loss", false)

	# 2) 성격 스케일
	var p_mods = sdef.get("personality_modifiers", {})
	var personality_scale = _calc_personality_scale(entity, p_mods)

	# 3) 관계 스케일
	var r_def = sdef.get("relationship_scaling", {})
	var relationship_scale = _calc_relationship_scale(context, r_def)

	# 4) 상황 스케일
	var c_mods = sdef.get("context_modifiers", {})
	var context_scale = _calc_context_scale(context, c_mods)

	# 5) COR 손실 혐오
	var loss_mult: float = 2.5 if is_loss else 1.0

	# 6) 최종 계산
	var total_scale = personality_scale * relationship_scale * context_scale
	var final_instant = instant * total_scale * loss_mult
	var final_per_tick = per_tick * total_scale * loss_mult

	# 7) Stress 주입
	ed.stress = clampf(ed.stress + final_instant, 0.0, STRESS_CLAMP_MAX)

	if absf(final_per_tick) > 0.01:
		ed.stress_traces.append({
			"source_id": event_id,
			"per_tick": final_per_tick,
			"decay_rate": decay_rate,
		})

	# 8) 감정 직접 주입
	var emo_inject = sdef.get("emotion_inject", {})
	_inject_emotions(ed, emo_inject, total_scale)

	# 9) 디버그 로그
	if GameConfig.DEBUG_STRESS_LOG:
		var ename = entity.entity_name if "entity_name" in entity else "?"
		print("[STRESS_EVENT] %s | %s | inst=%.0f ptk=%.1f | p=%.2f r=%.2f c=%.2f | loss=%s" % [
			ename, event_id, final_instant, final_per_tick,
			personality_scale, relationship_scale, context_scale,
			str(is_loss)
		])


func _calc_personality_scale(entity, p_mods: Dictionary) -> float:
	if p_mods.is_empty():
		return 1.0

	var pd = entity.personality
	var scale: float = 1.0

	for key in p_mods:
		if key == "traits":
			continue
		var mod = p_mods[key]
		if typeof(mod) != TYPE_DICTIONARY:
			continue
		var weight = float(mod.get("weight", 0.0))
		var direction = mod.get("direction", "high_amplifies")

		# 축 또는 facet 값 가져오기
		var value: float = 0.5
		if pd != null:
			if key.ends_with("_axis"):
				var axis_id: String = key.substr(0, key.length() - 5)
				value = StatQuery.get_normalized(entity, StringName("HEXACO_" + axis_id))
			else:
				value = float(pd.facets.get(key, 0.5))

		# 방향에 따른 배수
		var deviation: float
		if direction == "high_amplifies":
			deviation = (value - 0.5) * 2.0
		else:
			deviation = (0.5 - value) * 2.0

		scale *= (1.0 + weight * deviation)

	# Trait 배수
	var trait_mods = p_mods.get("traits", {})
	if typeof(trait_mods) == TYPE_DICTIONARY:
		for trait_id in trait_mods:
			if _entity_has_trait(entity, trait_id):
				scale *= float(trait_mods[trait_id])

	return clampf(scale, 0.05, 4.0)


func _calc_relationship_scale(context: Dictionary, r_def: Dictionary) -> float:
	var method = r_def.get("method", "none")
	if method == "none" or method == "":
		return 1.0
	if method == "bond_strength":
		var bond = float(context.get("bond_strength", 0.5))
		var min_m = float(r_def.get("min_mult", 0.3))
		var max_m = float(r_def.get("max_mult", 1.5))
		return clampf(min_m + (max_m - min_m) * bond, min_m, max_m)
	return 1.0


func _calc_context_scale(context: Dictionary, c_mods: Dictionary) -> float:
	var scale: float = 1.0
	for key in c_mods:
		if context.get(key, false):
			scale *= float(c_mods[key])
	return clampf(scale, 0.1, 5.0)


func _entity_has_trait(entity, trait_id: String) -> bool:
	for t in entity.display_traits:
		if t.get("id", "") == trait_id:
			return true
	return false


func _inject_emotions(ed, emo_inject: Dictionary, scale: float) -> void:
	for key in emo_inject:
		var raw_val = float(emo_inject[key]) * scale
		# key 형식: "sadness_fast", "trust_slow"
		var last_underscore: int = key.rfind("_")
		if last_underscore < 0:
			continue
		var emo_name: String = key.substr(0, last_underscore)
		var layer: String = key.substr(last_underscore + 1)
		if layer == "fast":
			if ed.fast.has(emo_name):
				ed.fast[emo_name] = clampf(ed.fast.get(emo_name, 0.0) + raw_val, 0.0, 100.0)
		elif layer == "slow":
			if ed.slow.has(emo_name):
				ed.slow[emo_name] = clampf(ed.slow.get(emo_name, 0.0) + raw_val, -50.0, 100.0)


# ── Phase 4: Rebound Queue Processing ────────────────────────────────
func _process_rebound_queue(ed: RefCounted) -> void:
	## Gross (1998): suppressed stress surfaces when denial coping terminates
	## Queue format: Array of {amount: float, delay: int}
	var queue = ed.get_meta("rebound_queue", [])
	if queue.is_empty():
		return

	var remaining: Array = []
	var total_rebound: float = 0.0

	for entry in queue:
		entry["delay"] -= 1
		if entry["delay"] <= 0:
			total_rebound += entry.get("amount", 0.0)
		else:
			remaining.append(entry)

	ed.set_meta("rebound_queue", remaining)

	if total_rebound > 0.0:
		ed.stress = clampf(ed.stress + total_rebound, 0.0, STRESS_CLAMP_MAX)
		# Clears hidden accumulator proportionally (rebounded stress is no longer hidden)
		var hidden: float = ed.get_meta("hidden_threat_accumulator", 0.0)
		ed.set_meta("hidden_threat_accumulator", maxf(0.0, hidden - total_rebound))


# ── Debug log ─────────────────────────────────────────────────────────
func _debug_log(entity: RefCounted, ed, delta: float) -> void:
	if not GameConfig.DEBUG_STRESS_LOG:
		return

	var ename = entity.entity_name
	var parts: Array = []
	for key in ed.stress_breakdown:
		parts.append("%s:%.1f" % [key, ed.stress_breakdown[key]])

	print("[STRESS] %s | S:%.0f(D%+.1f) R:%.0f A:%.1f Res:%.2f GAS:%d | %s" % [
		ename, ed.stress, delta, ed.reserve, ed.allostatic,
		ed.resilience, ed.gas_stage, ", ".join(parts)
	])
