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
