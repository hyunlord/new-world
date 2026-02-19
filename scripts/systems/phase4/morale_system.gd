extends "res://scripts/core/simulation_system.gd"

# NO class_name — headless compatibility
# Phase 4: Morale System
# Implements personal and settlement morale based on psychological research

var _entity_manager: RefCounted
var _cfg: Dictionary = {}                   # morale_config.json
var _personal_morale: Dictionary = {}       # entity_id -> float (-1~1)
var _settlement_morale: Dictionary = {}     # settlement_id -> Dictionary
var _grievance: Dictionary = {}             # settlement_id -> float (0~1)
var _expectation_base: Dictionary = {}      # entity_id -> float (hedonic treadmill baseline)


func _init() -> void:
	system_name = "morale"
	priority = 40   # after contagion(38)
	tick_interval = 5   # every 5 ticks


func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_load_config()


func _load_config() -> void:
	var path: String = "res://data/morale_config.json"
	if not FileAccess.file_exists(path):
		_cfg = {}
		return

	var file = FileAccess.open(path, FileAccess.READ)
	if file == null:
		_cfg = {}
		return

	var json = JSON.new()
	var parse_err: int = json.parse(file.get_as_text())
	if parse_err != OK or typeof(json.data) != TYPE_DICTIONARY:
		_cfg = {}
		return

	_cfg = json.data


func execute_tick(tick: int) -> void:
	if _entity_manager == null:
		return

	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity = alive[i]
		var eid: int = entity.id
		var morale: float = calculate_personal_morale(entity)
		_personal_morale[eid] = morale
		tick_hedonic_treadmill(entity, morale)

	# Settlement aggregation every 20 ticks
	if tick % 20 == 0:
		_aggregate_settlement_morales()

	# Grievance update every tick (using settlement morale)
	if tick % 5 == 0:
		_update_all_grievances(tick)


## [Diener, Emmons, Larsen & Griffin, 1985 - Subjective Well-Being (SWB)]
## SWB 3성분 모델: 긍정 정서(PA) + 부정 정서(NA) + 삶의 만족도(LS).
## morale_personal = 0.40*PA - 0.30*NA + 0.30*LS
## * Maslow multiplier (욕구 충족 여부) * Herzberg 위생 요인 보정
## * Warr Vitamin Model 기여 - Hedonic Treadmill 기대값
## Reference: Diener, E. et al. (1985). Satisfaction with Life Scale. Journal of Personality Assessment, 49(1).
func calculate_personal_morale(entity) -> float:
	var ed = entity.emotion_data
	if ed == null:
		return 0.0

	# Maslow 최우선 차단
	var maslow_mult: float = _get_maslow_multiplier(entity)
	if maslow_mult == 0.0:
		# 기아/수면 부족 → 감정 마비
		_set_maslow_blocked_message(entity)
		return 0.0

	# PA: valence 기반 (0~1 정규화)
	var PA: float = clampf((ed.valence + 100.0) / 200.0, 0.0, 1.0)
	# NA: negative affect (arousal 높고 valence 낮을 때)
	var neg_arousal: float = clampf(ed.arousal / 100.0, 0.0, 1.0)
	var neg_valence: float = clampf((-ed.valence + 100.0) / 200.0, 0.0, 1.0)
	var NA: float = neg_arousal * neg_valence
	# LS: 욕구 충족 기반 삶의 만족도 proxy
	var LS: float = (entity.hunger + entity.energy + entity.social) / 3.0

	var weights = _cfg.get("personal_morale", {})
	var pa_w: float = float(weights.get("affect_positive_weight", 0.40))
	var na_w: float = float(weights.get("affect_negative_weight", 0.30))
	var ls_w: float = float(weights.get("life_satisfaction_weight", 0.30))

	var base: float = pa_w * PA - na_w * NA + ls_w * LS
	base *= maslow_mult

	# Herzberg 위생 요인 (미충족 시만 패널티)
	base += _apply_hygiene_factors(entity)

	# Warr Vitamin Model
	base += _calculate_warr_contributions(entity)

	# Hedonic Treadmill 기대값 차감
	var eid: int = entity.id
	var expectation: float = _expectation_base.get(eid, 0.5)
	base -= (expectation - 0.5) * 0.2

	return clampf(base, -1.0, 1.0)


