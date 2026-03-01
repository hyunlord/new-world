extends "res://scripts/core/simulation/simulation_system.gd"

## Stress System — Phase 1 Pipeline
## Lazarus & Folkman (1984) Transactional Model
## Selye (1956) GAS reserve
## McEwen (1998) Allostatic Load
## Hobfoll (1989) COR loss aversion
## Yerkes & Dodson (1908) Eustress efficiency

var _entity_manager: RefCounted
var _stressor_defs: Dictionary = {}
var _trauma_scar_system = null  # TraumaScarSystem (RefCounted), set by main.gd

# ── Constants ─────────────────────────────────────────────────────────
const _TraitEffectCache = preload("res://scripts/systems/psychology/trait_effect_cache.gd")
const StatCurveScript = preload("res://scripts/core/stats/stat_curve.gd")
const STRESS_CLAMP_MAX: float = 2000.0
const STRESS_EPSILON: float = 0.05

const _TICK_SCALAR_LEN: int = 40
const _TICK_FLAG_LEN: int = 3
const _EMOTION_ORDER: Array[String] = [
	"fear",
	"anger",
	"sadness",
	"disgust",
	"surprise",
	"joy",
	"trust",
	"anticipation"
]
const _EMOTION_INDEX: Dictionary = {
	"fear": 0,
	"anger": 1,
	"sadness": 2,
	"disgust": 3,
	"surprise": 4,
	"joy": 5,
	"trust": 6,
	"anticipation": 7
}

# ── Phase 4 Extension: C05 Denial + Rebound Queue ─────────────────────
## Gross (1998) Emotion Regulation — cognitive reappraisal and suppression
## Folkman & Lazarus (1988) — denial as maladaptive avoidant coping
const DENIAL_REDIRECT_FRACTION: float = 0.60   # fraction of stress redirected to hidden accumulator
const DENIAL_MAX_ACCUMULATOR: float = 800.0    # cap on hidden threat accumulator
const REBOUND_DECAY_PER_TICK: float = 0.0      # rebounds don't decay (full delayed payment)

var _tick_scalar_inputs: PackedFloat32Array = PackedFloat32Array()
var _tick_flags: PackedByteArray = PackedByteArray()
var _tick_trace_per_tick: PackedFloat32Array = PackedFloat32Array()
var _tick_trace_decay: PackedFloat32Array = PackedFloat32Array()
var _rebound_amounts: PackedFloat32Array = PackedFloat32Array()
var _rebound_delays: PackedInt32Array = PackedInt32Array()
var _event_fast_current: PackedFloat32Array = PackedFloat32Array()
var _event_slow_current: PackedFloat32Array = PackedFloat32Array()


func _init() -> void:
	system_name = "stress"
	priority = 34        # after emotion(32), before social(37)
	tick_interval = GameConfig.STRESS_SYSTEM_TICK_INTERVAL


## Initializes the stress system with the entity manager and loads stressor definitions from JSON.
func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_load_stressor_defs()


## Sets the TraumaScarSystem reference used to apply scar-based resilience modifiers.
func set_trauma_scar_system(tss) -> void:
	_trauma_scar_system = tss


## Schedule a delayed stress rebound — called by CopingSystem when C05 Denial expires.
## Gross (1998): suppressed stress is stored, not eliminated; it rebounds when defenses drop.
## rebound_queue meta: Array of {amount: float, delay: int}
func schedule_rebound(entity_id: int, amount: float, delay_ticks: int) -> void:
	var entity = _entity_manager.get_entity(entity_id)
	if entity == null or entity.emotion_data == null:
		return
	var ed = entity.emotion_data
	var queue = ed.get_meta("rebound_queue", [])
	queue.append({"amount": amount, "delay": delay_ticks})
	ed.set_meta("rebound_queue", queue)


