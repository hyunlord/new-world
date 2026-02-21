## [Schwartz 1992, Axelrod 1997, Kohlberg 1969, Festinger 1957, Erikson 1950]
## 가치관 시스템 — 33개 가치관의 형성, 변화, 충돌 해소
## 참조: const ValueSystem = preload("res://scripts/systems/value_system.gd")
extends RefCounted

const ValueDefs = preload("res://scripts/core/value_defs.gd")

var _entity_manager = null
var _rng: RandomNumberGenerator = RandomNumberGenerator.new()

## ── 형성 비율 상수 ────────────────────────────────────────
const GENETIC_WEIGHT: float = 0.15
const CULTURE_WEIGHT: float = 0.40
const HEXACO_WEIGHT: float = 0.20
const NOISE_WEIGHT: float = 0.25

## [Axelrod 1997] 1회 상호작용당 최대 문화 드리프트
const CULTURE_DRIFT_PER_INTERACTION: float = 0.02

## 경험 이벤트 1회 최대 delta
const EVENT_MAX_DELTA: float = 0.30

## [Festinger 1957] 위반 횟수 기준 자기합리화 발동
const VIOLATION_RATIONALIZE_COUNT: int = 3
const RATIONALIZATION_DELTA: float = 0.05


## ── 3.1 HEXACO → 가치관 시드 ────────────────────────────
## facet(0~1) → 가치관 시드(-1~+1)
## z(f) = 2*(f-0.5) — facet을 대칭 범위로 변환
static func compute_hexaco_seed(hexaco: Dictionary) -> Dictionary:
	var seed: Dictionary = {}
	for vkey in ValueDefs.KEYS:
		var mapping = ValueDefs.HEXACO_SEED_MAP.get(vkey, {})
		var raw: float = 0.0
		var total_weight: float = 0.0
		for facet_name in mapping:
			var w: float = mapping[facet_name]
			var f: float = hexaco.get(facet_name, 0.5)
			var z: float = clampf(2.0 * (f - 0.5), -1.0, 1.0)
			raw += z * w
			total_weight += absf(w)
		if total_weight > 0.0:
			raw /= total_weight
		seed[vkey] = clampf(raw, -1.0, 1.0)
	return seed


## SimulationEngine 등록용 — 시스템 초기화
func init(entity_manager) -> void:
	_entity_manager = entity_manager
	_rng.randomize()


## SimulationEngine 등록용 — 실행 우선순위 (family_system=52 다음)
func get_priority() -> int:
	return 55


## SimulationEngine 등록용 — tick 200마다 실행
func get_tick_interval() -> int:
	return 200


## tick마다 호출: Kohlberg 진급 + peer influence 1회
func update(_delta: float) -> void:
	if _entity_manager == null:
		return
	var entities: Array = _entity_manager.get_all_alive()
	for entity in entities:
		if entity.values.is_empty():
			continue
		var age_years: float = entity.age_days / 365.0
		var hexaco_dict: Dictionary = _get_hexaco_dict(entity.personality)

		# [Kohlberg 1969] 도덕 발달 단계 진급 체크
		check_moral_stage_progression(entity, hexaco_dict, age_years)

		# [Axelrod 1997] peer influence — 같은 정착지 무작위 1명과 가치관 수렴
		if "settlement_id" in entity and entity.settlement_id >= 0:
			var neighbors: Array = _entity_manager.get_entities_in_settlement(
				entity.settlement_id
			)
			if neighbors.size() > 1:
				var other = neighbors[_rng.randi() % neighbors.size()]
				if other.id != entity.id and not other.values.is_empty():
					apply_peer_influence(entity, other.values, 0.3, age_years)


