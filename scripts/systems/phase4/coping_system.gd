extends "res://scripts/core/simulation_system.gd"

# NO class_name — headless compatibility
# Phase 4: Coping Trait System
# Implements 15 coping strategies based on psychological research

const COPING_DEFS_PATH: String = "res://data/coping_definitions.json"
const COPING_COUNT_MAX: float = 15.0

var _entity_manager: RefCounted
var _coping_defs: Dictionary = {}   # loaded from JSON
var _entity_coping: Dictionary = {} # entity_id -> coping state
var _rng: RandomNumberGenerator


func _init() -> void:
	system_name = "coping"
	priority = 42   # after morale(40)
	tick_interval = 30  # coping acquisition check every 30 ticks
	_rng = RandomNumberGenerator.new()
	_rng.randomize()


func init(entity_manager: RefCounted, rng: RandomNumberGenerator) -> void:
	_entity_manager = entity_manager
	if rng != null:
		_rng = rng
	_load_coping_defs()


func _load_coping_defs() -> void:
	_coping_defs.clear()
	if not FileAccess.file_exists(COPING_DEFS_PATH):
		return
	var file = FileAccess.open(COPING_DEFS_PATH, FileAccess.READ)
	if file == null:
		return
	var text: String = file.get_as_text()
	file.close()

	var json = JSON.new()
	var err: int = json.parse(text)
	if err != OK:
		return

	var raw = json.get_data()
	if typeof(raw) != TYPE_DICTIONARY:
		return

	if raw.has("copings") and typeof(raw.get("copings", {})) == TYPE_DICTIONARY:
		_coping_defs = raw.get("copings", {}).duplicate(true)
	else:
		_coping_defs = raw.duplicate(true)


func _make_default_state() -> Dictionary:
	return {
		"owned": {},
		"denial_accumulator": 0.0,
		"denial_timer": 0,
		"dependency_score": 0.0,
		"helplessness_score": 0.0,
		"control_appraisal_cap": 1.0,
		"rebound_queue": [],
		"cooldowns": {},
		"break_count": 0,
		"last_break_type": "",
		"substance_recent_timer": 0,
	}


func _get_entity_id(entity) -> int:
	if entity == null:
		return -1
	if "entity_id" in entity:
		return int(entity.entity_id)
	if "id" in entity:
		return int(entity.id)
	return -1


func _get_state(entity) -> Dictionary:
	var eid: int = _get_entity_id(entity)
	if eid < 0:
		return {}
	if not _entity_coping.has(eid):
		_entity_coping[eid] = _make_default_state()
	return _entity_coping[eid]


func execute_tick(tick: int) -> void:
	if _entity_manager == null:
		return
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity = alive[i]
		var eid: int = _get_entity_id(entity)
		if eid < 0:
			continue
		if not _entity_coping.has(eid):
			continue
		var cs = _entity_coping[eid]

		# Cooldown 카운트다운
		var cooldowns = cs.get("cooldowns", {})
		for cid in cooldowns.keys():
			cooldowns[cid] = maxf(0.0, float(cooldowns[cid]) - 1.0)
		cs["cooldowns"] = cooldowns

		# owned.cooldown 동기화
		var owned: Dictionary = cs.get("owned", {})
		for owned_id in owned.keys():
			var owned_entry = owned.get(owned_id, {})
			if typeof(owned_entry) == TYPE_DICTIONARY:
				owned_entry["cooldown"] = float(cooldowns.get(owned_id, 0.0))
				owned[owned_id] = owned_entry
		cs["owned"] = owned

		# C05 Denial 타이머
		var denial_timer: int = int(cs.get("denial_timer", 0))
		if denial_timer > 0:
			cs["denial_timer"] = denial_timer - 1
			if int(cs.get("denial_timer", 0)) <= 0:
				_explode_denial_debt(entity, cs)

		# allostatic 과부하 시 조기 폭발
		if float(cs.get("denial_accumulator", 0.0)) > 0.01:
			if entity.emotion_data != null and entity.emotion_data.allostatic / 100.0 > 0.8:
				_explode_denial_debt(entity, cs)

		# C13 rebound 처리
		var rebound_queue = cs.get("rebound_queue", [])
		var remaining_rebounds: Array = []
		for rb in rebound_queue:
			rb["delay"] = int(rb.get("delay", 0)) - 1
			if int(rb.get("delay", 0)) <= 0:
				# rebound 발동
				if entity.emotion_data != null:
					entity.emotion_data.stress = clampf(
						entity.emotion_data.stress + float(rb.get("stress_rebound", 0.0)) * 300.0,
						0.0, 2000.0)
					entity.emotion_data.allostatic = clampf(
						entity.emotion_data.allostatic + float(rb.get("allostatic_add", 0.0)) * 100.0,
						0.0, 100.0)
			else:
				remaining_rebounds.append(rb)
		cs["rebound_queue"] = remaining_rebounds

		# C13 금단 스트레스
		var recent_timer: int = int(cs.get("substance_recent_timer", 0))
		if recent_timer > 0:
			cs["substance_recent_timer"] = recent_timer - 1
		if float(cs.get("dependency_score", 0.0)) > 0.6 and int(cs.get("substance_recent_timer", 0)) <= 0:
			if entity.emotion_data != null:
				entity.emotion_data.stress = clampf(entity.emotion_data.stress + 0.30 * 300.0, 0.0, 2000.0)
			cs["substance_recent_timer"] = 24


