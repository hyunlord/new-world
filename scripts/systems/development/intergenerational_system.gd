extends "res://scripts/core/simulation_system.gd"
# NO class_name (Godot 4.6 headless compatibility)

var _epi_config: Dictionary = {}
var _entity_manager  # injected
var _settlement_manager  # injected (may be null)
var _current_T: float = 0.30  # current transmission rate, can drift


func _init() -> void:
	system_name = "intergenerational"
	priority = 45  # after contagion(38), before population
	tick_interval = 240  # every 10 days (24 ticks/day)


func init(entity_manager, settlement_manager) -> void:
	_entity_manager = entity_manager
	_settlement_manager = settlement_manager
	_load_config()
	_current_T = _epi_config.get("transmission_rate_base", 0.30)


func _load_config() -> void:
	var file = FileAccess.open("res://data/epigenetic_config.json", FileAccess.READ)
	if file == null:
		push_error("[IntergenerationalSystem] Cannot open res://data/epigenetic_config.json")
		return
	var json = JSON.new()
	var err = json.parse(file.get_as_text())
	file.close()
	if err != OK:
		push_error("[IntergenerationalSystem] JSON parse error in epigenetic_config.json")
		return
	var data = json.get_data()
	if data is Dictionary:
		_epi_config = data


func execute_tick(tick: int) -> void:
	# Apply Meaney repair to entities with high parenting quality.
	if _entity_manager != null and _entity_manager.has_method("get_alive_entities"):
		var alive = _entity_manager.get_alive_entities()
		for i in range(alive.size()):
			var entity = alive[i]
			var parenting_quality = entity.get_meta("parenting_quality", 0.5)
			apply_meaney_repair(entity, parenting_quality, tick)

	# Child epigenetic load is calculated at birth, not every tick.
	if _settlement_manager == null:
		return
	if not _settlement_manager.has_method("get_active_settlements"):
		return
	var active_settlements = _settlement_manager.get_active_settlements()
	for i in range(active_settlements.size()):
		var settlement = active_settlements[i]
		check_generational_convergence(settlement.id, tick)


## [Yehuda et al., 2016 - FKBP5 methylation] Transgenerational epigenetic inheritance; T=0.30 (dampened, not copied).
## [Meaney, M.J., 2001 - Maternal care and offspring stress reactivity] Nurture > genetics as player lever (65% vs 35%).
func calculate_child_epigenetic_load(mother, father, adversity_index: float) -> float:
	## [Yehuda et al., 2016 - FKBP5 methylation / Transgenerational epigenetic inheritance]
	## [Meaney, M.J., 2001 - Maternal care and offspring stress reactivity via epigenetic mechanisms]
	## Trauma transmission is dampened inheritance, NOT copying (T=0.25~0.35, never 1.0).
	## Prenatal stress pathway (maternal cortisol exposure) is a separate channel from genetic epigenetics.
	## Design Note: T=0.30 base; genetics 35% vs nurture 65% — nurture is the more powerful lever for players.
	## Reference: Yehuda, R. et al. (2016). Biological Psychiatry, 80(5).
	## Reference: Meaney, M.J. (2001). Annual Review of Neuroscience, 24(1).
	var epi_load_m = mother.get_meta("epigenetic_load_effective", 0.05) if mother != null else 0.05
	var allo_norm_m = 0.0
	if mother != null and mother.emotion_data != null:
		allo_norm_m = clampf(mother.emotion_data.allostatic / 100.0, 0.0, 1.0)
	var scar_m = _scar_index(mother)
	var mother_weights = _epi_config.get("mother_state_weights", {})
	var m_state = clampf(
		mother_weights.get("epigenetic_load_effective", 0.50) * epi_load_m
		+ mother_weights.get("allostatic_load_normalized", 0.30) * allo_norm_m
		+ mother_weights.get("scar_index", 0.20) * scar_m,
		0.0,
		1.0
	)

	var epi_load_f = father.get_meta("epigenetic_load_effective", 0.05) if father != null else 0.05
	var allo_norm_f = 0.0
	if father != null and father.emotion_data != null:
		allo_norm_f = clampf(father.emotion_data.allostatic / 100.0, 0.0, 1.0)
	var scar_f = _scar_index(father)
	var father_weights = _epi_config.get("father_state_weights", {})
	var f_state = clampf(
		father_weights.get("epigenetic_load_effective", 0.60) * epi_load_f
		+ father_weights.get("allostatic_load_normalized", 0.25) * allo_norm_f
		+ father_weights.get("scar_index", 0.15) * scar_f,
		0.0,
		1.0
	)

	var base_t = _epi_config.get("transmission_rate_base", 0.30)
	var max_t = _epi_config.get("transmission_rate_max", 0.40)
	var bonus_t = _epi_config.get("transmission_rate_adversity_bonus", 0.10)
	var T = clampf(base_t + bonus_t * adversity_index, base_t, max_t)
	_current_T = T

	var preg_stress = mother.get_meta("pregnancy_avg_stress", 0.0) if mother != null else 0.0
	var malnutrition = mother.get_meta("malnutrition_index", 0.0) if mother != null else 0.0
	var prenatal_weights = _epi_config.get("prenatal_weights", {})
	var prenatal_max = _epi_config.get("prenatal_max", 0.35)
	var prenatal = clampf(
		prenatal_weights.get("pregnancy_avg_stress", 0.25) * preg_stress
		+ prenatal_weights.get("malnutrition_index", 0.10) * malnutrition,
		0.0,
		prenatal_max
	)

	var baseline = _epi_config.get("baseline", 0.05)
	var mw = _epi_config.get("maternal_weight", 0.65)
	var pw2 = _epi_config.get("paternal_weight", 0.35)
	var result = clampf(baseline + T * (mw * m_state + pw2 * f_state) + prenatal, 0.0, 1.0)
	return result


