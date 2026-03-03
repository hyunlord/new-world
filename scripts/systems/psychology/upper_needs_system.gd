extends "res://scripts/core/simulation/simulation_system.gd"

## [Deci & Ryan 1985, Maslow 1943, Bandura 1977]
## 상위 7개 욕구(자율성·유능감·자아실현·의미·인정·소속·친밀)의 감쇠와 충족을 처리.
##
## 감쇠: 매 실행 시 7개 욕구 모두 소량 감소.
## 충족: 직업·정착지·파트너·스킬 수준·직업-가치관 정합에 따라 회복.
## 레벨업 보너스: SimulationBus.skill_leveled_up 구독 → 유능감/자아실현 즉시 상승.

const _UPPER_NEEDS_SCALAR_COUNT: int = 29
const _UPPER_NEEDS_FLAG_COUNT: int = 3
const _UPPER_NEEDS_SKILL_LEVEL_COUNT: int = 5

var _entity_manager: RefCounted
var _upper_needs_scalar_inputs: PackedFloat32Array = PackedFloat32Array()
var _upper_needs_flag_inputs: PackedByteArray = PackedByteArray()
var _upper_needs_skill_levels: PackedInt32Array = PackedInt32Array()


## Initialize with entity manager and connect skill signal
func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	system_name = "upper_needs"
	priority = 12
	tick_interval = GameConfig.UPPER_NEEDS_TICK_INTERVAL
	## [Bandura 1977] 레벨업 이벤트 구독 — 즉각적인 자기효능감 스파이크
	SimulationBus.skill_leveled_up.connect(_on_skill_leveled_up)


func execute_tick(_tick: int) -> void:
	if _upper_needs_scalar_inputs.size() < _UPPER_NEEDS_SCALAR_COUNT:
		_upper_needs_scalar_inputs.resize(_UPPER_NEEDS_SCALAR_COUNT)
	if _upper_needs_flag_inputs.size() < _UPPER_NEEDS_FLAG_COUNT:
		_upper_needs_flag_inputs.resize(_UPPER_NEEDS_FLAG_COUNT)
	if _upper_needs_skill_levels.size() < _UPPER_NEEDS_SKILL_LEVEL_COUNT:
		_upper_needs_skill_levels.resize(_UPPER_NEEDS_SKILL_LEVEL_COUNT)
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity = alive[i]
		## 유아(infant/toddler)는 상위 욕구 미적용 — 발달 심리학상 ~4세 이전
		if entity.age_stage == "infant" or entity.age_stage == "toddler":
			continue
		if not _apply_upper_needs_rust_step(entity):
			_apply_decay(entity)
			_apply_fulfillment(entity)
			_clamp_upper_needs(entity)


