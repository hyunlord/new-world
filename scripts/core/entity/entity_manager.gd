extends RefCounted

const EntityDataScript = preload("res://scripts/core/entity/entity_data.gd")
const ChunkIndex = preload("res://scripts/core/world/chunk_index.gd")
const GameCalendarScript = preload("res://scripts/core/simulation/game_calendar.gd")
const PersonalityDataScript = preload("res://scripts/core/entity/personality_data.gd")
const PersonalityGeneratorScript = preload("res://scripts/systems/biology/personality_generator.gd")
const TraitSystem = preload("res://scripts/systems/psychology/trait_system.gd")
const ValueSystem = preload("res://scripts/systems/social/value_system.gd")
const BodyAttributes = preload("res://scripts/core/entity/body_attributes.gd")
const IntelligenceGeneratorScript = preload("res://scripts/systems/cognition/intelligence_generator.gd")

var _entities: Dictionary = {}  # id -> entity
var _next_id: int = 1
var _world_data: RefCounted
var _rng: RandomNumberGenerator
var _settlement_manager: RefCounted
var _personality_generator: RefCounted
var _intelligence_generator: RefCounted
var chunk_index: RefCounted  # ChunkIndex for O(1) spatial lookups
var total_deaths: int = 0
var total_births: int = 0


## Initialize with world data and RNG reference
func init(world_data: RefCounted, rng: RandomNumberGenerator) -> void:
	_world_data = world_data
	_rng = rng
	chunk_index = ChunkIndex.new()
	_personality_generator = PersonalityGeneratorScript.new()
	_personality_generator.init(rng)
	_intelligence_generator = IntelligenceGeneratorScript.new()
	_intelligence_generator.init(rng)