## [Maslow, 1943 - A Theory of Human Motivation]
## 욕구계층: 하위 욕구(생리/안전)가 미충족이면 상위 욕구(사회/자아)는 차단.
## 기아/수면 부족 → 생리 욕구 미충족 → 감정 마비 (multiplier=0).
## 안전 위협 → 안전 욕구 미충족 → 감정 억제 (multiplier=0.2~0.6).
## Reference: Maslow, A.H. (1943). A theory of human motivation. Psychological Review, 50(4), 370-396.
func _get_maslow_multiplier(entity) -> float:
	var mcfg = _cfg.get("personal_morale", {}).get("maslow", {})
	var hunger: float = entity.hunger
	var energy: float = entity.energy

	# 생리: 기아/수면 부족
	var food_sleep_critical: float = float(mcfg.get("food_sleep_critical", 0.3))
	var food_sleep_low: float = float(mcfg.get("food_sleep_low", 0.6))
	if hunger < food_sleep_critical or energy < food_sleep_critical:
		return float(mcfg.get("multiplier_food_sleep_critical", 0.0))
	if hunger < food_sleep_low or energy < food_sleep_low:
		return float(mcfg.get("multiplier_food_sleep_low", 0.4))

	# 안전: settlement_id < 0 → homeless → safety threat
	var safety_level: float = 1.0 if entity.settlement_id >= 0 else 0.2
	var safety_critical: float = float(mcfg.get("safety_critical", 0.3))
	var safety_low: float = float(mcfg.get("safety_low", 0.6))
	if safety_level < safety_critical:
		return float(mcfg.get("multiplier_safety_critical", 0.2))
	if safety_level < safety_low:
		return float(mcfg.get("multiplier_safety_low", 0.6))

	# 소속: social < 0.3
	var social: float = entity.social
	var belonging_low: float = float(mcfg.get("belonging_low", 0.3))
	if social < belonging_low:
		return float(mcfg.get("multiplier_belonging_low", 0.7))

	return 1.0


## [Herzberg, Mausner & Snyderman, 1959 - Two-Factor Theory]
## 위생 요인(Hygiene Factors): 충족해도 만족 증가 없음. 미충족 시만 불만족 패널티.
## 충족 기준(threshold=0.5): 안전, 소속, 기본 욕구 충족 여부.
## 미충족 시 penalty_rate=0.8 비율로 모랄 감소.
## Reference: Herzberg, F. et al. (1959). The Motivation to Work. Wiley.
func _apply_hygiene_factors(entity) -> float:
	var mcfg = _cfg.get("personal_morale", {})
	var threshold: float = float(mcfg.get("herzberg_threshold", 0.5))
	var penalty_rate: float = float(mcfg.get("herzberg_penalty_rate", 0.8))
	var delta: float = 0.0

	# 안전(settlement 존재)
	var safety: float = 1.0 if entity.settlement_id >= 0 else 0.0
	if safety < threshold:
		delta -= (threshold - safety) * penalty_rate * 0.3

	# 소속(social)
	if entity.social < threshold:
		delta -= (threshold - entity.social) * penalty_rate * 0.2

	# 기본 욕구 (hunger, energy)
	if entity.hunger < threshold:
		delta -= (threshold - entity.hunger) * penalty_rate * 0.25
	if entity.energy < threshold:
		delta -= (threshold - entity.energy) * penalty_rate * 0.25

	# 위생 요인은 절대 양수가 될 수 없음 (충족해도 보너스 없음)
	return minf(delta, 0.0)