## Called when mental break starts — increment break count
func on_mental_break_started(entity, break_type: String) -> void:
	var eid: int = _get_entity_id(entity)
	if eid < 0:
		return
	if not _entity_coping.has(eid):
		_entity_coping[eid] = _make_default_state()
	var cs = _entity_coping[eid]
	cs["break_count"] = int(cs.get("break_count", 0)) + 1
	cs["last_break_type"] = break_type


## Called when mental break ends — attempt coping acquisition
func on_mental_break_recovered(entity, break_type: String, tick: int) -> int:
	var eid: int = _get_entity_id(entity)
	if eid < 0:
		return 0
	if not _entity_coping.has(eid):
		_entity_coping[eid] = _make_default_state()
	var cs = _entity_coping[eid]

	# 1차 판정: 이번에 뭔가 배울지?
	var p_learn: float = _calculate_learn_probability(entity, cs, break_type, true)
	if _rng.randf() >= p_learn:
		return 0  # 아무것도 획득 안 함

	# 2차 분기: 신규 vs 기존 숙련도 상승
	var N: int = cs.get("owned", {}).size()
	var S_N: float = log(1.0 + float(N)) / log(1.0 + COPING_COUNT_MAX)
	var p_new: float = p_learn * (1.0 - S_N)
	var p_upgrade: float = p_learn * S_N
	var denom: float = maxf(p_new + p_upgrade, 0.001)

	if _rng.randf() < (p_new / denom):
		# 신규 획득
		return 1 if _attempt_acquire_new(entity, cs, break_type, tick) else 0

	# 기존 숙련도 상승
	return 1 if _attempt_upgrade_existing(entity, cs, tick) else 0


func attempt_acquire_on_break_recovery(entity, break_type: String, tick: int) -> int:
	return on_mental_break_recovered(entity, break_type, tick)