## Spawn a new entity at the given position
## initial_age: age in ticks (0 = newborn child, use AGE_TEEN_END+ for adults)
func spawn_entity(pos: Vector2i, gender_override: String = "", initial_age: int = 0, parent_a: RefCounted = null, parent_b: RefCounted = null) -> RefCounted:
	var entity = EntityDataScript.new()
	entity.id = _next_id
	_next_id += 1
	entity.position = pos
	entity.hunger = 0.7 + _rng.randf() * 0.3
	entity.energy = 0.7 + _rng.randf() * 0.3
	entity.social = 0.5 + _rng.randf() * 0.5
	# Gender (50:50 or override) — must be set before name generation
	if gender_override != "":
		entity.gender = gender_override
	else:
		entity.gender = "female" if _rng.randf() < 0.5 else "male"
	entity.entity_name = NameGenerator.generate_name(entity.gender)
	# Personality (HEXACO 24-facet, Cholesky-correlated with parental inheritance)
	var pa_pd = parent_a.personality if parent_a != null else null
	var pb_pd = parent_b.personality if parent_b != null else null
	entity.personality = _personality_generator.generate_personality(entity.gender, "", pa_pd, pb_pd)
	# Initialize trait salience from personality facets
	TraitSystem.update_trait_strengths(entity)
	# ── 지능 초기화 [Gardner 1983 + CHC hybrid] ──────────────────────
	var intel_result: Dictionary = _intelligence_generator.generate(
		entity.gender,
		entity.personality.facets,
		parent_a,
		parent_b,
	)
	entity.general_intelligence = intel_result["g"]
	entity.intelligence_potentials = intel_result["potentials"]
	entity.intelligences = intel_result["effective"]
	# ── 가치관 초기화 [Schwartz 1992, ValueSystem] ──────────────────
	var pa_values = parent_a.values if (parent_a != null and not parent_a.values.is_empty()) else null
	var pb_values = parent_b.values if (parent_b != null and not parent_b.values.is_empty()) else null
	entity.values = ValueSystem.initialize_values(
		entity.personality.facets,
		pa_values,
		pb_values,
		null,
		_rng,
	)
	entity.moral_stage = 1
	# Bootstrap moral stage for pre-existing adults (initial_age > 0)
	if initial_age > 0:
		var _age_years: float = GameConfig.get_age_years(initial_age)
		var _hexaco: Dictionary = ValueSystem._get_hexaco_dict(entity.personality)
		for _s in range(6):
			if not ValueSystem.check_moral_stage_progression(entity, _hexaco, _age_years):
				break
	# Emotions (defaults)
	entity.emotions = {
		"happiness": 0.5,
		"loneliness": 0.0,
		"stress": 0.0,
		"grief": 0.0,
		"love": 0.0,
	}
	# Age and age stage
	entity.age = initial_age
	entity.age_stage = GameConfig.get_age_stage(entity.age)
	# Set birth_tick: negative for pre-existing entities (born before game start)
	if initial_age > 0:
		entity.birth_tick = -initial_age
	entity.birth_date = GameCalendarScript.birth_date_from_tick(entity.birth_tick, _rng)
	# Frailty: N(1.0, 0.15), clamped [0.5, 2.0] (Vaupel frailty model)
	entity.frailty = clampf(_rng.randfn(1.0, 0.15), 0.5, 2.0)
	# [Layer 1.5] Body Attributes — potential/trainability/realized 3-레이어
	var _body_age_y: float = GameConfig.get_age_years(entity.age)
	var _is_male: bool = entity.gender == "male"
	entity.body = BodyAttributes.new()
	# ── potential 생성 (6축, 성별 delta 적용, 부모 상속) ──
	for _b_axis in ["str", "agi", "end", "tou", "rec", "dr"]:
		var _b_sex_d: int = GameConfig.BODY_SEX_DELTA_MALE.get(_b_axis, 0) * (1 if _is_male else -1)
		var _b_raw: int
		if parent_a != null and parent_b != null \
				and parent_a.body != null and parent_b.body != null \
				and not parent_a.body.potential.is_empty() \
				and not parent_b.body.potential.is_empty():
			## ── Parental inheritance path (Bouchard & McGue 2003) ────────────────
			var _h2: float = GameConfig.BODY_HERITABILITY.get(_b_axis, 0.72)
			var _pa_pot: int = parent_a.body.potential.get(_b_axis, GameConfig.BODY_POTENTIAL_MEAN)
			var _pb_pot: int = parent_b.body.potential.get(_b_axis, GameConfig.BODY_POTENTIAL_MEAN)
			var _mid_parent: float = (_pa_pot + _pb_pot) / 2.0
			var _env_raw: float = _rng.randfn(float(GameConfig.BODY_POTENTIAL_MEAN), float(GameConfig.BODY_POTENTIAL_SD))
			var _base: float = _mid_parent * _h2 + _env_raw * (1.0 - _h2)
			_base += _rng.randfn(0.0, float(GameConfig.BODY_POTENTIAL_SD) * 0.10)
			if _rng.randf() < GameConfig.BODY_MUTATION_RATE:
				_base += _rng.randfn(0.0, float(GameConfig.BODY_POTENTIAL_MEAN) * GameConfig.BODY_MUTATION_SD)
			_b_raw = int(_base) + _b_sex_d
		else:
			## ── First generation: population-level random (unchanged) ────────────
			_b_raw = int(_rng.randfn(float(GameConfig.BODY_POTENTIAL_MEAN + _b_sex_d), float(GameConfig.BODY_POTENTIAL_SD)))
		entity.body.potential[_b_axis] = clampi(_b_raw, GameConfig.BODY_POTENTIAL_MIN, GameConfig.BODY_POTENTIAL_MAX)
	# ── trainability 생성 (5축, ACTN3/COL 상관 포함, 부모 상속) ──
	var _t_h2: float = GameConfig.BODY_TRAINABILITY_HERITABILITY
	var _has_parents_t: bool = parent_a != null and parent_b != null \
			and parent_a.body != null and parent_b.body != null \
			and not parent_a.body.trainability.is_empty() \
			and not parent_b.body.trainability.is_empty()
	# str base — inherited or random
	var _str_t_base: int
	if _has_parents_t:
		var _pa_st: int = parent_a.body.trainability.get("str", GameConfig.TRAINABILITY_MEAN)
		var _pb_st: int = parent_b.body.trainability.get("str", GameConfig.TRAINABILITY_MEAN)
		var _mid_st: float = (_pa_st + _pb_st) / 2.0
		var _env_st: float = _rng.randfn(float(GameConfig.TRAINABILITY_MEAN), float(GameConfig.TRAINABILITY_SD))
		var _base_st: float = _mid_st * _t_h2 + _env_st * (1.0 - _t_h2)
		_base_st += _rng.randfn(0.0, float(GameConfig.TRAINABILITY_SD) * 0.10)
		if _rng.randf() < GameConfig.BODY_MUTATION_RATE:
			_base_st += _rng.randfn(0.0, float(GameConfig.TRAINABILITY_MEAN) * GameConfig.BODY_MUTATION_SD)
		_str_t_base = int(_base_st)
	else:
		_str_t_base = int(_rng.randfn(float(GameConfig.TRAINABILITY_MEAN), float(GameConfig.TRAINABILITY_SD)))
	# end base — inherited or random
	var _end_t_base: int
	if _has_parents_t:
		var _pa_et: int = parent_a.body.trainability.get("end", GameConfig.TRAINABILITY_MEAN)
		var _pb_et: int = parent_b.body.trainability.get("end", GameConfig.TRAINABILITY_MEAN)
		var _mid_et: float = (_pa_et + _pb_et) / 2.0
		var _env_et: float = _rng.randfn(float(GameConfig.TRAINABILITY_MEAN), float(GameConfig.TRAINABILITY_SD))
		var _base_et: float = _mid_et * _t_h2 + _env_et * (1.0 - _t_h2)
		_base_et += _rng.randfn(0.0, float(GameConfig.TRAINABILITY_SD) * 0.10)
		if _rng.randf() < GameConfig.BODY_MUTATION_RATE:
			_base_et += _rng.randfn(0.0, float(GameConfig.TRAINABILITY_MEAN) * GameConfig.BODY_MUTATION_SD)
		_end_t_base = int(_base_et)
	else:
		_end_t_base = int(_rng.randfn(float(GameConfig.TRAINABILITY_MEAN), float(GameConfig.TRAINABILITY_SD)))
	# agi — inherited or random
	var _agi_t_raw: int
	if _has_parents_t:
		var _pa_at: int = parent_a.body.trainability.get("agi", GameConfig.TRAINABILITY_MEAN)
		var _pb_at: int = parent_b.body.trainability.get("agi", GameConfig.TRAINABILITY_MEAN)
		var _mid_at: float = (_pa_at + _pb_at) / 2.0
		var _env_at: float = _rng.randfn(float(GameConfig.TRAINABILITY_MEAN), float(GameConfig.TRAINABILITY_SD))
		var _base_at: float = _mid_at * _t_h2 + _env_at * (1.0 - _t_h2)
		_base_at += _rng.randfn(0.0, float(GameConfig.TRAINABILITY_SD) * 0.10)
		if _rng.randf() < GameConfig.BODY_MUTATION_RATE:
			_base_at += _rng.randfn(0.0, float(GameConfig.TRAINABILITY_MEAN) * GameConfig.BODY_MUTATION_SD)
		_agi_t_raw = int(_base_at)
	else:
		_agi_t_raw = int(_rng.randfn(float(GameConfig.TRAINABILITY_MEAN), float(GameConfig.TRAINABILITY_SD)))
	var _agi_t: int = clampi(_agi_t_raw, GameConfig.TRAINABILITY_MIN, GameConfig.TRAINABILITY_MAX)
	# ACTN3 효과: STR↔END 역상관 (-1=XX지구력형, +1=RR파워형)
	var _actn3: float = _rng.randf_range(-1.0, 1.0)
	var _str_t: int = clampi(_str_t_base + int(_actn3 * 75.0), GameConfig.TRAINABILITY_MIN, GameConfig.TRAINABILITY_MAX)
	var _end_t: int = clampi(_end_t_base - int(_actn3 * 50.0), GameConfig.TRAINABILITY_MIN, GameConfig.TRAINABILITY_MAX)
	# TOU: 0.7 독립 + 0.3 STR 상관 (기계적 하중 경로 공유)
	var _tou_t_ind: int = int(_rng.randfn(float(GameConfig.TRAINABILITY_MEAN), float(GameConfig.TRAINABILITY_SD)))
	var _tou_t: int = clampi(int(0.7 * float(_tou_t_ind) + 0.3 * float(_str_t)), GameConfig.TRAINABILITY_MIN, GameConfig.TRAINABILITY_MAX)
	# REC: 0.6 독립 + 0.4 END 상관 (PPARGC1A 미토콘드리아 경로 공유)
	var _rec_t_ind: int = int(_rng.randfn(float(GameConfig.TRAINABILITY_MEAN), float(GameConfig.TRAINABILITY_SD)))
	var _rec_t: int = clampi(int(0.6 * float(_rec_t_ind) + 0.4 * float(_end_t)) + int(_actn3 * 20.0), GameConfig.TRAINABILITY_MIN, GameConfig.TRAINABILITY_MAX)
	entity.body.trainability = {"str": _str_t, "agi": _agi_t, "end": _end_t, "tou": _tou_t, "rec": _rec_t}
	# ── innate_immunity (부모 상속, Brodin 2015 h²=0.65) ──
	var _imm_base_inherit: int
	if parent_a != null and parent_b != null \
			and parent_a.body != null and parent_b.body != null:
		var _h2_dr: float = GameConfig.BODY_HERITABILITY.get("dr", 0.65)
		var _pa_imm: int = parent_a.body.innate_immunity
		var _pb_imm: int = parent_b.body.innate_immunity
		var _mid_imm: float = (_pa_imm + _pb_imm) / 2.0
		var _env_imm: float = _rng.randfn(float(GameConfig.INNATE_IMMUNITY_MEAN), float(GameConfig.INNATE_IMMUNITY_SD))
		var _base_imm: float = _mid_imm * _h2_dr + _env_imm * (1.0 - _h2_dr)
		_base_imm += _rng.randfn(0.0, float(GameConfig.INNATE_IMMUNITY_SD) * 0.10)
		if _rng.randf() < GameConfig.BODY_MUTATION_RATE:
			_base_imm += _rng.randfn(0.0, float(GameConfig.INNATE_IMMUNITY_MEAN) * GameConfig.BODY_MUTATION_SD)
		_imm_base_inherit = int(_base_imm)
	else:
		_imm_base_inherit = GameConfig.INNATE_IMMUNITY_MEAN
	if not _is_male:
		_imm_base_inherit += GameConfig.INNATE_IMMUNITY_SEX_DELTA_FEMALE
	entity.body.innate_immunity = clampi(int(_rng.randfn(float(_imm_base_inherit), float(GameConfig.INNATE_IMMUNITY_SD) * 0.5)), 0, 1000)
	# ── training_xp 초기화 ──
	entity.body.training_xp = {"str": 0.0, "agi": 0.0, "end": 0.0, "tou": 0.0, "rec": 0.0}
	# ── realized 초기화 (훈련 XP 0, potential × age_curve) ──
	var _realized_values: Dictionary = entity.body.calc_realized_values_batch(_body_age_y)
	for _r_axis in ["str", "agi", "end", "tou", "rec", "dr"]:
		entity.body.realized[_r_axis] = int(_realized_values.get(_r_axis, 0))

	## ── Layer 1.5: Appearance Generation [Eagly 1991, Stulp 2015] ───────────────
	if parent_a != null and parent_b != null:
		var _attr_mid: float = (parent_a.attractiveness + parent_b.attractiveness) / 2.0
		var _attr_env: float = _rng.randfn(GameConfig.APPEARANCE_ATTRACT_MEAN, GameConfig.APPEARANCE_ATTRACT_SD)
		entity.attractiveness = clampf(
			_attr_mid * GameConfig.APPEARANCE_ATTRACT_HERITABILITY
			+ _attr_env * (1.0 - GameConfig.APPEARANCE_ATTRACT_HERITABILITY)
			+ _rng.randfn(0.0, 0.05),
			0.05, 0.95
		)
	else:
		entity.attractiveness = clampf(_rng.randfn(GameConfig.APPEARANCE_ATTRACT_MEAN, GameConfig.APPEARANCE_ATTRACT_SD), 0.05, 0.95)

	var _height_sex_delta: float = GameConfig.APPEARANCE_HEIGHT_SEX_DELTA_MALE * (1.0 if entity.gender == "male" else -1.0)
	if parent_a != null and parent_b != null:
		var _ht_mid: float = (parent_a.height + parent_b.height) / 2.0
		var _ht_env: float = _rng.randfn(GameConfig.APPEARANCE_HEIGHT_MEAN + _height_sex_delta, GameConfig.APPEARANCE_HEIGHT_SD)
		entity.height = clampf(
			_ht_mid * GameConfig.APPEARANCE_HEIGHT_HERITABILITY
			+ _ht_env * (1.0 - GameConfig.APPEARANCE_HEIGHT_HERITABILITY)
			+ _rng.randfn(0.0, 0.04),
			GameConfig.APPEARANCE_HEIGHT_SD_CLAMP_LOW,
			GameConfig.APPEARANCE_HEIGHT_SD_CLAMP_HIGH
		)
	else:
		entity.height = clampf(_rng.randfn(GameConfig.APPEARANCE_HEIGHT_MEAN + _height_sex_delta, GameConfig.APPEARANCE_HEIGHT_SD), 0.05, 0.95)

	entity.hair_color = _weighted_random_string(GameConfig.HAIR_COLOR_WEIGHTS)
	entity.eye_color = _weighted_random_string(GameConfig.EYE_COLOR_WEIGHTS)

	entity.distinguishing_marks = []
	var _mark_pool: Array = GameConfig.DISTINGUISHING_MARK_IDS.duplicate()
	_mark_pool.shuffle()
	for _mark_id in _mark_pool:
		if entity.distinguishing_marks.size() >= 2:
			break
		if _rng.randf() < GameConfig.DISTINGUISHING_MARK_CHANCE:
			entity.distinguishing_marks.append(_mark_id)

	## ── Layer 7: Blood Type [ABO Genetics — Human Definition v3 §13] ──────────
	if parent_a != null and parent_b != null:
		var _g1: String = parent_a.blood_genotype
		var _g2: String = parent_b.blood_genotype
		## 알파벳순 정렬로 교배키 생성
		var _cross_key: String = (_g1 + "_" + _g2) if _g1 <= _g2 else (_g2 + "_" + _g1)
		var _cross: Dictionary = GameConfig.BLOOD_CROSS_TABLE.get(_cross_key, {"OO": 100})
		entity.blood_genotype = _weighted_random_string(_cross)
	else:
		## 초기 스폰: 표현형 먼저 결정 → 유전자형 결정
		var _phenotype: String = _weighted_random_string(GameConfig.BLOOD_TYPE_SPAWN_WEIGHTS)
		var _geno_dist: Dictionary = GameConfig.BLOOD_GENOTYPE_FROM_PHENOTYPE.get(_phenotype, {"OO": 100})
		entity.blood_genotype = _weighted_random_string(_geno_dist)
	entity.blood_type = GameConfig.BLOOD_GENOTYPE_TO_PHENOTYPE.get(entity.blood_genotype, "O")

	## ── Layer 7: Zodiac Sign ─────────────────────────────────────────────────────
	if entity.birth_date.size() > 0:
		entity.zodiac_sign = _get_zodiac_sign(
			entity.birth_date.get("month", 1),
			entity.birth_date.get("day", 1)
		)

	## ── Layer 7: Speech Style — Upgraded [Costa & McCrae 1992, Ekman 1992] ──────
	## HEXACO axis 6개 + Plutchik 3감정 → tone/verbosity/humor
	var _axes: Dictionary = entity.personality.axes  # H/E/X/A/C/O (0~1)
	var _H: float = _axes.get("H", 0.5)
	var _E: float = _axes.get("E", 0.5)
	var _X: float = _axes.get("X", 0.5)
	var _A: float = _axes.get("A", 0.5)
	var _C: float = _axes.get("C", 0.5)
	var _O: float = _axes.get("O", 0.5)
	## 감정은 spawn 시점에서는 기본값 0 (출생시 감정 없음)
	## → entity.emotion_data가 null이면 0으로 처리
	var _anger: float = 0.0
	var _joy:   float = 0.0
	var _fear:  float = 0.0
	if entity.emotion_data != null:
		_anger = entity.emotion_data.get_emotion("anger") / 100.0
		_joy   = entity.emotion_data.get_emotion("joy")   / 100.0
		_fear  = entity.emotion_data.get_emotion("fear")  / 100.0

	## tone 점수 계산 [Costa & McCrae 1992, Ashton & Lee 2007]
	var _agg:  float = clampf(0.30*(1.0-_A) + 0.20*(1.0-_H) + 0.15*_X + 0.10*(1.0-_C) + 0.20*_anger + 0.05*(1.0-_fear), 0.0, 1.0)
	var _gent: float = clampf(0.30*_A + 0.20*_H + 0.15*_E + 0.10*_C + 0.15*(1.0-_anger) + 0.10*_joy, 0.0, 1.0)
	var _form: float = clampf(0.35*_C + 0.25*_H + 0.15*(1.0-_X) + 0.10*(1.0-_O) + 0.10*_fear + 0.05*(1.0-_joy), 0.0, 1.0)
	var _cas:  float = clampf(0.35*_X + 0.20*_O + 0.15*_joy + 0.15*(1.0-_C) + 0.10*_A + 0.05*(1.0-_fear), 0.0, 1.0)
	var _sarc: float = clampf(0.30*_O + 0.25*(1.0-_A) + 0.15*_X + 0.15*_anger + 0.10*_joy + 0.05*(1.0-_H), 0.0, 1.0)

	## 결정 규칙 (임계값 + 우선순위)
	var _speech_tone: String = "casual"  # default
	if _agg >= 0.65 and _anger >= 0.55:
		_speech_tone = "aggressive"
	elif _sarc >= 0.62 and _O >= 0.55 and _anger >= 0.35:
		_speech_tone = "sarcastic"
	elif _form >= 0.62 and _C >= 0.65:
		_speech_tone = "formal"
	elif _gent >= 0.62 and _A >= 0.65 and _H >= 0.60 and _anger <= 0.35:
		_speech_tone = "gentle"
	else:
		## 조건 미달 시 최고 점수 tone
		var _tone_scores_v2: Dictionary = {"aggressive": _agg, "gentle": _gent, "formal": _form, "casual": _cas, "sarcastic": _sarc}
		var _best: float = -1.0
		for _t2 in _tone_scores_v2:
			if _tone_scores_v2[_t2] > _best:
				_best = _tone_scores_v2[_t2]
				_speech_tone = _t2
	entity.speech_tone = _speech_tone

	## verbosity [Funder & Colvin 1991]: X 주도, 두려움이 말수 억제
	var _ver: float = clampf(0.55*_X + 0.15*_O + 0.10*_joy + 0.05*_anger + 0.10*(1.0-_fear) + 0.05*(1.0-_C), 0.0, 1.0)
	if _ver >= 0.67 or _X >= 0.75:
		entity.speech_verbosity = "talkative"
	elif _ver <= 0.33 or _X <= 0.25 or (_fear >= 0.70 and _X <= 0.55):
		entity.speech_verbosity = "taciturn"
	else:
		entity.speech_verbosity = "normal"

	## humor [Greengross & Miller 2011]: Openness + Extraversion + joy → humor drive
	var _hum: float = clampf(0.35*_O + 0.20*_X + 0.15*_joy + 0.15*(1.0-_A) + 0.10*(1.0-_fear) + 0.05*(1.0-_C), 0.0, 1.0)
	if _hum < 0.40:
		entity.speech_humor = "none"
	else:
		var _slap: float = clampf(0.45*_X + 0.30*_joy + 0.15*(1.0-_C) + 0.10*(1.0-_fear), 0.0, 1.0)
		var _wit:  float = clampf(0.45*_O + 0.20*_X + 0.15*_joy + 0.10*_C + 0.10*_H - 0.20*_anger, 0.0, 1.0)
		var _dry:  float = clampf(0.35*_C + 0.25*(1.0-_X) + 0.20*_O + 0.10*(1.0-_joy) + 0.10*_anger, 0.0, 1.0)
		if _slap >= 0.60 and _X >= 0.65 and _joy >= 0.55:
			entity.speech_humor = "slapstick"
		elif _wit >= 0.60 and _O >= 0.60:
			entity.speech_humor = "witty"
		else:
			## 가드 미달 시 최고 점수 스타일 (동률: X 높으면 slapstick, O 높으면 witty)
			if _slap >= _wit and _slap >= _dry:
				entity.speech_humor = "slapstick"
			elif _wit >= _dry:
				entity.speech_humor = "witty"
			else:
				entity.speech_humor = "dry"

	## ── Layer 7: Preferences [Linden et al. 2010] ────────────────────────────────
	var _o_axis: float = _O  ## Openness axis (구 O_inquisitiveness + O_aesthetic 대체)
	if _o_axis > 0.65 and _rng.randf() < 0.4:
		entity.favorite_food = GameConfig.PREFERENCE_FOOD_OPTIONS[_rng.randi() % 2 + 1]
	else:
		entity.favorite_food = "food"

	entity.favorite_color = GameConfig.PREFERENCE_COLOR_OPTIONS[_rng.randi() % GameConfig.PREFERENCE_COLOR_OPTIONS.size()]

	var _c_axis: float = _C  ## Conscientiousness axis (구 C_diligence + C_organization 대체)
	var _season_weights: Array = [25, 25, 25, 25]
	if _X > 0.6:  _season_weights[1] += 10
	if _c_axis > 0.6: _season_weights[3] += 10
	if _o_axis > 0.6: _season_weights[2] += 10
	entity.favorite_season = GameConfig.PREFERENCE_SEASON_OPTIONS[_weighted_index(_season_weights)]

	entity.disliked_things = []
	var _dislike_pool: Array = GameConfig.PREFERENCE_DISLIKE_IDS.duplicate()
	_dislike_pool.shuffle()
	for _di in range(mini(2, _dislike_pool.size())):
		if _rng.randf() < 0.35:
			entity.disliked_things.append(_dislike_pool[_di])

	# ── entity 속도/근력 ──
	entity.speed = float(entity.body.realized.get("agi", 700)) * GameConfig.BODY_SPEED_SCALE + GameConfig.BODY_SPEED_BASE
	entity.strength = float(entity.body.realized.get("str", 700)) / 1000.0
	_entities[entity.id] = entity
	_world_data.register_entity(pos, entity.id)
	chunk_index.add_entity(entity.id, pos)
	SimulationBus.emit_event("entity_spawned", {
		"entity_id": entity.id,
		"entity_name": entity.entity_name,
		"position": pos,
		"tick": 0,
	})
	return entity


