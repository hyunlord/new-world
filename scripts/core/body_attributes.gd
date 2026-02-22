## [Layer 1.5] 신체 능력치 6축 — 나이 커브 시스템
## [Eveleth & Tanner 1990] 성장 로지스틱, [Gurven et al. 2008] 감쇠 커브
## [Dodds et al. 2019] 피크 나이, [Vaupel et al. 1979] frailty 이질성
## 참조: const BodyAttributes = preload("res://scripts/core/body_attributes.gd")
extends RefCounted

## ── 6축 필드 (0.0 ~ 1.0) ───────────────────────────────
var str_val: float = 0.5  # Strength (근력)
var agi: float = 0.5      # Agility (민첩)
var end_val: float = 0.5  # Endurance (지구력)
var tou: float = 0.5      # Toughness (강인함)
var rec: float = 0.5      # Recuperation (회복력)
var dr: float = 0.5       # Disease Resistance (면역력)

## ── 나이 커브 파라미터 ──────────────────────────────────
## a50: 성장 50% 도달 나이(세), k: 가파름
## t0: 1차 감쇠 시작, r1: 1차 감쇠율
## t1: 2차 감쇠 시작, r2: 2차 가속 감쇠율
const CURVE_PARAMS: Dictionary = {
	"str_val": { "a50": 16.0, "k": 0.35, "t0": 35.0, "r1": 0.007, "t1": 70.0, "r2": 0.030 },
	"agi":     { "a50": 14.0, "k": 0.45, "t0": 25.0, "r1": 0.009, "t1": 65.0, "r2": 0.035 },
	"end_val": { "a50": 15.0, "k": 0.38, "t0": 30.0, "r1": 0.008, "t1": 70.0, "r2": 0.020 },
	"tou":     { "a50": 17.0, "k": 0.32, "t0": 40.0, "r1": 0.007, "t1": 75.0, "r2": 0.020 },
	"rec":     { "a50": 12.0, "k": 0.50, "t0": 20.0, "r1": 0.011, "t1": 60.0, "r2": 0.030 },
	"dr":      { "a50":  6.0, "k": 0.90, "t0": 55.0, "r1": 0.010, "t1": 75.0, "r2": 0.030 },
}

## frailty 개인차 배수
## frailty=1.0(평균)→1.0, frailty=0.5(허약)→0.85, frailty=2.0(강건)→1.10
static func _frailty_scale(frailty: float) -> float:
	return clampf(0.70 + 0.30 * clampf(frailty / 1.0, 0.5, 1.5), 0.85, 1.10)

## 단일 축 나이 커브 계산 (0.05 ~ 1.0)
## grow(로지스틱) × decl1(초중년 감쇠) × decl2(노년 가속 감쇠)
static func compute_age_curve(axis: String, age_years: float) -> float:
	var p: Dictionary = CURVE_PARAMS[axis]
	var grow: float = 1.0 / (1.0 + exp(-p["k"] * (age_years - p["a50"])))
	var decl1: float = exp(-p["r1"] * maxf(age_years - p["t0"], 0.0))
	var decl2: float = exp(-p["r2"] * maxf(age_years - p["t1"], 0.0))
	var raw: float = clampf(grow * decl1 * decl2, 0.05, 1.0)
	if axis == "dr":
		var maternal_bonus: float = 0.20 * exp(-age_years / 0.5)
		raw = clampf(raw + maternal_bonus, 0.05, 1.0)
	return raw

## 전 축 계산 — BodyAttributes 인스턴스로 반환
## frailty: Vaupel(1979) 이질성 배수 [0.5, 2.0]
## sex_delta: 향후 성별 보정용 (현재 미적용)
static func compute_all(age_years: float, frailty: float, sex_delta: Dictionary = {}) -> Dictionary:
	var fs: float = _frailty_scale(frailty)
	var result: Dictionary = {}
	for axis in CURVE_PARAMS:
		result[axis] = clampf(compute_age_curve(axis, age_years) * fs, 0.05, 1.0)
	return result

## 직렬화
func to_dict() -> Dictionary:
	return {
		"str_val": str_val,
		"agi": agi,
		"end_val": end_val,
		"tou": tou,
		"rec": rec,
		"dr": dr,
	}

## 역직렬화
static func from_dict(data: Dictionary):
	var b = load("res://scripts/core/body_attributes.gd").new()
	b.str_val = data.get("str_val", 0.5)
	b.agi     = data.get("agi",     0.5)
	b.end_val = data.get("end_val", 0.5)
	b.tou     = data.get("tou",     0.5)
	b.rec     = data.get("rec",     0.5)
	b.dr      = data.get("dr",      0.5)
	return b
