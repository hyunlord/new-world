extends "res://scripts/core/simulation_system.gd"

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
const STRESS_CLAMP_MAX: float = 2000.0
const STRESS_EPSILON: float = 0.05

const BASE_DECAY_PER_TICK: float = 1.2
const DECAY_FRAC: float = 0.006
const SAFE_DECAY_BONUS: float = 0.8
const SLEEP_DECAY_BONUS: float = 1.5
const SUPPORT_DECAY_MULT: float = 0.12

const THRESHOLD_TENSE: float = 200.0
const THRESHOLD_CRISIS: float = 350.0
const THRESHOLD_BREAK_RISK: float = 500.0

const RESERVE_MAX: float = 100.0

const ALLO_RATE: float = 0.035
const ALLO_STRESS_THRESHOLD: float = 250.0
const ALLO_RECOVERY_THRESHOLD: float = 120.0
const ALLO_RECOVERY_RATE: float = 0.003

const EMOTION_STRESS_THRESHOLD: float = 20.0
const EMOTION_WEIGHTS: Dictionary = {
	"fear": 0.09, "anger": 0.06, "sadness": 0.05,
	"disgust": 0.04, "surprise": 0.03,
	"joy": -0.05, "trust": -0.04, "anticipation": -0.02
}
const VA_GAMMA: float = 3.0
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
	tick_interval = 2    # every 2 ticks (same cadence as needs_system)


func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_load_stressor_defs()


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


func execute_tick(tick: int) -> void:
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

	# 1) Lazarus Appraisal Scale
	var appraisal_scale: float = _calc_appraisal_scale(entity, pd, ed)

	# 2) 지속 스트레서 (욕구 결핍)
	var continuous_input: float = _calc_continuous_stressors(entity, appraisal_scale, breakdown)

	# 3) 스트레스 후유증 traces
	var trace_input: float = _process_stress_traces(ed, breakdown)

	# 4) 감정 → 스트레스
	var emotion_input: float = _calc_emotion_contribution(ed, breakdown)

	# 5) 회복
	var recovery: float = _calc_recovery(entity, ed, pd, is_sleeping, is_safe, breakdown)

	# 6) 최종 업데이트 (Phase 5: ACE stress gain multiplier — Felitti 1998 dose-response)
	var ace_stress_mult: float = float(entity.get_meta("ace_stress_gain_mult", 1.0))
	var delta: float = (continuous_input + trace_input + emotion_input) * ace_stress_mult - recovery
	if absf(delta) < STRESS_EPSILON:
		delta = 0.0

	# Phase 4: C05 Denial — redirect fraction of incoming stress to hidden accumulator
	## Folkman & Lazarus (1988): denial suppresses immediate distress at cost of later rebound
	var denial_active: bool = ed.get_meta("denial_active", false)
	if denial_active and delta > 0.0:
		var redirected: float = delta * DENIAL_REDIRECT_FRACTION
		var hidden: float = ed.get_meta("hidden_threat_accumulator", 0.0)
		hidden = minf(hidden + redirected, DENIAL_MAX_ACCUMULATOR)
		ed.set_meta("hidden_threat_accumulator", hidden)
		delta -= redirected  # only partial stress hits immediately

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

	# 7) Reserve (GAS)
	_update_reserve(ed, pd, is_sleeping)

	# 8) Allostatic Load
	_update_allostatic(ed)

	# 9) 스트레스 상태
	_update_stress_state(ed)

	# 10) Resilience
	_update_resilience(entity, ed, pd)

	# 11) 스트레스 → 감정 역방향
	_apply_stress_to_emotions(ed)

	# 12) 디버그 로그
	_debug_log(entity, ed, delta)


