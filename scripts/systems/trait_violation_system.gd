## scripts/systems/trait_violation_system.gd
## Phase 3B — CK3식 Trait 반대행동 시스템
##
## 설계 근거: Cognitive Dissonance Theory (Festinger, 1957)
## 게임 레퍼런스: CK3 Stress System, RimWorld Thought System, DF Value Belief System
## 작성일: 2026-02-18
##
## 학술 근거:
##   - Festinger (1957): Cognitive Dissonance
##   - Bandura (1999): Moral Disengagement
##   - McEwen (1998): Allostatic Load (탈감작/PTSD 분기 조건)
##   - Post (1992): Kindling Theory
##   - Tedeschi & Calhoun (1996): PTG
##   - DSM-5: Intrusive Thoughts / PTSD re-experiencing
##   - Weiner (1985): Attribution Theory (context modifier)
##   - Tangney et al. (2007): Relational Context in Moral Emotion
## 대안으로 고려:
##   - 단순 lookup 테이블 → 기각 (facet 강도, 맥락, 이력 반영 불가)
##   - stress_system에 통합 → 기각 (내부갈등과 외부스트레스는 다른 경로)

extends "res://scripts/core/simulation_system.gd"

# ── 상수 ──────────────────────────────────────────────────────────────────────

## 탈감작: 반복 위반 시 스트레스 배수 감소율 (Bandura 1999)
const DESENSITIZE_DECAY: float = 0.85
## 탈감작 최소값 (30%까지 — 완전 무감각은 되지 않음)
const DESENSITIZE_MIN: float = 0.30
## PTSD 누적: 고스트레스 상태에서 반복 위반 시 민감도 증가 (Kindling Theory)
const PTSD_INCREASE: float = 1.10
## PTSD 최대 민감화 배수
const PTSD_MAX: float = 2.00
## 탈감작/PTSD 분기점 — allostatic_load 기준 (McEwen 1998)
const ALLOSTATIC_THRESHOLD: float = 0.50
## Intrusive thought 기본 발동 확률/tick
const INTRUSIVE_BASE_CHANCE: float = 0.005
## violation_history 감쇠 시작 tick 수 (1년 = 365일 × 12 tick/day)
const VIOLATION_HISTORY_DECAY_TICKS: int = 365 * 12
## severe 반응 임계값
const SEVERE_THRESHOLD: float = 30.0
## moderate 반응 임계값
const MODERATE_THRESHOLD: float = 15.0
## PTG 누적 스트레스 임계값 (30 tick 내)
const PTG_STRESS_THRESHOLD: float = 100.0
## PTG 발동을 위한 최대 allostatic_load
const PTG_ALLOSTATIC_MAX: float = 0.60
## 감정 기준선 황폐화 (DF desensitization side effect)
const DESENSITIZE_EMOTION_PENALTY: float = 0.005

# ── Context Modifiers ──────────────────────────────────────────────────────────
## Attribution Theory (Weiner, 1985): 귀인 방식이 죄책감 강도 결정
## RimWorld: forced/willing 구분
const CONTEXT_MODIFIERS: Dictionary = {
	"forced_by_authority": 0.5,
	"survival_necessity": 0.4,
	"repeated_habit": 0.0,
	"no_witness": 0.85,
}

# ── Witness Modifiers ──────────────────────────────────────────────────────────
## Tangney et al. (2007): 목격자 관계 강도가 수치심 결정
## RimWorld: 관계 depth에 따른 mood penalty
const WITNESS_MODIFIERS: Dictionary = {
	"none": 0.85,
	"stranger": 1.1,
	"acquaintance": 1.3,
	"friend": 1.6,
	"family": 1.9,
	"partner": 2.1,
	"own_child": 2.8,
}

# ── Victim Relationship Modifiers ──────────────────────────────────────────────
## DF: 자식/파트너 관련 이벤트가 가장 강한 mood hit
const VICTIM_MODIFIERS: Dictionary = {
	"enemy": 0.3,
	"stranger": 0.8,
	"acquaintance": 1.0,
	"friend": 1.4,
	"family": 1.8,
	"partner": 2.0,
	"own_child": 2.8,
	"parent": 2.2,
}

# ── 멤버 변수 ──────────────────────────────────────────────────────────────────
var _trait_defs_indexed: Dictionary = {}  # {trait_id: trait_def} — 빠른 lookup용
var _action_violation_map: Dictionary = {}  # {action_id: [{trait_id, base_stress}]} — 미리 계산
var _stress_system = null          # 참조 주입 (main.gd)
var _trauma_scar_system = null     # Phase 3A 연동 (참조 주입)
var _entity_manager = null         # 참조 주입
var _rng: RandomNumberGenerator = RandomNumberGenerator.new()

