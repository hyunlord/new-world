extends "res://scripts/core/simulation_system.gd"

## Plutchik 8-emotion update engine with 3-layer temporal dynamics.
## Fast (episodic decay) + Slow (mood/baseline, OU process) + Memory trace (long-term scars).
## Appraisal-based impulse from events. HEXACO personality coupling.
## References:
##   Plutchik (1980, 2001), Russell (1980), Lazarus (1991), Scherer (2009)
##   Verduyn & Brans (2012), Hatfield et al. (1993)

const EmotionDataScript = preload("res://scripts/core/emotion_data.gd")

var _entity_manager: RefCounted
var _chronicle_system: RefCounted = null
var _event_presets: Dictionary = {}
var _pending_events: Dictionary = {}  # entity_id -> Array of event dicts
var _fast_half_life: Dictionary = {}
var _slow_half_life: Dictionary = {}
var _opposite_pairs: Dictionary = {}
var _inhibition_gamma: float = 0.3
var _memory_trace_threshold: float = 20.0
var _memory_trace_ratio: float = 0.3
var _memory_trace_default_hl_hours: float = 720.0  # 30 days
var _memory_trace_trauma_hl_hours: float = 8760.0  # 365 days
var _habituation_eta: float = 0.2
var _contagion_kappa: Dictionary = {}
var _contagion_distance_scale: float = 5.0
var _contagion_min_source: float = 10.0
var _break_beta: float = 60.0
var _break_tick_prob: float = 0.01
var _break_behaviors: Dictionary = {}
var _personality_sensitivity: Dictionary = {}
var _baselines: Dictionary = {}
var _half_life_adjustments: Dictionary = {}


func _init() -> void:
	system_name = "emotions"
	priority = 32
	tick_interval = 12  # Once per game day


func init(entity_manager: RefCounted) -> void:
	_entity_manager = entity_manager
	_load_event_presets()
	_load_decay_parameters()


func _load_event_presets() -> void:
	var path: String = "res://data/emotions/event_presets.json"
	if not FileAccess.file_exists(path):
		push_warning("[EmotionSystem] Event presets not found: %s" % path)
		return
	var f: FileAccess = FileAccess.open(path, FileAccess.READ)
	if f == null:
		return
	var json: JSON = JSON.new()
	if json.parse(f.get_as_text()) == OK:
		var data = json.data
		if data is Dictionary and data.has("events"):
			_event_presets = data["events"]


func _load_decay_parameters() -> void:
	var dp = SpeciesManager.decay_parameters
	_fast_half_life = dp.get("fast_half_life_hours", {"joy": 0.75, "trust": 2.0, "fear": 0.3, "surprise": 0.05, "sadness": 0.5, "disgust": 0.1, "anger": 0.4, "anticipation": 3.0})
	_slow_half_life = dp.get("slow_half_life_hours", {"joy": 48.0, "trust": 72.0, "fear": 24.0, "surprise": 6.0, "sadness": 120.0, "disgust": 12.0, "anger": 12.0, "anticipation": 36.0})
	_opposite_pairs = dp.get("opposite_pairs", {"joy": "sadness", "sadness": "joy", "trust": "disgust", "disgust": "trust", "fear": "anger", "anger": "fear", "surprise": "anticipation", "anticipation": "surprise"})
	_inhibition_gamma = float(dp.get("inhibition_gamma", 0.3))
	_memory_trace_threshold = float(dp.get("memory_trace_threshold", 20.0))
	_memory_trace_ratio = float(dp.get("memory_trace_ratio", 0.3))
	var mt_default_days = float(dp.get("memory_trace_default_half_life_days", 30))
	_memory_trace_default_hl_hours = mt_default_days * 24.0
	var mt_trauma_days = float(dp.get("memory_trace_trauma_half_life_days", 365))
	_memory_trace_trauma_hl_hours = mt_trauma_days * 24.0
	_habituation_eta = float(dp.get("habituation_eta", 0.2))
	var contagion_data = dp.get("contagion", {})
	_contagion_kappa = contagion_data.get("kappa", {"anger": 0.12, "fear": 0.10, "joy": 0.08, "disgust": 0.06, "trust": 0.06, "sadness": 0.04, "surprise": 0.03, "anticipation": 0.03})
	_contagion_distance_scale = float(contagion_data.get("distance_scale", 5.0))
	_contagion_min_source = float(contagion_data.get("min_source", 10.0))
	var break_data = dp.get("mental_break", {})
	_break_beta = float(break_data.get("beta", 60.0))
	_break_tick_prob = float(break_data.get("tick_prob", 0.01))
	_break_behaviors = break_data.get("behaviors", {})
	_personality_sensitivity = dp.get("personality_sensitivity", {})
	_baselines = dp.get("baselines", {})
	_half_life_adjustments = dp.get("half_life_adjustments", {})