## Move an entity to a new position
func move_entity(entity: RefCounted, new_pos: Vector2i) -> void:
	var old_pos: Vector2i = entity.position
	_world_data.move_entity(old_pos, new_pos, entity.id)
	chunk_index.update_entity(entity.id, old_pos, new_pos)
	entity.position = new_pos


## Kill an entity
func kill_entity(entity_id: int, cause: String, tick: int = -1) -> void:
	if not _entities.has(entity_id):
		return
	var entity = _entities[entity_id]
	# Register in DeceasedRegistry BEFORE removing
	if Engine.has_singleton("DeceasedRegistry") or entity.get_meta("_skip_deceased", false) == false:
		var registry: Node = Engine.get_main_loop().root.get_node_or_null("DeceasedRegistry")
		if registry != null:
			registry.register_death(entity, cause, tick)
	# Settlement cleanup
	if _settlement_manager != null and entity.settlement_id > 0:
		_settlement_manager.remove_member(entity.settlement_id, entity.id)
	var age_years: float = float(entity.age) / float(GameConfig.TICKS_PER_YEAR)
	entity.is_alive = false
	total_deaths += 1
	_world_data.unregister_entity(entity.position, entity.id)
	chunk_index.remove_entity(entity.id, entity.position)
	SimulationBus.emit_event("entity_died", {
		"entity_id": entity.id,
		"entity_name": entity.entity_name,
		"cause": cause,
		"position": entity.position,
		"tick": tick,
	})
	# Lifecycle signal for ChronicleSystem
	SimulationBus.entity_died.emit(entity.id, entity.entity_name, cause, age_years, tick)