func _scar_index(entity) -> float:
	if entity == null:
		return 0.0
	var trauma_scars = entity.get_meta("trauma_scars", [])
	if trauma_scars.is_empty():
		var prop_scars = entity.get("trauma_scars")
		if prop_scars is Array:
			trauma_scars = prop_scars
	return clampf(float(trauma_scars.size()) / 5.0, 0.0, 1.0)


## [Yehuda et al., 2016 - HPA axis sensitivity formula] Higher epigenetic load → heightened cortisol reactivity per stressor.
func get_hpa_sensitivity(epigenetic_load: float) -> float:
	## [Yehuda 2016 + epigenetic_config hpa_sensitivity_formula]
	## Higher epigenetic load → heightened HPA axis reactivity → more stress gain per stressor.
	## Formula: 1.0 + load * 0.6 (range: 1.0 to 1.6 for load 0~1)
	return 1.0 + epigenetic_load * 0.6


## [Meaney, M.J., 2001 - Licking/grooming model] High-quality nurturing can partially reverse epigenetic stress programming.
func apply_meaney_repair(entity, parenting_quality: float, _tick: int) -> void:
	## [Meaney, M.J., 2001 - Maternal licking/grooming model]
	## High-quality nurturing care can partially reverse epigenetic stress programming.
	## Repair is slow (rate=0.002/tick) and only activates above quality threshold.
	## This is the player's primary lever for breaking intergenerational trauma cycles.
	## Reference: Meaney, M.J. (2001). Annual Review of Neuroscience, 24(1).
	if entity == null:
		return
	var threshold = _epi_config.get("meaney_repair_threshold", 0.70)
	if parenting_quality < threshold:
		return
	var repair_rate = _epi_config.get("meaney_repair_rate", 0.002)
	var current_load = entity.get_meta("epigenetic_load_effective", 0.05)
	var new_load = maxf(0.05, current_load - repair_rate * (parenting_quality - threshold) * 2.0)
	entity.set_meta("epigenetic_load_effective", new_load)