## Queue an emotional event for an entity (called by other systems via SimulationBus)
func queue_event(entity_id: int, event_key: String, overrides: Dictionary = {}) -> void:
	if not _pending_events.has(entity_id):
		_pending_events[entity_id] = []
	var preset = _event_presets.get(event_key, {})
	if preset.is_empty():
		push_warning("[EmotionSystem] Unknown event preset: %s" % event_key)
		return
	var event: Dictionary = preset.duplicate()
	# Apply overrides (e.g. custom intensity)
	for key in overrides:
		event[key] = overrides[key]
	_pending_events[entity_id].append(event)


func execute_tick(tick: int) -> void:
	var alive: Array = _entity_manager.get_alive_entities()
	var dt_hours: float = float(tick_interval) / 12.0  # 12 ticks = 1 game day = 24h, so 1 tick = 2h

	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		# Initialize emotion_data if missing
		if entity.emotion_data == null:
			entity.emotion_data = EmotionDataScript.new()
		var ed: RefCounted = entity.emotion_data
		var pd: RefCounted = entity.personality
		if pd == null:
			continue

		# Gather events for this entity
		var events: Array = _pending_events.get(entity.id, [])

		# Step 1: Event -> emotion impulse (appraisal-based)
		var impulse: Dictionary = _calculate_event_impulse(events, pd, ed)

		# Step 2: Fast layer update (exponential decay + impulse)
		# ★ stress gain multipliers (set by stress_system each tick)
		var stress_neg_gain: float = ed.get_meta("stress_neg_gain_mult", 1.0)
		var stress_pos_gain: float = ed.get_meta("stress_pos_gain_mult", 1.0)
		var stress_blunt: float = ed.get_meta("stress_blunt_mult", 1.0)
		var _negative_emos: Array = ["fear", "anger", "sadness", "disgust", "surprise"]
		var _positive_emos: Array = ["joy", "trust", "anticipation"]

		for emo in ed.fast:
			var hl: float = _get_adjusted_half_life(emo, pd, "fast")
			var k: float = 0.693147 / hl
			var emo_impulse: float = impulse.get(emo, 0.0)
			# Apply stress sensitivity to impulse
			if emo in _negative_emos:
				emo_impulse *= stress_neg_gain * stress_blunt
			elif emo in _positive_emos:
				emo_impulse *= stress_pos_gain * stress_blunt
			ed.fast[emo] = ed.fast[emo] * exp(-k * dt_hours) + emo_impulse
			ed.fast[emo] = maxf(ed.fast[emo], 0.0)

		# Step 3: Slow layer update (Ornstein-Uhlenbeck mean-reverting process)
		# ★ stress shifts OU target baselines
		var _stress_mu_map: Dictionary = {
			"sadness": ed.get_meta("stress_mu_sadness", 0.0),
			"anger":   ed.get_meta("stress_mu_anger",   0.0),
			"fear":    ed.get_meta("stress_mu_fear",    0.0),
			"joy":     ed.get_meta("stress_mu_joy",     0.0),
			"trust":   ed.get_meta("stress_mu_trust",   0.0),
		}
		for emo in ed.slow:
			var baseline: float = _get_baseline(emo, pd)
			var mu_shift: float = _stress_mu_map.get(emo, 0.0)
			var effective_baseline: float = clampf(baseline + mu_shift, 0.0, 30.0)
			var hl_slow: float = _slow_half_life.get(emo, 48.0)
			var k_slow: float = 0.693147 / hl_slow
			var sigma: float = 0.5  # mood fluctuation
			ed.slow[emo] = effective_baseline + (ed.slow[emo] - effective_baseline) * exp(-k_slow * dt_hours)
			ed.slow[emo] += sigma * sqrt(dt_hours) * _randfn()
			ed.slow[emo] = clampf(ed.slow[emo], 0.0, 30.0)

		# Step 4: Memory trace decay
		for emo in ed.memory_traces:
			var traces: Array = ed.memory_traces[emo]
			for j in range(traces.size() - 1, -1, -1):
				traces[j].intensity *= exp(-traces[j].decay_rate * dt_hours)
				if traces[j].intensity < 0.5:
					traces.remove_at(j)

		# Step 5: Opposite emotion inhibition
		for emo in _opposite_pairs:
			var opp: String = _opposite_pairs[emo]
			var opp_total: float = ed.get_emotion(opp)
			if opp_total > 0.0:
				ed.fast[emo] = maxf(0.0, ed.fast[emo] - _inhibition_gamma * opp_total)

		# Step 6: Valence-Arousal recalculation
		ed.recalculate_va()

		# Step 7: Stress accumulation handled by StressSystem (stress_system.gd, priority=34)

		# Step 8: Record habituation for processed events
		for j in range(events.size()):
			var cat = events[j].get("category", "generic")
			_record_habituation(ed, cat, tick)

		# Step 9: Write back legacy emotions for backward compatibility
		var legacy: Dictionary = ed.to_legacy_dict()
		for key in legacy:
			entity.emotions[key] = legacy[key]

		# Step 11: Mental break check
		_check_mental_break(entity, dt_hours, tick)

	# Step 10: Emotional contagion (settlement-scoped)
	var settlement_groups: Dictionary = {}
	for i in range(alive.size()):
		var entity: RefCounted = alive[i]
		var sid: int = entity.settlement_id
		if not settlement_groups.has(sid):
			settlement_groups[sid] = []
		settlement_groups[sid].append(entity)
	for sid in settlement_groups:
		_apply_contagion_settlement(settlement_groups[sid], dt_hours)

	# Clear pending events
	_pending_events.clear()


