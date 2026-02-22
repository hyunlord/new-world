## [Layer 1.5] 신체 능력치 6축 — 나이 커브 시스템
## [Eveleth & Tanner 1990] 성장 로지스틱, [Gurven et al. 2008] 감쇠 커브
## [Dodds et al. 2019] 피크 나이, [Vaupel et al. 1979] frailty 이질성
## 참조: const BodyAttributes = preload("res://scripts/core/body_attributes.gd")
extends RefCounted

## 유전적 잠재력 (생성 시 고정, 불변)
var potentials: Dictionary = {}
# {"str": float, "agi": float, "end": float, "tou": float, "rec": float, "dr": float}

## 발현 능력 (매년 재계산)
var realized: Dictionary = {}
# {"str": float, "agi": float, "end": float, "tou": float, "rec": float, "dr": float}

## ── 나이 커브 파라미터 ──────────────────────────────────
## a50: 성장 50% 도달 나이(세), k: 가파름
## t0: 1차 감쇠 시작, r1: 1차 감쇠율
## t1: 2차 감쇠 시작, r2: 2차 가속 감쇠율
const CURVE_PARAMS: Dictionary = {
	"str": { "a50": 16.0, "k": 0.35, "t0": 35.0, "r1": 0.007, "t1": 70.0, "r2": 0.030 },
	"agi": { "a50": 14.0, "k": 0.45, "t0": 25.0, "r1": 0.009, "t1": 65.0, "r2": 0.035 },
	"end": { "a50": 15.0, "k": 0.38, "t0": 30.0, "r1": 0.008, "t1": 70.0, "r2": 0.020 },
	"tou": { "a50": 17.0, "k": 0.32, "t0": 40.0, "r1": 0.007, "t1": 75.0, "r2": 0.020 },
	"rec": { "a50": 12.0, "k": 0.50, "t0": 20.0, "r1": 0.011, "t1": 60.0, "r2": 0.030 },
	"dr":  { "a50":  6.0, "k": 0.90, "t0": 55.0, "r1": 0.010, "t1": 75.0, "r2": 0.030 },
}

## 남성 potential 보정값 (여성 기준 = 0.0). 음수 = 여성이 높은 축.
const SEX_DELTA_MALE: Dictionary = {
	"str": +0.12,
	"agi": +0.03,
	"end": -0.02,
	"tou": +0.08,
	"rec": -0.02,
	"dr":  -0.07,
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

## 에이전트 생성 시 1회 호출. min(U,U) 분포로 potential 생성.
## frailty 반영(±15%) 후 성별 delta 적용, clamp [0.02, 1.0].
static func generate_potentials(
	rng: RandomNumberGenerator,
	is_male: bool,
	frailty: float
) -> Dictionary:
	var result: Dictionary = {}
	var frailty_offset: float = (frailty - 1.0) * 0.15
	for axis in CURVE_PARAMS:
		var r1: float = rng.randf()
		var r2: float = rng.randf()
		var base: float = minf(r1, r2)
		base += frailty_offset
		if is_male:
			base += SEX_DELTA_MALE[axis]
		result[axis] = clampf(base, 0.02, 1.0)
	return result

## 나이 변화 시 호출. realized = potential × age_curve, clamp [0.02, 1.0].
static func compute_realized(
	potentials: Dictionary,
	age_years: float,
	modifiers: Dictionary = {}
) -> Dictionary:
	var result: Dictionary = {}
	for axis in potentials:
		var curve_val: float = compute_age_curve(axis, age_years)
		result[axis] = clampf(potentials[axis] * curve_val, 0.02, 1.0)
	return result

## 직렬화
func to_dict() -> Dictionary:
	return {
		"potentials": potentials.duplicate(),
		"realized": realized.duplicate(),
	}

## 역직렬화
static func from_dict(data: Dictionary):
	var b = load("res://scripts/core/body_attributes.gd").new()
	b.potentials = data.get("potentials", {})
	b.realized = data.get("realized", {})
	return b
