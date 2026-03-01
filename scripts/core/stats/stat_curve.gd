extends RefCounted

## 모든 Growth / Influence 커브의 수학 구현체.
## 순수 함수만 포함. 상태 없음. Rust 전환 시 이 파일만 교체.
##
## Growth Curve: XP/이벤트 → 스탯 변화량 계산
## Influence Curve: 스탯값 → 다른 스탯/파라미터에 주는 영향력 계산

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GROWTH CURVES
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _XP_CURVE_CACHE_MAX: int = 128

static var _bridge_checked: bool = false
static var _bridge_ref: Object = null
static var _xp_curve_cache: Dictionary = {}

## LOG_DIMINISHING: 스킬, 신체 훈련
## "처음은 빠르게, 올라갈수록 점점 더 많은 XP 필요"
## 반환: 다음 레벨까지 필요한 XP (float)
static func log_xp_required(level: int, params: Dictionary) -> float:
	if level <= 0:
		return 0.0
	var xp_meta: Dictionary = _get_xp_curve_meta(params)
	var base: float = float(xp_meta.get("base", 100.0))
	var exponent: float = float(xp_meta.get("exponent", 1.8))
	var bp: Array = xp_meta.get("breakpoints", [])
	var bm: Array = xp_meta.get("multipliers", [])
	var bp_packed: PackedInt32Array = xp_meta.get("breakpoints_packed", PackedInt32Array())
	var bm_packed: PackedFloat32Array = xp_meta.get("multipliers_packed", PackedFloat32Array())
	var rust_result: Variant = _call_sim_bridge(
		"stat_log_xp_required",
		[
			level,
			base,
			exponent,
			bp_packed,
			bm_packed
		]
	)
	if rust_result != null:
		return float(rust_result)
	var mult: float = bm[0] if bm.size() > 0 else 1.0
	for i in range(bp.size()):
		if level >= bp[i] and i + 1 < bm.size():
			mult = bm[i + 1]
	return base * pow(float(level), exponent) * mult

## LOG_DIMINISHING의 역함수: 누적 XP → 현재 레벨
static func xp_to_level(xp: float, params: Dictionary, max_level: int = 100) -> int:
	var xp_meta: Dictionary = _get_xp_curve_meta(params)
	var base: float = float(xp_meta.get("base", 100.0))
	var exponent: float = float(xp_meta.get("exponent", 1.8))
	var bp: Array = xp_meta.get("breakpoints", [])
	var bm: Array = xp_meta.get("multipliers", [])
	var bp_packed: PackedInt32Array = xp_meta.get("breakpoints_packed", PackedInt32Array())
	var bm_packed: PackedFloat32Array = xp_meta.get("multipliers_packed", PackedFloat32Array())
	var rust_result: Variant = _call_sim_bridge(
		"stat_xp_to_level",
		[
			xp,
			base,
			exponent,
			bp_packed,
			bm_packed,
			max_level
		]
	)
	if rust_result != null:
		return int(rust_result)

	var cumulative: float = 0.0
	for lv in range(1, max_level + 1):
		cumulative += log_xp_required(lv, params)
		if cumulative > xp:
			return lv - 1
	return max_level


## LOG_DIMINISHING 진행도 계산:
## 입력된 현재 level/xp 기준으로 xp_at_level, xp_to_next, progress_in_level 산출.
static func skill_xp_progress(
	level: int,
	xp: float,
	params: Dictionary,
	max_level: int = 100
) -> Dictionary:
	var xp_meta: Dictionary = _get_xp_curve_meta(params)
	var base: float = float(xp_meta.get("base", 100.0))
	var exponent: float = float(xp_meta.get("exponent", 1.8))
	var bp: Array = xp_meta.get("breakpoints", [])
	var bm: Array = xp_meta.get("multipliers", [])
	var bp_packed: PackedInt32Array = xp_meta.get("breakpoints_packed", PackedInt32Array())
	var bm_packed: PackedFloat32Array = xp_meta.get("multipliers_packed", PackedFloat32Array())
	var rust_result: Variant = _call_sim_bridge(
		"stat_skill_xp_progress",
		[
			level,
			xp,
			base,
			exponent,
			bp_packed,
			bm_packed,
			max_level
		]
	)
	if rust_result is Dictionary:
		return rust_result

	var clamped_level: int = clampi(level, 0, max_level)
	var xp_at_level: float = 0.0
	for lv in range(1, clamped_level + 1):
		xp_at_level += log_xp_required(lv, params)

	var xp_to_next: float = 0.0
	if clamped_level < max_level:
		xp_to_next = log_xp_required(clamped_level + 1, params)

	return {
		"level": clamped_level,
		"max_level": max_level,
		"xp_at_level": xp_at_level,
		"xp_to_next": xp_to_next,
		"progress_in_level": xp - xp_at_level,
	}

## SCURVE: 성격, 가치관
## "초반은 빠르게 형성, 중반에 굳어짐, 후반은 거의 불변"
## 반환: 변화 속도 배수 (float)
static func scurve_speed(current_value: int, params: Dictionary) -> float:
	var bps: Array = params.get("phase_breakpoints", [300, 700])
	var speeds: Array = params.get("phase_speeds", [1.5, 1.0, 0.3])
	var bps_packed: PackedInt32Array = params.get("_phase_breakpoints_packed", PackedInt32Array())
	var speeds_packed: PackedFloat32Array = params.get("_phase_speeds_packed", PackedFloat32Array())
	if bps_packed.size() != bps.size():
		bps_packed = _to_packed_i32(bps)
		params["_phase_breakpoints_packed"] = bps_packed
	if speeds_packed.size() != speeds.size():
		speeds_packed = _to_packed_f32(speeds)
		params["_phase_speeds_packed"] = speeds_packed
	var rust_result: Variant = _call_sim_bridge(
		"stat_scurve_speed",
		[
			current_value,
			bps_packed,
			speeds_packed
		]
	)
	if rust_result != null:
		return float(rust_result)
	if current_value < bps[0]:
		return float(speeds[0]) if speeds.size() > 0 else 1.0
	elif current_value < bps[1]:
		return float(speeds[1]) if speeds.size() > 1 else 1.0
	else:
		return float(speeds[2]) if speeds.size() > 2 else 0.3