## [Warr, 1987 - Work, Unemployment, and Mental Health - Vitamin Model]
## CE형(Constant Effect): 일정 수준까지 선형 증가 후 캡 — 더 많아도 증가 없음.
## AD형(Additional Decrement): 포물선 — 최적값 이상 또는 이하에서 모두 감소.
## 직업 자율성, 사회 접촉, 정보 부하가 AD형 포물선 효과.
## Reference: Warr, P. (1987). Work, Unemployment, and Mental Health. Oxford University Press.
func _calculate_warr_contributions(entity) -> float:
	var vcfg = _cfg.get("warr_vitamin", {})
	var AD_defs = vcfg.get("AD", {})
	var CE_threshold: float = float(vcfg.get("CE_threshold", 0.7))
	var CE_max: float = float(vcfg.get("CE_max_contribution", 0.10))

	var total: float = 0.0

	# AD형: 자율성 proxy (C axis — 계획/성실성이 높을수록 자율 추구)
	var pd = entity.personality
	if pd != null and AD_defs.size() > 0:
		var autonomy_cfg = AD_defs.get("autonomy", {})
		var a: float = float(autonomy_cfg.get("a", 1.5))
		var h: float = float(autonomy_cfg.get("h", 0.6))
		var k: float = float(autonomy_cfg.get("k", 0.15))
		var autonomy_val: float = float(pd.axes.get("C", 0.5))
		total += -a * (autonomy_val - h) * (autonomy_val - h) + k

		# AD형: 사회 접촉
		var soc_cfg = AD_defs.get("social_contact", {})
		a = float(soc_cfg.get("a", 2.0))
		h = float(soc_cfg.get("h", 0.5))
		k = float(soc_cfg.get("k", 0.12))
		var soc_val: float = entity.social
		total += -a * (soc_val - h) * (soc_val - h) + k

	# CE형: 정보 부하 (X axis — 외향성이 높을수록 자극 추구)
	if pd != null:
		var x_val: float = float(pd.axes.get("X", 0.5))
		var ce: float = minf(x_val / CE_threshold, 1.0) * CE_max
		total += ce

	return clampf(total, -0.3, 0.3)


## [Huppert & So, 2013 - Flourishing in Europe]
## Flourishing 임계(0.6 이상): 행동 효율이 선형으로 증가 (최대 1.55배).
## 보통 구간(0.3~0.6): 1.0 근방.
## Languishing(<0.3): 0.30~0.55배로 급격히 저하.
## Reference: Huppert, F.A. & So, T.T.C. (2013). Psychological Medicine, 43(1), 1-13.
func get_behavior_weight_multiplier(entity_id: int) -> float:
	var morale: float = _personal_morale.get(entity_id, 0.5)
	var bwm = _cfg.get("behavior_weight_multiplier", {})

	var flourishing_threshold: float = float(bwm.get("flourishing_threshold", 0.6))
	if morale >= flourishing_threshold:
		var f_min: float = float(bwm.get("flourishing_min", 1.2))
		var f_max: float = float(bwm.get("flourishing_max", 1.55))
		var slope: float = (f_max - f_min) / (1.0 - flourishing_threshold)
		return clampf(f_min + (morale - flourishing_threshold) * slope, f_min, f_max)

	var normal_max: float = float(bwm.get("normal_max", 1.2))
	var normal_min: float = float(bwm.get("normal_min", 0.85))
	if morale >= 0.3:
		var t: float = (morale - 0.3) / (flourishing_threshold - 0.3)
		return lerpf(normal_min, normal_max, t)

	var dis_max: float = float(bwm.get("dissatisfied_max", 0.85))
	var dis_min: float = float(bwm.get("dissatisfied_min", 0.55))
	if morale >= 0.0:
		var t: float = morale / 0.3
		return lerpf(dis_min, dis_max, t)

	# Languishing
	var lang_min: float = float(bwm.get("languishing_min", 0.30))
	var lang_max: float = float(bwm.get("languishing_max", 0.55))
	var t: float = clampf((morale + 1.0) / 1.0, 0.0, 1.0)
	return lerpf(lang_min, lang_max, t)


## [Brickman & Campbell, 1971 - Hedonic Relativism and Planning the Good Society]
## 쾌락 쳇바퀴: 모랄이 높으면 기대값이 천천히 상승 → 같은 수준이 "평범"하게 느껴짐.
## rate=0.002/tick: 현재 모랄의 0.2%씩 기대값 수렴.
## Reference: Brickman, P. & Campbell, D.T. (1971). In M.H. Appley (Ed.), Adaptation-level theory. Academic Press.
func tick_hedonic_treadmill(entity, current_morale: float) -> void:
	var eid: int = entity.id
	var mcfg = _cfg.get("personal_morale", {})
	var rate: float = float(mcfg.get("hedonic_treadmill_rate", 0.002))
	var threshold: float = float(mcfg.get("hedonic_treadmill_threshold", 0.5))
	var cap: float = float(mcfg.get("hedonic_treadmill_cap", 0.8))

	var current_base: float = _expectation_base.get(eid, 0.5)
	# morale > threshold면 기대값 상승
	if current_morale > threshold:
		current_base = lerpf(current_base, minf(current_morale, cap), rate)
	else:
		# 저모랄 → 기대값 천천히 하강
		current_base = lerpf(current_base, maxf(current_morale, 0.2), rate * 0.5)
	_expectation_base[eid] = current_base