## Get entity by ID
func get_entity(id: int) -> RefCounted:
	return _entities.get(id, null)


## Get all alive entities
func get_alive_entities() -> Array:
	var result: Array = []
	var all_entities: Array = _entities.values()
	for i in range(all_entities.size()):
		var entity = all_entities[i]
		if entity.is_alive:
			result.append(entity)
	return result


## Get alive entity count
func get_alive_count() -> int:
	var count: int = 0
	var all_entities: Array = _entities.values()
	for i in range(all_entities.size()):
		var entity = all_entities[i]
		if entity.is_alive:
			count += 1
	return count


## Get entities within radius of position (chunk-based, O(chunks * chunk_size))
func get_entities_near(pos: Vector2i, radius: int) -> Array:
	var result: Array = []
	var ids: Array = chunk_index.get_nearby_entity_ids(pos, radius)
	for i in range(ids.size()):
		var entity = _entities.get(ids[i], null)
		if entity != null and entity.is_alive:
			var dx: int = absi(entity.position.x - pos.x)
			var dy: int = absi(entity.position.y - pos.y)
			if dx <= radius and dy <= radius:
				result.append(entity)
	return result


## Weighted random string selection from Dictionary{key: weight_int}
func _weighted_random_string(weights: Dictionary) -> String:
	var total: int = 0
	for k in weights:
		total += int(weights[k])
	var roll: int = _rng.randi() % total
	var cumulative: int = 0
	for k in weights:
		cumulative += int(weights[k])
		if roll < cumulative:
			return k
	return weights.keys()[0]