## DECAY_NATURAL: 욕구 자연 감소
## 반환: 새로운 스탯 값 (int, 0~1000 clamp)
static func need_decay(current: int, decay_per_year: int,
		ticks_elapsed: int, metabolic_mult: float,
		ticks_per_year: int = 4380) -> int:
	var rust_result: Variant = _call_sim_bridge(
		"stat_need_decay",
		[
			current,
			decay_per_year,
			ticks_elapsed,
			metabolic_mult,
			ticks_per_year
		]
	)
	if rust_result != null:
		return int(rust_result)

	var decay_per_tick: float = float(decay_per_year) / float(ticks_per_year)
	var total_decay: float = decay_per_tick * float(ticks_elapsed) * metabolic_mult
	return clampi(current - int(total_decay), 0, 1000)


## Continuous stress input from unmet hunger/energy/social needs.
## Returns Dictionary:
## { "hunger": float, "energy_deficit": float, "social_isolation": float, "total": float }
static func stress_continuous_inputs(
	hunger: float,
	energy: float,
	social: float,
	appraisal_scale: float
) -> Dictionary:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_continuous_inputs",
		[
			hunger,
			energy,
			social,
			appraisal_scale
		]
	)
	if rust_result is Dictionary:
		return rust_result

	var h_def: float = clampf((0.35 - hunger) / 0.35, 0.0, 1.0)
	var e_def: float = clampf((0.40 - energy) / 0.40, 0.0, 1.0)
	var soc_def: float = clampf((0.25 - social) / 0.25, 0.0, 1.0)

	var s_hunger: float = (3.0 * h_def + 9.0 * h_def * h_def) * appraisal_scale
	var s_energy: float = (2.0 * e_def + 10.0 * e_def * e_def) * appraisal_scale
	var s_social: float = 2.0 * soc_def * soc_def * appraisal_scale

	return {
		"hunger": s_hunger,
		"energy_deficit": s_energy,
		"social_isolation": s_social,
		"total": s_hunger + s_energy + s_social,
	}


## Lazarus appraisal-derived stress scale.
## Returns a clamped multiplier [0.7, 1.9].
static func stress_appraisal_scale(
	hunger: float,
	energy: float,
	social: float,
	threat: float,
	conflict: float,
	support_score: float,
	extroversion: float,
	fear_value: float,
	trust_value: float,
	conscientiousness: float,
	openness: float,
	reserve_ratio: float
) -> float:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_appraisal_scale",
		[
			hunger,
			energy,
			social,
			threat,
			conflict,
			support_score,
			extroversion,
			fear_value,
			trust_value,
			conscientiousness,
			openness,
			reserve_ratio
		]
	)
	if rust_result != null:
		return float(rust_result)

	var d_dep: float = 0.45 * (1.0 - hunger) + 0.35 * (1.0 - energy) + 0.20 * (1.0 - social)
	var d: float = clampf(0.30 * d_dep + 0.40 * threat + 0.20 * conflict, 0.0, 1.0)
	var r_physical: float = 0.5 * hunger + 0.5 * energy
	var r_safety: float = 1.0 - threat
	var r: float = clampf(0.30 * r_physical + 0.30 * r_safety + 0.25 * support_score + 0.15 * 0.5, 0.0, 1.0)

	var threat_appraisal: float = d * (
		1.0
		+ 0.55 * (extroversion - 0.5) * 2.0
		+ 0.25 * (fear_value / 100.0)
		- 0.15 * (trust_value / 100.0)
	)
	var coping_appraisal: float = r * (
		1.0
		+ 0.35 * (conscientiousness - 0.5) * 2.0
		+ 0.20 * (openness - 0.5) * 2.0
		+ 0.20 * reserve_ratio
	)
	var imbalance: float = maxf(0.0, threat_appraisal - coping_appraisal)
	return clampf(1.0 + 0.8 * imbalance, 0.7, 1.9)


## Combined primary step:
## appraisal scale + continuous unmet-needs stress output.
## Returns Dictionary:
## { "appraisal_scale", "hunger", "energy_deficit", "social_isolation", "total" }
static func stress_primary_step(
	hunger: float,
	energy: float,
	social: float,
	threat: float,
	conflict: float,
	support_score: float,
	extroversion: float,
	fear_value: float,
	trust_value: float,
	conscientiousness: float,
	openness: float,
	reserve_ratio: float
) -> Dictionary:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_primary_step",
		[
			hunger,
			energy,
			social,
			threat,
			conflict,
			support_score,
			extroversion,
			fear_value,
			trust_value,
			conscientiousness,
			openness,
			reserve_ratio
		]
	)
	if rust_result is Dictionary:
		return rust_result

	var appraisal: float = stress_appraisal_scale(
		hunger,
		energy,
		social,
		threat,
		conflict,
		support_score,
		extroversion,
		fear_value,
		trust_value,
		conscientiousness,
		openness,
		reserve_ratio
	)
	var continuous: Dictionary = stress_continuous_inputs(hunger, energy, social, appraisal)
	continuous["appraisal_scale"] = appraisal
	return continuous


