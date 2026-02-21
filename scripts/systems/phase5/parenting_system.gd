extends "res://scripts/core/simulation_system.gd"

# NO class_name — headless compatibility

var _entity_manager
var _attachment_system


func _init() -> void:
	system_name = "parenting"
	priority = 46  # after intergenerational(45), before population(50)
	tick_interval = 240  # every 10 days (24 ticks/day)


func init(entity_manager) -> void:
	_entity_manager = entity_manager
	var AttachmentSystem = load("res://scripts/systems/phase5/attachment_system.gd")
	_attachment_system = AttachmentSystem.new()


func execute_tick(tick: int) -> void:
	if _entity_manager == null:
		return
	var alive = _entity_manager.get_alive_entities()
	for entity in alive:
		if entity.emotion_data == null:
			continue

		# Update parenting quality for ALL adults (used by intergenerational each tick)
		if entity.age_stage == "adult" or entity.age_stage == "elder":
			var quality: float = _attachment_system.get_full_parenting_quality(entity)
			entity.set_meta("parenting_quality", quality)

		# One-time adulthood transition for entities that just crossed teen→adult
		if entity.age_stage == "adult" and not bool(entity.get_meta("adulthood_applied", false)):
			_apply_adulthood_transition(entity, tick)

		# Bandura coping modeling: children observe parents
		var childhood_data = entity.get_meta("childhood_data", null)
		if childhood_data is Dictionary:
			_apply_bandura_modeling(entity, childhood_data, tick)


## [Felitti 1998 + Teicher & Samson 2016 + Bowlby 1969]
## One-time permanent embedding of childhood adversity into adult stress parameters.
## ACE modifiers → stress_gain and break_threshold multipliers stored as entity meta.
## HEXACO caps applied once, irreversible.
## Reference: Felitti 1998, Teicher 2016, Bowlby 1969.
func _apply_adulthood_transition(entity, tick: int) -> void:
	entity.set_meta("adulthood_applied", true)

	# 1. ACE modifiers from ace_tracker or fallback
	var mods: Dictionary = _get_ace_modifiers(entity)

	# 2. Store multipliers in entity meta (read by stress_system + mental_break_system)
	entity.set_meta("ace_stress_gain_mult", float(mods.get("stress_gain_mult", 1.0)))
	entity.set_meta("ace_break_threshold_mult", float(mods.get("break_threshold_mult", 1.0)))

	# 3. Apply allostatic base directly to emotion_data
	var allo_base: float = float(mods.get("allostatic_base", 0.0))
	if allo_base > 0.0 and entity.emotion_data != null:
		entity.emotion_data.allostatic = clampf(
			entity.emotion_data.allostatic + allo_base, 0.0, 100.0
		)

	# 4. Apply HEXACO caps via ace_tracker
	var ace_tracker = entity.get_meta("ace_tracker", null)
	if ace_tracker != null and ace_tracker.has_method("apply_hexaco_caps"):
		ace_tracker.apply_hexaco_caps(entity)

	# 5. Apply attachment adult effects
	var attachment_type: String = str(entity.get_meta("attachment_type", "secure"))
	_attachment_system.apply_adult_effects(entity, attachment_type)

	# 6. Apply HPA sensitivity from epigenetic load
	var epi_load: float = float(entity.get_meta("epigenetic_load_effective", 0.05))
	var hpa_mult: float = 1.0 + epi_load * 0.6
	var current_stress_mult: float = float(entity.get_meta("ace_stress_gain_mult", 1.0))
	entity.set_meta("ace_stress_gain_mult", current_stress_mult * hpa_mult)

	# 7. Chronicle log
	var chronicle = Engine.get_main_loop().root.get_node_or_null("ChronicleSystem")
	if chronicle != null:
		var params: Dictionary = {"name": entity.entity_name}
		var desc: String = Locale.trf("ADULTHOOD_TRANSITION", params)
		chronicle.log_event("adulthood_transition", entity.id, desc, 3, [], tick, {
			"key": "ADULTHOOD_TRANSITION",
			"params": params,
		})

	emit_event("adulthood_transition", {
		"entity_id": entity.id,
		"entity_name": entity.entity_name,
		"ace_stress_mult": entity.get_meta("ace_stress_gain_mult", 1.0),
		"ace_break_mult": entity.get_meta("ace_break_threshold_mult", 1.0),
		"attachment_type": attachment_type,
		"tick": tick,
	})


