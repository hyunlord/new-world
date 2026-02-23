## [Layer 1.5] 신체 능력치 — potential/trainability/realized 3-레이어
## [HERITAGE 1999] trainability h²=0.47, [Ahtiainen 2016] 개인차 8.5배
## [ACTN3/ACE] STR↔END 역상관, [Weaver 2016] TOU 아동기 민감기
## 참조: const BodyAttributes = preload("res://scripts/core/body_attributes.gd")
extends RefCounted

## ── 유전 기반 (태생 결정, 불변) ───────────────────────────
## potential: 0~10,000 int. 훈련 없이 나이 커브만 적용한 기준값
var potential: Dictionary = {}
## {"str":int, "agi":int, "end":int, "tou":int, "rec":int, "dr":int}

## trainability: 0~1,000 int. 훈련 반응성 재능 (DR 제외 5축)
var trainability: Dictionary = {}
## {"str":int, "agi":int, "end":int, "tou":int, "rec":int}

## innate_immunity: 0~1,000 int. DR 대체 (선천 면역, 불변)
var innate_immunity: int = 500

## ── 누적 훈련 ────────────────────────────────────────────
var training_xp: Dictionary = {}
## {"str":float, "agi":float, "end":float, "tou":float, "rec":float}

## ── 아동기 환경 추적 (0~12세, age_system이 매년 갱신) ──
var child_nutrition_sum: float = 0.0
var child_nutrition_count: int = 0
var child_activity_sum: float = 0.0
var child_activity_count: int = 0
var childhood_finalized: bool = false

## ── 실현값 캐시 (매년 재계산) ─────────────────────────────
var realized: Dictionary = {}
## {"str":int, "agi":int, "end":int, "tou":int, "rec":int, "dr":int}

const CURVE_PARAMS: Dictionary = {
	"str": { "a50": 16.0, "k": 0.35, "t0": 35.0, "r1": 0.007, "t1": 70.0, "r2": 0.030 },
	"agi": { "a50": 14.0, "k": 0.45, "t0": 25.0, "r1": 0.009, "t1": 65.0, "r2": 0.035 },
	"end": { "a50": 15.0, "k": 0.38, "t0": 30.0, "r1": 0.008, "t1": 70.0, "r2": 0.020 },
	"tou": { "a50": 17.0, "k": 0.32, "t0": 40.0, "r1": 0.007, "t1": 75.0, "r2": 0.020 },
	"rec": { "a50": 12.0, "k": 0.50, "t0": 20.0, "r1": 0.011, "t1": 60.0, "r2": 0.030 },
	"dr":  { "a50":  6.0, "k": 0.90, "t0": 55.0, "r1": 0.010, "t1": 75.0, "r2": 0.030 },
}

## 훈련 상한 배수 (potential 대비 최대 추가량)
## STR: 파워리프팅+250%, END: VO2max elite+150%, AGI: 유전율 0.80→+30%
## TOU: BMD+20%, REC: HRV+60%
const TRAINING_CEILING: Dictionary = {
	"str": 2.50,
	"end": 1.50,
	"agi": 0.30,
	"tou": 0.20,
	"rec": 0.60,
}

## 단일 축 나이 커브 계산 (0.02 ~ 1.0)
## grow(로지스틱) × decl1(초중년 감쇠) × decl2(노년 가속 감쇠)
static func compute_age_curve(axis: String, age_years: float) -> float:
	if not CURVE_PARAMS.has(axis):
		return 0.5
	var p: Dictionary = CURVE_PARAMS[axis]
	var grow: float = 1.0 / (1.0 + exp(-p["k"] * (age_years - p["a50"])))
	var decl1: float = exp(-p["r1"] * maxf(age_years - p["t0"], 0.0))
	var decl2: float = exp(-p["r2"] * maxf(age_years - p["t1"], 0.0))
	var raw: float = clampf(grow * decl1 * decl2, 0.02, 1.0)
	if axis == "dr":
		var maternal_bonus: float = 0.20 * exp(-age_years / 0.5)
		return clampf(raw + maternal_bonus, 0.02, 1.0)
	return raw