## Updates stress for all alive entities each tick: processes rebound queues, continuous stressors, emotion contributions, recovery, allostatic load, and reserve.
func execute_tick(_tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	for i in range(alive.size()):
		var entity = alive[i]
		if entity.emotion_data == null:
			continue
		var is_sleeping: bool = entity.current_action == "sleep"
		var is_safe: bool = entity.settlement_id >= 0
		# Phase 4: process any scheduled stress rebounds (C05 Denial expiry)
		_process_rebound_queue(entity.emotion_data)
		_update_entity_stress(entity, is_sleeping, is_safe)


func _update_entity_stress(entity: RefCounted, is_sleeping: bool, is_safe: bool) -> void:
	var ed = entity.emotion_data

	var breakdown: Dictionary = {}
	var hunger: float = StatQuery.get_normalized(entity, &"NEED_HUNGER")
	var energy: float = StatQuery.get_normalized(entity, &"NEED_ENERGY")
	var social: float = StatQuery.get_normalized(entity, &"NEED_SOCIAL")
	var ace_stress_mult: float = float(entity.get_meta("ace_stress_gain_mult", 1.0))
	var trait_accum_mult: float = _TraitEffectCache.get_stress_accum_mult(entity)
	var denial_active: bool = ed.get_meta("denial_active", false)
	var hidden: float = ed.get_meta("hidden_threat_accumulator", 0.0)
	var support_score: float = _calc_support_score(entity)
	var E_axis: float = StatQuery.get_normalized(entity, &"HEXACO_E")
	var C_axis: float = StatQuery.get_normalized(entity, &"HEXACO_C")
	var X_axis: float = StatQuery.get_normalized(entity, &"HEXACO_X")
	var O_axis: float = StatQuery.get_normalized(entity, &"HEXACO_O")
	var A_axis: float = StatQuery.get_normalized(entity, &"HEXACO_A")
	var H_axis: float = StatQuery.get_normalized(entity, &"HEXACO_H")

	var fear_val: float = ed.get_emotion("fear")
	var anger_val: float = ed.get_emotion("anger")
	var sadness_val: float = ed.get_emotion("sadness")
	var disgust_val: float = ed.get_emotion("disgust")
	var surprise_val: float = ed.get_emotion("surprise")
	var joy_val: float = ed.get_emotion("joy")
	var trust_val: float = ed.get_emotion("trust")
	var anticipation_val: float = ed.get_emotion("anticipation")
	var reserve_ratio: float = ed.reserve / 100.0
	var avoidant_mult: float = (
		GameConfig.ATTACHMENT_AVOIDANT_ALLO_MULT
		if str(entity.get_meta("attachment_type", "secure")) == "avoidant"
		else 1.0
	)
	var scar_resilience_mod: float = 0.0
	if _trauma_scar_system != null:
		scar_resilience_mod = _trauma_scar_system.get_scar_resilience_mod(entity)

	var trace_count: int = ed.stress_traces.size()
	_tick_trace_per_tick.resize(trace_count)
	_tick_trace_decay.resize(trace_count)
	for i in range(trace_count):
		var trace_data: Dictionary = ed.stress_traces[i]
		_tick_trace_per_tick[i] = float(trace_data.get("per_tick", 0.0))
		_tick_trace_decay[i] = float(trace_data.get("decay_rate", 0.05))

	if _tick_scalar_inputs.size() != _TICK_SCALAR_LEN:
		_tick_scalar_inputs.resize(_TICK_SCALAR_LEN)
	_tick_scalar_inputs[0] = hunger
	_tick_scalar_inputs[1] = energy
	_tick_scalar_inputs[2] = social
	_tick_scalar_inputs[3] = 0.0
	_tick_scalar_inputs[4] = 0.0
	_tick_scalar_inputs[5] = support_score
	_tick_scalar_inputs[6] = E_axis
	_tick_scalar_inputs[7] = fear_val
	_tick_scalar_inputs[8] = trust_val
	_tick_scalar_inputs[9] = C_axis
	_tick_scalar_inputs[10] = O_axis
	_tick_scalar_inputs[11] = reserve_ratio
	_tick_scalar_inputs[12] = anger_val
	_tick_scalar_inputs[13] = sadness_val
	_tick_scalar_inputs[14] = disgust_val
	_tick_scalar_inputs[15] = surprise_val
	_tick_scalar_inputs[16] = joy_val
	_tick_scalar_inputs[17] = anticipation_val
	_tick_scalar_inputs[18] = ed.valence
	_tick_scalar_inputs[19] = ed.arousal
	_tick_scalar_inputs[20] = ed.stress
	_tick_scalar_inputs[21] = ed.resilience
	_tick_scalar_inputs[22] = ed.reserve
	_tick_scalar_inputs[23] = ed.stress_delta_last
	_tick_scalar_inputs[24] = float(ed.gas_stage)
	_tick_scalar_inputs[25] = ed.allostatic
	_tick_scalar_inputs[26] = ace_stress_mult
	_tick_scalar_inputs[27] = trait_accum_mult
	_tick_scalar_inputs[28] = STRESS_EPSILON
	_tick_scalar_inputs[29] = DENIAL_REDIRECT_FRACTION
	_tick_scalar_inputs[30] = hidden
	_tick_scalar_inputs[31] = DENIAL_MAX_ACCUMULATOR
	_tick_scalar_inputs[32] = avoidant_mult
	_tick_scalar_inputs[33] = E_axis
	_tick_scalar_inputs[34] = C_axis
	_tick_scalar_inputs[35] = X_axis
	_tick_scalar_inputs[36] = O_axis
	_tick_scalar_inputs[37] = A_axis
	_tick_scalar_inputs[38] = H_axis
	_tick_scalar_inputs[39] = scar_resilience_mod

	if _tick_flags.size() != _TICK_FLAG_LEN:
		_tick_flags.resize(_TICK_FLAG_LEN)
	_tick_flags[0] = 1 if is_sleeping else 0
	_tick_flags[1] = 1 if is_safe else 0
	_tick_flags[2] = 1 if denial_active else 0
	var tick_step: Dictionary = StatCurveScript.stress_tick_step(
		_tick_trace_per_tick,
		_tick_trace_decay,
		0.01,
		_tick_scalar_inputs,
		_tick_flags
	)

	var s_hunger: float = float(tick_step.get("hunger", 0.0))
	if s_hunger > STRESS_EPSILON:
		breakdown["hunger"] = s_hunger
	var s_energy: float = float(tick_step.get("energy_deficit", 0.0))
	if s_energy > STRESS_EPSILON:
		breakdown["energy_deficit"] = s_energy
	var s_social: float = float(tick_step.get("social_isolation", 0.0))
	if s_social > STRESS_EPSILON:
		breakdown["social_isolation"] = s_social

	var updated: PackedFloat32Array = tick_step.get("updated_per_tick", PackedFloat32Array())
	var active_mask: PackedByteArray = tick_step.get("active_mask", PackedByteArray())
	var usable_len: int = mini(trace_count, mini(updated.size(), active_mask.size()))
	var next_traces: Array = []
	for i in range(usable_len):
		var trace: Dictionary = ed.stress_traces[i]
		var contribution: float = float(_tick_trace_per_tick[i])
		trace["per_tick"] = float(updated[i])
		if int(active_mask[i]) != 0:
			var key: String = "trace_%s" % str(trace.get("source_id", "unknown"))
			breakdown[key] = contribution
			next_traces.append(trace)
	ed.stress_traces = next_traces

	for i in range(_EMOTION_ORDER.size()):
		var emotion_name: String = _EMOTION_ORDER[i]
		var contrib: float = float(tick_step.get(emotion_name, 0.0))
		if absf(contrib) > STRESS_EPSILON:
			breakdown["emo_%s" % emotion_name] = contrib
	var va_contrib: float = float(tick_step.get("va_composite", 0.0))
	if va_contrib > STRESS_EPSILON:
		breakdown["va_composite"] = va_contrib
	var recovery: float = float(tick_step.get("recovery", 0.0))
	breakdown["recovery"] = -recovery

	var delta: float = float(tick_step.get("delta", 0.0))
	ed.set_meta("hidden_threat_accumulator", float(tick_step.get("hidden_threat_accumulator", hidden)))

	ed.stress = clampf(float(tick_step.get("stress", ed.stress)), 0.0, STRESS_CLAMP_MAX)
	ed.stress_delta_last = delta
	ed.stress_breakdown = breakdown

	# Shaken 상태 카운트다운
	var shaken_remaining: int = ed.get_meta("shaken_remaining", 0)
	if shaken_remaining > 0:
		shaken_remaining -= 1
		ed.set_meta("shaken_remaining", shaken_remaining)
		if shaken_remaining <= 0:
			ed.set_meta("shaken_work_penalty", 0.0)

	ed.reserve = float(tick_step.get("reserve", ed.reserve))
	ed.gas_stage = int(tick_step.get("gas_stage", ed.gas_stage))
	ed.allostatic = float(tick_step.get("allostatic", ed.allostatic))
	ed.resilience = float(tick_step.get("resilience", ed.resilience))
	var state_snapshot: Dictionary = tick_step

	# 8) 스트레스 상태
	_update_stress_state(ed, state_snapshot)

	# 9) 스트레스 → 감정 역방향
	_apply_stress_to_emotions(ed, state_snapshot)

	# 10) 디버그 로그
	_debug_log(entity, ed, delta)


# ── 8) 스트레스 상태 ──────────────────────────────────────────────────
func _update_stress_state(ed, snapshot: Dictionary) -> void:
	ed.stress_state = int(snapshot.get("stress_state", 0))


# ── 11) 스트레스 → 감정 역방향 ───────────────────────────────────────
func _apply_stress_to_emotions(ed, snapshot: Dictionary) -> void:
	ed.set_meta("stress_mu_sadness", float(snapshot.get("stress_mu_sadness", 0.0)))
	ed.set_meta("stress_mu_anger", float(snapshot.get("stress_mu_anger", 0.0)))
	ed.set_meta("stress_mu_fear", float(snapshot.get("stress_mu_fear", 0.0)))
	ed.set_meta("stress_mu_joy", float(snapshot.get("stress_mu_joy", 0.0)))
	ed.set_meta("stress_mu_trust", float(snapshot.get("stress_mu_trust", 0.0)))
	ed.set_meta("stress_neg_gain_mult", float(snapshot.get("stress_neg_gain_mult", 1.0)))
	ed.set_meta("stress_pos_gain_mult", float(snapshot.get("stress_pos_gain_mult", 1.0)))
	ed.set_meta("stress_blunt_mult", float(snapshot.get("stress_blunt_mult", 1.0)))


# ── Support score ─────────────────────────────────────────────────────
func _calc_support_score(_entity: RefCounted) -> float:
	var relationships = []
	if relationships.is_empty():
		return 0.3

	var strong: float = 0.0
	var weak_sum: float = 0.0

	for rel in relationships:
		var strength: float = rel.get("strength", 0.0)
		if strength > strong:
			weak_sum += strong
			strong = strength
		else:
			weak_sum += strength

	return clampf(0.65 * strong + 0.35 * (1.0 - exp(-weak_sum / 1.5)), 0.0, 1.0)


# ── Yerkes-Dodson work efficiency ─────────────────────────────────────
## Returns a Yerkes-Dodson work efficiency multiplier (0.35–1.10) based on the entity's current stress level.
func get_work_efficiency(ed) -> float:
	var shaken_penalty: float = ed.get_meta("shaken_work_penalty", 0.0)
	return StatCurveScript.stress_work_efficiency(ed.stress, shaken_penalty)


# ── Event stress injection (COR loss aversion x2.5) ──────────────────
## Injects an immediate stress event into an entity's emotion_data, with optional per-tick trace and COR loss-aversion multiplier.
func inject_stress_event(ed, source_id: String, instant: float,
		per_tick: float = 0.0, decay_rate: float = 0.05,
		is_loss: bool = false, appraisal_scale: float = 1.0) -> void:
	var scaled: Dictionary = StatCurveScript.stress_event_scaled(
		instant,
		per_tick,
		is_loss,
		1.0,
		1.0,
		1.0,
		appraisal_scale
	)
	var final_instant: float = float(scaled.get("final_instant", 0.0))
	var final_per_tick: float = float(scaled.get("final_per_tick", 0.0))

	ed.stress = clampf(ed.stress + final_instant, 0.0, STRESS_CLAMP_MAX)

	if absf(final_per_tick) > 0.01:
		ed.stress_traces.append({
			"source_id": source_id,
			"per_tick": final_per_tick,
			"decay_rate": decay_rate,
		})


# ── 스트레서 이벤트 데이터 로드 ────────────────────────────────────────
func _load_stressor_defs() -> void:
	var path: String = "res://data/stressor_events.json"
	if not FileAccess.file_exists(path):
		push_warning("[StressSystem] stressor_events.json not found")
		return
	var f = FileAccess.open(path, FileAccess.READ)
	if f == null:
		push_warning("[StressSystem] Cannot open stressor_events.json")
		return
	var text: String = f.get_as_text()
	f.close()
	var json = JSON.new()
	var err: int = json.parse(text)
	if err != OK:
		push_error("[StressSystem] stressor_events.json parse error: " + json.get_error_message())
		return
	var raw = json.get_data()
	# _comment 키 제거
	for key in raw.keys():
		if not key.begins_with("_comment"):
			var raw_def: Variant = raw[key]
			if typeof(raw_def) != TYPE_DICTIONARY:
				_stressor_defs[key] = raw_def
				continue
			var stressor_def: Dictionary = raw_def
			var compiled_personality: Dictionary = _compile_personality_modifiers(stressor_def.get("personality_modifiers", {}))
			var compiled_relationship: Dictionary = _compile_relationship_scaling(stressor_def.get("relationship_scaling", {}))
			var compiled_context: Dictionary = _compile_context_modifiers(stressor_def.get("context_modifiers", {}))
			var compiled_emo: Dictionary = _compile_emotion_inject(stressor_def.get("emotion_inject", {}))
			stressor_def["_p_specs"] = compiled_personality.get("specs", [])
			stressor_def["_p_traits"] = compiled_personality.get("traits", {})
			stressor_def["_r_method"] = compiled_relationship.get("method", "none")
			stressor_def["_r_min_mult"] = compiled_relationship.get("min_mult", 0.3)
			stressor_def["_r_max_mult"] = compiled_relationship.get("max_mult", 1.5)
			stressor_def["_c_keys"] = compiled_context.get("keys", PackedStringArray())
			stressor_def["_c_multipliers"] = compiled_context.get("multipliers", PackedFloat32Array())
			stressor_def["_emo_fast"] = compiled_emo.get("fast", PackedFloat32Array())
			stressor_def["_emo_slow"] = compiled_emo.get("slow", PackedFloat32Array())
			_stressor_defs[key] = stressor_def


# ── Personality-aware 이벤트 스트레스 주입 ───────────────────────────
## 학술: Lazarus (1984) 개인별 appraisal
## HEXACO: 같은 사건도 성격에 따라 stress 강도가 다름
## COR (Hobfoll 1989): is_loss=true → 2.5배
func inject_event(entity, event_id: String, context: Dictionary = {}) -> void:
	if entity == null:
		return
	var ed = entity.emotion_data
	if ed == null:
		return
	if not _stressor_defs.has(event_id):
		push_warning("[StressSystem] Unknown stressor event: %s" % event_id)
		return

	var sdef = _stressor_defs[event_id]

	# 1) Base 값
	var instant = float(sdef.get("base_instant", 0.0))
	var per_tick = float(sdef.get("base_per_tick", 0.0))
	var decay_rate = float(sdef.get("base_decay_rate", 0.05))
	var is_loss = sdef.get("is_loss", false)

	# 2) 성격 스케일
	var p_specs: Array = sdef.get("_p_specs", [])
	var p_traits: Dictionary = sdef.get("_p_traits", {})
	var personality_scale = _calc_personality_scale(entity, p_specs, p_traits)

	# 3) 관계 스케일
	var r_method: String = String(sdef.get("_r_method", "none"))
	var r_min_mult: float = float(sdef.get("_r_min_mult", 0.3))
	var r_max_mult: float = float(sdef.get("_r_max_mult", 1.5))
	var relationship_scale = _calc_relationship_scale(
		context,
		r_method,
		r_min_mult,
		r_max_mult
	)

	# 4) 상황 스케일
	var c_keys: PackedStringArray = sdef.get("_c_keys", PackedStringArray())
	var c_multipliers: PackedFloat32Array = sdef.get("_c_multipliers", PackedFloat32Array())
	var context_scale = _calc_context_scale(context, c_keys, c_multipliers)

	# 5) 최종 계산 (Rust curve helper)
	var scaled: Dictionary = StatCurveScript.stress_event_scaled(
		instant,
		per_tick,
		is_loss,
		personality_scale,
		relationship_scale,
		context_scale,
		1.0
	)
	var total_scale: float = float(scaled.get("total_scale", 1.0))
	var final_instant: float = float(scaled.get("final_instant", 0.0))
	var final_per_tick: float = float(scaled.get("final_per_tick", 0.0))

	# 6) Stress 주입
	ed.stress = clampf(ed.stress + final_instant, 0.0, STRESS_CLAMP_MAX)

	if absf(final_per_tick) > 0.01:
		ed.stress_traces.append({
			"source_id": event_id,
			"per_tick": final_per_tick,
			"decay_rate": decay_rate,
		})

	# 7) 감정 직접 주입
	var emo_fast: PackedFloat32Array = sdef.get("_emo_fast", PackedFloat32Array())
	var emo_slow: PackedFloat32Array = sdef.get("_emo_slow", PackedFloat32Array())
	_inject_emotions(ed, emo_fast, emo_slow, total_scale)

	# 8) 디버그 로그
	if GameConfig.DEBUG_STRESS_LOG:
		var ename = entity.entity_name if "entity_name" in entity else "?"
		print("[STRESS_EVENT] %s | %s | inst=%.0f ptk=%.1f | p=%.2f r=%.2f c=%.2f | loss=%s" % [
			ename, event_id, final_instant, final_per_tick,
			personality_scale, relationship_scale, context_scale,
			str(is_loss)
		])


func _calc_personality_scale(entity, p_specs: Array, p_traits: Dictionary) -> float:
	if p_specs.is_empty() and p_traits.is_empty():
		return 1.0

	var pd = entity.personality
	var trait_id_map: Dictionary = _build_trait_id_map(entity)
	var values: PackedFloat32Array = PackedFloat32Array()
	var weights: PackedFloat32Array = PackedFloat32Array()
	var high_amplifies: PackedByteArray = PackedByteArray()
	var trait_multipliers: PackedFloat32Array = PackedFloat32Array()

	for item in p_specs:
		if typeof(item) != TYPE_DICTIONARY:
			continue
		var spec: Dictionary = item
		var weight: float = float(spec.get("weight", 0.0))
		var is_high_amplifies: bool = bool(spec.get("high_amplifies", true))
		var spec_kind: String = String(spec.get("kind", "facet"))
		var spec_id: String = String(spec.get("id", ""))

		# 축 또는 facet 값 가져오기
		var value: float = 0.5
		if pd != null:
			if spec_kind == "axis":
				var axis_stat: String = String(spec.get("axis_stat", "HEXACO_" + spec_id))
				value = StatQuery.get_normalized(entity, StringName(axis_stat))
			else:
				value = float(pd.facets.get(spec_id, 0.5))

		values.append(value)
		weights.append(weight)
		high_amplifies.append(1 if is_high_amplifies else 0)

	# Trait 배수
	for trait_id in p_traits:
		if trait_id_map.has(trait_id):
			trait_multipliers.append(float(p_traits[trait_id]))

	return StatCurveScript.stress_personality_scale(
		values,
		weights,
		high_amplifies,
		trait_multipliers
	)


func _calc_relationship_scale(
	context: Dictionary,
	method: String,
	min_m: float,
	max_m: float
) -> float:
	var bond: float = float(context.get("bond_strength", 0.5))
	return StatCurveScript.stress_relationship_scale(method, bond, min_m, max_m)


func _calc_context_scale(
	context: Dictionary,
	c_keys: PackedStringArray,
	c_multipliers: PackedFloat32Array
) -> float:
	var active_multipliers: PackedFloat32Array = PackedFloat32Array()
	var count: int = mini(c_keys.size(), c_multipliers.size())
	for idx in range(count):
		var context_key: String = c_keys[idx]
		if context.get(context_key, false):
			active_multipliers.append(c_multipliers[idx])
	return StatCurveScript.stress_context_scale(active_multipliers)


func _compile_personality_modifiers(p_mods: Variant) -> Dictionary:
	var specs: Array = []
	var traits: Dictionary = {}
	if typeof(p_mods) != TYPE_DICTIONARY:
		return {"specs": specs, "traits": traits}

	var p_mods_dict: Dictionary = p_mods
	var trait_mods: Variant = p_mods_dict.get("traits", {})
	if typeof(trait_mods) == TYPE_DICTIONARY:
		traits = trait_mods

	for key in p_mods_dict:
		if key == "traits":
			continue
		var mod: Variant = p_mods_dict[key]
		if typeof(mod) != TYPE_DICTIONARY:
			continue
		var mod_dict: Dictionary = mod
		var key_str: String = String(key)
		var spec_kind: String = "facet"
		var spec_id: String = key_str
		if key_str.ends_with("_axis"):
			spec_kind = "axis"
			spec_id = key_str.substr(0, key_str.length() - 5)
		specs.append({
			"kind": spec_kind,
			"id": spec_id,
			"axis_stat": "HEXACO_" + spec_id,
			"weight": float(mod_dict.get("weight", 0.0)),
			"high_amplifies": String(mod_dict.get("direction", "high_amplifies")) == "high_amplifies",
		})

	return {"specs": specs, "traits": traits}


func _compile_relationship_scaling(r_def: Variant) -> Dictionary:
	var method: String = "none"
	var min_mult: float = 0.3
	var max_mult: float = 1.5
	if typeof(r_def) == TYPE_DICTIONARY:
		var r_def_dict: Dictionary = r_def
		method = String(r_def_dict.get("method", "none"))
		min_mult = float(r_def_dict.get("min_mult", 0.3))
		max_mult = float(r_def_dict.get("max_mult", 1.5))
	return {
		"method": method,
		"min_mult": min_mult,
		"max_mult": max_mult,
	}


func _compile_context_modifiers(c_mods: Variant) -> Dictionary:
	var keys: PackedStringArray = PackedStringArray()
	var multipliers: PackedFloat32Array = PackedFloat32Array()
	if typeof(c_mods) != TYPE_DICTIONARY:
		return {"keys": keys, "multipliers": multipliers}

	var c_mods_dict: Dictionary = c_mods
	for key in c_mods_dict:
		keys.append(String(key))
		multipliers.append(float(c_mods_dict[key]))
	return {"keys": keys, "multipliers": multipliers}


func _build_trait_id_map(entity) -> Dictionary:
	var out: Dictionary = {}
	for t in entity.display_traits:
		var trait_id: String = String(t.get("id", ""))
		if not trait_id.is_empty():
			out[trait_id] = true
	return out


func _compile_emotion_inject(emo_inject: Variant) -> Dictionary:
	var fast: PackedFloat32Array = PackedFloat32Array()
	var slow: PackedFloat32Array = PackedFloat32Array()
	fast.resize(_EMOTION_ORDER.size())
	slow.resize(_EMOTION_ORDER.size())

	if typeof(emo_inject) != TYPE_DICTIONARY:
		return {"fast": fast, "slow": slow}

	var emo_inject_dict: Dictionary = emo_inject
	for key in emo_inject_dict:
		var key_str: String = String(key)
		var raw_val: float = float(emo_inject_dict[key])
		var last_underscore: int = key_str.rfind("_")
		if last_underscore < 0:
			continue
		var emo_name: String = key_str.substr(0, last_underscore)
		var layer: String = key_str.substr(last_underscore + 1)
		var idx: int = int(_EMOTION_INDEX.get(emo_name, -1))
		if idx < 0:
			continue
		if layer == "fast":
			fast[idx] += raw_val
		elif layer == "slow":
			slow[idx] += raw_val

	return {"fast": fast, "slow": slow}


func _inject_emotions(
	ed,
	emo_fast: PackedFloat32Array,
	emo_slow: PackedFloat32Array,
	scale: float
) -> void:
	if emo_fast.size() == 0 and emo_slow.size() == 0:
		return

	_event_fast_current.resize(_EMOTION_ORDER.size())
	_event_slow_current.resize(_EMOTION_ORDER.size())
	for idx in range(_EMOTION_ORDER.size()):
		var emotion_name: String = _EMOTION_ORDER[idx]
		_event_fast_current[idx] = float(ed.fast.get(emotion_name, 0.0))
		_event_slow_current[idx] = float(ed.slow.get(emotion_name, 0.0))

	var out: Dictionary = StatCurveScript.stress_emotion_inject_step(
		_event_fast_current,
		_event_slow_current,
		emo_fast,
		emo_slow,
		scale
	)
	var next_fast: PackedFloat32Array = out.get("fast", _event_fast_current)
	var next_slow: PackedFloat32Array = out.get("slow", _event_slow_current)
	var count: int = mini(_EMOTION_ORDER.size(), mini(next_fast.size(), next_slow.size()))
	for idx in range(count):
		var emotion_name: String = _EMOTION_ORDER[idx]
		if ed.fast.has(emotion_name):
			ed.fast[emotion_name] = float(next_fast[idx])
		if ed.slow.has(emotion_name):
			ed.slow[emotion_name] = float(next_slow[idx])


# ── Phase 4: Rebound Queue Processing ────────────────────────────────
func _process_rebound_queue(ed: RefCounted) -> void:
	## Gross (1998): suppressed stress surfaces when denial coping terminates
	## Queue format: Array of {amount: float, delay: int}
	var queue: Array = ed.get_meta("rebound_queue", [])
	if queue.is_empty():
		return

	var queue_count: int = queue.size()
	_rebound_amounts.resize(queue_count)
	_rebound_delays.resize(queue_count)
	for idx in range(queue_count):
		var entry: Variant = queue[idx]
		if typeof(entry) == TYPE_DICTIONARY:
			var entry_dict: Dictionary = entry
			_rebound_amounts[idx] = float(entry_dict.get("amount", 0.0))
			_rebound_delays[idx] = int(entry_dict.get("delay", 0))
		else:
			_rebound_amounts[idx] = 0.0
			_rebound_delays[idx] = 1

	var step: Dictionary = StatCurveScript.stress_rebound_queue_step(
		_rebound_amounts,
		_rebound_delays,
		REBOUND_DECAY_PER_TICK
	)
	var total_rebound: float = float(step.get("total_rebound", 0.0))
	var remaining_amounts: PackedFloat32Array = step.get("remaining_amounts", PackedFloat32Array())
	var remaining_delays: PackedInt32Array = step.get("remaining_delays", PackedInt32Array())
	var remaining: Array = []
	var remaining_count: int = mini(remaining_amounts.size(), remaining_delays.size())
	for idx in range(remaining_count):
		remaining.append({
			"amount": float(remaining_amounts[idx]),
			"delay": int(remaining_delays[idx]),
		})

	ed.set_meta("rebound_queue", remaining)

	if total_rebound > 0.0:
		ed.stress = clampf(ed.stress + total_rebound, 0.0, STRESS_CLAMP_MAX)
		# Clears hidden accumulator proportionally (rebounded stress is no longer hidden)
		var hidden: float = ed.get_meta("hidden_threat_accumulator", 0.0)
		ed.set_meta("hidden_threat_accumulator", maxf(0.0, hidden - total_rebound))


# ── Debug log ─────────────────────────────────────────────────────────
func _debug_log(entity: RefCounted, ed, delta: float) -> void:
	if not GameConfig.DEBUG_STRESS_LOG:
		return

	var ename = entity.entity_name
	var parts: Array = []
	for key in ed.stress_breakdown:
		parts.append("%s:%.1f" % [key, ed.stress_breakdown[key]])

	print("[STRESS] %s | S:%.0f(D%+.1f) R:%.0f A:%.1f Res:%.2f GAS:%d | %s" % [
		ename, ed.stress, delta, ed.reserve, ed.allostatic,
		ed.resilience, ed.gas_stage, ", ".join(parts)
	])