## Emotion-to-stress contribution with fixed weights and VA composite term.
## Returns Dictionary:
## { fear, anger, sadness, disgust, surprise, joy, trust, anticipation, va_composite, total }
static func stress_emotion_contribution(
	fear: float,
	anger: float,
	sadness: float,
	disgust: float,
	surprise: float,
	joy: float,
	trust: float,
	anticipation: float,
	valence: float,
	arousal: float
) -> Dictionary:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_emotion_contribution",
		[
			fear,
			anger,
			sadness,
			disgust,
			surprise,
			joy,
			trust,
			anticipation,
			valence,
			arousal
		]
	)
	if rust_result is Dictionary:
		return rust_result

	const emotion_stress_threshold: float = 20.0
	var values: Dictionary = {
		"fear": fear,
		"anger": anger,
		"sadness": sadness,
		"disgust": disgust,
		"surprise": surprise,
		"joy": joy,
		"trust": trust,
		"anticipation": anticipation,
	}
	var weights: Dictionary = {
		"fear": 0.09,
		"anger": 0.06,
		"sadness": 0.05,
		"disgust": 0.04,
		"surprise": 0.03,
		"joy": -0.05,
		"trust": -0.04,
		"anticipation": -0.02,
	}
	var out: Dictionary = {}
	var total: float = 0.0
	for key in weights.keys():
		var excess: float = maxf(0.0, float(values.get(key, 0.0)) - emotion_stress_threshold)
		var contrib: float = float(weights.get(key, 0.0)) * excess
		out[key] = contrib
		total += contrib

	var neg: float = clampf(-valence / 100.0, 0.0, 1.0)
	var ar: float = clampf(arousal / 100.0, 0.0, 1.0)
	var va_contrib: float = 3.0 * ar * neg
	out["va_composite"] = va_contrib
	out["total"] = total + va_contrib
	return out


## Stress recovery decay value per tick.
static func stress_recovery_value(
	stress: float,
	support_score: float,
	resilience: float,
	reserve: float,
	is_sleeping: bool,
	is_safe: bool
) -> float:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_recovery_value",
		[
			stress,
			support_score,
			resilience,
			reserve,
			is_sleeping,
			is_safe
		]
	)
	if rust_result != null:
		return float(rust_result)

	var decay: float = 1.2 + 0.006 * stress
	if is_safe:
		decay += 0.8
	if is_sleeping:
		decay += 1.5
	decay *= 1.0 + 0.12 * support_score
	decay *= 1.0 + 0.10 * (resilience - 0.5) * 2.0
	if reserve < 30.0:
		decay *= 0.85
	return decay


## Final stress delta step with denial redirect handling.
## Returns Dictionary: { "delta": float, "hidden_threat_accumulator": float }
static func stress_delta_step(
	continuous_input: float,
	trace_input: float,
	emotion_input: float,
	ace_stress_mult: float,
	trait_accum_mult: float,
	recovery: float,
	epsilon: float,
	denial_active: bool,
	denial_redirect_fraction: float,
	hidden_threat_accumulator: float,
	denial_max_accumulator: float
) -> Dictionary:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_delta_step",
		[
			continuous_input,
			trace_input,
			emotion_input,
			ace_stress_mult,
			trait_accum_mult,
			recovery,
			epsilon,
			denial_active,
			denial_redirect_fraction,
			hidden_threat_accumulator,
			denial_max_accumulator
		]
	)
	if rust_result is Dictionary:
		return rust_result

	var delta: float = (continuous_input + trace_input + emotion_input) * ace_stress_mult * trait_accum_mult - recovery
	if absf(delta) < epsilon:
		delta = 0.0

	var hidden: float = hidden_threat_accumulator
	if denial_active and delta > 0.0:
		var redirected: float = delta * denial_redirect_fraction
		hidden = minf(hidden + redirected, denial_max_accumulator)
		delta -= redirected

	return {
		"delta": delta,
		"hidden_threat_accumulator": hidden,
	}


## Combined step:
## emotion contribution + recovery + final delta(denial redirect).
## Returns Dictionary:
## { fear, anger, sadness, disgust, surprise, joy, trust, anticipation, va_composite,
##   emotion_total, recovery, delta, hidden_threat_accumulator }
static func stress_emotion_recovery_delta_step(
	fear: float,
	anger: float,
	sadness: float,
	disgust: float,
	surprise: float,
	joy: float,
	trust: float,
	anticipation: float,
	valence: float,
	arousal: float,
	stress: float,
	support_score: float,
	resilience: float,
	reserve: float,
	is_sleeping: bool,
	is_safe: bool,
	continuous_input: float,
	trace_input: float,
	ace_stress_mult: float,
	trait_accum_mult: float,
	epsilon: float,
	denial_active: bool,
	denial_redirect_fraction: float,
	hidden_threat_accumulator: float,
	denial_max_accumulator: float
) -> Dictionary:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_emotion_recovery_delta_step",
		[
			fear,
			anger,
			sadness,
			disgust,
			surprise,
			joy,
			trust,
			anticipation,
			valence,
			arousal,
			stress,
			support_score,
			resilience,
			reserve,
			is_sleeping,
			is_safe,
			continuous_input,
			trace_input,
			ace_stress_mult,
			trait_accum_mult,
			epsilon,
			denial_active,
			denial_redirect_fraction,
			hidden_threat_accumulator,
			denial_max_accumulator
		]
	)
	if rust_result is Dictionary:
		return rust_result

	var emotion_out: Dictionary = stress_emotion_contribution(
		fear,
		anger,
		sadness,
		disgust,
		surprise,
		joy,
		trust,
		anticipation,
		valence,
		arousal
	)
	var recovery: float = stress_recovery_value(
		stress,
		support_score,
		resilience,
		reserve,
		is_sleeping,
		is_safe
	)
	var delta_out: Dictionary = stress_delta_step(
		continuous_input,
		trace_input,
		float(emotion_out.get("total", 0.0)),
		ace_stress_mult,
		trait_accum_mult,
		recovery,
		epsilon,
		denial_active,
		denial_redirect_fraction,
		hidden_threat_accumulator,
		denial_max_accumulator
	)
	return {
		"fear": float(emotion_out.get("fear", 0.0)),
		"anger": float(emotion_out.get("anger", 0.0)),
		"sadness": float(emotion_out.get("sadness", 0.0)),
		"disgust": float(emotion_out.get("disgust", 0.0)),
		"surprise": float(emotion_out.get("surprise", 0.0)),
		"joy": float(emotion_out.get("joy", 0.0)),
		"trust": float(emotion_out.get("trust", 0.0)),
		"anticipation": float(emotion_out.get("anticipation", 0.0)),
		"va_composite": float(emotion_out.get("va_composite", 0.0)),
		"emotion_total": float(emotion_out.get("total", 0.0)),
		"recovery": recovery,
		"delta": float(delta_out.get("delta", 0.0)),
		"hidden_threat_accumulator": float(delta_out.get("hidden_threat_accumulator", hidden_threat_accumulator)),
	}


