extends "res://scripts/core/simulation/simulation_system.gd"
## StatSyncSystem: entity 필드 → stat_cache 동기화 브릿지.
## priority=1 — 매 tick 모든 시스템보다 먼저 실행.

const _SIM_BRIDGE_NODE_NAME: String = "SimBridge"
const _SIM_BRIDGE_DERIVED_METHOD: String = "body_stat_sync_derived_scores"

var _entity_manager: RefCounted
var _bridge_checked: bool = false
var _sim_bridge: Object = null

func _init() -> void:
	system_name = "stat_sync"
	priority = 1
	tick_interval = 10


## entity_manager를 받아 초기화
func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager


func _get_sim_bridge() -> Object:
	if _bridge_checked:
		return _sim_bridge
	_bridge_checked = true
	var tree: SceneTree = Engine.get_main_loop() as SceneTree
	if tree == null:
		return null
	var root: Node = tree.get_root()
	if root == null:
		return null
	var node: Node = root.get_node_or_null(_SIM_BRIDGE_NODE_NAME)
	if node != null and node.has_method(_SIM_BRIDGE_DERIVED_METHOD):
		_sim_bridge = node
	return _sim_bridge


## Syncs all alive entities' raw fields (needs, HEXACO, emotions, skills) into their stat_cache each tick.
func execute_tick(_tick: int) -> void:
	if _entity_manager == null:
		return
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		_sync_entity(alive[i])


func _sync_entity(entity: RefCounted) -> void:
	# Needs: float 0~1 → int 0~1000
	StatQuery.set_value(entity, &"NEED_HUNGER", int(entity.hunger * 1000), 0)
	StatQuery.set_value(entity, &"NEED_THIRST", int(entity.thirst * 1000), 0)
	StatQuery.set_value(entity, &"NEED_ENERGY", int(entity.energy * 1000), 0)
	StatQuery.set_value(entity, &"NEED_WARMTH", int(entity.warmth * 1000), 0)
	StatQuery.set_value(entity, &"NEED_SAFETY", int(entity.safety * 1000), 0)
	StatQuery.set_value(entity, &"NEED_SOCIAL", int(entity.social * 1000), 0)

	# 상위 욕구 (Maslow Relatedness + Growth / Alderfer ERG / Deci & Ryan SDT)
	StatQuery.set_value(entity, &"NEED_BELONGING",          int(entity.belonging * 1000), 0)
	StatQuery.set_value(entity, &"NEED_INTIMACY",           int(entity.intimacy * 1000), 0)
	StatQuery.set_value(entity, &"NEED_RECOGNITION",        int(entity.recognition * 1000), 0)
	StatQuery.set_value(entity, &"NEED_AUTONOMY",           int(entity.autonomy * 1000), 0)
	StatQuery.set_value(entity, &"NEED_COMPETENCE",         int(entity.competence * 1000), 0)
	StatQuery.set_value(entity, &"NEED_SELF_ACTUALIZATION", int(entity.self_actualization * 1000), 0)
	StatQuery.set_value(entity, &"NEED_MEANING",            int(entity.meaning * 1000), 0)
	StatQuery.set_value(entity, &"NEED_TRANSCENDENCE",    int(entity.transcendence * 1000), 0)

	## SKILL levels: int 0–100 — data-driven, syncs all skills the entity has trained
	for _sid in entity.skill_levels.keys():
		var _level: int = int(entity.skill_levels.get(_sid, 0))
		StatQuery.set_value(entity, _sid, _level, 0)

	# HEXACO axes: float 0~1 → int 0~1000
	var pd = entity.personality
	if pd == null:
		return
	StatQuery.set_value(entity, &"HEXACO_H", int(pd.axes.get("H", 0.5) * 1000), 0)
	StatQuery.set_value(entity, &"HEXACO_E", int(pd.axes.get("E", 0.5) * 1000), 0)
	StatQuery.set_value(entity, &"HEXACO_X", int(pd.axes.get("X", 0.5) * 1000), 0)
	StatQuery.set_value(entity, &"HEXACO_A", int(pd.axes.get("A", 0.5) * 1000), 0)
	StatQuery.set_value(entity, &"HEXACO_C", int(pd.axes.get("C", 0.5) * 1000), 0)
	StatQuery.set_value(entity, &"HEXACO_O", int(pd.axes.get("O", 0.5) * 1000), 0)

	# Emotion meta stats
	var ed = entity.emotion_data
	if ed == null:
		_compute_derived(entity)
		_sync_facets(entity)
		_sync_intelligences(entity)
		_sync_economic(entity)
		_sync_social_identity(entity)
		return
	StatQuery.set_value(entity, &"EMOTION_STRESS",     int(ed.stress * 20.0), 0)
	StatQuery.set_value(entity, &"EMOTION_ALLOSTATIC",  int(ed.allostatic), 0)
	StatQuery.set_value(entity, &"EMOTION_RESERVE",     int(ed.reserve), 0)
	_compute_derived(entity)
	_sync_facets(entity)
	_sync_intelligences(entity)
	_sync_economic(entity)
	_sync_social_identity(entity)