## Apply one upper-needs step via Rust bridge.
## Returns true when Rust path was applied, false when fallback is required.
func _apply_upper_needs_rust_step(entity: RefCounted) -> bool:
	var has_job: bool = entity.job != "none"
	var has_settlement: bool = entity.settlement_id > 0
	var has_partner: bool = entity.partner_id > -1

	_upper_needs_skill_levels[0] = int(entity.skill_levels.get(&"SKILL_FORAGING", 0))
	_upper_needs_skill_levels[1] = int(entity.skill_levels.get(&"SKILL_WOODCUTTING", 0))
	_upper_needs_skill_levels[2] = int(entity.skill_levels.get(&"SKILL_MINING", 0))
	_upper_needs_skill_levels[3] = int(entity.skill_levels.get(&"SKILL_CONSTRUCTION", 0))
	_upper_needs_skill_levels[4] = int(entity.skill_levels.get(&"SKILL_HUNTING", 0))

	var best_skill_norm: float = 0.0
	var best_skill_variant: Variant = SimBridge.body_upper_needs_best_skill_normalized(
		_upper_needs_skill_levels,
		100
	)
	if best_skill_variant != null:
		best_skill_norm = float(best_skill_variant)
	else:
		best_skill_norm = _get_best_skill_normalized(entity)

	var alignment: float = 0.0
	if has_job:
		var values: Dictionary = entity.values
		var job_code: int = _get_job_code(entity.job)
		var alignment_variant: Variant = SimBridge.body_upper_needs_job_alignment(
			job_code,
			float(values.get(&"CRAFTSMANSHIP", 0.0)),
			float(values.get(&"SKILL", 0.0)),
			float(values.get(&"HARD_WORK", 0.0)),
			float(values.get(&"NATURE", 0.0)),
			float(values.get(&"INDEPENDENCE", 0.0))
		)
		if alignment_variant != null:
			alignment = float(alignment_variant)
		else:
			alignment = _get_job_value_alignment(entity)

	_upper_needs_scalar_inputs[0] = entity.competence
	_upper_needs_scalar_inputs[1] = entity.autonomy
	_upper_needs_scalar_inputs[2] = entity.self_actualization
	_upper_needs_scalar_inputs[3] = entity.meaning
	_upper_needs_scalar_inputs[4] = entity.transcendence
	_upper_needs_scalar_inputs[5] = entity.recognition
	_upper_needs_scalar_inputs[6] = entity.belonging
	_upper_needs_scalar_inputs[7] = entity.intimacy

	_upper_needs_scalar_inputs[8] = GameConfig.UPPER_NEEDS_COMPETENCE_DECAY
	_upper_needs_scalar_inputs[9] = GameConfig.UPPER_NEEDS_AUTONOMY_DECAY
	_upper_needs_scalar_inputs[10] = GameConfig.UPPER_NEEDS_SELF_ACTUATION_DECAY
	_upper_needs_scalar_inputs[11] = GameConfig.UPPER_NEEDS_MEANING_DECAY
	_upper_needs_scalar_inputs[12] = GameConfig.UPPER_NEEDS_TRANSCENDENCE_DECAY
	_upper_needs_scalar_inputs[13] = GameConfig.UPPER_NEEDS_RECOGNITION_DECAY
	_upper_needs_scalar_inputs[14] = GameConfig.UPPER_NEEDS_BELONGING_DECAY
	_upper_needs_scalar_inputs[15] = GameConfig.UPPER_NEEDS_INTIMACY_DECAY

	_upper_needs_scalar_inputs[16] = GameConfig.UPPER_NEEDS_COMPETENCE_JOB_GAIN
	_upper_needs_scalar_inputs[17] = GameConfig.UPPER_NEEDS_AUTONOMY_JOB_GAIN
	_upper_needs_scalar_inputs[18] = GameConfig.UPPER_NEEDS_BELONGING_SETTLEMENT_GAIN
	_upper_needs_scalar_inputs[19] = GameConfig.UPPER_NEEDS_INTIMACY_PARTNER_GAIN
	_upper_needs_scalar_inputs[20] = GameConfig.UPPER_NEEDS_RECOGNITION_SKILL_COEFF
	_upper_needs_scalar_inputs[21] = GameConfig.UPPER_NEEDS_SELF_ACTUATION_SKILL_COEFF
	_upper_needs_scalar_inputs[22] = GameConfig.UPPER_NEEDS_MEANING_BASE_GAIN
	_upper_needs_scalar_inputs[23] = GameConfig.UPPER_NEEDS_MEANING_ALIGNED_GAIN
	_upper_needs_scalar_inputs[24] = GameConfig.UPPER_NEEDS_TRANSCENDENCE_SETTLEMENT_GAIN
	_upper_needs_scalar_inputs[25] = GameConfig.UPPER_NEEDS_TRANSCENDENCE_SACRIFICE_COEFF
	_upper_needs_scalar_inputs[26] = best_skill_norm
	_upper_needs_scalar_inputs[27] = alignment
	_upper_needs_scalar_inputs[28] = float(entity.values.get(&"SACRIFICE", 0.0))

	_upper_needs_flag_inputs[0] = 1 if has_job else 0
	_upper_needs_flag_inputs[1] = 1 if has_settlement else 0
	_upper_needs_flag_inputs[2] = 1 if has_partner else 0

	var step_variant: Variant = SimBridge.body_upper_needs_step_packed(
		_upper_needs_scalar_inputs,
		_upper_needs_flag_inputs
	)
	if step_variant is PackedFloat32Array:
		var step_values: PackedFloat32Array = step_variant
		if step_values.size() >= 8:
			entity.competence = float(step_values[0])
			entity.autonomy = float(step_values[1])
			entity.self_actualization = float(step_values[2])
			entity.meaning = float(step_values[3])
			entity.transcendence = float(step_values[4])
			entity.recognition = float(step_values[5])
			entity.belonging = float(step_values[6])
			entity.intimacy = float(step_values[7])
			return true
	return false


## Maps job string to a compact job-code used by native alignment math.
func _get_job_code(job: String) -> int:
	match job:
		"builder", "miner":
			return 1
		"gatherer", "lumberjack":
			return 2
	return 0


## Apply time-based decay to all 7 upper needs.
func _apply_decay(entity: RefCounted) -> void:
	entity.competence         -= GameConfig.UPPER_NEEDS_COMPETENCE_DECAY
	entity.autonomy           -= GameConfig.UPPER_NEEDS_AUTONOMY_DECAY
	entity.self_actualization -= GameConfig.UPPER_NEEDS_SELF_ACTUATION_DECAY
	entity.meaning            -= GameConfig.UPPER_NEEDS_MEANING_DECAY
	entity.transcendence      -= GameConfig.UPPER_NEEDS_TRANSCENDENCE_DECAY
	entity.recognition        -= GameConfig.UPPER_NEEDS_RECOGNITION_DECAY
	entity.belonging          -= GameConfig.UPPER_NEEDS_BELONGING_DECAY
	entity.intimacy           -= GameConfig.UPPER_NEEDS_INTIMACY_DECAY