## ── 3.2 초기화: 유전 + 문화 + HEXACO + 노이즈 합성 ──────
## parent_a_values, parent_b_values: 부모 values dict (없으면 null)
## culture_values: 정착지 shared_values (없으면 null)
## hexaco: 본인 HEXACO facet dictionary
static func initialize_values(
	hexaco: Dictionary,
	parent_a_values = null,
	parent_b_values = null,
	culture_values = null,
	rng: RandomNumberGenerator = null,
) -> Dictionary:
	if rng == null:
		rng = RandomNumberGenerator.new()
		rng.randomize()

	var hexaco_seed = compute_hexaco_seed(hexaco)
	var result: Dictionary = {}

	for vkey in ValueDefs.KEYS:
		var genetic: float = 0.0
		if parent_a_values != null and parent_b_values != null:
			genetic = (parent_a_values.get(vkey, 0.0) + parent_b_values.get(vkey, 0.0)) * 0.5 \
				+ rng.randf_range(-0.10, 0.10)
		elif parent_a_values != null:
			genetic = parent_a_values.get(vkey, 0.0) + rng.randf_range(-0.15, 0.15)
		else:
			genetic = hexaco_seed.get(vkey, 0.0)

		var culture: float = 0.0
		if culture_values != null:
			culture = culture_values.get(vkey, 0.0)

		var noise: float = rng.randf_range(-0.30, 0.30)
		var final_val: float = (
			genetic * GENETIC_WEIGHT
			+ culture * CULTURE_WEIGHT
			+ hexaco_seed.get(vkey, 0.0) * HEXACO_WEIGHT
			+ noise * NOISE_WEIGHT
		)
		result[vkey] = clampf(final_val, -1.0, 1.0)

	return result


## ── 3.3 연령별 Plasticity [Erikson 1950] ─────────────────
## 0~7세: 최대, 25세 이후: 거의 고정
static func get_plasticity(age_years: float) -> float:
	if age_years < 7.0:
		return 1.0
	elif age_years < 15.0:
		return lerpf(1.0, 0.70, (age_years - 7.0) / 8.0)
	elif age_years < 25.0:
		return lerpf(0.70, 0.30, (age_years - 15.0) / 10.0)
	elif age_years < 50.0:
		return lerpf(0.30, 0.10, (age_years - 25.0) / 25.0)
	else:
		return 0.10


## ── 3.4 문화 전파 [Axelrod 1997] ─────────────────────────
## 다른 에이전트와 상호작용 시 가치관이 조금씩 수렴
## entity_data: 업데이트할 엔티티 (values dict 포함)
## other_values: 상대방 values dict
## relationship_closeness: 친밀도 0~1
static func apply_peer_influence(
	entity_data,
	other_values: Dictionary,
	relationship_closeness: float,
	age_years: float,
) -> void:
	var plasticity: float = get_plasticity(age_years)
	for vkey in ValueDefs.KEYS:
		var my_val: float = entity_data.values.get(vkey, 0.0)
		var other_val: float = other_values.get(vkey, 0.0)
		var delta: float = (other_val - my_val) * relationship_closeness * plasticity \
			* CULTURE_DRIFT_PER_INTERACTION
		if absf(delta) > 0.001:
			entity_data.values[vkey] = clampf(my_val + delta, -1.0, 1.0)


## ── 3.5 경험 이벤트 → 가치관 변화 ───────────────────────
## event: value_events.json의 한 항목
## { "affected_values": { "LOYALTY": 0.15, ... }, "intensity": 0.8 }
static func apply_experience_event(
	entity_data,
	event: Dictionary,
	age_years: float,
) -> void:
	var plasticity: float = get_plasticity(age_years)
	var intensity: float = event.get("intensity", 0.5)
	var affected = event.get("affected_values", {})

	for vkey in affected:
		var base_delta: float = affected[vkey]
		var delta: float = clampf(base_delta, -EVENT_MAX_DELTA, EVENT_MAX_DELTA) \
			* intensity * plasticity
		var old_val: float = entity_data.values.get(vkey, 0.0)
		entity_data.values[vkey] = clampf(old_val + delta, -1.0, 1.0)


## ── 3.6 [Festinger 1957] 자기합리화 ──────────────────────
## 위반 횟수 임계치 초과 시 해당 가치관 소폭 하락
static func apply_rationalization(entity_data, vkey) -> void:
	var count: int = entity_data.value_violation_count.get(vkey, 0) + 1
	entity_data.value_violation_count[vkey] = count

	if count >= VIOLATION_RATIONALIZE_COUNT:
		var current: float = entity_data.values.get(vkey, 0.0)
		var direction: float = -1.0 if current > 0.0 else 1.0
		entity_data.values[vkey] = clampf(
			current + direction * RATIONALIZATION_DELTA,
			-1.0, 1.0
		)
		entity_data.value_violation_count[vkey] = 0