## COMPOSITE 파생 스탯 계산 및 stat_cache에 저장.
## HEXACO, Emotion, Body, Value 스탯을 조합하여 8개 파생 스탯을 만든다.
## StatSyncSystem priority=1이므로 다른 시스템보다 먼저 실행됨.
func _compute_derived(entity: RefCounted) -> void:
	var X: float = StatQuery.get_normalized(entity, &"HEXACO_X")
	var A: float = StatQuery.get_normalized(entity, &"HEXACO_A")
	var H: float = StatQuery.get_normalized(entity, &"HEXACO_H")
	var E: float = StatQuery.get_normalized(entity, &"HEXACO_E")
	var O: float = StatQuery.get_normalized(entity, &"HEXACO_O")
	var C: float = StatQuery.get_normalized(entity, &"HEXACO_C")
	var joy: float = StatQuery.get_normalized(entity, &"EMOTION_JOY")
	var anticipation: float = StatQuery.get_normalized(entity, &"EMOTION_ANTICIPATION")
	var anger: float = StatQuery.get_normalized(entity, &"EMOTION_ANGER")
	var str_pot: float = StatQuery.get_normalized(entity, &"BODY_STR_POTENTIAL")
	var romance: float = StatQuery.get_normalized(entity, &"VALUE_ROMANCE")
	var truth: float = StatQuery.get_normalized(entity, &"VALUE_TRUTH")
	var artwork: float = StatQuery.get_normalized(entity, &"VALUE_ARTWORK")
	var knowledge: float = StatQuery.get_normalized(entity, &"VALUE_KNOWLEDGE")
	var merriment: float = StatQuery.get_normalized(entity, &"VALUE_MERRIMENT")
	var friendship: float = StatQuery.get_normalized(entity, &"VALUE_FRIENDSHIP")
	var competition: float = StatQuery.get_normalized(entity, &"VALUE_COMPETITION")
	var recognition: float = StatQuery.get_normalized(entity, &"NEED_RECOGNITION")
	## Intelligence inputs [Visser 2006, CHC]
	var i_ling: float = StatQuery.get_normalized(entity, &"INTEL_LINGUISTIC")
	var i_log: float = StatQuery.get_normalized(entity, &"INTEL_LOGICAL")
	var i_spa: float = StatQuery.get_normalized(entity, &"INTEL_SPATIAL")
	var i_mus: float = StatQuery.get_normalized(entity, &"INTEL_MUSICAL")
	var i_kin: float = StatQuery.get_normalized(entity, &"INTEL_KINESTHETIC")
	var i_inter: float = StatQuery.get_normalized(entity, &"INTEL_INTERPERSONAL")
	var i_intra: float = StatQuery.get_normalized(entity, &"INTEL_INTRAPERSONAL")
	var i_nat: float = StatQuery.get_normalized(entity, &"INTEL_NATURALISTIC")
	## Appearance inputs [Eagly et al. 1991, Stulp et al. 2015]
	var attract: float = entity.attractiveness
	var ent_height: float = entity.height
	var age_years: float = GameConfig.get_age_years(entity.age)

	var charisma: float = 0.0
	var intimidation: float = 0.0
	var allure: float = 0.0
	var trustworthiness: float = 0.0
	var creativity: float = 0.0
	var wisdom: float = 0.0
	var popularity: float = 0.0
	var risk_tolerance: float = 0.0
	var used_rust: bool = false
	var bridge: Object = _get_sim_bridge()
	if bridge != null:
		var inputs: PackedFloat32Array = PackedFloat32Array([
			X, A, H, E, O, C, joy, anticipation, anger, str_pot, romance, truth, artwork,
			knowledge, merriment, friendship, competition, recognition, i_ling, i_log, i_spa,
			i_mus, i_kin, i_inter, i_intra, i_nat, attract, ent_height, age_years,
		])
		var rust_variant: Variant = bridge.call(_SIM_BRIDGE_DERIVED_METHOD, inputs)
		if rust_variant is PackedFloat32Array:
			var out: PackedFloat32Array = rust_variant
			if out.size() >= 8:
				charisma = float(out[0])
				intimidation = float(out[1])
				allure = float(out[2])
				trustworthiness = float(out[3])
				creativity = float(out[4])
				wisdom = float(out[5])
				popularity = float(out[6])
				risk_tolerance = float(out[7])
				used_rust = true

	if not used_rust:
		## Charisma [Interpersonal + Linguistic + personality]
		charisma = clampf(i_inter * 0.16 + i_ling * 0.10 + X * 0.22 + A * 0.16 + H * 0.08 + joy * 0.08 + (1.0 - E) * 0.06 + recognition * 0.04 + merriment * 0.05 + friendship * 0.05, 0.0, 1.0)
		## Intimidation [Human Definition v3 §14: Strength + Height + Anger + voice + experience]
		intimidation = clampf(str_pot * 0.28 + ent_height * 0.17 + anger * 0.22 + (1.0 - E) * 0.12 + X * 0.10 + competition * 0.06 + i_kin * 0.05, 0.0, 1.0)
		## Allure [Human Definition v3 §14: Attractiveness + Charisma + fashion + ROMANCE]
		allure = clampf(attract * 0.35 + charisma * 0.35 + romance * 0.20 + X * 0.10, 0.0, 1.0)
		trustworthiness = clampf(H * 0.40 + A * 0.30 + truth * 0.30, 0.0, 1.0)
		## Creativity [Spatial + Musical + Openness]
		creativity = clampf(i_spa * 0.15 + i_mus * 0.10 + i_ling * 0.05 + O * 0.25 + anticipation * 0.15 + artwork * 0.15 + i_intra * 0.05 + X * 0.10, 0.0, 1.0)
		## Wisdom [Intrapersonal + Logical + age]
		var age_factor: float = clampf((age_years - 25.0) / 35.0, 0.0, 1.0)
		wisdom = clampf(i_intra * 0.16 + i_log * 0.12 + C * 0.14 + O * 0.10 + A * 0.10 + knowledge * 0.14 + age_factor * 0.12 + i_ling * 0.06 + i_nat * 0.06, 0.0, 1.0)
		popularity = clampf(charisma * 0.50 + merriment * 0.25 + friendship * 0.25, 0.0, 1.0)
		risk_tolerance = clampf((1.0 - E) * 0.40 + O * 0.30 + competition * 0.15 + (1.0 - C) * 0.15, 0.0, 1.0)

	StatQuery.set_value(entity, &"DERIVED_CHARISMA", int(charisma * 1000), 0)
	StatQuery.set_value(entity, &"DERIVED_INTIMIDATION", int(intimidation * 1000), 0)
	StatQuery.set_value(entity, &"DERIVED_ALLURE", int(allure * 1000), 0)
	StatQuery.set_value(entity, &"DERIVED_TRUSTWORTHINESS", int(trustworthiness * 1000), 0)
	StatQuery.set_value(entity, &"DERIVED_CREATIVITY", int(creativity * 1000), 0)
	StatQuery.set_value(entity, &"DERIVED_WISDOM", int(wisdom * 1000), 0)
	StatQuery.set_value(entity, &"DERIVED_POPULARITY", int(popularity * 1000), 0)
	StatQuery.set_value(entity, &"DERIVED_RISK_TOLERANCE", int(risk_tolerance * 1000), 0)