## Combined step:
## trace decay batch + emotion/recovery/delta.
## Returns Dictionary:
## { total_trace_contribution, updated_per_tick, active_mask,
##   fear, anger, sadness, disgust, surprise, joy, trust, anticipation,
##   va_composite, emotion_total, recovery, delta, hidden_threat_accumulator }
static func stress_trace_emotion_recovery_delta_step(
	per_tick: PackedFloat32Array,
	decay_rate: PackedFloat32Array,
	min_keep: float,
	fear: float,
	anger: float,
	sadness: float,
	disgust: float,
	surprise: float,
	joy: float,
	trust: float,
	anticipation: float,
	valence: float,
	arousal: float,
	stress: float,
	support_score: float,
	resilience: float,
	reserve: float,
	is_sleeping: bool,
	is_safe: bool,
	continuous_input: float,
	ace_stress_mult: float,
	trait_accum_mult: float,
	epsilon: float,
	denial_active: bool,
	denial_redirect_fraction: float,
	hidden_threat_accumulator: float,
	denial_max_accumulator: float
) -> Dictionary:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_trace_emotion_recovery_delta_step",
		[
			per_tick,
			decay_rate,
			min_keep,
			fear,
			anger,
			sadness,
			disgust,
			surprise,
			joy,
			trust,
			anticipation,
			valence,
			arousal,
			stress,
			support_score,
			resilience,
			reserve,
			is_sleeping,
			is_safe,
			continuous_input,
			ace_stress_mult,
			trait_accum_mult,
			epsilon,
			denial_active,
			denial_redirect_fraction,
			hidden_threat_accumulator,
			denial_max_accumulator
		]
	)
	if rust_result is Dictionary:
		return rust_result

	var trace_out: Dictionary = stress_trace_batch_step(per_tick, decay_rate, min_keep)
	var erd_out: Dictionary = stress_emotion_recovery_delta_step(
		fear,
		anger,
		sadness,
		disgust,
		surprise,
		joy,
		trust,
		anticipation,
		valence,
		arousal,
		stress,
		support_score,
		resilience,
		reserve,
		is_sleeping,
		is_safe,
		continuous_input,
		float(trace_out.get("total_contribution", 0.0)),
		ace_stress_mult,
		trait_accum_mult,
		epsilon,
		denial_active,
		denial_redirect_fraction,
		hidden_threat_accumulator,
		denial_max_accumulator
	)
	return {
		"total_trace_contribution": float(trace_out.get("total_contribution", 0.0)),
		"updated_per_tick": trace_out.get("updated_per_tick", PackedFloat32Array()),
		"active_mask": trace_out.get("active_mask", PackedByteArray()),
		"fear": float(erd_out.get("fear", 0.0)),
		"anger": float(erd_out.get("anger", 0.0)),
		"sadness": float(erd_out.get("sadness", 0.0)),
		"disgust": float(erd_out.get("disgust", 0.0)),
		"surprise": float(erd_out.get("surprise", 0.0)),
		"joy": float(erd_out.get("joy", 0.0)),
		"trust": float(erd_out.get("trust", 0.0)),
		"anticipation": float(erd_out.get("anticipation", 0.0)),
		"va_composite": float(erd_out.get("va_composite", 0.0)),
		"emotion_total": float(erd_out.get("emotion_total", 0.0)),
		"recovery": float(erd_out.get("recovery", 0.0)),
		"delta": float(erd_out.get("delta", 0.0)),
		"hidden_threat_accumulator": float(erd_out.get("hidden_threat_accumulator", hidden_threat_accumulator)),
	}