## ── 3.7 가치관 충돌 해소 ─────────────────────────────────
## 두 가치관 충돌 시 승자를 결정
## 반환: { "winner": StringName, "loser": StringName, "margin": float }
static func resolve_conflict(
	entity_data,
	val_a,
	val_b,
	emotions: Dictionary = {},
	rng: RandomNumberGenerator = null,
) -> Dictionary:
	if rng == null:
		rng = RandomNumberGenerator.new()
		rng.randomize()

	var score_a: float = _conflict_score(entity_data, val_a, emotions, rng)
	var score_b: float = _conflict_score(entity_data, val_b, emotions, rng)

	var winner = val_a if score_a >= score_b else val_b
	var loser = val_b if score_a >= score_b else val_a
	return {
		"winner": winner,
		"loser": loser,
		"margin": absf(score_a - score_b),
		"winner_score": maxf(score_a, score_b),
	}


static func _conflict_score(entity_data, vkey, emotions: Dictionary, rng: RandomNumberGenerator) -> float:
	var base: float = absf(entity_data.values.get(vkey, 0.0))
	var stage: int = entity_data.moral_stage
	var kohlberg_mod: float = ValueDefs.KOHLBERG_MODIFIERS.get(stage, {}).get(vkey, 1.0)
	var emotion_mod: float = 1.0 + _emotion_modifier(vkey, emotions)
	var noise: float = rng.randf_range(0.95, 1.05)
	return base * kohlberg_mod * emotion_mod * noise


## 감정 → 가치관 결정력 보정 (내부 헬퍼)
static func _emotion_modifier(vkey, emotions: Dictionary) -> float:
	var fear: float = emotions.get("fear", 0.0) / 100.0
	var anger: float = emotions.get("anger", 0.0) / 100.0
	var joy: float = emotions.get("joy", 0.0) / 100.0
	var sadness: float = emotions.get("sadness", 0.0) / 100.0
	var disgust: float = emotions.get("disgust", 0.0) / 100.0
	var trust: float = emotions.get("trust", 0.0) / 100.0
	match vkey:
		&"PEACE", &"TRANQUILITY":
			return fear * 0.30
		&"MARTIAL_PROWESS", &"COMPETITION":
			return anger * 0.30
		&"HARMONY", &"MERRIMENT":
			return joy * 0.20
		&"FAMILY", &"LOYALTY":
			return sadness * 0.20 + trust * 0.15
		&"TRUTH", &"FAIRNESS":
			return disgust * 0.25
		&"LAW":
			return fear * 0.15 + trust * 0.10
		&"CUNNING":
			return fear * 0.20 - trust * 0.15
		&"SACRIFICE":
			return sadness * 0.15 + trust * 0.20
	return 0.0


## PersonalityData 또는 Dictionary에서 HEXACO facet dict 추출
static func _get_hexaco_dict(personality) -> Dictionary:
	if personality == null:
		return {}
	if personality is Dictionary:
		return personality
	if "facets" in personality:
		return personality.facets
	# fallback: 6축만
	var d: Dictionary = {}
	for axis in ["h", "e", "x", "a", "c", "o"]:
		if axis in personality:
			d[axis] = personality.get(axis)
	return d


## ── 3.8 Kohlberg 도덕 발달 단계 진급 ────────────────────
## 조건 충족 시 moral_stage를 1 올림. 반환: true if 진급 발생.
static func check_moral_stage_progression(
	entity_data,
	hexaco: Dictionary,
	age_years: float,
) -> bool:
	var current: int = entity_data.moral_stage
	var next: int = current + 1
	if next > 6:
		return false
	if ValueDefs.KOHLBERG_AGE_REQ.size() <= next:
		return false
	if age_years < float(ValueDefs.KOHLBERG_AGE_REQ[next]):
		return false
	if not ValueDefs.KOHLBERG_THRESHOLDS.has(next):
		return false

	var req = ValueDefs.KOHLBERG_THRESHOLDS[next]

	var openness: float = (
		hexaco.get("aesthetic_appreciation", 0.5)
		+ hexaco.get("inquisitiveness", 0.5)
		+ hexaco.get("creativity", 0.5)
		+ hexaco.get("unconventionality", 0.5)
	) / 4.0
	if openness < req.get("min_openness", 0.0):
		return false

	for vkey in req.get("min_values", {}):
		if entity_data.values.get(vkey, 0.0) < req["min_values"][vkey]:
			return false

	entity_data.moral_stage = next
	return true