## [Clark & Oswald, 1994 - Unhappiness and Unemployment]
## 정착지 모랄이 낮을수록 이주 확률 상승. k=+10 (양수): morale 낮으면 확률 ↑.
## 버그 수정 주석: 원래 k=-8.0이었으나 이 경우 morale 높을수록 이주 확률 상승 (반대!).
## k를 양수로 수정하여 올바른 방향 보장.
## Reference: Clark, A.E. & Oswald, A.J. (1994). Economic Journal, 104(424), 648-659.
func get_migration_probability(entity_id: int) -> float:
	var sid: int = _get_settlement_id_for_entity(entity_id)
	if sid < 0:
		return 0.0
	var sm_data = _settlement_morale.get(sid, {})
	var morale_s: float = sm_data.get("morale_true", 0.5)

	var mcfg = _cfg.get("migration", {})
	var k: float = float(mcfg.get("k", 10.0))           # 양수 — morale 낮으면 확률 ↑
	var m0: float = float(mcfg.get("threshold_morale", 0.35))
	var max_prob: float = float(mcfg.get("max_probability", 0.95))

	# sigmoid: p = 1/(1+exp(-k*(m0 - morale_s)))
	# morale_s=0.2 (낮음) → m0-morale_s=0.15>0 → sigmoid>0.5 → 높은 확률 ✅
	# morale_s=0.8 (높음) → m0-morale_s=-0.45<0 → sigmoid<0.5 → 낮은 확률 ✅
	var p_base: float = 1.0 / (1.0 + exp(-k * (m0 - morale_s)))

	# A_patience로 저항
	var patience: float = 0.5
	var entity = _entity_manager.get_entity(entity_id) if _entity_manager else null
	if entity and entity.personality:
		patience = float(entity.personality.facets.get("A_patience", 0.5))
	var resistance: float = float(mcfg.get("patience_resistance", 0.3)) * patience

	return clampf(p_base - resistance, 0.0, max_prob)


func _update_all_grievances(tick: int) -> void:
	for sid in _settlement_morale:
		var sm_data = _settlement_morale.get(sid, {})
		var morale_s: float = sm_data.get("morale_true", 0.5)
		_update_grievance(int(sid), morale_s, tick)


## [Gurr, 1970 - Why Men Rebel - Relative Deprivation Theory]
## 상대적 박탈감(grievance): 기대값과 현실 모랄의 차이가 클수록 grievance 상승.
## tau_rise=48틱: 저모랄 지속 시 grievance 상승 속도.
## tau_decay=192틱: grievance 자연 감소 속도 (4배 느림 — 한번 쌓이면 쉽게 안 꺼짐).
## Reference: Gurr, T.R. (1970). Why Men Rebel. Princeton University Press.
func _update_grievance(sid: int, morale_s: float, tick: int) -> void:
	var _unused_tick: int = tick
	var rcfg = _cfg.get("rebellion", {})
	var tau_rise: float = float(rcfg.get("grievance_rise_tau", 48.0))
	var tau_decay: float = float(rcfg.get("grievance_decay_tau", 192.0))

	var g: float = _grievance.get(sid, 0.0)
	var expectation_settlement: float = 0.5   # 정착지 평균 기대값

	# 기대값 대비 모랄 부족분
	var shortfall: float = maxf(0.0, expectation_settlement - morale_s)
	var g_next: float = clampf(
		g + (shortfall / tau_rise) - (g / tau_decay),
		0.0, 1.0
	)
	_grievance[sid] = g_next