## Full stress tick step:
## primary + trace/emotion/recovery/delta + post/resilience.
static func stress_tick_step(
	per_tick: PackedFloat32Array,
	decay_rate: PackedFloat32Array,
	min_keep: float,
	scalar_inputs: PackedFloat32Array,
	flags: PackedByteArray
) -> Dictionary:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_tick_step",
		[
			per_tick,
			decay_rate,
			min_keep,
			scalar_inputs,
			flags
		]
	)
	if rust_result is Dictionary:
		return rust_result

	var sf = func(idx: int, fallback: float) -> float:
		return float(scalar_inputs[idx]) if idx < scalar_inputs.size() else fallback
	var bf = func(idx: int) -> bool:
		return int(flags[idx]) != 0 if idx < flags.size() else false

	var primary: Dictionary = stress_primary_step(
		sf.call(0, 0.5),
		sf.call(1, 0.5),
		sf.call(2, 0.5),
		sf.call(3, 0.0),
		sf.call(4, 0.0),
		sf.call(5, 0.3),
		sf.call(6, 0.5),
		sf.call(7, 0.0),
		sf.call(8, 0.0),
		sf.call(9, 0.5),
		sf.call(10, 0.5),
		sf.call(11, 0.5)
	)
	var tr: Dictionary = stress_trace_emotion_recovery_delta_step(
		per_tick,
		decay_rate,
		min_keep,
		sf.call(7, 0.0),
		sf.call(12, 0.0),
		sf.call(13, 0.0),
		sf.call(14, 0.0),
		sf.call(15, 0.0),
		sf.call(16, 0.0),
		sf.call(8, 0.0),
		sf.call(17, 0.0),
		sf.call(18, 0.0),
		sf.call(19, 0.0),
		sf.call(20, 0.0),
		sf.call(5, 0.3),
		sf.call(21, 0.5),
		sf.call(22, 50.0),
		bf.call(0),
		bf.call(1),
		float(primary.get("total", 0.0)),
		sf.call(26, 1.0),
		sf.call(27, 1.0),
		sf.call(28, 0.05),
		bf.call(2),
		sf.call(29, 0.6),
		sf.call(30, 0.0),
		sf.call(31, 800.0)
	)
	var next_stress: float = clampf(sf.call(20, 0.0) + float(tr.get("delta", 0.0)), 0.0, 2000.0)
	var post: Dictionary = stress_post_update_resilience_step(
		sf.call(22, 50.0),
		next_stress,
		sf.call(21, 0.5),
		float(tr.get("delta", 0.0)),
		int(round(sf.call(24, 0.0))),
		bf.call(0),
		sf.call(25, 0.0),
		sf.call(32, 1.0),
		sf.call(33, 0.5),
		sf.call(34, 0.5),
		sf.call(35, 0.5),
		sf.call(36, 0.5),
		sf.call(37, 0.5),
		sf.call(38, 0.5),
		sf.call(5, 0.3),
		sf.call(0, 0.5),
		sf.call(1, 0.5),
		sf.call(39, 0.0)
	)

	return {
		"appraisal_scale": float(primary.get("appraisal_scale", 1.0)),
		"hunger": float(primary.get("hunger", 0.0)),
		"energy_deficit": float(primary.get("energy_deficit", 0.0)),
		"social_isolation": float(primary.get("social_isolation", 0.0)),
		"continuous_total": float(primary.get("total", 0.0)),
		"total_trace_contribution": float(tr.get("total_trace_contribution", 0.0)),
		"updated_per_tick": tr.get("updated_per_tick", PackedFloat32Array()),
		"active_mask": tr.get("active_mask", PackedByteArray()),
		"fear": float(tr.get("fear", 0.0)),
		"anger": float(tr.get("anger", 0.0)),
		"sadness": float(tr.get("sadness", 0.0)),
		"disgust": float(tr.get("disgust", 0.0)),
		"surprise": float(tr.get("surprise", 0.0)),
		"joy": float(tr.get("joy", 0.0)),
		"trust": float(tr.get("trust", 0.0)),
		"anticipation": float(tr.get("anticipation", 0.0)),
		"va_composite": float(tr.get("va_composite", 0.0)),
		"emotion_total": float(tr.get("emotion_total", 0.0)),
		"recovery": float(tr.get("recovery", 0.0)),
		"delta": float(tr.get("delta", 0.0)),
		"hidden_threat_accumulator": float(tr.get("hidden_threat_accumulator", sf.call(30, 0.0))),
		"stress": next_stress,
		"reserve": float(post.get("reserve", sf.call(22, 50.0))),
		"gas_stage": int(post.get("gas_stage", int(round(sf.call(24, 0.0))))),
		"allostatic": float(post.get("allostatic", sf.call(25, 0.0))),
		"stress_state": int(post.get("stress_state", 0)),
		"stress_mu_sadness": float(post.get("stress_mu_sadness", 0.0)),
		"stress_mu_anger": float(post.get("stress_mu_anger", 0.0)),
		"stress_mu_fear": float(post.get("stress_mu_fear", 0.0)),
		"stress_mu_joy": float(post.get("stress_mu_joy", 0.0)),
		"stress_mu_trust": float(post.get("stress_mu_trust", 0.0)),
		"stress_neg_gain_mult": float(post.get("stress_neg_gain_mult", 1.0)),
		"stress_pos_gain_mult": float(post.get("stress_pos_gain_mult", 1.0)),
		"stress_blunt_mult": float(post.get("stress_blunt_mult", 1.0)),
		"resilience": float(post.get("resilience", sf.call(21, 0.5))),
	}


## Combined post-stress step:
## reserve + GAS transition + allostatic update + stress state snapshot
static func stress_post_update_step(
	reserve: float,
	stress: float,
	resilience: float,
	stress_delta_last: float,
	gas_stage: int,
	is_sleeping: bool,
	allostatic: float,
	avoidant_allostatic_mult: float
) -> Dictionary:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_post_update_step",
		[
			reserve,
			stress,
			resilience,
			stress_delta_last,
			gas_stage,
			is_sleeping,
			allostatic,
			avoidant_allostatic_mult
		]
	)
	if rust_result is Dictionary:
		return rust_result

	var reserve_step: Dictionary = stress_reserve_step(
		reserve,
		stress,
		resilience,
		stress_delta_last,
		gas_stage,
		is_sleeping
	)
	var next_allostatic: float = stress_allostatic_step(allostatic, stress, avoidant_allostatic_mult)
	var snapshot: Dictionary = stress_state_snapshot(stress, next_allostatic)
	snapshot["reserve"] = float(reserve_step.get("reserve", reserve))
	snapshot["gas_stage"] = int(reserve_step.get("gas_stage", gas_stage))
	snapshot["allostatic"] = next_allostatic
	return snapshot