# ═══════════════════════════════════════════════════
# Appraisal-Based Impulse (Lazarus 1991, Scherer 2009)
# ═══════════════════════════════════════════════════

func _calculate_event_impulse(events: Array, pd: RefCounted, ed: RefCounted) -> Dictionary:
	var total: Dictionary = {}
	for emo in ed.fast:
		total[emo] = 0.0

	for j in range(events.size()):
		var event: Dictionary = events[j]
		var g: float = float(event.get("goal_congruence", 0.0))
		var n: float = float(event.get("novelty", 0.0))
		var c: float = float(event.get("controllability", 0.5))
		var a: float = float(event.get("agency", 0.0))
		var m: float = float(event.get("norm_violation", 0.0))
		var p: float = float(event.get("pathogen", 0.0))
		var b: float = float(event.get("social_bond", 0.0))
		var fr: float = float(event.get("future_relevance", 0.0))
		var base_intensity: float = float(event.get("intensity", 30.0))

		# Habituation factor
		var hab: float = _get_habituation(ed, event.get("category", "generic"))

		# Personality sensitivity
		var sens: Dictionary = _get_personality_sensitivity(pd)

		# Appraisal -> 8 emotion impulses
		var impulse: Dictionary = {}
		impulse["joy"] = base_intensity * maxf(0.0, g) * (1.0 + 0.5 * n) * sens.get("joy", 1.0)
		impulse["sadness"] = base_intensity * maxf(0.0, -g) * (1.0 - c) * sens.get("sadness", 1.0)
		impulse["anger"] = base_intensity * maxf(0.0, -g) * c * maxf(0.0, -a + m) * sens.get("anger", 1.0)
		impulse["fear"] = base_intensity * maxf(0.0, -g) * (1.0 - c) * (0.5 + 0.5 * n) * sens.get("fear", 1.0)
		impulse["disgust"] = base_intensity * (p + 0.7 * m) * (0.5 + 0.5 * maxf(0.0, -g)) * sens.get("disgust", 1.0)
		impulse["surprise"] = base_intensity * n * sens.get("surprise", 1.0)
		impulse["trust"] = base_intensity * maxf(0.0, b) * (1.0 - p) * (1.0 - m) * sens.get("trust", 1.0)
		impulse["anticipation"] = base_intensity * fr * (0.5 + 0.5 * maxf(0.0, g)) * sens.get("anticipation", 1.0)

		# Apply habituation and accumulate
		for emo in impulse:
			impulse[emo] *= hab
			total[emo] += impulse[emo]

		# Create memory traces for strong impulses
		if base_intensity >= _memory_trace_threshold:
			_create_memory_trace(ed, impulse, event)

	return total