## [Lazarus & Folkman, 1984 - Transactional Model of Stress / Coping]
## 스트레스 고조 후 회복 시 새로운 Coping 전략 획득 가능성 계산.
## K(n): 누적 브레이크 압력 포화함수 — 브레이크를 많이 겪을수록 학습 동기 ↑
## S(N): 기보유 Coping 포화 — 이미 많이 보유하면 신규 획득 확률 ↓
## Reference: Lazarus, R.S. & Folkman, S. (1984). Stress, Appraisal, and Coping. Springer.
## [Lazarus & Folkman, 1984 - Transactional Model of Stress]
## K(n)/S(N) 포화 함수로 학습 확률 조절. sigmoid를 통한 1차 판정.
func _calculate_learn_probability(entity, coping_state: Dictionary,
		break_type: String, is_recovery: bool) -> float:
	if entity == null or entity.emotion_data == null:
		return 0.0
	var _unused_break_type: String = break_type
	var ed = entity.emotion_data
	var stress_norm: float = clampf(ed.stress / 2000.0, 0.0, 1.0)
	var allostatic_norm: float = ed.allostatic / 100.0

	# K(n): 누적 브레이크 압력 포화형 (n = 브레이크 경험 횟수)
	var n: int = int(coping_state.get("break_count", 0))
	var K_n: float = 1.0 - exp(-0.35 * float(n))

	# S(N): 기보유 Coping 포화 (N = 현재 보유 수)
	var N: int = coping_state.get("owned", {}).size()
	var S_N: float = log(1.0 + float(N)) / log(1.0 + COPING_COUNT_MAX)

	# logit 계산
	var logit: float = (
		-2.5
		+ 1.0 * clampf((stress_norm - 0.6) / 0.4, 0.0, 1.0)
		+ 0.7 * clampf((allostatic_norm - 0.5) / 0.5, 0.0, 1.0)
		+ 1.2 * (1.0 if is_recovery else 0.0)
		+ 1.4 * K_n
		- 1.1 * S_N
	)
	# sigmoid
	var p_learn: float = 1.0 / (1.0 + exp(-logit))
	p_learn *= (1.0 if is_recovery else 0.30)
	return clampf(p_learn, 0.0, 1.0)


## [Carver, Scheier & Weintraub, 1989 - COPE Scale]
## 각 Coping 전략의 효용 점수를 HEXACO 성격 가중치로 계산 후 softmax 선택.
## Softmax 선택 이유: 독립 베르누이 (각각 독립 확률)는 동시 다중 획득 가능 → 인플레이션.
## softmax는 "여러 후보 중 정확히 1개 선택"을 보장함.
## Reference: Carver, C.S. et al. (1989). COPE: Journal of Personality and Social Psychology, 56(2).
## [Carver et al., 1989 - COPE Scale]
## softmax로 전략 경쟁 선택 (독립 베르누이 대신 — 동시 다중 획득 방지).
func _calculate_utility_scores(entity, coping_state: Dictionary,
		break_type: String) -> Dictionary:
	var utilities: Dictionary = {}
	if entity == null or entity.emotion_data == null:
		return utilities

	var ed = entity.emotion_data
	var pd = entity.personality
	var _allostatic_norm: float = ed.allostatic / 100.0
	var control: float = _get_control_appraisal(entity, coping_state)
	var owned: Dictionary = coping_state.get("owned", {})
	var n: int = int(coping_state.get("break_count", 0))
	var K_n: float = 1.0 - exp(-0.35 * float(n))

	for cid in _coping_defs:
		var cdef = _coping_defs[cid]
		if typeof(cdef) != TYPE_DICTIONARY:
			continue

		# 1) log(base_rate) 기저
		var U: float = log(maxf(float(cdef.get("base_rate", 0.05)), 0.001))

		# 2) HEXACO 성격 가중치 (각 facet 기여)
		var hw = cdef.get("hexaco_weights", {})
		if typeof(hw) == TYPE_DICTIONARY:
			for facet_key in hw:
				var facet_val: float = 0.5
				if pd != null:
					facet_val = float(pd.facets.get(facet_key, 0.5))
				U += 2.0 * float(hw[facet_key]) * (2.0 * (facet_val - 0.5))

		# 3) 브레이크 유형 가중치
		var bw = cdef.get("break_weights", {})
		if typeof(bw) == TYPE_DICTIONARY:
			U += 2.5 * float(bw.get(break_type, 0.0))

		# 4) control_appraisal 조건
		var ctrl_min: float = float(cdef.get("control_appraisal_min", 0.0))
		var ctrl_max: float = float(cdef.get("control_appraisal_max", 1.0))
		if control < ctrl_min or control > ctrl_max:
			U -= 5.0  # 조건 불충족 → 강한 패널티

		# 5) 부적응 편향 (C05/C11/C13만)
		if bool(cdef.get("is_maladaptive", false)):
			U += _calculate_maladaptive_bias(entity, coping_state, K_n)

		# 6) 상충 패널티
		var conflicts: Array = cdef.get("conflicts", [])
		for conflict_id in conflicts:
			if owned.has(conflict_id):
				U -= 3.0

		# 7) 동일 focus 포화 패널티
		var focus: String = str(cdef.get("focus", ""))
		var same_focus_count: int = 0
		for owned_id in owned:
			if _coping_defs.has(owned_id):
				var odef = _coping_defs[owned_id]
				if typeof(odef) == TYPE_DICTIONARY:
					if str(odef.get("focus", "")) == focus:
						same_focus_count += 1
		U -= 0.25 * float(same_focus_count)

		utilities[cid] = U

	return utilities