## [Felitti 1998 — dose-response curve; ace_adult_modifiers.json 3-segment acceleration]
## Retrieve ACE adult modifiers from ace_tracker instance or compute from ace_score_total.
func _get_ace_modifiers(entity) -> Dictionary:
	var ace_tracker = entity.get_meta("ace_tracker", null)
	if ace_tracker != null and ace_tracker.has_method("calculate_adult_modifiers"):
		return ace_tracker.calculate_adult_modifiers()

	# Fallback: use ace_score_total stored directly in meta
	var score: float = float(entity.get_meta("ace_score_total", 0.0))
	# Simple inline table (mirrors ace_adult_modifiers.json key values)
	var ace_int: int = int(clampf(score, 0.0, 10.0))
	var table: Array = [
		{"stress_gain_mult": 1.00, "break_threshold_mult": 1.00, "allostatic_base": 0.0},
		{"stress_gain_mult": 1.06, "break_threshold_mult": 0.98, "allostatic_base": 3.0},
		{"stress_gain_mult": 1.12, "break_threshold_mult": 0.95, "allostatic_base": 6.0},
		{"stress_gain_mult": 1.18, "break_threshold_mult": 0.92, "allostatic_base": 9.0},
		{"stress_gain_mult": 1.24, "break_threshold_mult": 0.90, "allostatic_base": 12.0},
		{"stress_gain_mult": 1.40, "break_threshold_mult": 0.84, "allostatic_base": 19.0},
		{"stress_gain_mult": 1.56, "break_threshold_mult": 0.79, "allostatic_base": 26.0},
		{"stress_gain_mult": 1.72, "break_threshold_mult": 0.74, "allostatic_base": 33.0},
		{"stress_gain_mult": 1.94, "break_threshold_mult": 0.66, "allostatic_base": 43.0},
		{"stress_gain_mult": 2.16, "break_threshold_mult": 0.58, "allostatic_base": 53.0},
		{"stress_gain_mult": 2.38, "break_threshold_mult": 0.51, "allostatic_base": 63.0},
	]
	if ace_int >= 0 and ace_int < table.size():
		return table[ace_int]
	return {"stress_gain_mult": 1.0, "break_threshold_mult": 1.0, "allostatic_base": 0.0}


## [Bandura, 1977 - Social Learning Theory]
## Children observe adult caregivers' coping strategies and build familiarity with them.
## High-attachment, observant children learn adaptive coping more readily.
## Maladaptive coping (substance_use) spreads via observation despite low success rate.
## Design Note: simplified model — uses coping_modeling_rate_mult from attachment meta.
## Reference: Bandura, A. (1977). Social learning theory. Prentice-Hall.
func _apply_bandura_modeling(entity, childhood_data: Dictionary, _tick: int) -> void:
	# Only applies to children (not teens+ or adults)
	var stage: String = str(childhood_data.get("current_stage", ""))
	if stage == "" or stage == "adult" or stage == "teen":
		return

	# Get coping familiarity dict for this child
	var familiarity = childhood_data.get("coping_familiarity", {})
	if not (familiarity is Dictionary):
		familiarity = {}

	# Attachment-based learning rate modifier
	var coping_mult: float = float(entity.get_meta("attachment_coping_mult", 1.0))

	# Observed caregiver coping patterns stored in childhood_data
	var observed_coping = childhood_data.get("observed_caregiver_coping", {})
	if not (observed_coping is Dictionary) or observed_coping.is_empty():
		return

	for coping_id in observed_coping:
		var observation_strength: float = float(observed_coping.get(coping_id, 0.0))
		var base_rate: float = 0.002 * coping_mult * observation_strength
		# Maladaptive coping modeled more readily (immediate visible relief)
		if coping_id in ["substance_use", "behavioral_disengagement", "denial", "self_blame"]:
			base_rate *= 1.5
		var current: float = float(familiarity.get(coping_id, 0.0))
		familiarity[coping_id] = clampf(current + base_rate, 0.0, 1.0)

	childhood_data["coping_familiarity"] = familiarity
	entity.set_meta("childhood_data", childhood_data)


## [Felitti 1998 + Teicher & Samson 2016 - Adulthood transition public callback]
## Called by external systems (stress_system, population_system) when entity age_stage
## transitions to adult. Applies ACE modifiers, HEXACO caps, and attachment effects permanently.
## Idempotent: no-ops if adulthood_applied meta is already set.
func on_agent_reaches_adulthood(entity, tick: int) -> void:
	if entity == null:
		return
	if bool(entity.get_meta("adulthood_applied", false)):
		return
	_apply_adulthood_transition(entity, tick)