# ═══════════════════════════════════════════════════
# HEXACO -> Emotion Coupling
# ═══════════════════════════════════════════════════

func _get_personality_sensitivity(pd: RefCounted) -> Dictionary:
	var result: Dictionary = {}
	for emo in _personality_sensitivity:
		var config = _personality_sensitivity[emo]
		if config is Array:
			var total: float = 0.0
			for i in range(config.size()):
				var entry = config[i]
				var axis_id = str(entry.get("axis", "X"))
				var coeff = float(entry.get("coeff", 0.0))
				var z = pd.to_zscore(pd.axes.get(axis_id, 0.5))
				total += coeff * z
			result[emo] = exp(total)
		elif config is Dictionary:
			var axis_id = str(config.get("axis", "X"))
			var coeff = float(config.get("coeff", 0.0))
			var z = pd.to_zscore(pd.axes.get(axis_id, 0.5))
			result[emo] = exp(coeff * z)
		else:
			result[emo] = 1.0
	return result


func _get_adjusted_half_life(emo: String, pd: RefCounted, layer: String) -> float:
	var base: float = _fast_half_life.get(emo, 1.0) if layer == "fast" else _slow_half_life.get(emo, 48.0)
	var adj = _half_life_adjustments.get(emo, {})
	if adj.is_empty():
		return base
	var axis_id = str(adj.get("axis", "X"))
	var coeff = float(adj.get("coeff", 0.0))
	var z = pd.to_zscore(pd.axes.get(axis_id, 0.5))
	return base * exp(coeff * z)


func _get_baseline(emo: String, pd: RefCounted) -> float:
	var cfg = _baselines.get(emo, {})
	var base_val = float(cfg.get("base", 0.0))
	if not cfg.has("axis"):
		return base_val
	var axis_id = str(cfg.get("axis", "X"))
	var scale_val = float(cfg.get("scale", 0.0))
	var min_val = float(cfg.get("min", 0.0))
	var max_val = float(cfg.get("max", 100.0))
	var z = pd.to_zscore(pd.axes.get(axis_id, 0.5))
	return clampf(base_val + scale_val * z, min_val, max_val)


# ═══════════════════════════════════════════════════
# Habituation
# ═══════════════════════════════════════════════════

func _get_habituation(ed: RefCounted, category: String) -> float:
	if not ed.habituation.has(category):
		return 1.0
	var data = ed.habituation[category]
	var n_count: int = int(data.get("count", 0))
	var eta: float = _habituation_eta
	return exp(-eta * float(n_count))


func _record_habituation(ed: RefCounted, category: String, current_tick: int) -> void:
	if not ed.habituation.has(category):
		ed.habituation[category] = {"count": 0, "last_tick": current_tick}
	ed.habituation[category]["count"] = int(ed.habituation[category].get("count", 0)) + 1
	ed.habituation[category]["last_tick"] = current_tick