# ── 1) Lazarus Appraisal Scale ────────────────────────────────────────
func _calc_appraisal_scale(entity: RefCounted, pd, ed) -> float:
	var hunger: float = entity.hunger
	var energy: float = entity.energy
	var social: float = entity.social
	var threat: float = 0.0
	var conflict: float = 0.0

	var D_dep: float = 0.45 * (1.0 - hunger) + 0.35 * (1.0 - energy) + 0.20 * (1.0 - social)
	var D: float = clampf(0.30 * D_dep + 0.40 * threat + 0.20 * conflict, 0.0, 1.0)

	var R_physical: float = 0.5 * hunger + 0.5 * energy
	var R_safety: float = 1.0 - threat
	var R_support: float = _calc_support_score(entity)
	var R: float = clampf(0.30 * R_physical + 0.30 * R_safety + 0.25 * R_support + 0.15 * 0.5, 0.0, 1.0)

	var E_axis: float = pd.axes.get("E", 0.5) if pd != null else 0.5
	var fear_val: float = ed.get_emotion("fear") if ed != null else 0.0
	var trust_val: float = ed.get_emotion("trust") if ed != null else 0.0

	var threat_appraisal: float = D * (1.0 + 0.55 * (E_axis - 0.5) * 2.0 + 0.25 * (fear_val / 100.0) - 0.15 * (trust_val / 100.0))

	var C_axis: float = pd.axes.get("C", 0.5) if pd != null else 0.5
	var O_axis: float = pd.axes.get("O", 0.5) if pd != null else 0.5
	var reserve_ratio: float = ed.reserve / 100.0 if ed != null else 0.5
	var coping_appraisal: float = R * (1.0 + 0.35 * (C_axis - 0.5) * 2.0 + 0.20 * (O_axis - 0.5) * 2.0 + 0.20 * reserve_ratio)

	var imbalance: float = maxf(0.0, threat_appraisal - coping_appraisal)
	return clampf(1.0 + 0.8 * imbalance, 0.7, 1.9)


# ── 2) 지속 스트레서 ──────────────────────────────────────────────────
func _calc_continuous_stressors(entity: RefCounted, appraisal_scale: float, breakdown: Dictionary) -> float:
	var total: float = 0.0
	var hunger: float = entity.hunger
	var energy: float = entity.energy
	var social: float = entity.social

	var h_def: float = clampf((0.35 - hunger) / 0.35, 0.0, 1.0)
	var s_hunger: float = (3.0 * h_def + 9.0 * h_def * h_def) * appraisal_scale
	if s_hunger > STRESS_EPSILON:
		total += s_hunger
		breakdown["hunger"] = s_hunger

	var e_def: float = clampf((0.40 - energy) / 0.40, 0.0, 1.0)
	var s_energy: float = (2.0 * e_def + 10.0 * e_def * e_def) * appraisal_scale
	if s_energy > STRESS_EPSILON:
		total += s_energy
		breakdown["energy_deficit"] = s_energy

	var soc_def: float = clampf((0.25 - social) / 0.25, 0.0, 1.0)
	var s_social: float = 2.0 * soc_def * soc_def * appraisal_scale
	if s_social > STRESS_EPSILON:
		total += s_social
		breakdown["social_isolation"] = s_social

	return total


# ── 3) 스트레스 후유증 traces ─────────────────────────────────────────
func _process_stress_traces(ed, breakdown: Dictionary) -> float:
	var total: float = 0.0
	var to_remove: Array = []

	for i in range(ed.stress_traces.size()):
		var trace = ed.stress_traces[i]
		var contribution: float = trace.get("per_tick", 0.0)
		total += contribution

		var decay: float = trace.get("decay_rate", 0.05)
		trace["per_tick"] = contribution * (1.0 - decay)

		if trace["per_tick"] < 0.01:
			to_remove.append(i)
		else:
			var key = "trace_%s" % trace.get("source_id", "unknown")
			breakdown[key] = contribution

	for i in range(to_remove.size() - 1, -1, -1):
		ed.stress_traces.remove_at(to_remove[i])

	return total