## Combined post-stress step with resilience update.
## Returns Dictionary:
## reserve, gas_stage, allostatic, stress_state, stress_mu_*, stress_*_gain_mult, stress_blunt_mult, resilience
static func stress_post_update_resilience_step(
	reserve: float,
	stress: float,
	resilience: float,
	stress_delta_last: float,
	gas_stage: int,
	is_sleeping: bool,
	allostatic: float,
	avoidant_allostatic_mult: float,
	e_axis: float,
	c_axis: float,
	x_axis: float,
	o_axis: float,
	a_axis: float,
	h_axis: float,
	support_score: float,
	hunger: float,
	energy: float,
	scar_resilience_mod: float
) -> Dictionary:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_post_update_resilience_step",
		[
			reserve,
			stress,
			resilience,
			stress_delta_last,
			gas_stage,
			is_sleeping,
			allostatic,
			avoidant_allostatic_mult,
			e_axis,
			c_axis,
			x_axis,
			o_axis,
			a_axis,
			h_axis,
			support_score,
			hunger,
			energy,
			scar_resilience_mod
		]
	)
	if rust_result is Dictionary:
		return rust_result

	var out: Dictionary = stress_post_update_step(
		reserve,
		stress,
		resilience,
		stress_delta_last,
		gas_stage,
		is_sleeping,
		allostatic,
		avoidant_allostatic_mult
	)
	var next_resilience: float = stress_resilience_value(
		e_axis,
		c_axis,
		x_axis,
		o_axis,
		a_axis,
		h_axis,
		support_score,
		float(out.get("allostatic", allostatic)),
		hunger,
		energy,
		scar_resilience_mod
	)
	out["resilience"] = next_resilience
	return out


## Reserve + GAS stage transition step.
## Returns Dictionary: { "reserve": float, "gas_stage": int }
static func stress_reserve_step(
	reserve: float,
	stress: float,
	resilience: float,
	stress_delta_last: float,
	gas_stage: int,
	is_sleeping: bool
) -> Dictionary:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_reserve_step",
		[
			reserve,
			stress,
			resilience,
			stress_delta_last,
			gas_stage,
			is_sleeping
		]
	)
	if rust_result is Dictionary:
		return rust_result

	var drain: float = maxf(0.0, (stress - 150.0) / 350.0) * (0.7 + 0.6 * (1.0 - resilience))
	var recover_base: float = 0.4 + 0.6 * resilience
	var recover: float = recover_base * (1.0 if is_sleeping else 0.15)
	var next_reserve: float = clampf(reserve - drain + recover, 0.0, 100.0)

	var next_stage: int = gas_stage
	if (stress_delta_last > 40.0 or stress > 400.0) and next_stage == 0:
		next_stage = 1
	if next_reserve >= 30.0 and stress < 500.0 and next_stage == 1:
		next_stage = 2
	if next_reserve < 30.0:
		next_stage = 3

	return {
		"reserve": next_reserve,
		"gas_stage": next_stage,
	}


## Allostatic load update step.
static func stress_allostatic_step(
	allostatic: float,
	stress: float,
	avoidant_allostatic_mult: float
) -> float:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_allostatic_step",
		[
			allostatic,
			stress,
			avoidant_allostatic_mult
		]
	)
	if rust_result != null:
		return float(rust_result)

	var next: float = allostatic
	if stress > 250.0:
		var allo_inc: float = 0.035 * maxf(0.0, stress - 250.0) / 250.0
		allo_inc = minf(allo_inc, 0.05)
		next = clampf(next + allo_inc * avoidant_allostatic_mult, 0.0, 100.0)
	if stress < 120.0:
		next = clampf(next - 0.003, 0.0, 100.0)
	return next


## Stress state bucket + stress-driven emotion meta snapshot.
## Returns Dictionary with:
## stress_state, stress_mu_*, stress_neg_gain_mult, stress_pos_gain_mult, stress_blunt_mult
static func stress_state_snapshot(
	stress: float,
	allostatic: float
) -> Dictionary:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_state_snapshot",
		[
			stress,
			allostatic
		]
	)
	if rust_result is Dictionary:
		return rust_result

	var stress_state: int = 0
	if stress >= 500.0:
		stress_state = 3
	elif stress >= 350.0:
		stress_state = 2
	elif stress >= 200.0:
		stress_state = 1

	var s1: float = clampf((stress - 100.0) / 400.0, 0.0, 1.0)
	var s2: float = clampf((stress - 300.0) / 400.0, 0.0, 1.0)
	var allo_ratio: float = allostatic / 100.0

	var stress_mu_sadness: float = 6.0 * s1 + 10.0 * allo_ratio
	var stress_mu_anger: float = 4.0 * s1 + 8.0 * allo_ratio
	var stress_mu_fear: float = 5.0 * s1 + 12.0 * allo_ratio
	var stress_mu_joy: float = -(5.0 * s1 + 12.0 * allo_ratio)
	var stress_mu_trust: float = -(4.0 * s1 + 10.0 * allo_ratio)

	var stress_neg_gain_mult: float = 1.0 + 0.7 * s2
	var stress_pos_gain_mult: float = 1.0 - 0.5 * s2
	var blunt_denom: float = 1.0 + 0.9 * allo_ratio * (2.0 if allo_ratio > 0.6 else 1.0)
	var stress_blunt_mult: float = 1.0 / blunt_denom

	return {
		"stress_state": stress_state,
		"stress_mu_sadness": stress_mu_sadness,
		"stress_mu_anger": stress_mu_anger,
		"stress_mu_fear": stress_mu_fear,
		"stress_mu_joy": stress_mu_joy,
		"stress_mu_trust": stress_mu_trust,
		"stress_neg_gain_mult": stress_neg_gain_mult,
		"stress_pos_gain_mult": stress_pos_gain_mult,
		"stress_blunt_mult": stress_blunt_mult,
	}