# ═══════════════════════════════════════════════════
# Emotional Contagion
# ═══════════════════════════════════════════════════

## Apply emotional contagion within a settlement
## Scope: settlement-only to avoid O(n²) global computation
func _apply_contagion_settlement(members: Array, dt_hours: float) -> void:
	if members.size() < 2:
		return

	# Pre-collect emotion snapshots to prevent order-dependent effects
	var snapshots: Array = []
	for i in range(members.size()):
		var entity: RefCounted = members[i]
		if entity.emotion_data == null:
			snapshots.append(null)
			continue
		var snap: Dictionary = {}
		for emo in _contagion_kappa:
			snap[emo] = entity.emotion_data.get_emotion(emo)
		snapshots.append(snap)

	for i in range(members.size()):
		var target: RefCounted = members[i]
		if target.emotion_data == null or target.personality == null:
			continue
		var ed: RefCounted = target.emotion_data

		# Susceptibility: E↑ and A↑ → more susceptible to emotional contagion
		var z_E: float = target.personality.to_zscore(target.personality.axes.get("E", 0.5))
		var z_A: float = target.personality.to_zscore(target.personality.axes.get("A", 0.5))
		var susceptibility: float = exp(0.2 * z_E + 0.1 * z_A)

		var delta: Dictionary = {}
		for emo in _contagion_kappa:
			delta[emo] = 0.0

		for j in range(members.size()):
			if i == j:
				continue
			var source: RefCounted = members[j]
			if snapshots[j] == null:
				continue

			# Distance decay
			var dx: float = float(target.position.x - source.position.x)
			var dy: float = float(target.position.y - source.position.y)
			var distance: float = sqrt(dx * dx + dy * dy)
			var distance_factor: float = exp(-distance / _contagion_distance_scale)

			# Skip if too far (optimization)
			if distance_factor < 0.01:
				continue

			# Relationship strength (default 0.5 if no relationship system)
			var relationship: float = 0.5

			for emo in _contagion_kappa:
				var source_val: float = snapshots[j][emo]
				if source_val > _contagion_min_source:
					delta[emo] += _contagion_kappa[emo] * source_val * distance_factor * relationship * susceptibility * dt_hours

		# Apply contagion deltas to fast layer
		for emo in delta:
			if delta[emo] > 0.0:
				ed.fast[emo] = ed.fast[emo] + delta[emo]


# ═══════════════════════════════════════════════════
# Mental Break Detection
# ═══════════════════════════════════════════════════

## Check if entity should enter a mental break state
func _check_mental_break(entity: RefCounted, dt_hours: float, tick: int) -> void:
	var ed: RefCounted = entity.emotion_data
	if ed == null:
		return
	var pd: RefCounted = entity.personality
	if pd == null:
		return

	# If already in a break, count down and exit when done
	if ed.mental_break_type != "":
		ed.mental_break_remaining -= dt_hours
		# Apply energy drain during break
		var behavior = _break_behaviors.get(ed.mental_break_type, {})
		var drain: float = float(behavior.get("energy_drain", 1.0))
		entity.energy = maxf(0.0, entity.energy - drain * dt_hours / 24.0)
		if ed.mental_break_remaining <= 0.0:
			# Break ends — reduce stress to 50% (not 0, prevents immediate re-break)
			ed.stress *= 0.5
			var old_type: String = ed.mental_break_type
			ed.mental_break_type = ""
			ed.mental_break_remaining = 0.0
			emit_event("mental_break_ended", {
				"entity_id": entity.id,
				"entity_name": entity.entity_name,
				"break_type": old_type,
			})
			if _chronicle_system != null:
				var end_type_name: String = Locale.ltr("MENTAL_BREAK_TYPE_" + old_type.to_upper())
				var end_desc: String = Locale.ltr("CHRONICLE_MENTAL_BREAK_END").format({
					"name": entity.entity_name,
					"break_type": end_type_name,
				})
				_chronicle_system.log_event("mental_break", entity.id, end_desc, 3, [], tick)
		return

	# C(Conscientiousness) ↑ → higher threshold (more self-control)
	var z_C: float = pd.to_zscore(pd.axes.get("C", 0.5))
	var threshold: float = 300.0 + 50.0 * z_C

	# Skip check if stress is well below threshold
	if ed.stress < threshold * 0.5:
		return

	# Sigmoid probability
	var p: float = 1.0 / (1.0 + exp(-(ed.stress - threshold) / _break_beta))

	if randf() < p * _break_tick_prob:
		var break_type: String = _determine_break_type(ed)
		if break_type != "":
			ed.mental_break_type = break_type
			var start_behavior = _break_behaviors.get(break_type, {})
			ed.mental_break_remaining = float(start_behavior.get("duration_hours", 1.0))
			# Override entity action
			entity.current_action = "mental_break"
			entity.current_goal = break_type
			entity.action_timer = 0
			emit_event("mental_break_started", {
				"entity_id": entity.id,
				"entity_name": entity.entity_name,
				"break_type": break_type,
				"stress": ed.stress,
				"threshold": threshold,
			})
			if _chronicle_system != null:
				var start_type_name: String = Locale.ltr("MENTAL_BREAK_TYPE_" + break_type.to_upper())
				var start_desc: String = Locale.ltr("CHRONICLE_MENTAL_BREAK_START").format({
					"name": entity.entity_name,
					"break_type": start_type_name,
				})
				_chronicle_system.log_event("mental_break", entity.id, start_desc, 4, [], tick)