## 생일 month(1~12) + day(1~31) → zodiac_sign String
static func _get_zodiac_sign(month: int, day: int) -> String:
	## 황도 12궁 경계: [월, 해당월_경계일, 경계일_이상_별자리]
	var _boundaries: Array = [
		[1, 20, "aquarius"], [2, 19, "pisces"],      [3, 21, "aries"],
		[4, 20, "taurus"],   [5, 21, "gemini"],      [6, 21, "cancer"],
		[7, 23, "leo"],      [8, 23, "virgo"],        [9, 23, "libra"],
		[10, 23, "scorpio"], [11, 22, "sagittarius"], [12, 22, "capricorn"]
	]
	## 이전 달 말일 기준 별자리 (경계일 미만일 때)
	var _prev_signs: Array = [
		"capricorn", "aquarius", "pisces", "aries", "taurus", "gemini",
		"cancer", "leo", "virgo", "libra", "scorpio", "sagittarius"
	]
	for i in range(_boundaries.size()):
		var _b: Array = _boundaries[i]
		if month == _b[0]:
			return _b[2] if day >= _b[1] else _prev_signs[i]
	return "capricorn"  ## fallback (12월 말)


## Weighted random index from Array[int] weights
func _weighted_index(weights: Array) -> int:
	var total: int = 0
	for w in weights:
		total += int(w)
	var roll: int = _rng.randi() % total
	var cumulative: int = 0
	for i in range(weights.size()):
		cumulative += int(weights[i])
		if roll < cumulative:
			return i
	return 0


## Serialize all entities
func to_save_data() -> Array:
	var result: Array = []
	var all_entities: Array = _entities.values()
	for i in range(all_entities.size()):
		var entity = all_entities[i]
		result.append(entity.to_dict())
	return result


## Register a birth (called by FamilySystem)
func register_birth() -> void:
	total_births += 1


## Load entities from saved data
func load_save_data(data: Array, world_data: RefCounted) -> void:
	_entities.clear()
	_next_id = 1
	total_deaths = 0
	total_births = 0
	chunk_index.clear()
	for i in range(data.size()):
		var item = data[i]
		if item is Dictionary:
			var entity = EntityDataScript.from_dict(item)
			_entities[entity.id] = entity
			if entity.is_alive:
				world_data.register_entity(entity.position, entity.id)
				chunk_index.add_entity(entity.id, entity.position)
			if entity.id >= _next_id:
				_next_id = entity.id + 1