func _sync_facets(entity: RefCounted) -> void:
	var pd = entity.personality
	if pd == null:
		return
	var facets: Dictionary = pd.facets
	if facets.is_empty():
		return
	var facet_map: Dictionary = {
		"H_sincerity":          &"FACET_H_SINCERITY",
		"H_fairness":           &"FACET_H_FAIRNESS",
		"H_greed_avoidance":    &"FACET_H_GREED_AVOIDANCE",
		"H_modesty":            &"FACET_H_MODESTY",
		"E_fearfulness":        &"FACET_E_FEARFULNESS",
		"E_anxiety":            &"FACET_E_ANXIETY",
		"E_dependence":         &"FACET_E_DEPENDENCE",
		"E_sentimentality":     &"FACET_E_SENTIMENTALITY",
		"X_social_self_esteem": &"FACET_X_SOCIAL_SELF_ESTEEM",
		"X_social_boldness":    &"FACET_X_SOCIAL_BOLDNESS",
		"X_sociability":        &"FACET_X_SOCIABILITY",
		"X_liveliness":         &"FACET_X_LIVELINESS",
		"A_forgiveness":        &"FACET_A_FORGIVENESS",
		"A_gentleness":         &"FACET_A_GENTLENESS",
		"A_flexibility":        &"FACET_A_FLEXIBILITY",
		"A_patience":           &"FACET_A_PATIENCE",
		"C_organization":       &"FACET_C_ORGANIZATION",
		"C_diligence":          &"FACET_C_DILIGENCE",
		"C_perfectionism":      &"FACET_C_PERFECTIONISM",
		"C_prudence":           &"FACET_C_PRUDENCE",
		"O_aesthetic":          &"FACET_O_AESTHETIC",
		"O_inquisitiveness":    &"FACET_O_INQUISITIVENESS",
		"O_creativity":         &"FACET_O_CREATIVITY",
		"O_unconventionality":  &"FACET_O_UNCONVENTIONALITY",
	}
	for fkey in facet_map:
		var stat_id: StringName = facet_map[fkey]
		var fval: float = float(facets.get(fkey, 0.5))
		StatQuery.set_value(entity, stat_id, int(fval * 1000), 0)