# PTG 누적 추적: {entity_id: {tick_start, accumulated_stress}}
var _ptg_tracker: Dictionary = {}


# ── 초기화 ─────────────────────────────────────────────────────────────────────

func _init() -> void:
	system_name = "trait_violation"
	priority = 37
	tick_interval = 1


## 의존성 주입
func init(deps: Dictionary) -> void:
	priority = 37
	tick_interval = 1
	_entity_manager = deps.get("entity_manager")
	_rng.randomize()
	_load_trait_defs()


## trait_definitions_fixed.json 로드 + action → violation_stress 역색인 구축
func _load_trait_defs() -> void:
	var path = "res://data/personality/trait_definitions_fixed.json"
	var file = FileAccess.open(path, FileAccess.READ)
	if file == null:
		push_error("[TraitViolationSystem] Cannot open %s" % path)
		return
	var json = JSON.new()
	var err: int = json.parse(file.get_as_text())
	file.close()
	if err != OK:
		push_error("[TraitViolationSystem] JSON parse error in %s" % path)
		return
	var defs = json.get_data()
	if not defs is Array:
		push_error("[TraitViolationSystem] trait_definitions_fixed.json must be Array")
		return

	# 배열 → dict 색인
	for tdef in defs:
		var tid = tdef.get("id", "")
		if tid.is_empty():
			continue
		_trait_defs_indexed[tid] = tdef

	# action → trait violation_stress 역색인 구축
	for tid in _trait_defs_indexed:
		var tdef = _trait_defs_indexed[tid]
		var effects: Dictionary = tdef.get("effects", {})
		var stress_mods: Dictionary = effects.get("stress_modifiers", {})
		var viol: Dictionary = stress_mods.get("violation_stress", {})
		if viol.is_empty():
			continue
		for action_id in viol:
			if not _action_violation_map.has(action_id):
				_action_violation_map[action_id] = []
			_action_violation_map[action_id].append({
				"trait_id": tid,
				"base_stress": float(viol[action_id]),
			})

	print("[TraitViolationSystem] Loaded %d traits, %d action mappings" % [
		_trait_defs_indexed.size(), _action_violation_map.size()
	])


# ── 메인 공개 함수 ─────────────────────────────────────────────────────────────

## 에이전트가 action을 실행했을 때 호출 (behavior_system에서)
## action_id: 수행된 행동 ID (예: "lie", "torture")
## context: {
##   "forced_by_authority": bool,
##   "survival_necessity": bool,
##   "witness_relationship": String,  # "none"/"stranger"/"friend"/"family"/"partner"/"own_child"
##   "victim_relationship": String,   # "enemy"/"stranger"/"friend"/...
##   "is_habit": bool,
##   "tick": int,
## }
func on_action_performed(entity: RefCounted, action_id: String, context: Dictionary) -> void:
	if _action_violation_map.is_empty():
		return
	var violations = _action_violation_map.get(action_id, [])
	if violations.is_empty():
		return

	var tick = context.get("tick", 0)
	var total_stress: float = 0.0

	# context modifier 계산
	var ctx_mult: float = _calc_context_modifier(context)
	# 습관적 반복이면 스트레스 면제 (탈감작 완전 경로)
	if ctx_mult <= 0.0:
		_update_violation_history(entity, action_id, tick, true)
		return

	# witness/victim modifier
	var witness_rel = context.get("witness_relationship", "none")
	var victim_rel = context.get("victim_relationship", "stranger")
	var witness_mult: float = WITNESS_MODIFIERS.get(witness_rel, 1.0)
	var victim_mult: float = VICTIM_MODIFIERS.get(victim_rel, 1.0)

	for viol_entry in violations:
		var trait_id: String = viol_entry["trait_id"]
		var base_stress: float = viol_entry["base_stress"]

		# 에이전트가 해당 trait를 보유하는지 확인
		if not _entity_has_trait(entity, trait_id):
			continue

		# facet 강도에 따른 스케일 (극단적일수록 강화)
		var facet_scale: float = _calc_facet_scale(entity, trait_id)

		# violation_history에서 탈감작/PTSD 배수 가져오기
		var hist_mults = _get_history_mults(entity, action_id)
		var desensitize_mult: float = hist_mults[0]
		var ptsd_mult: float = hist_mults[1]

		# 최종 스트레스 계산
		var stress: float = base_stress * facet_scale * desensitize_mult * ptsd_mult
		stress *= ctx_mult * witness_mult * victim_mult
		stress = maxf(stress, 0.0)
		total_stress += stress

	if total_stress <= 0.0:
		_update_violation_history(entity, action_id, tick, false)
		return

	# violation_history 업데이트 (탈감작/PTSD 분기)
	_update_violation_history(entity, action_id, tick, false)

	# 스트레스 주입
	_inject_violation_stress(entity, action_id, total_stress, tick)

	# PTG 추적
	_track_ptg_stress(entity, total_stress, tick)

	# breakdown 계층화 로그
	var severity: String = "minor"
	if total_stress >= SEVERE_THRESHOLD:
		severity = "severe"
	elif total_stress >= MODERATE_THRESHOLD:
		severity = "moderate"

	print("[TraitViolationSystem] %s violated trait via '%s': stress=%.1f (%s)" % [
		entity.entity_name, action_id, total_stress, severity
	])

	# severe 반응: Phase 3A scar 확률 +10%
	if severity == "severe" and _trauma_scar_system != null:
		if _trauma_scar_system.has_method("notify_severe_violation"):
			_trauma_scar_system.notify_severe_violation(entity, action_id, tick)