## Batch step for stress traces.
## Input arrays must be aligned by index.
## Returns Dictionary:
## { "total_contribution": float, "updated_per_tick": PackedFloat32Array, "active_mask": PackedByteArray }
static func stress_trace_batch_step(
	per_tick: PackedFloat32Array,
	decay_rate: PackedFloat32Array,
	min_keep: float = 0.01
) -> Dictionary:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_trace_batch_step",
		[
			per_tick,
			decay_rate,
			min_keep
		]
	)
	if rust_result is Dictionary:
		return rust_result

	var len: int = mini(per_tick.size(), decay_rate.size())
	var updated: PackedFloat32Array = PackedFloat32Array()
	var active: PackedByteArray = PackedByteArray()
	updated.resize(len)
	active.resize(len)
	var total: float = 0.0
	for i in range(len):
		var contribution: float = float(per_tick[i])
		total += contribution
		var next: float = contribution * (1.0 - float(decay_rate[i]))
		updated[i] = next
		active[i] = 1 if next >= min_keep else 0
	return {
		"total_contribution": total,
		"updated_per_tick": updated,
		"active_mask": active,
	}


## Resilience value update.
static func stress_resilience_value(
	e_axis: float,
	c_axis: float,
	x_axis: float,
	o_axis: float,
	a_axis: float,
	h_axis: float,
	support_score: float,
	allostatic: float,
	hunger: float,
	energy: float,
	scar_resilience_mod: float
) -> float:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_resilience_value",
		[
			e_axis,
			c_axis,
			x_axis,
			o_axis,
			a_axis,
			h_axis,
			support_score,
			allostatic,
			hunger,
			energy,
			scar_resilience_mod
		]
	)
	if rust_result != null:
		return float(rust_result)

	var r: float = (
		0.35 * (1.0 - e_axis)
		+ 0.25 * c_axis
		+ 0.15 * x_axis
		+ 0.10 * o_axis
		+ 0.10 * a_axis
		+ 0.05 * h_axis
		+ 0.25 * support_score
		- 0.30 * (allostatic / 100.0)
	)
	var fatigue_penalty: float = clampf((0.3 - energy) / 0.3, 0.0, 0.3) + clampf((0.3 - hunger) / 0.3, 0.0, 0.2)
	r -= 0.20 * fatigue_penalty
	r += scar_resilience_mod
	return clampf(r, 0.05, 1.0)


## Yerkes-Dodson work efficiency multiplier.
static func stress_work_efficiency(
	stress: float,
	shaken_penalty: float
) -> float:
	var rust_result: Variant = _call_sim_bridge(
		"stat_stress_work_efficiency",
		[
			stress,
			shaken_penalty
		]
	)
	if rust_result != null:
		return float(rust_result)

	var perf: float
	if stress < 150.0:
		perf = 1.0 + 0.0006 * stress
	elif stress < 350.0:
		perf = 1.09 - 0.0004 * (stress - 150.0)
	else:
		perf = 1.01 - 0.0012 * (stress - 350.0)
	perf += shaken_penalty
	return clampf(perf, 0.35, 1.10)

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# INFLUENCE CURVES
# 반환값: float 배수 (1.0 = 중립, >1.0 = 증폭, <1.0 = 감쇠)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## SIGMOID_EXTREME: 성격 → 행동
## "평범한 사람들(200~800)은 거의 같다. 진짜 극단만 질적으로 다르다."
static func sigmoid_extreme(value: int, params: Dictionary) -> float:
	var flat_lo: float = float(params.get("flat_zone", [200, 800])[0]) / 1000.0
	var flat_hi: float = float(params.get("flat_zone", [200, 800])[1]) / 1000.0
	var pole: float = float(params.get("pole_multiplier", 3.0))
	var rust_result: Variant = _call_sim_bridge(
		"stat_sigmoid_extreme",
		[
			value,
			int(round(flat_lo * 1000.0)),
			int(round(flat_hi * 1000.0)),
			pole
		]
	)
	if rust_result != null:
		return float(rust_result)
	var norm: float = float(value) / 1000.0

	if norm >= flat_lo and norm <= flat_hi:
		var t_mid: float = (norm - flat_lo) / (flat_hi - flat_lo)
		return lerpf(0.7, 1.3, t_mid)
	elif norm < flat_lo:
		var t_lo: float = 1.0 - (norm / flat_lo)
		var bottom: float = 1.0 / pole
		return maxf(bottom, lerpf(0.7, bottom, pow(t_lo, 1.5)))
	else:
		var t_hi: float = (norm - flat_hi) / (1.0 - flat_hi)
		return minf(pole, lerpf(1.3, pole, pow(t_hi, 1.5)))

## POWER: 신체 → 전투/작업
## effect = (value / 1000)^exponent
static func power_influence(value: int, params: Dictionary) -> float:
	var exponent: float = float(params.get("exponent", 1.5))
	var rust_result: Variant = _call_sim_bridge("stat_power_influence", [value, exponent])
	if rust_result != null:
		return float(rust_result)
	return pow(float(value) / 1000.0, exponent)

## THRESHOLD_POWER: 임계값 이하에서만 활성화
## "욕구 결핍 → 스트레스: 결핍 없으면 0, 결핍 심할수록 지수적 증가"
static func threshold_power(value: int, params: Dictionary) -> float:
	var threshold: int = int(params.get("threshold", 350))
	var exponent: float = float(params.get("exponent", 2.0))
	var max_output: float = float(params.get("max_output", 12.0))
	var rust_result: Variant = _call_sim_bridge(
		"stat_threshold_power",
		[
			value,
			threshold,
			exponent,
			max_output
		]
	)
	if rust_result != null:
		return float(rust_result)
	if value >= threshold:
		return 0.0
	var deficit: float = float(threshold - value) / float(threshold)
	return minf(max_output, pow(deficit, exponent) * max_output)