# ── 4) 감정 → 스트레스 ───────────────────────────────────────────────
func _calc_emotion_contribution(ed, breakdown: Dictionary) -> float:
	var total: float = 0.0
	if ed == null:
		return 0.0

	for emotion_name in EMOTION_WEIGHTS:
		var weight: float = EMOTION_WEIGHTS[emotion_name]
		var value: float = ed.get_emotion(emotion_name)
		var excess: float = maxf(0.0, value - EMOTION_STRESS_THRESHOLD)
		var contrib: float = weight * excess
		if absf(contrib) > STRESS_EPSILON:
			total += contrib
			breakdown["emo_%s" % emotion_name] = contrib

	var valence: float = ed.valence
	var arousal: float = ed.arousal
	var neg: float = clampf(-valence / 100.0, 0.0, 1.0)
	var ar: float = clampf(arousal / 100.0, 0.0, 1.0)
	var va_contrib: float = VA_GAMMA * ar * neg
	if va_contrib > STRESS_EPSILON:
		total += va_contrib
		breakdown["va_composite"] = va_contrib

	return total


# ── 5) 회복 ──────────────────────────────────────────────────────────
func _calc_recovery(entity: RefCounted, ed, pd, is_sleeping: bool, is_safe: bool, breakdown: Dictionary) -> float:
	var decay: float = BASE_DECAY_PER_TICK + DECAY_FRAC * ed.stress

	if is_safe:
		decay += SAFE_DECAY_BONUS
	if is_sleeping:
		decay += SLEEP_DECAY_BONUS

	var support: float = _calc_support_score(entity)
	decay *= 1.0 + SUPPORT_DECAY_MULT * support

	var resilience: float = ed.resilience if ed != null else 0.5
	decay *= 1.0 + 0.10 * (resilience - 0.5) * 2.0

	if ed.reserve < 30.0:
		decay *= 0.85

	breakdown["recovery"] = -decay
	return decay


# ── 7) Reserve (GAS) ─────────────────────────────────────────────────
func _update_reserve(ed, pd, is_sleeping: bool) -> void:
	var resilience: float = ed.resilience

	var drain: float = maxf(0.0, (ed.stress - 150.0) / 350.0) * (0.7 + 0.6 * (1.0 - resilience))
	var recover_base: float = 0.4 + 0.6 * resilience
	var recover: float = recover_base * (1.0 if is_sleeping else 0.15)

	ed.reserve = clampf(ed.reserve - drain + recover, 0.0, RESERVE_MAX)

	if ed.stress_delta_last > 40.0 or ed.stress > 400.0:
		if ed.gas_stage == 0:
			ed.gas_stage = 1  # Alarm

	if ed.reserve >= 30.0 and ed.stress < 500.0:
		if ed.gas_stage == 1:
			ed.gas_stage = 2  # Resistance

	if ed.reserve < 30.0:
		ed.gas_stage = 3  # Exhaustion


# ── 8) Allostatic Load ────────────────────────────────────────────────
func _update_allostatic(ed) -> void:
	if ed.stress > ALLO_STRESS_THRESHOLD:
		var allo_inc: float = ALLO_RATE * maxf(0.0, ed.stress - ALLO_STRESS_THRESHOLD) / ALLO_STRESS_THRESHOLD
		allo_inc = minf(allo_inc, 0.05)
		ed.allostatic = clampf(ed.allostatic + allo_inc, 0.0, 100.0)

	if ed.stress < ALLO_RECOVERY_THRESHOLD:
		ed.allostatic = clampf(ed.allostatic - ALLO_RECOVERY_RATE, 0.0, 100.0)


# ── 9) 스트레스 상태 ──────────────────────────────────────────────────
func _update_stress_state(ed) -> void:
	if ed.stress >= THRESHOLD_BREAK_RISK:
		ed.stress_state = 3
	elif ed.stress >= THRESHOLD_CRISIS:
		ed.stress_state = 2
	elif ed.stress >= THRESHOLD_TENSE:
		ed.stress_state = 1
	else:
		ed.stress_state = 0