## [Leadership & Group Dynamics] 리더 모랄의 집단 영향 (leader_weight=2.0).
## 표준편차 패널티: 구성원 간 모랄 편차가 클수록 집단 응집력 저하.
func _aggregate_settlement_morales() -> void:
	if _entity_manager == null:
		return

	var alive: Array = _entity_manager.get_alive_entities()
	var by_settlement: Dictionary = {}

	for i in range(alive.size()):
		var entity = alive[i]
		var sid: int = entity.settlement_id
		if sid < 0:
			continue
		if not by_settlement.has(sid):
			by_settlement[sid] = {"morales": [], "leader_morale": -999.0}
		var m: float = _personal_morale.get(entity.id, 0.5)
		by_settlement[sid]["morales"].append(m)

	var smcfg = _cfg.get("settlement_morale", {})
	var leader_weight: float = float(smcfg.get("leader_weight", 2.0))
	var std_penalty_rate: float = float(smcfg.get("std_dev_penalty_rate", 0.15))
	var _unused_leader_weight: float = leader_weight

	_settlement_morale.clear()
	for sid in by_settlement:
		var morales: Array = by_settlement[sid]["morales"]
		if morales.is_empty():
			continue

		# 단순 평균 (리더 가중은 생략, 리더 구분 불가)
		var sum_m: float = 0.0
		for m in morales:
			sum_m += m
		var avg: float = sum_m / float(morales.size())

		# 표준편차 패널티
		var variance: float = 0.0
		for m in morales:
			variance += (m - avg) * (m - avg)
		var std_dev: float = sqrt(variance / float(morales.size()))
		var std_penalty: float = std_dev * std_penalty_rate

		var raw: float = avg - std_penalty
		# morale_true: 반란/이주 판정용 (floor 없음)
		# morale: 생산/행동 계산용 (floor 0.10)
		_settlement_morale[sid] = {
			"morale_true": clampf(raw, -1.0, 1.0),
			"morale": maxf(raw, 0.10),
			"member_count": morales.size(),
		}


## [Gurr, 1970 / Clark & Oswald, 1994] grievance + Gini 불평등 기반 반란 확률
func check_rebellion_probability(sid: int, tick: int) -> float:
	var _unused_tick: int = tick
	var rcfg = _cfg.get("rebellion", {})
	var k: float = float(rcfg.get("k", 12.0))
	var threshold_g: float = float(rcfg.get("threshold_grievance", 0.35))
	var p_max: float = float(rcfg.get("weekly_max_probability", 0.12))
	var low_morale_threshold: float = float(rcfg.get("low_morale_threshold", 0.20))
	var low_morale_mult: float = float(rcfg.get("low_morale_multiplier", 1.5))

	var g: float = _grievance.get(sid, 0.0)
	var sm_data = _settlement_morale.get(sid, {})
	var morale_s: float = sm_data.get("morale_true", 0.5)

	# Gini 근사 (단순화: 표준편차 기반)
	var gini_proxy: float = 0.3   # placeholder

	var p: float = p_max * (1.0 / (1.0 + exp(-k * (g - threshold_g))))
	p *= (0.5 + 0.5 * gini_proxy)

	if morale_s < low_morale_threshold:
		p *= low_morale_mult

	return clampf(p, 0.0, p_max)


func _set_maslow_blocked_message(entity) -> void:
	var reason: String
	if entity.hunger < 0.3:
		reason = Locale.ltr("MASLOW_BLOCKED_FOOD")
	elif entity.energy < 0.3:
		reason = Locale.ltr("MASLOW_BLOCKED_SLEEP")
	elif entity.settlement_id < 0:
		reason = Locale.ltr("MASLOW_BLOCKED_SAFETY")
	else:
		reason = Locale.ltr("MASLOW_BLOCKED_SAFETY")
	entity.set_meta("morale_blocked_reason", reason)


func get_personal_morale(entity_id: int) -> float:
	return _personal_morale.get(entity_id, 0.5)


func get_settlement_morale(settlement_id: int) -> float:
	return _settlement_morale.get(settlement_id, {}).get("morale", 0.5)


func get_settlement_morale_true(settlement_id: int) -> float:
	return _settlement_morale.get(settlement_id, {}).get("morale_true", 0.5)


func get_morale_state_label(entity_id: int) -> String:
	var m: float = _personal_morale.get(entity_id, 0.5)
	if m >= 0.6:
		return Locale.ltr("MORALE_STATE_FLOURISHING")
	elif m >= 0.3:
		return Locale.ltr("MORALE_STATE_SATISFIED")
	elif m >= 0.0:
		return Locale.ltr("MORALE_STATE_NEUTRAL")
	elif m >= -0.3:
		return Locale.ltr("MORALE_STATE_DISSATISFIED")
	else:
		return Locale.ltr("MORALE_STATE_LANGUISHING")


func _get_settlement_id_for_entity(entity_id: int) -> int:
	var entity = _entity_manager.get_entity(entity_id) if _entity_manager else null
	if entity == null:
		return -1
	return entity.settlement_id
