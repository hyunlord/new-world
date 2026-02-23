extends RefCounted

## 모든 Growth / Influence 커브의 수학 구현체.
## 순수 함수만 포함. 상태 없음. Rust 전환 시 이 파일만 교체.
##
## Growth Curve: XP/이벤트 → 스탯 변화량 계산
## Influence Curve: 스탯값 → 다른 스탯/파라미터에 주는 영향력 계산

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GROWTH CURVES
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

## LOG_DIMINISHING: 스킬, 신체 훈련
## "처음은 빠르게, 올라갈수록 점점 더 많은 XP 필요"
## 반환: 다음 레벨까지 필요한 XP (float)
static func log_xp_required(level: int, params: Dictionary) -> float:
	if level <= 0:
		return 0.0
	var base: float = float(params.get("base_xp", 100.0))
	var exponent: float = float(params.get("exponent", 1.8))
	var bp: Array = params.get("level_breakpoints", [])
	var bm: Array = params.get("breakpoint_multipliers", [])
	var mult: float = bm[0] if bm.size() > 0 else 1.0
	for i in range(bp.size()):
		if level >= bp[i] and i + 1 < bm.size():
			mult = bm[i + 1]
	return base * pow(float(level), exponent) * mult

## LOG_DIMINISHING의 역함수: 누적 XP → 현재 레벨
static func xp_to_level(xp: float, params: Dictionary, max_level: int = 100) -> int:
	var cumulative: float = 0.0
	for lv in range(1, max_level + 1):
		cumulative += log_xp_required(lv, params)
		if cumulative > xp:
			return lv - 1
	return max_level

## SCURVE: 성격, 가치관
## "초반은 빠르게 형성, 중반에 굳어짐, 후반은 거의 불변"
## 반환: 변화 속도 배수 (float)
static func scurve_speed(current_value: int, params: Dictionary) -> float:
	var bps: Array = params.get("phase_breakpoints", [300, 700])
	var speeds: Array = params.get("phase_speeds", [1.5, 1.0, 0.3])
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
	var decay_per_tick: float = float(decay_per_year) / float(ticks_per_year)
	var total_decay: float = decay_per_tick * float(ticks_elapsed) * metabolic_mult
	return clampi(current - int(total_decay), 0, 1000)

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
	var norm: float = float(value) / 1000.0

	if norm >= flat_lo and norm <= flat_hi:
		var t: float = (norm - flat_lo) / (flat_hi - flat_lo)
		return lerpf(0.7, 1.3, t)
	elif norm < flat_lo:
		var t: float = 1.0 - (norm / flat_lo)
		var bottom: float = 1.0 / pole
		return maxf(bottom, lerpf(0.7, bottom, pow(t, 1.5)))
	else:
		var t: float = (norm - flat_hi) / (1.0 - flat_hi)
		return minf(pole, lerpf(1.3, pole, pow(t, 1.5)))

## POWER: 신체 → 전투/작업
## effect = (value / 1000)^exponent
static func power_influence(value: int, params: Dictionary) -> float:
	var exponent: float = float(params.get("exponent", 1.5))
	return pow(float(value) / 1000.0, exponent)

## THRESHOLD_POWER: 임계값 이하에서만 활성화
## "욕구 결핍 → 스트레스: 결핍 없으면 0, 결핍 심할수록 지수적 증가"
static func threshold_power(value: int, params: Dictionary) -> float:
	var threshold: int = int(params.get("threshold", 350))
	var exponent: float = float(params.get("exponent", 2.0))
	var max_output: float = float(params.get("max_output", 12.0))
	if value >= threshold:
		return 0.0
	var deficit: float = float(threshold - value) / float(threshold)
	return minf(max_output, pow(deficit, exponent) * max_output)

## LINEAR: 단순 비례
static func linear_influence(value: int, _params: Dictionary) -> float:
	return float(value) / 1000.0

## STEP: 임계값 이분
static func step_influence(value: int, params: Dictionary) -> float:
	var threshold: int = int(params.get("threshold", 500))
	var above: float = float(params.get("above_value", 1.0))
	var below: float = float(params.get("below_value", 0.0))
	return above if value >= threshold else below

## STEP_LINEAR: 단계별 선형 보간
static func step_linear(value: int, params: Dictionary) -> float:
	var steps: Array = params.get("steps", [])
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