## [Aldwin & Revenson, 1987 - Maladaptive Coping Bias]
## C05(Denial)/C11(Disengagement)/C13(Substance Use)에만 적용되는 부적응 편향.
## 극심한 allostatic 부하 + 낮은 통제감 상황에서 부적응 전략이 선택될 확률 증가.
## Reference: Aldwin, C.M. & Revenson, T.A. (1987). Journal of Personality and Social Psychology, 53(2).
## [Aldwin & Revenson, 1987 - Maladaptive Coping Bias]
## C05/C11/C13 부적응 전략은 극심한 스트레스 상황에서 선택 확률 증가.
## allostatic 부하 높음 + 통제감 낮음 + 브레이크 누적 → 부적응 편향 발동.
func _calculate_maladaptive_bias(entity, coping_state: Dictionary, K_n: float) -> float:
	if entity == null or entity.emotion_data == null:
		return 0.0
	var ed = entity.emotion_data
	var pd = entity.personality
	var allostatic_norm: float = ed.allostatic / 100.0
	var control: float = _get_control_appraisal(entity, coping_state)

	var E_anxiety: float = 0.5
	var C_prudence: float = 0.5
	var H_sincerity: float = 0.5
	if pd != null:
		E_anxiety = float(pd.facets.get("E_anxiety", 0.5))
		C_prudence = float(pd.facets.get("C_prudence", 0.5))
		H_sincerity = float(pd.facets.get("H_sincerity", 0.5))

	# 위반 스트레스 정규화 (0~1)
	var violation_stress_norm: float = clampf(ed.stress / 2000.0, 0.0, 1.0)

	var bias: float = (
		-0.4
		+ 0.9 * clampf((allostatic_norm - 0.65) / 0.35, -1.0, 1.0)
		+ 0.6 * clampf((0.45 - control) / 0.45, -1.0, 1.0)
		+ 0.4 * K_n
		+ 0.5 * (E_anxiety - 0.5) / 0.5
		- 0.6 * (C_prudence - 0.5) / 0.5
		- 0.4 * (H_sincerity - 0.5) / 0.5
		+ 0.3 * violation_stress_norm
	)
	return bias


func _softmax_select(utilities: Dictionary) -> String:
	if utilities.is_empty():
		return ""
	# softmax 확률 계산
	var max_u: float = -INF
	for k in utilities:
		if float(utilities[k]) > max_u:
			max_u = float(utilities[k])

	var exp_sum: float = 0.0
	var exp_vals: Dictionary = {}
	for k in utilities:
		var ev: float = exp(float(utilities[k]) - max_u)
		exp_vals[k] = ev
		exp_sum += ev

	var roll: float = _rng.randf() * exp_sum
	var cumulative: float = 0.0
	for k in exp_vals:
		cumulative += float(exp_vals[k])
		if roll <= cumulative:
			return str(k)

	return str(utilities.keys()[0])


func _attempt_acquire_new(entity, cs: Dictionary, break_type: String, tick: int) -> bool:
	var owned: Dictionary = cs.get("owned", {})
	var utilities: Dictionary = _calculate_utility_scores(entity, cs, break_type)
	var candidates: Dictionary = {}
	for cid in utilities:
		if not owned.has(cid):
			candidates[cid] = utilities[cid]
	if candidates.is_empty():
		return false

	var selected_id: String = _softmax_select(candidates)
	if selected_id == "":
		return false

	owned[selected_id] = {
		"proficiency": 0.0,
		"cooldown": 0.0,
	}
	cs["owned"] = owned

	var cooldowns: Dictionary = cs.get("cooldowns", {})
	cooldowns[selected_id] = 0.0
	cs["cooldowns"] = cooldowns

	_log_coping_acquired(entity, selected_id, tick)
	return true