## [GPT Generational Stability - Fixed-point convergence] E* = (baseline + prenatal) / (1 - T); T=0.30 → stable at E*≈0.21.
func check_generational_convergence(settlement_id: int, tick: int) -> void:
	## [GPT Generational Stability Analysis — Fixed-point convergence theory]
	## Simplified recursion: E_{n+1} = baseline + T*E_n + prenatal_env
	## Stable fixed point: E* = (baseline + prenatal_env) / (1 - T)
	## T=0.30, prenatal=0.10 → E*≈0.21 (safe). T=0.30, prenatal=0.35 → E*≈0.57 (high-load equilibrium).
	## Collapse condition: war/famine + low parenting + high ACE ratio → T temporarily rises.
	## Reference: GPT convergence analysis (fixed-point theorem), Yehuda 2016 (empirical basis).
	if _settlement_manager == null or _entity_manager == null:
		return

	var members = []
	if _settlement_manager.has_method("get_settlement"):
		var settlement = _settlement_manager.get_settlement(settlement_id)
		if settlement != null:
			var ids = settlement.member_ids
			for i in range(ids.size()):
				var entity = _entity_manager.get_entity(ids[i]) if _entity_manager.has_method("get_entity") else null
				if entity != null and entity.is_alive:
					members.append(entity)

	# Fallback path if settlement member IDs are unavailable
	if members.is_empty() and _entity_manager.has_method("get_alive_entities"):
		var alive = _entity_manager.get_alive_entities()
		for i in range(alive.size()):
			var entity = alive[i]
			if entity.settlement_id == settlement_id:
				members.append(entity)

	if members.is_empty():
		return

	var total_members = float(members.size())
	var parenting_sum = 0.0
	var ace_high_count = 0
	var epi_sum = 0.0

	for i in range(members.size()):
		var entity = members[i]
		var parenting_quality = entity.get_meta("parenting_quality", 0.5)
		var ace_score = entity.get_meta("ace_score", 0.0)
		var epi = entity.get_meta("epigenetic_load_effective", 0.05)
		parenting_sum += parenting_quality
		epi_sum += epi
		if ace_score > 4.0:
			ace_high_count += 1

	var avg_parenting = parenting_sum / total_members
	var high_ace_ratio = float(ace_high_count) / total_members
	var avg_epi = epi_sum / total_members
	var adversity = _get_settlement_adversity(settlement_id)

	var collapse = _epi_config.get("collapse_conditions", {})
	if adversity > collapse.get("adversity_threshold", 0.85) \
	and avg_parenting < collapse.get("parenting_quality_threshold", 0.30) \
	and high_ace_ratio > 0.35:
		_current_T = clampf(
			_current_T + collapse.get("transmission_rate_boost", 0.05),
			0.0,
			_epi_config.get("transmission_rate_max", 0.40)
		)
		var chronicle = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
		if chronicle:
			chronicle.log_event("generational_collapse_risk", -1, Locale.ltr("GENERATIONAL_COLLAPSE_RISK"), 3, [], tick)
	else:
		_current_T = lerpf(_current_T, _epi_config.get("transmission_rate_base", 0.30), 0.01)
		if avg_epi < 0.25:
			var chronicle2 = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
			if chronicle2:
				chronicle2.log_event("generational_convergence", -1, Locale.ltr("GENERATIONAL_CONVERGENCE"), 2, [], tick)


func _get_settlement_adversity(settlement_id: int) -> float:
	if _settlement_manager == null or _entity_manager == null:
		return 0.0

	var members = []
	if _settlement_manager.has_method("get_settlement"):
		var settlement = _settlement_manager.get_settlement(settlement_id)
		if settlement != null:
			var ids = settlement.member_ids
			for i in range(ids.size()):
				var entity = _entity_manager.get_entity(ids[i]) if _entity_manager.has_method("get_entity") else null
				if entity != null and entity.is_alive:
					members.append(entity)

	if members.is_empty() and _entity_manager.has_method("get_alive_entities"):
		var alive = _entity_manager.get_alive_entities()
		for i in range(alive.size()):
			var entity = alive[i]
			if entity.settlement_id == settlement_id:
				members.append(entity)

	if members.is_empty():
		return 0.0

	var stress_sum = 0.0
	for i in range(members.size()):
		var entity = members[i]
		var stress = 0.0
		if entity.emotion_data != null:
			stress = entity.emotion_data.stress
		elif entity.emotions is Dictionary:
			stress = float(entity.emotions.get("stress", 0.0)) * 1000.0
		stress_sum += stress

	var avg_stress = stress_sum / float(members.size())
	return clampf(avg_stress / 2000.0, 0.0, 1.0)