## 나이별 훈련 효율 배수 반환 (0.0~1.0)
## 과거 축적 gain은 유지됨 — 오직 "지금 훈련 효과"만 결정
static func get_age_trainability_modifier(axis: String, age_years: float) -> float:
	match axis:
		"str":
			if age_years < 13.0: return 0.60
			elif age_years < 18.0: return 0.85
			elif age_years < 31.0: return 1.00
			elif age_years < 51.0: return 0.85
			elif age_years < 66.0: return 0.65
			elif age_years < 81.0: return 0.45
			else: return 0.30
		"end":
			if age_years < 13.0: return 0.70
			elif age_years < 18.0: return 0.90
			elif age_years < 31.0: return 1.00
			elif age_years < 51.0: return 0.90
			elif age_years < 66.0: return 0.75
			elif age_years < 81.0: return 0.55
			else: return 0.35
		"agi":
			if age_years < 7.0: return 0.70
			elif age_years < 14.0: return 1.00
			elif age_years < 18.0: return 0.85
			elif age_years < 31.0: return 0.80
			elif age_years < 51.0: return 0.65
			else: return 0.45
		"tou":
			if age_years < 13.0: return 1.00
			elif age_years < 26.0: return 0.90
			elif age_years < 41.0: return 0.50
			elif age_years < 61.0: return 0.30
			else: return 0.15
		"rec":
			if age_years < 13.0: return 0.60
			elif age_years < 18.0: return 0.85
			elif age_years < 31.0: return 1.00
			elif age_years < 51.0: return 0.80
			elif age_years < 66.0: return 0.60
			elif age_years < 81.0: return 0.40
			else: return 0.25
	return 1.00

## 훈련으로 추가된 능력치 반환 (int)
## DR은 훈련 미적용 (innate_immunity 고정)
func calc_training_gain(axis: String) -> int:
	if axis == "dr" or not trainability.has(axis):
		return 0
	var pot: int = potential.get(axis, 700)
	var max_gain: float = float(pot) * TRAINING_CEILING.get(axis, 0.5)
	var xp: float = training_xp.get(axis, 0.0)
	var xp_progress: float = clampf(xp / GameConfig.XP_FOR_FULL_PROGRESS, 0.0, 1.0)
	var xp_factor: float = 1.0 - exp(-3.0 * xp_progress)
	var train_factor: float = clampf(float(trainability.get(axis, 500)) / 500.0, 0.1, 2.0)
	var gain: float = max_gain * xp_factor * train_factor
	return clampi(int(gain), 0, int(max_gain * 2.0))

## 아동기 환경 평균 → trainability 영구 수정
## age_system이 12세 도달 시 1회 호출
func finalize_childhood_environment() -> void:
	if childhood_finalized:
		return
	childhood_finalized = true
	var avg_nutrition: float = child_nutrition_sum / maxf(float(child_nutrition_count), 1.0)
	var avg_activity: float = child_activity_sum / maxf(float(child_activity_count), 1.0)

	var nutrition_mod: float = 1.0
	if avg_nutrition < 0.30:
		nutrition_mod = 1.0 - (0.30 - avg_nutrition) * 0.50

	var activity_mod: float = 1.0
	if avg_activity > 0.70:
		activity_mod = 1.0 + (avg_activity - 0.70) * 0.10

	var total_mod: float = clampf(nutrition_mod * activity_mod, 0.85, 1.15)
	if total_mod == 1.0:
		return

	for axis in trainability.keys():
		trainability[axis] = clampi(
			int(float(trainability[axis]) * total_mod),
			GameConfig.TRAINABILITY_MIN,
			GameConfig.TRAINABILITY_MAX
		)

func to_dict() -> Dictionary:
	return {
		"potential": potential.duplicate(),
		"trainability": trainability.duplicate(),
		"innate_immunity": innate_immunity,
		"training_xp": training_xp.duplicate(),
		"child_nutrition_sum": child_nutrition_sum,
		"child_nutrition_count": child_nutrition_count,
		"child_activity_sum": child_activity_sum,
		"child_activity_count": child_activity_count,
		"childhood_finalized": childhood_finalized,
		"realized": realized.duplicate(),
	}

static func from_dict(data: Dictionary):
	var b = load("res://scripts/core/body_attributes.gd").new()
	# Support old "potentials" key for save migration
	b.potential = data.get("potential", data.get("potentials", {}))
	b.trainability = data.get("trainability", {})
	b.innate_immunity = data.get("innate_immunity", 500)
	b.training_xp = data.get("training_xp", {})
	b.child_nutrition_sum = data.get("child_nutrition_sum", 0.0)
	b.child_nutrition_count = data.get("child_nutrition_count", 0)
	b.child_activity_sum = data.get("child_activity_sum", 0.0)
	b.child_activity_count = data.get("child_activity_count", 0)
	b.childhood_finalized = data.get("childhood_finalized", false)
	b.realized = data.get("realized", {})
	return b