func _attempt_upgrade_existing(entity, cs: Dictionary, tick: int) -> bool:
	var owned: Dictionary = cs.get("owned", {})
	if owned.is_empty():
		return false

	var break_type: String = str(cs.get("last_break_type", ""))
	var utilities: Dictionary = _calculate_utility_scores(entity, cs, break_type)
	var candidates: Dictionary = {}
	for cid in owned:
		candidates[cid] = float(utilities.get(cid, 0.0))
	if candidates.is_empty():
		return false

	var selected_id: String = _softmax_select(candidates)
	if selected_id == "":
		return false

	var entry: Dictionary = owned.get(selected_id, {})
	var old_prof: float = float(entry.get("proficiency", 0.0))
	entry["proficiency"] = clampf(old_prof + 0.12, 0.0, 1.0)
	owned[selected_id] = entry
	cs["owned"] = owned

	_log_coping_upgraded(entity, selected_id, tick)
	return true


func execute_coping(entity, coping_id: String, tick: int) -> bool:
	if entity == null or entity.emotion_data == null:
		return false
	var cs: Dictionary = _get_state(entity)
	if cs.is_empty():
		return false

	var owned: Dictionary = cs.get("owned", {})
	if not owned.has(coping_id):
		return false

	var cooldowns: Dictionary = cs.get("cooldowns", {})
	if float(cooldowns.get(coping_id, 0.0)) > 0.0:
		return false

	var entry: Dictionary = owned.get(coping_id, {})
	var prof: float = clampf(float(entry.get("proficiency", 0.0)), 0.0, 1.0)

	if coping_id.begins_with("C05"):
		_execute_denial(entity, cs, prof)
	elif coping_id.begins_with("C11"):
		_execute_disengagement(entity, cs, prof)
	elif coping_id.begins_with("C12") or coping_id.findn("rumination") >= 0:
		_execute_rumination(entity, cs, prof)
	elif coping_id.begins_with("C13"):
		_execute_substance_use(entity, cs, prof)
	else:
		_execute_generic(entity, cs, coping_id, prof)

	if coping_id.begins_with("C09"):
		entity.set_meta("phase4_venting_pending", true)
		entity.set_meta("phase4_venting_tick", tick)

	var cdef = _coping_defs.get(coping_id, {})
	var cooldown_ticks: float = 0.0
	if typeof(cdef) == TYPE_DICTIONARY:
		cooldown_ticks = float(cdef.get("cooldown_ticks", cdef.get("cooldown", 0.0)))
	cooldowns[coping_id] = maxf(cooldown_ticks, 0.0)
	cs["cooldowns"] = cooldowns

	entry["cooldown"] = float(cooldowns.get(coping_id, 0.0))
	owned[coping_id] = entry
	cs["owned"] = owned

	return true


func _execute_generic(entity, cs: Dictionary, coping_id: String, prof: float) -> void:
	var cdef = _coping_defs.get(coping_id, {})
	if typeof(cdef) != TYPE_DICTIONARY:
		return
	var ed = entity.emotion_data

	var stress_relief: float = float(cdef.get("stress_relief", cdef.get("stress_reduce", 0.0)))
	if stress_relief > 0.0:
		var relief_amt: float = stress_relief if stress_relief > 1.0 else stress_relief * 300.0
		ed.stress = clampf(ed.stress - relief_amt * (0.5 + 0.5 * prof), 0.0, 2000.0)

	var reserve_add: float = float(cdef.get("reserve_add", cdef.get("reserve_gain", 0.0)))
	if reserve_add != 0.0:
		var reserve_amt: float = reserve_add if absf(reserve_add) > 1.0 else reserve_add * 100.0
		ed.reserve = clampf(ed.reserve + reserve_amt * (0.5 + 0.5 * prof), 0.0, 100.0)

	var allostatic_add: float = float(cdef.get("allostatic_add", 0.0))
	if allostatic_add != 0.0:
		var allo_amt: float = allostatic_add if absf(allostatic_add) > 1.0 else allostatic_add * 100.0
		ed.allostatic = clampf(ed.allostatic + allo_amt, 0.0, 100.0)

	if bool(cdef.get("is_maladaptive", false)):
		var dep: float = float(cs.get("dependency_score", 0.0))
		cs["dependency_score"] = clampf(dep + 0.01, 0.0, 1.0)