## Determine break type based on dominant negative emotion
func _determine_break_type(ed: RefCounted) -> String:
	# Special check: Outrage (Surprise+Anger dyad) > 60 → outrage_violence
	var outrage: float = ed.get_dyad("outrage")
	if outrage > 60.0:
		return "outrage_violence"

	# Otherwise, dominant negative emotion determines type
	var candidates: Dictionary = {
		"panic": ed.get_emotion("fear"),
		"rage": ed.get_emotion("anger"),
		"shutdown": ed.get_emotion("sadness"),
		"purge": ed.get_emotion("disgust"),
	}

	var max_type: String = "shutdown"  # default fallback
	var max_val: float = 0.0
	for btype in candidates:
		if candidates[btype] > max_val:
			max_val = candidates[btype]
			max_type = btype
	return max_type


# ═══════════════════════════════════════════════════
# Memory Trace Creation
# ═══════════════════════════════════════════════════

func _create_memory_trace(ed: RefCounted, impulse: Dictionary, event: Dictionary) -> void:
	var source: String = str(event.get("description", "unknown"))
	var is_trauma: bool = event.get("is_trauma", false)
	var base_decay: float = 0.693147 / (_memory_trace_trauma_hl_hours if is_trauma else _memory_trace_default_hl_hours)

	for emo in impulse:
		if impulse[emo] > _memory_trace_threshold:
			var traces: Array = ed.memory_traces.get(emo, [])
			# Cap at 20 traces per emotion to prevent unbounded growth
			if traces.size() >= 20:
				# Remove weakest trace
				var min_idx: int = 0
				var min_val: float = traces[0].intensity
				for k in range(1, traces.size()):
					if traces[k].intensity < min_val:
						min_val = traces[k].intensity
						min_idx = k
				traces.remove_at(min_idx)
			traces.append({
				"source": source,
				"intensity": impulse[emo] * _memory_trace_ratio,
				"decay_rate": base_decay,
			})
			ed.memory_traces[emo] = traces


# ═══════════════════════════════════════════════════
# Box-Muller Transform (randfn replacement)
# ═══════════════════════════════════════════════════

var _spare_normal: float = 0.0
var _has_spare: bool = false


func _randfn() -> float:
	if _has_spare:
		_has_spare = false
		return _spare_normal
	var u: float = randf()
	var v: float = randf()
	# Prevent log(0)
	while u < 1e-10:
		u = randf()
	var mag: float = sqrt(-2.0 * log(u))
	_spare_normal = mag * sin(TAU * v)
	_has_spare = true
	return mag * cos(TAU * v)