## LINEAR: 단순 비례
static func linear_influence(value: int, _params: Dictionary) -> float:
	var rust_result: Variant = _call_sim_bridge("stat_linear_influence", [value])
	if rust_result != null:
		return float(rust_result)
	return float(value) / 1000.0

## STEP: 임계값 이분
static func step_influence(value: int, params: Dictionary) -> float:
	var threshold: int = int(params.get("threshold", 500))
	var above: float = float(params.get("above_value", 1.0))
	var below: float = float(params.get("below_value", 0.0))
	var rust_result: Variant = _call_sim_bridge(
		"stat_step_influence",
		[
			value,
			threshold,
			above,
			below
		]
	)
	if rust_result != null:
		return float(rust_result)
	return above if value >= threshold else below

## STEP_LINEAR: 단계별 선형 보간
static func step_linear(value: int, params: Dictionary) -> float:
	var steps: Array = params.get("steps", [])
	var below_thresholds: PackedInt32Array = params.get("_step_below_thresholds_packed", PackedInt32Array())
	var multipliers: PackedFloat32Array = params.get("_step_multipliers_packed", PackedFloat32Array())
	if below_thresholds.size() != steps.size() or multipliers.size() != steps.size():
		below_thresholds.resize(steps.size())
		multipliers.resize(steps.size())
		for i in range(steps.size()):
			var step: Dictionary = steps[i]
			below_thresholds[i] = int(step.get("below", 0))
			multipliers[i] = float(step.get("multiply", 1.0))
		params["_step_below_thresholds_packed"] = below_thresholds
		params["_step_multipliers_packed"] = multipliers
	var rust_result: Variant = _call_sim_bridge(
		"stat_step_linear",
		[
			value,
			below_thresholds,
			multipliers
		]
	)
	if rust_result != null:
		return float(rust_result)

	var result: float = 1.0
	for step in steps:
		if value < int(step.get("below", 0)):
			result = float(step.get("multiply", 1.0))
	return result

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# DISPATCH
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## affect entry의 curve 필드에 따라 올바른 커브 함수를 실행
## 반환: float 배수
static func apply(value: int, affect: Dictionary) -> float:
	var curve: String = affect.get("curve", "LINEAR")
	var params: Dictionary = affect.get("params", {})
	match curve:
		"SIGMOID_EXTREME":   return sigmoid_extreme(value, params)
		"POWER":             return power_influence(value, params)
		"THRESHOLD_POWER":   return threshold_power(value, params)
		"STEP":              return step_influence(value, params)
		"STEP_LINEAR":       return step_linear(value, params)
		"LINEAR":            return linear_influence(value, params)
		_:
			push_error("StatCurve: unknown curve type: " + curve)
			return 1.0


static func _to_packed_i32(values: Array) -> PackedInt32Array:
	var packed: PackedInt32Array = PackedInt32Array()
	packed.resize(values.size())
	for i in range(values.size()):
		packed[i] = int(values[i])
	return packed


static func _to_packed_f32(values: Array) -> PackedFloat32Array:
	var packed: PackedFloat32Array = PackedFloat32Array()
	packed.resize(values.size())
	for i in range(values.size()):
		packed[i] = float(values[i])
	return packed


static func _xp_curve_cache_key(bp: Array, bm: Array) -> String:
	return JSON.stringify(bp) + "|" + JSON.stringify(bm)


static func _get_xp_curve_meta(params: Dictionary) -> Dictionary:
	var base: float = float(params.get("base_xp", 100.0))
	var exponent: float = float(params.get("exponent", 1.8))
	var bp: Array = params.get("level_breakpoints", [])
	var bm: Array = params.get("breakpoint_multipliers", [])
	var cache_key: String = _xp_curve_cache_key(bp, bm)

	var cached: Dictionary = {}
	if _xp_curve_cache.has(cache_key):
		cached = _xp_curve_cache[cache_key]
	else:
		cached = {
			"breakpoints": bp.duplicate(),
			"multipliers": bm.duplicate(),
			"breakpoints_packed": _to_packed_i32(bp),
			"multipliers_packed": _to_packed_f32(bm),
		}
		if _xp_curve_cache.size() >= _XP_CURVE_CACHE_MAX:
			_xp_curve_cache.clear()
		_xp_curve_cache[cache_key] = cached

	return {
		"base": base,
		"exponent": exponent,
		"breakpoints": cached.get("breakpoints", []),
		"multipliers": cached.get("multipliers", []),
		"breakpoints_packed": cached.get("breakpoints_packed", PackedInt32Array()),
		"multipliers_packed": cached.get("multipliers_packed", PackedFloat32Array()),
	}


static func _get_sim_bridge() -> Object:
	if _bridge_checked:
		return _bridge_ref
	_bridge_checked = true

	var tree: SceneTree = Engine.get_main_loop() as SceneTree
	if tree != null and tree.root != null:
		var node_from_root: Node = tree.root.get_node_or_null(_SIM_BRIDGE_NODE_NAME)
		if node_from_root != null:
			_bridge_ref = node_from_root
			return _bridge_ref

	if Engine.has_singleton(_SIM_BRIDGE_NODE_NAME):
		_bridge_ref = Engine.get_singleton(_SIM_BRIDGE_NODE_NAME)

	return _bridge_ref


static func _call_sim_bridge(method_name: String, args: Array):
	var bridge: Object = _get_sim_bridge()
	if bridge == null:
		return null
	if not bridge.has_method(method_name):
		return null
	return bridge.callv(method_name, args)
