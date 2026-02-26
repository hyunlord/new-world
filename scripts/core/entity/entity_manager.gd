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
	for _r_axis in ["str", "agi", "end", "tou", "rec"]:
		var _r_curve: float = BodyAttributes.compute_age_curve(_r_axis, _body_age_y)
		entity.body.realized[_r_axis] = clampi(int(float(entity.body.potential[_r_axis]) * _r_curve), 0, 15000)
	entity.body.realized["dr"] = clampi(int(float(entity.body.potential["dr"]) * BodyAttributes.compute_age_curve("dr", _body_age_y)), 0, 10000)

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

	## ── Layer 7: Speech Style [Human Definition v3 §13] ─────────────────────────
	var _facets: Dictionary = entity.personality.facets
	var _a_forgive: float = _facets.get("A_forgiveness", 0.5)
	var _a_gentle_f: float = _facets.get("A_gentleness",  0.5)
	var _h_sincere: float  = _facets.get("H_sincerity",   0.5)
	var _c_organ: float    = _facets.get("C_organization",0.5)
	var _o_inq: float      = _facets.get("O_inquisitiveness", 0.5)
	var _e_senti: float    = _facets.get("E_sentimentality",  0.5)
	var _x_bold: float     = _facets.get("X_social_boldness",  0.5)
	var _x_socio: float    = _facets.get("X_sociability",      0.5)

	var _tone_scores: Dictionary = {
		"aggressive": (1.0 - _a_forgive) * 0.5 + (1.0 - _h_sincere) * 0.5,
		"gentle":     _a_gentle_f * 0.6 + _e_senti * 0.4,
		"formal":     _c_organ * 0.5 + (1.0 - _x_bold) * 0.5,
		"sarcastic":  _o_inq * 0.5 + (1.0 - _h_sincere) * 0.5,
		"casual":     _x_socio * 0.4 + 0.3,
	}
	var _best_tone: String = "casual"
	var _best_score: float = -1.0
	for _t in _tone_scores:
		if _tone_scores[_t] > _best_score:
			_best_score = _tone_scores[_t]
			_best_tone = _t
	entity.speech_tone = _best_tone

	var _x_avg: float = (_x_bold + _x_socio + _facets.get("X_liveliness", 0.5) + _facets.get("X_social_self_esteem", 0.5)) / 4.0
	if _x_avg < 0.35:
		entity.speech_verbosity = "taciturn"
	elif _x_avg > 0.65:
		entity.speech_verbosity = "talkative"
	else:
		entity.speech_verbosity = "normal"

	var _merriment: float = entity.values.get(&"MERRIMENT", 0.0)
	var _ling_intel: float = entity.intelligences.get("linguistic", 0.5)
	var _humor_score: float = (_merriment + 1.0) / 2.0
	if _humor_score > 0.55:
		entity.speech_humor = "witty" if _ling_intel > 0.55 else "slapstick"
	elif _humor_score < 0.35 and _ling_intel > 0.55:
		entity.speech_humor = "dry"
	else:
		entity.speech_humor = "none"

	## ── Layer 7: Preferences [Linden et al. 2010] ────────────────────────────────
	var _o_axis: float = (_facets.get("O_inquisitiveness", 0.5) + _facets.get("O_aesthetic", 0.5)) / 2.0
	if _o_axis > 0.65 and _rng.randf() < 0.4:
		entity.favorite_food = GameConfig.PREFERENCE_FOOD_OPTIONS[_rng.randi() % 2 + 1]
	else:
		entity.favorite_food = "food"

	entity.favorite_color = GameConfig.PREFERENCE_COLOR_OPTIONS[_rng.randi() % GameConfig.PREFERENCE_COLOR_OPTIONS.size()]

	var _c_axis: float = (_facets.get("C_diligence", 0.5) + _facets.get("C_organization", 0.5)) / 2.0
	var _season_weights: Array = [25, 25, 25, 25]
	if _x_avg > 0.6:  _season_weights[1] += 10
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