func _sync_intelligences(entity: RefCounted) -> void:
	var intel: Dictionary = entity.intelligences
	if intel == null or intel.is_empty():
		return
	var intel_map: Dictionary = {
		"linguistic":     &"INTEL_LINGUISTIC",
		"logical":        &"INTEL_LOGICAL",
		"spatial":        &"INTEL_SPATIAL",
		"musical":        &"INTEL_MUSICAL",
		"kinesthetic":    &"INTEL_KINESTHETIC",
		"interpersonal":  &"INTEL_INTERPERSONAL",
		"intrapersonal":  &"INTEL_INTRAPERSONAL",
		"naturalistic":   &"INTEL_NATURALISTIC",
	}
	for ikey in intel_map:
		var stat_id: StringName = intel_map[ikey]
		var ival: float = float(intel.get(ikey, 0.5))
		StatQuery.set_value(entity, stat_id, int(ival * 1000), 0)


func _sync_economic(entity: RefCounted) -> void:
	## Economic tendencies: float -1~+1 → int -1000~+1000
	StatQuery.set_value(entity, &"ECON_SAVING", int(entity.economic_tendencies.get("saving", 0.0) * 1000), 0)
	StatQuery.set_value(entity, &"ECON_RISK", int(entity.economic_tendencies.get("risk", 0.0) * 1000), 0)
	StatQuery.set_value(entity, &"ECON_GENEROSITY", int(entity.economic_tendencies.get("generosity", 0.0) * 1000), 0)
	StatQuery.set_value(entity, &"ECON_MATERIALISM", int(entity.economic_tendencies.get("materialism", 0.0) * 1000), 0)
	## Job satisfaction & wealth & status
	StatQuery.set_value(entity, &"JOB_SATISFACTION", int(entity.job_satisfaction * 1000), 0)
	StatQuery.set_value(entity, &"WEALTH_NORM", int(entity.wealth_norm * 1000), 0)
	StatQuery.set_value(entity, &"STATUS_SCORE", int((entity.status_score + 1.0) / 2.0 * 1000), 0)


func _sync_social_identity(entity: RefCounted) -> void:
	## Social identity stats for stat_cache queries
	StatQuery.set_value(entity, &"SOCIAL_TITLE_COUNT", entity.titles.size(), 0)
	StatQuery.set_value(entity, &"SOCIAL_FACTION_LOYALTY", int(entity.faction_loyalty * 1000), 0)
	StatQuery.set_value(entity, &"SOCIAL_FAITH_STRENGTH", int(entity.faith_strength * 1000), 0)