# ── 10) Resilience ────────────────────────────────────────────────────
func _update_resilience(entity: RefCounted, ed, pd) -> void:
	if pd == null:
		ed.resilience = 0.5
		return

	var E: float = pd.axes.get("E", 0.5)
	var C: float = pd.axes.get("C", 0.5)
	var X: float = pd.axes.get("X", 0.5)
	var O: float = pd.axes.get("O", 0.5)
	var A: float = pd.axes.get("A", 0.5)
	var H: float = pd.axes.get("H", 0.5)

	var support: float = _calc_support_score(entity)

	var r: float = (0.35 * (1.0 - E)
		+ 0.25 * C
		+ 0.15 * X
		+ 0.10 * O
		+ 0.10 * A
		+ 0.05 * H
		+ 0.25 * support
		- 0.30 * (ed.allostatic / 100.0))

	var hunger: float = entity.hunger
	var energy: float = entity.energy
	var fatigue_penalty: float = clampf((0.3 - energy) / 0.3, 0.0, 0.3) + clampf((0.3 - hunger) / 0.3, 0.0, 0.2)
	r -= 0.20 * fatigue_penalty

	# 트라우마 흉터 회복력 모디파이어 (음수 = 회복 더 느림)
	if _trauma_scar_system != null:
		r += _trauma_scar_system.get_scar_resilience_mod(entity)
	ed.resilience = clampf(r, 0.05, 1.0)


# ── 11) 스트레스 → 감정 역방향 ───────────────────────────────────────
func _apply_stress_to_emotions(ed) -> void:
	var s1: float = clampf((ed.stress - 100.0) / 400.0, 0.0, 1.0)
	var s2: float = clampf((ed.stress - 300.0) / 400.0, 0.0, 1.0)
	var allo_ratio: float = ed.allostatic / 100.0

	ed.set_meta("stress_mu_sadness", 6.0 * s1 + 10.0 * allo_ratio)
	ed.set_meta("stress_mu_anger", 4.0 * s1 + 8.0 * allo_ratio)
	ed.set_meta("stress_mu_fear", 5.0 * s1 + 12.0 * allo_ratio)
	ed.set_meta("stress_mu_joy", -(5.0 * s1 + 12.0 * allo_ratio))
	ed.set_meta("stress_mu_trust", -(4.0 * s1 + 10.0 * allo_ratio))

	ed.set_meta("stress_neg_gain_mult", 1.0 + 0.7 * s2)
	ed.set_meta("stress_pos_gain_mult", 1.0 - 0.5 * s2)

	var blunt_denom: float = 1.0 + 0.9 * allo_ratio * (2.0 if allo_ratio > 0.6 else 1.0)
	ed.set_meta("stress_blunt_mult", 1.0 / blunt_denom)


# ── Support score ─────────────────────────────────────────────────────
func _calc_support_score(entity: RefCounted) -> float:
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
	# 6.5) 트라우마 흉터 민감도 배수 + 재활성화 체크
	if _trauma_scar_system != null:
		total_scale *= _trauma_scar_system.get_scar_stress_sensitivity(entity)
		# event_id를 context_type으로 사용해 흉터 재활성화 체크
		_trauma_scar_system.check_reactivation(entity, event_id, 0)
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
	if OS.is_debug_build():
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
				value = float(pd.axes.get(axis_id, 0.5))
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
	# Direct context_modifier override (e.g. age-stage scaling from mortality_system)
	if context.has("context_modifier"):
		scale *= float(context.get("context_modifier", 1.0))
	return clampf(scale, 0.1, 5.0)


func _entity_has_trait(entity, trait_id: String) -> bool:
	if entity == null or not "trait_strengths" in entity:
		return false
	return float(entity.trait_strengths.get(trait_id, 0.0)) >= 0.10


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
	if not OS.is_debug_build():
		return
	if absf(delta) < 1.0 and ed.stress < 50.0:
		return

	var ename = entity.entity_name
	var parts: Array = []
	for key in ed.stress_breakdown:
		parts.append("%s:%.1f" % [key, ed.stress_breakdown[key]])

	print("[STRESS] %s | S:%.0f(D%+.1f) R:%.0f A:%.1f Res:%.2f GAS:%d | %s" % [
		ename, ed.stress, delta, ed.reserve, ed.allostatic,
		ed.resilience, ed.gas_stage, ", ".join(parts)
	])