## [Compas et al., 2001 - Coping with Stress in Childhood and Adolescence]
## 부정(Denial) = 위협을 인식하지 않으려는 인지적 회피.
## hidden_threat_accumulator에 실제 위협의 80%를 "부채"로 누적.
## 72틱 후 또는 allostatic > 0.8 → 누적된 부채가 1.5배로 폭발적 귀환 (delayed explosion).
## Reference: Compas, B.E. et al. (2001). Psychological Bulletin, 127(1), 87-127.
## [Compas et al., 2001 - Coping with Stress in Childhood and Adolescence]
## hidden_threat_accumulator에 위협을 부채로 누적. 72틱 후 1.5배로 폭발.
func _execute_denial(entity, cs: Dictionary, prof: float) -> void:
	var ed = entity.emotion_data
	# 즉각 스트레스 감소 (부채는 나중에 폭발)
	var relief: float = 150.0 * (0.5 + 0.5 * prof)
	ed.stress = maxf(0.0, ed.stress - relief)
	# 숨겨진 부채 누적
	var real_threat: float = ed.stress * 0.4
	cs["denial_accumulator"] = float(cs.get("denial_accumulator", 0.0)) + real_threat * 0.8
	cs["denial_timer"] = 72
	# allostatic 영구 페널티
	ed.allostatic = clampf(ed.allostatic + 2.0, 0.0, 100.0)


func _explode_denial_debt(entity, cs: Dictionary) -> void:
	var ed = entity.emotion_data
	var debt: float = float(cs.get("denial_accumulator", 0.0))
	if debt > 0.0:
		ed.stress = clampf(ed.stress + debt * 1.5, 0.0, 2000.0)
		cs["denial_accumulator"] = 0.0
		cs["denial_timer"] = 0


## [Seligman, 1975 - Learned Helplessness Theory]
## 목표 포기(Behavioral Disengagement) = 통제 불가능 상황에서 저항을 멈추는 학습된 무기력.
## helplessness_score 누적 → control_appraisal_cap이 영구적으로 0.3으로 고정됨.
## 즉각 GAS 소모 정지 vs 장기 통제감 영구 손상의 트레이드오프.
## Reference: Seligman, M.E.P. (1975). Helplessness: On Depression, Development, and Death. Freeman.
## [Seligman, 1975 - Learned Helplessness Theory]
## 목표 포기 → 즉각 GAS 소모 정지. helplessness 누적 → control_appraisal_cap 영구 저하.
func _execute_disengagement(entity, cs: Dictionary, prof: float) -> void:
	var _unused_prof: float = prof
	var ed = entity.emotion_data
	# GAS 소모 즉시 정지 (reserve 일부 회복)
	ed.reserve = minf(ed.reserve + 10.0, 100.0)
	# helplessness 누적
	var helplessness: float = float(cs.get("helplessness_score", 0.0)) + 0.10
	cs["helplessness_score"] = minf(helplessness, 1.0)
	# allostatic 페널티
	ed.allostatic = clampf(ed.allostatic + 3.0, 0.0, 100.0)
	# 학습된 무기력 → control_appraisal_cap 영구 저하
	if helplessness > 0.8:
		cs["control_appraisal_cap"] = 0.3