# ── Tick 처리 ─────────────────────────────────────────────────────────────────

## 매 tick: intrusive thought + violation_history 시간 감쇠
func execute_tick(tick: int) -> void:
	if _entity_manager == null:
		return
	var entities = _entity_manager.get_alive_entities()
	for entity in entities:
		if not entity.is_alive:
			continue
		if not entity.has_method("get") or not ("violation_history" in entity):
			continue
		_decay_violation_history(entity, tick)
		_process_intrusive_thoughts(entity, tick)
		_check_ptg(entity, tick)


# ── Intrusive Thought ──────────────────────────────────────────────────────────

## PTSD re-experiencing (DSM-5): 과거 severe 위반이 불시에 플래시백
## DF 참고: intrusive thought 시스템
func _process_intrusive_thoughts(entity: RefCounted, current_tick: int) -> void:
	var hist: Dictionary = entity.violation_history
	if hist.is_empty():
		return

	for action_id in hist:
		var record = hist[action_id]
		var ptsd_mult: float = float(record.get("ptsd_mult", 1.0))
		# ptsd_mult > 1.4인 경우만 (심각한 PTSD 누적)
		if ptsd_mult < 1.4:
			continue

		# 시간 경과에 따른 확률 감소 (지수 감쇠)
		var ticks_since: int = current_tick - int(record.get("last_tick", current_tick))
		var decay_factor: float = exp(-float(ticks_since) / float(VIOLATION_HISTORY_DECAY_TICKS))
		var chance: float = INTRUSIVE_BASE_CHANCE * (ptsd_mult - 1.0) * decay_factor

		# Phase 3A trauma_scar 있으면 확률 2배
		if entity.has("trauma_scars") and not entity.trauma_scars.is_empty():
			chance *= 2.0

		if _rng.randf() < chance:
			var stress_amount: float = 15.0 + _rng.randf() * 25.0  # 15~40
			_inject_violation_stress(entity, action_id, stress_amount, current_tick)
			print("[TraitViolationSystem] %s intrusive thought: '%s' stress=%.1f" % [
				entity.entity_name, action_id, stress_amount
			])
			if SimulationBus.has_signal("simulation_event"):
				SimulationBus.emit_event("violation_intrusive_thought", {
					"entity_id": entity.id,
					"entity_name": entity.entity_name,
					"action_id": action_id,
					"stress": stress_amount,
					"tick": current_tick,
				})


# ── violation_history 시간 감쇠 ───────────────────────────────────────────────

## Ebbinghaus Forgetting Curve: 기억은 시간이 지나면 희미해진다
## 오래 전 위반의 탈감작 효과가 복귀 → 다시 죄책감을 느낄 수 있게 됨
func _decay_violation_history(entity: RefCounted, current_tick: int) -> void:
	var hist: Dictionary = entity.violation_history
	if hist.is_empty():
		return

	var to_remove: Array = []
	for action_id in hist:
		var record = hist[action_id]
		var ticks_since: int = current_tick - int(record.get("last_tick", current_tick))

		# 1년(VIOLATION_HISTORY_DECAY_TICKS) 경과 후 감쇠 시작
		if ticks_since <= VIOLATION_HISTORY_DECAY_TICKS:
			continue

		var count = record.get("count", 0.0)
		count = maxf(float(count) - 0.01, 0.0)
		record["count"] = count

		# desensitize_mult → 1.0으로 복귀
		var dm = float(record.get("desensitize_mult", 1.0))
		record["desensitize_mult"] = lerpf(dm, 1.0, 0.001)

		# ptsd_mult → 1.0으로 복귀
		var pm = float(record.get("ptsd_mult", 1.0))
		record["ptsd_mult"] = lerpf(pm, 1.0, 0.001)

		if count < 0.1:
			to_remove.append(action_id)

	for action_id in to_remove:
		hist.erase(action_id)