## Apply situation-based fulfillment gains.
func _apply_fulfillment(entity: RefCounted) -> void:
	## [SDT — Deci & Ryan 1985] 직업 보유 → 유능감·자율성 회복
	if entity.job != "none":
		entity.competence += GameConfig.UPPER_NEEDS_COMPETENCE_JOB_GAIN
		entity.autonomy   += GameConfig.UPPER_NEEDS_AUTONOMY_JOB_GAIN

	## [Maslow 1943 L3] 정착지 소속 → 소속감 회복
	if entity.settlement_id > 0:
		entity.belonging += GameConfig.UPPER_NEEDS_BELONGING_SETTLEMENT_GAIN

	## [Bowlby 1969] 파트너 있음 → 친밀감 회복
	if entity.partner_id > -1:
		entity.intimacy += GameConfig.UPPER_NEEDS_INTIMACY_PARTNER_GAIN

	## [Bandura 1977] 스킬 숙련도 → 사회적 인정·자아실현 회복
	var best_skill_norm: float = _get_best_skill_normalized(entity)
	entity.recognition        += GameConfig.UPPER_NEEDS_RECOGNITION_SKILL_COEFF   * best_skill_norm
	entity.self_actualization += GameConfig.UPPER_NEEDS_SELF_ACTUATION_SKILL_COEFF * best_skill_norm

	## [Csikszentmihalyi 1990 Flow] 의미감 — 기본 소량 + 직업-가치관 정합 보너스
	entity.meaning += GameConfig.UPPER_NEEDS_MEANING_BASE_GAIN
	if entity.job != "none":
		var alignment: float = _get_job_value_alignment(entity)
		entity.meaning += GameConfig.UPPER_NEEDS_MEANING_ALIGNED_GAIN * alignment

	## [Koltko-Rivera 2006] Transcendence — community + sacrifice-value alignment
	if entity.settlement_id > 0:
		entity.transcendence += GameConfig.UPPER_NEEDS_TRANSCENDENCE_SETTLEMENT_GAIN
	var sacrifice_norm: float = clampf((entity.values.get(&"SACRIFICE", 0.0) + 1.0) / 2.0, 0.0, 1.0)
	entity.transcendence += GameConfig.UPPER_NEEDS_TRANSCENDENCE_SACRIFICE_COEFF * sacrifice_norm


## Return the highest skill level among 5 core skills, normalized to 0.0–1.0.
func _get_best_skill_normalized(entity: RefCounted) -> float:
	var best: int = 0
	for skill_id in [&"SKILL_FORAGING", &"SKILL_WOODCUTTING", &"SKILL_MINING",
			&"SKILL_CONSTRUCTION", &"SKILL_HUNTING"]:
		var lvl: int = int(entity.skill_levels.get(skill_id, 0))
		if lvl > best:
			best = lvl
	return float(best) / 100.0


## Return job-value alignment score 0.0–1.0.
## [SDT — Deci & Ryan 1985, Csikszentmihalyi 1990]
func _get_job_value_alignment(entity: RefCounted) -> float:
	var vals: Dictionary = entity.values
	var alignment: float = 0.0
	match entity.job:
		"builder", "miner":
			alignment += maxf(0.0, float(vals.get(&"CRAFTSMANSHIP", 0.0))) * 0.5
			alignment += maxf(0.0, float(vals.get(&"SKILL",         0.0))) * 0.3
			alignment += maxf(0.0, float(vals.get(&"HARD_WORK",     0.0))) * 0.2
		"gatherer", "lumberjack":
			alignment += maxf(0.0, float(vals.get(&"NATURE",       0.0))) * 0.5
			alignment += maxf(0.0, float(vals.get(&"INDEPENDENCE", 0.0))) * 0.3
			alignment += maxf(0.0, float(vals.get(&"HARD_WORK",    0.0))) * 0.2
	return clampf(alignment, 0.0, 1.0)


## Clamp all 7 upper need fields to [0.0, 1.0].
func _clamp_upper_needs(entity: RefCounted) -> void:
	entity.competence         = clampf(entity.competence,         0.0, 1.0)
	entity.autonomy           = clampf(entity.autonomy,           0.0, 1.0)
	entity.self_actualization = clampf(entity.self_actualization, 0.0, 1.0)
	entity.meaning            = clampf(entity.meaning,            0.0, 1.0)
	entity.transcendence      = clampf(entity.transcendence,      0.0, 1.0)
	entity.recognition        = clampf(entity.recognition,        0.0, 1.0)
	entity.belonging          = clampf(entity.belonging,          0.0, 1.0)
	entity.intimacy           = clampf(entity.intimacy,           0.0, 1.0)


## [Bandura 1977] Skill level-up event handler.
## Mastery achievement creates an immediate self-efficacy spike.
func _on_skill_leveled_up(entity_id: int, _entity_name: String, _skill_id: StringName,
		old_level: int, new_level: int, _tick: int) -> void:
	if new_level <= old_level:
		return
	var entity: RefCounted = _entity_manager.get_entity(entity_id)
	if entity == null:
		return
	## Check is_alive field — entity may have died before signal processed
	if entity.get("is_alive") == false:
		return
	entity.competence         = minf(1.0, entity.competence + GameConfig.UPPER_NEEDS_SKILLUP_COMPETENCE_BONUS)
	entity.self_actualization = minf(1.0, entity.self_actualization + GameConfig.UPPER_NEEDS_SKILLUP_SELF_ACT_BONUS)
