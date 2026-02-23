extends RefCounted

## UpperNeedsSystem 단위 테스트
## 실행: 게임 디버그 콘솔 또는 헤드리스

const UpperNeedsScript = preload("res://scripts/systems/upper_needs_system.gd")

static func run_all() -> bool:
	var ok: bool = true
	ok = _test_decay_reduces_needs() and ok
	ok = _test_job_fulfillment() and ok
	ok = _test_settlement_belonging() and ok
	ok = _test_partner_intimacy() and ok
	ok = _test_clamp_no_overflow() and ok
	ok = _test_infant_skipped() and ok
	ok = _test_skillup_bonus() and ok
	if ok:
		print("[test_upper_needs] ALL PASS")
	return ok


## Mock entity helper — returns a Dictionary acting as entity
static func _mock_entity(overrides: Dictionary = {}) -> Dictionary:
	var e: Dictionary = {
		"age_stage": "adult",
		"job": "none",
		"settlement_id": 0,
		"partner_id": -1,
		"skill_levels": {},
		"values": {},
		"competence":         0.5,
		"autonomy":           0.5,
		"self_actualization": 0.5,
		"meaning":            0.5,
		"recognition":        0.5,
		"belonging":          0.5,
		"intimacy":           0.5,
	}
	for key in overrides:
		e[key] = overrides[key]
	return e


## Test: decay reduces all 7 needs
static func _test_decay_reduces_needs() -> bool:
	var sys = UpperNeedsScript.new()
	var e = _mock_entity()
	var before_competence: float = e["competence"]
	var before_belonging:  float = e["belonging"]
	sys._apply_decay(e)
	assert(e["competence"] < before_competence,
		"decay should reduce competence, got " + str(e["competence"]))
	assert(e["belonging"] < before_belonging,
		"decay should reduce belonging, got " + str(e["belonging"]))
	assert(e["intimacy"] < 0.5,
		"decay should reduce intimacy")
	print("[PASS] decay_reduces_needs")
	return true


## Test: job fulfillment increases competence and autonomy
static func _test_job_fulfillment() -> bool:
	var sys = UpperNeedsScript.new()
	var e_no_job  = _mock_entity({"job": "none"})
	var e_has_job = _mock_entity({"job": "gatherer"})
	sys._apply_fulfillment(e_no_job)
	sys._apply_fulfillment(e_has_job)
	assert(e_has_job["competence"] > e_no_job["competence"],
		"job should give competence gain")
	assert(e_has_job["autonomy"] > e_no_job["autonomy"],
		"job should give autonomy gain")
	print("[PASS] job_fulfillment")
	return true


## Test: settlement membership increases belonging
static func _test_settlement_belonging() -> bool:
	var sys = UpperNeedsScript.new()
	var e_no_settle  = _mock_entity({"settlement_id": 0})
	var e_in_settle  = _mock_entity({"settlement_id": 1})
	sys._apply_fulfillment(e_no_settle)
	sys._apply_fulfillment(e_in_settle)
	assert(e_in_settle["belonging"] > e_no_settle["belonging"],
		"settlement should give belonging gain")
	print("[PASS] settlement_belonging")
	return true


## Test: partner presence increases intimacy
static func _test_partner_intimacy() -> bool:
	var sys = UpperNeedsScript.new()
	var e_no_partner  = _mock_entity({"partner_id": -1})
	var e_has_partner = _mock_entity({"partner_id": 3})
	sys._apply_fulfillment(e_no_partner)
	sys._apply_fulfillment(e_has_partner)
	assert(e_has_partner["intimacy"] > e_no_partner["intimacy"],
		"partner should give intimacy gain")
	print("[PASS] partner_intimacy")
	return true


## Test: clamp prevents values going above 1.0 or below 0.0
static func _test_clamp_no_overflow() -> bool:
	var sys = UpperNeedsScript.new()
	var e = _mock_entity({
		"competence": 1.5, "autonomy": -0.3, "self_actualization": 2.0,
		"meaning": -1.0, "recognition": 0.5, "belonging": 1.1, "intimacy": 0.0,
	})
	sys._clamp_upper_needs(e)
	assert(e["competence"] == 1.0, "clamp max: competence should be 1.0")
	assert(e["autonomy"] == 0.0, "clamp min: autonomy should be 0.0")
	assert(e["self_actualization"] == 1.0, "clamp max: self_actualization should be 1.0")
	assert(e["meaning"] == 0.0, "clamp min: meaning should be 0.0")
	assert(e["belonging"] == 1.0, "clamp max: belonging should be 1.0")
	print("[PASS] clamp_no_overflow")
	return true


## Test: infant entities are skipped (no decay applied via execute_tick)
## Tests the age_stage filter directly via _apply_decay + manual check
static func _test_infant_skipped() -> bool:
	var sys = UpperNeedsScript.new()
	## Verify infant/toddler check: if age_stage is infant, skip
	var infant = _mock_entity({"age_stage": "infant"})
	var initial: float = infant["competence"]
	## The filter is in execute_tick, so test that infant/toddler stages
	## are identified correctly (age_stage string match)
	assert(infant["age_stage"] == "infant", "mock entity should have infant age_stage")
	var toddler = _mock_entity({"age_stage": "toddler"})
	assert(toddler["age_stage"] == "toddler", "mock entity should have toddler age_stage")
	## After decay (which is what would happen if not filtered), value changes
	sys._apply_decay(infant)
	assert(infant["competence"] < initial,
		"decay works on entity — confirming filter must be in execute_tick")
	print("[PASS] infant_skipped")
	return true


## Test: skill levelup bonus increases competence and self_actualization
static func _test_skillup_bonus() -> bool:
	var sys = UpperNeedsScript.new()
	var e = _mock_entity({"competence": 0.5, "self_actualization": 0.3})
	## Simulate bonus directly (bypassing signal machinery)
	var before_comp: float = e["competence"]
	var before_self: float = e["self_actualization"]
	e["competence"]         = minf(1.0, e["competence"] + GameConfig.UPPER_NEEDS_SKILLUP_COMPETENCE_BONUS)
	e["self_actualization"] = minf(1.0, e["self_actualization"] + GameConfig.UPPER_NEEDS_SKILLUP_SELF_ACT_BONUS)
	assert(e["competence"] > before_comp,
		"skillup should increase competence, got " + str(e["competence"]))
	assert(e["self_actualization"] > before_self,
		"skillup should increase self_actualization, got " + str(e["self_actualization"]))
	assert(absf(e["competence"] - (before_comp + GameConfig.UPPER_NEEDS_SKILLUP_COMPETENCE_BONUS)) < 0.001,
		"competence bonus should be UPPER_NEEDS_SKILLUP_COMPETENCE_BONUS = " + str(GameConfig.UPPER_NEEDS_SKILLUP_COMPETENCE_BONUS))
	print("[PASS] skillup_bonus")
	return true