## [Carver et al., 1989 - COPE Scale / Nolen-Hoeksema, 1991 - Response Styles Theory]
## Rumination의 두 하위 유형: Reflection(성찰, 적응적) vs Brooding(자기몰입, 부적응).
## proficiency 0.5를 경계로 sigmoid로 부드럽게 전환 (이산 스위치 대신 혼합 비율).
## Reference: Treynor, W. et al. (2003). Rumination reconsidered. Cognitive Therapy and Research, 27(3).
## [Carver et al., 1989 - COPE Scale / Nolen-Hoeksema, 1991 - Response Styles Theory]
## Rumination 두 하위 유형: proficiency 0.5 미만 = Reflection(적응적), 이상 = Brooding(부적응).
## sigmoid로 부드럽게 전환 (이산 스위치 대신).
## Reference: Treynor, W. et al. (2003). Rumination reconsidered. Cognitive Therapy and Research, 27(3).
func _execute_rumination(entity, cs: Dictionary, prof: float) -> void:
	var _unused_cs: Dictionary = cs
	var ed = entity.emotion_data
	# sigmoid로 Reflection vs Brooding 비율 결정
	var brooding_weight: float = 1.0 / (1.0 + exp(-10.0 * (prof - 0.5)))
	var reflection_weight: float = 1.0 - brooding_weight
	# Reflection: 약간의 통찰 (reserve 소폭 회복)
	if reflection_weight > 0.3:
		ed.reserve = minf(ed.reserve + 5.0 * reflection_weight, 100.0)
	# Brooding: allostatic 증가, 수면 품질 저하
	if brooding_weight > 0.3:
		ed.allostatic = clampf(ed.allostatic + 4.0 * brooding_weight, 0.0, 100.0)
		entity.set_meta("sleep_quality_penalty", entity.get_meta("sleep_quality_penalty", 0.0) + 0.2 * brooding_weight)


## [Cooper et al., 1995 - Drinking to Regulate Negative Affect]
## 물질(주류 등)로 즉각적 스트레스를 차단하나, 12틱 후 반동(rebound) 발생.
## dependency_score 누적 → 0.6 초과 시 물질 없으면 금단 스트레스 0.30 직격.
## Reference: Cooper, M.L. et al. (1995). Journal of Personality and Social Psychology, 69(5).
## [Cooper et al., 1995 - Drinking to Regulate Negative Affect]
## 물질 사용: 즉각적 스트레스 차단 + 12틱 후 반동 + 의존성 누적.
## Reference: Cooper, M.L. et al. (1995). Journal of Personality and Social Psychology, 69(5).
func _execute_substance_use(entity, cs: Dictionary, prof: float) -> void:
	var ed = entity.emotion_data
	# 즉각 스트레스 감소
	var relief: float = 300.0 * (0.4 + 0.6 * prof)
	ed.stress = maxf(0.0, ed.stress - relief)
	# 12틱 후 반동 스케줄
	var rebound_queue = cs.get("rebound_queue", [])
	rebound_queue.append({
		"delay": 12,
		"stress_rebound": 0.15 * prof,   # normalized 0~1
		"allostatic_add": 0.08            # normalized 0~1
	})
	cs["rebound_queue"] = rebound_queue
	# 의존성 누적
	var dep: float = float(cs.get("dependency_score", 0.0)) + 0.05
	cs["dependency_score"] = minf(dep, 1.0)
	cs["substance_recent_timer"] = 24


func _get_control_appraisal(entity, cs: Dictionary) -> float:
	var ed = entity.emotion_data
	var pd = entity.personality
	# 기본값: reserve / 100 * C_axis
	var C_axis: float = 0.5
	if pd != null:
		C_axis = float(pd.axes.get("C", 0.5))
	var base: float = (ed.reserve / 100.0) * 0.6 + C_axis * 0.4
	# helplessness cap 적용
	var cap: float = float(cs.get("control_appraisal_cap", 1.0))
	return clampf(base, 0.0, cap)


func _log_coping_acquired(entity, coping_id: String, tick: int) -> void:
	var cdef = _coping_defs.get(coping_id, {})
	var name_key: String = coping_id
	if typeof(cdef) == TYPE_DICTIONARY:
		name_key = str(cdef.get("name_key", coping_id))
	var desc: String = Locale.trf("COPING_ACQUIRED", {"name": Locale.ltr(name_key)})
	var chronicle = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
	if chronicle:
		chronicle.log_event("coping_acquired", _get_entity_id(entity), desc, 2, [], tick)


func _log_coping_upgraded(entity, coping_id: String, tick: int) -> void:
	var cdef = _coping_defs.get(coping_id, {})
	var name_key: String = coping_id
	if typeof(cdef) == TYPE_DICTIONARY:
		name_key = str(cdef.get("name_key", coping_id))
	var desc: String = Locale.trf("COPING_UPGRADED", {"name": Locale.ltr(name_key)})
	var chronicle = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
	if chronicle:
		chronicle.log_event("coping_upgraded", _get_entity_id(entity), desc, 2, [], tick)