# ── Post-Traumatic Growth ──────────────────────────────────────────────────────

## Tedeschi & Calhoun (1996): 극단적 고통 후 역설적 성장
## RimWorld의 Inspiration 상태와 유사
func _check_ptg(entity: RefCounted, current_tick: int) -> void:
	if not _ptg_tracker.has(entity.id):
		return
	var tracker = _ptg_tracker[entity.id]
	# 30 tick 이상 지났으면 리셋
	if current_tick - int(tracker.get("tick_start", current_tick)) > 30:
		_ptg_tracker.erase(entity.id)
		return


## PTG 발동 여부 판단 (on_action_performed에서 호출)
func _track_ptg_stress(entity: RefCounted, stress: float, current_tick: int) -> void:
	if not _ptg_tracker.has(entity.id):
		_ptg_tracker[entity.id] = {"tick_start": current_tick, "accumulated_stress": 0.0}
	var tracker = _ptg_tracker[entity.id]

	# 30 tick 넘었으면 리셋
	if current_tick - int(tracker.get("tick_start", current_tick)) > 30:
		tracker["tick_start"] = current_tick
		tracker["accumulated_stress"] = 0.0

	tracker["accumulated_stress"] = float(tracker.get("accumulated_stress", 0.0)) + stress

	# PTG 조건: 누적 > 100, allostatic_load < 0.6
	if float(tracker.get("accumulated_stress", 0.0)) < PTG_STRESS_THRESHOLD:
		return
	var allostatic: float = _get_allostatic_load(entity)
	if allostatic >= PTG_ALLOSTATIC_MAX:
		return

	_apply_ptg(entity, current_tick)
	_ptg_tracker.erase(entity.id)


## PTG 효과 적용 (trait별 다른 성장)
func _apply_ptg(entity: RefCounted, current_tick: int) -> void:
	var ptg_type: String = ""
	# f_creative → 창작 행동 가중치 일시 부스트
	if _entity_has_trait(entity, "f_creative"):
		ptg_type = "creative"
	elif _entity_has_trait(entity, "f_curious"):
		ptg_type = "curious"
	elif _entity_has_trait(entity, "f_forgiving"):
		ptg_type = "forgiving"
	else:
		return  # PTG 성향 없음

	print("[TraitViolationSystem] %s Post-Traumatic Growth: %s" % [entity.entity_name, ptg_type])
	if SimulationBus.has_signal("simulation_event"):
		SimulationBus.emit_event("violation_ptg", {
			"entity_id": entity.id,
			"entity_name": entity.entity_name,
			"ptg_type": ptg_type,
			"tick": current_tick,
		})


# ── Settlement Norm 인터페이스 (Phase 4 stub) ──────────────────────────────────

## Phase 4에서 구현 예정: 정착지 집단 규범 기반 수정자
## RimWorld Ideology system — 집단 규범이 개인 도덕 판단을 조정
## norm = 0.0~1.0 (0=금기, 1=허용) → modifier = 1.0 + (0.5 - norm)
func _get_settlement_norm_modifier(_settlement, _action: String) -> float:
	# TODO Phase 4
	return 1.0


# ── 내부 헬퍼 ─────────────────────────────────────────────────────────────────

## context dict → multiplier
func _calc_context_modifier(context: Dictionary) -> float:
	if context.get("is_habit", false):
		return CONTEXT_MODIFIERS.get("repeated_habit", 0.0)
	var mult: float = 1.0
	if context.get("forced_by_authority", false):
		mult *= CONTEXT_MODIFIERS.get("forced_by_authority", 0.5)
	if context.get("survival_necessity", false):
		mult *= CONTEXT_MODIFIERS.get("survival_necessity", 0.4)
	var witness_rel = context.get("witness_relationship", "")
	if witness_rel == "none" or witness_rel.is_empty():
		mult *= CONTEXT_MODIFIERS.get("no_witness", 0.85)
	return mult


## entity가 주어진 trait를 활성화하고 있는지 확인
## entity.active_traits: 활성화된 trait 정의 Dictionary 배열
func _entity_has_trait(entity: RefCounted, trait_id: String) -> bool:
	if not entity.has("active_traits"):
		return false
	var traits: Array = entity.active_traits
	for t in traits:
		if t.get("id", "") == trait_id:
			return true
	return false


