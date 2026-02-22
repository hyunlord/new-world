extends RefCounted

## StatCurve 단위 테스트
## 실행: 게임 디버그 콘솔 또는 헤드리스

const StatCurveScript = preload("res://scripts/core/stat_curve.gd")

static func run_all() -> bool:
	var ok: bool = true
	ok = _test_sigmoid_extreme() and ok
	ok = _test_threshold_power() and ok
	ok = _test_power_influence() and ok
	ok = _test_log_xp() and ok
	ok = _test_scurve_speed() and ok
	if ok:
		print("[test_stat_curve] ALL PASS")
	return ok

static func _test_sigmoid_extreme() -> bool:
	var p: Dictionary = {"flat_zone": [200, 800], "pole_multiplier": 3.0}
	var mid: float = StatCurveScript.sigmoid_extreme(500, p)
	assert(absf(mid - 1.0) < 0.1, "sigmoid_extreme(500) should ≈ 1.0, got " + str(mid))
	var high: float = StatCurveScript.sigmoid_extreme(980, p)
	assert(high > 2.5, "sigmoid_extreme(980) should > 2.5, got " + str(high))
	var low: float = StatCurveScript.sigmoid_extreme(20, p)
	assert(low < 0.5, "sigmoid_extreme(20) should < 0.5, got " + str(low))
	print("[PASS] sigmoid_extreme")
	return true

static func _test_threshold_power() -> bool:
	var p: Dictionary = {"threshold": 350, "exponent": 2.0, "max_output": 12.0}
	assert(StatCurveScript.threshold_power(400, p) == 0.0,
		"threshold_power(400) should be 0.0")
	assert(StatCurveScript.threshold_power(350, p) == 0.0,
		"threshold_power(350) should be 0.0 (at threshold)")
	var at_zero: float = StatCurveScript.threshold_power(0, p)
	assert(absf(at_zero - 12.0) < 0.01,
		"threshold_power(0) should be max_output=12.0, got " + str(at_zero))
	var mid_deficit: float = StatCurveScript.threshold_power(175, p)
	assert(mid_deficit > 2.0 and mid_deficit < 4.0,
		"threshold_power(175) should be 2~4, got " + str(mid_deficit))
	print("[PASS] threshold_power")
	return true

static func _test_power_influence() -> bool:
	var p1: Dictionary = {"exponent": 1.0}
	assert(absf(StatCurveScript.power_influence(500, p1) - 0.5) < 0.001,
		"power(500, exp=1.0) should be 0.5")
	var p2: Dictionary = {"exponent": 2.0}
	assert(absf(StatCurveScript.power_influence(1000, p2) - 1.0) < 0.001,
		"power(1000, exp=2.0) should be 1.0")
	assert(StatCurveScript.power_influence(500, p2) < StatCurveScript.power_influence(500, p1),
		"higher exponent should give lower mid-value")
	print("[PASS] power_influence")
	return true

static func _test_log_xp() -> bool:
	var p: Dictionary = {
		"base_xp": 100.0, "exponent": 1.8,
		"level_breakpoints": [25, 50, 75],
		"breakpoint_multipliers": [1.0, 1.5, 2.0, 3.0]
	}
	var xp1: float = StatCurveScript.log_xp_required(1, p)
	var xp50: float = StatCurveScript.log_xp_required(50, p)
	assert(xp50 > xp1 * 100.0, "lv50 XP should be >> lv1 XP")
	var xp25: float = StatCurveScript.log_xp_required(25, p)
	var xp26: float = StatCurveScript.log_xp_required(26, p)
	assert(xp26 > xp25 * 1.4, "breakpoint at 25 should increase XP requirement")
	print("[PASS] log_xp_required")
	return true

static func _test_scurve_speed() -> bool:
	var p: Dictionary = {
		"phase_breakpoints": [300, 700],
		"phase_speeds": [1.5, 1.0, 0.3]
	}
	assert(StatCurveScript.scurve_speed(100, p) == 1.5,
		"scurve speed in phase 1 should be 1.5")
	assert(StatCurveScript.scurve_speed(500, p) == 1.0,
		"scurve speed in phase 2 should be 1.0")
	assert(StatCurveScript.scurve_speed(800, p) == 0.3,
		"scurve speed in phase 3 should be 0.3")
	print("[PASS] scurve_speed")
	return true