## facet 강도 비례 스케일 계산
## facet이 극단적일수록(예: 0.98) 더 강한 죄책감
func _calc_facet_scale(entity: RefCounted, trait_id: String) -> float:
	if not _trait_defs_indexed.has(trait_id):
		return 1.0
	var tdef = _trait_defs_indexed[trait_id]
	var axis: String = tdef.get("axis", "")
	if axis.is_empty():
		return 1.0
	if not entity.has("personality") or entity.personality == null:
		return 1.0
	var facet_val: float = 0.5
	if entity.personality.has_method("get_facet_value"):
		facet_val = entity.personality.get_facet_value(axis)
	elif entity.personality.has("facets"):
		var facets: Dictionary = entity.personality.facets
		facet_val = float(facets.get(axis, 0.5))
	var threshold: float = 0.6
	if facet_val <= threshold:
		return 1.0
	return (facet_val - threshold) / (1.0 - threshold)


## violation_history에서 탈감작/PTSD 배수 반환 [desensitize_mult, ptsd_mult]
func _get_history_mults(entity: RefCounted, action_id: String) -> Array:
	if not entity.has("violation_history"):
		return [1.0, 1.0]
	var record = entity.violation_history.get(action_id, {})
	var dm: float = float(record.get("desensitize_mult", 1.0))
	var pm: float = float(record.get("ptsd_mult", 1.0))
	return [dm, pm]


## violation_history 업데이트: 탈감작 or PTSD 분기
## Bandura (1999): 반복 노출이 도덕적 자기검열을 약화
## McEwen (1998): allostatic_load 기반 분기
func _update_violation_history(entity: RefCounted, action_id: String, tick: int, is_habit: bool) -> void:
	if not entity.has("violation_history"):
		return

	if not entity.violation_history.has(action_id):
		entity.violation_history[action_id] = {
			"count": 0.0,
			"desensitize_mult": 1.0,
			"ptsd_mult": 1.0,
			"last_tick": tick,
		}

	var record = entity.violation_history[action_id]
	record["count"] = float(record.get("count", 0.0)) + 1.0
	record["last_tick"] = tick

	if is_habit:
		return

	var allostatic: float = _get_allostatic_load(entity)

	if allostatic < ALLOSTATIC_THRESHOLD:
		# 탈감작 경로 (Bandura 1999): 반복 → 무뎌짐
		var dm: float = float(record.get("desensitize_mult", 1.0))
		dm = maxf(dm * DESENSITIZE_DECAY, DESENSITIZE_MIN)
		record["desensitize_mult"] = dm
		# DF 황폐화: 감정 기준선 하락
		_apply_desensitization_side_effect(entity)
	else:
		# PTSD 누적 경로 (Kindling Theory): 소진 상태에서 반복 → 더 깊은 상처
		var pm: float = float(record.get("ptsd_mult", 1.0))
		pm = minf(pm * PTSD_INCREASE, PTSD_MAX)
		record["ptsd_mult"] = pm


## allostatic_load 추정값 반환 (stress_system의 값 미러링)
## stress_system 없으면 emotions.stress로 근사
func _get_allostatic_load(entity: RefCounted) -> float:
	if _stress_system != null and _stress_system.has_method("get_allostatic_load"):
		return _stress_system.get_allostatic_load(entity)
	if entity.has("emotions"):
		return clampf(float(entity.emotions.get("stress", 0.0)), 0.0, 1.0)
	return 0.0


## DF 황폐화: 탈감작 시 joy/trust 기준선 소폭 하락
func _apply_desensitization_side_effect(entity: RefCounted) -> void:
	if not entity.has("emotions"):
		return
	var joy: float = float(entity.emotions.get("happiness", 0.5))
	entity.emotions["happiness"] = maxf(joy - DESENSITIZE_EMOTION_PENALTY, 0.0)


## stress_system.inject_event()를 통해 violation 스트레스 주입
func _inject_violation_stress(entity: RefCounted, action_id: String, stress: float, tick: int) -> void:
	if _stress_system == null:
		# stress_system 없으면 직접 emotions.stress 수정
		if entity.has("emotions"):
			var s: float = float(entity.emotions.get("stress", 0.0))
			entity.emotions["stress"] = minf(s + stress * 0.01, 1.0)
		return
	if _stress_system.has_method("inject_event"):
		_stress_system.inject_event(entity, "violation_" + action_id, {
			"base_intensity": stress,
			"tick": tick,
			"source": "trait_violation",
		})
	elif _stress_system.has_method("inject_stress"):
		_stress_system.inject_stress(entity, stress, tick)
